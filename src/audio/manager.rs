// 音频设备管理器 - 低级音频硬件抽象
// 开发心理：提供跨平台的音频设备管理，抽象不同操作系统的音频API
// 设计原则：设备枚举、音频流管理、延迟优化、错误恢复

use crate::core::{GameError, Result};
use crate::audio::{AudioSystemConfig, AudioChannels, AudioBitDepth, ThreadPriority};
use serde::{Deserialize, Serialize};

// 音频传输方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMethod {
    Reliable,    // 可靠传输
    Unreliable,  // 不可靠传输（适用于实时音频）
    Streaming,   // 流式传输
}
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};

// 音频设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub max_channels: u32,
    pub supported_sample_rates: Vec<u32>,
    pub supported_bit_depths: Vec<AudioBitDepth>,
    pub latency_ms: f64,
    pub is_capture: bool,
}

// 设备统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub latency_ms: f64,
    pub buffer_underruns: u32,
    pub buffer_overruns: u32,
    pub dropped_frames: u64,
    pub sample_rate: u32,
    pub channels: u32,
}

impl Default for DeviceStats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            latency_ms: 0.0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            dropped_frames: 0,
            sample_rate: 44100,
            channels: 2,
        }
    }
}

// 音频缓冲区
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub data: Vec<f32>,
    pub channels: u32,
    pub sample_rate: u32,
    pub frame_count: u32,
    pub timestamp: Instant,
}

impl AudioBuffer {
    pub fn new(channels: u32, sample_rate: u32, frame_count: u32) -> Self {
        let data_size = (channels * frame_count) as usize;
        Self {
            data: vec![0.0; data_size],
            channels,
            sample_rate,
            frame_count,
            timestamp: Instant::now(),
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }

    pub fn mix_in(&mut self, other: &AudioBuffer, volume: f32) {
        if self.channels != other.channels || self.frame_count != other.frame_count {
            warn!("音频缓冲区格式不匹配，跳过混音");
            return;
        }

        for (i, &sample) in other.data.iter().enumerate() {
            if i < self.data.len() {
                self.data[i] += sample * volume;
            }
        }
    }

    pub fn apply_volume(&mut self, volume: f32) {
        for sample in &mut self.data {
            *sample *= volume;
        }
    }

    pub fn get_rms(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }

        let sum_of_squares: f32 = self.data.iter().map(|&x| x * x).sum();
        (sum_of_squares / self.data.len() as f32).sqrt()
    }
}

// 音频流配置
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub channels: u32,
    pub sample_rate: u32,
    pub bit_depth: AudioBitDepth,
    pub buffer_size: u32,
    pub enable_callback: bool,
}

// 音频管理器
pub struct AudioManager {
    config: AudioSystemConfig,
    current_device: Option<AudioDevice>,
    available_devices: Vec<AudioDevice>,
    stats: DeviceStats,
    
    // 音频线程和缓冲区
    audio_thread: Option<JoinHandle<()>>,
    is_running: Arc<RwLock<bool>>,
    
    // 播放队列
    playback_queue: Arc<Mutex<VecDeque<QueuedAudio>>>,
    
    // 混音缓冲区
    mix_buffer: AudioBuffer,
    output_buffer: AudioBuffer,
    
    // 性能监控
    last_stats_update: Instant,
    callback_times: VecDeque<f64>,
    
    // 活跃音频实例
    active_sounds: HashMap<u64, ActiveSound>,
    
    // 主音量控制
    master_volume: f32,
}

#[derive(Debug, Clone)]
struct QueuedAudio {
    instance_id: u64,
    buffer: AudioBuffer,
    volume: f32,
    delivery_method: DeliveryMethod,
    queued_at: Instant,
}

#[derive(Debug, Clone)]
struct ActiveSound {
    instance_id: u64,
    buffer: AudioBuffer,
    position: usize,
    volume: f32,
    is_looping: bool,
    is_playing: bool,
    fade_target: Option<f32>,
    fade_duration: Option<Duration>,
    fade_start: Option<Instant>,
}

impl AudioManager {
    pub fn new(config: AudioSystemConfig) -> Result<Self> {
        info!("初始化音频设备管理器");
        
        // 枚举可用音频设备
        let available_devices = Self::enumerate_devices()?;
        if available_devices.is_empty() {
            return Err(GameError::AudioError("未找到可用的音频设备".to_string()));
        }
        
        // 选择默认设备
        let current_device = available_devices
            .iter()
            .find(|d| d.is_default && !d.is_capture)
            .or_else(|| available_devices.iter().find(|d| !d.is_capture))
            .cloned();
        
        if current_device.is_none() {
            return Err(GameError::AudioError("未找到可用的输出设备".to_string()));
        }
        
        // 创建混音缓冲区
        let mix_buffer = AudioBuffer::new(
            config.channels as u32,
            config.sample_rate,
            config.buffer_size,
        );
        
        let output_buffer = AudioBuffer::new(
            config.channels as u32,
            config.sample_rate,
            config.buffer_size,
        );
        
        let mut manager = Self {
            config,
            current_device,
            available_devices,
            stats: DeviceStats::default(),
            
            audio_thread: None,
            is_running: Arc::new(RwLock::new(false)),
            
            playback_queue: Arc::new(Mutex::new(VecDeque::new())),
            
            mix_buffer,
            output_buffer,
            
            last_stats_update: Instant::now(),
            callback_times: VecDeque::with_capacity(60),
            
            active_sounds: HashMap::new(),
            master_volume: 1.0,
        };
        
        // 启动音频线程
        manager.start_audio_thread()?;
        
        Ok(manager)
    }
    
    // 枚举音频设备
    fn enumerate_devices() -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();
        
        // 在真实实现中，这里会调用平台特定的API
        // Windows: WASAPI, DirectSound
        // macOS: Core Audio
        // Linux: ALSA, PulseAudio, JACK
        
        // 添加默认设备（模拟）
        devices.push(AudioDevice {
            id: "default".to_string(),
            name: "默认音频设备".to_string(),
            is_default: true,
            max_channels: 2,
            supported_sample_rates: vec![22050, 44100, 48000, 96000],
            supported_bit_depths: vec![AudioBitDepth::Bit16, AudioBitDepth::Bit24, AudioBitDepth::Bit32],
            latency_ms: 10.0,
            is_capture: false,
        });
        
        debug!("发现 {} 个音频设备", devices.len());
        Ok(devices)
    }
    
    // 启动音频处理线程
    fn start_audio_thread(&mut self) -> Result<()> {
        if self.audio_thread.is_some() {
            return Ok(());
        }
        
        let is_running = self.is_running.clone();
        let playback_queue = self.playback_queue.clone();
        let config = self.config.clone();
        
        *is_running.write().unwrap() = true;
        
        let thread_handle = thread::Builder::new()
            .name("AudioManager".to_string())
            .spawn(move || {
                Self::audio_thread_main(is_running, playback_queue, config);
            })
            .map_err(|e| GameError::AudioError(format!("创建音频线程失败: {}", e)))?;
        
        self.audio_thread = Some(thread_handle);
        
        info!("音频处理线程已启动");
        Ok(())
    }
    
    // 音频线程主循环
    fn audio_thread_main(
        is_running: Arc<RwLock<bool>>,
        playback_queue: Arc<Mutex<VecDeque<QueuedAudio>>>,
        config: AudioSystemConfig,
    ) {
        let sample_duration = Duration::from_secs_f64(1.0 / config.sample_rate as f64);
        let buffer_duration = Duration::from_secs_f64(
            config.buffer_size as f64 / config.sample_rate as f64
        );
        
        let mut mix_buffer = AudioBuffer::new(
            config.channels as u32,
            config.sample_rate,
            config.buffer_size,
        );
        
        while *is_running.read().unwrap() {
            let start_time = Instant::now();
            
            // 清空混音缓冲区
            mix_buffer.clear();
            
            // 处理播放队列
            if let Ok(mut queue) = playback_queue.lock() {
                let mut processed = Vec::new();
                
                while let Some(queued) = queue.pop_front() {
                    // 将音频数据混入缓冲区
                    mix_buffer.mix_in(&queued.buffer, queued.volume);
                    processed.push(queued.instance_id);
                }
                
                if !processed.is_empty() {
                    debug!("处理了 {} 个音频样本", processed.len());
                }
            }
            
            // 模拟音频输出延迟
            let processing_time = start_time.elapsed();
            if processing_time < buffer_duration {
                thread::sleep(buffer_duration - processing_time);
            }
        }
        
        debug!("音频线程已退出");
    }
    
    // 播放音频实例
    pub fn play_sound(&mut self, instance: &crate::audio::AudioInstance) -> Result<()> {
        // 创建音频缓冲区（这里应该从实际的音频数据加载）
        let buffer = AudioBuffer::new(
            self.config.channels as u32,
            self.config.sample_rate,
            self.config.buffer_size,
        );
        
        let active_sound = ActiveSound {
            instance_id: instance.id,
            buffer: buffer.clone(),
            position: 0,
            volume: instance.volume,
            is_looping: instance.is_looping,
            is_playing: true,
            fade_target: None,
            fade_duration: None,
            fade_start: None,
        };
        
        self.active_sounds.insert(instance.id, active_sound);
        
        // 添加到播放队列
        let queued = QueuedAudio {
            instance_id: instance.id,
            buffer,
            volume: instance.volume * self.master_volume,
            delivery_method: DeliveryMethod::Reliable,
            queued_at: Instant::now(),
        };
        
        if let Ok(mut queue) = self.playback_queue.lock() {
            queue.push_back(queued);
        }
        
        debug!("开始播放音频实例: {}", instance.id);
        Ok(())
    }
    
    // 停止音频实例
    pub fn stop_sound(&mut self, instance_id: u64) -> Result<()> {
        if let Some(sound) = self.active_sounds.get_mut(&instance_id) {
            sound.is_playing = false;
        }
        
        self.active_sounds.remove(&instance_id);
        debug!("停止音频实例: {}", instance_id);
        Ok(())
    }
    
    // 淡出音频
    pub fn fade_out_sound(&mut self, instance_id: u64, duration: Duration) -> Result<()> {
        if let Some(sound) = self.active_sounds.get_mut(&instance_id) {
            sound.fade_target = Some(0.0);
            sound.fade_duration = Some(duration);
            sound.fade_start = Some(Instant::now());
        }
        
        debug!("开始淡出音频实例: {} (时长: {:?})", instance_id, duration);
        Ok(())
    }
    
    // 设置音频实例音量
    pub fn set_sound_volume(&mut self, instance_id: u64, volume: f32) -> Result<()> {
        if let Some(sound) = self.active_sounds.get_mut(&instance_id) {
            sound.volume = volume;
        }
        
        Ok(())
    }
    
    // 播放音乐
    pub fn play_music(
        &mut self,
        track_id: &str,
        volume: f32,
        loop_music: bool,
        fade_in: Option<Duration>,
    ) -> Result<()> {
        // TODO: 实现音乐播放逻辑
        info!("播放音乐: {} (音量: {}, 循环: {})", track_id, volume, loop_music);
        Ok(())
    }
    
    // 停止音乐
    pub fn stop_music(&mut self, fade_out: Option<Duration>) -> Result<()> {
        // TODO: 实现音乐停止逻辑
        info!("停止音乐 (淡出: {:?})", fade_out);
        Ok(())
    }
    
    // 暂停音乐
    pub fn pause_music(&mut self) -> Result<()> {
        info!("暂停音乐");
        Ok(())
    }
    
    // 恢复音乐
    pub fn resume_music(&mut self) -> Result<()> {
        info!("恢复音乐");
        Ok(())
    }
    
    // 设置3D音频位置
    pub fn set_listener_transform(&mut self, transform: crate::audio::AudioTransform) -> Result<()> {
        // TODO: 实现3D音频监听器位置设置
        debug!("设置监听器位置: {:?}", transform.position);
        Ok(())
    }
    
    // 设置音频实例3D位置
    pub fn set_sound_transform(&mut self, instance_id: u64, transform: crate::audio::AudioTransform) -> Result<()> {
        // TODO: 实现3D音频源位置设置
        debug!("设置音频实例 {} 位置: {:?}", instance_id, transform.position);
        Ok(())
    }
    
    // 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) -> Result<()> {
        self.master_volume = volume.clamp(0.0, 1.0);
        
        // 更新所有活跃音频的音量
        for sound in self.active_sounds.values_mut() {
            sound.volume *= self.master_volume;
        }
        
        debug!("设置主音量: {}", self.master_volume);
        Ok(())
    }
    
    // 更新音频管理器状态
    pub fn update(&mut self, delta_time: Duration) -> Result<()> {
        // 更新淡入淡出效果
        self.update_fading_sounds();
        
        // 移除已停止的音频
        self.cleanup_stopped_sounds();
        
        // 更新性能统计
        self.update_performance_stats(delta_time);
        
        Ok(())
    }
    
    // 更新淡入淡出音频
    fn update_fading_sounds(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        for (&instance_id, sound) in &mut self.active_sounds {
            if let (Some(fade_target), Some(fade_duration), Some(fade_start)) = 
                (sound.fade_target, sound.fade_duration, sound.fade_start) {
                
                let elapsed = now.duration_since(fade_start);
                let progress = (elapsed.as_secs_f32() / fade_duration.as_secs_f32()).min(1.0);
                
                if progress >= 1.0 {
                    sound.volume = fade_target;
                    sound.fade_target = None;
                    sound.fade_duration = None;
                    sound.fade_start = None;
                    
                    if fade_target == 0.0 {
                        to_remove.push(instance_id);
                    }
                } else {
                    // 线性插值计算当前音量
                    let original_volume = sound.volume;
                    sound.volume = original_volume + (fade_target - original_volume) * progress;
                }
            }
        }
        
        // 移除淡出完成的音频
        for instance_id in to_remove {
            self.active_sounds.remove(&instance_id);
            debug!("淡出完成，移除音频实例: {}", instance_id);
        }
    }
    
    // 清理已停止的音频
    fn cleanup_stopped_sounds(&mut self) {
        let mut to_remove = Vec::new();
        
        for (&instance_id, sound) in &self.active_sounds {
            if !sound.is_playing {
                to_remove.push(instance_id);
            }
        }
        
        for instance_id in to_remove {
            self.active_sounds.remove(&instance_id);
        }
    }
    
    // 更新性能统计
    fn update_performance_stats(&mut self, delta_time: Duration) {
        let now = Instant::now();
        
        // 记录回调时间
        self.callback_times.push_back(delta_time.as_secs_f64() * 1000.0);
        if self.callback_times.len() > 60 {
            self.callback_times.pop_front();
        }
        
        // 每秒更新一次统计信息
        if now.duration_since(self.last_stats_update).as_secs() >= 1 {
            // 计算平均延迟
            if !self.callback_times.is_empty() {
                self.stats.latency_ms = self.callback_times.iter().sum::<f64>() / self.callback_times.len() as f64;
            }
            
            // 估算CPU使用率（简化版）
            self.stats.cpu_usage = (self.active_sounds.len() as f64 / 32.0 * 100.0).min(100.0);
            
            // 更新内存使用
            self.stats.memory_usage = (self.active_sounds.len() * std::mem::size_of::<ActiveSound>()) as u64;
            
            self.last_stats_update = now;
        }
    }
    
    // 获取设备统计信息
    pub fn get_device_stats(&self) -> Result<DeviceStats> {
        Ok(self.stats.clone())
    }
    
    // 获取可用设备列表
    pub fn get_available_devices(&self) -> &[AudioDevice] {
        &self.available_devices
    }
    
    // 切换音频设备
    pub fn switch_device(&mut self, device_id: &str) -> Result<()> {
        if let Some(device) = self.available_devices.iter().find(|d| d.id == device_id).cloned() {
            self.current_device = Some(device.clone());
            info!("切换到音频设备: {}", device.name);
            
            // 在真实实现中，这里需要重新初始化音频流
            Ok(())
        } else {
            Err(GameError::AudioError(format!("设备不存在: {}", device_id)))
        }
    }
    
    // 获取当前设备信息
    pub fn get_current_device(&self) -> Option<&AudioDevice> {
        self.current_device.as_ref()
    }
    
    // 测试音频延迟
    pub fn test_latency(&mut self) -> Result<f64> {
        let start = Instant::now();
        
        // 播放测试音调
        let test_buffer = AudioBuffer::new(2, 44100, 1024);
        let test_queued = QueuedAudio {
            instance_id: u64::MAX, // 特殊ID用于测试
            buffer: test_buffer,
            volume: 0.1, // 低音量避免干扰
            delivery_method: DeliveryMethod::Reliable,
            queued_at: start,
        };
        
        if let Ok(mut queue) = self.playback_queue.lock() {
            queue.push_back(test_queued);
        }
        
        // 简化的延迟测量（实际实现需要音频回环测试）
        let latency = self.config.buffer_size as f64 / self.config.sample_rate as f64 * 1000.0;
        
        debug!("测试音频延迟: {:.2}ms", latency);
        Ok(latency)
    }
    
    // 关闭音频管理器
    pub fn shutdown(&mut self) {
        info!("关闭音频设备管理器");
        
        // 停止音频线程
        if let Ok(mut running) = self.is_running.write() {
            *running = false;
        }
        
        if let Some(handle) = self.audio_thread.take() {
            if let Err(e) = handle.join() {
                warn!("音频线程关闭失败: {:?}", e);
            }
        }
        
        // 清理所有音频
        self.active_sounds.clear();
        
        if let Ok(mut queue) = self.playback_queue.lock() {
            queue.clear();
        }
        
        info!("音频设备管理器已关闭");
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// 音频格式转换工具
pub fn convert_sample_rate(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input_rate == output_rate {
        return input.to_vec();
    }
    
    let ratio = output_rate as f64 / input_rate as f64;
    let output_len = (input.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    
    // 简单的线性插值重采样
    for i in 0..output_len {
        let src_index = i as f64 / ratio;
        let src_index_floor = src_index.floor() as usize;
        let src_index_ceil = (src_index.ceil() as usize).min(input.len() - 1);
        
        if src_index_floor >= input.len() {
            output.push(0.0);
            continue;
        }
        
        let fraction = src_index - src_index_floor as f64;
        let sample = if src_index_floor == src_index_ceil {
            input[src_index_floor]
        } else {
            let sample1 = input[src_index_floor];
            let sample2 = input[src_index_ceil];
            sample1 + (sample2 - sample1) * fraction as f32
        };
        
        output.push(sample);
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_buffer_creation() {
        let buffer = AudioBuffer::new(2, 44100, 1024);
        assert_eq!(buffer.channels, 2);
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.frame_count, 1024);
        assert_eq!(buffer.data.len(), 2048); // 2 channels * 1024 frames
    }
    
    #[test]
    fn test_audio_buffer_mixing() {
        let mut buffer1 = AudioBuffer::new(2, 44100, 1024);
        let mut buffer2 = AudioBuffer::new(2, 44100, 1024);
        
        buffer1.data[0] = 0.5;
        buffer2.data[0] = 0.3;
        
        buffer1.mix_in(&buffer2, 1.0);
        assert!((buffer1.data[0] - 0.8).abs() < 0.001);
    }
    
    #[test]
    fn test_sample_rate_conversion() {
        let input = vec![0.0, 1.0, 0.0, -1.0];
        let output = convert_sample_rate(&input, 44100, 48000);
        
        // 输出应该比输入稍长
        assert!(output.len() > input.len());
    }
}