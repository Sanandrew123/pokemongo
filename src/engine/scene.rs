/*
 * 场景管理系统 - Scene Manager System
 * 
 * 开发心理过程：
 * 设计灵活的场景管理系统，支持场景切换、层级管理、实体生命周期等
 * 需要考虑场景加载优化、内存管理和状态持久化
 * 重点关注场景间的平滑过渡和数据隔离
 */

use bevy::prelude::*;
use std::collections::HashMap;
use crate::core::error::{GameResult, GameError};
use crate::core::math::{Vec2, Vec3};

// 场景类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneType {
    MainMenu,       // 主菜单
    Overworld,      // 大世界
    Battle,         // 战斗场景
    Town,           // 城镇
    Route,          // 路线
    Building,       // 建筑内部
    Cave,           // 洞穴
    Gym,            // 道馆
    PokemonCenter,  // 宝可梦中心
    Shop,           // 商店
    Dialog,         // 对话场景
    Inventory,      // 背包场景
    Settings,       // 设置场景
}

// 场景状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneState {
    Unloaded,       // 未加载
    Loading,        // 正在加载
    Active,         // 活跃状态
    Paused,         // 暂停状态
    Unloading,      // 正在卸载
    Background,     // 后台运行
}

// 场景层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SceneLayer {
    Background = 0,  // 背景层
    World = 1,       // 世界层
    Objects = 2,     // 物体层
    Characters = 3,  // 角色层
    Effects = 4,     // 特效层
    UI = 5,         // UI层
    Overlay = 6,     // 覆盖层
    Debug = 7,       // 调试层
}

// 场景过渡类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneTransition {
    None,           // 无过渡
    Fade,           // 淡入淡出
    Slide,          // 滑动
    Zoom,           // 缩放
    Dissolve,       // 溶解
    Wipe,           // 擦除
    Custom(u32),    // 自定义过渡
}

// 场景实体
#[derive(Debug, Component)]
pub struct SceneEntity {
    pub scene_id: String,
    pub layer: SceneLayer,
    pub persistent: bool,
    pub auto_cleanup: bool,
}

// 场景数据
#[derive(Debug, Clone)]
pub struct SceneData {
    pub metadata: HashMap<String, String>,
    pub variables: HashMap<String, f32>,
    pub flags: HashMap<String, bool>,
    pub spawn_points: HashMap<String, Vec3>,
    pub boundaries: Vec<(Vec2, Vec2)>,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            variables: HashMap::new(),
            flags: HashMap::new(),
            spawn_points: HashMap::new(),
            boundaries: Vec::new(),
        }
    }
}

// 场景配置
#[derive(Debug, Clone)]
pub struct SceneConfig {
    pub id: String,
    pub scene_type: SceneType,
    pub name: String,
    pub description: String,
    pub resource_path: String,
    pub preload_resources: Vec<String>,
    pub dependencies: Vec<String>,
    pub max_entities: usize,
    pub enable_physics: bool,
    pub enable_audio: bool,
    pub ambient_light: Color,
    pub gravity: Vec3,
    pub time_scale: f32,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            scene_type: SceneType::Overworld,
            name: String::new(),
            description: String::new(),
            resource_path: String::new(),
            preload_resources: Vec::new(),
            dependencies: Vec::new(),
            max_entities: 1000,
            enable_physics: true,
            enable_audio: true,
            ambient_light: Color::WHITE,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_scale: 1.0,
        }
    }
}

// 场景实例
#[derive(Debug)]
pub struct Scene {
    pub config: SceneConfig,
    pub state: SceneState,
    pub data: SceneData,
    pub entities: Vec<Entity>,
    pub layer_entities: HashMap<SceneLayer, Vec<Entity>>,
    pub systems: Vec<Box<dyn System<In = (), Out = ()>>>,
    pub last_update: f64,
    pub load_time: f64,
    pub entity_count: usize,
    pub memory_usage: usize,
}

impl Scene {
    pub fn new(config: SceneConfig) -> Self {
        Self {
            config,
            state: SceneState::Unloaded,
            data: SceneData::default(),
            entities: Vec::new(),
            layer_entities: HashMap::new(),
            systems: Vec::new(),
            last_update: 0.0,
            load_time: 0.0,
            entity_count: 0,
            memory_usage: 0,
        }
    }

    // 添加实体到场景
    pub fn add_entity(&mut self, entity: Entity, layer: SceneLayer) {
        self.entities.push(entity);
        self.layer_entities
            .entry(layer)
            .or_insert_with(Vec::new)
            .push(entity);
        self.entity_count += 1;
    }

    // 从场景移除实体
    pub fn remove_entity(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
        for entities in self.layer_entities.values_mut() {
            entities.retain(|&e| e != entity);
        }
        self.entity_count = self.entity_count.saturating_sub(1);
    }

    // 获取指定层的实体
    pub fn get_entities_by_layer(&self, layer: SceneLayer) -> Option<&Vec<Entity>> {
        self.layer_entities.get(&layer)
    }

    // 设置场景变量
    pub fn set_variable(&mut self, key: String, value: f32) {
        self.data.variables.insert(key, value);
    }

    // 获取场景变量
    pub fn get_variable(&self, key: &str) -> Option<f32> {
        self.data.variables.get(key).copied()
    }

    // 设置场景标志
    pub fn set_flag(&mut self, key: String, value: bool) {
        self.data.flags.insert(key, value);
    }

    // 获取场景标志
    pub fn get_flag(&self, key: &str) -> bool {
        self.data.flags.get(key).copied().unwrap_or(false)
    }

    // 添加生成点
    pub fn add_spawn_point(&mut self, name: String, position: Vec3) {
        self.data.spawn_points.insert(name, position);
    }

    // 获取生成点
    pub fn get_spawn_point(&self, name: &str) -> Option<Vec3> {
        self.data.spawn_points.get(name).copied()
    }
}

// 场景切换请求
#[derive(Debug, Clone)]
pub struct SceneTransitionRequest {
    pub from_scene: Option<String>,
    pub to_scene: String,
    pub transition_type: SceneTransition,
    pub duration: f32,
    pub preserve_entities: Vec<Entity>,
    pub data_transfer: HashMap<String, String>,
}

// 场景事件
#[derive(Debug, Clone)]
pub enum SceneEvent {
    SceneLoadStarted(String),
    SceneLoadCompleted(String),
    SceneLoadFailed(String, String),
    SceneUnloaded(String),
    SceneActivated(String),
    SceneDeactivated(String),
    TransitionStarted(String, String),
    TransitionCompleted(String, String),
    EntitySpawned(String, Entity),
    EntityDestroyed(String, Entity),
}

// 场景管理器主结构
pub struct SceneManager {
    // 场景注册表
    scene_configs: HashMap<String, SceneConfig>,
    active_scenes: HashMap<String, Scene>,
    
    // 当前状态
    current_scene: Option<String>,
    scene_stack: Vec<String>,
    
    // 过渡管理
    transition_state: Option<SceneTransitionRequest>,
    transition_progress: f32,
    transition_timer: f32,
    
    // 统计信息
    total_scenes: usize,
    loaded_scenes: usize,
    total_entities: usize,
    memory_usage: usize,
}

impl SceneManager {
    // 创建新的场景管理器
    pub fn new() -> GameResult<Self> {
        Ok(Self {
            scene_configs: HashMap::new(),
            active_scenes: HashMap::new(),
            current_scene: None,
            scene_stack: Vec::new(),
            transition_state: None,
            transition_progress: 0.0,
            transition_timer: 0.0,
            total_scenes: 0,
            loaded_scenes: 0,
            total_entities: 0,
            memory_usage: 0,
        })
    }

    // 初始化场景管理器
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化场景管理器...");
        
        // 注册默认场景
        self.register_default_scenes()?;
        
        info!("场景管理器初始化完成");
        Ok(())
    }

    // 关闭场景管理器
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("关闭场景管理器...");
        
        // 卸载所有场景
        self.unload_all_scenes()?;
        
        self.scene_configs.clear();
        self.scene_stack.clear();
        self.current_scene = None;
        self.transition_state = None;
        
        info!("场景管理器已关闭");
        Ok(())
    }

    // 更新场景管理器
    pub fn update(&mut self, delta_time: f32) -> GameResult<()> {
        // 更新场景过渡
        self.update_transition(delta_time)?;
        
        // 更新活跃场景
        self.update_active_scenes(delta_time)?;
        
        // 更新统计信息
        self.update_statistics();
        
        Ok(())
    }

    // 注册场景配置
    pub fn register_scene(&mut self, config: SceneConfig) -> GameResult<()> {
        if self.scene_configs.contains_key(&config.id) {
            return Err(GameError::Scene(format!("场景已注册: {}", config.id)));
        }

        self.scene_configs.insert(config.id.clone(), config);
        self.total_scenes += 1;

        info!("注册场景: {}", self.scene_configs.last_key_value().unwrap().0);
        Ok(())
    }

    // 加载场景
    pub fn load_scene(&mut self, 
        commands: &mut Commands,
        scene_id: &str
    ) -> GameResult<()> {
        
        let config = self.scene_configs.get(scene_id)
            .ok_or_else(|| GameError::Scene(format!("场景配置不存在: {}", scene_id)))?
            .clone();

        if self.active_scenes.contains_key(scene_id) {
            warn!("场景已加载: {}", scene_id);
            return Ok(());
        }

        info!("开始加载场景: {}", scene_id);

        let mut scene = Scene::new(config);
        scene.state = SceneState::Loading;
        scene.load_time = self.get_current_time();

        // 预加载资源
        self.preload_scene_resources(&scene)?;

        // 初始化场景系统
        self.initialize_scene_systems(commands, &mut scene)?;

        // 生成场景实体
        self.spawn_scene_entities(commands, &mut scene)?;

        scene.state = SceneState::Active;
        self.active_scenes.insert(scene_id.to_string(), scene);
        self.loaded_scenes += 1;

        info!("场景加载完成: {}", scene_id);
        Ok(())
    }

    // 卸载场景
    pub fn unload_scene(&mut self, 
        commands: &mut Commands,
        scene_id: &str
    ) -> GameResult<()> {
        
        if let Some(mut scene) = self.active_scenes.remove(scene_id) {
            info!("开始卸载场景: {}", scene_id);
            
            scene.state = SceneState::Unloading;
            
            // 清理场景实体
            self.cleanup_scene_entities(commands, &scene);
            
            // 卸载场景资源
            self.unload_scene_resources(&scene)?;
            
            self.loaded_scenes = self.loaded_scenes.saturating_sub(1);
            info!("场景卸载完成: {}", scene_id);
        }
        
        Ok(())
    }

    // 切换到指定场景
    pub fn switch_to_scene(&mut self, 
        scene_id: &str, 
        transition: SceneTransition,
        duration: f32
    ) -> GameResult<()> {
        
        if !self.scene_configs.contains_key(scene_id) {
            return Err(GameError::Scene(format!("场景配置不存在: {}", scene_id)));
        }

        let transition_request = SceneTransitionRequest {
            from_scene: self.current_scene.clone(),
            to_scene: scene_id.to_string(),
            transition_type: transition,
            duration,
            preserve_entities: Vec::new(),
            data_transfer: HashMap::new(),
        };

        self.transition_state = Some(transition_request);
        self.transition_progress = 0.0;
        self.transition_timer = 0.0;

        info!("开始场景切换: {:?} -> {}", self.current_scene, scene_id);
        Ok(())
    }

    // 推入场景到栈
    pub fn push_scene(&mut self, 
        commands: &mut Commands,
        scene_id: &str
    ) -> GameResult<()> {
        
        // 暂停当前场景
        if let Some(current_id) = &self.current_scene {
            if let Some(scene) = self.active_scenes.get_mut(current_id) {
                scene.state = SceneState::Paused;
            }
            self.scene_stack.push(current_id.clone());
        }

        // 加载并激活新场景
        self.load_scene(commands, scene_id)?;
        self.current_scene = Some(scene_id.to_string());

        info!("推入场景: {} (栈深度: {})", scene_id, self.scene_stack.len());
        Ok(())
    }

    // 从栈中弹出场景
    pub fn pop_scene(&mut self, commands: &mut Commands) -> GameResult<Option<String>> {
        if let Some(current_id) = &self.current_scene {
            // 卸载当前场景
            self.unload_scene(commands, current_id)?;
        }

        // 恢复栈顶场景
        if let Some(previous_id) = self.scene_stack.pop() {
            if let Some(scene) = self.active_scenes.get_mut(&previous_id) {
                scene.state = SceneState::Active;
            }
            self.current_scene = Some(previous_id.clone());
            
            info!("弹出场景: {} (栈深度: {})", previous_id, self.scene_stack.len());
            Ok(Some(previous_id))
        } else {
            self.current_scene = None;
            Ok(None)
        }
    }

    // 获取当前场景
    pub fn get_current_scene(&self) -> Option<&str> {
        self.current_scene.as_deref()
    }

    // 获取场景实例
    pub fn get_scene(&self, scene_id: &str) -> Option<&Scene> {
        self.active_scenes.get(scene_id)
    }

    // 获取可变场景实例
    pub fn get_scene_mut(&mut self, scene_id: &str) -> Option<&mut Scene> {
        self.active_scenes.get_mut(scene_id)
    }

    // 获取活跃场景
    pub fn get_active_scene_mut(&mut self) -> Option<&mut Scene> {
        if let Some(scene_id) = &self.current_scene {
            self.active_scenes.get_mut(scene_id)
        } else {
            None
        }
    }

    // 暂停场景
    pub fn pause_scene(&mut self, scene_id: &str) -> GameResult<()> {
        if let Some(scene) = self.active_scenes.get_mut(scene_id) {
            if scene.state == SceneState::Active {
                scene.state = SceneState::Paused;
                info!("暂停场景: {}", scene_id);
            }
        }
        Ok(())
    }

    // 恢复场景
    pub fn resume_scene(&mut self, scene_id: &str) -> GameResult<()> {
        if let Some(scene) = self.active_scenes.get_mut(scene_id) {
            if scene.state == SceneState::Paused {
                scene.state = SceneState::Active;
                info!("恢复场景: {}", scene_id);
            }
        }
        Ok(())
    }

    // 在场景中生成实体
    pub fn spawn_entity_in_scene(&mut self, 
        commands: &mut Commands,
        scene_id: &str,
        layer: SceneLayer,
        bundle: impl Bundle
    ) -> GameResult<Entity> {
        
        let entity = commands.spawn(bundle).id();
        
        // 添加场景实体组件
        commands.entity(entity).insert(SceneEntity {
            scene_id: scene_id.to_string(),
            layer,
            persistent: false,
            auto_cleanup: true,
        });

        // 添加到场景
        if let Some(scene) = self.active_scenes.get_mut(scene_id) {
            scene.add_entity(entity, layer);
        }

        self.total_entities += 1;
        Ok(entity)
    }

    // 销毁场景中的实体
    pub fn despawn_entity_in_scene(&mut self, 
        commands: &mut Commands,
        scene_id: &str,
        entity: Entity
    ) -> GameResult<()> {
        
        commands.entity(entity).despawn_recursive();
        
        if let Some(scene) = self.active_scenes.get_mut(scene_id) {
            scene.remove_entity(entity);
        }

        self.total_entities = self.total_entities.saturating_sub(1);
        Ok(())
    }

    // 获取场景统计信息
    pub fn get_scene_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        stats.insert("total_scenes".to_string(), self.total_scenes);
        stats.insert("loaded_scenes".to_string(), self.loaded_scenes);
        stats.insert("total_entities".to_string(), self.total_entities);
        stats.insert("memory_usage_mb".to_string(), self.memory_usage / 1024 / 1024);
        stats.insert("scene_stack_depth".to_string(), self.scene_stack.len());
        
        stats
    }

    // 私有辅助方法

    // 注册默认场景
    fn register_default_scenes(&mut self) -> GameResult<()> {
        // 主菜单场景
        let main_menu_config = SceneConfig {
            id: "main_menu".to_string(),
            scene_type: SceneType::MainMenu,
            name: "主菜单".to_string(),
            description: "游戏主菜单场景".to_string(),
            resource_path: "scenes/main_menu".to_string(),
            enable_physics: false,
            ambient_light: Color::rgba(1.0, 1.0, 1.0, 0.8),
            ..Default::default()
        };
        self.register_scene(main_menu_config)?;

        // 大世界场景
        let overworld_config = SceneConfig {
            id: "overworld".to_string(),
            scene_type: SceneType::Overworld,
            name: "大世界".to_string(),
            description: "游戏主世界场景".to_string(),
            resource_path: "scenes/overworld".to_string(),
            max_entities: 2000,
            enable_physics: true,
            enable_audio: true,
            ambient_light: Color::rgba(1.0, 1.0, 0.9, 1.0),
            ..Default::default()
        };
        self.register_scene(overworld_config)?;

        // 战斗场景
        let battle_config = SceneConfig {
            id: "battle".to_string(),
            scene_type: SceneType::Battle,
            name: "战斗".to_string(),
            description: "宝可梦战斗场景".to_string(),
            resource_path: "scenes/battle".to_string(),
            max_entities: 500,
            enable_physics: false,
            ambient_light: Color::rgba(1.0, 0.95, 0.8, 1.0),
            time_scale: 1.0,
            ..Default::default()
        };
        self.register_scene(battle_config)?;

        Ok(())
    }

    // 更新场景过渡
    fn update_transition(&mut self, delta_time: f32) -> GameResult<()> {
        if let Some(transition) = &mut self.transition_state {
            self.transition_timer += delta_time;
            self.transition_progress = (self.transition_timer / transition.duration).min(1.0);

            // 检查过渡是否完成
            if self.transition_progress >= 1.0 {
                // 完成过渡
                self.complete_transition()?;
            }
        }
        Ok(())
    }

    // 完成场景过渡
    fn complete_transition(&mut self) -> GameResult<()> {
        if let Some(transition) = self.transition_state.take() {
            // 卸载旧场景
            if let Some(from_scene) = &transition.from_scene {
                // 这里应该卸载场景，简化实现
                info!("过渡：卸载场景 {}", from_scene);
            }

            // 激活新场景
            self.current_scene = Some(transition.to_scene.clone());
            info!("过渡完成：激活场景 {}", transition.to_scene);

            self.transition_progress = 0.0;
            self.transition_timer = 0.0;
        }
        Ok(())
    }

    // 更新活跃场景
    fn update_active_scenes(&mut self, delta_time: f32) -> GameResult<()> {
        let current_time = self.get_current_time();
        
        for scene in self.active_scenes.values_mut() {
            if scene.state == SceneState::Active {
                scene.last_update = current_time;
                // 这里可以更新场景特定的逻辑
            }
        }
        
        Ok(())
    }

    // 预加载场景资源
    fn preload_scene_resources(&self, scene: &Scene) -> GameResult<()> {
        for resource_id in &scene.config.preload_resources {
            debug!("预加载资源: {}", resource_id);
            // 这里应该调用资源管理器预加载资源
        }
        Ok(())
    }

    // 初始化场景系统
    fn initialize_scene_systems(&self, _commands: &mut Commands, scene: &mut Scene) -> GameResult<()> {
        debug!("初始化场景系统: {}", scene.config.id);
        
        // 根据场景类型初始化不同的系统
        match scene.config.scene_type {
            SceneType::Battle => {
                // 初始化战斗系统
            },
            SceneType::Overworld => {
                // 初始化世界系统
            },
            _ => {}
        }
        
        Ok(())
    }

    // 生成场景实体
    fn spawn_scene_entities(&self, _commands: &mut Commands, scene: &mut Scene) -> GameResult<()> {
        debug!("生成场景实体: {}", scene.config.id);
        
        // 根据场景配置生成实体
        // 这里应该从场景文件或配置中读取实体信息
        
        Ok(())
    }

    // 清理场景实体
    fn cleanup_scene_entities(&self, commands: &mut Commands, scene: &Scene) {
        for &entity in &scene.entities {
            commands.entity(entity).despawn_recursive();
        }
        debug!("清理场景实体: {} 个", scene.entities.len());
    }

    // 卸载场景资源
    fn unload_scene_resources(&self, scene: &Scene) -> GameResult<()> {
        for resource_id in &scene.config.preload_resources {
            debug!("卸载资源: {}", resource_id);
            // 这里应该调用资源管理器卸载资源
        }
        Ok(())
    }

    // 卸载所有场景
    fn unload_all_scenes(&mut self) -> GameResult<()> {
        let scene_ids: Vec<String> = self.active_scenes.keys().cloned().collect();
        
        for scene_id in scene_ids {
            // 这里需要Commands参数，简化实现
            self.active_scenes.remove(&scene_id);
        }
        
        self.loaded_scenes = 0;
        self.total_entities = 0;
        Ok(())
    }

    // 更新统计信息
    fn update_statistics(&mut self) {
        self.memory_usage = self.active_scenes.values()
            .map(|scene| scene.memory_usage)
            .sum();
    }

    // 获取当前时间
    fn get_current_time(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
}

// Bevy系统实现
pub fn scene_management_system(
    mut scene_manager: ResMut<SceneManager>,
    time: Res<Time>,
) {
    let _ = scene_manager.update(time.delta_seconds());
}

// 场景实体清理系统
pub fn scene_entity_cleanup_system(
    mut commands: Commands,
    query: Query<(Entity, &SceneEntity)>,
    mut scene_manager: ResMut<SceneManager>,
) {
    for (entity, scene_entity) in query.iter() {
        // 检查实体所属场景是否还活跃
        if !scene_manager.active_scenes.contains_key(&scene_entity.scene_id) {
            if scene_entity.auto_cleanup {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// 场景事件处理系统
pub fn scene_events_system(
    mut scene_events: EventReader<SceneEvent>,
    mut scene_manager: ResMut<SceneManager>,
) {
    for event in scene_events.iter() {
        match event {
            SceneEvent::SceneLoadCompleted(scene_id) => {
                info!("场景加载完成事件: {}", scene_id);
            },
            SceneEvent::SceneLoadFailed(scene_id, error) => {
                error!("场景加载失败: {} - {}", scene_id, error);
            },
            SceneEvent::TransitionCompleted(from, to) => {
                info!("场景切换完成: {} -> {}", from, to);
            },
            _ => {
                debug!("场景事件: {:?}", event);
            }
        }
    }
}