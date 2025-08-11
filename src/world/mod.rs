// 世界系统
// 开发心理：世界系统管理游戏地图、NPC、环境、事件触发等核心要素
// 设计原则：模块化地图、动态加载、事件驱动、性能优化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use glam::{Vec2, Vec3};

pub mod map;
pub mod npc;
pub mod environment;
pub mod events;

// 世界ID类型
pub type WorldId = u32;
pub type MapId = u32;
pub type EntityId = u64;

// 世界数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub id: WorldId,
    pub name: String,
    pub description: String,
    
    // 地图系统
    pub maps: HashMap<MapId, map::GameMap>,
    pub current_map: Option<MapId>,
    
    // 实体系统
    pub entities: HashMap<EntityId, WorldEntity>,
    pub next_entity_id: EntityId,
    
    // 环境系统
    pub environment: environment::Environment,
    
    // 事件系统
    pub events: events::EventManager,
    
    // 世界状态
    pub world_flags: HashMap<String, bool>,
    pub world_variables: HashMap<String, i32>,
    
    // 时间系统
    pub world_time: WorldTime,
    
    // 天气系统
    pub weather: WeatherSystem,
}

// 世界实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEntity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
    pub active: bool,
    pub persistent: bool,        // 是否持久化
    
    // 组件数据
    pub components: HashMap<String, EntityComponent>,
}

// 实体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    NPC,
    WildPokemon,
    Item,
    Interactable,
    Trigger,
    Decoration,
}

// 实体组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityComponent {
    Sprite { sprite_id: u32, animation: Option<String> },
    Collider { width: f32, height: f32, solid: bool },
    Movement { speed: f32, direction: Vec2 },
    AI { behavior: String, state: HashMap<String, String> },
    Interaction { interaction_type: String, data: HashMap<String, String> },
    #[cfg(feature = "pokemon-wip")]
    Pokemon { species_id: u32, level: u8, stats: Option<crate::pokemon::stats::PokemonStats> },
    #[cfg(not(feature = "pokemon-wip"))]
    Pokemon { species_id: u32, level: u8, stats: Option<crate::world::npc::PokemonStats> },
    Item { item_id: u32, quantity: u32 },
}

// 世界时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTime {
    pub day: u32,
    pub hour: u8,           // 0-23
    pub minute: u8,         // 0-59
    pub time_scale: f32,    // 时间流逝速度倍率
}

// 天气系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSystem {
    pub current_weather: Weather,
    pub weather_duration: f32,  // 当前天气剩余时间
    pub weather_intensity: f32, // 天气强度 0.0-1.0
    pub weather_transition: Option<Weather>, // 正在转换到的天气
}

// 天气类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,      // 晴朗
    Rain,       // 下雨
    Snow,       // 下雪
    Fog,        // 雾
    Storm,      // 暴风雨
    Sandstorm,  // 沙尘暴
}

// 世界管理器
pub struct WorldManager {
    // 当前活跃的世界
    current_world: Option<World>,
    
    // 世界缓存
    world_cache: HashMap<WorldId, World>,
    
    // 加载状态
    loading_maps: Vec<MapId>,
    
    // 更新计时器
    update_timer: f32,
    auto_save_timer: f32,
    
    // 配置
    auto_save_interval: f32,    // 自动保存间隔
    max_cached_worlds: usize,   // 最大缓存世界数
    
    // 统计
    total_entities_created: u64,
    total_maps_loaded: u64,
    frame_count: u64,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            current_world: None,
            world_cache: HashMap::new(),
            loading_maps: Vec::new(),
            update_timer: 0.0,
            auto_save_timer: 0.0,
            auto_save_interval: 300.0, // 5分钟
            max_cached_worlds: 3,
            total_entities_created: 0,
            total_maps_loaded: 0,
            frame_count: 0,
        }
    }
    
    // 创建新世界
    pub fn create_world(&mut self, name: String, description: String) -> Result<WorldId, GameError> {
        let world_id = self.generate_world_id();
        
        let world = World {
            id: world_id,
            name: name.clone(),
            description,
            maps: HashMap::new(),
            current_map: None,
            entities: HashMap::new(),
            next_entity_id: 1,
            environment: environment::Environment::new(),
            events: events::EventManager::new(),
            world_flags: HashMap::new(),
            world_variables: HashMap::new(),
            world_time: WorldTime {
                day: 1,
                hour: 12,
                minute: 0,
                time_scale: 1.0,
            },
            weather: WeatherSystem {
                current_weather: Weather::Clear,
                weather_duration: 3600.0, // 1小时
                weather_intensity: 0.5,
                weather_transition: None,
            },
        };
        
        self.world_cache.insert(world_id, world);
        debug!("创建新世界: '{}' ID={}", name, world_id);
        
        Ok(world_id)
    }
    
    // 加载世界
    pub fn load_world(&mut self, world_id: WorldId) -> Result<(), GameError> {
        if let Some(world) = self.world_cache.get(&world_id).cloned() {
            self.current_world = Some(world);
            debug!("从缓存加载世界: ID={}", world_id);
            return Ok(());
        }
        
        // 从文件加载
        match self.load_world_from_file(world_id) {
            Ok(world) => {
                self.current_world = Some(world.clone());
                self.world_cache.insert(world_id, world);
                debug!("从文件加载世界: ID={}", world_id);
                Ok(())
            },
            Err(e) => {
                error!("加载世界失败: {}", e);
                Err(e)
            }
        }
    }
    
    // 获取当前世界
    pub fn get_current_world(&self) -> Option<&World> {
        self.current_world.as_ref()
    }
    
    // 获取当前世界(可变)
    pub fn get_current_world_mut(&mut self) -> Option<&mut World> {
        self.current_world.as_mut()
    }
    
    // 切换地图
    pub fn switch_map(&mut self, map_id: MapId) -> Result<(), GameError> {
        if let Some(ref mut world) = self.current_world {
            if world.maps.contains_key(&map_id) {
                world.current_map = Some(map_id);
                debug!("切换到地图: ID={}", map_id);
                
                // 触发地图切换事件
                world.events.trigger_event("map_changed", HashMap::new());
                
                Ok(())
            } else {
                // 尝试加载地图
                self.load_map(map_id)?;
                world.current_map = Some(map_id);
                Ok(())
            }
        } else {
            Err(GameError::World("没有活跃的世界".to_string()))
        }
    }
    
    // 加载地图
    pub fn load_map(&mut self, map_id: MapId) -> Result<(), GameError> {
        if self.loading_maps.contains(&map_id) {
            return Err(GameError::World("地图正在加载中".to_string()));
        }
        
        self.loading_maps.push(map_id);
        
        // 实际的地图加载逻辑
        let game_map = self.load_map_from_file(map_id)?;
        
        if let Some(ref mut world) = self.current_world {
            world.maps.insert(map_id, game_map);
            self.total_maps_loaded += 1;
            debug!("加载地图: ID={}", map_id);
        }
        
        self.loading_maps.retain(|&id| id != map_id);
        Ok(())
    }
    
    // 创建实体
    pub fn create_entity(
        &mut self,
        entity_type: EntityType,
        position: Vec3,
        components: Vec<EntityComponent>,
    ) -> Result<EntityId, GameError> {
        if let Some(ref mut world) = self.current_world {
            let entity_id = world.next_entity_id;
            world.next_entity_id += 1;
            
            let mut entity_components = HashMap::new();
            for (i, component) in components.into_iter().enumerate() {
                entity_components.insert(format!("component_{}", i), component);
            }
            
            let entity = WorldEntity {
                id: entity_id,
                entity_type,
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
                active: true,
                persistent: true,
                components: entity_components,
            };
            
            world.entities.insert(entity_id, entity);
            self.total_entities_created += 1;
            
            debug!("创建实体: 类型={:?} ID={} 位置={:?}", entity_type, entity_id, position);
            Ok(entity_id)
        } else {
            Err(GameError::World("没有活跃的世界".to_string()))
        }
    }
    
    // 销毁实体
    pub fn destroy_entity(&mut self, entity_id: EntityId) -> Result<(), GameError> {
        if let Some(ref mut world) = self.current_world {
            if world.entities.remove(&entity_id).is_some() {
                debug!("销毁实体: ID={}", entity_id);
                Ok(())
            } else {
                Err(GameError::World(format!("实体不存在: {}", entity_id)))
            }
        } else {
            Err(GameError::World("没有活跃的世界".to_string()))
        }
    }
    
    // 获取实体
    pub fn get_entity(&self, entity_id: EntityId) -> Option<&WorldEntity> {
        self.current_world.as_ref()?.entities.get(&entity_id)
    }
    
    // 获取实体(可变)
    pub fn get_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut WorldEntity> {
        self.current_world.as_mut()?.entities.get_mut(&entity_id)
    }
    
    // 按类型查找实体
    pub fn find_entities_by_type(&self, entity_type: EntityType) -> Vec<EntityId> {
        if let Some(world) = &self.current_world {
            world.entities
                .iter()
                .filter(|(_, entity)| entity.entity_type == entity_type && entity.active)
                .map(|(&id, _)| id)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    // 按位置查找实体
    pub fn find_entities_near(&self, position: Vec3, radius: f32) -> Vec<EntityId> {
        if let Some(world) = &self.current_world {
            world.entities
                .iter()
                .filter(|(_, entity)| {
                    entity.active && (entity.position - position).length() <= radius
                })
                .map(|(&id, _)| id)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    // 更新世界
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        self.frame_count += 1;
        self.update_timer += delta_time;
        self.auto_save_timer += delta_time;
        
        if let Some(ref mut world) = self.current_world {
            // 更新世界时间
            self.update_world_time(&mut world.world_time, delta_time);
            
            // 更新天气
            self.update_weather(&mut world.weather, delta_time);
            
            // 更新环境
            world.environment.update(delta_time)?;
            
            // 更新事件系统
            world.events.update(delta_time)?;
            
            // 更新活跃实体
            for entity in world.entities.values_mut() {
                if entity.active {
                    self.update_entity(entity, delta_time)?;
                }
            }
        }
        
        // 自动保存检查
        if self.auto_save_timer >= self.auto_save_interval {
            self.save_current_world()?;
            self.auto_save_timer = 0.0;
        }
        
        Ok(())
    }
    
    // 保存当前世界
    pub fn save_current_world(&mut self) -> Result<(), GameError> {
        if let Some(ref world) = self.current_world {
            self.save_world_to_file(world)?;
            debug!("保存世界: {} (ID: {})", world.name, world.id);
        }
        Ok(())
    }
    
    // 私有方法
    fn generate_world_id(&self) -> WorldId {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;
        timestamp
    }
    
    fn update_world_time(&self, world_time: &mut WorldTime, delta_time: f32) {
        let minutes_passed = (delta_time * world_time.time_scale) / 60.0;
        let total_minutes = world_time.minute as f32 + minutes_passed;
        
        if total_minutes >= 60.0 {
            let hours_passed = total_minutes as u32 / 60;
            world_time.hour = (world_time.hour + hours_passed as u8) % 24;
            world_time.minute = (total_minutes % 60.0) as u8;
            
            if world_time.hour == 0 {
                world_time.day += 1;
            }
        } else {
            world_time.minute = total_minutes as u8;
        }
    }
    
    fn update_weather(&self, weather: &mut WeatherSystem, delta_time: f32) {
        weather.weather_duration -= delta_time;
        
        if weather.weather_duration <= 0.0 {
            // 随机切换天气
            let new_weather = match fastrand::u8(0..6) {
                0 => Weather::Clear,
                1 => Weather::Rain,
                2 => Weather::Snow,
                3 => Weather::Fog,
                4 => Weather::Storm,
                5 => Weather::Sandstorm,
                _ => Weather::Clear,
            };
            
            weather.current_weather = new_weather;
            weather.weather_duration = 1800.0 + fastrand::f32() * 3600.0; // 30分钟到1.5小时
            weather.weather_intensity = 0.3 + fastrand::f32() * 0.7; // 0.3到1.0
            
            debug!("天气变化: {:?} 强度: {:.1}", new_weather, weather.weather_intensity);
        }
    }
    
    fn update_entity(&self, entity: &mut WorldEntity, delta_time: f32) -> Result<(), GameError> {
        // 更新实体组件
        for (component_name, component) in &mut entity.components {
            match component {
                EntityComponent::Movement { speed, direction } => {
                    let movement = *direction * *speed * delta_time;
                    entity.position.x += movement.x;
                    entity.position.z += movement.y;
                },
                EntityComponent::AI { behavior, state } => {
                    // 简化的AI更新
                    if behavior == "random_walk" {
                        // 随机移动逻辑
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn load_world_from_file(&self, world_id: WorldId) -> Result<World, GameError> {
        let filename = format!("worlds/world_{}.json", world_id);
        
        match std::fs::read_to_string(&filename) {
            Ok(data) => {
                match serde_json::from_str::<World>(&data) {
                    Ok(world) => Ok(world),
                    Err(e) => Err(GameError::World(format!("反序列化世界失败: {}", e))),
                }
            },
            Err(e) => Err(GameError::World(format!("读取世界文件失败: {}", e))),
        }
    }
    
    fn save_world_to_file(&self, world: &World) -> Result<(), GameError> {
        std::fs::create_dir_all("worlds").ok();
        let filename = format!("worlds/world_{}.json", world.id);
        
        match serde_json::to_string_pretty(world) {
            Ok(data) => {
                match std::fs::write(&filename, data) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(GameError::World(format!("写入世界文件失败: {}", e))),
                }
            },
            Err(e) => Err(GameError::World(format!("序列化世界失败: {}", e))),
        }
    }
    
    fn load_map_from_file(&self, map_id: MapId) -> Result<map::GameMap, GameError> {
        // 简化实现
        Ok(map::GameMap::new(
            map_id,
            "default_map".to_string(),
            Vec2::new(1000.0, 1000.0)
        ))
    }
}

// 世界统计信息
#[derive(Debug, Clone)]
pub struct WorldStats {
    pub entities_count: usize,
    pub maps_loaded: usize,
    pub current_day: u32,
    pub current_time: String,
    pub current_weather: Weather,
    pub frame_count: u64,
    pub total_entities_created: u64,
    pub total_maps_loaded: u64,
}

impl WorldManager {
    pub fn get_stats(&self) -> WorldStats {
        if let Some(world) = &self.current_world {
            WorldStats {
                entities_count: world.entities.len(),
                maps_loaded: world.maps.len(),
                current_day: world.world_time.day,
                current_time: format!("{}:{:02}", world.world_time.hour, world.world_time.minute),
                current_weather: world.weather.current_weather,
                frame_count: self.frame_count,
                total_entities_created: self.total_entities_created,
                total_maps_loaded: self.total_maps_loaded,
            }
        } else {
            WorldStats {
                entities_count: 0,
                maps_loaded: 0,
                current_day: 0,
                current_time: "00:00".to_string(),
                current_weather: Weather::Clear,
                frame_count: self.frame_count,
                total_entities_created: self.total_entities_created,
                total_maps_loaded: self.total_maps_loaded,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_manager_creation() {
        let manager = WorldManager::new();
        assert!(manager.current_world.is_none());
        assert_eq!(manager.world_cache.len(), 0);
    }
    
    #[test]
    fn test_world_creation() {
        let mut manager = WorldManager::new();
        
        let world_id = manager.create_world(
            "测试世界".to_string(),
            "用于测试的世界".to_string(),
        ).unwrap();
        
        assert!(world_id > 0);
        assert!(manager.world_cache.contains_key(&world_id));
    }
    
    #[test]
    fn test_entity_creation() {
        let mut manager = WorldManager::new();
        let world_id = manager.create_world("测试".to_string(), "测试".to_string()).unwrap();
        manager.load_world(world_id).unwrap();
        
        let entity_id = manager.create_entity(
            EntityType::NPC,
            Vec3::new(100.0, 0.0, 200.0),
            vec![
                EntityComponent::Sprite { sprite_id: 1, animation: None },
                EntityComponent::Collider { width: 32.0, height: 32.0, solid: true },
            ],
        ).unwrap();
        
        assert!(entity_id > 0);
        assert!(manager.get_entity(entity_id).is_some());
        
        let entity = manager.get_entity(entity_id).unwrap();
        assert_eq!(entity.entity_type, EntityType::NPC);
        assert_eq!(entity.position, Vec3::new(100.0, 0.0, 200.0));
    }
}