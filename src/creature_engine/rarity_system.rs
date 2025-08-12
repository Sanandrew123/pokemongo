/*
 * Pokemon Go - Rarity System
 * 开发心理过程:
 * 1. 设计综合稀有度系统,平衡游戏经济和玩家获得感
 * 2. 实现动态稀有度调整,基于市场供需和玩家行为
 * 3. 集成多维度稀有度判定:基础稀有度、条件稀有度、时间稀有度
 * 4. 提供稀有度预测和趋势分析,支持运营决策
 * 5. 支持事件驱动的特殊稀有度和限时稀有度系统
 */

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{CreatureEngineError, CreatureEngineResult, CreatureTemplate, GeneratedCreature};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CreatureRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythical,
}

impl std::fmt::Display for CreatureRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreatureRarity::Common => write!(f, "Common"),
            CreatureRarity::Uncommon => write!(f, "Uncommon"),
            CreatureRarity::Rare => write!(f, "Rare"),
            CreatureRarity::Epic => write!(f, "Epic"),
            CreatureRarity::Legendary => write!(f, "Legendary"),
            CreatureRarity::Mythical => write!(f, "Mythical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityDistribution {
    pub base_weights: HashMap<CreatureRarity, f64>,
    pub conditional_modifiers: HashMap<String, RarityModifier>,
    pub temporal_modifiers: HashMap<String, TemporalRarityModifier>,
    pub location_modifiers: HashMap<String, LocationRarityModifier>,
    pub event_modifiers: HashMap<String, EventRarityModifier>,
    pub player_modifiers: PlayerBasedModifiers,
    pub global_multipliers: GlobalRarityMultipliers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityModifier {
    pub condition_type: ConditionType,
    pub rarity_adjustments: HashMap<CreatureRarity, f64>,
    pub activation_threshold: f64,
    pub duration: Option<u32>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Weather(String),
    TimeOfDay(u8, u8),
    Season(String),
    PlayerLevel(u8),
    ItemPossession(String),
    QuestProgress(String),
    RegionUnlock(String),
    CreatureCaught(String),
    StatThreshold(String, u32),
    CombinationPresence(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalRarityModifier {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub rarity_boosts: HashMap<CreatureRarity, f64>,
    pub affected_creatures: Vec<String>,
    pub cycle_type: CycleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleType {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    Event,
    OneTime,
    Recurring(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRarityModifier {
    pub location_type: LocationType,
    pub biome_preferences: HashMap<String, f64>,
    pub proximity_bonuses: Vec<ProximityBonus>,
    pub exclusion_zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    Urban,
    Rural,
    Coastal,
    Mountain,
    Forest,
    Desert,
    Arctic,
    Volcanic,
    Cave,
    Underwater,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProximityBonus {
    pub landmark_type: String,
    pub distance_threshold: f64,
    pub rarity_boost: f64,
    pub affected_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRarityModifier {
    pub event_id: String,
    pub event_type: EventType,
    pub duration: EventDuration,
    pub rarity_changes: RarityChanges,
    pub participation_requirements: Vec<String>,
    pub exclusive_creatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    CommunityDay,
    RaidWeekend,
    Safari,
    Holiday,
    Spotlight,
    Migration,
    Invasion,
    Tournament,
    Collaboration,
    Anniversary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDuration {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub recurring: Option<RecurrencePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrencePattern {
    pub interval: IntervalType,
    pub frequency: u32,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntervalType {
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityChanges {
    pub base_rarity_multipliers: HashMap<CreatureRarity, f64>,
    pub creature_specific_changes: HashMap<String, CreatureRarity>,
    pub new_creature_introductions: Vec<String>,
    pub temporary_removals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBasedModifiers {
    pub level_bonuses: HashMap<u8, f64>,
    pub achievement_bonuses: HashMap<String, f64>,
    pub loyalty_bonuses: LoyaltyBonusSystem,
    pub streak_bonuses: StreakBonusSystem,
    pub social_bonuses: SocialBonusSystem,
    pub spending_bonuses: SpendingBonusSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyBonusSystem {
    pub daily_login_bonus: f64,
    pub consecutive_day_multipliers: HashMap<u32, f64>,
    pub account_age_bonuses: HashMap<u32, f64>,
    pub activity_score_thresholds: HashMap<u32, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakBonusSystem {
    pub catch_streak_bonuses: HashMap<u32, f64>,
    pub evolution_streak_bonuses: HashMap<u32, f64>,
    pub battle_streak_bonuses: HashMap<u32, f64>,
    pub research_streak_bonuses: HashMap<u32, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialBonusSystem {
    pub friend_count_bonuses: HashMap<u32, f64>,
    pub trade_frequency_bonuses: HashMap<u32, f64>,
    pub gift_exchange_bonuses: HashMap<u32, f64>,
    pub raid_participation_bonuses: HashMap<u32, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingBonusSystem {
    pub purchase_tier_bonuses: HashMap<String, f64>,
    pub premium_member_benefits: HashMap<String, f64>,
    pub seasonal_pass_bonuses: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRarityMultipliers {
    pub economy_adjustment: f64,
    pub population_balance: f64,
    pub seasonal_global_modifier: f64,
    pub special_event_modifier: f64,
    pub maintenance_mode_modifier: f64,
}

#[derive(Debug)]
pub struct RaritySystem {
    distribution: RarityDistribution,
    rng: ChaCha8Rng,
    analytics: RarityAnalytics,
    dynamic_adjustments: DynamicAdjustmentSystem,
    market_tracker: MarketTracker,
}

#[derive(Debug)]
struct RarityAnalytics {
    rarity_statistics: HashMap<CreatureRarity, RarityStats>,
    trend_analyzer: TrendAnalyzer,
    prediction_engine: RarityPredictor,
    market_impact_analyzer: MarketImpactAnalyzer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RarityStats {
    pub total_generated: u64,
    pub current_market_supply: u64,
    pub average_generation_rate: f64,
    pub demand_score: f64,
    pub price_volatility: f64,
    pub player_satisfaction_score: f64,
}

#[derive(Debug)]
struct TrendAnalyzer {
    historical_data: Vec<RarityTrendPoint>,
    moving_averages: HashMap<CreatureRarity, f64>,
    volatility_indices: HashMap<CreatureRarity, f64>,
    anomaly_detector: AnomalyDetector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RarityTrendPoint {
    timestamp: chrono::DateTime<chrono::Utc>,
    rarity_counts: HashMap<CreatureRarity, u32>,
    market_conditions: MarketConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketConditions {
    supply_demand_ratio: f64,
    player_activity_level: f64,
    event_activity: bool,
    economic_indicators: HashMap<String, f64>,
}

#[derive(Debug)]
struct AnomalyDetector {
    detection_algorithms: Vec<Box<dyn AnomalyDetectionAlgorithm>>,
    anomaly_history: Vec<RarityAnomaly>,
    threshold_parameters: HashMap<String, f64>,
}

trait AnomalyDetectionAlgorithm {
    fn detect_anomaly(&self, data: &[RarityTrendPoint]) -> Vec<RarityAnomaly>;
    fn get_confidence_level(&self) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RarityAnomaly {
    anomaly_type: AnomalyType,
    affected_rarities: Vec<CreatureRarity>,
    detected_at: chrono::DateTime<chrono::Utc>,
    severity: f64,
    suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AnomalyType {
    UnexpectedSpike,
    UnexpectedDrop,
    UnusualDistribution,
    MarketManipulation,
    SystemError,
    EventImpact,
}

#[derive(Debug)]
struct RarityPredictor {
    prediction_models: Vec<Box<dyn RarityPredictionModel>>,
    ensemble_weights: Vec<f64>,
    prediction_horizon: u32,
    accuracy_tracker: AccuracyTracker,
}

trait RarityPredictionModel {
    fn predict_rarity_distribution(&self, historical_data: &[RarityTrendPoint], horizon: u32) -> HashMap<CreatureRarity, f64>;
    fn get_model_name(&self) -> &str;
    fn update_model(&mut self, new_data: &[RarityTrendPoint]);
}

#[derive(Debug)]
struct AccuracyTracker {
    model_accuracies: HashMap<String, f64>,
    prediction_history: Vec<PredictionResult>,
    error_analysis: ErrorAnalysis,
}

#[derive(Debug, Clone)]
struct PredictionResult {
    predicted_values: HashMap<CreatureRarity, f64>,
    actual_values: HashMap<CreatureRarity, f64>,
    prediction_timestamp: chrono::DateTime<chrono::Utc>,
    model_used: String,
}

#[derive(Debug)]
struct ErrorAnalysis {
    mean_absolute_errors: HashMap<String, f64>,
    root_mean_square_errors: HashMap<String, f64>,
    directional_accuracy: HashMap<String, f64>,
    confidence_intervals: HashMap<String, (f64, f64)>,
}

#[derive(Debug)]
struct MarketImpactAnalyzer {
    impact_models: Vec<Box<dyn MarketImpactModel>>,
    sensitivity_analysis: SensitivityAnalysis,
    scenario_generator: ScenarioGenerator,
}

trait MarketImpactModel {
    fn assess_impact(&self, rarity_change: &RarityChange) -> MarketImpactAssessment;
    fn get_impact_factors(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
struct RarityChange {
    rarity_type: CreatureRarity,
    old_probability: f64,
    new_probability: f64,
    affected_creatures: Vec<String>,
    change_reason: String,
}

#[derive(Debug, Clone)]
struct MarketImpactAssessment {
    economic_impact: f64,
    player_satisfaction_impact: f64,
    competitive_balance_impact: f64,
    long_term_sustainability_impact: f64,
    recommended_mitigations: Vec<String>,
}

#[derive(Debug)]
struct SensitivityAnalysis {
    parameter_sensitivities: HashMap<String, f64>,
    interaction_effects: HashMap<(String, String), f64>,
    critical_thresholds: HashMap<String, f64>,
}

#[derive(Debug)]
struct ScenarioGenerator {
    scenario_templates: Vec<ScenarioTemplate>,
    parameter_ranges: HashMap<String, (f64, f64)>,
    correlation_matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
struct ScenarioTemplate {
    name: String,
    description: String,
    parameter_adjustments: HashMap<String, f64>,
    expected_outcomes: Vec<String>,
}

#[derive(Debug)]
struct DynamicAdjustmentSystem {
    adjustment_rules: Vec<AdjustmentRule>,
    trigger_monitors: Vec<TriggerMonitor>,
    adjustment_history: Vec<DynamicAdjustment>,
    cooldown_manager: CooldownManager,
}

#[derive(Debug, Clone)]
struct AdjustmentRule {
    rule_id: String,
    trigger_conditions: Vec<TriggerCondition>,
    adjustment_actions: Vec<AdjustmentAction>,
    priority: u8,
    max_adjustments_per_period: u32,
    adjustment_magnitude_limits: (f64, f64),
}

#[derive(Debug, Clone)]
enum TriggerCondition {
    RarityImbalance(CreatureRarity, f64),
    MarketVolatility(f64),
    PlayerComplaint(String, u32),
    EconomicIndicator(String, f64),
    CompetitiveImbalance(f64),
    SeasonalPattern(String),
}

#[derive(Debug, Clone)]
enum AdjustmentAction {
    ModifyBaseWeight(CreatureRarity, f64),
    ActivateTemporalModifier(String),
    IntroduceEventModifier(String),
    AdjustGlobalMultiplier(f64),
    ImplementEmergencyMeasure(String),
}

#[derive(Debug)]
struct TriggerMonitor {
    monitor_id: String,
    monitored_metrics: Vec<String>,
    check_frequency: u32,
    last_check: chrono::DateTime<chrono::Utc>,
    alert_thresholds: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct DynamicAdjustment {
    adjustment_id: String,
    applied_at: chrono::DateTime<chrono::Utc>,
    rule_triggered: String,
    changes_made: Vec<AdjustmentAction>,
    expected_duration: Option<u32>,
    effectiveness_score: Option<f64>,
}

#[derive(Debug)]
struct CooldownManager {
    rule_cooldowns: HashMap<String, chrono::DateTime<chrono::Utc>>,
    global_adjustment_limit: u32,
    current_adjustment_count: u32,
    reset_period: u32,
}

#[derive(Debug)]
struct MarketTracker {
    supply_tracking: SupplyTracker,
    demand_tracking: DemandTracker,
    price_tracking: PriceTracker,
    transaction_analyzer: TransactionAnalyzer,
}

#[derive(Debug)]
struct SupplyTracker {
    current_supplies: HashMap<CreatureRarity, u64>,
    supply_history: Vec<SupplySnapshot>,
    generation_rates: HashMap<CreatureRarity, f64>,
    supply_forecasts: HashMap<CreatureRarity, Vec<f64>>,
}

#[derive(Debug, Clone)]
struct SupplySnapshot {
    timestamp: chrono::DateTime<chrono::Utc>,
    supplies: HashMap<CreatureRarity, u64>,
    generation_events: Vec<GenerationEvent>,
}

#[derive(Debug, Clone)]
struct GenerationEvent {
    rarity: CreatureRarity,
    quantity: u32,
    generation_method: String,
    location: Option<String>,
    player_context: Option<String>,
}

#[derive(Debug)]
struct DemandTracker {
    demand_indicators: HashMap<CreatureRarity, DemandIndicators>,
    demand_forecasts: HashMap<CreatureRarity, Vec<f64>>,
    demand_drivers: Vec<DemandDriver>,
}

#[derive(Debug, Clone)]
struct DemandIndicators {
    search_frequency: u64,
    trade_requests: u64,
    market_prices: Vec<f64>,
    player_preferences: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct DemandDriver {
    driver_type: String,
    impact_strength: f64,
    affected_rarities: Vec<CreatureRarity>,
    temporal_pattern: TemporalPattern,
}

#[derive(Debug, Clone)]
enum TemporalPattern {
    Constant,
    Seasonal(String),
    Cyclical(u32),
    EventDriven(String),
    Trending(f64),
}

#[derive(Debug)]
struct PriceTracker {
    current_prices: HashMap<CreatureRarity, f64>,
    price_history: Vec<PriceSnapshot>,
    volatility_indices: HashMap<CreatureRarity, f64>,
    arbitrage_opportunities: Vec<ArbitrageOpportunity>,
}

#[derive(Debug, Clone)]
struct PriceSnapshot {
    timestamp: chrono::DateTime<chrono::Utc>,
    prices: HashMap<CreatureRarity, f64>,
    volume_data: HashMap<CreatureRarity, u64>,
    market_sentiment: MarketSentiment,
}

#[derive(Debug, Clone)]
enum MarketSentiment {
    Bullish,
    Bearish,
    Neutral,
    Volatile,
}

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    rarity_type: CreatureRarity,
    price_difference: f64,
    profit_potential: f64,
    risk_level: RiskLevel,
    expiry_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

#[derive(Debug)]
struct TransactionAnalyzer {
    transaction_patterns: Vec<TransactionPattern>,
    fraud_detector: FraudDetector,
    market_maker_identifier: MarketMakerIdentifier,
}

#[derive(Debug, Clone)]
struct TransactionPattern {
    pattern_id: String,
    pattern_type: PatternType,
    frequency: f64,
    typical_volume: f64,
    participants: Vec<String>,
}

#[derive(Debug, Clone)]
enum PatternType {
    NormalTrading,
    HighFrequencyTrading,
    LargeBlockTrading,
    SeasonalPattern,
    ManipulativePattern,
    WashTrading,
}

#[derive(Debug)]
struct FraudDetector {
    detection_algorithms: Vec<Box<dyn FraudDetectionAlgorithm>>,
    suspicious_activities: Vec<SuspiciousActivity>,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

trait FraudDetectionAlgorithm {
    fn detect_fraud(&self, transactions: &[Transaction]) -> Vec<SuspiciousActivity>;
    fn get_confidence_threshold(&self) -> f64;
}

#[derive(Debug, Clone)]
struct Transaction {
    transaction_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    buyer: String,
    seller: String,
    rarity: CreatureRarity,
    quantity: u32,
    price: f64,
    transaction_type: TransactionType,
}

#[derive(Debug, Clone)]
enum TransactionType {
    Trade,
    Purchase,
    Gift,
    Auction,
    MarketOrder,
}

#[derive(Debug, Clone)]
struct SuspiciousActivity {
    activity_type: SuspiciousActivityType,
    participants: Vec<String>,
    detected_at: chrono::DateTime<chrono::Utc>,
    confidence_level: f64,
    investigation_priority: InvestigationPriority,
}

#[derive(Debug, Clone)]
enum SuspiciousActivityType {
    PriceManipulation,
    VolumeManipulation,
    WashTrading,
    Collusion,
    Hoarding,
    DumpingAndPumping,
}

#[derive(Debug, Clone)]
enum InvestigationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
struct MarketMakerIdentifier {
    identified_market_makers: Vec<MarketMaker>,
    analysis_algorithms: Vec<Box<dyn MarketMakerAnalysisAlgorithm>>,
}

trait MarketMakerAnalysisAlgorithm {
    fn identify_market_makers(&self, transactions: &[Transaction]) -> Vec<MarketMaker>;
    fn assess_market_impact(&self, market_maker: &MarketMaker) -> MarketMakerImpact;
}

#[derive(Debug, Clone)]
struct MarketMaker {
    participant_id: String,
    trading_volume: f64,
    bid_ask_spreads: HashMap<CreatureRarity, f64>,
    market_share: f64,
    liquidity_provision_score: f64,
}

#[derive(Debug, Clone)]
struct MarketMakerImpact {
    liquidity_impact: f64,
    price_stability_impact: f64,
    market_efficiency_impact: f64,
    competition_impact: f64,
}

impl Default for RarityDistribution {
    fn default() -> Self {
        let mut base_weights = HashMap::new();
        base_weights.insert(CreatureRarity::Common, 0.50);
        base_weights.insert(CreatureRarity::Uncommon, 0.30);
        base_weights.insert(CreatureRarity::Rare, 0.15);
        base_weights.insert(CreatureRarity::Epic, 0.04);
        base_weights.insert(CreatureRarity::Legendary, 0.009);
        base_weights.insert(CreatureRarity::Mythical, 0.001);

        Self {
            base_weights,
            conditional_modifiers: HashMap::new(),
            temporal_modifiers: HashMap::new(),
            location_modifiers: HashMap::new(),
            event_modifiers: HashMap::new(),
            player_modifiers: PlayerBasedModifiers::default(),
            global_multipliers: GlobalRarityMultipliers::default(),
        }
    }
}

impl Default for PlayerBasedModifiers {
    fn default() -> Self {
        Self {
            level_bonuses: HashMap::new(),
            achievement_bonuses: HashMap::new(),
            loyalty_bonuses: LoyaltyBonusSystem::default(),
            streak_bonuses: StreakBonusSystem::default(),
            social_bonuses: SocialBonusSystem::default(),
            spending_bonuses: SpendingBonusSystem::default(),
        }
    }
}

impl Default for LoyaltyBonusSystem {
    fn default() -> Self {
        Self {
            daily_login_bonus: 0.01,
            consecutive_day_multipliers: HashMap::new(),
            account_age_bonuses: HashMap::new(),
            activity_score_thresholds: HashMap::new(),
        }
    }
}

impl Default for StreakBonusSystem {
    fn default() -> Self {
        Self {
            catch_streak_bonuses: HashMap::new(),
            evolution_streak_bonuses: HashMap::new(),
            battle_streak_bonuses: HashMap::new(),
            research_streak_bonuses: HashMap::new(),
        }
    }
}

impl Default for SocialBonusSystem {
    fn default() -> Self {
        Self {
            friend_count_bonuses: HashMap::new(),
            trade_frequency_bonuses: HashMap::new(),
            gift_exchange_bonuses: HashMap::new(),
            raid_participation_bonuses: HashMap::new(),
        }
    }
}

impl Default for SpendingBonusSystem {
    fn default() -> Self {
        Self {
            purchase_tier_bonuses: HashMap::new(),
            premium_member_benefits: HashMap::new(),
            seasonal_pass_bonuses: HashMap::new(),
        }
    }
}

impl Default for GlobalRarityMultipliers {
    fn default() -> Self {
        Self {
            economy_adjustment: 1.0,
            population_balance: 1.0,
            seasonal_global_modifier: 1.0,
            special_event_modifier: 1.0,
            maintenance_mode_modifier: 1.0,
        }
    }
}

impl RaritySystem {
    pub fn new(distribution: &RarityDistribution) -> CreatureEngineResult<Self> {
        let rng = ChaCha8Rng::from_entropy();
        let analytics = RarityAnalytics::new()?;
        let dynamic_adjustments = DynamicAdjustmentSystem::new()?;
        let market_tracker = MarketTracker::new()?;

        Ok(Self {
            distribution: distribution.clone(),
            rng,
            analytics,
            dynamic_adjustments,
            market_tracker,
        })
    }

    pub fn determine_rarity(&mut self, template: &CreatureTemplate) -> CreatureEngineResult<CreatureRarity> {
        let mut weights = self.distribution.base_weights.clone();
        
        self.apply_conditional_modifiers(&mut weights, template)?;
        self.apply_temporal_modifiers(&mut weights)?;
        self.apply_global_multipliers(&mut weights)?;
        
        self.select_rarity_by_weight(&mut weights)
    }

    pub fn calculate_rarity_probability(&self, rarity: CreatureRarity, template: &CreatureTemplate) -> CreatureEngineResult<f64> {
        let mut weights = self.distribution.base_weights.clone();
        
        self.apply_conditional_modifiers(&mut weights, template)?;
        self.apply_temporal_modifiers(&mut weights)?;
        self.apply_global_multipliers(&mut weights)?;
        
        let total_weight: f64 = weights.values().sum();
        Ok(weights.get(&rarity).copied().unwrap_or(0.0) / total_weight)
    }

    pub fn get_rarity_forecast(&self, horizon_days: u32) -> CreatureEngineResult<RarityForecast> {
        self.analytics.prediction_engine.generate_forecast(horizon_days)
    }

    pub fn analyze_rarity_trends(&self) -> CreatureEngineResult<RarityTrendAnalysis> {
        self.analytics.trend_analyzer.generate_analysis()
    }

    pub fn detect_rarity_anomalies(&self) -> CreatureEngineResult<Vec<RarityAnomaly>> {
        self.analytics.trend_analyzer.anomaly_detector.detect_current_anomalies()
    }

    pub fn assess_market_impact(&self, proposed_changes: &[RarityChange]) -> CreatureEngineResult<Vec<MarketImpactAssessment>> {
        let mut assessments = Vec::new();
        
        for change in proposed_changes {
            let assessment = self.analytics.market_impact_analyzer.assess_impact(change);
            assessments.push(assessment);
        }
        
        Ok(assessments)
    }

    pub fn update_rarity_distribution(&mut self, updates: RarityDistributionUpdate) -> CreatureEngineResult<()> {
        if let Some(new_weights) = updates.base_weight_changes {
            for (rarity, weight) in new_weights {
                self.distribution.base_weights.insert(rarity, weight);
            }
        }
        
        if let Some(new_modifiers) = updates.conditional_modifier_changes {
            for (key, modifier) in new_modifiers {
                self.distribution.conditional_modifiers.insert(key, modifier);
            }
        }
        
        if let Some(new_temporal) = updates.temporal_modifier_changes {
            for (key, modifier) in new_temporal {
                self.distribution.temporal_modifiers.insert(key, modifier);
            }
        }
        
        self.validate_distribution()?;
        Ok(())
    }

    pub fn add_event_rarity_modifier(&mut self, event: EventRarityModifier) -> CreatureEngineResult<()> {
        self.distribution.event_modifiers.insert(event.event_id.clone(), event);
        Ok(())
    }

    pub fn remove_expired_modifiers(&mut self) -> CreatureEngineResult<Vec<String>> {
        let now = chrono::Utc::now();
        let mut removed_modifiers = Vec::new();
        
        self.distribution.temporal_modifiers.retain(|key, modifier| {
            if let Some(end_time) = modifier.end_time {
                if now > end_time {
                    removed_modifiers.push(key.clone());
                    return false;
                }
            }
            true
        });
        
        self.distribution.event_modifiers.retain(|key, event| {
            if now > event.duration.end {
                removed_modifiers.push(key.clone());
                return false;
            }
            true
        });
        
        Ok(removed_modifiers)
    }

    pub fn get_active_modifiers(&self) -> Vec<ActiveModifier> {
        let mut active = Vec::new();
        let now = chrono::Utc::now();
        
        for (key, modifier) in &self.distribution.conditional_modifiers {
            active.push(ActiveModifier {
                modifier_id: key.clone(),
                modifier_type: "Conditional".to_string(),
                activation_time: None,
                expiry_time: None,
                current_impact: self.calculate_modifier_impact(modifier),
            });
        }
        
        for (key, modifier) in &self.distribution.temporal_modifiers {
            if modifier.start_time <= now && modifier.end_time.map_or(true, |end| now <= end) {
                active.push(ActiveModifier {
                    modifier_id: key.clone(),
                    modifier_type: "Temporal".to_string(),
                    activation_time: Some(modifier.start_time),
                    expiry_time: modifier.end_time,
                    current_impact: self.calculate_temporal_modifier_impact(modifier),
                });
            }
        }
        
        for (key, event) in &self.distribution.event_modifiers {
            if now >= event.duration.start && now <= event.duration.end {
                active.push(ActiveModifier {
                    modifier_id: key.clone(),
                    modifier_type: "Event".to_string(),
                    activation_time: Some(event.duration.start),
                    expiry_time: Some(event.duration.end),
                    current_impact: self.calculate_event_modifier_impact(event),
                });
            }
        }
        
        active
    }

    pub fn tier_count(&self) -> usize {
        CreatureRarity::all_variants().len()
    }

    pub fn get_rarity_statistics(&self) -> HashMap<CreatureRarity, RarityStats> {
        self.analytics.rarity_statistics.clone()
    }

    fn apply_conditional_modifiers(&self, weights: &mut HashMap<CreatureRarity, f64>, template: &CreatureTemplate) -> CreatureEngineResult<()> {
        for modifier in self.distribution.conditional_modifiers.values() {
            if self.check_condition_met(&modifier.condition_type, template)? {
                for (rarity, adjustment) in &modifier.rarity_adjustments {
                    if let Some(weight) = weights.get_mut(rarity) {
                        *weight *= adjustment;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply_temporal_modifiers(&self, weights: &mut HashMap<CreatureRarity, f64>) -> CreatureEngineResult<()> {
        let now = chrono::Utc::now();
        
        for modifier in self.distribution.temporal_modifiers.values() {
            if modifier.start_time <= now && modifier.end_time.map_or(true, |end| now <= end) {
                for (rarity, boost) in &modifier.rarity_boosts {
                    if let Some(weight) = weights.get_mut(rarity) {
                        *weight *= boost;
                    }
                }
            }
        }
        
        Ok(())
    }

    fn apply_global_multipliers(&self, weights: &mut HashMap<CreatureRarity, f64>) -> CreatureEngineResult<()> {
        let global = &self.distribution.global_multipliers;
        let combined_multiplier = global.economy_adjustment 
            * global.population_balance 
            * global.seasonal_global_modifier 
            * global.special_event_modifier 
            * global.maintenance_mode_modifier;
        
        for weight in weights.values_mut() {
            *weight *= combined_multiplier;
        }
        
        Ok(())
    }

    fn select_rarity_by_weight(&mut self, weights: &HashMap<CreatureRarity, f64>) -> CreatureEngineResult<CreatureRarity> {
        let total_weight: f64 = weights.values().sum();
        if total_weight <= 0.0 {
            return Ok(CreatureRarity::Common);
        }
        
        let random_value = self.rng.gen::<f64>() * total_weight;
        let mut cumulative_weight = 0.0;
        
        for rarity in CreatureRarity::all_variants() {
            cumulative_weight += weights.get(&rarity).unwrap_or(&0.0);
            if random_value <= cumulative_weight {
                return Ok(rarity);
            }
        }
        
        Ok(CreatureRarity::Common)
    }

    fn check_condition_met(&self, condition: &ConditionType, template: &CreatureTemplate) -> CreatureEngineResult<bool> {
        match condition {
            ConditionType::Weather(weather) => {
                Ok(false)
            }
            ConditionType::TimeOfDay(start, end) => {
                let now = chrono::Local::now();
                let hour = now.time().hour() as u8;
                Ok(hour >= *start && hour <= *end)
            }
            ConditionType::Season(season) => {
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn validate_distribution(&self) -> CreatureEngineResult<()> {
        let total_weight: f64 = self.distribution.base_weights.values().sum();
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(CreatureEngineError::RarityError(
                format!("Base weights sum to {}, expected 1.0", total_weight)
            ));
        }
        Ok(())
    }

    fn calculate_modifier_impact(&self, modifier: &RarityModifier) -> f64 {
        modifier.rarity_adjustments.values().sum::<f64>() / modifier.rarity_adjustments.len() as f64
    }

    fn calculate_temporal_modifier_impact(&self, modifier: &TemporalRarityModifier) -> f64 {
        modifier.rarity_boosts.values().sum::<f64>() / modifier.rarity_boosts.len() as f64
    }

    fn calculate_event_modifier_impact(&self, event: &EventRarityModifier) -> f64 {
        event.rarity_changes.base_rarity_multipliers.values().sum::<f64>() 
            / event.rarity_changes.base_rarity_multipliers.len() as f64
    }
}

impl CreatureRarity {
    pub fn all_variants() -> Vec<CreatureRarity> {
        vec![
            CreatureRarity::Common,
            CreatureRarity::Uncommon,
            CreatureRarity::Rare,
            CreatureRarity::Epic,
            CreatureRarity::Legendary,
            CreatureRarity::Mythical,
        ]
    }
}

impl RarityAnalytics {
    fn new() -> CreatureEngineResult<Self> {
        Ok(Self {
            rarity_statistics: HashMap::new(),
            trend_analyzer: TrendAnalyzer::new(),
            prediction_engine: RarityPredictor::new(),
            market_impact_analyzer: MarketImpactAnalyzer::new(),
        })
    }
}

impl TrendAnalyzer {
    fn new() -> Self {
        Self {
            historical_data: Vec::new(),
            moving_averages: HashMap::new(),
            volatility_indices: HashMap::new(),
            anomaly_detector: AnomalyDetector::new(),
        }
    }
    
    fn generate_analysis(&self) -> CreatureEngineResult<RarityTrendAnalysis> {
        Ok(RarityTrendAnalysis {
            trends: HashMap::new(),
            volatility: HashMap::new(),
            predictions: HashMap::new(),
            anomalies: Vec::new(),
        })
    }
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            detection_algorithms: Vec::new(),
            anomaly_history: Vec::new(),
            threshold_parameters: HashMap::new(),
        }
    }
    
    fn detect_current_anomalies(&self) -> CreatureEngineResult<Vec<RarityAnomaly>> {
        Ok(Vec::new())
    }
}

impl RarityPredictor {
    fn new() -> Self {
        Self {
            prediction_models: Vec::new(),
            ensemble_weights: Vec::new(),
            prediction_horizon: 30,
            accuracy_tracker: AccuracyTracker::new(),
        }
    }
    
    fn generate_forecast(&self, horizon_days: u32) -> CreatureEngineResult<RarityForecast> {
        Ok(RarityForecast {
            horizon_days,
            predictions: HashMap::new(),
            confidence_intervals: HashMap::new(),
            market_scenarios: Vec::new(),
        })
    }
}

impl AccuracyTracker {
    fn new() -> Self {
        Self {
            model_accuracies: HashMap::new(),
            prediction_history: Vec::new(),
            error_analysis: ErrorAnalysis::new(),
        }
    }
}

impl ErrorAnalysis {
    fn new() -> Self {
        Self {
            mean_absolute_errors: HashMap::new(),
            root_mean_square_errors: HashMap::new(),
            directional_accuracy: HashMap::new(),
            confidence_intervals: HashMap::new(),
        }
    }
}

impl MarketImpactAnalyzer {
    fn new() -> Self {
        Self {
            impact_models: Vec::new(),
            sensitivity_analysis: SensitivityAnalysis::new(),
            scenario_generator: ScenarioGenerator::new(),
        }
    }
    
    fn assess_impact(&self, change: &RarityChange) -> MarketImpactAssessment {
        MarketImpactAssessment {
            economic_impact: 0.5,
            player_satisfaction_impact: 0.3,
            competitive_balance_impact: 0.4,
            long_term_sustainability_impact: 0.6,
            recommended_mitigations: vec!["Monitor closely".to_string()],
        }
    }
}

impl SensitivityAnalysis {
    fn new() -> Self {
        Self {
            parameter_sensitivities: HashMap::new(),
            interaction_effects: HashMap::new(),
            critical_thresholds: HashMap::new(),
        }
    }
}

impl ScenarioGenerator {
    fn new() -> Self {
        Self {
            scenario_templates: Vec::new(),
            parameter_ranges: HashMap::new(),
            correlation_matrix: Vec::new(),
        }
    }
}

impl DynamicAdjustmentSystem {
    fn new() -> CreatureEngineResult<Self> {
        Ok(Self {
            adjustment_rules: Vec::new(),
            trigger_monitors: Vec::new(),
            adjustment_history: Vec::new(),
            cooldown_manager: CooldownManager::new(),
        })
    }
}

impl CooldownManager {
    fn new() -> Self {
        Self {
            rule_cooldowns: HashMap::new(),
            global_adjustment_limit: 10,
            current_adjustment_count: 0,
            reset_period: 86400,
        }
    }
}

impl MarketTracker {
    fn new() -> CreatureEngineResult<Self> {
        Ok(Self {
            supply_tracking: SupplyTracker::new(),
            demand_tracking: DemandTracker::new(),
            price_tracking: PriceTracker::new(),
            transaction_analyzer: TransactionAnalyzer::new(),
        })
    }
}

impl SupplyTracker {
    fn new() -> Self {
        Self {
            current_supplies: HashMap::new(),
            supply_history: Vec::new(),
            generation_rates: HashMap::new(),
            supply_forecasts: HashMap::new(),
        }
    }
}

impl DemandTracker {
    fn new() -> Self {
        Self {
            demand_indicators: HashMap::new(),
            demand_forecasts: HashMap::new(),
            demand_drivers: Vec::new(),
        }
    }
}

impl PriceTracker {
    fn new() -> Self {
        Self {
            current_prices: HashMap::new(),
            price_history: Vec::new(),
            volatility_indices: HashMap::new(),
            arbitrage_opportunities: Vec::new(),
        }
    }
}

impl TransactionAnalyzer {
    fn new() -> Self {
        Self {
            transaction_patterns: Vec::new(),
            fraud_detector: FraudDetector::new(),
            market_maker_identifier: MarketMakerIdentifier::new(),
        }
    }
}

impl FraudDetector {
    fn new() -> Self {
        Self {
            detection_algorithms: Vec::new(),
            suspicious_activities: Vec::new(),
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }
}

impl MarketMakerIdentifier {
    fn new() -> Self {
        Self {
            identified_market_makers: Vec::new(),
            analysis_algorithms: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityDistributionUpdate {
    pub base_weight_changes: Option<HashMap<CreatureRarity, f64>>,
    pub conditional_modifier_changes: Option<HashMap<String, RarityModifier>>,
    pub temporal_modifier_changes: Option<HashMap<String, TemporalRarityModifier>>,
    pub global_multiplier_changes: Option<GlobalRarityMultipliers>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModifier {
    pub modifier_id: String,
    pub modifier_type: String,
    pub activation_time: Option<chrono::DateTime<chrono::Utc>>,
    pub expiry_time: Option<chrono::DateTime<chrono::Utc>>,
    pub current_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityForecast {
    pub horizon_days: u32,
    pub predictions: HashMap<CreatureRarity, Vec<f64>>,
    pub confidence_intervals: HashMap<CreatureRarity, Vec<(f64, f64)>>,
    pub market_scenarios: Vec<MarketScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketScenario {
    pub scenario_name: String,
    pub probability: f64,
    pub expected_changes: HashMap<CreatureRarity, f64>,
    pub impact_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityTrendAnalysis {
    pub trends: HashMap<CreatureRarity, TrendDirection>,
    pub volatility: HashMap<CreatureRarity, f64>,
    pub predictions: HashMap<CreatureRarity, f64>,
    pub anomalies: Vec<RarityAnomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_system_creation() {
        let distribution = RarityDistribution::default();
        let system = RaritySystem::new(&distribution);
        assert!(system.is_ok());
    }

    #[test]
    fn test_rarity_probability_calculation() {
        let distribution = RarityDistribution::default();
        let system = RaritySystem::new(&distribution).unwrap();
        
        // Create a mock template for testing
        let template = CreatureTemplate {
            id: "test".to_string(),
            name: "Test Creature".to_string(),
            // ... fill in required fields with defaults
            description: String::new(),
            category: String::new(),
            base_stats: HashMap::new(),
            types: Vec::new(),
            abilities: Vec::new(),
            possible_traits: Vec::new(),
            evolution_chain: Vec::new(),
            spawn_data: super::super::templates::SpawnData::default(),
            visual_data: super::super::templates::VisualData::default(),
            behavioral_data: super::super::templates::BehavioralData::default(),
            inheritance: None,
            tags: Vec::new(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let prob = system.calculate_rarity_probability(CreatureRarity::Common, &template);
        assert!(prob.is_ok());
        assert!(prob.unwrap() > 0.0);
    }

    #[test]
    fn test_rarity_distribution_validation() {
        let distribution = RarityDistribution::default();
        let system = RaritySystem::new(&distribution).unwrap();
        let result = system.validate_distribution();
        assert!(result.is_ok());
    }
}