// 实体管理器
// 开发心理：实体是ECS中的基础单位，仅包含ID，实际数据存储在组件中
// 设计原则：ID复用、内存紧凑、快速分配、版本管理

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use log::{debug, warn};
use crate::core::error::GameError;
use super::EntityId;

// 实体管理器
pub struct EntityManager {
    // 活跃实体
    entities: HashMap<EntityId, EntityMetadata>,
    
    // 可复用的实体ID
    free_ids: VecDeque<EntityId>,
    
    // 下一个实体ID
    next_id: EntityId,
    
    // 已销毁但未清理的实体
    destroyed_entities: Vec<EntityId>,
    
    // 配置
    max_entities: usize,
    
    // 统计信息
    total_created: u64,
    total_destroyed: u64,
}

// 实体元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub id: EntityId,
    pub generation: u32,        // 版本号，用于检测悬空引用
    pub created_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub component_count: usize,
    pub tags: Vec<String>,      // 实体标签
    pub active: bool,
}

// 实体信息
#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub id: EntityId,
    pub generation: u32,
    pub created_at: std::time::SystemTime,
    pub component_count: usize,
    pub tags: Vec<String>,
    pub active: bool,
    pub age: std::time::Duration,
}

// 实体查询
pub struct EntityQuery {
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub min_components: Option<usize>,
    pub max_components: Option<usize>,
    pub created_after: Option<std::time::SystemTime>,
    pub created_before: Option<std::time::SystemTime>,
    pub active_only: bool,
}

impl EntityManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            entities: HashMap::with_capacity(max_entities),
            free_ids: VecDeque::new(),
            next_id: 1,
            destroyed_entities: Vec::new(),
            max_entities,
            total_created: 0,
            total_destroyed: 0,
        }
    }
    
    // 创建实体
    pub fn create_entity(&mut self) -> Result<EntityId, GameError> {
        // 检查实体数量限制
        if self.entities.len() >= self.max_entities {
            return Err(GameError::ECS(format!("实体数量已达上限: {}", self.max_entities)));
        }
        
        // 尝试复用ID
        let entity_id = if let Some(id) = self.free_ids.pop_front() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };
        
        // 创建实体元数据
        let now = std::time::SystemTime::now();
        let metadata = EntityMetadata {
            id: entity_id,
            generation: 1,
            created_at: now,
            last_accessed: now,
            component_count: 0,
            tags: Vec::new(),
            active: true,
        };
        
        self.entities.insert(entity_id, metadata);
        self.total_created += 1;
        
        debug!("创建实体: {} (总数: {})", entity_id, self.entities.len());
        Ok(entity_id)
    }
    
    // 标记实体为销毁
    pub fn destroy_entity(&mut self, entity_id: EntityId) -> Result<(), GameError> {
        if !self.entities.contains_key(&entity_id) {
            return Err(GameError::ECS(format!("实体不存在: {}", entity_id)));
        }
        
        // 标记为非活跃
        if let Some(metadata) = self.entities.get_mut(&entity_id) {
            metadata.active = false;
        }
        
        // 添加到销毁列表
        self.destroyed_entities.push(entity_id);
        
        debug!("标记实体销毁: {}", entity_id);
        Ok(())
    }
    
    // 清理已销毁的实体
    pub fn cleanup_destroyed_entities(&mut self) {
        for entity_id in self.destroyed_entities.drain(..) {
            if let Some(_metadata) = self.entities.remove(&entity_id) {
                // 将ID添加到可复用列表
                self.free_ids.push_back(entity_id);
                self.total_destroyed += 1;
                
                debug!("清理实体: {}", entity_id);
            }
        }
    }
    
    // 检查实体是否存在
    pub fn exists(&self, entity_id: EntityId) -> bool {
        self.entities.get(&entity_id)
            .map(|metadata| metadata.active)
            .unwrap_or(false)
    }
    
    // 检查实体是否有效（存在且活跃）
    pub fn is_valid(&self, entity_id: EntityId) -> bool {
        self.entities.get(&entity_id)
            .map(|metadata| metadata.active)
            .unwrap_or(false)
    }
    
    // 获取实体元数据
    pub fn get_metadata(&self, entity_id: EntityId) -> Option<&EntityMetadata> {
        self.entities.get(&entity_id)
    }
    
    // 获取可变实体元数据
    pub fn get_metadata_mut(&mut self, entity_id: EntityId) -> Option<&mut EntityMetadata> {
        self.entities.get_mut(&entity_id)
    }
    
    // 获取实体信息
    pub fn get_entity_info(&self, entity_id: EntityId) -> Option<EntityInfo> {
        self.entities.get(&entity_id).map(|metadata| {
            let age = std::time::SystemTime::now()
                .duration_since(metadata.created_at)
                .unwrap_or_default();
            
            EntityInfo {
                id: metadata.id,
                generation: metadata.generation,
                created_at: metadata.created_at,
                component_count: metadata.component_count,
                tags: metadata.tags.clone(),
                active: metadata.active,
                age,
            }
        })
    }
    
    // 添加标签
    pub fn add_tag(&mut self, entity_id: EntityId, tag: String) -> Result<(), GameError> {
        if let Some(metadata) = self.entities.get_mut(&entity_id) {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag.clone());
                debug!("添加实体标签: {} -> {}", entity_id, tag);
            }
            Ok(())
        } else {
            Err(GameError::ECS(format!("实体不存在: {}", entity_id)))
        }
    }
    
    // 移除标签
    pub fn remove_tag(&mut self, entity_id: EntityId, tag: &str) -> Result<bool, GameError> {
        if let Some(metadata) = self.entities.get_mut(&entity_id) {
            if let Some(pos) = metadata.tags.iter().position(|t| t == tag) {
                metadata.tags.remove(pos);
                debug!("移除实体标签: {} -> {}", entity_id, tag);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(GameError::ECS(format!("实体不存在: {}", entity_id)))
        }
    }
    
    // 检查标签
    pub fn has_tag(&self, entity_id: EntityId, tag: &str) -> bool {
        self.entities.get(&entity_id)
            .map(|metadata| metadata.tags.contains(&tag.to_string()))
            .unwrap_or(false)
    }
    
    // 根据标签查找实体
    pub fn find_entities_with_tag(&self, tag: &str) -> Vec<EntityId> {
        self.entities.iter()
            .filter_map(|(&id, metadata)| {
                if metadata.active && metadata.tags.contains(&tag.to_string()) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 根据标签查找实体（多个标签）
    pub fn find_entities_with_tags(&self, tags: &[String], require_all: bool) -> Vec<EntityId> {
        self.entities.iter()
            .filter_map(|(&id, metadata)| {
                if !metadata.active {
                    return None;
                }
                
                let matches = if require_all {
                    tags.iter().all(|tag| metadata.tags.contains(tag))
                } else {
                    tags.iter().any(|tag| metadata.tags.contains(tag))
                };
                
                if matches {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 查询实体
    pub fn query_entities(&self, query: &EntityQuery) -> Vec<EntityId> {
        self.entities.iter()
            .filter_map(|(&id, metadata)| {
                // 检查活跃状态
                if query.active_only && !metadata.active {
                    return None;
                }
                
                // 检查包含标签
                if !query.include_tags.is_empty() {
                    let has_all_included = query.include_tags.iter()
                        .all(|tag| metadata.tags.contains(tag));
                    if !has_all_included {
                        return None;
                    }
                }
                
                // 检查排除标签
                if !query.exclude_tags.is_empty() {
                    let has_any_excluded = query.exclude_tags.iter()
                        .any(|tag| metadata.tags.contains(tag));
                    if has_any_excluded {
                        return None;
                    }
                }
                
                // 检查组件数量
                if let Some(min_components) = query.min_components {
                    if metadata.component_count < min_components {
                        return None;
                    }
                }
                
                if let Some(max_components) = query.max_components {
                    if metadata.component_count > max_components {
                        return None;
                    }
                }
                
                // 检查创建时间
                if let Some(created_after) = query.created_after {
                    if metadata.created_at <= created_after {
                        return None;
                    }
                }
                
                if let Some(created_before) = query.created_before {
                    if metadata.created_at >= created_before {
                        return None;
                    }
                }
                
                Some(id)
            })
            .collect()
    }
    
    // 更新组件数量
    pub fn update_component_count(&mut self, entity_id: EntityId, delta: i32) -> Result<(), GameError> {
        if let Some(metadata) = self.entities.get_mut(&entity_id) {
            metadata.component_count = ((metadata.component_count as i32) + delta).max(0) as usize;
            metadata.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(GameError::ECS(format!("实体不存在: {}", entity_id)))
        }
    }
    
    // 获取实体数量
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }
    
    // 获取活跃实体数量
    pub fn get_active_entity_count(&self) -> usize {
        self.entities.values().filter(|metadata| metadata.active).count()
    }
    
    // 获取所有实体ID
    pub fn get_all_entities(&self) -> Vec<EntityId> {
        self.entities.keys().copied().collect()
    }
    
    // 获取活跃实体ID
    pub fn get_active_entities(&self) -> Vec<EntityId> {
        self.entities.iter()
            .filter_map(|(&id, metadata)| {
                if metadata.active {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 获取实体统计信息
    pub fn get_statistics(&self) -> EntityStatistics {
        let now = std::time::SystemTime::now();
        
        let mut ages = Vec::new();
        let mut component_counts = Vec::new();
        let mut tag_usage = std::collections::HashMap::new();
        
        for metadata in self.entities.values() {
            if let Ok(age) = now.duration_since(metadata.created_at) {
                ages.push(age);
            }
            
            component_counts.push(metadata.component_count);
            
            for tag in &metadata.tags {
                *tag_usage.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let average_age = if !ages.is_empty() {
            ages.iter().sum::<std::time::Duration>() / ages.len() as u32
        } else {
            std::time::Duration::default()
        };
        
        let average_components = if !component_counts.is_empty() {
            component_counts.iter().sum::<usize>() as f32 / component_counts.len() as f32
        } else {
            0.0
        };
        
        EntityStatistics {
            total_entities: self.entities.len(),
            active_entities: self.get_active_entity_count(),
            total_created: self.total_created,
            total_destroyed: self.total_destroyed,
            free_ids_count: self.free_ids.len(),
            average_age,
            average_components,
            tag_usage,
        }
    }
    
    // 压缩实体ID空间
    pub fn defragment(&mut self) {
        // 将所有活跃实体重新分配连续的ID
        let mut new_entities = HashMap::new();
        let mut new_id = 1;
        
        let old_to_new_mapping: HashMap<EntityId, EntityId> = self.entities.iter()
            .filter(|(_, metadata)| metadata.active)
            .map(|(&old_id, _)| {
                let new_id_assigned = new_id;
                new_id += 1;
                (old_id, new_id_assigned)
            })
            .collect();
        
        for (old_id, metadata) in &self.entities {
            if let Some(&new_id) = old_to_new_mapping.get(old_id) {
                let mut new_metadata = metadata.clone();
                new_metadata.id = new_id;
                new_entities.insert(new_id, new_metadata);
            }
        }
        
        self.entities = new_entities;
        self.next_id = new_id;
        self.free_ids.clear();
        
        debug!("实体ID空间压缩完成: 重新分配 {} 个实体", old_to_new_mapping.len());
    }
    
    // 序列化实体管理器
    pub fn serialize(&self) -> Result<Vec<u8>, GameError> {
        let snapshot = EntityManagerSnapshot {
            entities: self.entities.clone(),
            next_id: self.next_id,
            total_created: self.total_created,
            total_destroyed: self.total_destroyed,
        };
        
        bincode::serialize(&snapshot)
            .map_err(|e| GameError::ECS(format!("序列化实体管理器失败: {}", e)))
    }
    
    // 反序列化实体管理器
    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), GameError> {
        let snapshot: EntityManagerSnapshot = bincode::deserialize(data)
            .map_err(|e| GameError::ECS(format!("反序列化实体管理器失败: {}", e)))?;
        
        self.entities = snapshot.entities;
        self.next_id = snapshot.next_id;
        self.total_created = snapshot.total_created;
        self.total_destroyed = snapshot.total_destroyed;
        self.free_ids.clear();
        self.destroyed_entities.clear();
        
        debug!("反序列化实体管理器成功: {} 个实体", self.entities.len());
        Ok(())
    }
}

// 实体统计信息
#[derive(Debug, Clone)]
pub struct EntityStatistics {
    pub total_entities: usize,
    pub active_entities: usize,
    pub total_created: u64,
    pub total_destroyed: u64,
    pub free_ids_count: usize,
    pub average_age: std::time::Duration,
    pub average_components: f32,
    pub tag_usage: HashMap<String, u32>,
}

// 实体管理器快照
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityManagerSnapshot {
    entities: HashMap<EntityId, EntityMetadata>,
    next_id: EntityId,
    total_created: u64,
    total_destroyed: u64,
}

impl Default for EntityQuery {
    fn default() -> Self {
        Self {
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            min_components: None,
            max_components: None,
            created_after: None,
            created_before: None,
            active_only: true,
        }
    }
}

impl EntityQuery {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_tag(mut self, tag: String) -> Self {
        self.include_tags.push(tag);
        self
    }
    
    pub fn without_tag(mut self, tag: String) -> Self {
        self.exclude_tags.push(tag);
        self
    }
    
    pub fn min_components(mut self, count: usize) -> Self {
        self.min_components = Some(count);
        self
    }
    
    pub fn max_components(mut self, count: usize) -> Self {
        self.max_components = Some(count);
        self
    }
    
    pub fn created_after(mut self, time: std::time::SystemTime) -> Self {
        self.created_after = Some(time);
        self
    }
    
    pub fn created_before(mut self, time: std::time::SystemTime) -> Self {
        self.created_before = Some(time);
        self
    }
    
    pub fn include_inactive(mut self) -> Self {
        self.active_only = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let mut manager = EntityManager::new(1000);
        
        let entity1 = manager.create_entity().unwrap();
        let entity2 = manager.create_entity().unwrap();
        
        assert_ne!(entity1, entity2);
        assert_eq!(manager.get_entity_count(), 2);
    }
    
    #[test]
    fn test_entity_destruction() {
        let mut manager = EntityManager::new(1000);
        
        let entity = manager.create_entity().unwrap();
        assert!(manager.exists(entity));
        
        manager.destroy_entity(entity).unwrap();
        manager.cleanup_destroyed_entities();
        
        assert!(!manager.exists(entity));
        assert_eq!(manager.get_entity_count(), 0);
    }
    
    #[test]
    fn test_entity_tags() {
        let mut manager = EntityManager::new(1000);
        let entity = manager.create_entity().unwrap();
        
        manager.add_tag(entity, "player".to_string()).unwrap();
        manager.add_tag(entity, "movable".to_string()).unwrap();
        
        assert!(manager.has_tag(entity, "player"));
        assert!(manager.has_tag(entity, "movable"));
        assert!(!manager.has_tag(entity, "enemy"));
        
        let entities_with_tag = manager.find_entities_with_tag("player");
        assert_eq!(entities_with_tag.len(), 1);
        assert_eq!(entities_with_tag[0], entity);
    }
    
    #[test]
    fn test_entity_query() {
        let mut manager = EntityManager::new(1000);
        
        let entity1 = manager.create_entity().unwrap();
        let entity2 = manager.create_entity().unwrap();
        
        manager.add_tag(entity1, "player".to_string()).unwrap();
        manager.add_tag(entity2, "enemy".to_string()).unwrap();
        
        let query = EntityQuery::new()
            .with_tag("player".to_string());
        
        let results = manager.query_entities(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entity1);
    }
    
    #[test]
    fn test_id_reuse() {
        let mut manager = EntityManager::new(1000);
        
        let entity1 = manager.create_entity().unwrap();
        let entity2 = manager.create_entity().unwrap();
        
        manager.destroy_entity(entity1).unwrap();
        manager.cleanup_destroyed_entities();
        
        let entity3 = manager.create_entity().unwrap();
        
        // ID应该被复用
        assert_eq!(entity3, entity1);
        assert_ne!(entity3, entity2);
    }
    
    #[test]
    fn test_entity_statistics() {
        let mut manager = EntityManager::new(1000);
        
        let entity1 = manager.create_entity().unwrap();
        let entity2 = manager.create_entity().unwrap();
        
        manager.add_tag(entity1, "player".to_string()).unwrap();
        manager.add_tag(entity1, "alive".to_string()).unwrap();
        manager.add_tag(entity2, "enemy".to_string()).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_entities, 2);
        assert_eq!(stats.active_entities, 2);
        assert!(stats.tag_usage.contains_key("player"));
        assert!(stats.tag_usage.contains_key("enemy"));
    }
}