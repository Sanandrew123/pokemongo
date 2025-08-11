/*
 * Pokemon Go - Creature Engine Module
 * 开发心理过程:
 * 1. 设计先进的生物引擎系统,支持程序化生成和复杂的生物特性
 * 2. 整合模板系统、进化树、平衡性和稀有度管理
 * 3. 实现变异系统和特性系统,让每个生物都独一无二
 * 4. 提供完整的数据验证和错误处理机制
 * 5. 支持可扩展的生物属性和行为系统
 */

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub mod generator;
pub mod templates;
pub mod evolution_tree;
pub mod balance_system;
pub mod rarity_system;
pub mod trait_system;
pub mod mutation;
pub mod validator;

pub use generator::*;
pub use templates::*;
pub use evolution_tree::*;
pub use balance_system::*;
pub use rarity_system::*;
pub use trait_system::*;
pub use mutation::*;
pub use validator::*;

#[derive(Debug, Clone, Error)]
pub enum CreatureEngineError {
    #[error("Invalid creature template: {0}")]
    InvalidTemplate(String),
    #[error("Evolution tree error: {0}")]
    EvolutionError(String),
    #[error("Balance validation failed: {0}")]
    BalanceError(String),
    #[error("Rarity system error: {0}")]
    RarityError(String),
    #[error("Trait system error: {0}")]
    TraitError(String),
    #[error("Mutation error: {0}")]
    MutationError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Resource loading error: {0}")]
    ResourceError(String),
}

pub type CreatureEngineResult<T> = Result<T, CreatureEngineError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureConfig {
    pub max_base_stats: u32,
    pub min_base_stats: u32,
    pub max_level: u8,
    pub evolution_requirements: EvolutionRequirements,
    pub mutation_rates: MutationRates,
    pub rarity_distribution: RarityDistribution,
    pub trait_pools: TraitPools,
    pub balance_constraints: BalanceConstraints,
}

impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            max_base_stats: 780,
            min_base_stats: 200,
            max_level: 100,
            evolution_requirements: EvolutionRequirements::default(),
            mutation_rates: MutationRates::default(),
            rarity_distribution: RarityDistribution::default(),
            trait_pools: TraitPools::default(),
            balance_constraints: BalanceConstraints::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatureEngine {
    config: CreatureConfig,
    generator: CreatureGenerator,
    templates: TemplateManager,
    evolution_tree: EvolutionTree,
    balance_system: BalanceSystem,
    rarity_system: RaritySystem,
    trait_system: TraitSystem,
    mutation_system: MutationSystem,
    validator: CreatureValidator,
}

impl CreatureEngine {
    pub fn new(config: CreatureConfig) -> CreatureEngineResult<Self> {
        let generator = CreatureGenerator::new(&config)?;
        let templates = TemplateManager::new()?;
        let evolution_tree = EvolutionTree::new(&config.evolution_requirements)?;
        let balance_system = BalanceSystem::new(&config.balance_constraints)?;
        let rarity_system = RaritySystem::new(&config.rarity_distribution)?;
        let trait_system = TraitSystem::new(&config.trait_pools)?;
        let mutation_system = MutationSystem::new(&config.mutation_rates)?;
        let validator = CreatureValidator::new(&config)?;

        Ok(Self {
            config,
            generator,
            templates,
            evolution_tree,
            balance_system,
            rarity_system,
            trait_system,
            mutation_system,
            validator,
        })
    }

    pub fn generate_creature(&mut self, template_id: &str, level: u8) -> CreatureEngineResult<GeneratedCreature> {
        let template = self.templates.get_template(template_id)?;
        let rarity = self.rarity_system.determine_rarity(&template)?;
        let traits = self.trait_system.generate_traits(&template, rarity)?;
        let mut creature = self.generator.generate_from_template(&template, level, rarity, traits)?;
        
        if self.mutation_system.should_mutate(&creature)? {
            creature = self.mutation_system.apply_mutations(creature)?;
        }

        self.validator.validate_creature(&creature)?;
        self.balance_system.apply_balance(&mut creature)?;
        
        Ok(creature)
    }

    pub fn evolve_creature(&mut self, creature: &GeneratedCreature) -> CreatureEngineResult<Option<GeneratedCreature>> {
        if !self.evolution_tree.can_evolve(creature)? {
            return Ok(None);
        }

        let evolution_id = self.evolution_tree.get_next_evolution(&creature.template_id)?;
        let evolution_template = self.templates.get_template(&evolution_id)?;
        
        let evolved = self.generator.evolve_creature(creature, &evolution_template)?;
        self.validator.validate_creature(&evolved)?;
        
        Ok(Some(evolved))
    }

    pub fn get_creature_stats(&self, creature: &GeneratedCreature) -> CreatureEngineResult<CreatureStats> {
        self.generator.calculate_stats(creature)
    }

    pub fn mutate_creature(&mut self, creature: &GeneratedCreature) -> CreatureEngineResult<GeneratedCreature> {
        let mutated = self.mutation_system.force_mutation(creature.clone())?;
        self.validator.validate_creature(&mutated)?;
        self.balance_system.apply_balance(&mut mutated.clone())?;
        Ok(mutated)
    }

    pub fn validate_creature_balance(&self, creature: &GeneratedCreature) -> CreatureEngineResult<BalanceReport> {
        self.balance_system.analyze_creature(creature)
    }

    pub fn get_available_traits(&self, template_id: &str) -> CreatureEngineResult<Vec<TraitDefinition>> {
        let template = self.templates.get_template(template_id)?;
        self.trait_system.get_available_traits(&template)
    }

    pub fn reload_templates(&mut self) -> CreatureEngineResult<()> {
        self.templates.reload_all_templates()
    }

    pub fn export_creature_data(&self, creature: &GeneratedCreature) -> CreatureEngineResult<String> {
        self.validator.export_creature_data(creature)
    }

    pub fn import_creature_data(&mut self, data: &str) -> CreatureEngineResult<GeneratedCreature> {
        let creature = self.validator.import_creature_data(data)?;
        self.validator.validate_creature(&creature)?;
        Ok(creature)
    }

    pub fn get_engine_stats(&self) -> EngineStats {
        EngineStats {
            total_templates: self.templates.template_count(),
            active_mutations: self.mutation_system.active_mutation_count(),
            evolution_chains: self.evolution_tree.chain_count(),
            trait_definitions: self.trait_system.trait_count(),
            balance_violations: self.balance_system.violation_count(),
            rarity_tiers: self.rarity_system.tier_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCreature {
    pub id: String,
    pub template_id: String,
    pub level: u8,
    pub rarity: CreatureRarity,
    pub base_stats: HashMap<String, u32>,
    pub traits: Vec<CreatureTrait>,
    pub mutations: Vec<Mutation>,
    pub generation_seed: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureStats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub sp_attack: u32,
    pub sp_defense: u32,
    pub speed: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_templates: usize,
    pub active_mutations: usize,
    pub evolution_chains: usize,
    pub trait_definitions: usize,
    pub balance_violations: usize,
    pub rarity_tiers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_engine_creation() {
        let config = CreatureConfig::default();
        let engine = CreatureEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_creature_generation() {
        let config = CreatureConfig::default();
        let mut engine = CreatureEngine::new(config).unwrap();
        // Would need mock templates for full testing
    }

    #[test]
    fn test_engine_stats() {
        let config = CreatureConfig::default();
        let engine = CreatureEngine::new(config).unwrap();
        let stats = engine.get_engine_stats();
        assert_eq!(stats.total_templates, 0); // No templates loaded yet
    }
}