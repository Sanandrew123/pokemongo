// 精灵系统
// 开发心理：精灵是2D游戏的核心显示单元，需要高效管理、动画支持、批量渲染
// 设计原则：纹理图集优化、动画状态机、内存池管理、批量绘制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::renderer::{Renderer2D, TextureInfo, BlendMode};
use glam::{Vec2, Vec3, Vec4, Mat4};

// 精灵ID类型
pub type SpriteId = u32;

// 动画ID类型
pub type AnimationId = u32;

// 精灵状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteState {
    Hidden,         // 隐藏
    Visible,        // 可见
    Animated,       // 播放动画
    Paused,         // 暂停
    Destroyed,      // 已销毁
}

// 精灵锚点
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,        // 左上角
    TopCenter,      // 上中
    TopRight,       // 右上角
    CenterLeft,     // 左中
    Center,         // 中心
    CenterRight,    // 右中
    BottomLeft,     // 左下角
    BottomCenter,   // 下中
    BottomRight,    // 右下角
    Custom(Vec2),   // 自定义锚点
}

// 精灵翻转
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteFlip {
    pub horizontal: bool,
    pub vertical: bool,
}

// 纹理区域 (UV坐标)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextureRegion {
    pub u: f32,         // 左上角U坐标
    pub v: f32,         // 左上角V坐标
    pub width: f32,     // 宽度 (UV空间)
    pub height: f32,    // 高度 (UV空间)
}

// 精灵数据
#[derive(Debug, Clone)]
pub struct Sprite {
    pub id: SpriteId,
    pub name: String,
    
    // 变换信息
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,          // 弧度
    pub anchor: Anchor,
    pub flip: SpriteFlip,
    
    // 渲染信息
    pub texture_id: u32,
    pub texture_region: TextureRegion,
    pub color: Vec4,
    pub blend_mode: BlendMode,
    pub depth: f32,             // 深度排序
    pub visible: bool,
    
    // 状态信息
    pub state: SpriteState,
    pub layer: u32,             // 渲染层级
    pub tag: String,            // 标签
    
    // 动画信息
    pub current_animation: Option<AnimationId>,
    pub animation_time: f32,
    pub animation_speed: f32,
    pub loop_animation: bool,
    
    // 物理信息 (简单)
    pub velocity: Vec2,
    pub angular_velocity: f32,
    
    // 生命周期
    pub lifetime: Option<f32>,  // 生存时间
    pub age: f32,               // 当前年龄
    
    // 自定义数据
    pub user_data: HashMap<String, String>,
}

// 精灵动画帧
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub texture_region: TextureRegion,
    pub duration: f32,          // 帧时长
    pub offset: Vec2,           // 位置偏移
    pub color_tint: Vec4,       // 颜色调制
}

// 精灵动画
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    pub id: AnimationId,
    pub name: String,
    pub frames: Vec<AnimationFrame>,
    pub total_duration: f32,
    pub loop_mode: AnimationLoopMode,
    pub events: Vec<AnimationEvent>, // 动画事件
}

// 动画循环模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationLoopMode {
    None,           // 不循环
    Loop,           // 循环
    PingPong,       // 往返
    Reverse,        // 反向播放
}

// 动画事件
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub frame_index: usize,     // 触发帧
    pub event_type: String,     // 事件类型
    pub parameters: HashMap<String, String>, // 事件参数
}

// 纹理图集
#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub texture_id: u32,
    pub width: u32,
    pub height: u32,
    pub regions: HashMap<String, TextureRegion>,
    pub animations: HashMap<String, SpriteAnimation>,
}

// 精灵批次
#[derive(Debug)]
pub struct SpriteBatch {
    pub texture_id: u32,
    pub blend_mode: BlendMode,
    pub layer: u32,
    pub sprites: Vec<SpriteId>,
    pub vertex_count: usize,
    pub last_used_frame: u64,
}

// 精灵管理器
pub struct SpriteManager {
    // 精灵管理
    sprites: HashMap<SpriteId, Sprite>,
    next_sprite_id: SpriteId,
    
    // 动画管理
    animations: HashMap<AnimationId, SpriteAnimation>,
    next_animation_id: AnimationId,
    
    // 图集管理
    atlases: HashMap<String, TextureAtlas>,
    
    // 批次管理
    batches: Vec<SpriteBatch>,
    batch_sprites: HashMap<SpriteId, usize>, // sprite -> batch index
    
    // 渲染层级
    layers: HashMap<u32, Vec<SpriteId>>,
    max_layers: u32,
    
    // 内存池
    sprite_pool: Vec<Sprite>,
    
    // 配置
    max_sprites_per_batch: usize,
    auto_batching: bool,
    frustum_culling: bool,
    
    // 统计
    total_sprites_created: u64,
    total_sprites_destroyed: u64,
    total_animations_played: u64,
    frame_count: u64,
    
    // 缓存
    visible_sprites: Vec<SpriteId>,
    animation_cache: HashMap<String, AnimationId>,
    
    // 回调
    animation_callbacks: HashMap<(SpriteId, String), Box<dyn FnMut(&Sprite) + Send>>,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            next_sprite_id: 1,
            animations: HashMap::new(),
            next_animation_id: 1,
            atlases: HashMap::new(),
            batches: Vec::new(),
            batch_sprites: HashMap::new(),
            layers: HashMap::new(),
            max_layers: 100,
            sprite_pool: Vec::new(),
            max_sprites_per_batch: 1000,
            auto_batching: true,
            frustum_culling: true,
            total_sprites_created: 0,
            total_sprites_destroyed: 0,
            total_animations_played: 0,
            frame_count: 0,
            visible_sprites: Vec::new(),
            animation_cache: HashMap::new(),
            animation_callbacks: HashMap::new(),
        }
    }
    
    // 创建精灵
    pub fn create_sprite(
        &mut self,
        name: String,
        texture_id: u32,
        position: Vec2,
    ) -> Result<SpriteId, GameError> {
        let sprite_id = self.next_sprite_id;
        self.next_sprite_id += 1;
        
        let sprite = if let Some(mut pooled_sprite) = self.sprite_pool.pop() {
            // 重用池中的精灵
            pooled_sprite.id = sprite_id;
            pooled_sprite.name = name;
            pooled_sprite.position = position;
            pooled_sprite.texture_id = texture_id;
            pooled_sprite.reset_to_defaults();
            pooled_sprite
        } else {
            // 创建新精灵
            Sprite {
                id: sprite_id,
                name: name.clone(),
                position,
                scale: Vec2::ONE,
                rotation: 0.0,
                anchor: Anchor::Center,
                flip: SpriteFlip { horizontal: false, vertical: false },
                texture_id,
                texture_region: TextureRegion { u: 0.0, v: 0.0, width: 1.0, height: 1.0 },
                color: Vec4::ONE,
                blend_mode: BlendMode::Alpha,
                depth: 0.0,
                visible: true,
                state: SpriteState::Visible,
                layer: 0,
                tag: String::new(),
                current_animation: None,
                animation_time: 0.0,
                animation_speed: 1.0,
                loop_animation: true,
                velocity: Vec2::ZERO,
                angular_velocity: 0.0,
                lifetime: None,
                age: 0.0,
                user_data: HashMap::new(),
            }
        };
        
        self.sprites.insert(sprite_id, sprite);
        self.total_sprites_created += 1;
        
        // 添加到渲染层级
        self.add_to_layer(sprite_id, 0);
        
        debug!("创建精灵: '{}' ID={}", name, sprite_id);
        Ok(sprite_id)
    }
    
    // 销毁精灵
    pub fn destroy_sprite(&mut self, sprite_id: SpriteId) -> Result<(), GameError> {
        if let Some(mut sprite) = self.sprites.remove(&sprite_id) {
            // 从渲染层级中移除
            self.remove_from_layer(sprite_id);
            
            // 从批次中移除
            self.remove_from_batch(sprite_id);
            
            // 清理并回收到池中
            sprite.state = SpriteState::Destroyed;
            sprite.user_data.clear();
            if self.sprite_pool.len() < 100 { // 限制池大小
                self.sprite_pool.push(sprite);
            }
            
            self.total_sprites_destroyed += 1;
            debug!("销毁精灵: ID={}", sprite_id);
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 获取精灵
    pub fn get_sprite(&self, sprite_id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(&sprite_id)
    }
    
    // 获取可变精灵
    pub fn get_sprite_mut(&mut self, sprite_id: SpriteId) -> Option<&mut Sprite> {
        self.sprites.get_mut(&sprite_id)
    }
    
    // 设置精灵位置
    pub fn set_sprite_position(&mut self, sprite_id: SpriteId, position: Vec2) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.position = position;
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 设置精灵缩放
    pub fn set_sprite_scale(&mut self, sprite_id: SpriteId, scale: Vec2) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.scale = scale;
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 设置精灵旋转
    pub fn set_sprite_rotation(&mut self, sprite_id: SpriteId, rotation: f32) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.rotation = rotation;
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 设置精灵颜色
    pub fn set_sprite_color(&mut self, sprite_id: SpriteId, color: Vec4) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.color = color;
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 设置精灵可见性
    pub fn set_sprite_visible(&mut self, sprite_id: SpriteId, visible: bool) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            if sprite.visible != visible {
                sprite.visible = visible;
                sprite.state = if visible { SpriteState::Visible } else { SpriteState::Hidden };
            }
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 设置精灵层级
    pub fn set_sprite_layer(&mut self, sprite_id: SpriteId, layer: u32) -> Result<(), GameError> {
        if layer >= self.max_layers {
            return Err(GameError::Sprite(format!("层级超出范围: {}", layer)));
        }
        
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            let old_layer = sprite.layer;
            sprite.layer = layer;
            
            // 更新层级映射
            self.remove_from_layer_index(sprite_id, old_layer);
            self.add_to_layer(sprite_id, layer);
            
            // 标记需要重新批次
            self.remove_from_batch(sprite_id);
            
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 播放动画
    pub fn play_animation(
        &mut self,
        sprite_id: SpriteId,
        animation_name: &str,
        loop_animation: bool,
    ) -> Result<(), GameError> {
        let animation_id = self.find_animation_by_name(animation_name)
            .ok_or_else(|| GameError::Sprite(format!("动画不存在: {}", animation_name)))?;
        
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.current_animation = Some(animation_id);
            sprite.animation_time = 0.0;
            sprite.loop_animation = loop_animation;
            sprite.state = SpriteState::Animated;
            
            self.total_animations_played += 1;
            debug!("播放动画: 精灵={} 动画='{}' 循环={}", sprite_id, animation_name, loop_animation);
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 停止动画
    pub fn stop_animation(&mut self, sprite_id: SpriteId) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.current_animation = None;
            sprite.animation_time = 0.0;
            sprite.state = if sprite.visible { SpriteState::Visible } else { SpriteState::Hidden };
            debug!("停止动画: 精灵={}", sprite_id);
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 暂停动画
    pub fn pause_animation(&mut self, sprite_id: SpriteId) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            if sprite.state == SpriteState::Animated {
                sprite.state = SpriteState::Paused;
                debug!("暂停动画: 精灵={}", sprite_id);
            }
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 恢复动画
    pub fn resume_animation(&mut self, sprite_id: SpriteId) -> Result<(), GameError> {
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            if sprite.state == SpriteState::Paused && sprite.current_animation.is_some() {
                sprite.state = SpriteState::Animated;
                debug!("恢复动画: 精灵={}", sprite_id);
            }
            Ok(())
        } else {
            Err(GameError::Sprite(format!("精灵不存在: {}", sprite_id)))
        }
    }
    
    // 创建动画
    pub fn create_animation(
        &mut self,
        name: String,
        frames: Vec<AnimationFrame>,
        loop_mode: AnimationLoopMode,
    ) -> Result<AnimationId, GameError> {
        let animation_id = self.next_animation_id;
        self.next_animation_id += 1;
        
        let total_duration = frames.iter().map(|f| f.duration).sum();
        
        let animation = SpriteAnimation {
            id: animation_id,
            name: name.clone(),
            frames,
            total_duration,
            loop_mode,
            events: Vec::new(),
        };
        
        self.animations.insert(animation_id, animation);
        self.animation_cache.insert(name.clone(), animation_id);
        
        debug!("创建动画: '{}' ID={} 时长={:.2}s", name, animation_id, total_duration);
        Ok(animation_id)
    }
    
    // 加载纹理图集
    pub fn load_texture_atlas(
        &mut self,
        name: String,
        texture_id: u32,
        width: u32,
        height: u32,
    ) -> Result<(), GameError> {
        let atlas = TextureAtlas {
            texture_id,
            width,
            height,
            regions: HashMap::new(),
            animations: HashMap::new(),
        };
        
        self.atlases.insert(name.clone(), atlas);
        debug!("加载纹理图集: '{}' 纹理ID={} 尺寸={}x{}", name, texture_id, width, height);
        Ok(())
    }
    
    // 添加图集区域
    pub fn add_atlas_region(
        &mut self,
        atlas_name: &str,
        region_name: String,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), GameError> {
        if let Some(atlas) = self.atlases.get_mut(atlas_name) {
            let region = TextureRegion {
                u: x as f32 / atlas.width as f32,
                v: y as f32 / atlas.height as f32,
                width: width as f32 / atlas.width as f32,
                height: height as f32 / atlas.height as f32,
            };
            
            atlas.regions.insert(region_name.clone(), region);
            debug!("添加图集区域: '{}' 区域='{}' UV=({:.3}, {:.3}, {:.3}, {:.3})",
                atlas_name, region_name, region.u, region.v, region.width, region.height);
            Ok(())
        } else {
            Err(GameError::Sprite(format!("图集不存在: {}", atlas_name)))
        }
    }
    
    // 从图集创建精灵
    pub fn create_sprite_from_atlas(
        &mut self,
        name: String,
        atlas_name: &str,
        region_name: &str,
        position: Vec2,
    ) -> Result<SpriteId, GameError> {
        let (texture_id, region) = {
            let atlas = self.atlases.get(atlas_name)
                .ok_or_else(|| GameError::Sprite(format!("图集不存在: {}", atlas_name)))?;
            
            let region = atlas.regions.get(region_name)
                .ok_or_else(|| GameError::Sprite(format!("图集区域不存在: {}", region_name)))?;
            
            (atlas.texture_id, *region)
        };
        
        let sprite_id = self.create_sprite(name, texture_id, position)?;
        
        if let Some(sprite) = self.sprites.get_mut(&sprite_id) {
            sprite.texture_region = region;
        }
        
        Ok(sprite_id)
    }
    
    // 更新精灵系统
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        self.frame_count += 1;
        
        // 更新动画
        self.update_animations(delta_time)?;
        
        // 更新物理
        self.update_physics(delta_time);
        
        // 更新生命周期
        self.update_lifetimes(delta_time);
        
        // 更新批次
        if self.auto_batching {
            self.update_batches()?;
        }
        
        Ok(())
    }
    
    // 渲染所有精灵
    pub fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 收集可见精灵
        self.collect_visible_sprites();
        
        // 按层级和深度排序
        self.sort_sprites_for_rendering();
        
        // 批量渲染
        if self.auto_batching {
            self.render_batched(renderer)?;
        } else {
            self.render_immediate(renderer)?;
        }
        
        Ok(())
    }
    
    // 查找精灵
    pub fn find_sprites_by_tag(&self, tag: &str) -> Vec<SpriteId> {
        self.sprites
            .iter()
            .filter(|(_, sprite)| sprite.tag == tag)
            .map(|(&id, _)| id)
            .collect()
    }
    
    // 查找精灵
    pub fn find_sprites_by_name(&self, name: &str) -> Vec<SpriteId> {
        self.sprites
            .iter()
            .filter(|(_, sprite)| sprite.name == name)
            .map(|(&id, _)| id)
            .collect()
    }
    
    // 查找精灵
    pub fn find_sprites_in_layer(&self, layer: u32) -> Vec<SpriteId> {
        self.layers.get(&layer).cloned().unwrap_or_default()
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> SpriteStats {
        SpriteStats {
            total_sprites: self.sprites.len(),
            visible_sprites: self.visible_sprites.len(),
            animated_sprites: self.sprites.values()
                .filter(|s| s.state == SpriteState::Animated)
                .count(),
            total_animations: self.animations.len(),
            total_atlases: self.atlases.len(),
            batches_count: self.batches.len(),
            sprites_created: self.total_sprites_created,
            sprites_destroyed: self.total_sprites_destroyed,
            animations_played: self.total_animations_played,
            memory_usage: self.calculate_memory_usage(),
            pooled_sprites: self.sprite_pool.len(),
        }
    }
    
    // 清空所有精灵
    pub fn clear_all(&mut self) {
        self.sprites.clear();
        self.batches.clear();
        self.batch_sprites.clear();
        self.layers.clear();
        self.visible_sprites.clear();
        self.animation_callbacks.clear();
        debug!("清空所有精灵");
    }
    
    // 私有方法
    fn add_to_layer(&mut self, sprite_id: SpriteId, layer: u32) {
        self.layers.entry(layer).or_insert_with(Vec::new).push(sprite_id);
    }
    
    fn remove_from_layer(&mut self, sprite_id: SpriteId) {
        if let Some(sprite) = self.sprites.get(&sprite_id) {
            self.remove_from_layer_index(sprite_id, sprite.layer);
        }
    }
    
    fn remove_from_layer_index(&mut self, sprite_id: SpriteId, layer: u32) {
        if let Some(layer_sprites) = self.layers.get_mut(&layer) {
            layer_sprites.retain(|&id| id != sprite_id);
        }
    }
    
    fn remove_from_batch(&mut self, sprite_id: SpriteId) {
        if let Some(batch_index) = self.batch_sprites.remove(&sprite_id) {
            if let Some(batch) = self.batches.get_mut(batch_index) {
                batch.sprites.retain(|&id| id != sprite_id);
            }
        }
    }
    
    fn find_animation_by_name(&self, name: &str) -> Option<AnimationId> {
        self.animation_cache.get(name).copied()
    }
    
    fn update_animations(&mut self, delta_time: f32) -> Result<(), GameError> {
        let mut events_to_trigger = Vec::new();
        
        for sprite in self.sprites.values_mut() {
            if sprite.state != SpriteState::Animated {
                continue;
            }
            
            if let Some(animation_id) = sprite.current_animation {
                if let Some(animation) = self.animations.get(&animation_id) {
                    let old_time = sprite.animation_time;
                    sprite.animation_time += delta_time * sprite.animation_speed;
                    
                    // 处理循环
                    let mut animation_finished = false;
                    
                    match animation.loop_mode {
                        AnimationLoopMode::None => {
                            if sprite.animation_time >= animation.total_duration {
                                sprite.animation_time = animation.total_duration;
                                animation_finished = true;
                            }
                        }
                        AnimationLoopMode::Loop => {
                            if sprite.animation_time >= animation.total_duration {
                                sprite.animation_time %= animation.total_duration;
                            }
                        }
                        AnimationLoopMode::PingPong => {
                            let cycle_duration = animation.total_duration * 2.0;
                            sprite.animation_time %= cycle_duration;
                            
                            if sprite.animation_time > animation.total_duration {
                                sprite.animation_time = cycle_duration - sprite.animation_time;
                            }
                        }
                        AnimationLoopMode::Reverse => {
                            sprite.animation_time %= animation.total_duration;
                            sprite.animation_time = animation.total_duration - sprite.animation_time;
                        }
                    }
                    
                    // 更新纹理区域
                    if let Some(frame) = self.get_animation_frame(animation, sprite.animation_time) {
                        sprite.texture_region = frame.texture_region;
                    }
                    
                    // 检查动画事件
                    for event in &animation.events {
                        let event_frame_time = self.get_frame_time(animation, event.frame_index);
                        if old_time < event_frame_time && sprite.animation_time >= event_frame_time {
                            events_to_trigger.push((sprite.id, event.clone()));
                        }
                    }
                    
                    // 处理动画结束
                    if animation_finished {
                        sprite.state = SpriteState::Visible;
                        sprite.current_animation = None;
                    }
                }
            }
        }
        
        // 触发动画事件
        for (sprite_id, event) in events_to_trigger {
            self.trigger_animation_event(sprite_id, &event);
        }
        
        Ok(())
    }
    
    fn update_physics(&mut self, delta_time: f32) {
        for sprite in self.sprites.values_mut() {
            if sprite.velocity != Vec2::ZERO {
                sprite.position += sprite.velocity * delta_time;
            }
            
            if sprite.angular_velocity != 0.0 {
                sprite.rotation += sprite.angular_velocity * delta_time;
            }
        }
    }
    
    fn update_lifetimes(&mut self, delta_time: f32) {
        let mut sprites_to_destroy = Vec::new();
        
        for sprite in self.sprites.values_mut() {
            sprite.age += delta_time;
            
            if let Some(lifetime) = sprite.lifetime {
                if sprite.age >= lifetime {
                    sprites_to_destroy.push(sprite.id);
                }
            }
        }
        
        for sprite_id in sprites_to_destroy {
            self.destroy_sprite(sprite_id).ok();
        }
    }
    
    fn update_batches(&mut self) -> Result<(), GameError> {
        // 清空现有批次
        self.batches.clear();
        self.batch_sprites.clear();
        
        // 按纹理和混合模式分组
        let mut texture_groups: HashMap<(u32, BlendMode, u32), Vec<SpriteId>> = HashMap::new();
        
        for (&sprite_id, sprite) in &self.sprites {
            if !sprite.visible || sprite.state == SpriteState::Hidden {
                continue;
            }
            
            let key = (sprite.texture_id, sprite.blend_mode, sprite.layer);
            texture_groups.entry(key).or_insert_with(Vec::new).push(sprite_id);
        }
        
        // 创建批次
        for ((texture_id, blend_mode, layer), sprite_ids) in texture_groups {
            if sprite_ids.is_empty() {
                continue;
            }
            
            let batch = SpriteBatch {
                texture_id,
                blend_mode,
                layer,
                sprites: sprite_ids.clone(),
                vertex_count: sprite_ids.len() * 4, // 每个精灵4个顶点
                last_used_frame: self.frame_count,
            };
            
            let batch_index = self.batches.len();
            self.batches.push(batch);
            
            // 记录精灵到批次的映射
            for sprite_id in sprite_ids {
                self.batch_sprites.insert(sprite_id, batch_index);
            }
        }
        
        debug!("更新批次: {} 个批次", self.batches.len());
        Ok(())
    }
    
    fn collect_visible_sprites(&mut self) {
        self.visible_sprites.clear();
        
        for (&sprite_id, sprite) in &self.sprites {
            if sprite.visible && sprite.state != SpriteState::Hidden && sprite.state != SpriteState::Destroyed {
                self.visible_sprites.push(sprite_id);
            }
        }
    }
    
    fn sort_sprites_for_rendering(&mut self) {
        self.visible_sprites.sort_by(|&a, &b| {
            let sprite_a = &self.sprites[&a];
            let sprite_b = &self.sprites[&b];
            
            // 首先按层级排序
            let layer_cmp = sprite_a.layer.cmp(&sprite_b.layer);
            if layer_cmp != std::cmp::Ordering::Equal {
                return layer_cmp;
            }
            
            // 然后按深度排序
            sprite_a.depth.partial_cmp(&sprite_b.depth).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    fn render_batched(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        for batch in &self.batches {
            self.render_sprite_batch(renderer, batch)?;
        }
        Ok(())
    }
    
    fn render_immediate(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        for &sprite_id in &self.visible_sprites {
            if let Some(sprite) = self.sprites.get(&sprite_id) {
                self.render_single_sprite(renderer, sprite)?;
            }
        }
        Ok(())
    }
    
    fn render_sprite_batch(&self, renderer: &mut Renderer2D, batch: &SpriteBatch) -> Result<(), GameError> {
        debug!("渲染精灵批次: 纹理={} 精灵数={} 层级={}", 
            batch.texture_id, batch.sprites.len(), batch.layer);
        Ok(())
    }
    
    fn render_single_sprite(&self, renderer: &mut Renderer2D, sprite: &Sprite) -> Result<(), GameError> {
        if !sprite.visible {
            return Ok(());
        }
        
        // 计算实际渲染尺寸
        let size = self.calculate_sprite_size(sprite);
        let position = self.calculate_sprite_position(sprite, size);
        
        // 计算源区域
        let source_rect = Some((
            Vec2::new(sprite.texture_region.u, sprite.texture_region.v),
            Vec2::new(sprite.texture_region.width, sprite.texture_region.height),
        ));
        
        renderer.draw_sprite(
            position,
            size,
            sprite.texture_id,
            source_rect,
            sprite.color,
            sprite.rotation,
            sprite.flip.horizontal,
            sprite.flip.vertical,
        )?;
        
        Ok(())
    }
    
    fn calculate_sprite_size(&self, sprite: &Sprite) -> Vec2 {
        // 简化实现：假设纹理大小为基础大小
        let base_size = Vec2::new(
            sprite.texture_region.width * 100.0, // 假设基础大小
            sprite.texture_region.height * 100.0,
        );
        base_size * sprite.scale
    }
    
    fn calculate_sprite_position(&self, sprite: &Sprite, size: Vec2) -> Vec2 {
        match sprite.anchor {
            Anchor::TopLeft => sprite.position,
            Anchor::TopCenter => sprite.position - Vec2::new(size.x * 0.5, 0.0),
            Anchor::TopRight => sprite.position - Vec2::new(size.x, 0.0),
            Anchor::CenterLeft => sprite.position - Vec2::new(0.0, size.y * 0.5),
            Anchor::Center => sprite.position - size * 0.5,
            Anchor::CenterRight => sprite.position - Vec2::new(size.x, size.y * 0.5),
            Anchor::BottomLeft => sprite.position - Vec2::new(0.0, size.y),
            Anchor::BottomCenter => sprite.position - Vec2::new(size.x * 0.5, size.y),
            Anchor::BottomRight => sprite.position - size,
            Anchor::Custom(offset) => sprite.position - size * offset,
        }
    }
    
    fn get_animation_frame(&self, animation: &SpriteAnimation, time: f32) -> Option<&AnimationFrame> {
        let mut current_time = 0.0;
        for frame in &animation.frames {
            current_time += frame.duration;
            if time <= current_time {
                return Some(frame);
            }
        }
        animation.frames.last()
    }
    
    fn get_frame_time(&self, animation: &SpriteAnimation, frame_index: usize) -> f32 {
        let mut time = 0.0;
        for i in 0..=frame_index.min(animation.frames.len() - 1) {
            time += animation.frames[i].duration;
        }
        time
    }
    
    fn trigger_animation_event(&mut self, sprite_id: SpriteId, event: &AnimationEvent) {
        debug!("触发动画事件: 精灵={} 事件={} 帧={}", 
            sprite_id, event.event_type, event.frame_index);
        
        // 触发回调
        let callback_key = (sprite_id, event.event_type.clone());
        if let Some(callback) = self.animation_callbacks.get_mut(&callback_key) {
            if let Some(sprite) = self.sprites.get(&sprite_id) {
                callback(sprite);
            }
        }
    }
    
    fn calculate_memory_usage(&self) -> usize {
        let mut total = 0;
        
        // 精灵数据
        total += self.sprites.len() * std::mem::size_of::<Sprite>();
        
        // 动画数据
        for animation in self.animations.values() {
            total += std::mem::size_of::<SpriteAnimation>();
            total += animation.frames.len() * std::mem::size_of::<AnimationFrame>();
            total += animation.events.len() * std::mem::size_of::<AnimationEvent>();
        }
        
        // 图集数据
        for atlas in self.atlases.values() {
            total += std::mem::size_of::<TextureAtlas>();
            total += atlas.regions.len() * (std::mem::size_of::<String>() + std::mem::size_of::<TextureRegion>());
        }
        
        // 批次数据
        total += self.batches.len() * std::mem::size_of::<SpriteBatch>();
        
        total
    }
}

impl Sprite {
    fn reset_to_defaults(&mut self) {
        self.position = Vec2::ZERO;
        self.scale = Vec2::ONE;
        self.rotation = 0.0;
        self.anchor = Anchor::Center;
        self.flip = SpriteFlip { horizontal: false, vertical: false };
        self.texture_region = TextureRegion { u: 0.0, v: 0.0, width: 1.0, height: 1.0 };
        self.color = Vec4::ONE;
        self.blend_mode = BlendMode::Alpha;
        self.depth = 0.0;
        self.visible = true;
        self.state = SpriteState::Visible;
        self.layer = 0;
        self.tag.clear();
        self.current_animation = None;
        self.animation_time = 0.0;
        self.animation_speed = 1.0;
        self.loop_animation = true;
        self.velocity = Vec2::ZERO;
        self.angular_velocity = 0.0;
        self.lifetime = None;
        self.age = 0.0;
        self.user_data.clear();
    }
}

// 精灵统计信息
#[derive(Debug, Clone)]
pub struct SpriteStats {
    pub total_sprites: usize,
    pub visible_sprites: usize,
    pub animated_sprites: usize,
    pub total_animations: usize,
    pub total_atlases: usize,
    pub batches_count: usize,
    pub sprites_created: u64,
    pub sprites_destroyed: u64,
    pub animations_played: u64,
    pub memory_usage: usize,
    pub pooled_sprites: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_manager_creation() {
        let manager = SpriteManager::new();
        assert_eq!(manager.sprites.len(), 0);
        assert_eq!(manager.animations.len(), 0);
        assert_eq!(manager.next_sprite_id, 1);
    }
    
    #[test]
    fn test_sprite_creation() {
        let mut manager = SpriteManager::new();
        
        let sprite_id = manager.create_sprite(
            "test_sprite".to_string(),
            1,
            Vec2::new(100.0, 200.0),
        ).unwrap();
        
        assert_eq!(sprite_id, 1);
        assert_eq!(manager.sprites.len(), 1);
        
        let sprite = manager.get_sprite(sprite_id).unwrap();
        assert_eq!(sprite.name, "test_sprite");
        assert_eq!(sprite.position, Vec2::new(100.0, 200.0));
        assert_eq!(sprite.texture_id, 1);
    }
    
    #[test]
    fn test_sprite_properties() {
        let mut manager = SpriteManager::new();
        let sprite_id = manager.create_sprite("test".to_string(), 1, Vec2::ZERO).unwrap();
        
        // 测试位置设置
        manager.set_sprite_position(sprite_id, Vec2::new(50.0, 75.0)).unwrap();
        let sprite = manager.get_sprite(sprite_id).unwrap();
        assert_eq!(sprite.position, Vec2::new(50.0, 75.0));
        
        // 测试缩放设置
        manager.set_sprite_scale(sprite_id, Vec2::new(2.0, 3.0)).unwrap();
        let sprite = manager.get_sprite(sprite_id).unwrap();
        assert_eq!(sprite.scale, Vec2::new(2.0, 3.0));
        
        // 测试旋转设置
        manager.set_sprite_rotation(sprite_id, 1.57).unwrap();
        let sprite = manager.get_sprite(sprite_id).unwrap();
        assert_eq!(sprite.rotation, 1.57);
    }
    
    #[test]
    fn test_sprite_destruction() {
        let mut manager = SpriteManager::new();
        let sprite_id = manager.create_sprite("test".to_string(), 1, Vec2::ZERO).unwrap();
        
        assert_eq!(manager.sprites.len(), 1);
        
        manager.destroy_sprite(sprite_id).unwrap();
        assert_eq!(manager.sprites.len(), 0);
        assert!(manager.get_sprite(sprite_id).is_none());
    }
    
    #[test]
    fn test_animation_creation() {
        let mut manager = SpriteManager::new();
        
        let frames = vec![
            AnimationFrame {
                texture_region: TextureRegion { u: 0.0, v: 0.0, width: 0.25, height: 1.0 },
                duration: 0.1,
                offset: Vec2::ZERO,
                color_tint: Vec4::ONE,
            },
            AnimationFrame {
                texture_region: TextureRegion { u: 0.25, v: 0.0, width: 0.25, height: 1.0 },
                duration: 0.1,
                offset: Vec2::ZERO,
                color_tint: Vec4::ONE,
            },
        ];
        
        let animation_id = manager.create_animation(
            "walk".to_string(),
            frames,
            AnimationLoopMode::Loop,
        ).unwrap();
        
        assert_eq!(animation_id, 1);
        assert_eq!(manager.animations.len(), 1);
        
        let animation = manager.animations.get(&animation_id).unwrap();
        assert_eq!(animation.name, "walk");
        assert_eq!(animation.frames.len(), 2);
        assert_eq!(animation.total_duration, 0.2);
    }
    
    #[test]
    fn test_texture_atlas() {
        let mut manager = SpriteManager::new();
        
        manager.load_texture_atlas("test_atlas".to_string(), 1, 256, 256).unwrap();
        
        manager.add_atlas_region(
            "test_atlas",
            "sprite1".to_string(),
            0, 0, 64, 64,
        ).unwrap();
        
        let atlas = manager.atlases.get("test_atlas").unwrap();
        assert_eq!(atlas.regions.len(), 1);
        
        let region = atlas.regions.get("sprite1").unwrap();
        assert_eq!(region.u, 0.0);
        assert_eq!(region.v, 0.0);
        assert_eq!(region.width, 0.25); // 64/256
        assert_eq!(region.height, 0.25); // 64/256
    }
}