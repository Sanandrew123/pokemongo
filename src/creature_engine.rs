// 生物创造引擎
// 开发心理：提供强大的生物程序化生成系统，支持模板、进化树、平衡性检查
// 设计原则：可扩展性、数据驱动、平衡性、创新性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use log::{debug, info, warn, error};
use crate::core::error::GameError;
use crate::pokemon::{PokemonType, BaseStats, SpeciesId};

// 生物创造引擎
pub struct CreatureEngine {
    // 模板系统
    templates: HashMap<String, CreatureTemplate>,
    
    // 生成器
    generator: CreatureGenerator,
    
    // 平衡系统
    balance_system: BalanceSystem,
    
    // 进化树构建器
    evolution_builder: EvolutionTreeBuilder,
    
    // 稀有度系统
    rarity_system: RaritySystem,
    
    // 特性系统
    trait_system: TraitSystem,
    
    // 变异系统
    mutation_system: MutationSystem,
    
    // 验证器
    validator: CreatureValidator,
    
    // 统计信息
    statistics: CreatureEngineStats,
}

// 生物模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CreatureCategory,
    pub base_type: PokemonType,
    pub secondary_type: Option<PokemonType>,
    pub base_stats: BaseStats,
    pub stat_ranges: StatRanges,
    pub abilities: Vec<String>,
    pub possible_moves: Vec<String>,
    pub evolution_requirements: Vec<EvolutionRequirement>,
    pub habitat: Habitat,
    pub rarity_tier: RarityTier,
    pub size_category: SizeCategory,
    pub weight_range: (f32, f32),
    pub height_range: (f32, f32),
    pub generation_rules: GenerationRules,
}

// 生物分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatureCategory {
    Beast,          // 野兽
    Dragon,         // 龙类
    Spirit,         // 精灵
    Elemental,      // 元素
    Mechanical,     // 机械
    Plant,          // 植物
    Aquatic,        // 水栖
    Aerial,         // 飞行
    Mythical,       // 神话
    Artificial,     // 人造
}

// 属性值范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRanges {
    pub hp: (u16, u16),
    pub attack: (u16, u16),
    pub defense: (u16, u16),
    pub sp_attack: (u16, u16),
    pub sp_defense: (u16, u16),
    pub speed: (u16, u16),
}

// 进化需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionRequirement {
    Level(u8),
    Item(String),
    Trade,
    Friendship(u8),
    TimeOfDay(TimeOfDay),
    Location(String),
    Stats(StatRequirement),
    Custom(String, HashMap<String, String>),
}

// 时间段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Day,
    Night,
    Dawn,
    Dusk,
}

// 属性需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRequirement {
    pub stat_type: String,
    pub minimum_value: u16,
}

// 栖息地
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habitat {
    pub primary: String,
    pub secondary: Vec<String>,
    pub preferred_weather: Vec<String>,
    pub altitude_range: Option<(i32, i32)>,
    pub temperature_range: Option<(f32, f32)>,
}

// 稀有度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RarityTier {
    Common = 1,
    Uncommon = 2,
    Rare = 3,
    Epic = 4,
    Legendary = 5,
    Mythical = 6,
}

// 大小分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeCategory {
    Tiny,       // 微型
    Small,      // 小型
    Medium,     // 中型
    Large,      // 大型
    Huge,       // 巨型
    Colossal,   // 超巨型
}

// 生成规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRules {
    pub allow_stat_mutation: bool,
    pub allow_type_variation: bool,
    pub allow_ability_mixing: bool,
    pub preserve_theme: bool,
    pub mutation_rate: f32,
    pub custom_constraints: HashMap<String, String>,
}

// 生物生成器
pub struct CreatureGenerator {
    templates: HashMap<String, CreatureTemplate>,
    generation_algorithms: HashMap<String, GenerationAlgorithm>,
    random_seed: Option<u64>,
}

// 生成算法
pub enum GenerationAlgorithm {
    Template,           // 基于模板
    Hybrid,             // 混合生成
    Random,             // 随机生成
    Evolutionary,       // 进化式生成
    GeneticAlgorithm,   // 遗传算法
}

// 平衡系统
pub struct BalanceSystem {
    stat_caps: HashMap<String, u16>,
    type_balance_matrix: HashMap<(PokemonType, PokemonType), f32>,
    power_scaling_rules: PowerScalingRules,
    tier_restrictions: HashMap<RarityTier, StatLimits>,
}

// 功率缩放规则
#[derive(Debug, Clone)]
pub struct PowerScalingRules {
    pub base_stat_total_limits: HashMap<RarityTier, (u16, u16)>,
    pub individual_stat_caps: HashMap<String, u16>,
    pub type_effectiveness_modifiers: HashMap<PokemonType, f32>,
}

// 属性限制
#[derive(Debug, Clone)]
pub struct StatLimits {
    pub total_minimum: u16,
    pub total_maximum: u16,
    pub individual_maximum: u16,
    pub required_distribution: Option<StatDistribution>,
}

// 属性分布
#[derive(Debug, Clone)]
pub struct StatDistribution {
    pub tank_ratio: f32,      // 防御型比例
    pub sweeper_ratio: f32,   // 攻击型比例
    pub support_ratio: f32,   // 辅助型比例
    pub balanced_ratio: f32,  // 平衡型比例
}

// 进化树构建器
pub struct EvolutionTreeBuilder {
    evolution_chains: HashMap<String, EvolutionChain>,
    branching_rules: Vec<BranchingRule>,
}

// 进化链
#[derive(Debug, Clone)]
pub struct EvolutionChain {
    pub base_form: String,
    pub stages: Vec<EvolutionStage>,
    pub alternate_forms: Vec<AlternateForm>,
    pub mega_evolutions: Vec<MegaEvolution>,
}

// 进化阶段
#[derive(Debug, Clone)]
pub struct EvolutionStage {
    pub stage_number: u8,
    pub requirements: Vec<EvolutionRequirement>,
    pub stat_changes: StatChanges,
    pub type_changes: TypeChanges,
    pub new_abilities: Vec<String>,
    pub appearance_changes: AppearanceChanges,
}

// 属性变化
#[derive(Debug, Clone)]
pub struct StatChanges {
    pub hp_modifier: f32,
    pub attack_modifier: f32,
    pub defense_modifier: f32,
    pub sp_attack_modifier: f32,
    pub sp_defense_modifier: f32,
    pub speed_modifier: f32,
}

// 类型变化
#[derive(Debug, Clone)]
pub struct TypeChanges {
    pub primary_type: Option<PokemonType>,
    pub secondary_type: Option<PokemonType>,
    pub add_secondary: bool,
    pub remove_secondary: bool,
}

// 外观变化
#[derive(Debug, Clone)]
pub struct AppearanceChanges {
    pub size_multiplier: f32,
    pub weight_multiplier: f32,
    pub color_shifts: Vec<ColorShift>,
    pub pattern_changes: Vec<PatternChange>,
}

// 颜色偏移
#[derive(Debug, Clone)]
pub struct ColorShift {
    pub region: String,
    pub hue_shift: f32,
    pub saturation_change: f32,
    pub brightness_change: f32,
}

// 图案变化
#[derive(Debug, Clone)]
pub struct PatternChange {
    pub pattern_type: String,
    pub intensity: f32,
    pub coverage: f32,
}

// 替代形态
#[derive(Debug, Clone)]
pub struct AlternateForm {
    pub name: String,
    pub trigger_condition: String,
    pub stat_modifications: StatChanges,
    pub ability_changes: Vec<String>,
}

// 超级进化
#[derive(Debug, Clone)]
pub struct MegaEvolution {
    pub name: String,
    pub required_item: String,
    pub stat_boosts: StatChanges,
    pub ability_override: String,
    pub temporary: bool,
}

// 分支规则
#[derive(Debug, Clone)]
pub struct BranchingRule {
    pub condition_type: String,
    pub parameters: HashMap<String, String>,
    pub target_branches: Vec<String>,
}

// 稀有度系统
pub struct RaritySystem {
    tier_probabilities: HashMap<RarityTier, f32>,
    spawn_modifiers: HashMap<String, f32>,
    legendary_restrictions: LegendaryRestrictions,
}

// 传说限制
#[derive(Debug, Clone)]
pub struct LegendaryRestrictions {
    pub max_legendaries_per_region: u8,
    pub unique_legendary_names: bool,
    pub power_requirements: StatLimits,
    pub special_abilities_required: bool,
}

// 特性系统
pub struct TraitSystem {
    available_traits: HashMap<String, Trait>,
    trait_combinations: Vec<TraitCombination>,
    inheritance_rules: HashMap<String, InheritanceRule>,
}

// 特性
#[derive(Debug, Clone)]
pub struct Trait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<TraitEffect>,
    pub rarity: RarityTier,
    pub prerequisites: Vec<String>,
}

// 特性效果
#[derive(Debug, Clone)]
pub enum TraitEffect {
    StatModifier { stat: String, multiplier: f32 },
    TypeResistance { pokemon_type: PokemonType, resistance: f32 },
    AbilityGranted { ability_name: String },
    BehaviorModification { behavior: String, intensity: f32 },
    AppearanceChange { change_type: String, parameters: HashMap<String, String> },
}

// 特性组合
#[derive(Debug, Clone)]
pub struct TraitCombination {
    pub traits: Vec<String>,
    pub synergy_bonus: f32,
    pub conflicts: Vec<String>,
}

// 继承规则
#[derive(Debug, Clone)]
pub struct InheritanceRule {
    pub trait_name: String,
    pub inheritance_chance: f32,
    pub mutation_chance: f32,
    pub dominant: bool,
}

// 变异系统
pub struct MutationSystem {
    mutation_types: HashMap<String, MutationType>,
    mutation_triggers: Vec<MutationTrigger>,
    stability_factors: HashMap<String, f32>,
}

// 变异类型
#[derive(Debug, Clone)]
pub enum MutationType {
    StatShift { stat: String, range: (f32, f32) },
    TypeChange { from: PokemonType, to: Vec<PokemonType>, probability: f32 },
    AbilityMutation { base_ability: String, variants: Vec<String> },
    SizeVariation { scale_range: (f32, f32) },
    ColorVariation { intensity: f32 },
    BehaviorChange { new_behavior: String },
}

// 变异触发器
#[derive(Debug, Clone)]
pub struct MutationTrigger {
    pub trigger_type: String,
    pub conditions: HashMap<String, String>,
    pub mutation_probability: f32,
    pub applicable_mutations: Vec<String>,
}

// 生物验证器
pub struct CreatureValidator {
    validation_rules: Vec<ValidationRule>,
    error_tolerance: f32,
    auto_fix_enabled: bool,
}

// 验证规则
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_name: String,
    pub rule_type: ValidationType,
    pub severity: ValidationSeverity,
    pub auto_fixable: bool,
}

// 验证类型
#[derive(Debug, Clone)]
pub enum ValidationType {
    StatTotalRange { min: u16, max: u16 },
    TypeCombinationValid,
    AbilityCompatibility,
    EvolutionChainConsistency,
    RarityAppropriate,
    BalanceCheck,
}

// 验证严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// 引擎统计
#[derive(Debug, Clone, Default)]
pub struct CreatureEngineStats {
    pub creatures_generated: u64,
    pub templates_created: u64,
    pub evolution_chains_built: u64,
    pub mutations_applied: u64,
    pub validation_passes: u64,
    pub validation_failures: u64,
    pub average_generation_time: f64,
}

// 生成的生物
#[derive(Debug, Clone)]
pub struct GeneratedCreature {
    pub id: String,
    pub name: String,
    pub template_id: Option<String>,
    pub category: CreatureCategory,
    pub primary_type: PokemonType,
    pub secondary_type: Option<PokemonType>,
    pub stats: BaseStats,
    pub abilities: Vec<String>,
    pub moves: Vec<String>,
    pub traits: Vec<String>,
    pub rarity: RarityTier,
    pub size: SizeCategory,
    pub weight: f32,
    pub height: f32,
    pub appearance: CreatureAppearance,
    pub habitat: Habitat,
    pub generation_metadata: GenerationMetadata,
}

// 生物外观
#[derive(Debug, Clone)]
pub struct CreatureAppearance {
    pub base_colors: Vec<String>,
    pub patterns: Vec<String>,
    pub special_features: Vec<String>,
    pub texture_variations: HashMap<String, String>,
}

// 生成元数据
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub generation_algorithm: String,
    pub parent_templates: Vec<String>,
    pub mutations_applied: Vec<String>,
    pub generation_time: std::time::Duration,
    pub validation_score: f32,
    pub created_at: std::time::SystemTime,
}

impl CreatureEngine {
    pub fn new() -> Result<Self, GameError> {
        Ok(Self {
            templates: HashMap::new(),
            generator: CreatureGenerator::new(),
            balance_system: BalanceSystem::new()?,
            evolution_builder: EvolutionTreeBuilder::new(),
            rarity_system: RaritySystem::new(),
            trait_system: TraitSystem::new(),
            mutation_system: MutationSystem::new(),
            validator: CreatureValidator::new(),
            statistics: CreatureEngineStats::default(),
        })
    }
    
    // 加载模板
    pub fn load_template(&mut self, template: CreatureTemplate) -> Result<(), GameError> {
        let template_id = template.id.clone();
        self.templates.insert(template_id.clone(), template);
        self.statistics.templates_created += 1;
        
        info!("加载生物模板: {}", template_id);
        Ok(())
    }
    
    // 生成生物
    pub fn generate_creature(&mut self, template_id: &str, options: GenerationOptions) -> Result<GeneratedCreature, GameError> {
        let start_time = std::time::Instant::now();
        
        let template = self.templates.get(template_id)
            .ok_or_else(|| GameError::ConfigError(format!("模板不存在: {}", template_id)))?;
        
        // 生成基础生物
        let mut creature = self.generator.generate_from_template(template, &options)?;
        
        // 应用变异
        if options.allow_mutations {
            self.mutation_system.apply_mutations(&mut creature, options.mutation_intensity)?;
            self.statistics.mutations_applied += 1;
        }
        
        // 验证生物
        let validation_result = self.validator.validate_creature(&creature)?;
        if !validation_result.is_valid() {
            if options.strict_validation {
                return Err(GameError::ConfigError("生物验证失败".to_string()));
            } else {
                warn!("生物验证警告: {:?}", validation_result);
            }
        }
        
        // 更新统计
        let generation_time = start_time.elapsed();
        self.statistics.creatures_generated += 1;
        self.statistics.average_generation_time = 
            (self.statistics.average_generation_time * (self.statistics.creatures_generated - 1) as f64 + 
             generation_time.as_secs_f64()) / self.statistics.creatures_generated as f64;
        
        creature.generation_metadata.generation_time = generation_time;
        
        info!("生成生物成功: {} (模板: {})", creature.name, template_id);
        Ok(creature)
    }
    
    // 创建进化链
    pub fn create_evolution_chain(&mut self, base_template_id: &str, chain_config: EvolutionChainConfig) -> Result<EvolutionChain, GameError> {
        let evolution_chain = self.evolution_builder.build_chain(base_template_id, chain_config)?;
        self.statistics.evolution_chains_built += 1;
        
        info!("创建进化链: {} -> {} 阶段", base_template_id, evolution_chain.stages.len());
        Ok(evolution_chain)
    }
    
    // 混合生物
    pub fn hybridize_creatures(&mut self, parent1_id: &str, parent2_id: &str, options: HybridizationOptions) -> Result<GeneratedCreature, GameError> {
        let parent1 = self.templates.get(parent1_id)
            .ok_or_else(|| GameError::ConfigError(format!("父本模板不存在: {}", parent1_id)))?;
        let parent2 = self.templates.get(parent2_id)
            .ok_or_else(|| GameError::ConfigError(format!("母本模板不存在: {}", parent2_id)))?;
        
        let hybrid = self.generator.create_hybrid(parent1, parent2, &options)?;
        
        info!("生物混合成功: {} + {} = {}", parent1_id, parent2_id, hybrid.name);
        Ok(hybrid)
    }
    
    // 获取统计信息
    pub fn get_statistics(&self) -> &CreatureEngineStats {
        &self.statistics
    }
}

// 生成选项
#[derive(Debug, Clone)]
pub struct GenerationOptions {
    pub allow_mutations: bool,
    pub mutation_intensity: f32,
    pub preferred_rarity: Option<RarityTier>,
    pub size_preferences: Option<SizeCategory>,
    pub type_restrictions: Vec<PokemonType>,
    pub stat_focus: Option<StatFocus>,
    pub strict_validation: bool,
    pub preserve_theme: bool,
}

// 属性焦点
#[derive(Debug, Clone, Copy)]
pub enum StatFocus {
    Offensive,
    Defensive, 
    Speed,
    Balanced,
    Specialized(String),
}

// 进化链配置
#[derive(Debug, Clone)]
pub struct EvolutionChainConfig {
    pub max_stages: u8,
    pub allow_branching: bool,
    pub stat_growth_rate: f32,
    pub type_stability: bool,
    pub appearance_continuity: f32,
}

// 杂交选项
#[derive(Debug, Clone)]
pub struct HybridizationOptions {
    pub stat_inheritance_ratio: f32,  // 0.5 = 50/50, 0.7 = 70% parent1
    pub type_combination_mode: TypeCombinationMode,
    pub ability_mixing_allowed: bool,
    pub preserve_rarity: bool,
    pub mutation_chance: f32,
}

// 类型组合模式
#[derive(Debug, Clone, Copy)]
pub enum TypeCombinationMode {
    Dominant,       // 使用主导类型
    Blend,          // 类型混合
    Random,         // 随机选择
    Novel,          // 创造新组合
}

// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<ValidationWarning>,
    pub errors: Vec<ValidationError>,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
    pub rule_name: String,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub rule_name: String,
    pub severity: ValidationSeverity,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }
}

// 各个子系统的基础实现
impl CreatureGenerator {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            generation_algorithms: HashMap::new(),
            random_seed: None,
        }
    }
    
    pub fn generate_from_template(&self, template: &CreatureTemplate, options: &GenerationOptions) -> Result<GeneratedCreature, GameError> {
        let mut rng = thread_rng();
        
        // 生成属性值
        let stats = self.generate_stats(&template.stat_ranges, &mut rng);
        
        // 选择能力
        let abilities = self.select_abilities(&template.abilities, 1, &mut rng);
        
        // 生成外观
        let appearance = self.generate_appearance(template, &mut rng);
        
        Ok(GeneratedCreature {
            id: format!("gen_{}", rng.gen::<u32>()),
            name: self.generate_name(&template.name, &mut rng),
            template_id: Some(template.id.clone()),
            category: template.category,
            primary_type: template.base_type,
            secondary_type: template.secondary_type,
            stats,
            abilities,
            moves: self.select_moves(&template.possible_moves, 4, &mut rng),
            traits: Vec::new(),
            rarity: template.rarity_tier,
            size: template.size_category,
            weight: rng.gen_range(template.weight_range.0..=template.weight_range.1),
            height: rng.gen_range(template.height_range.0..=template.height_range.1),
            appearance,
            habitat: template.habitat.clone(),
            generation_metadata: GenerationMetadata {
                generation_algorithm: "template".to_string(),
                parent_templates: vec![template.id.clone()],
                mutations_applied: Vec::new(),
                generation_time: std::time::Duration::default(),
                validation_score: 1.0,
                created_at: std::time::SystemTime::now(),
            },
        })
    }
    
    pub fn create_hybrid(&self, parent1: &CreatureTemplate, parent2: &CreatureTemplate, options: &HybridizationOptions) -> Result<GeneratedCreature, GameError> {
        let mut rng = thread_rng();
        
        // 混合属性
        let stats = self.blend_stats(&parent1.base_stats, &parent2.base_stats, options.stat_inheritance_ratio);
        
        // 类型组合
        let (primary_type, secondary_type) = self.combine_types(
            parent1.base_type, parent1.secondary_type,
            parent2.base_type, parent2.secondary_type,
            options.type_combination_mode
        );
        
        Ok(GeneratedCreature {
            id: format!("hybrid_{}", rng.gen::<u32>()),
            name: format!("{}×{}", parent1.name, parent2.name),
            template_id: None,
            category: if rng.gen::<f32>() < 0.5 { parent1.category } else { parent2.category },
            primary_type,
            secondary_type,
            stats,
            abilities: self.mix_abilities(&parent1.abilities, &parent2.abilities, &mut rng),
            moves: self.mix_moves(&parent1.possible_moves, &parent2.possible_moves, &mut rng),
            traits: Vec::new(),
            rarity: if options.preserve_rarity { 
                std::cmp::max(parent1.rarity_tier, parent2.rarity_tier) 
            } else { 
                parent1.rarity_tier 
            },
            size: if rng.gen::<f32>() < 0.5 { parent1.size_category } else { parent2.size_category },
            weight: (parent1.weight_range.0 + parent2.weight_range.0) / 2.0,
            height: (parent1.height_range.0 + parent2.height_range.0) / 2.0,
            appearance: self.blend_appearance(parent1, parent2, &mut rng),
            habitat: parent1.habitat.clone(), // 简化：使用父本1的栖息地
            generation_metadata: GenerationMetadata {
                generation_algorithm: "hybrid".to_string(),
                parent_templates: vec![parent1.id.clone(), parent2.id.clone()],
                mutations_applied: Vec::new(),
                generation_time: std::time::Duration::default(),
                validation_score: 0.8,
                created_at: std::time::SystemTime::now(),
            },
        })
    }
    
    // 辅助方法
    fn generate_stats(&self, ranges: &StatRanges, rng: &mut impl Rng) -> BaseStats {
        BaseStats {
            hp: rng.gen_range(ranges.hp.0..=ranges.hp.1),
            attack: rng.gen_range(ranges.attack.0..=ranges.attack.1),
            defense: rng.gen_range(ranges.defense.0..=ranges.defense.1),
            sp_attack: rng.gen_range(ranges.sp_attack.0..=ranges.sp_attack.1),
            sp_defense: rng.gen_range(ranges.sp_defense.0..=ranges.sp_defense.1),
            speed: rng.gen_range(ranges.speed.0..=ranges.speed.1),
        }
    }
    
    fn select_abilities(&self, available: &[String], count: usize, rng: &mut impl Rng) -> Vec<String> {
        if available.len() <= count {
            available.to_vec()
        } else {
            let mut selected = Vec::new();
            let mut indices: Vec<usize> = (0..available.len()).collect();
            
            for _ in 0..count {
                if !indices.is_empty() {
                    let idx = rng.gen_range(0..indices.len());
                    let ability_idx = indices.remove(idx);
                    selected.push(available[ability_idx].clone());
                }
            }
            
            selected
        }
    }
    
    fn select_moves(&self, available: &[String], count: usize, rng: &mut impl Rng) -> Vec<String> {
        self.select_abilities(available, count, rng) // 相同的逻辑
    }
    
    fn generate_name(&self, base_name: &str, rng: &mut impl Rng) -> String {
        let prefixes = ["Neo", "Alpha", "Beta", "Gamma", "Delta", "Omega"];
        let suffixes = ["X", "Prime", "Plus", "Max", "Ultra"];
        
        match rng.gen_range(0..3) {
            0 => format!("{}{}", prefixes[rng.gen_range(0..prefixes.len())], base_name),
            1 => format!("{}{}", base_name, suffixes[rng.gen_range(0..suffixes.len())]),
            _ => base_name.to_string(),
        }
    }
    
    fn generate_appearance(&self, template: &CreatureTemplate, rng: &mut impl Rng) -> CreatureAppearance {
        let colors = vec!["Red".to_string(), "Blue".to_string(), "Green".to_string(), 
                         "Yellow".to_string(), "Purple".to_string()];
        let patterns = vec!["Stripes".to_string(), "Spots".to_string(), "Solid".to_string()];
        
        CreatureAppearance {
            base_colors: vec![colors[rng.gen_range(0..colors.len())].clone()],
            patterns: vec![patterns[rng.gen_range(0..patterns.len())].clone()],
            special_features: Vec::new(),
            texture_variations: HashMap::new(),
        }
    }
    
    fn blend_stats(&self, stats1: &BaseStats, stats2: &BaseStats, ratio: f32) -> BaseStats {
        BaseStats {
            hp: ((stats1.hp as f32 * ratio) + (stats2.hp as f32 * (1.0 - ratio))) as u16,
            attack: ((stats1.attack as f32 * ratio) + (stats2.attack as f32 * (1.0 - ratio))) as u16,
            defense: ((stats1.defense as f32 * ratio) + (stats2.defense as f32 * (1.0 - ratio))) as u16,
            sp_attack: ((stats1.sp_attack as f32 * ratio) + (stats2.sp_attack as f32 * (1.0 - ratio))) as u16,
            sp_defense: ((stats1.sp_defense as f32 * ratio) + (stats2.sp_defense as f32 * (1.0 - ratio))) as u16,
            speed: ((stats1.speed as f32 * ratio) + (stats2.speed as f32 * (1.0 - ratio))) as u16,
        }
    }
    
    fn combine_types(&self, 
        type1_1: PokemonType, type1_2: Option<PokemonType>,
        type2_1: PokemonType, type2_2: Option<PokemonType>,
        mode: TypeCombinationMode
    ) -> (PokemonType, Option<PokemonType>) {
        match mode {
            TypeCombinationMode::Dominant => (type1_1, type1_2),
            TypeCombinationMode::Blend => (type1_1, Some(type2_1)),
            TypeCombinationMode::Random => {
                let mut rng = thread_rng();
                if rng.gen::<bool>() { (type1_1, type1_2) } else { (type2_1, type2_2) }
            },
            TypeCombinationMode::Novel => (type1_1, Some(type2_1)),
        }
    }
    
    fn mix_abilities(&self, abilities1: &[String], abilities2: &[String], rng: &mut impl Rng) -> Vec<String> {
        let mut combined = abilities1.to_vec();
        combined.extend_from_slice(abilities2);
        combined.sort();
        combined.dedup();
        
        self.select_abilities(&combined, 2, rng)
    }
    
    fn mix_moves(&self, moves1: &[String], moves2: &[String], rng: &mut impl Rng) -> Vec<String> {
        let mut combined = moves1.to_vec();
        combined.extend_from_slice(moves2);
        combined.sort();
        combined.dedup();
        
        self.select_abilities(&combined, 4, rng)
    }
    
    fn blend_appearance(&self, parent1: &CreatureTemplate, parent2: &CreatureTemplate, rng: &mut impl Rng) -> CreatureAppearance {
        // 简化的外观混合
        CreatureAppearance {
            base_colors: vec!["Mixed".to_string()],
            patterns: vec!["Hybrid".to_string()],
            special_features: Vec::new(),
            texture_variations: HashMap::new(),
        }
    }
}

// 其他子系统的基础实现
impl BalanceSystem {
    pub fn new() -> Result<Self, GameError> {
        let mut stat_caps = HashMap::new();
        stat_caps.insert("hp".to_string(), 255);
        stat_caps.insert("attack".to_string(), 255);
        stat_caps.insert("defense".to_string(), 255);
        
        Ok(Self {
            stat_caps,
            type_balance_matrix: HashMap::new(),
            power_scaling_rules: PowerScalingRules {
                base_stat_total_limits: HashMap::new(),
                individual_stat_caps: HashMap::new(),
                type_effectiveness_modifiers: HashMap::new(),
            },
            tier_restrictions: HashMap::new(),
        })
    }
}

impl EvolutionTreeBuilder {
    pub fn new() -> Self {
        Self {
            evolution_chains: HashMap::new(),
            branching_rules: Vec::new(),
        }
    }
    
    pub fn build_chain(&self, base_template_id: &str, config: EvolutionChainConfig) -> Result<EvolutionChain, GameError> {
        Ok(EvolutionChain {
            base_form: base_template_id.to_string(),
            stages: Vec::new(),
            alternate_forms: Vec::new(),
            mega_evolutions: Vec::new(),
        })
    }
}

impl RaritySystem {
    pub fn new() -> Self {
        let mut tier_probabilities = HashMap::new();
        tier_probabilities.insert(RarityTier::Common, 0.5);
        tier_probabilities.insert(RarityTier::Uncommon, 0.25);
        tier_probabilities.insert(RarityTier::Rare, 0.15);
        tier_probabilities.insert(RarityTier::Epic, 0.07);
        tier_probabilities.insert(RarityTier::Legendary, 0.025);
        tier_probabilities.insert(RarityTier::Mythical, 0.005);
        
        Self {
            tier_probabilities,
            spawn_modifiers: HashMap::new(),
            legendary_restrictions: LegendaryRestrictions {
                max_legendaries_per_region: 3,
                unique_legendary_names: true,
                power_requirements: StatLimits {
                    total_minimum: 600,
                    total_maximum: 720,
                    individual_maximum: 150,
                    required_distribution: None,
                },
                special_abilities_required: true,
            },
        }
    }
}

impl TraitSystem {
    pub fn new() -> Self {
        Self {
            available_traits: HashMap::new(),
            trait_combinations: Vec::new(),
            inheritance_rules: HashMap::new(),
        }
    }
}

impl MutationSystem {
    pub fn new() -> Self {
        Self {
            mutation_types: HashMap::new(),
            mutation_triggers: Vec::new(),
            stability_factors: HashMap::new(),
        }
    }
    
    pub fn apply_mutations(&self, creature: &mut GeneratedCreature, intensity: f32) -> Result<(), GameError> {
        if intensity <= 0.0 {
            return Ok(());
        }
        
        let mut rng = thread_rng();
        
        // 随机属性变异
        if rng.gen::<f32>() < intensity * 0.3 {
            let stat_modifier = rng.gen_range(0.8..1.2);
            creature.stats.attack = ((creature.stats.attack as f32) * stat_modifier) as u16;
            creature.generation_metadata.mutations_applied.push("stat_mutation".to_string());
        }
        
        // 能力变异
        if rng.gen::<f32>() < intensity * 0.2 {
            if !creature.abilities.is_empty() {
                let idx = rng.gen_range(0..creature.abilities.len());
                creature.abilities[idx] = format!("Mutated_{}", creature.abilities[idx]);
                creature.generation_metadata.mutations_applied.push("ability_mutation".to_string());
            }
        }
        
        Ok(())
    }
}

impl CreatureValidator {
    pub fn new() -> Self {
        Self {
            validation_rules: Vec::new(),
            error_tolerance: 0.1,
            auto_fix_enabled: true,
        }
    }
    
    pub fn validate_creature(&self, creature: &GeneratedCreature) -> Result<ValidationResult, GameError> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut score = 1.0;
        
        // 属性总和检查
        let total_stats = creature.stats.hp + creature.stats.attack + creature.stats.defense + 
                         creature.stats.sp_attack + creature.stats.sp_defense + creature.stats.speed;
        
        if total_stats < 200 {
            errors.push(ValidationError {
                message: "属性总和过低".to_string(),
                rule_name: "MinStatTotal".to_string(),
                severity: ValidationSeverity::Error,
            });
            score *= 0.8;
        }
        
        if total_stats > 800 {
            warnings.push(ValidationWarning {
                message: "属性总和过高，可能影响平衡性".to_string(),
                rule_name: "MaxStatTotal".to_string(),
            });
            score *= 0.95;
        }
        
        // 类型组合检查
        if creature.primary_type == creature.secondary_type.unwrap_or(creature.primary_type) {
            if creature.secondary_type.is_some() {
                warnings.push(ValidationWarning {
                    message: "主副类型相同".to_string(),
                    rule_name: "TypeDuplication".to_string(),
                });
                score *= 0.9;
            }
        }
        
        let is_valid = errors.is_empty() && score >= 0.7;
        
        Ok(ValidationResult {
            is_valid,
            warnings,
            errors,
            score,
        })
    }
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            allow_mutations: true,
            mutation_intensity: 0.1,
            preferred_rarity: None,
            size_preferences: None,
            type_restrictions: Vec::new(),
            stat_focus: None,
            strict_validation: false,
            preserve_theme: true,
        }
    }
}

impl Default for HybridizationOptions {
    fn default() -> Self {
        Self {
            stat_inheritance_ratio: 0.5,
            type_combination_mode: TypeCombinationMode::Blend,
            ability_mixing_allowed: true,
            preserve_rarity: false,
            mutation_chance: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creature_engine_creation() {
        let engine = CreatureEngine::new();
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_template_loading() {
        let mut engine = CreatureEngine::new().unwrap();
        
        let template = CreatureTemplate {
            id: "test_dragon".to_string(),
            name: "Test Dragon".to_string(),
            description: "A test dragon creature".to_string(),
            category: CreatureCategory::Dragon,
            base_type: PokemonType::Dragon,
            secondary_type: Some(PokemonType::Fire),
            base_stats: BaseStats {
                hp: 100,
                attack: 120,
                defense: 80,
                sp_attack: 140,
                sp_defense: 85,
                speed: 95,
            },
            stat_ranges: StatRanges {
                hp: (80, 120),
                attack: (100, 140),
                defense: (60, 100),
                sp_attack: (120, 160),
                sp_defense: (65, 105),
                speed: (75, 115),
            },
            abilities: vec!["Blaze".to_string(), "Solar Power".to_string()],
            possible_moves: vec!["Flamethrower".to_string(), "Dragon Pulse".to_string()],
            evolution_requirements: Vec::new(),
            habitat: Habitat {
                primary: "Mountain".to_string(),
                secondary: vec!["Cave".to_string()],
                preferred_weather: vec!["Sunny".to_string()],
                altitude_range: Some((1000, 3000)),
                temperature_range: Some((15.0, 35.0)),
            },
            rarity_tier: RarityTier::Rare,
            size_category: SizeCategory::Large,
            weight_range: (200.0, 300.0),
            height_range: (2.5, 3.5),
            generation_rules: GenerationRules {
                allow_stat_mutation: true,
                allow_type_variation: false,
                allow_ability_mixing: true,
                preserve_theme: true,
                mutation_rate: 0.1,
                custom_constraints: HashMap::new(),
            },
        };
        
        let result = engine.load_template(template);
        assert!(result.is_ok());
        assert_eq!(engine.statistics.templates_created, 1);
    }
    
    #[test]
    fn test_creature_generation() {
        let mut engine = CreatureEngine::new().unwrap();
        
        // 加载模板（使用简化的模板用于测试）
        let template = CreatureTemplate {
            id: "simple_fire".to_string(),
            name: "Simple Fire".to_string(),
            description: "A simple fire creature".to_string(),
            category: CreatureCategory::Elemental,
            base_type: PokemonType::Fire,
            secondary_type: None,
            base_stats: BaseStats {
                hp: 50, attack: 60, defense: 40, sp_attack: 70, sp_defense: 50, speed: 55,
            },
            stat_ranges: StatRanges {
                hp: (40, 60), attack: (50, 70), defense: (30, 50),
                sp_attack: (60, 80), sp_defense: (40, 60), speed: (45, 65),
            },
            abilities: vec!["Blaze".to_string()],
            possible_moves: vec!["Ember".to_string(), "Tackle".to_string()],
            evolution_requirements: Vec::new(),
            habitat: Habitat {
                primary: "Grassland".to_string(),
                secondary: Vec::new(),
                preferred_weather: vec!["Sunny".to_string()],
                altitude_range: None,
                temperature_range: Some((20.0, 40.0)),
            },
            rarity_tier: RarityTier::Common,
            size_category: SizeCategory::Small,
            weight_range: (10.0, 20.0),
            height_range: (0.5, 1.0),
            generation_rules: GenerationRules {
                allow_stat_mutation: false,
                allow_type_variation: false,
                allow_ability_mixing: false,
                preserve_theme: true,
                mutation_rate: 0.0,
                custom_constraints: HashMap::new(),
            },
        };
        
        engine.load_template(template).unwrap();
        
        let options = GenerationOptions::default();
        let result = engine.generate_creature("simple_fire", options);
        
        assert!(result.is_ok());
        let creature = result.unwrap();
        assert_eq!(creature.template_id, Some("simple_fire".to_string()));
        assert_eq!(creature.primary_type, PokemonType::Fire);
        assert!(creature.stats.hp >= 40 && creature.stats.hp <= 60);
    }
    
    #[test]
    fn test_validation_system() {
        let validator = CreatureValidator::new();
        
        let creature = GeneratedCreature {
            id: "test_creature".to_string(),
            name: "Test Creature".to_string(),
            template_id: None,
            category: CreatureCategory::Beast,
            primary_type: PokemonType::Normal,
            secondary_type: None,
            stats: BaseStats {
                hp: 100, attack: 100, defense: 100,
                sp_attack: 100, sp_defense: 100, speed: 100,
            },
            abilities: vec!["Test Ability".to_string()],
            moves: vec!["Test Move".to_string()],
            traits: Vec::new(),
            rarity: RarityTier::Common,
            size: SizeCategory::Medium,
            weight: 50.0,
            height: 1.5,
            appearance: CreatureAppearance {
                base_colors: vec!["Brown".to_string()],
                patterns: vec!["Solid".to_string()],
                special_features: Vec::new(),
                texture_variations: HashMap::new(),
            },
            habitat: Habitat {
                primary: "Forest".to_string(),
                secondary: Vec::new(),
                preferred_weather: vec!["Clear".to_string()],
                altitude_range: None,
                temperature_range: Some((10.0, 30.0)),
            },
            generation_metadata: GenerationMetadata {
                generation_algorithm: "test".to_string(),
                parent_templates: Vec::new(),
                mutations_applied: Vec::new(),
                generation_time: std::time::Duration::from_millis(100),
                validation_score: 1.0,
                created_at: std::time::SystemTime::now(),
            },
        };
        
        let result = validator.validate_creature(&creature);
        assert!(result.is_ok());
        
        let validation = result.unwrap();
        assert!(validation.is_valid());
        assert!(validation.score > 0.7);
    }
}