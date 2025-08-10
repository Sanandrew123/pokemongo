// 组件管理器 (简化版本用于编译)
use std::collections::HashMap;
use std::any::{Any, TypeId};
use crate::core::error::GameError;
use super::{EntityId, ComponentId, Component};

pub struct ComponentManager {
    components: HashMap<EntityId, HashMap<ComponentId, Box<dyn Component>>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, component: T) -> Result<(), GameError> {
        let type_id = TypeId::of::<T>();
        self.components
            .entry(entity_id)
            .or_insert_with(HashMap::new)
            .insert(type_id, Box::new(component));
        Ok(())
    }
    
    pub fn remove_component(&mut self, entity_id: EntityId, component_id: ComponentId) -> Result<(), GameError> {
        if let Some(entity_components) = self.components.get_mut(&entity_id) {
            entity_components.remove(&component_id);
        }
        Ok(())
    }
    
    pub fn get_component<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&entity_id)?
            .get(&type_id)?
            .as_any()
            .downcast_ref::<T>()
    }
    
    pub fn get_component_mut<T: Component>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&entity_id)?
            .get_mut(&type_id)?
            .as_any_mut()
            .downcast_mut::<T>()
    }
    
    pub fn has_component<T: Component>(&self, entity_id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&entity_id)
            .map(|components| components.contains_key(&type_id))
            .unwrap_or(false)
    }
    
    pub fn has_component_by_id(&self, entity_id: EntityId, component_id: ComponentId) -> bool {
        self.components
            .get(&entity_id)
            .map(|components| components.contains_key(&component_id))
            .unwrap_or(false)
    }
    
    pub fn get_entity_components(&self, entity_id: EntityId) -> Vec<ComponentId> {
        self.components
            .get(&entity_id)
            .map(|components| components.keys().copied().collect())
            .unwrap_or_default()
    }
    
    pub fn get_component_stats(&self) -> HashMap<ComponentId, usize> {
        let mut stats = HashMap::new();
        for components in self.components.values() {
            for &component_id in components.keys() {
                *stats.entry(component_id).or_insert(0) += 1;
            }
        }
        stats
    }
}