// ECS系统 (Entity Component System)
// 开发心理：ECS提供高性能的数据驱动架构，分离数据和逻辑，易于并行化
// 设计原则：数据局部性、系统解耦、可扩展性、缓存友好

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::any::{Any, TypeId};
use log::{debug, warn, error, info};
use crate::core::error::GameError;

pub mod entity;
pub mod component;
pub mod system;
pub mod world;
pub mod query;

// 实体ID类型
pub type EntityId = u64;

// 组件ID类型  
pub type ComponentId = TypeId;

// 系统ID类型
pub type SystemId = u64;

// ECS世界
pub struct ECSWorld {
    // 实体管理
    entity_manager: entity::EntityManager,
    
    // 组件管理
    component_manager: component::ComponentManager,
    
    // 系统管理
    system_manager: system::SystemManager,
    
    // 查询缓存
    query_cache: HashMap<String, query::QueryResult>,
    
    // 配置
    config: ECSConfig,
    
    // 统计信息
    statistics: ECSStatistics,
    
    // 帧计数
    frame_count: u64,
}

// ECS配置
#[derive(Debug, Clone)]
pub struct ECSConfig {
    pub max_entities: usize,
    pub max_components_per_entity: usize,
    pub enable_parallel_systems: bool,
    pub enable_query_caching: bool,
    pub statistics_enabled: bool,
    pub debug_mode: bool,
}

// ECS统计信息
#[derive(Debug, Clone, Default)]
pub struct ECSStatistics {
    pub entities_created: u64,
    pub entities_destroyed: u64,
    pub components_added: u64,
    pub components_removed: u64,
    pub systems_executed: u64,
    pub queries_executed: u64,
    pub query_cache_hits: u64,
    pub total_update_time: std::time::Duration,
    pub average_frame_time: std::time::Duration,
}

// 组件特征
pub trait Component: Any + Send + Sync + 'static {
    fn type_name() -> &'static str where Self: Sized;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Component>;
}

// 系统特征
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn update(&mut self, world: &mut ECSWorld, delta_time: f32) -> Result<(), GameError>;
    fn dependencies(&self) -> &[SystemId] { &[] }
    fn conflicts(&self) -> &[SystemId] { &[] }
    fn enabled(&self) -> bool { true }
}

// 查询特征
pub trait Query {
    type Item;
    fn matches(&self, entity: EntityId, world: &ECSWorld) -> bool;
    fn get(&self, entity: EntityId, world: &ECSWorld) -> Option<Self::Item>;
}

impl ECSWorld {
    pub fn new() -> Self {
        Self::with_config(ECSConfig::default())
    }
    
    pub fn with_config(config: ECSConfig) -> Self {
        Self {
            entity_manager: entity::EntityManager::new(config.max_entities),
            component_manager: component::ComponentManager::new(),
            system_manager: system::SystemManager::new(),
            query_cache: HashMap::new(),
            config,
            statistics: ECSStatistics::default(),
            frame_count: 0,
        }
    }
    
    // 创建实体
    pub fn create_entity(&mut self) -> Result<EntityId, GameError> {
        let entity_id = self.entity_manager.create_entity()?;
        
        if self.config.statistics_enabled {
            self.statistics.entities_created += 1;
        }
        
        debug!("创建实体: {}", entity_id);
        Ok(entity_id)
    }
    
    // 销毁实体
    pub fn destroy_entity(&mut self, entity_id: EntityId) -> Result<(), GameError> {
        // 移除所有组件
        let component_types = self.component_manager.get_entity_components(entity_id);
        for component_type in component_types {
            self.component_manager.remove_component(entity_id, component_type)?;
        }
        
        // 销毁实体
        self.entity_manager.destroy_entity(entity_id)?;
        
        // 清除查询缓存
        if self.config.enable_query_caching {
            self.invalidate_query_cache();
        }
        
        if self.config.statistics_enabled {
            self.statistics.entities_destroyed += 1;
        }
        
        debug!("销毁实体: {}", entity_id);
        Ok(())
    }
    
    // 添加组件
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, component: T) -> Result<(), GameError> {
        // 检查实体是否存在
        if !self.entity_manager.exists(entity_id) {
            return Err(GameError::ECS(format!("实体不存在: {}", entity_id)));
        }
        
        self.component_manager.add_component(entity_id, component)?;
        
        // 清除查询缓存
        if self.config.enable_query_caching {
            self.invalidate_query_cache();
        }
        
        if self.config.statistics_enabled {
            self.statistics.components_added += 1;
        }
        
        debug!("添加组件: {} -> {}", entity_id, std::any::type_name::<T>());
        Ok(())
    }
    
    // 移除组件
    pub fn remove_component<T: Component>(&mut self, entity_id: EntityId) -> Result<(), GameError> {
        let component_type = TypeId::of::<T>();
        self.component_manager.remove_component(entity_id, component_type)?;
        
        // 清除查询缓存
        if self.config.enable_query_caching {
            self.invalidate_query_cache();
        }
        
        if self.config.statistics_enabled {
            self.statistics.components_removed += 1;
        }
        
        debug!("移除组件: {} -> {}", entity_id, std::any::type_name::<T>());
        Ok(())
    }
    
    // 获取组件
    pub fn get_component<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        self.component_manager.get_component::<T>(entity_id)
    }
    
    // 获取可变组件
    pub fn get_component_mut<T: Component>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        self.component_manager.get_component_mut::<T>(entity_id)
    }
    
    // 检查组件是否存在
    pub fn has_component<T: Component>(&self, entity_id: EntityId) -> bool {
        self.component_manager.has_component::<T>(entity_id)
    }
    
    // 注册系统
    pub fn register_system<T: System + 'static>(&mut self, system: T) -> Result<SystemId, GameError> {
        let system_id = self.system_manager.register_system(Box::new(system))?;
        
        info!("注册系统: {} (ID: {})", std::any::type_name::<T>(), system_id);
        Ok(system_id)
    }
    
    // 移除系统
    pub fn remove_system(&mut self, system_id: SystemId) -> Result<(), GameError> {
        self.system_manager.remove_system(system_id)?;
        
        debug!("移除系统: {}", system_id);
        Ok(())
    }
    
    // 启用系统
    pub fn enable_system(&mut self, system_id: SystemId) -> Result<(), GameError> {
        self.system_manager.set_system_enabled(system_id, true)
    }
    
    // 禁用系统
    pub fn disable_system(&mut self, system_id: SystemId) -> Result<(), GameError> {
        self.system_manager.set_system_enabled(system_id, false)
    }
    
    // 更新世界
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        let frame_start = std::time::Instant::now();
        self.frame_count += 1;
        
        // 更新系统
        self.system_manager.update(self, delta_time)?;
        
        // 清理已销毁的实体
        self.entity_manager.cleanup_destroyed_entities();
        
        // 更新统计信息
        if self.config.statistics_enabled {
            let frame_time = frame_start.elapsed();
            self.statistics.total_update_time += frame_time;
            self.statistics.average_frame_time = 
                self.statistics.total_update_time / self.frame_count as u32;
        }
        
        Ok(())
    }
    
    // 查询实体
    pub fn query<Q: Query>(&mut self, query: Q) -> Vec<(EntityId, Q::Item)> {
        // 检查查询缓存
        let query_key = format!("{:?}", std::any::type_name::<Q>());
        
        if self.config.enable_query_caching {
            if let Some(cached_result) = self.query_cache.get(&query_key) {
                if self.config.statistics_enabled {
                    self.statistics.query_cache_hits += 1;
                }
                // 简化实现：实际应该返回缓存的结果
            }
        }
        
        // 执行查询
        let mut results = Vec::new();
        
        for entity_id in self.entity_manager.get_all_entities() {
            if query.matches(entity_id, self) {
                if let Some(item) = query.get(entity_id, self) {
                    results.push((entity_id, item));
                }
            }
        }
        
        // 缓存查询结果
        if self.config.enable_query_caching {
            let query_result = query::QueryResult {
                entity_count: results.len(),
                timestamp: std::time::Instant::now(),
            };
            self.query_cache.insert(query_key, query_result);
        }
        
        if self.config.statistics_enabled {
            self.statistics.queries_executed += 1;
        }
        
        debug!("查询执行完成: 匹配 {} 个实体", results.len());
        results
    }
    
    // 批量创建实体
    pub fn create_entities(&mut self, count: usize) -> Result<Vec<EntityId>, GameError> {
        let mut entities = Vec::with_capacity(count);
        
        for _ in 0..count {
            let entity_id = self.create_entity()?;
            entities.push(entity_id);
        }
        
        debug!("批量创建实体: {} 个", count);
        Ok(entities)
    }
    
    // 获取实体数量
    pub fn get_entity_count(&self) -> usize {
        self.entity_manager.get_entity_count()
    }
    
    // 获取活跃实体列表
    pub fn get_all_entities(&self) -> Vec<EntityId> {
        self.entity_manager.get_all_entities()
    }
    
    // 获取组件统计
    pub fn get_component_stats(&self) -> HashMap<ComponentId, usize> {
        self.component_manager.get_component_stats()
    }
    
    // 获取系统信息
    pub fn get_system_info(&self, system_id: SystemId) -> Option<system::SystemInfo> {
        self.system_manager.get_system_info(system_id)
    }
    
    // 获取ECS统计信息
    pub fn get_statistics(&self) -> &ECSStatistics {
        &self.statistics
    }
    
    // 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.statistics = ECSStatistics::default();
        self.frame_count = 0;
        debug!("ECS统计信息已重置");
    }
    
    // 序列化世界状态
    pub fn serialize_world(&self) -> Result<Vec<u8>, GameError> {
        // 简化实现：实际应该序列化所有实体和组件
        let world_data = WorldSnapshot {
            entities: self.entity_manager.get_all_entities(),
            frame_count: self.frame_count,
            timestamp: std::time::SystemTime::now(),
        };
        
        bincode::serialize(&world_data)
            .map_err(|e| GameError::ECS(format!("序列化世界失败: {}", e)))
    }
    
    // 反序列化世界状态
    pub fn deserialize_world(&mut self, data: &[u8]) -> Result<(), GameError> {
        let world_data: WorldSnapshot = bincode::deserialize(data)
            .map_err(|e| GameError::ECS(format!("反序列化世界失败: {}", e)))?;
        
        self.frame_count = world_data.frame_count;
        
        info!("反序列化世界成功: {} 个实体", world_data.entities.len());
        Ok(())
    }
    
    // 私有方法
    fn invalidate_query_cache(&mut self) {
        if !self.query_cache.is_empty() {
            self.query_cache.clear();
            debug!("查询缓存已清除");
        }
    }
}

// 世界快照
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorldSnapshot {
    entities: Vec<EntityId>,
    frame_count: u64,
    timestamp: std::time::SystemTime,
}

impl Default for ECSConfig {
    fn default() -> Self {
        Self {
            max_entities: 100000,
            max_components_per_entity: 32,
            enable_parallel_systems: true,
            enable_query_caching: true,
            statistics_enabled: true,
            debug_mode: false,
        }
    }
}

// 自动实现Component特征的宏
#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn type_name() -> &'static str {
                std::any::type_name::<$type>()
            }
            
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            
            fn clone_box(&self) -> Box<dyn Component> {
                Box::new(self.clone())
            }
        }
    };
}

// 查询构建器
pub struct QueryBuilder {
    required_components: HashSet<ComponentId>,
    excluded_components: HashSet<ComponentId>,
    entity_filter: Option<Box<dyn Fn(EntityId) -> bool>>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            required_components: HashSet::new(),
            excluded_components: HashSet::new(),
            entity_filter: None,
        }
    }
    
    pub fn with<T: Component>(mut self) -> Self {
        self.required_components.insert(TypeId::of::<T>());
        self
    }
    
    pub fn without<T: Component>(mut self) -> Self {
        self.excluded_components.insert(TypeId::of::<T>());
        self
    }
    
    pub fn filter<F: Fn(EntityId) -> bool + 'static>(mut self, filter: F) -> Self {
        self.entity_filter = Some(Box::new(filter));
        self
    }
    
    pub fn execute(&self, world: &ECSWorld) -> Vec<EntityId> {
        let mut results = Vec::new();
        
        for entity_id in world.get_all_entities() {
            // 检查必需组件
            let has_required = self.required_components.iter()
                .all(|&component_id| world.component_manager.has_component_by_id(entity_id, component_id));
            
            if !has_required {
                continue;
            }
            
            // 检查排除组件
            let has_excluded = self.excluded_components.iter()
                .any(|&component_id| world.component_manager.has_component_by_id(entity_id, component_id));
            
            if has_excluded {
                continue;
            }
            
            // 检查自定义过滤器
            if let Some(ref filter) = self.entity_filter {
                if !filter(entity_id) {
                    continue;
                }
            }
            
            results.push(entity_id);
        }
        
        results
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 测试组件
    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }
    
    impl_component!(Position);
    
    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
        dz: f32,
    }
    
    impl_component!(Velocity);
    
    #[test]
    fn test_ecs_world_creation() {
        let world = ECSWorld::new();
        assert_eq!(world.get_entity_count(), 0);
    }
    
    #[test]
    fn test_entity_creation_destruction() {
        let mut world = ECSWorld::new();
        
        let entity_id = world.create_entity().unwrap();
        assert!(entity_id > 0);
        assert_eq!(world.get_entity_count(), 1);
        
        world.destroy_entity(entity_id).unwrap();
        assert_eq!(world.get_entity_count(), 0);
    }
    
    #[test]
    fn test_component_operations() {
        let mut world = ECSWorld::new();
        let entity_id = world.create_entity().unwrap();
        
        let position = Position { x: 1.0, y: 2.0, z: 3.0 };
        world.add_component(entity_id, position.clone()).unwrap();
        
        assert!(world.has_component::<Position>(entity_id));
        
        let retrieved_position = world.get_component::<Position>(entity_id);
        assert!(retrieved_position.is_some());
        assert_eq!(*retrieved_position.unwrap(), position);
        
        world.remove_component::<Position>(entity_id).unwrap();
        assert!(!world.has_component::<Position>(entity_id));
    }
    
    #[test]
    fn test_batch_entity_creation() {
        let mut world = ECSWorld::new();
        
        let entities = world.create_entities(100).unwrap();
        assert_eq!(entities.len(), 100);
        assert_eq!(world.get_entity_count(), 100);
    }
    
    #[test]
    fn test_query_builder() {
        let world = ECSWorld::new();
        
        let query = QueryBuilder::new()
            .with::<Position>()
            .with::<Velocity>()
            .without::<Position>(); // 这会导致查询结果为空
        
        let results = query.execute(&world);
        assert!(results.is_empty());
    }
}