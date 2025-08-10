// ECS世界辅助模块 (简化版本用于编译)
use crate::core::error::GameError;

pub struct WorldBuilder {
    max_entities: usize,
    enable_parallel: bool,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            max_entities: 100000,
            enable_parallel: true,
        }
    }
    
    pub fn max_entities(mut self, count: usize) -> Self {
        self.max_entities = count;
        self
    }
    
    pub fn build(self) -> Result<super::ECSWorld, GameError> {
        let config = super::ECSConfig {
            max_entities: self.max_entities,
            max_components_per_entity: 32,
            enable_parallel_systems: self.enable_parallel,
            enable_query_caching: true,
            statistics_enabled: true,
            debug_mode: false,
        };
        
        Ok(super::ECSWorld::with_config(config))
    }
}

impl Default for WorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}