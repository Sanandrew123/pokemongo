/*
 * 音频管理系统 - Audio Manager System
 * 
 * 开发心理过程：
 * 设计完整的音频系统，支持音乐、音效、语音等多种音频类型
 * 需要考虑3D音频定位、音量控制、音频混合和性能优化
 * 重点关注音频质量和低延迟播放
 */

use bevy::prelude::*;
use bevy::audio::*;
use std::collections::HashMap;
use std::path::Path;
use crate::core::error::{GameResult, GameError};
use crate::core::math::Vec3;
use crate::engine::EngineConfig;
use crate::ffi::{AudioEngine, CAudioBuffer, C3DAudioParams};

// 音频类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioType {
    Music,      // 背景音乐
    SoundEffect, // 音效
    Voice,      // 语音
    Ambient,    // 环境音
    UI,         // 界面音效
}

// 音频格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
    Flac,
}

// 音频状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioState {
    Stopped,
    Playing,
    Paused,
    Fading,
}

// 音频源信息
#[derive(Debug, Clone)]
pub struct AudioSource {
    pub id: u32,
    pub name: String,
    pub audio_type: AudioType,
    pub handle: Handle<AudioSource>,
    pub state: AudioState,
    pub volume: f32,
    pub pitch: f32,
    pub loop_enabled: bool,
    pub position: Option<Vec3>,
    pub max_distance: f32,
    pub rolloff_factor: f32,
    pub fade_duration: f32,
    pub fade_target_volume: f32,
    pub fade_timer: f32,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            audio_type: AudioType::SoundEffect,
            handle: Handle::default(),
            state: AudioState::Stopped,
            volume: 1.0,
            pitch: 1.0,
            loop_enabled: false,
            position: None,
            max_distance: 100.0,
            rolloff_factor: 1.0,
            fade_duration: 0.0,
            fade_target_volume: 0.0,
            fade_timer: 0.0,
        }
    }
}

// 音频监听器
#[derive(Debug, Clone)]
pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub velocity: Vec3,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            velocity: Vec3::ZERO,
        }
    }
}

// 音频配置
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sound_effect_volume: f32,
    pub voice_volume: f32,
    pub ambient_volume: f32,
    pub ui_volume: f32,
    pub enable_3d_audio: bool,
    pub max_audio_sources: usize,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub doppler_factor: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sound_effect_volume: 1.0,
            voice_volume: 1.0,
            ambient_volume: 0.6,
            ui_volume: 0.8,
            enable_3d_audio: true,
            max_audio_sources: 32,
            sample_rate: 44100,
            buffer_size: 1024,
            doppler_factor: 1.0,
        }
    }
}

// 音频事件
#[derive(Debug, Clone)]
pub enum AudioEvent {
    SourceStarted(u32),
    SourceStopped(u32),
    SourcePaused(u32),
    SourceResumed(u32),
    FadeCompleted(u32),
    VolumeChanged(AudioType, f32),
    ListenerMoved(Vec3),
}

// 音频管理器主结构
pub struct AudioManager {
    config: AudioConfig,
    listener: AudioListener,
    
    // 音频源管理
    audio_sources: HashMap<u32, AudioSource>,
    loaded_audio: HashMap<String, Handle<AudioSource>>,
    
    // 播放控制
    active_sources: Vec<u32>,
    paused_sources: Vec<u32>,
    
    // 音频引擎
    audio_engine: Option<AudioEngine>,
    
    // 统计信息
    active_source_count: u32,
    total_memory_usage: usize,
    
    next_source_id: u32,
}

impl AudioManager {
    // 创建新的音频管理器
    pub fn new(engine_config: &EngineConfig) -> GameResult<Self> {
        let audio_engine = if cfg!(feature = "custom-audio") {
            Some(AudioEngine::new(44100, 1024)?)
        } else {
            None
        };

        Ok(Self {
            config: AudioConfig::default(),
            listener: AudioListener::default(),
            audio_sources: HashMap::new(),
            loaded_audio: HashMap::new(),
            active_sources: Vec::new(),
            paused_sources: Vec::new(),
            audio_engine,
            active_source_count: 0,
            total_memory_usage: 0,
            next_source_id: 1,
        })
    }

    // 初始化音频管理器
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化音频管理器...");
        
        // 预留音频源容量
        self.audio_sources.reserve(self.config.max_audio_sources);
        self.active_sources.reserve(self.config.max_audio_sources);
        
        info!("音频管理器初始化完成");
        Ok(())
    }

    // 关闭音频管理器
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("关闭音频管理器...");
        
        // 停止所有播放中的音频
        self.stop_all()?;
        
        // 清理资源
        self.audio_sources.clear();
        self.loaded_audio.clear();
        self.active_sources.clear();
        self.paused_sources.clear();
        
        info!("音频管理器已关闭");
        Ok(())
    }

    // 更新音频系统
    pub fn update(&mut self, delta_time: f32) -> GameResult<()> {
        // 更新淡入淡出效果
        self.update_fading(delta_time)?;
        
        // 更新3D音频位置
        if self.config.enable_3d_audio {
            self.update_3d_audio()?;
        }
        
        // 清理已停止的音频源
        self.cleanup_stopped_sources();
        
        // 更新统计信息
        self.active_source_count = self.active_sources.len() as u32;
        
        Ok(())
    }

    // 加载音频文件
    pub fn load_audio(&mut self, 
        asset_server: &AssetServer,
        path: &str, 
        audio_type: AudioType
    ) -> GameResult<u32> {
        
        if self.loaded_audio.contains_key(path) {
            return Err(GameError::Audio(format!("音频已加载: {}", path)));
        }

        let handle: Handle<AudioSource> = asset_server.load(path);
        let source_id = self.next_source_id;
        self.next_source_id += 1;

        let audio_source = AudioSource {
            id: source_id,
            name: path.to_string(),
            audio_type,
            handle: handle.clone(),
            ..Default::default()
        };

        self.audio_sources.insert(source_id, audio_source);
        self.loaded_audio.insert(path.to_string(), handle);

        info!("已加载音频: {} (ID: {})", path, source_id);
        Ok(source_id)
    }

    // 播放音频
    pub fn play(&mut self, 
        commands: &mut Commands,
        source_id: u32, 
        volume: Option<f32>,
        pitch: Option<f32>,
        loop_enabled: Option<bool>
    ) -> GameResult<()> {
        
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        if let Some(vol) = volume {
            audio_source.volume = vol.max(0.0).min(1.0);
        }
        if let Some(p) = pitch {
            audio_source.pitch = p.max(0.1).min(3.0);
        }
        if let Some(looping) = loop_enabled {
            audio_source.loop_enabled = looping;
        }

        // 计算最终音量
        let final_volume = self.calculate_final_volume(audio_source);

        // 使用Bevy音频系统播放
        let audio_bundle = AudioBundle {
            source: audio_source.handle.clone(),
            settings: PlaybackSettings {
                repeat: audio_source.loop_enabled,
                volume: Volume::new_relative(final_volume),
                speed: audio_source.pitch,
                ..default()
            },
        };

        commands.spawn(audio_bundle);

        audio_source.state = AudioState::Playing;
        
        if !self.active_sources.contains(&source_id) {
            self.active_sources.push(source_id);
        }
        
        self.paused_sources.retain(|&id| id != source_id);

        info!("播放音频: {} (音量: {:.2})", audio_source.name, final_volume);
        Ok(())
    }

    // 停止音频
    pub fn stop(&mut self, source_id: u32) -> GameResult<()> {
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        audio_source.state = AudioState::Stopped;
        self.active_sources.retain(|&id| id != source_id);
        self.paused_sources.retain(|&id| id != source_id);

        info!("停止音频: {}", audio_source.name);
        Ok(())
    }

    // 暂停音频
    pub fn pause(&mut self, source_id: u32) -> GameResult<()> {
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        if audio_source.state == AudioState::Playing {
            audio_source.state = AudioState::Paused;
            self.active_sources.retain(|&id| id != source_id);
            
            if !self.paused_sources.contains(&source_id) {
                self.paused_sources.push(source_id);
            }
            
            info!("暂停音频: {}", audio_source.name);
        }
        
        Ok(())
    }

    // 恢复音频
    pub fn resume(&mut self, commands: &mut Commands, source_id: u32) -> GameResult<()> {
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        if audio_source.state == AudioState::Paused {
            self.play(commands, source_id, None, None, None)?;
            info!("恢复音频: {}", audio_source.name);
        }
        
        Ok(())
    }

    // 淡入播放
    pub fn fade_in(&mut self, 
        commands: &mut Commands,
        source_id: u32, 
        duration: f32,
        target_volume: Option<f32>
    ) -> GameResult<()> {
        
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        let start_volume = 0.0;
        let end_volume = target_volume.unwrap_or(audio_source.volume);

        audio_source.volume = start_volume;
        audio_source.fade_duration = duration;
        audio_source.fade_target_volume = end_volume;
        audio_source.fade_timer = 0.0;
        audio_source.state = AudioState::Fading;

        self.play(commands, source_id, Some(start_volume), None, None)?;
        
        info!("淡入播放: {} (时长: {:.2}s)", audio_source.name, duration);
        Ok(())
    }

    // 淡出停止
    pub fn fade_out(&mut self, source_id: u32, duration: f32) -> GameResult<()> {
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        if audio_source.state == AudioState::Playing {
            audio_source.fade_duration = duration;
            audio_source.fade_target_volume = 0.0;
            audio_source.fade_timer = 0.0;
            audio_source.state = AudioState::Fading;
            
            info!("淡出音频: {} (时长: {:.2}s)", audio_source.name, duration);
        }
        
        Ok(())
    }

    // 设置3D位置
    pub fn set_3d_position(&mut self, source_id: u32, position: Vec3) -> GameResult<()> {
        let audio_source = self.audio_sources.get_mut(&source_id)
            .ok_or_else(|| GameError::Audio(format!("音频源不存在: {}", source_id)))?;

        audio_source.position = Some(position);
        Ok(())
    }

    // 设置监听器位置
    pub fn set_listener_position(&mut self, position: Vec3, forward: Vec3, up: Vec3) {
        self.listener.position = position;
        self.listener.forward = forward.normalize();
        self.listener.up = up.normalize();
    }

    // 设置监听器速度（用于多普勒效应）
    pub fn set_listener_velocity(&mut self, velocity: Vec3) {
        self.listener.velocity = velocity;
    }

    // 停止所有音频
    pub fn stop_all(&mut self) -> GameResult<()> {
        let active_sources: Vec<u32> = self.active_sources.clone();
        for source_id in active_sources {
            self.stop(source_id)?;
        }
        
        let paused_sources: Vec<u32> = self.paused_sources.clone();
        for source_id in paused_sources {
            self.stop(source_id)?;
        }
        
        info!("已停止所有音频");
        Ok(())
    }

    // 暂停所有音频
    pub fn pause_all(&mut self) -> GameResult<()> {
        let active_sources: Vec<u32> = self.active_sources.clone();
        for source_id in active_sources {
            self.pause(source_id)?;
        }
        
        info!("已暂停所有音频");
        Ok(())
    }

    // 恢复所有音频
    pub fn resume_all(&mut self, commands: &mut Commands) -> GameResult<()> {
        let paused_sources: Vec<u32> = self.paused_sources.clone();
        for source_id in paused_sources {
            self.resume(commands, source_id)?;
        }
        
        info!("已恢复所有音频");
        Ok(())
    }

    // 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.master_volume = volume.max(0.0).min(1.0);
        info!("设置主音量: {:.2}", self.config.master_volume);
    }

    // 设置分类音量
    pub fn set_volume_by_type(&mut self, audio_type: AudioType, volume: f32) {
        let vol = volume.max(0.0).min(1.0);
        
        match audio_type {
            AudioType::Music => self.config.music_volume = vol,
            AudioType::SoundEffect => self.config.sound_effect_volume = vol,
            AudioType::Voice => self.config.voice_volume = vol,
            AudioType::Ambient => self.config.ambient_volume = vol,
            AudioType::UI => self.config.ui_volume = vol,
        }
        
        info!("设置{:?}音量: {:.2}", audio_type, vol);
    }

    // 获取分类音量
    pub fn get_volume_by_type(&self, audio_type: AudioType) -> f32 {
        match audio_type {
            AudioType::Music => self.config.music_volume,
            AudioType::SoundEffect => self.config.sound_effect_volume,
            AudioType::Voice => self.config.voice_volume,
            AudioType::Ambient => self.config.ambient_volume,
            AudioType::UI => self.config.ui_volume,
        }
    }

    // 启用/禁用3D音频
    pub fn set_3d_audio_enabled(&mut self, enabled: bool) {
        self.config.enable_3d_audio = enabled;
        info!("3D音频: {}", if enabled { "启用" } else { "禁用" });
    }

    // 获取活跃音频源数量
    pub fn get_active_source_count(&self) -> u32 {
        self.active_source_count
    }

    // 获取内存使用量
    pub fn get_memory_usage(&self) -> usize {
        self.total_memory_usage
    }

    // 私有辅助方法

    // 计算最终音量
    fn calculate_final_volume(&self, audio_source: &AudioSource) -> f32 {
        let type_volume = self.get_volume_by_type(audio_source.audio_type);
        let mut final_volume = self.config.master_volume * type_volume * audio_source.volume;

        // 3D音频衰减
        if let Some(pos) = audio_source.position {
            if self.config.enable_3d_audio {
                let distance = (pos - self.listener.position).length();
                let attenuation = self.calculate_distance_attenuation(
                    distance, 
                    audio_source.max_distance, 
                    audio_source.rolloff_factor
                );
                final_volume *= attenuation;
            }
        }

        final_volume.max(0.0).min(1.0)
    }

    // 计算距离衰减
    fn calculate_distance_attenuation(&self, distance: f32, max_distance: f32, rolloff: f32) -> f32 {
        if distance >= max_distance {
            0.0
        } else {
            1.0 - (distance / max_distance).powf(rolloff)
        }
    }

    // 更新淡入淡出效果
    fn update_fading(&mut self, delta_time: f32) -> GameResult<()> {
        let fading_sources: Vec<u32> = self.audio_sources
            .iter()
            .filter(|(_, source)| source.state == AudioState::Fading)
            .map(|(&id, _)| id)
            .collect();

        for source_id in fading_sources {
            let audio_source = self.audio_sources.get_mut(&source_id).unwrap();
            
            audio_source.fade_timer += delta_time;
            let progress = (audio_source.fade_timer / audio_source.fade_duration).min(1.0);
            
            let start_volume = audio_source.volume;
            let target_volume = audio_source.fade_target_volume;
            let current_volume = start_volume + (target_volume - start_volume) * progress;
            
            audio_source.volume = current_volume;

            if progress >= 1.0 {
                // 淡入淡出完成
                audio_source.state = if target_volume > 0.0 {
                    AudioState::Playing
                } else {
                    AudioState::Stopped
                };

                if audio_source.state == AudioState::Stopped {
                    self.active_sources.retain(|&id| id != source_id);
                }
            }
        }

        Ok(())
    }

    // 更新3D音频
    fn update_3d_audio(&mut self) -> GameResult<()> {
        if let Some(ref audio_engine) = self.audio_engine {
            for (source_id, audio_source) in &self.audio_sources {
                if let Some(pos) = audio_source.position {
                    if audio_source.state == AudioState::Playing {
                        // 计算3D音频参数
                        let (gain, pan) = audio_engine.apply_3d_audio(
                            self.listener.position,
                            pos
                        )?;

                        // 这里应该应用计算出的gain和pan值
                        // 简化实现，实际需要与音频引擎集成
                    }
                }
            }
        }

        Ok(())
    }

    // 清理已停止的音频源
    fn cleanup_stopped_sources(&mut self) {
        self.active_sources.retain(|&source_id| {
            if let Some(source) = self.audio_sources.get(&source_id) {
                source.state == AudioState::Playing || source.state == AudioState::Fading
            } else {
                false
            }
        });

        self.paused_sources.retain(|&source_id| {
            if let Some(source) = self.audio_sources.get(&source_id) {
                source.state == AudioState::Paused
            } else {
                false
            }
        });
    }
}

// 便捷函数
impl AudioManager {
    // 播放一次性音效
    pub fn play_sound_effect(&mut self, 
        commands: &mut Commands,
        asset_server: &AssetServer,
        path: &str,
        volume: Option<f32>
    ) -> GameResult<u32> {
        
        let source_id = self.load_audio(asset_server, path, AudioType::SoundEffect)?;
        self.play(commands, source_id, volume, None, Some(false))?;
        Ok(source_id)
    }

    // 播放循环背景音乐
    pub fn play_background_music(&mut self, 
        commands: &mut Commands,
        asset_server: &AssetServer,
        path: &str,
        volume: Option<f32>
    ) -> GameResult<u32> {
        
        // 停止当前背景音乐
        self.stop_music()?;
        
        let source_id = self.load_audio(asset_server, path, AudioType::Music)?;
        self.play(commands, source_id, volume, None, Some(true))?;
        Ok(source_id)
    }

    // 停止背景音乐
    pub fn stop_music(&mut self) -> GameResult<()> {
        let music_sources: Vec<u32> = self.audio_sources
            .iter()
            .filter(|(_, source)| source.audio_type == AudioType::Music)
            .map(|(&id, _)| id)
            .collect();

        for source_id in music_sources {
            if self.active_sources.contains(&source_id) {
                self.stop(source_id)?;
            }
        }

        Ok(())
    }

    // 播放UI音效
    pub fn play_ui_sound(&mut self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        sound_name: &str
    ) -> GameResult<()> {
        
        let path = format!("audio/ui/{}.ogg", sound_name);
        self.play_sound_effect(commands, asset_server, &path, Some(self.config.ui_volume))?;
        Ok(())
    }

    // 批量预加载音频
    pub fn preload_audio_batch(&mut self, 
        asset_server: &AssetServer,
        audio_list: &[(String, AudioType)]
    ) -> GameResult<Vec<u32>> {
        
        let mut loaded_ids = Vec::new();
        
        for (path, audio_type) in audio_list {
            match self.load_audio(asset_server, path, *audio_type) {
                Ok(id) => loaded_ids.push(id),
                Err(e) => warn!("加载音频失败: {} - {}", path, e),
            }
        }
        
        info!("批量预加载音频完成: {}/{}", loaded_ids.len(), audio_list.len());
        Ok(loaded_ids)
    }

    // 卸载音频
    pub fn unload_audio(&mut self, source_id: u32) -> GameResult<()> {
        // 先停止音频
        if self.active_sources.contains(&source_id) || self.paused_sources.contains(&source_id) {
            self.stop(source_id)?;
        }

        // 移除音频源
        if let Some(audio_source) = self.audio_sources.remove(&source_id) {
            self.loaded_audio.remove(&audio_source.name);
            info!("卸载音频: {}", audio_source.name);
        }

        Ok(())
    }

    // 获取音频信息
    pub fn get_audio_info(&self, source_id: u32) -> Option<&AudioSource> {
        self.audio_sources.get(&source_id)
    }

    // 检查音频是否正在播放
    pub fn is_playing(&self, source_id: u32) -> bool {
        self.audio_sources.get(&source_id)
            .map(|source| source.state == AudioState::Playing)
            .unwrap_or(false)
    }

    // 设置多普勒效应强度
    pub fn set_doppler_factor(&mut self, factor: f32) {
        self.config.doppler_factor = factor.max(0.0).min(5.0);
    }
}

// Bevy系统实现
pub fn audio_system(
    mut commands: Commands,
    mut audio_manager: ResMut<AudioManager>,
    time: Res<Time>,
) {
    let _ = audio_manager.update(time.delta_seconds());
}

// 音频事件处理系统
pub fn audio_events_system(
    mut audio_events: EventReader<AudioEvent>,
    mut audio_manager: ResMut<AudioManager>,
) {
    for event in audio_events.iter() {
        match event {
            AudioEvent::VolumeChanged(audio_type, volume) => {
                audio_manager.set_volume_by_type(*audio_type, *volume);
            },
            AudioEvent::ListenerMoved(position) => {
                audio_manager.set_listener_position(*position, Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0));
            },
            _ => {
                debug!("音频事件: {:?}", event);
            }
        }
    }
}