// 音频系统模块 - 高质量3D空间音频引擎
// 开发心理：音频是游戏沉浸感的重要组成部分，需要支持空间音效、动态混音
// 设计原则：低延迟、高质量、内存高效、支持多种音频格式

pub mod manager;
pub mod sound;
pub mod music;

// 重新导出主要类型
pub use manager::{AudioManager, AudioDevice, DeviceStats, DeliveryMethod};
pub use sound::{SoundBuffer, SoundInstance, SampleFormat, ChannelLayout, SoundEffect, SoundEffectType};
pub use music::{MusicTrack, MusicCategory, MoodTag, GameContext, PlaylistManager};

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceManager, ResourceHandle};
use crate::core::event_system::{Event, EventSystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use log::{info, debug, warn, error};

// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSystemConfig {
    pub enable_audio: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub ambient_volume: f32,
    
    // 设备配置
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: AudioChannels,
    pub bit_depth: AudioBitDepth,
    
    // 3D音频配置
    pub enable_3d_audio: bool,
    pub doppler_factor: f32,
    pub speed_of_sound: f32,
    pub max_distance: f32,
    pub rolloff_factor: f32,
    
    // 性能配置
    pub max_simultaneous_sounds: u32,
    pub audio_thread_priority: ThreadPriority,
    pub enable_audio_streaming: bool,
    pub streaming_buffer_size: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioChannels {
    Mono = 1,
    Stereo = 2,
    Surround5_1 = 6,
    Surround7_1 = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBitDepth {
    Bit16 = 16,
    Bit24 = 24,
    Bit32 = 32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadPriority {
    Low,
    Normal,
    High,
    RealTime,
}

impl Default for AudioSystemConfig {
    fn default() -> Self {
        Self {
            enable_audio: true,
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            voice_volume: 1.0,
            ambient_volume: 0.5,
            
            sample_rate: 44100,
            buffer_size: 1024,
            channels: AudioChannels::Stereo,
            bit_depth: AudioBitDepth::Bit16,
            
            enable_3d_audio: true,
            doppler_factor: 1.0,
            speed_of_sound: 343.3,
            max_distance: 1000.0,
            rolloff_factor: 1.0,
            
            max_simultaneous_sounds: 32,
            audio_thread_priority: ThreadPriority::High,
            enable_audio_streaming: true,
            streaming_buffer_size: 4096,
        }
    }
}

// 音频格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    MP3,
    OGG,
    FLAC,
    M4A,
}

// 音频类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCategory {
    Music,           // 背景音乐
    SFX,            // 音效
    Voice,          // 语音
    Ambient,        // 环境音
    UI,             // 界面音效
    Pokemon,        // 宝可梦叫声
    Battle,         // 战斗音效
}

// 音频状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioState {
    Stopped,
    Playing,
    Paused,
    Fading,
}

// 3D音频位置信息
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioTransform {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub orientation: [f32; 3],
}

impl Default for AudioTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, -1.0],
        }
    }
}

// 音频监听器（玩家）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioListener {
    pub transform: AudioTransform,
    pub master_gain: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            transform: AudioTransform::default(),
            master_gain: 1.0,
        }
    }
}

// 音频事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlayEvent {
    pub sound_id: String,
    pub category: AudioCategory,
    pub volume: f32,
    pub transform: Option<AudioTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStopEvent {
    pub sound_id: String,
    pub fade_out_duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVolumeChangeEvent {
    pub category: AudioCategory,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicChangeEvent {
    pub track_name: String,
    pub fade_duration: Duration,
    pub loop_music: bool,
}

// 实现Event特征
impl Event for AudioPlayEvent {
    fn event_type(&self) -> &'static str { "AudioPlay" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for AudioStopEvent {
    fn event_type(&self) -> &'static str { "AudioStop" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for AudioVolumeChangeEvent {
    fn event_type(&self) -> &'static str { "AudioVolumeChange" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for MusicChangeEvent {
    fn event_type(&self) -> &'static str { "MusicChange" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// 音频实例信息
#[derive(Debug, Clone)]
pub struct AudioInstance {
    pub id: u64,
    pub sound_id: String,
    pub category: AudioCategory,
    pub state: AudioState,
    pub volume: f32,
    pub pitch: f32,
    pub transform: Option<AudioTransform>,
    pub is_looping: bool,
    pub start_time: std::time::Instant,
    pub duration: Option<Duration>,
}

// 音频统计
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AudioStats {
    pub active_sounds: u32,
    pub total_sounds_played: u64,
    pub audio_memory_usage: u64,
    pub cpu_usage_percent: f64,
    pub buffer_underruns: u32,
    pub sample_rate: u32,
    pub latency_ms: f64,
    pub channels_used: u32,
}

// 音频系统
pub struct AudioSystem {
    config: AudioSystemConfig,
    stats: AudioStats,
    
    // 组件
    manager: Option<AudioManager>,
    playlist_manager: PlaylistManager,
    
    // 实例管理
    active_instances: HashMap<u64, AudioInstance>,
    next_instance_id: u64,
    
    // 音频资源
    sound_buffers: HashMap<String, ResourceHandle<SoundBuffer>>,
    music_tracks: HashMap<String, ResourceHandle<MusicTrack>>,
    
    // 监听器
    listener: AudioListener,
    
    // 分类音量
    category_volumes: HashMap<AudioCategory, f32>,
    
    // 性能监控
    last_stats_update: std::time::Instant,
}

impl AudioSystem {
    pub fn new(config: AudioSystemConfig) -> Result<Self> {
        info!("初始化音频系统");
        
        if !config.enable_audio {
            info!("音频系统已禁用");
            return Ok(Self::new_disabled());
        }
        
        // 初始化音频管理器
        let manager = AudioManager::new(config.clone())?;
        
        // 初始化分类音量
        let mut category_volumes = HashMap::new();
        category_volumes.insert(AudioCategory::Music, config.music_volume);
        category_volumes.insert(AudioCategory::SFX, config.sfx_volume);
        category_volumes.insert(AudioCategory::Voice, config.voice_volume);
        category_volumes.insert(AudioCategory::Ambient, config.ambient_volume);
        category_volumes.insert(AudioCategory::UI, config.sfx_volume);
        category_volumes.insert(AudioCategory::Pokemon, config.sfx_volume);
        category_volumes.insert(AudioCategory::Battle, config.sfx_volume);
        
        Ok(Self {
            config,
            stats: AudioStats::default(),
            
            manager: Some(manager),
            playlist_manager: PlaylistManager::new(),
            
            active_instances: HashMap::new(),
            next_instance_id: 1,
            
            sound_buffers: HashMap::new(),
            music_tracks: HashMap::new(),
            
            listener: AudioListener::default(),
            category_volumes,
            
            last_stats_update: std::time::Instant::now(),
        })
    }
    
    fn new_disabled() -> Self {
        Self {
            config: AudioSystemConfig {
                enable_audio: false,
                ..AudioSystemConfig::default()
            },
            stats: AudioStats::default(),
            
            manager: None,
            playlist_manager: PlaylistManager::new(),
            
            active_instances: HashMap::new(),
            next_instance_id: 1,
            
            sound_buffers: HashMap::new(),
            music_tracks: HashMap::new(),
            
            listener: AudioListener::default(),
            category_volumes: HashMap::new(),
            
            last_stats_update: std::time::Instant::now(),
        }
    }
    
    // 加载音频资源
    pub fn load_sound(&mut self, id: &str, path: &Path, category: AudioCategory) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        debug!("加载音频: {} -> {:?}", id, path);
        
        let handle = ResourceManager::instance().load::<SoundBuffer>(
            &format!("sound_{}", id),
            path,
            crate::core::resource_manager::ResourceType::Audio,
            crate::core::resource_manager::ResourcePriority::Normal,
        )?;
        
        self.sound_buffers.insert(id.to_string(), handle);
        Ok(())
    }
    
    // 加载音乐
    pub fn load_music(&mut self, id: &str, path: &Path) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        debug!("加载音乐: {} -> {:?}", id, path);
        
        let handle = ResourceManager::instance().load::<MusicTrack>(
            &format!("music_{}", id),
            path,
            crate::core::resource_manager::ResourceType::Audio,
            crate::core::resource_manager::ResourcePriority::High,
        )?;
        
        self.music_tracks.insert(id.to_string(), handle);
        Ok(())
    }
    
    // 播放音效
    pub fn play_sound(
        &mut self,
        sound_id: &str,
        category: AudioCategory,
        volume: f32,
        pitch: f32,
        transform: Option<AudioTransform>,
    ) -> Result<u64> {
        if !self.config.enable_audio {
            return Ok(0);
        }
        
        // 检查音频资源
        if !self.sound_buffers.contains_key(sound_id) {
            return Err(GameError::AudioError(format!("音频资源不存在: {}", sound_id)));
        }
        
        // 检查同时播放数量限制
        if self.active_instances.len() >= self.config.max_simultaneous_sounds as usize {
            self.cleanup_finished_sounds();
            
            if self.active_instances.len() >= self.config.max_simultaneous_sounds as usize {
                warn!("达到最大同时播放数量限制");
                return Err(GameError::AudioError("音频播放数量已达上限".to_string()));
            }
        }
        
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        
        // 计算最终音量
        let category_volume = self.category_volumes.get(&category).copied().unwrap_or(1.0);
        let final_volume = volume * category_volume * self.config.master_volume;
        
        // 创建音频实例
        let instance = AudioInstance {
            id: instance_id,
            sound_id: sound_id.to_string(),
            category,
            state: AudioState::Playing,
            volume: final_volume,
            pitch,
            transform,
            is_looping: false,
            start_time: std::time::Instant::now(),
            duration: None, // TODO: 从音频数据获取
        };
        
        // 播放音频
        if let Some(ref mut manager) = self.manager {
            manager.play_sound(&instance)?;
        }
        
        self.active_instances.insert(instance_id, instance);
        self.stats.active_sounds += 1;
        self.stats.total_sounds_played += 1;
        
        debug!("播放音效: {} (实例ID: {})", sound_id, instance_id);
        Ok(instance_id)
    }
    
    // 播放循环音效
    pub fn play_looped_sound(
        &mut self,
        sound_id: &str,
        category: AudioCategory,
        volume: f32,
        transform: Option<AudioTransform>,
    ) -> Result<u64> {
        let instance_id = self.play_sound(sound_id, category, volume, 1.0, transform)?;
        
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            instance.is_looping = true;
        }
        
        Ok(instance_id)
    }
    
    // 停止音效
    pub fn stop_sound(&mut self, instance_id: u64, fade_out: Option<Duration>) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            if let Some(fade_duration) = fade_out {
                instance.state = AudioState::Fading;
                if let Some(ref mut manager) = self.manager {
                    manager.fade_out_sound(instance_id, fade_duration)?;
                }
            } else {
                instance.state = AudioState::Stopped;
                if let Some(ref mut manager) = self.manager {
                    manager.stop_sound(instance_id)?;
                }
                self.active_instances.remove(&instance_id);
                self.stats.active_sounds -= 1;
            }
            
            debug!("停止音效实例: {}", instance_id);
        }
        
        Ok(())
    }
    
    // 播放音乐
    pub fn play_music(&mut self, track_id: &str, loop_music: bool, fade_in: Option<Duration>) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        if !self.music_tracks.contains_key(track_id) {
            return Err(GameError::AudioError(format!("音乐资源不存在: {}", track_id)));
        }
        
        let volume = self.category_volumes.get(&AudioCategory::Music).copied().unwrap_or(1.0) * self.config.master_volume;
        
        if let Some(ref mut manager) = self.manager {
            manager.play_music(track_id, volume, loop_music, fade_in)?;
        }
        
        info!("播放音乐: {} (循环: {})", track_id, loop_music);
        Ok(())
    }
    
    // 停止音乐
    pub fn stop_music(&mut self, fade_out: Option<Duration>) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        if let Some(ref mut manager) = self.manager {
            manager.stop_music(fade_out)?;
        }
        
        debug!("停止音乐");
        Ok(())
    }
    
    // 暂停/恢复音乐
    pub fn pause_music(&mut self) -> Result<()> {
        if let Some(ref mut manager) = self.manager {
            manager.pause_music()?;
        }
        Ok(())
    }
    
    pub fn resume_music(&mut self) -> Result<()> {
        if let Some(ref mut manager) = self.manager {
            manager.resume_music()?;
        }
        Ok(())
    }
    
    // 设置分类音量
    pub fn set_category_volume(&mut self, category: AudioCategory, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.category_volumes.insert(category, clamped_volume);
        
        // 更新所有该分类的活跃音频
        for instance in self.active_instances.values_mut() {
            if instance.category == category {
                let new_volume = instance.volume * clamped_volume * self.config.master_volume;
                if let Some(ref mut manager) = self.manager {
                    manager.set_sound_volume(instance.id, new_volume)?;
                }
            }
        }
        
        // 发送事件
        EventSystem::dispatch(AudioVolumeChangeEvent {
            category,
            volume: clamped_volume,
        })?;
        
        debug!("设置分类音量: {:?} = {}", category, clamped_volume);
        Ok(())
    }
    
    // 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) -> Result<()> {
        self.config.master_volume = volume.clamp(0.0, 1.0);
        
        // 更新所有活跃音频
        for instance in self.active_instances.values() {
            let category_volume = self.category_volumes.get(&instance.category).copied().unwrap_or(1.0);
            let new_volume = instance.volume * category_volume * self.config.master_volume;
            if let Some(ref mut manager) = self.manager {
                manager.set_sound_volume(instance.id, new_volume)?;
            }
        }
        
        if let Some(ref mut manager) = self.manager {
            manager.set_master_volume(self.config.master_volume)?;
        }
        
        debug!("设置主音量: {}", self.config.master_volume);
        Ok(())
    }
    
    // 更新监听器位置
    pub fn set_listener_transform(&mut self, transform: AudioTransform) -> Result<()> {
        if !self.config.enable_3d_audio {
            return Ok(());
        }
        
        self.listener.transform = transform;
        
        if let Some(ref mut manager) = self.manager {
            manager.set_listener_transform(transform)?;
        }
        
        Ok(())
    }
    
    // 更新音频实例位置
    pub fn set_sound_transform(&mut self, instance_id: u64, transform: AudioTransform) -> Result<()> {
        if !self.config.enable_3d_audio {
            return Ok(());
        }
        
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            instance.transform = Some(transform);
            
            if let Some(ref mut manager) = self.manager {
                manager.set_sound_transform(instance_id, transform)?;
            }
        }
        
        Ok(())
    }
    
    // 更新音频系统
    pub fn update(&mut self, delta_time: Duration) -> Result<()> {
        if !self.config.enable_audio {
            return Ok(());
        }
        
        // 清理已完成的音效
        self.cleanup_finished_sounds();
        
        // 更新音频管理器
        if let Some(ref mut manager) = self.manager {
            manager.update(delta_time)?;
        }
        
        // 更新统计信息
        self.update_stats();
        
        // 更新播放列表管理器
        self.playlist_manager.update(delta_time);
        
        Ok(())
    }
    
    // 清理已完成的音效
    fn cleanup_finished_sounds(&mut self) {
        let mut to_remove = Vec::new();
        
        for (&instance_id, instance) in &self.active_instances {
            // 检查非循环音效是否已完成
            if !instance.is_looping && instance.state == AudioState::Stopped {
                to_remove.push(instance_id);
            }
            
            // 检查是否已超时（用于检测异常情况）
            if let Some(duration) = instance.duration {
                if instance.start_time.elapsed() > duration + Duration::from_secs(1) {
                    to_remove.push(instance_id);
                }
            }
        }
        
        for instance_id in to_remove {
            self.active_instances.remove(&instance_id);
            if self.stats.active_sounds > 0 {
                self.stats.active_sounds -= 1;
            }
        }
    }
    
    // 更新统计信息
    fn update_stats(&mut self) {
        let now = std::time::Instant::now();
        
        if now.duration_since(self.last_stats_update).as_secs() >= 1 {
            self.stats.active_sounds = self.active_instances.len() as u32;
            
            if let Some(ref manager) = self.manager {
                // 从音频管理器获取详细统计
                if let Ok(device_stats) = manager.get_device_stats() {
                    self.stats.cpu_usage_percent = device_stats.cpu_usage;
                    self.stats.latency_ms = device_stats.latency_ms;
                    self.stats.buffer_underruns += device_stats.buffer_underruns;
                }
            }
            
            self.last_stats_update = now;
        }
    }
    
    // 预加载音频包
    pub fn preload_audio_pack(&mut self, pack_name: &str) -> Result<()> {
        // TODO: 实现音频包预加载
        info!("预加载音频包: {}", pack_name);
        Ok(())
    }
    
    // 卸载音频包
    pub fn unload_audio_pack(&mut self, pack_name: &str) -> Result<()> {
        // TODO: 实现音频包卸载
        info!("卸载音频包: {}", pack_name);
        Ok(())
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> &AudioStats {
        &self.stats
    }
    
    // 获取配置
    pub fn get_config(&self) -> &AudioSystemConfig {
        &self.config
    }
    
    // 获取活跃音频实例
    pub fn get_active_instances(&self) -> &HashMap<u64, AudioInstance> {
        &self.active_instances
    }
    
    // 是否可用
    pub fn is_available(&self) -> bool {
        self.config.enable_audio && self.manager.is_some()
    }
    
    // 关闭音频系统
    pub fn shutdown(&mut self) {
        info!("关闭音频系统");
        
        // 停止所有音频
        for &instance_id in self.active_instances.keys().collect::<Vec<_>>() {
            let _ = self.stop_sound(instance_id, None);
        }
        
        let _ = self.stop_music(None);
        
        // 关闭音频管理器
        if let Some(mut manager) = self.manager.take() {
            manager.shutdown();
        }
        
        self.active_instances.clear();
        self.sound_buffers.clear();
        self.music_tracks.clear();
        
        info!("音频系统已关闭");
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// 全局音频系统
static mut AUDIO_SYSTEM: Option<AudioSystem> = None;
static AUDIO_INIT: std::sync::Once = std::sync::Once::new();

pub struct Audio;

impl Audio {
    pub fn init(config: AudioSystemConfig) -> Result<()> {
        unsafe {
            AUDIO_INIT.call_once(|| {
                match AudioSystem::new(config) {
                    Ok(system) => {
                        AUDIO_SYSTEM = Some(system);
                    },
                    Err(e) => {
                        error!("音频系统初始化失败: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
    
    pub fn instance() -> Result<&'static mut AudioSystem> {
        unsafe {
            AUDIO_SYSTEM.as_mut()
                .ok_or_else(|| GameError::AudioError("音频系统未初始化".to_string()))
        }
    }
    
    pub fn cleanup() {
        unsafe {
            if let Some(ref mut system) = AUDIO_SYSTEM {
                system.shutdown();
            }
            AUDIO_SYSTEM = None;
        }
    }
}

// 音频工具函数
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.log10()
}

pub fn calculate_3d_volume(
    listener_pos: [f32; 3],
    source_pos: [f32; 3],
    max_distance: f32,
    rolloff_factor: f32,
) -> f32 {
    let distance = {
        let dx = source_pos[0] - listener_pos[0];
        let dy = source_pos[1] - listener_pos[1];
        let dz = source_pos[2] - listener_pos[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };
    
    if distance >= max_distance {
        0.0
    } else {
        1.0 - (distance / max_distance).powf(rolloff_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_config_default() {
        let config = AudioSystemConfig::default();
        assert!(config.enable_audio);
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, AudioChannels::Stereo);
    }
    
    #[test]
    fn test_db_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.5).abs() < 0.1);
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_3d_volume_calculation() {
        let listener = [0.0, 0.0, 0.0];
        let source_close = [1.0, 0.0, 0.0];
        let source_far = [100.0, 0.0, 0.0];
        
        let volume_close = calculate_3d_volume(listener, source_close, 50.0, 1.0);
        let volume_far = calculate_3d_volume(listener, source_far, 50.0, 1.0);
        
        assert!(volume_close > volume_far);
        assert_eq!(volume_far, 0.0);
    }
}