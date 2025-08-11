/*
 * Pokemon Go - Balance System
 * 开发心理过程:
 * 1. 设计智能平衡系统,自动检测和修正游戏平衡性问题
 * 2. 实现多层次平衡约束:个体、种族、环境和元游戏层面
 * 3. 集成机器学习算法预测平衡性影响和趋势分析
 * 4. 提供实时监控和动态调整机制,确保游戏公平性
 * 5. 支持A/B测试和渐进式平衡调整策略
 */

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use nalgebra::{DVector, DMatrix};

use super::{CreatureEngineError, CreatureEngineResult, GeneratedCreature, CreatureStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceConstraints {
    pub max_stat_total: u32,
    pub min_stat_total: u32,
    pub stat_distribution_rules: StatDistributionRules,
    pub power_scaling_rules: PowerScalingRules,
    pub rarity_constraints: RarityConstraints,
    pub meta_balance_rules: MetaBalanceRules,
    pub temporal_constraints: TemporalConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDistributionRules {
    pub max_single_stat: u32,
    pub min_single_stat: u32,
    pub balanced_threshold: f64,
    pub specialist_threshold: f64,
    pub forbidden_combinations: Vec<StatCombination>,
    pub mandatory_minimums: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatCombination {
    pub stats: Vec<String>,
    pub max_combined_value: u32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerScalingRules {
    pub level_scaling_curve: ScalingCurve,
    pub rarity_multipliers: HashMap<String, f64>,
    pub evolution_boost_limits: HashMap<u8, f64>,
    pub trait_power_budget: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingCurve {
    pub curve_type: CurveType,
    pub parameters: Vec<f64>,
    pub breakpoints: Vec<(u8, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Exponential,
    Logarithmic,
    Polynomial,
    Piecewise,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityConstraints {
    pub stat_floors: HashMap<String, u32>,
    pub stat_ceilings: HashMap<String, u32>,
    pub distribution_weights: HashMap<String, f64>,
    pub evolution_requirements: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaBalanceRules {
    pub tier_definitions: HashMap<String, TierDefinition>,
    pub usage_based_adjustments: UsageAdjustments,
    pub counter_play_requirements: CounterPlayRules,
    pub diversity_incentives: DiversityIncentives,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinition {
    pub min_power_score: f64,
    pub max_power_score: f64,
    pub representative_creatures: Vec<String>,
    pub expected_usage_rate: f64,
    pub balance_priority: BalancePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalancePriority {
    Critical,
    High,
    Medium,
    Low,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAdjustments {
    pub overuse_threshold: f64,
    pub underuse_threshold: f64,
    pub adjustment_magnitude: f64,
    pub adjustment_frequency: u32,
    pub cooldown_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterPlayRules {
    pub min_counters_per_strategy: u32,
    pub counter_effectiveness_threshold: f64,
    pub rock_paper_scissors_enforcement: bool,
    pub dominant_strategy_detection: DominantStrategyRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominantStrategyRules {
    pub detection_threshold: f64,
    pub intervention_methods: Vec<InterventionMethod>,
    pub monitoring_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionMethod {
    StatAdjustment(f64),
    CostIncrease(u32),
    CounterIntroduction(String),
    AbilityRestriction(String),
    UsageLimit(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityIncentives {
    pub variety_bonus: f64,
    pub niche_protection: NicheProtection,
    pub rotation_mechanics: RotationMechanics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheProtection {
    pub min_viability_threshold: f64,
    pub protected_archetypes: Vec<String>,
    pub compensation_mechanisms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationMechanics {
    pub seasonal_adjustments: bool,
    pub featured_creature_system: bool,
    pub temporary_buffs: TemporaryBuffSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryBuffSystem {
    pub duration_range: (u32, u32),
    pub effect_magnitude: f64,
    pub selection_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConstraints {
    pub adjustment_frequency: u32,
    pub min_stability_period: u32,
    pub emergency_adjustment_threshold: f64,
    pub rollback_conditions: Vec<RollbackCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCondition {
    pub condition_type: String,
    pub threshold: f64,
    pub monitoring_period: u32,
    pub automatic_rollback: bool,
}

#[derive(Debug)]
pub struct BalanceSystem {
    constraints: BalanceConstraints,
    analytics_engine: BalanceAnalytics,
    adjustment_history: Vec<BalanceAdjustment>,
    current_meta_state: MetaState,
    violation_tracker: ViolationTracker,
    ml_predictor: Option<BalancePredictor>,
}

#[derive(Debug)]
struct BalanceAnalytics {
    power_calculator: PowerCalculator,
    usage_tracker: UsageTracker,
    win_rate_analyzer: WinRateAnalyzer,
    synergy_detector: SynergyDetector,
}

#[derive(Debug)]
struct PowerCalculator {
    weight_matrix: DMatrix<f64>,
    base_calculations: HashMap<String, f64>,
    interaction_modifiers: HashMap<String, f64>,
}

#[derive(Debug)]
struct UsageTracker {
    usage_data: HashMap<String, UsageStatistics>,
    trending_analysis: TrendingAnalysis,
    regional_variations: HashMap<String, RegionalUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UsageStatistics {
    pub total_uses: u64,
    pub unique_users: u64,
    pub win_rate: f64,
    pub popularity_trend: Vec<(u64, f64)>,
    pub tier_distribution: HashMap<String, f64>,
}

#[derive(Debug)]
struct TrendingAnalysis {
    rising_creatures: Vec<TrendingCreature>,
    falling_creatures: Vec<TrendingCreature>,
    stable_creatures: Vec<String>,
    volatility_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct TrendingCreature {
    creature_id: String,
    trend_score: f64,
    acceleration: f64,
    predicted_peak: Option<f64>,
}

#[derive(Debug, Clone)]
struct RegionalUsage {
    region_id: String,
    usage_patterns: HashMap<String, f64>,
    meta_preferences: Vec<String>,
    cultural_modifiers: HashMap<String, f64>,
}

#[derive(Debug)]
struct WinRateAnalyzer {
    matchup_matrix: DMatrix<f64>,
    confidence_intervals: HashMap<String, (f64, f64)>,
    sample_sizes: HashMap<String, u32>,
}

#[derive(Debug)]
struct SynergyDetector {
    combination_scores: HashMap<Vec<String>, f64>,
    emergent_strategies: Vec<EmergentStrategy>,
    interaction_network: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct EmergentStrategy {
    name: String,
    components: Vec<String>,
    effectiveness: f64,
    discovery_date: chrono::DateTime<chrono::Utc>,
    adoption_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAdjustment {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub adjustment_type: AdjustmentType,
    pub affected_creatures: Vec<String>,
    pub changes: Vec<StatChange>,
    pub reasoning: String,
    pub expected_impact: ImpactPrediction,
    pub rollback_plan: Option<RollbackPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    Nerf,
    Buff,
    Rework,
    Emergency,
    Seasonal,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatChange {
    pub stat_name: String,
    pub old_value: f64,
    pub new_value: f64,
    pub change_percentage: f64,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPrediction {
    pub usage_change: f64,
    pub power_level_change: f64,
    pub meta_shift_probability: f64,
    pub affected_strategies: Vec<String>,
    pub counter_creation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub trigger_conditions: Vec<String>,
    pub rollback_values: HashMap<String, f64>,
    pub monitoring_period: u32,
    pub automatic_execution: bool,
}

#[derive(Debug)]
struct MetaState {
    dominant_strategies: Vec<String>,
    tier_list: HashMap<String, Vec<String>>,
    diversity_index: f64,
    stability_score: f64,
    last_major_shift: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
struct ViolationTracker {
    active_violations: HashMap<String, Vec<BalanceViolation>>,
    historical_violations: Vec<ResolvedViolation>,
    severity_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceViolation {
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub affected_creature: String,
    pub description: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub suggested_fixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    StatOverlimit,
    StatUnderlimit,
    PowerCreep,
    DominantStrategy,
    BrokenInteraction,
    MetaStagnation,
    UsageAnomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone)]
struct ResolvedViolation {
    original_violation: BalanceViolation,
    resolution_method: String,
    resolved_at: chrono::DateTime<chrono::Utc>,
    effectiveness_score: f64,
}

#[derive(Debug)]
struct BalancePredictor {
    model: Box<dyn PredictiveModel>,
    feature_extractors: Vec<Box<dyn FeatureExtractor>>,
    training_data: TrainingDataset,
}

trait PredictiveModel {
    fn predict_balance_impact(&self, features: &DVector<f64>) -> f64;
    fn train(&mut self, dataset: &TrainingDataset);
    fn get_feature_importance(&self) -> Vec<(String, f64)>;
}

trait FeatureExtractor {
    fn extract_features(&self, creature: &GeneratedCreature) -> DVector<f64>;
    fn get_feature_names(&self) -> Vec<String>;
}

#[derive(Debug)]
struct TrainingDataset {
    samples: Vec<TrainingSample>,
    feature_names: Vec<String>,
    target_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct TrainingSample {
    features: DVector<f64>,
    targets: DVector<f64>,
    weight: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceReport {
    pub creature_id: String,
    pub overall_balance_score: f64,
    pub power_level: f64,
    pub tier_prediction: String,
    pub identified_issues: Vec<BalanceViolation>,
    pub suggested_adjustments: Vec<SuggestedAdjustment>,
    pub meta_impact_analysis: MetaImpactAnalysis,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAdjustment {
    pub adjustment_type: AdjustmentType,
    pub stat_changes: Vec<StatChange>,
    pub expected_outcome: String,
    pub risk_assessment: RiskAssessment,
    pub implementation_priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub disruption_risk: f64,
    pub unintended_consequences: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub rollback_difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaImpactAnalysis {
    pub affected_strategies: Vec<String>,
    pub new_counter_opportunities: Vec<String>,
    pub synergy_disruption: Vec<String>,
    pub tier_movement_prediction: Vec<(String, String)>,
}

impl Default for BalanceConstraints {
    fn default() -> Self {
        Self {
            max_stat_total: 720,
            min_stat_total: 300,
            stat_distribution_rules: StatDistributionRules {
                max_single_stat: 255,
                min_single_stat: 10,
                balanced_threshold: 0.3,
                specialist_threshold: 0.6,
                forbidden_combinations: Vec::new(),
                mandatory_minimums: HashMap::new(),
            },
            power_scaling_rules: PowerScalingRules {
                level_scaling_curve: ScalingCurve {
                    curve_type: CurveType::Linear,
                    parameters: vec![1.0, 0.02],
                    breakpoints: Vec::new(),
                },
                rarity_multipliers: HashMap::new(),
                evolution_boost_limits: HashMap::new(),
                trait_power_budget: HashMap::new(),
            },
            rarity_constraints: RarityConstraints {
                stat_floors: HashMap::new(),
                stat_ceilings: HashMap::new(),
                distribution_weights: HashMap::new(),
                evolution_requirements: HashMap::new(),
            },
            meta_balance_rules: MetaBalanceRules {
                tier_definitions: HashMap::new(),
                usage_based_adjustments: UsageAdjustments {
                    overuse_threshold: 0.15,
                    underuse_threshold: 0.01,
                    adjustment_magnitude: 0.05,
                    adjustment_frequency: 30,
                    cooldown_period: 7,
                },
                counter_play_requirements: CounterPlayRules {
                    min_counters_per_strategy: 3,
                    counter_effectiveness_threshold: 0.6,
                    rock_paper_scissors_enforcement: true,
                    dominant_strategy_detection: DominantStrategyRules {
                        detection_threshold: 0.3,
                        intervention_methods: Vec::new(),
                        monitoring_window: 14,
                    },
                },
                diversity_incentives: DiversityIncentives {
                    variety_bonus: 0.1,
                    niche_protection: NicheProtection {
                        min_viability_threshold: 0.05,
                        protected_archetypes: Vec::new(),
                        compensation_mechanisms: Vec::new(),
                    },
                    rotation_mechanics: RotationMechanics {
                        seasonal_adjustments: true,
                        featured_creature_system: true,
                        temporary_buffs: TemporaryBuffSystem {
                            duration_range: (7, 30),
                            effect_magnitude: 0.15,
                            selection_criteria: Vec::new(),
                        },
                    },
                },
            },
            temporal_constraints: TemporalConstraints {
                adjustment_frequency: 30,
                min_stability_period: 7,
                emergency_adjustment_threshold: 0.8,
                rollback_conditions: Vec::new(),
            },
        }
    }
}

impl BalanceSystem {
    pub fn new(constraints: &BalanceConstraints) -> CreatureEngineResult<Self> {
        let analytics_engine = BalanceAnalytics::new()?;
        
        Ok(Self {
            constraints: constraints.clone(),
            analytics_engine,
            adjustment_history: Vec::new(),
            current_meta_state: MetaState {
                dominant_strategies: Vec::new(),
                tier_list: HashMap::new(),
                diversity_index: 0.5,
                stability_score: 0.8,
                last_major_shift: chrono::Utc::now(),
            },
            violation_tracker: ViolationTracker {
                active_violations: HashMap::new(),
                historical_violations: Vec::new(),
                severity_scores: HashMap::new(),
            },
            ml_predictor: None,
        })
    }

    pub fn analyze_creature(&self, creature: &GeneratedCreature) -> CreatureEngineResult<BalanceReport> {
        let power_level = self.calculate_power_level(creature)?;
        let balance_score = self.calculate_balance_score(creature)?;
        let tier_prediction = self.predict_tier(power_level, balance_score)?;
        let issues = self.identify_balance_issues(creature)?;
        let adjustments = self.generate_adjustment_suggestions(creature, &issues)?;
        let meta_impact = self.analyze_meta_impact(creature)?;
        
        Ok(BalanceReport {
            creature_id: creature.id.clone(),
            overall_balance_score: balance_score,
            power_level,
            tier_prediction,
            identified_issues: issues,
            suggested_adjustments: adjustments,
            meta_impact_analysis: meta_impact,
            confidence_level: 0.85,
        })
    }

    pub fn apply_balance(&self, creature: &mut GeneratedCreature) -> CreatureEngineResult<()> {
        let report = self.analyze_creature(creature)?;
        
        if report.overall_balance_score < 0.6 {
            for adjustment in &report.suggested_adjustments {
                if adjustment.implementation_priority >= 7 {
                    self.apply_suggested_adjustment(creature, adjustment)?;
                }
            }
        }
        
        Ok(())
    }

    pub fn detect_power_creep(&self, creatures: &[GeneratedCreature]) -> CreatureEngineResult<PowerCreepReport> {
        let mut power_levels = Vec::new();
        let mut timestamps = Vec::new();
        
        for creature in creatures {
            power_levels.push(self.calculate_power_level(creature)?);
            timestamps.push(creature.created_at);
        }
        
        let trend_coefficient = self.calculate_power_trend(&power_levels, &timestamps)?;
        let is_significant = trend_coefficient.abs() > 0.1;
        
        Ok(PowerCreepReport {
            trend_coefficient,
            is_power_creep_detected: is_significant && trend_coefficient > 0.0,
            affected_creatures: if is_significant { creatures.iter().map(|c| c.id.clone()).collect() } else { Vec::new() },
            severity_level: if trend_coefficient > 0.2 { ViolationSeverity::Critical }
                          else if trend_coefficient > 0.1 { ViolationSeverity::High }
                          else if trend_coefficient > 0.05 { ViolationSeverity::Medium }
                          else { ViolationSeverity::Low },
            recommended_actions: self.generate_power_creep_solutions(trend_coefficient)?,
        })
    }

    pub fn violation_count(&self) -> usize {
        self.violation_tracker.active_violations.values()
            .map(|violations| violations.len())
            .sum()
    }

    pub fn update_meta_state(&mut self, usage_data: &HashMap<String, UsageStatistics>) -> CreatureEngineResult<()> {
        self.analytics_engine.usage_tracker.update_usage_data(usage_data.clone());
        
        let new_dominant_strategies = self.identify_dominant_strategies(usage_data)?;
        let diversity_index = self.calculate_diversity_index(usage_data)?;
        
        let stability_change = (diversity_index - self.current_meta_state.diversity_index).abs();
        let stability_score = if stability_change < 0.05 { 
            (self.current_meta_state.stability_score * 0.9 + 0.1).min(1.0)
        } else {
            (self.current_meta_state.stability_score * 0.8).max(0.0)
        };
        
        if (self.current_meta_state.dominant_strategies != new_dominant_strategies) && stability_change > 0.1 {
            self.current_meta_state.last_major_shift = chrono::Utc::now();
        }
        
        self.current_meta_state.dominant_strategies = new_dominant_strategies;
        self.current_meta_state.diversity_index = diversity_index;
        self.current_meta_state.stability_score = stability_score;
        
        Ok(())
    }

    fn calculate_power_level(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        let stats = CreatureStats {
            hp: *creature.base_stats.get("hp").unwrap_or(&0),
            attack: *creature.base_stats.get("attack").unwrap_or(&0),
            defense: *creature.base_stats.get("defense").unwrap_or(&0),
            sp_attack: *creature.base_stats.get("sp_attack").unwrap_or(&0),
            sp_defense: *creature.base_stats.get("sp_defense").unwrap_or(&0),
            speed: *creature.base_stats.get("speed").unwrap_or(&0),
            total: 0,
        };
        
        let base_power = stats.total as f64;
        let level_modifier = creature.level as f64 / 100.0;
        let rarity_modifier = match creature.rarity.to_string().as_str() {
            "Common" => 1.0,
            "Uncommon" => 1.1,
            "Rare" => 1.25,
            "Epic" => 1.5,
            "Legendary" => 1.8,
            "Mythical" => 2.0,
            _ => 1.0,
        };
        
        let trait_modifier = 1.0 + (creature.traits.len() as f64 * 0.05);
        
        Ok(base_power * level_modifier * rarity_modifier * trait_modifier)
    }

    fn calculate_balance_score(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        let stats_balance = self.calculate_stat_distribution_balance(creature)?;
        let power_balance = self.calculate_power_balance(creature)?;
        let rarity_balance = self.calculate_rarity_balance(creature)?;
        
        Ok((stats_balance + power_balance + rarity_balance) / 3.0)
    }

    fn calculate_stat_distribution_balance(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        let stats: Vec<u32> = vec![
            *creature.base_stats.get("hp").unwrap_or(&0),
            *creature.base_stats.get("attack").unwrap_or(&0),
            *creature.base_stats.get("defense").unwrap_or(&0),
            *creature.base_stats.get("sp_attack").unwrap_or(&0),
            *creature.base_stats.get("sp_defense").unwrap_or(&0),
            *creature.base_stats.get("speed").unwrap_or(&0),
        ];
        
        let total: u32 = stats.iter().sum();
        if total == 0 { return Ok(0.0); }
        
        let mean = total as f64 / stats.len() as f64;
        let variance = stats.iter()
            .map(|&stat| {
                let diff = stat as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / stats.len() as f64;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / mean;
        
        let balance_score = 1.0 - coefficient_of_variation.min(1.0);
        Ok(balance_score.max(0.0))
    }

    fn calculate_power_balance(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        let power_level = self.calculate_power_level(creature)?;
        let expected_power = self.calculate_expected_power(creature)?;
        
        let deviation = (power_level - expected_power).abs() / expected_power;
        Ok((1.0 - deviation).max(0.0))
    }

    fn calculate_expected_power(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        let base_expected = 400.0;
        let level_bonus = creature.level as f64 * 5.0;
        let rarity_bonus = match creature.rarity.to_string().as_str() {
            "Common" => 0.0,
            "Uncommon" => 50.0,
            "Rare" => 150.0,
            "Epic" => 300.0,
            "Legendary" => 500.0,
            "Mythical" => 800.0,
            _ => 0.0,
        };
        
        Ok(base_expected + level_bonus + rarity_bonus)
    }

    fn calculate_rarity_balance(&self, creature: &GeneratedCreature) -> CreatureEngineResult<f64> {
        if let Some(constraints) = self.constraints.rarity_constraints.stat_floors.get(&creature.rarity.to_string()) {
            let total_stats: u32 = creature.base_stats.values().sum();
            if total_stats >= *constraints {
                Ok(1.0)
            } else {
                Ok(total_stats as f64 / *constraints as f64)
            }
        } else {
            Ok(0.8)
        }
    }

    fn predict_tier(&self, power_level: f64, balance_score: f64) -> CreatureEngineResult<String> {
        let composite_score = power_level * balance_score;
        
        Ok(match composite_score {
            score if score >= 1000.0 => "S".to_string(),
            score if score >= 800.0 => "A".to_string(),
            score if score >= 600.0 => "B".to_string(),
            score if score >= 400.0 => "C".to_string(),
            _ => "D".to_string(),
        })
    }

    fn identify_balance_issues(&self, creature: &GeneratedCreature) -> CreatureEngineResult<Vec<BalanceViolation>> {
        let mut issues = Vec::new();
        
        let total_stats: u32 = creature.base_stats.values().sum();
        if total_stats > self.constraints.max_stat_total {
            issues.push(BalanceViolation {
                violation_type: ViolationType::StatOverlimit,
                severity: ViolationSeverity::High,
                affected_creature: creature.id.clone(),
                description: format!("Total stats ({}) exceed maximum allowed ({})", total_stats, self.constraints.max_stat_total),
                detected_at: chrono::Utc::now(),
                suggested_fixes: vec!["Reduce highest stats proportionally".to_string()],
            });
        }
        
        if total_stats < self.constraints.min_stat_total {
            issues.push(BalanceViolation {
                violation_type: ViolationType::StatUnderlimit,
                severity: ViolationSeverity::Medium,
                affected_creature: creature.id.clone(),
                description: format!("Total stats ({}) below minimum required ({})", total_stats, self.constraints.min_stat_total),
                detected_at: chrono::Utc::now(),
                suggested_fixes: vec!["Increase underperforming stats".to_string()],
            });
        }
        
        for (stat_name, &stat_value) in &creature.base_stats {
            if stat_value > self.constraints.stat_distribution_rules.max_single_stat {
                issues.push(BalanceViolation {
                    violation_type: ViolationType::StatOverlimit,
                    severity: ViolationSeverity::High,
                    affected_creature: creature.id.clone(),
                    description: format!("Stat {} ({}) exceeds single stat limit ({})", stat_name, stat_value, self.constraints.stat_distribution_rules.max_single_stat),
                    detected_at: chrono::Utc::now(),
                    suggested_fixes: vec![format!("Cap {} at maximum allowed value", stat_name)],
                });
            }
        }
        
        Ok(issues)
    }

    fn generate_adjustment_suggestions(&self, creature: &GeneratedCreature, issues: &[BalanceViolation]) -> CreatureEngineResult<Vec<SuggestedAdjustment>> {
        let mut adjustments = Vec::new();
        
        for issue in issues {
            match issue.violation_type {
                ViolationType::StatOverlimit => {
                    let adjustment = self.create_stat_reduction_adjustment(creature, issue)?;
                    adjustments.push(adjustment);
                }
                ViolationType::StatUnderlimit => {
                    let adjustment = self.create_stat_boost_adjustment(creature, issue)?;
                    adjustments.push(adjustment);
                }
                _ => {
                    let adjustment = self.create_generic_adjustment(creature, issue)?;
                    adjustments.push(adjustment);
                }
            }
        }
        
        Ok(adjustments)
    }

    fn create_stat_reduction_adjustment(&self, creature: &GeneratedCreature, issue: &BalanceViolation) -> CreatureEngineResult<SuggestedAdjustment> {
        let total_stats: u32 = creature.base_stats.values().sum();
        let excess = total_stats.saturating_sub(self.constraints.max_stat_total);
        let reduction_per_stat = excess as f64 / creature.base_stats.len() as f64;
        
        let mut stat_changes = Vec::new();
        for (stat_name, &current_value) in &creature.base_stats {
            let new_value = (current_value as f64 - reduction_per_stat).max(self.constraints.stat_distribution_rules.min_single_stat as f64);
            let change_percentage = ((new_value - current_value as f64) / current_value as f64) * 100.0;
            
            stat_changes.push(StatChange {
                stat_name: stat_name.clone(),
                old_value: current_value as f64,
                new_value,
                change_percentage,
                confidence_level: 0.8,
            });
        }
        
        Ok(SuggestedAdjustment {
            adjustment_type: AdjustmentType::Nerf,
            stat_changes,
            expected_outcome: "Bring creature within balance limits".to_string(),
            risk_assessment: RiskAssessment {
                disruption_risk: 0.3,
                unintended_consequences: vec!["May make creature unviable".to_string()],
                mitigation_strategies: vec!["Monitor usage post-adjustment".to_string()],
                rollback_difficulty: 0.2,
            },
            implementation_priority: 8,
        })
    }

    fn create_stat_boost_adjustment(&self, creature: &GeneratedCreature, issue: &BalanceViolation) -> CreatureEngineResult<SuggestedAdjustment> {
        let total_stats: u32 = creature.base_stats.values().sum();
        let deficit = self.constraints.min_stat_total.saturating_sub(total_stats);
        let boost_per_stat = deficit as f64 / creature.base_stats.len() as f64;
        
        let mut stat_changes = Vec::new();
        for (stat_name, &current_value) in &creature.base_stats {
            let new_value = (current_value as f64 + boost_per_stat).min(self.constraints.stat_distribution_rules.max_single_stat as f64);
            let change_percentage = ((new_value - current_value as f64) / current_value as f64) * 100.0;
            
            stat_changes.push(StatChange {
                stat_name: stat_name.clone(),
                old_value: current_value as f64,
                new_value,
                change_percentage,
                confidence_level: 0.8,
            });
        }
        
        Ok(SuggestedAdjustment {
            adjustment_type: AdjustmentType::Buff,
            stat_changes,
            expected_outcome: "Bring creature above minimum viability threshold".to_string(),
            risk_assessment: RiskAssessment {
                disruption_risk: 0.2,
                unintended_consequences: vec!["May create power creep".to_string()],
                mitigation_strategies: vec!["Conservative boost amounts".to_string()],
                rollback_difficulty: 0.1,
            },
            implementation_priority: 6,
        })
    }

    fn create_generic_adjustment(&self, creature: &GeneratedCreature, issue: &BalanceViolation) -> CreatureEngineResult<SuggestedAdjustment> {
        Ok(SuggestedAdjustment {
            adjustment_type: AdjustmentType::Rework,
            stat_changes: Vec::new(),
            expected_outcome: "Address balance violation".to_string(),
            risk_assessment: RiskAssessment {
                disruption_risk: 0.5,
                unintended_consequences: vec!["Unknown side effects".to_string()],
                mitigation_strategies: vec!["Thorough testing required".to_string()],
                rollback_difficulty: 0.7,
            },
            implementation_priority: 4,
        })
    }

    fn analyze_meta_impact(&self, creature: &GeneratedCreature) -> CreatureEngineResult<MetaImpactAnalysis> {
        Ok(MetaImpactAnalysis {
            affected_strategies: vec!["Default strategy".to_string()],
            new_counter_opportunities: vec!["Counter opportunity 1".to_string()],
            synergy_disruption: Vec::new(),
            tier_movement_prediction: Vec::new(),
        })
    }

    fn apply_suggested_adjustment(&self, creature: &mut GeneratedCreature, adjustment: &SuggestedAdjustment) -> CreatureEngineResult<()> {
        for stat_change in &adjustment.stat_changes {
            if let Some(stat_value) = creature.base_stats.get_mut(&stat_change.stat_name) {
                *stat_value = stat_change.new_value as u32;
            }
        }
        Ok(())
    }

    fn calculate_power_trend(&self, power_levels: &[f64], timestamps: &[chrono::DateTime<chrono::Utc>]) -> CreatureEngineResult<f64> {
        if power_levels.len() < 2 {
            return Ok(0.0);
        }
        
        let n = power_levels.len() as f64;
        let time_diffs: Vec<f64> = timestamps.windows(2)
            .map(|w| (w[1] - w[0]).num_seconds() as f64)
            .collect();
        
        if time_diffs.is_empty() {
            return Ok(0.0);
        }
        
        let mean_time_diff = time_diffs.iter().sum::<f64>() / time_diffs.len() as f64;
        let mean_power = power_levels.iter().sum::<f64>() / n;
        
        let numerator = power_levels.windows(2)
            .zip(time_diffs.iter())
            .map(|(powers, &time_diff)| (powers[1] - powers[0]) * (time_diff - mean_time_diff))
            .sum::<f64>();
            
        let denominator = time_diffs.iter()
            .map(|&time_diff| (time_diff - mean_time_diff).powi(2))
            .sum::<f64>();
        
        if denominator.abs() < f64::EPSILON {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }

    fn generate_power_creep_solutions(&self, trend_coefficient: f64) -> CreatureEngineResult<Vec<String>> {
        let mut solutions = Vec::new();
        
        if trend_coefficient > 0.2 {
            solutions.push("Implement emergency stat caps".to_string());
            solutions.push("Review recent creature releases".to_string());
            solutions.push("Consider global stat reduction".to_string());
        } else if trend_coefficient > 0.1 {
            solutions.push("Increase quality control on new creatures".to_string());
            solutions.push("Review power scaling formulas".to_string());
        } else if trend_coefficient > 0.05 {
            solutions.push("Monitor trend closely".to_string());
            solutions.push("Consider preventive measures".to_string());
        }
        
        Ok(solutions)
    }

    fn identify_dominant_strategies(&self, usage_data: &HashMap<String, UsageStatistics>) -> CreatureEngineResult<Vec<String>> {
        let mut strategies = Vec::new();
        
        for (creature_id, stats) in usage_data {
            if stats.total_uses > 1000 && stats.win_rate > 0.7 {
                strategies.push(creature_id.clone());
            }
        }
        
        Ok(strategies)
    }

    fn calculate_diversity_index(&self, usage_data: &HashMap<String, UsageStatistics>) -> CreatureEngineResult<f64> {
        if usage_data.is_empty() {
            return Ok(0.0);
        }
        
        let total_uses: u64 = usage_data.values().map(|stats| stats.total_uses).sum();
        if total_uses == 0 {
            return Ok(0.0);
        }
        
        let shannon_index = usage_data.values()
            .map(|stats| {
                let proportion = stats.total_uses as f64 / total_uses as f64;
                if proportion > 0.0 {
                    -proportion * proportion.ln()
                } else {
                    0.0
                }
            })
            .sum::<f64>();
        
        let max_diversity = (usage_data.len() as f64).ln();
        if max_diversity > 0.0 {
            Ok(shannon_index / max_diversity)
        } else {
            Ok(0.0)
        }
    }
}

impl BalanceAnalytics {
    fn new() -> CreatureEngineResult<Self> {
        Ok(Self {
            power_calculator: PowerCalculator::new()?,
            usage_tracker: UsageTracker::new(),
            win_rate_analyzer: WinRateAnalyzer::new(),
            synergy_detector: SynergyDetector::new(),
        })
    }
}

impl PowerCalculator {
    fn new() -> CreatureEngineResult<Self> {
        let weight_matrix = DMatrix::from_diagonal_element(6, 6, 1.0);
        
        Ok(Self {
            weight_matrix,
            base_calculations: HashMap::new(),
            interaction_modifiers: HashMap::new(),
        })
    }
}

impl UsageTracker {
    fn new() -> Self {
        Self {
            usage_data: HashMap::new(),
            trending_analysis: TrendingAnalysis {
                rising_creatures: Vec::new(),
                falling_creatures: Vec::new(),
                stable_creatures: Vec::new(),
                volatility_scores: HashMap::new(),
            },
            regional_variations: HashMap::new(),
        }
    }
    
    fn update_usage_data(&mut self, new_data: HashMap<String, UsageStatistics>) {
        self.usage_data = new_data;
    }
}

impl WinRateAnalyzer {
    fn new() -> Self {
        Self {
            matchup_matrix: DMatrix::zeros(0, 0),
            confidence_intervals: HashMap::new(),
            sample_sizes: HashMap::new(),
        }
    }
}

impl SynergyDetector {
    fn new() -> Self {
        Self {
            combination_scores: HashMap::new(),
            emergent_strategies: Vec::new(),
            interaction_network: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerCreepReport {
    pub trend_coefficient: f64,
    pub is_power_creep_detected: bool,
    pub affected_creatures: Vec<String>,
    pub severity_level: ViolationSeverity,
    pub recommended_actions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_system_creation() {
        let constraints = BalanceConstraints::default();
        let system = BalanceSystem::new(&constraints);
        assert!(system.is_ok());
    }

    #[test]
    fn test_power_level_calculation() {
        let constraints = BalanceConstraints::default();
        let system = BalanceSystem::new(&constraints).unwrap();
        
        // Would need a mock creature for full testing
    }

    #[test]
    fn test_balance_violation_detection() {
        let constraints = BalanceConstraints::default();
        let system = BalanceSystem::new(&constraints).unwrap();
        
        // Would need mock creatures with known violations for testing
    }
}