// 音乐播放系统 - 背景音乐和环境音效管理
// 开发心理：专注于长时间播放的背景音乐，支持无缝循环、交叉淡化、情境切换
// 设计原则：内存流式播放、无缝过渡、情境感知、性能优化

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceManager, ResourceHandle};
use crate::audio::{AudioCategory, AudioInstance, AudioTransform};
use crate::audio::sound::{SoundBuffer, SoundInstance, SoundEffect, SoundEffectType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use log::{info, debug, warn, error};

// 音乐轨道信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub category: MusicCategory,
    pub file_path: String,
    pub duration: Duration,
    pub intro_duration: Option<Duration>,
    pub loop_start: Option<Duration>,
    pub loop_end: Option<Duration>,
    pub bpm: Option<u16>,
    pub key_signature: Option<String>,
    pub mood_tags: Vec<MoodTag>,
    pub game_contexts: Vec<GameContext>,
    pub fade_in_duration: Duration,
    pub fade_out_duration: Duration,
    pub priority: u8, // 0-255, 越高优先级越高
}

// 音乐分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicCategory {
    MainTheme,     // 主题音乐
    Battle,        // 战斗音乐
    Overworld,     // 大世界音乐
    Town,          // 城镇音乐
    Route,         // 路线音乐
    Gym,           // 道馆音乐
    Victory,       // 胜利音乐
    Defeat,        // 失败音乐
    Menu,          // 菜单音乐
    Cutscene,      // 过场音乐
    Credits,       // 制作人员音乐
    Ambient,       // 环境音乐
}

// 情绪标签
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoodTag {
    Calm,         // 平静
    Energetic,    // 充满活力
    Mysterious,   // 神秘
    Heroic,       // 英雄
    Melancholy,   // 忧郁
    Triumphant,   // 胜利
    Tense,        // 紧张
    Peaceful,     // 和平
    Epic,         // 史诗
    Nostalgic,    // 怀旧
}

// 游戏情境
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameContext {
    MainMenu,
    CharacterSelection,
    PokemonCenter,
    WildGrass,
    Cave,
    Ocean,
    Mountain,
    Forest,
    Desert,
    City,
    GymLeader,
    EliteFour,
    Champion,
    LegendaryEncounter,
    Evolution,
    Credits,
}

// 音乐播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicState {
    Stopped,
    Playing,
    Paused,
    FadingIn,
    FadingOut,
    Crossfading,
}

// 音乐播放器实例
#[derive(Debug)]
pub struct MusicPlayer {
    pub id: u64,
    pub track: MusicTrack,
    pub sound_instance: Option<SoundInstance>,
    pub state: MusicState,
    pub volume: f32,
    pub base_volume: f32, // 轨道基础音量
    pub category_volume: f32, // 分类音量
    pub master_volume: f32, // 主音量
    pub current_position: Duration,
    pub loop_count: u32,
    pub max_loops: Option<u32>,
    pub created_at: Instant,
    pub started_at: Option<Instant>,
    pub fade_state: Option<MusicFadeState>,
    pub effects: Vec<SoundEffect>,
    pub is_streaming: bool,
    pub stream_buffer_size: usize,
}

// 音乐淡入淡出状态
#[derive(Debug, Clone)]
pub struct MusicFadeState {
    pub fade_type: MusicFadeType,
    pub start_volume: f32,
    pub target_volume: f32,
    pub duration: Duration,
    pub start_time: Instant,
    pub curve: FadeCurve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicFadeType {
    FadeIn,
    FadeOut,
    CrossfadeOut, // 交叉淡化中的淡出部分
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl MusicPlayer {
    pub fn new(id: u64, track: MusicTrack, volume: f32) -> Self {
        Self {
            id,
            track,
            sound_instance: None,
            state: MusicState::Stopped,
            volume,
            base_volume: volume,
            category_volume: 1.0,
            master_volume: 1.0,
            current_position: Duration::ZERO,
            loop_count: 0,
            max_loops: None,
            created_at: Instant::now(),
            started_at: None,
            fade_state: None,
            effects: Vec::new(),
            is_streaming: false,
            stream_buffer_size: 4096,
        }
    }
    
    // 开始播放
    pub fn play(&mut self) -> Result<()> {
        self.load_sound_buffer()?;
        
        if let Some(ref mut instance) = self.sound_instance {
            instance.play()?;
            self.state = MusicState::Playing;
            self.started_at = Some(Instant::now());
            
            // 如果有淡入设置，开始淡入
            if self.track.fade_in_duration > Duration::ZERO {
                self.start_fade_in();
            }
        }
        
        info!("开始播放音乐: {} ({})", self.track.title, self.track.id);
        Ok(())
    }
    
    // 暂停播放
    pub fn pause(&mut self) {
        if let Some(ref mut instance) = self.sound_instance {
            instance.pause();
            self.state = MusicState::Paused;
        }
        debug!("暂停音乐: {}", self.track.id);
    }
    
    // 恢复播放
    pub fn resume(&mut self) -> Result<()> {
        if let Some(ref mut instance) = self.sound_instance {
            instance.resume();
            self.state = MusicState::Playing;
        }
        debug!("恢复音乐: {}", self.track.id);
        Ok(())
    }
    
    // 停止播放
    pub fn stop(&mut self) {
        if let Some(ref mut instance) = self.sound_instance {
            instance.stop();
        }
        
        self.state = MusicState::Stopped;
        self.current_position = Duration::ZERO;
        self.loop_count = 0;
        self.started_at = None;
        self.fade_state = None;
        
        debug!("停止音乐: {}", self.track.id);
    }
    
    // 淡出停止
    pub fn fade_out_and_stop(&mut self, duration: Option<Duration>) {
        let fade_duration = duration.unwrap_or(self.track.fade_out_duration);
        
        self.fade_state = Some(MusicFadeState {
            fade_type: MusicFadeType::FadeOut,
            start_volume: self.volume,
            target_volume: 0.0,
            duration: fade_duration,
            start_time: Instant::now(),
            curve: FadeCurve::Linear,
        });
        
        self.state = MusicState::FadingOut;
        debug!("开始淡出音乐: {} (时长: {:?})", self.track.id, fade_duration);
    }
    
    // 开始淡入
    fn start_fade_in(&mut self) {
        let original_volume = self.volume;
        self.volume = 0.0;
        
        self.fade_state = Some(MusicFadeState {
            fade_type: MusicFadeType::FadeIn,
            start_volume: 0.0,
            target_volume: original_volume,
            duration: self.track.fade_in_duration,
            start_time: Instant::now(),
            curve: FadeCurve::Linear,
        });
        
        self.state = MusicState::FadingIn;
    }
    
    // 设置播放位置
    pub fn set_position(&mut self, position: Duration) -> Result<()> {
        if let Some(ref mut instance) = self.sound_instance {
            instance.set_position(position.as_secs_f32())?;
            self.current_position = position;
        }
        Ok(())
    }
    
    // 设置音量
    pub fn set_volume(&mut self, volume: f32) {
        self.base_volume = volume.clamp(0.0, 1.0);
        self.update_final_volume();
    }
    
    // 设置分类音量
    pub fn set_category_volume(&mut self, volume: f32) {
        self.category_volume = volume.clamp(0.0, 1.0);
        self.update_final_volume();
    }
    
    // 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.update_final_volume();
    }
    
    // 更新最终音量
    fn update_final_volume(&mut self) {
        self.volume = self.base_volume * self.category_volume * self.master_volume;
        
        if let Some(ref mut instance) = self.sound_instance {
            instance.volume = self.volume;
        }
    }
    
    // 加载音频缓冲区
    fn load_sound_buffer(&mut self) -> Result<()> {
        if self.sound_instance.is_some() {
            return Ok(());
        }
        
        // 加载音频文件
        let buffer = SoundBuffer::load_from_file(Path::new(&self.track.file_path))?;
        let handle = ResourceManager::instance().store_resource(
            format!("music_{}", self.track.id),
            buffer,
        );
        
        // 创建音频实例
        let mut instance = SoundInstance::new(
            self.id,
            handle,
            AudioCategory::Music,
            self.volume,
            1.0,
        );
        
        // 设置循环
        instance.is_looping = true;
        
        // 设置循环点
        if let (Some(start), Some(end)) = (self.track.loop_start, self.track.loop_end) {
            // TODO: 将Duration转换为采样点并设置循环点
        }
        
        self.sound_instance = Some(instance);
        Ok(())
    }
    
    // 更新播放器状态
    pub fn update(&mut self, delta_time: Duration) -> Result<()> {
        // 更新当前播放位置
        if self.state == MusicState::Playing {
            self.current_position += delta_time;
        }
        
        // 更新淡入淡出
        self.update_fade();
        
        // 更新音频实例
        if let Some(ref mut instance) = self.sound_instance {
            instance.update_fade();
            
            // 检查是否播放完成
            if !instance.is_playing && self.state == MusicState::Playing {
                self.handle_track_end()?;
            }
        }
        
        Ok(())
    }
    
    // 更新淡入淡出状态
    fn update_fade(&mut self) {
        if let Some(ref fade) = self.fade_state.clone() {
            let elapsed = fade.start_time.elapsed();
            let progress = (elapsed.as_secs_f32() / fade.duration.as_secs_f32()).min(1.0);
            
            if progress >= 1.0 {
                // 淡化完成
                match fade.fade_type {
                    MusicFadeType::FadeIn => {
                        self.volume = fade.target_volume;
                        self.state = MusicState::Playing;
                    },
                    MusicFadeType::FadeOut => {
                        self.volume = 0.0;
                        self.stop();
                        return;
                    },
                    MusicFadeType::CrossfadeOut => {
                        self.volume = 0.0;
                        self.stop();
                        return;
                    },
                }
                
                self.fade_state = None;
            } else {
                // 计算当前音量
                let volume_progress = match fade.curve {
                    FadeCurve::Linear => progress,
                    FadeCurve::Exponential => progress * progress,
                    FadeCurve::Logarithmic => (progress * std::f32::consts::E).ln() / std::f32::consts::E.ln(),
                    FadeCurve::SCurve => 3.0 * progress * progress - 2.0 * progress * progress * progress,
                };
                
                self.volume = fade.start_volume + (fade.target_volume - fade.start_volume) * volume_progress;
            }
            
            // 更新音频实例音量
            if let Some(ref mut instance) = self.sound_instance {
                instance.volume = self.volume;
            }
        }
    }
    
    // 处理轨道播放结束
    fn handle_track_end(&mut self) -> Result<()> {
        self.loop_count += 1;
        
        // 检查是否达到最大循环次数
        if let Some(max_loops) = self.max_loops {
            if self.loop_count >= max_loops {
                self.stop();
                return Ok(());
            }
        }
        
        // 重置播放位置
        if let Some(loop_start) = self.track.loop_start {
            self.set_position(loop_start)?;
        } else {
            self.set_position(Duration::ZERO)?;
        }
        
        debug!("音乐 {} 循环播放，第 {} 次", self.track.id, self.loop_count);
        Ok(())
    }
    
    // 添加音效
    pub fn add_effect(&mut self, effect: SoundEffect) {
        self.effects.push(effect);
        if let Some(ref mut instance) = self.sound_instance {
            instance.add_effect(effect);
        }
    }
    
    // 移除音效
    pub fn remove_effect(&mut self, effect_type: SoundEffectType) {
        self.effects.retain(|e| e.effect_type != effect_type);
        if let Some(ref mut instance) = self.sound_instance {
            instance.remove_effect(effect_type);
        }
    }
}

// 音乐管理器
pub struct MusicManager {
    players: HashMap<String, MusicPlayer>,
    current_track: Option<String>,
    crossfade_queue: VecDeque<CrossfadeRequest>,
    track_library: HashMap<String, MusicTrack>,
    category_volumes: HashMap<MusicCategory, f32>,
    master_volume: f32,
    next_player_id: u64,
    
    // 智能播放列表
    current_context: Option<GameContext>,
    mood_weights: HashMap<MoodTag, f32>,
    recently_played: VecDeque<String>,
    max_recent_tracks: usize,
    
    // 性能统计
    streaming_enabled: bool,
    total_tracks_played: u64,
    total_playtime: Duration,
}

#[derive(Debug)]
struct CrossfadeRequest {
    from_track: String,
    to_track: String,
    duration: Duration,
    started_at: Instant,
}

impl MusicManager {
    pub fn new() -> Self {
        let mut category_volumes = HashMap::new();
        
        // 设置默认分类音量
        for category in [
            MusicCategory::MainTheme,
            MusicCategory::Battle,
            MusicCategory::Overworld,
            MusicCategory::Town,
            MusicCategory::Route,
            MusicCategory::Gym,
            MusicCategory::Victory,
            MusicCategory::Defeat,
            MusicCategory::Menu,
            MusicCategory::Cutscene,
            MusicCategory::Credits,
            MusicCategory::Ambient,
        ] {
            category_volumes.insert(category, 1.0);
        }
        
        Self {
            players: HashMap::new(),
            current_track: None,
            crossfade_queue: VecDeque::new(),
            track_library: HashMap::new(),
            category_volumes,
            master_volume: 1.0,
            next_player_id: 1,
            
            current_context: None,
            mood_weights: HashMap::new(),
            recently_played: VecDeque::new(),
            max_recent_tracks: 10,
            
            streaming_enabled: true,
            total_tracks_played: 0,
            total_playtime: Duration::ZERO,
        }
    }
    
    // 加载音乐库
    pub fn load_music_library(&mut self, library_path: &Path) -> Result<()> {
        // TODO: 从JSON或TOML文件加载音乐库配置
        info!("加载音乐库: {:?}", library_path);
        
        // 示例：添加一些默认音乐轨道
        self.add_track(MusicTrack {
            id: "main_theme".to_string(),
            title: "主题曲".to_string(),
            artist: Some("Game Composer".to_string()),
            category: MusicCategory::MainTheme,
            file_path: "assets/music/main_theme.ogg".to_string(),
            duration: Duration::from_secs(180),
            intro_duration: Some(Duration::from_secs(8)),
            loop_start: Some(Duration::from_secs(8)),
            loop_end: None,
            bpm: Some(120),
            key_signature: Some("C Major".to_string()),
            mood_tags: vec![MoodTag::Heroic, MoodTag::Epic],
            game_contexts: vec![GameContext::MainMenu],
            fade_in_duration: Duration::from_secs(2),
            fade_out_duration: Duration::from_secs(3),
            priority: 255,
        });
        
        info!("音乐库加载完成，共 {} 个轨道", self.track_library.len());
        Ok(())
    }
    
    // 添加音乐轨道
    pub fn add_track(&mut self, track: MusicTrack) {
        let track_id = track.id.clone();
        self.track_library.insert(track_id.clone(), track);
        debug!("添加音乐轨道: {}", track_id);
    }
    
    // 播放指定轨道
    pub fn play_track(&mut self, track_id: &str, volume: f32, crossfade_duration: Option<Duration>) -> Result<()> {
        let track = self.track_library.get(track_id)
            .ok_or_else(|| GameError::AudioError(format!("音乐轨道不存在: {}", track_id)))?
            .clone();
        
        // 如果有交叉淡化时间且当前有播放的轨道，进行交叉淡化
        if let (Some(duration), Some(current)) = (crossfade_duration, &self.current_track) {
            if current != track_id {
                self.crossfade_to_track(track_id, duration)?;
                return Ok(());
            }
        }
        
        // 停止当前播放的轨道
        if let Some(current) = &self.current_track {
            if let Some(player) = self.players.get_mut(current) {
                player.fade_out_and_stop(None);
            }
        }
        
        // 创建新的播放器
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        
        let mut player = MusicPlayer::new(player_id, track.clone(), volume);
        player.set_category_volume(self.category_volumes.get(&track.category).copied().unwrap_or(1.0));
        player.set_master_volume(self.master_volume);
        
        // 开始播放
        player.play()?;
        
        self.players.insert(track_id.to_string(), player);
        self.current_track = Some(track_id.to_string());
        
        // 更新播放历史
        self.add_to_recent_history(track_id);
        self.total_tracks_played += 1;
        
        info!("开始播放音乐: {}", track.title);
        Ok(())
    }
    
    // 交叉淡化到新轨道
    fn crossfade_to_track(&mut self, track_id: &str, duration: Duration) -> Result<()> {
        let current_track = self.current_track.as_ref()
            .ok_or_else(|| GameError::AudioError("没有当前播放的轨道".to_string()))?;
        
        // 为当前轨道开始淡出
        if let Some(player) = self.players.get_mut(current_track) {
            player.fade_state = Some(MusicFadeState {
                fade_type: MusicFadeType::CrossfadeOut,
                start_volume: player.volume,
                target_volume: 0.0,
                duration,
                start_time: Instant::now(),
                curve: FadeCurve::Linear,
            });
            player.state = MusicState::Crossfading;
        }
        
        // 添加到交叉淡化队列
        self.crossfade_queue.push_back(CrossfadeRequest {
            from_track: current_track.clone(),
            to_track: track_id.to_string(),
            duration,
            started_at: Instant::now(),
        });
        
        debug!("开始交叉淡化: {} -> {} (时长: {:?})", current_track, track_id, duration);
        Ok(())
    }
    
    // 停止当前音乐
    pub fn stop_current_music(&mut self, fade_out_duration: Option<Duration>) {
        if let Some(track_id) = &self.current_track {
            if let Some(player) = self.players.get_mut(track_id) {
                if let Some(duration) = fade_out_duration {
                    player.fade_out_and_stop(Some(duration));
                } else {
                    player.stop();
                }
            }
            self.current_track = None;
        }
    }
    
    // 暂停当前音乐
    pub fn pause_current_music(&mut self) {
        if let Some(track_id) = &self.current_track {
            if let Some(player) = self.players.get_mut(track_id) {
                player.pause();
            }
        }
    }
    
    // 恢复当前音乐
    pub fn resume_current_music(&mut self) -> Result<()> {
        if let Some(track_id) = &self.current_track {
            if let Some(player) = self.players.get_mut(track_id) {
                player.resume()?;
            }
        }
        Ok(())
    }
    
    // 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        
        // 更新所有播放器的主音量
        for player in self.players.values_mut() {
            player.set_master_volume(self.master_volume);
        }
        
        debug!("设置音乐主音量: {}", self.master_volume);
    }
    
    // 设置分类音量
    pub fn set_category_volume(&mut self, category: MusicCategory, volume: f32) {
        self.category_volumes.insert(category, volume.clamp(0.0, 1.0));
        
        // 更新所有该分类播放器的音量
        for player in self.players.values_mut() {
            if player.track.category == category {
                player.set_category_volume(volume);
            }
        }
        
        debug!("设置音乐分类 {:?} 音量: {}", category, volume);
    }
    
    // 设置游戏情境
    pub fn set_game_context(&mut self, context: GameContext) {
        self.current_context = Some(context);
        debug!("设置游戏情境: {:?}", context);
    }
    
    // 根据情境播放合适的音乐
    pub fn play_contextual_music(&mut self, volume: f32) -> Result<()> {
        if let Some(context) = &self.current_context {
            let suitable_tracks = self.find_tracks_for_context(*context);
            
            if let Some(track_id) = self.select_best_track(&suitable_tracks) {
                self.play_track(&track_id, volume, Some(Duration::from_secs(3)))?;
            }
        }
        
        Ok(())
    }
    
    // 查找适合情境的音乐
    fn find_tracks_for_context(&self, context: GameContext) -> Vec<&MusicTrack> {
        self.track_library
            .values()
            .filter(|track| track.game_contexts.contains(&context))
            .collect()
    }
    
    // 选择最佳音乐轨道
    fn select_best_track(&self, tracks: &[&MusicTrack]) -> Option<String> {
        if tracks.is_empty() {
            return None;
        }
        
        // 简化的选择算法：避免最近播放过的，优先选择高优先级的
        tracks
            .iter()
            .filter(|track| !self.recently_played.contains(&track.id))
            .max_by_key(|track| track.priority)
            .or_else(|| tracks.first())
            .map(|track| track.id.clone())
    }
    
    // 添加到播放历史
    fn add_to_recent_history(&mut self, track_id: &str) {
        self.recently_played.push_back(track_id.to_string());
        
        if self.recently_played.len() > self.max_recent_tracks {
            self.recently_played.pop_front();
        }
    }
    
    // 更新音乐管理器
    pub fn update(&mut self, delta_time: Duration) -> Result<()> {
        // 更新所有播放器
        let mut finished_players = Vec::new();
        
        for (track_id, player) in &mut self.players {
            player.update(delta_time)?;
            
            // 记录播放时间
            if player.state == MusicState::Playing {
                self.total_playtime += delta_time;
            }
            
            // 标记已停止的播放器
            if player.state == MusicState::Stopped {
                finished_players.push(track_id.clone());
            }
        }
        
        // 移除已停止的播放器
        for track_id in finished_players {
            self.players.remove(&track_id);
            if self.current_track.as_ref() == Some(&track_id) {
                self.current_track = None;
            }
        }
        
        // 处理交叉淡化队列
        self.update_crossfades()?;
        
        Ok(())
    }
    
    // 更新交叉淡化
    fn update_crossfades(&mut self) -> Result<()> {
        let mut completed_crossfades = Vec::new();
        
        for (i, crossfade) in self.crossfade_queue.iter().enumerate() {
            let elapsed = crossfade.started_at.elapsed();
            let progress = elapsed.as_secs_f32() / crossfade.duration.as_secs_f32();
            
            if progress >= 0.5 && self.current_track.as_ref() != Some(&crossfade.to_track) {
                // 在中点开始播放新轨道
                if let Some(track) = self.track_library.get(&crossfade.to_track) {
                    let player_id = self.next_player_id;
                    self.next_player_id += 1;
                    
                    let mut new_player = MusicPlayer::new(player_id, track.clone(), 0.0);
                    new_player.set_category_volume(
                        self.category_volumes.get(&track.category).copied().unwrap_or(1.0)
                    );
                    new_player.set_master_volume(self.master_volume);
                    
                    // 设置淡入
                    new_player.fade_state = Some(MusicFadeState {
                        fade_type: MusicFadeType::FadeIn,
                        start_volume: 0.0,
                        target_volume: new_player.base_volume,
                        duration: crossfade.duration / 2,
                        start_time: Instant::now(),
                        curve: FadeCurve::Linear,
                    });
                    
                    new_player.play()?;
                    new_player.state = MusicState::FadingIn;
                    
                    self.players.insert(crossfade.to_track.clone(), new_player);
                    self.current_track = Some(crossfade.to_track.clone());
                    
                    info!("交叉淡化：开始播放 {}", crossfade.to_track);
                }
            }
            
            if progress >= 1.0 {
                completed_crossfades.push(i);
            }
        }
        
        // 移除完成的交叉淡化
        for &i in completed_crossfades.iter().rev() {
            let crossfade = self.crossfade_queue.remove(i).unwrap();
            debug!("交叉淡化完成: {} -> {}", crossfade.from_track, crossfade.to_track);
        }
        
        Ok(())
    }
    
    // 获取播放统计信息
    pub fn get_stats(&self) -> MusicStats {
        MusicStats {
            total_tracks: self.track_library.len(),
            total_tracks_played: self.total_tracks_played,
            total_playtime: self.total_playtime,
            current_track: self.current_track.clone(),
            active_players: self.players.len(),
        }
    }
}

// 音乐统计信息
#[derive(Debug, Clone)]
pub struct MusicStats {
    pub total_tracks: usize,
    pub total_tracks_played: u64,
    pub total_playtime: Duration,
    pub current_track: Option<String>,
    pub active_players: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_music_track_creation() {
        let track = MusicTrack {
            id: "test_track".to_string(),
            title: "Test Track".to_string(),
            artist: Some("Test Artist".to_string()),
            category: MusicCategory::Battle,
            file_path: "test.ogg".to_string(),
            duration: Duration::from_secs(120),
            intro_duration: None,
            loop_start: Some(Duration::from_secs(10)),
            loop_end: None,
            bpm: Some(140),
            key_signature: Some("A Minor".to_string()),
            mood_tags: vec![MoodTag::Energetic, MoodTag::Tense],
            game_contexts: vec![GameContext::GymLeader],
            fade_in_duration: Duration::from_secs(1),
            fade_out_duration: Duration::from_secs(2),
            priority: 128,
        };
        
        assert_eq!(track.id, "test_track");
        assert_eq!(track.category, MusicCategory::Battle);
        assert_eq!(track.mood_tags.len(), 2);
    }
    
    #[test]
    fn test_music_manager_creation() {
        let manager = MusicManager::new();
        
        assert_eq!(manager.master_volume, 1.0);
        assert!(manager.current_track.is_none());
        assert_eq!(manager.players.len(), 0);
        assert_eq!(manager.category_volumes.len(), 12);
    }
    
    #[test]
    fn test_fade_curve_calculation() {
        let progress = 0.5f32;
        
        let linear = progress;
        let exponential = progress * progress;
        let s_curve = 3.0 * progress * progress - 2.0 * progress * progress * progress;
        
        assert!((linear - 0.5).abs() < 0.001);
        assert!((exponential - 0.25).abs() < 0.001);
        assert!((s_curve - 0.5).abs() < 0.001);
    }
}