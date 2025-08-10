// 系统管理器 (简化版本用于编译)
use std::collections::HashMap;
use crate::core::error::GameError;
use super::{SystemId, System, ECSWorld};

pub struct SystemManager {
    systems: HashMap<SystemId, Box<dyn System>>,
    next_id: SystemId,
    enabled_systems: HashMap<SystemId, bool>,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub id: SystemId,
    pub name: String,
    pub enabled: bool,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            next_id: 1,
            enabled_systems: HashMap::new(),
        }
    }
    
    pub fn register_system(&mut self, system: Box<dyn System>) -> Result<SystemId, GameError> {
        let id = self.next_id;
        self.next_id += 1;
        
        self.systems.insert(id, system);
        self.enabled_systems.insert(id, true);
        
        Ok(id)
    }
    
    pub fn remove_system(&mut self, system_id: SystemId) -> Result<(), GameError> {
        self.systems.remove(&system_id);
        self.enabled_systems.remove(&system_id);
        Ok(())
    }
    
    pub fn set_system_enabled(&mut self, system_id: SystemId, enabled: bool) -> Result<(), GameError> {
        if self.systems.contains_key(&system_id) {
            self.enabled_systems.insert(system_id, enabled);
            Ok(())
        } else {
            Err(GameError::ECS(format!("系统不存在: {}", system_id)))
        }
    }
    
    pub fn update(&mut self, world: &mut ECSWorld, delta_time: f32) -> Result<(), GameError> {
        for (&system_id, system) in &mut self.systems {
            if self.enabled_systems.get(&system_id).copied().unwrap_or(true) {
                system.update(world, delta_time)?;
            }
        }
        Ok(())
    }
    
    pub fn get_system_info(&self, system_id: SystemId) -> Option<SystemInfo> {
        if let Some(system) = self.systems.get(&system_id) {
            Some(SystemInfo {
                id: system_id,
                name: system.name().to_string(),
                enabled: self.enabled_systems.get(&system_id).copied().unwrap_or(true),
            })
        } else {
            None
        }
    }
}