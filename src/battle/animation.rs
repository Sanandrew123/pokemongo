// 战斗动画系统
// 开发心理：战斗动画是游戏体验的关键，需要流畅、华丽、有冲击感
// 设计原则：时间轴管理、缓动函数、特效组合、性能优化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;
use crate::pokemon::moves::MoveId;

// 动画类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationType {
    // 基础动画
    Idle,           // 待机
    Attack,         // 攻击
    Hit,            // 受击
    Faint,          // 濒死
    Victory,        // 胜利
    
    // 移动动画
    MoveIn,         // 进场
    MoveOut,        // 退场
    Switch,         // 切换
    
    // 技能动画
    Move(MoveId),   // 技能动画
    
    // 状态动画
    StatusApply,    // 状态施加
    StatusRemove,   // 状态移除
    Heal,           // 治疗
    
    // 特效动画
    Particle,       // 粒子特效
    Screen,         // 全屏特效
    UI,             // UI动画
    
    // 自定义动画
    Custom(String),
}

// 动画状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Ready,      // 准备
    Playing,    // 播放中
    Paused,     // 暂停
    Finished,   // 完成
    Cancelled,  // 取消
}

// 缓动函数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    
    // Quadratic
    QuadIn,
    QuadOut,
    QuadInOut,
    
    // Cubic
    CubicIn,
    CubicOut,
    CubicInOut,
    
    // Quartic
    QuartIn,
    QuartOut,
    QuartInOut,
    
    // Quintic
    QuintIn,
    QuintOut,
    QuintInOut,
    
    // Sinusoidal
    SineIn,
    SineOut,
    SineInOut,
    
    // Exponential
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    
    // Circular
    CircIn,
    CircOut,
    CircInOut,
    
    // Elastic
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    
    // Bounce
    BounceIn,
    BounceOut,
    BounceInOut,
    
    // Back
    BackIn,
    BackOut,
    BackInOut,
}

// 动画关键帧
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,                      // 时间点 (0.0-1.0)
    pub position: Option<glam::Vec3>,   // 位置
    pub rotation: Option<glam::Quat>,   // 旋转
    pub scale: Option<glam::Vec3>,      // 缩放
    pub color: Option<glam::Vec4>,      // 颜色 (RGBA)
    pub opacity: Option<f32>,           // 透明度
    pub custom_values: HashMap<String, f32>, // 自定义值
}

// 动画轨道
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub name: String,
    pub target: String,         // 目标对象名称
    pub property: String,       // 属性名称
    pub keyframes: Vec<Keyframe>,
    pub easing: EasingType,
    pub loop_mode: LoopMode,
}

// 循环模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    None,       // 不循环
    Loop,       // 循环
    PingPong,   // 往返
    Reverse,    // 反向播放
}

// 动画剪辑
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub id: String,
    pub name: String,
    pub duration: f32,          // 总时长(秒)
    pub tracks: Vec<AnimationTrack>,
    pub events: Vec<AnimationEvent>, // 动画事件
    pub priority: i32,          // 优先级
    pub blend_mode: BlendMode,  // 混合模式
}

// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Replace,    // 替换
    Add,        // 叠加
    Multiply,   // 相乘
    Override,   // 覆盖
}

// 动画事件
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub time: f32,              // 触发时间
    pub event_type: String,     // 事件类型
    pub parameters: HashMap<String, String>, // 事件参数
}

// 动画实例
#[derive(Debug, Clone)]
pub struct AnimationInstance {
    pub id: u64,
    pub clip_id: String,
    pub state: AnimationState,
    pub current_time: f32,      // 当前播放时间
    pub playback_speed: f32,    // 播放速度
    pub weight: f32,            // 混合权重
    pub start_time: f32,        // 开始时间
    pub end_time: f32,          // 结束时间
    pub loop_count: i32,        // 循环次数 (-1为无限)
    pub current_loop: i32,      // 当前循环
    pub reverse: bool,          // 是否反向播放
    pub events_fired: Vec<usize>, // 已触发的事件索引
}

// 动画目标对象
#[derive(Debug, Clone)]
pub struct AnimationTarget {
    pub name: String,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    pub color: glam::Vec4,
    pub opacity: f32,
    pub custom_properties: HashMap<String, f32>,
    pub visible: bool,
}

// 粒子系统配置
#[derive(Debug, Clone)]
pub struct ParticleConfig {
    pub max_particles: usize,
    pub emission_rate: f32,     // 发射速率(粒子/秒)
    pub lifetime: (f32, f32),   // 生命周期范围
    pub initial_velocity: (glam::Vec3, glam::Vec3), // 初始速度范围
    pub acceleration: glam::Vec3, // 加速度
    pub size: (f32, f32),       // 大小范围
    pub color_over_time: Vec<(f32, glam::Vec4)>, // 颜色随时间变化
    pub texture: String,        // 纹理名称
}

// 动画管理器
pub struct BattleAnimationManager {
    // 动画剪辑库
    clips: HashMap<String, AnimationClip>,
    
    // 活跃动画实例
    active_instances: HashMap<u64, AnimationInstance>,
    
    // 动画目标对象
    targets: HashMap<String, AnimationTarget>,
    
    // 粒子系统
    particle_systems: HashMap<String, ParticleConfig>,
    
    // 动画队列
    animation_queue: Vec<QueuedAnimation>,
    
    // 全局设置
    global_speed: f32,
    animation_quality: AnimationQuality,
    enable_particles: bool,
    max_concurrent_animations: usize,
    
    // 统计信息
    next_instance_id: u64,
    total_animations_played: u64,
    
    // 回调函数
    completion_callbacks: HashMap<u64, Box<dyn FnOnce() + Send>>,
}

// 队列中的动画
#[derive(Debug, Clone)]
struct QueuedAnimation {
    clip_id: String,
    target: String,
    delay: f32,
    parameters: HashMap<String, f32>,
    priority: i32,
}

// 动画质量设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationQuality {
    Low,     // 低质量，减少帧率和特效
    Medium,  // 中等质量
    High,    // 高质量，所有特效
    Ultra,   // 超高质量，额外的细节
}

impl BattleAnimationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            clips: HashMap::new(),
            active_instances: HashMap::new(),
            targets: HashMap::new(),
            particle_systems: HashMap::new(),
            animation_queue: Vec::new(),
            global_speed: 1.0,
            animation_quality: AnimationQuality::High,
            enable_particles: true,
            max_concurrent_animations: 50,
            next_instance_id: 1,
            total_animations_played: 0,
            completion_callbacks: HashMap::new(),
        };
        
        manager.load_default_clips();
        manager.setup_default_particles();
        manager
    }
    
    // 更新动画系统
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        let adjusted_delta = delta_time * self.global_speed;
        
        // 处理动画队列
        self.process_animation_queue(adjusted_delta)?;
        
        // 更新活跃动画
        self.update_active_animations(adjusted_delta)?;
        
        // 清理完成的动画
        self.cleanup_finished_animations();
        
        Ok(())
    }
    
    // 播放动画
    pub fn play_animation(
        &mut self, 
        clip_id: &str, 
        target: &str, 
        parameters: Option<HashMap<String, f32>>
    ) -> Result<u64, GameError> {
        let clip = self.clips.get(clip_id)
            .ok_or_else(|| GameError::Animation(format!("未找到动画剪辑: {}", clip_id)))?;
        
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        
        let instance = AnimationInstance {
            id: instance_id,
            clip_id: clip_id.to_string(),
            state: AnimationState::Ready,
            current_time: 0.0,
            playback_speed: 1.0,
            weight: 1.0,
            start_time: 0.0,
            end_time: clip.duration,
            loop_count: 0,
            current_loop: 0,
            reverse: false,
            events_fired: Vec::new(),
        };
        
        // 应用参数
        if let Some(params) = parameters {
            // 根据参数调整动画实例
            if let Some(&speed) = params.get("speed") {
                // instance.playback_speed = speed;
            }
            if let Some(&weight) = params.get("weight") {
                // instance.weight = weight;
            }
        }
        
        self.active_instances.insert(instance_id, instance);
        self.total_animations_played += 1;
        
        debug!("播放动画: {} 目标: {} ID: {}", clip_id, target, instance_id);
        Ok(instance_id)
    }
    
    // 停止动画
    pub fn stop_animation(&mut self, instance_id: u64) -> Result<(), GameError> {
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            instance.state = AnimationState::Finished;
            debug!("停止动画: ID {}", instance_id);
            Ok(())
        } else {
            Err(GameError::Animation(format!("动画实例不存在: {}", instance_id)))
        }
    }
    
    // 暂停动画
    pub fn pause_animation(&mut self, instance_id: u64) -> Result<(), GameError> {
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            if instance.state == AnimationState::Playing {
                instance.state = AnimationState::Paused;
                debug!("暂停动画: ID {}", instance_id);
            }
            Ok(())
        } else {
            Err(GameError::Animation(format!("动画实例不存在: {}", instance_id)))
        }
    }
    
    // 恢复动画
    pub fn resume_animation(&mut self, instance_id: u64) -> Result<(), GameError> {
        if let Some(instance) = self.active_instances.get_mut(&instance_id) {
            if instance.state == AnimationState::Paused {
                instance.state = AnimationState::Playing;
                debug!("恢复动画: ID {}", instance_id);
            }
            Ok(())
        } else {
            Err(GameError::Animation(format!("动画实例不存在: {}", instance_id)))
        }
    }
    
    // 队列动画播放
    pub fn queue_animation(
        &mut self,
        clip_id: String,
        target: String,
        delay: f32,
        priority: i32,
    ) {
        let queued = QueuedAnimation {
            clip_id,
            target,
            delay,
            parameters: HashMap::new(),
            priority,
        };
        
        // 按优先级插入
        let insert_pos = self.animation_queue
            .iter()
            .position(|q| q.priority < priority)
            .unwrap_or(self.animation_queue.len());
        
        self.animation_queue.insert(insert_pos, queued);
    }
    
    // 播放技能动画
    pub fn play_move_animation(
        &mut self,
        move_id: MoveId,
        attacker: &str,
        defender: &str,
    ) -> Result<Vec<u64>, GameError> {
        let clip_id = format!("move_{}", move_id);
        let mut animation_ids = Vec::new();
        
        // 攻击者动画
        if let Ok(id) = self.play_animation(&format!("{}_cast", clip_id), attacker, None) {
            animation_ids.push(id);
        }
        
        // 技能特效动画
        if let Ok(id) = self.play_animation(&clip_id, "battlefield", None) {
            animation_ids.push(id);
        }
        
        // 受击者动画
        self.queue_animation(
            format!("{}_hit", clip_id),
            defender.to_string(),
            0.5, // 延迟0.5秒
            100,
        );
        
        Ok(animation_ids)
    }
    
    // 获取动画状态
    pub fn get_animation_state(&self, instance_id: u64) -> Option<AnimationState> {
        self.active_instances.get(&instance_id).map(|i| i.state)
    }
    
    // 是否有动画正在播放
    pub fn is_any_animation_playing(&self) -> bool {
        self.active_instances.values().any(|i| i.state == AnimationState::Playing)
    }
    
    // 等待所有动画完成
    pub fn wait_for_all_animations(&self) -> bool {
        !self.is_any_animation_playing() && self.animation_queue.is_empty()
    }
    
    // 设置动画完成回调
    pub fn set_completion_callback<F>(&mut self, instance_id: u64, callback: F) 
    where 
        F: FnOnce() + Send + 'static 
    {
        self.completion_callbacks.insert(instance_id, Box::new(callback));
    }
    
    // 添加动画剪辑
    pub fn add_clip(&mut self, clip: AnimationClip) {
        debug!("添加动画剪辑: {} (时长: {:.2}s)", clip.name, clip.duration);
        self.clips.insert(clip.id.clone(), clip);
    }
    
    // 创建动画目标
    pub fn create_target(&mut self, name: String, initial_transform: Option<(glam::Vec3, glam::Quat, glam::Vec3)>) {
        let (pos, rot, scale) = initial_transform.unwrap_or_default();
        
        let target = AnimationTarget {
            name: name.clone(),
            position: pos,
            rotation: rot,
            scale: scale,
            color: glam::Vec4::ONE,
            opacity: 1.0,
            custom_properties: HashMap::new(),
            visible: true,
        };
        
        self.targets.insert(name, target);
    }
    
    // 获取目标当前状态
    pub fn get_target_transform(&self, name: &str) -> Option<(glam::Vec3, glam::Quat, glam::Vec3)> {
        self.targets.get(name).map(|t| (t.position, t.rotation, t.scale))
    }
    
    // 设置全局动画速度
    pub fn set_global_speed(&mut self, speed: f32) {
        self.global_speed = speed.max(0.0);
        debug!("设置全局动画速度: {:.2}", speed);
    }
    
    // 设置动画质量
    pub fn set_animation_quality(&mut self, quality: AnimationQuality) {
        self.animation_quality = quality;
        debug!("设置动画质量: {:?}", quality);
    }
    
    // 停止所有动画
    pub fn stop_all_animations(&mut self) {
        for instance in self.active_instances.values_mut() {
            instance.state = AnimationState::Cancelled;
        }
        self.animation_queue.clear();
        debug!("停止所有动画");
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> AnimationStats {
        AnimationStats {
            total_clips: self.clips.len(),
            active_animations: self.active_instances.len(),
            queued_animations: self.animation_queue.len(),
            total_played: self.total_animations_played,
            targets_count: self.targets.len(),
        }
    }
    
    // 私有方法
    fn load_default_clips(&mut self) {
        // 加载默认动画剪辑
        self.create_basic_clips();
        self.create_move_clips();
        self.create_ui_clips();
    }
    
    fn create_basic_clips(&mut self) {
        // 基础待机动画
        let idle_clip = AnimationClip {
            id: "idle".to_string(),
            name: "待机动画".to_string(),
            duration: 2.0,
            tracks: vec![
                AnimationTrack {
                    name: "idle_breathe".to_string(),
                    target: "pokemon".to_string(),
                    property: "scale_y".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            scale: Some(glam::Vec3::new(1.0, 1.0, 1.0)),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 0.5,
                            scale: Some(glam::Vec3::new(1.0, 1.05, 1.0)),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 1.0,
                            scale: Some(glam::Vec3::new(1.0, 1.0, 1.0)),
                            ..Default::default()
                        },
                    ],
                    easing: EasingType::SineInOut,
                    loop_mode: LoopMode::Loop,
                }
            ],
            events: Vec::new(),
            priority: 0,
            blend_mode: BlendMode::Replace,
        };
        
        self.clips.insert("idle".to_string(), idle_clip);
        
        // 攻击动画
        let attack_clip = AnimationClip {
            id: "attack_basic".to_string(),
            name: "基础攻击".to_string(),
            duration: 1.0,
            tracks: vec![
                AnimationTrack {
                    name: "attack_lunge".to_string(),
                    target: "pokemon".to_string(),
                    property: "position_x".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            position: Some(glam::Vec3::new(0.0, 0.0, 0.0)),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 0.2,
                            position: Some(glam::Vec3::new(-20.0, 0.0, 0.0)),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 0.4,
                            position: Some(glam::Vec3::new(30.0, 0.0, 0.0)),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 1.0,
                            position: Some(glam::Vec3::new(0.0, 0.0, 0.0)),
                            ..Default::default()
                        },
                    ],
                    easing: EasingType::QuartOut,
                    loop_mode: LoopMode::None,
                }
            ],
            events: vec![
                AnimationEvent {
                    time: 0.4,
                    event_type: "damage_point".to_string(),
                    parameters: HashMap::new(),
                }
            ],
            priority: 10,
            blend_mode: BlendMode::Override,
        };
        
        self.clips.insert("attack_basic".to_string(), attack_clip);
    }
    
    fn create_move_clips(&mut self) {
        // 这里会创建各种技能动画剪辑
        // 由于篇幅限制，只展示一个示例
        
        let tackle_clip = AnimationClip {
            id: "move_1".to_string(), // 撞击
            name: "撞击动画".to_string(),
            duration: 1.5,
            tracks: Vec::new(), // 实际实现中会有详细的轨道
            events: Vec::new(),
            priority: 20,
            blend_mode: BlendMode::Override,
        };
        
        self.clips.insert("move_1".to_string(), tackle_clip);
    }
    
    fn create_ui_clips(&mut self) {
        // UI动画剪辑
        let fade_in = AnimationClip {
            id: "ui_fade_in".to_string(),
            name: "UI淡入".to_string(),
            duration: 0.3,
            tracks: vec![
                AnimationTrack {
                    name: "fade".to_string(),
                    target: "ui_element".to_string(),
                    property: "opacity".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            opacity: Some(0.0),
                            ..Default::default()
                        },
                        Keyframe {
                            time: 1.0,
                            opacity: Some(1.0),
                            ..Default::default()
                        },
                    ],
                    easing: EasingType::QuadOut,
                    loop_mode: LoopMode::None,
                }
            ],
            events: Vec::new(),
            priority: 5,
            blend_mode: BlendMode::Replace,
        };
        
        self.clips.insert("ui_fade_in".to_string(), fade_in);
    }
    
    fn setup_default_particles(&mut self) {
        // 基础命中特效
        let hit_particles = ParticleConfig {
            max_particles: 50,
            emission_rate: 100.0,
            lifetime: (0.2, 0.5),
            initial_velocity: (
                glam::Vec3::new(-50.0, -50.0, -10.0),
                glam::Vec3::new(50.0, 50.0, 10.0)
            ),
            acceleration: glam::Vec3::new(0.0, -200.0, 0.0),
            size: (2.0, 8.0),
            color_over_time: vec![
                (0.0, glam::Vec4::new(1.0, 1.0, 0.0, 1.0)), // 黄色
                (0.5, glam::Vec4::new(1.0, 0.5, 0.0, 0.8)), // 橙色
                (1.0, glam::Vec4::new(1.0, 0.0, 0.0, 0.0)), // 红色淡出
            ],
            texture: "spark".to_string(),
        };
        
        self.particle_systems.insert("hit_sparks".to_string(), hit_particles);
    }
    
    fn process_animation_queue(&mut self, delta_time: f32) -> Result<(), GameError> {
        let mut to_start = Vec::new();
        
        for (index, queued) in self.animation_queue.iter_mut().enumerate() {
            queued.delay -= delta_time;
            if queued.delay <= 0.0 {
                to_start.push(index);
            }
        }
        
        // 启动延迟完成的动画
        for &index in to_start.iter().rev() {
            let queued = self.animation_queue.remove(index);
            self.play_animation(&queued.clip_id, &queued.target, Some(queued.parameters))?;
        }
        
        Ok(())
    }
    
    fn update_active_animations(&mut self, delta_time: f32) -> Result<(), GameError> {
        for instance in self.active_instances.values_mut() {
            if instance.state != AnimationState::Playing {
                if instance.state == AnimationState::Ready {
                    instance.state = AnimationState::Playing;
                }
                continue;
            }
            
            instance.current_time += delta_time * instance.playback_speed;
            
            if let Some(clip) = self.clips.get(&instance.clip_id) {
                // 检查动画事件
                self.check_animation_events(instance, clip);
                
                // 更新动画值
                self.apply_animation_values(instance, clip)?;
                
                // 检查是否完成
                if instance.current_time >= instance.end_time {
                    if instance.loop_count > 0 || instance.loop_count == -1 {
                        // 循环播放
                        instance.current_time = instance.start_time;
                        instance.current_loop += 1;
                        instance.events_fired.clear();
                        
                        if instance.loop_count > 0 && instance.current_loop >= instance.loop_count {
                            instance.state = AnimationState::Finished;
                        }
                    } else {
                        instance.state = AnimationState::Finished;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn check_animation_events(&mut self, instance: &mut AnimationInstance, clip: &AnimationClip) {
        for (event_index, event) in clip.events.iter().enumerate() {
            if !instance.events_fired.contains(&event_index) 
                && instance.current_time >= event.time {
                
                instance.events_fired.push(event_index);
                debug!("触发动画事件: {} 在时间 {:.2}s", event.event_type, event.time);
                
                // 处理动画事件
                self.handle_animation_event(event);
            }
        }
    }
    
    fn handle_animation_event(&mut self, event: &AnimationEvent) {
        match event.event_type.as_str() {
            "damage_point" => {
                // 伤害判定点
                debug!("伤害判定点触发");
            }
            "particle_burst" => {
                // 粒子爆发
                debug!("粒子爆发触发");
            }
            "sound_effect" => {
                // 音效播放
                if let Some(sound) = event.parameters.get("sound") {
                    debug!("播放音效: {}", sound);
                }
            }
            _ => {
                debug!("未知动画事件: {}", event.event_type);
            }
        }
    }
    
    fn apply_animation_values(&mut self, instance: &AnimationInstance, clip: &AnimationClip) -> Result<(), GameError> {
        let progress = instance.current_time / clip.duration;
        
        for track in &clip.tracks {
            if let Some(target) = self.targets.get_mut(&track.target) {
                // 计算当前值
                let value = self.interpolate_track_value(track, progress)?;
                
                // 应用到目标对象
                match track.property.as_str() {
                    "position" => {
                        if let Some(pos) = value.position {
                            target.position = pos;
                        }
                    }
                    "rotation" => {
                        if let Some(rot) = value.rotation {
                            target.rotation = rot;
                        }
                    }
                    "scale" => {
                        if let Some(scale) = value.scale {
                            target.scale = scale;
                        }
                    }
                    "opacity" => {
                        if let Some(opacity) = value.opacity {
                            target.opacity = opacity;
                        }
                    }
                    _ => {
                        // 自定义属性
                        for (key, value) in value.custom_values {
                            target.custom_properties.insert(key, value);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn interpolate_track_value(&self, track: &AnimationTrack, progress: f32) -> Result<Keyframe, GameError> {
        if track.keyframes.is_empty() {
            return Ok(Keyframe::default());
        }
        
        if track.keyframes.len() == 1 {
            return Ok(track.keyframes[0].clone());
        }
        
        // 找到当前时间点的关键帧区间
        let mut start_keyframe = &track.keyframes[0];
        let mut end_keyframe = &track.keyframes[track.keyframes.len() - 1];
        
        for i in 0..track.keyframes.len() - 1 {
            if progress >= track.keyframes[i].time && progress <= track.keyframes[i + 1].time {
                start_keyframe = &track.keyframes[i];
                end_keyframe = &track.keyframes[i + 1];
                break;
            }
        }
        
        // 计算插值系数
        let time_range = end_keyframe.time - start_keyframe.time;
        let local_progress = if time_range > 0.0 {
            (progress - start_keyframe.time) / time_range
        } else {
            0.0
        };
        
        // 应用缓动函数
        let eased_progress = self.apply_easing(track.easing, local_progress);
        
        // 执行插值
        Ok(self.interpolate_keyframes(start_keyframe, end_keyframe, eased_progress))
    }
    
    fn apply_easing(&self, easing: EasingType, t: f32) -> f32 {
        match easing {
            EasingType::Linear => t,
            EasingType::QuadIn => t * t,
            EasingType::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingType::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingType::SineInOut => 0.5 * (1.0 - (std::f32::consts::PI * t).cos()),
            // 其他缓动函数的实现...
            _ => t, // 默认线性
        }
    }
    
    fn interpolate_keyframes(&self, start: &Keyframe, end: &Keyframe, t: f32) -> Keyframe {
        let mut result = Keyframe::default();
        
        // 位置插值
        if let (Some(start_pos), Some(end_pos)) = (start.position, end.position) {
            result.position = Some(start_pos.lerp(end_pos, t));
        }
        
        // 旋转插值
        if let (Some(start_rot), Some(end_rot)) = (start.rotation, end.rotation) {
            result.rotation = Some(start_rot.slerp(end_rot, t));
        }
        
        // 缩放插值
        if let (Some(start_scale), Some(end_scale)) = (start.scale, end.scale) {
            result.scale = Some(start_scale.lerp(end_scale, t));
        }
        
        // 颜色插值
        if let (Some(start_color), Some(end_color)) = (start.color, end.color) {
            result.color = Some(start_color.lerp(end_color, t));
        }
        
        // 透明度插值
        if let (Some(start_opacity), Some(end_opacity)) = (start.opacity, end.opacity) {
            result.opacity = Some(start_opacity + (end_opacity - start_opacity) * t);
        }
        
        result
    }
    
    fn cleanup_finished_animations(&mut self) {
        let mut completed_ids = Vec::new();
        
        self.active_instances.retain(|&id, instance| {
            if matches!(instance.state, AnimationState::Finished | AnimationState::Cancelled) {
                completed_ids.push(id);
                false
            } else {
                true
            }
        });
        
        // 执行完成回调
        for id in completed_ids {
            if let Some(callback) = self.completion_callbacks.remove(&id) {
                callback();
            }
        }
    }
}

// 动画统计信息
#[derive(Debug, Clone)]
pub struct AnimationStats {
    pub total_clips: usize,
    pub active_animations: usize,
    pub queued_animations: usize,
    pub total_played: u64,
    pub targets_count: usize,
}

// 默认实现
impl Default for Keyframe {
    fn default() -> Self {
        Self {
            time: 0.0,
            position: None,
            rotation: None,
            scale: None,
            color: None,
            opacity: None,
            custom_values: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_manager_creation() {
        let manager = BattleAnimationManager::new();
        assert!(!manager.clips.is_empty());
        assert_eq!(manager.active_instances.len(), 0);
    }
    
    #[test]
    fn test_easing_functions() {
        let manager = BattleAnimationManager::new();
        
        assert_eq!(manager.apply_easing(EasingType::Linear, 0.5), 0.5);
        assert_eq!(manager.apply_easing(EasingType::QuadIn, 0.5), 0.25);
        assert_eq!(manager.apply_easing(EasingType::QuadOut, 0.5), 0.75);
    }
    
    #[test]
    fn test_animation_playback() {
        let mut manager = BattleAnimationManager::new();
        manager.create_target("test_target".to_string(), None);
        
        let animation_id = manager.play_animation("idle", "test_target", None).unwrap();
        assert!(manager.active_instances.contains_key(&animation_id));
        
        let state = manager.get_animation_state(animation_id);
        assert_eq!(state, Some(AnimationState::Ready));
    }
}