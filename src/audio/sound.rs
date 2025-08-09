// 音频声音处理模块 - 音效和音频缓冲管理
// 开发心理：专注于单个音频对象的生命周期管理，支持各种音频格式和效果处理
// 设计原则：内存高效、格式支持广泛、实时效果处理、空间音频

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceManager, ResourceHandle, ResourceType};
use crate::audio::{AudioFormat, AudioCategory, AudioTransform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};

// 音频数据格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    I16,     // 16-bit signed integer
    I32,     // 32-bit signed integer  
    F32,     // 32-bit float
}

// 音频声道配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Surround3_1,
    Surround5_1,
    Surround7_1,
}

impl ChannelLayout {
    pub fn channel_count(&self) -> u32 {
        match self {
            ChannelLayout::Mono => 1,
            ChannelLayout::Stereo => 2,
            ChannelLayout::Surround3_1 => 4,
            ChannelLayout::Surround5_1 => 6,
            ChannelLayout::Surround7_1 => 8,
        }
    }
}

// 音频缓冲区数据
#[derive(Debug, Clone)]
pub struct SoundBuffer {
    pub id: String,
    pub format: AudioFormat,
    pub sample_format: SampleFormat,
    pub sample_rate: u32,
    pub channels: ChannelLayout,
    pub data: Vec<u8>,
    pub duration: Duration,
    pub loop_start: Option<u32>,  // 循环起始采样点
    pub loop_end: Option<u32>,    // 循环结束采样点
    pub metadata: SoundMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub genre: Option<String>,
    pub bitrate: Option<u32>,
    pub tags: HashMap<String, String>,
}

impl Default for SoundMetadata {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            genre: None,
            bitrate: None,
            tags: HashMap::new(),
        }
    }
}

impl SoundBuffer {
    pub fn new(
        id: String,
        format: AudioFormat,
        sample_format: SampleFormat,
        sample_rate: u32,
        channels: ChannelLayout,
        data: Vec<u8>,
    ) -> Self {
        let sample_count = Self::calculate_sample_count(&data, sample_format, channels);
        let duration = Duration::from_secs_f64(sample_count as f64 / sample_rate as f64);
        
        Self {
            id,
            format,
            sample_format,
            sample_rate,
            channels,
            data,
            duration,
            loop_start: None,
            loop_end: None,
            metadata: SoundMetadata::default(),
        }
    }
    
    // 从文件加载音频缓冲区
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| GameError::AudioError("无效的文件扩展名".to_string()))?;
        
        let format = match extension.to_lowercase().as_str() {
            "wav" => AudioFormat::WAV,
            "mp3" => AudioFormat::MP3,
            "ogg" => AudioFormat::OGG,
            "flac" => AudioFormat::FLAC,
            "m4a" => AudioFormat::M4A,
            _ => return Err(GameError::AudioError(format!("不支持的音频格式: {}", extension))),
        };
        
        // 在真实实现中，这里会调用相应的解码器
        let (sample_rate, channels, sample_format, audio_data) = match format {
            AudioFormat::WAV => Self::decode_wav(path)?,
            AudioFormat::MP3 => Self::decode_mp3(path)?,
            AudioFormat::OGG => Self::decode_ogg(path)?,
            AudioFormat::FLAC => Self::decode_flac(path)?,
            AudioFormat::M4A => Self::decode_m4a(path)?,
        };
        
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(Self::new(
            id,
            format,
            sample_format,
            sample_rate,
            channels,
            audio_data,
        ))
    }
    
    // 计算采样点数量
    fn calculate_sample_count(data: &[u8], format: SampleFormat, channels: ChannelLayout) -> u32 {
        let bytes_per_sample = match format {
            SampleFormat::I16 => 2,
            SampleFormat::I32 | SampleFormat::F32 => 4,
        };
        
        let total_samples = data.len() / bytes_per_sample;
        total_samples as u32 / channels.channel_count()
    }
    
    // 获取指定位置的音频数据片段
    pub fn get_samples(&self, start_sample: u32, sample_count: u32) -> Vec<f32> {
        let bytes_per_sample = match self.sample_format {
            SampleFormat::I16 => 2,
            SampleFormat::I32 | SampleFormat::F32 => 4,
        };
        
        let channels = self.channels.channel_count() as usize;
        let start_byte = start_sample as usize * bytes_per_sample * channels;
        let byte_count = sample_count as usize * bytes_per_sample * channels;
        
        if start_byte >= self.data.len() {
            return Vec::new();
        }
        
        let end_byte = (start_byte + byte_count).min(self.data.len());
        let slice = &self.data[start_byte..end_byte];
        
        // 转换为f32格式
        match self.sample_format {
            SampleFormat::I16 => {
                slice
                    .chunks_exact(2)
                    .map(|chunk| {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        sample as f32 / i16::MAX as f32
                    })
                    .collect()
            },
            SampleFormat::I32 => {
                slice
                    .chunks_exact(4)
                    .map(|chunk| {
                        let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        sample as f32 / i32::MAX as f32
                    })
                    .collect()
            },
            SampleFormat::F32 => {
                slice
                    .chunks_exact(4)
                    .map(|chunk| {
                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                    })
                    .collect()
            },
        }
    }
    
    // 设置循环区间
    pub fn set_loop_points(&mut self, start_sample: u32, end_sample: u32) -> Result<()> {
        let total_samples = Self::calculate_sample_count(&self.data, self.sample_format, self.channels);
        
        if start_sample >= total_samples || end_sample > total_samples || start_sample >= end_sample {
            return Err(GameError::AudioError("无效的循环点".to_string()));
        }
        
        self.loop_start = Some(start_sample);
        self.loop_end = Some(end_sample);
        Ok(())
    }
    
    // 解码器实现（简化版本）
    fn decode_wav(path: &Path) -> Result<(u32, ChannelLayout, SampleFormat, Vec<u8>)> {
        // 简化的WAV文件解析
        let data = std::fs::read(path)
            .map_err(|e| GameError::AudioError(format!("读取WAV文件失败: {}", e)))?;
        
        if data.len() < 44 {
            return Err(GameError::AudioError("WAV文件头太短".to_string()));
        }
        
        // 检查WAV标识
        if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            return Err(GameError::AudioError("不是有效的WAV文件".to_string()));
        }
        
        // 解析基本信息（简化版）
        let channels = u16::from_le_bytes([data[22], data[23]]);
        let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);
        
        let channel_layout = match channels {
            1 => ChannelLayout::Mono,
            2 => ChannelLayout::Stereo,
            6 => ChannelLayout::Surround5_1,
            8 => ChannelLayout::Surround7_1,
            _ => return Err(GameError::AudioError(format!("不支持的声道数: {}", channels))),
        };
        
        let sample_format = match bits_per_sample {
            16 => SampleFormat::I16,
            32 => SampleFormat::I32,
            _ => return Err(GameError::AudioError(format!("不支持的位深度: {}", bits_per_sample))),
        };
        
        // 查找数据块（简化实现）
        let audio_data = if data.len() > 44 {
            data[44..].to_vec()
        } else {
            Vec::new()
        };
        
        Ok((sample_rate, channel_layout, sample_format, audio_data))
    }
    
    fn decode_mp3(_path: &Path) -> Result<(u32, ChannelLayout, SampleFormat, Vec<u8>)> {
        // TODO: 实现MP3解码（需要第三方库如minimp3）
        Err(GameError::AudioError("MP3解码器未实现".to_string()))
    }
    
    fn decode_ogg(_path: &Path) -> Result<(u32, ChannelLayout, SampleFormat, Vec<u8>)> {
        // TODO: 实现OGG Vorbis解码（需要第三方库如lewton）
        Err(GameError::AudioError("OGG解码器未实现".to_string()))
    }
    
    fn decode_flac(_path: &Path) -> Result<(u32, ChannelLayout, SampleFormat, Vec<u8>)> {
        // TODO: 实现FLAC解码（需要第三方库如claxon）
        Err(GameError::AudioError("FLAC解码器未实现".to_string()))
    }
    
    fn decode_m4a(_path: &Path) -> Result<(u32, ChannelLayout, SampleFormat, Vec<u8>)> {
        // TODO: 实现M4A/AAC解码（需要第三方库）
        Err(GameError::AudioError("M4A解码器未实现".to_string()))
    }
}

// 音频实例 - 正在播放的声音
#[derive(Debug, Clone)]
pub struct SoundInstance {
    pub id: u64,
    pub buffer: ResourceHandle<SoundBuffer>,
    pub category: AudioCategory,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,  // -1.0 (左) 到 1.0 (右)
    pub is_looping: bool,
    pub current_sample: u32,
    pub is_playing: bool,
    pub is_paused: bool,
    pub transform: Option<AudioTransform>,
    pub created_at: Instant,
    pub fade_state: Option<FadeState>,
    pub effects: Vec<SoundEffect>,
}

#[derive(Debug, Clone)]
pub struct FadeState {
    pub start_volume: f32,
    pub target_volume: f32,
    pub duration: Duration,
    pub start_time: Instant,
}

impl SoundInstance {
    pub fn new(
        id: u64,
        buffer: ResourceHandle<SoundBuffer>,
        category: AudioCategory,
        volume: f32,
        pitch: f32,
    ) -> Self {
        Self {
            id,
            buffer,
            category,
            volume: volume.clamp(0.0, 1.0),
            pitch: pitch.max(0.1),
            pan: 0.0,
            is_looping: false,
            current_sample: 0,
            is_playing: false,
            is_paused: false,
            transform: None,
            created_at: Instant::now(),
            fade_state: None,
            effects: Vec::new(),
        }
    }
    
    // 开始播放
    pub fn play(&mut self) -> Result<()> {
        self.is_playing = true;
        self.is_paused = false;
        debug!("开始播放音频实例: {}", self.id);
        Ok(())
    }
    
    // 暂停播放
    pub fn pause(&mut self) {
        self.is_paused = true;
        debug!("暂停音频实例: {}", self.id);
    }
    
    // 恢复播放
    pub fn resume(&mut self) {
        self.is_paused = false;
        debug!("恢复音频实例: {}", self.id);
    }
    
    // 停止播放
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
        self.current_sample = 0;
        debug!("停止音频实例: {}", self.id);
    }
    
    // 设置播放位置（以秒为单位）
    pub fn set_position(&mut self, seconds: f32) -> Result<()> {
        let buffer = ResourceManager::instance()
            .get_resource(&self.buffer)
            .ok_or_else(|| GameError::AudioError("音频缓冲区不可用".to_string()))?;
        
        let sample_rate = buffer.sample_rate;
        let target_sample = (seconds * sample_rate as f32) as u32;
        let total_samples = SoundBuffer::calculate_sample_count(
            &buffer.data, 
            buffer.sample_format, 
            buffer.channels
        );
        
        if target_sample >= total_samples {
            return Err(GameError::AudioError("播放位置超出范围".to_string()));
        }
        
        self.current_sample = target_sample;
        debug!("设置音频实例 {} 播放位置: {}s (样本: {})", self.id, seconds, target_sample);
        Ok(())
    }
    
    // 获取当前播放位置（以秒为单位）
    pub fn get_position(&self) -> Result<f32> {
        let buffer = ResourceManager::instance()
            .get_resource(&self.buffer)
            .ok_or_else(|| GameError::AudioError("音频缓冲区不可用".to_string()))?;
        
        let position_seconds = self.current_sample as f32 / buffer.sample_rate as f32;
        Ok(position_seconds)
    }
    
    // 获取总时长（以秒为单位）
    pub fn get_duration(&self) -> Result<f32> {
        let buffer = ResourceManager::instance()
            .get_resource(&self.buffer)
            .ok_or_else(|| GameError::AudioError("音频缓冲区不可用".to_string()))?;
        
        Ok(buffer.duration.as_secs_f32())
    }
    
    // 开始淡入淡出
    pub fn fade_to(&mut self, target_volume: f32, duration: Duration) {
        self.fade_state = Some(FadeState {
            start_volume: self.volume,
            target_volume: target_volume.clamp(0.0, 1.0),
            duration,
            start_time: Instant::now(),
        });
        
        debug!("开始淡化音频实例 {}: {} -> {} (时长: {:?})", 
               self.id, self.volume, target_volume, duration);
    }
    
    // 更新淡入淡出状态
    pub fn update_fade(&mut self) {
        if let Some(fade) = &self.fade_state {
            let elapsed = fade.start_time.elapsed();
            let progress = (elapsed.as_secs_f32() / fade.duration.as_secs_f32()).min(1.0);
            
            if progress >= 1.0 {
                self.volume = fade.target_volume;
                self.fade_state = None;
                
                // 如果淡出到0，停止播放
                if fade.target_volume == 0.0 {
                    self.stop();
                }
            } else {
                // 线性插值
                self.volume = fade.start_volume + (fade.target_volume - fade.start_volume) * progress;
            }
        }
    }
    
    // 添加音效
    pub fn add_effect(&mut self, effect: SoundEffect) {
        self.effects.push(effect);
        debug!("为音频实例 {} 添加效果: {:?}", self.id, effect.effect_type);
    }
    
    // 移除音效
    pub fn remove_effect(&mut self, effect_type: SoundEffectType) {
        self.effects.retain(|e| e.effect_type != effect_type);
        debug!("从音频实例 {} 移除效果: {:?}", self.id, effect_type);
    }
    
    // 获取下一帧音频数据
    pub fn get_next_samples(&mut self, sample_count: u32) -> Result<Vec<f32>> {
        let buffer = ResourceManager::instance()
            .get_resource(&self.buffer)
            .ok_or_else(|| GameError::AudioError("音频缓冲区不可用".to_string()))?;
        
        if !self.is_playing || self.is_paused {
            return Ok(vec![0.0; sample_count as usize * buffer.channels.channel_count() as usize]);
        }
        
        let total_samples = SoundBuffer::calculate_sample_count(
            &buffer.data, 
            buffer.sample_format, 
            buffer.channels
        );
        
        let mut samples = Vec::new();
        let mut remaining_samples = sample_count;
        let mut current_pos = self.current_sample;
        
        while remaining_samples > 0 {
            let available_samples = total_samples.saturating_sub(current_pos);
            
            if available_samples == 0 {
                if self.is_looping {
                    // 处理循环
                    current_pos = buffer.loop_start.unwrap_or(0);
                    continue;
                } else {
                    // 播放结束，填充静音
                    let silence_count = remaining_samples as usize * buffer.channels.channel_count() as usize;
                    samples.extend(vec![0.0; silence_count]);
                    self.stop();
                    break;
                }
            }
            
            let read_samples = remaining_samples.min(available_samples);
            let mut chunk = buffer.get_samples(current_pos, read_samples);
            
            // 应用音量和音调
            self.apply_volume_and_pitch(&mut chunk);
            
            // 应用音效
            self.apply_effects(&mut chunk);
            
            samples.extend(chunk);
            current_pos += read_samples;
            remaining_samples -= read_samples;
        }
        
        self.current_sample = current_pos;
        Ok(samples)
    }
    
    // 应用音量和音调
    fn apply_volume_and_pitch(&self, samples: &mut [f32]) {
        // 应用音量
        for sample in samples.iter_mut() {
            *sample *= self.volume;
        }
        
        // TODO: 实现音调调整（需要重采样）
        if (self.pitch - 1.0).abs() > 0.001 {
            // 简化实现：暂时跳过音调调整
        }
    }
    
    // 应用音效
    fn apply_effects(&self, samples: &mut [f32]) {
        for effect in &self.effects {
            effect.apply(samples);
        }
    }
}

// 音效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffectType {
    Reverb,
    Chorus,
    Delay,
    Distortion,
    LowPass,
    HighPass,
    Compressor,
}

// 音效参数
#[derive(Debug, Clone)]
pub struct SoundEffect {
    pub effect_type: SoundEffectType,
    pub parameters: HashMap<String, f32>,
    pub enabled: bool,
}

impl SoundEffect {
    pub fn new(effect_type: SoundEffectType) -> Self {
        Self {
            effect_type,
            parameters: HashMap::new(),
            enabled: true,
        }
    }
    
    // 应用音效到音频数据
    pub fn apply(&self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }
        
        match self.effect_type {
            SoundEffectType::LowPass => self.apply_lowpass(samples),
            SoundEffectType::HighPass => self.apply_highpass(samples),
            SoundEffectType::Compressor => self.apply_compressor(samples),
            // 其他效果的实现
            _ => {
                // TODO: 实现其他音效
            }
        }
    }
    
    // 低通滤波器（简化实现）
    fn apply_lowpass(&self, samples: &mut [f32]) {
        let cutoff = self.parameters.get("cutoff").copied().unwrap_or(0.5);
        let alpha = cutoff.clamp(0.0, 1.0);
        
        if samples.len() < 2 {
            return;
        }
        
        for i in 1..samples.len() {
            samples[i] = alpha * samples[i] + (1.0 - alpha) * samples[i - 1];
        }
    }
    
    // 高通滤波器（简化实现）
    fn apply_highpass(&self, samples: &mut [f32]) {
        let cutoff = self.parameters.get("cutoff").copied().unwrap_or(0.5);
        let alpha = cutoff.clamp(0.0, 1.0);
        
        if samples.len() < 2 {
            return;
        }
        
        let mut prev_input = samples[0];
        let mut prev_output = samples[0];
        
        for i in 1..samples.len() {
            let current_input = samples[i];
            samples[i] = alpha * (prev_output + current_input - prev_input);
            
            prev_input = current_input;
            prev_output = samples[i];
        }
    }
    
    // 压缩器（简化实现）
    fn apply_compressor(&self, samples: &mut [f32]) {
        let threshold = self.parameters.get("threshold").copied().unwrap_or(0.8);
        let ratio = self.parameters.get("ratio").copied().unwrap_or(4.0);
        
        for sample in samples.iter_mut() {
            let abs_sample = sample.abs();
            if abs_sample > threshold {
                let excess = abs_sample - threshold;
                let compressed_excess = excess / ratio;
                let new_amplitude = threshold + compressed_excess;
                *sample = sample.signum() * new_amplitude;
            }
        }
    }
}

// 声音预设
pub struct SoundPresets;

impl SoundPresets {
    // 创建音效预设
    pub fn create_reverb() -> SoundEffect {
        let mut effect = SoundEffect::new(SoundEffectType::Reverb);
        effect.parameters.insert("room_size".to_string(), 0.5);
        effect.parameters.insert("damping".to_string(), 0.3);
        effect.parameters.insert("wet_level".to_string(), 0.2);
        effect
    }
    
    pub fn create_chorus() -> SoundEffect {
        let mut effect = SoundEffect::new(SoundEffectType::Chorus);
        effect.parameters.insert("rate".to_string(), 1.5);
        effect.parameters.insert("depth".to_string(), 0.3);
        effect.parameters.insert("feedback".to_string(), 0.2);
        effect
    }
    
    pub fn create_lowpass(cutoff: f32) -> SoundEffect {
        let mut effect = SoundEffect::new(SoundEffectType::LowPass);
        effect.parameters.insert("cutoff".to_string(), cutoff);
        effect
    }
    
    pub fn create_compressor(threshold: f32, ratio: f32) -> SoundEffect {
        let mut effect = SoundEffect::new(SoundEffectType::Compressor);
        effect.parameters.insert("threshold".to_string(), threshold);
        effect.parameters.insert("ratio".to_string(), ratio);
        effect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_sound_buffer_creation() {
        let data = vec![0u8; 1024];
        let buffer = SoundBuffer::new(
            "test".to_string(),
            AudioFormat::WAV,
            SampleFormat::I16,
            44100,
            ChannelLayout::Stereo,
            data,
        );
        
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, ChannelLayout::Stereo);
        assert_eq!(buffer.sample_format, SampleFormat::I16);
    }
    
    #[test]
    fn test_sound_instance_creation() {
        let data = vec![0u8; 1024];
        let buffer = SoundBuffer::new(
            "test".to_string(),
            AudioFormat::WAV,
            SampleFormat::I16,
            44100,
            ChannelLayout::Stereo,
            data,
        );
        
        // 创建资源句柄（简化测试）
        let handle = ResourceManager::instance().create_handle("test_sound".to_string());
        let instance = SoundInstance::new(1, handle, AudioCategory::SFX, 0.8, 1.0);
        
        assert_eq!(instance.volume, 0.8);
        assert_eq!(instance.pitch, 1.0);
        assert!(!instance.is_playing);
    }
    
    #[test]
    fn test_channel_layout() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
        assert_eq!(ChannelLayout::Surround5_1.channel_count(), 6);
    }
    
    #[test]
    fn test_sound_effect_creation() {
        let effect = SoundPresets::create_lowpass(0.7);
        assert_eq!(effect.effect_type, SoundEffectType::LowPass);
        assert_eq!(effect.parameters.get("cutoff"), Some(&0.7));
        assert!(effect.enabled);
    }
}