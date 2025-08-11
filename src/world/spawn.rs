/*
* 开发心理过程：
* 1. 设计灵活的生成系统，支持Pokemon、NPC、物品的动态生成
* 2. 实现基于权重的随机生成算法，支持稀有度控制
* 3. 支持条件生成：时间、天气、玩家等级等因素影响
* 4. 集成生成冷却和限制机制，防止无限生成
* 5. 提供预设生成模板，快速配置不同区域的生成规则
* 6. 优化性能，支持大世界中大量生成点的高效管理
* 7. 实现生成事件系统，支持特殊事件触发的生成
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        species::{PokemonSpecies, SpeciesId},
        individual::IndividualPokemon,
    },
    world::{
        environment::{WeatherCondition, TimeOfDay, Season},
        tile::{TerrainType, TilePosition},
    },
    player::trainer::TrainerLevel,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Component)]
pub struct SpawnManager {
    /// 生成点列表
    pub spawn_points: HashMap<Uuid, SpawnPoint>,
    /// 全局生成配置
    pub global_config: GlobalSpawnConfig,
    /// 活跃生成区域
    pub active_regions: Vec<SpawnRegion>,
    /// 生成历史
    pub spawn_history: Vec<SpawnRecord>,
    /// 性能统计
    pub performance_stats: SpawnStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub id: Uuid,
    pub position: Vec2,
    pub spawn_type: SpawnType,
    pub spawn_rules: SpawnRules,
    pub cooldown: SpawnCooldown,
    pub last_spawn_time: Option<f64>,
    pub spawn_count: u32,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnType {
    Pokemon,        // Pokemon生成点
    WildPokemon,    // 野生Pokemon（随机遭遇）
    NPC,           // NPC生成点
    Item,          // 物品生成点
    Trainer,       // 训练师生成点
    Event,         // 事件生成点
    Custom(u16),   // 自定义类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRules {
    /// 生成表
    pub spawn_table: Vec<SpawnEntry>,
    /// 生成条件
    pub conditions: Vec<SpawnCondition>,
    /// 最大同时存在数量
    pub max_concurrent: u32,
    /// 生成半径
    pub spawn_radius: f32,
    /// 生成权重修正器
    pub weight_modifiers: Vec<WeightModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnEntry {
    pub id: String,
    pub entry_type: SpawnEntryType,
    pub weight: f32,
    pub level_range: Option<(u8, u8)>,
    pub rarity: SpawnRarity,
    pub conditions: Vec<SpawnCondition>,
    pub data: SpawnData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnEntryType {
    Pokemon(SpeciesId),
    NPC(String),
    Item(u32),
    Trainer(String),
    Event(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnRarity {
    Common,      // 常见 (60-70%)
    Uncommon,    // 不常见 (20-25%)
    Rare,        // 稀有 (8-12%)
    VeryRare,    // 非常稀有 (2-4%)
    Legendary,   // 传说 (0.1-1%)
    Mythical,    // 幻之 (<0.1%)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnCondition {
    TimeOfDay(TimeOfDay),
    Weather(WeatherCondition),
    Season(Season),
    PlayerLevel(u8, u8), // min, max
    TerrainType(TerrainType),
    ItemHeld(u32),
    QuestActive(String),
    PokemonInParty(SpeciesId),
    DateRange(String, String), // start_date, end_date
    Probability(f32), // 0.0-1.0
    CustomCondition(String, String), // condition_name, parameters
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnData {
    /// 特定于类型的数据
    pub specific_data: HashMap<String, String>,
    /// 生成数量范围
    pub quantity_range: (u32, u32),
    /// 生成位置偏移
    pub position_offset: Vec2,
    /// 生成时的特殊效果
    pub spawn_effects: Vec<SpawnEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnEffect {
    pub effect_type: SpawnEffectType,
    pub duration: f32,
    pub intensity: f32,
    pub sound: Option<String>,
    pub visual: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnEffectType {
    Particles,
    Sound,
    Flash,
    Smoke,
    Sparkles,
    Custom(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnCooldown {
    /// 冷却时间（秒）
    pub duration: f32,
    /// 冷却类型
    pub cooldown_type: CooldownType,
    /// 是否随机化冷却时间
    pub randomize: bool,
    /// 随机化范围 (0.0-1.0)
    pub randomize_range: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooldownType {
    Fixed,      // 固定冷却
    Scaling,    // 按稀有度缩放
    Adaptive,   // 根据生成数量自适应
    PlayerBased, // 基于玩家行为
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightModifier {
    pub condition: SpawnCondition,
    pub multiplier: f32,
    pub additive: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRegion {
    pub id: Uuid,
    pub name: String,
    pub bounds: SpawnBounds,
    pub region_type: RegionType,
    pub spawn_density: f32,
    pub biome_modifiers: HashMap<TerrainType, f32>,
    pub special_rules: Vec<SpecialSpawnRule>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnBounds {
    Circle { center: Vec2, radius: f32 },
    Rectangle { min: Vec2, max: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionType {
    Overworld,   // 主世界
    Cave,        // 洞穴
    Water,       // 水域
    Building,    // 建筑内部
    Special,     // 特殊区域
    Dungeon,     // 地牢
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialSpawnRule {
    pub rule_type: SpecialRuleType,
    pub trigger_condition: SpawnCondition,
    pub spawn_modifications: Vec<SpawnModification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialRuleType {
    Swarm,       // 群体生成
    Outbreak,    // 大量生成
    Migration,   // 迁徙
    Seasonal,    // 季节性
    Event,       // 事件性
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnModification {
    WeightMultiplier(f32),
    CooldownModifier(f32),
    QuantityMultiplier(f32),
    AddSpawnEntry(SpawnEntry),
    RemoveSpawnEntry(String),
}

#[derive(Debug, Clone)]
pub struct GlobalSpawnConfig {
    /// 全局生成率
    pub global_spawn_rate: f32,
    /// 最大同时生成数量
    pub max_concurrent_spawns: u32,
    /// 更新频率（秒）
    pub update_interval: f32,
    /// 是否启用动态生成
    pub dynamic_spawning: bool,
    /// 玩家附近的生成优先级
    pub player_proximity_priority: bool,
    /// 性能限制
    pub performance_limits: PerformanceLimits,
}

#[derive(Debug, Clone)]
pub struct PerformanceLimits {
    /// 每帧最大生成数量
    pub max_spawns_per_frame: u32,
    /// 生成计算时间限制（毫秒）
    pub max_computation_time_ms: f32,
    /// 内存使用限制
    pub max_memory_usage_mb: f32,
}

#[derive(Debug, Clone)]
pub struct SpawnRecord {
    pub timestamp: f64,
    pub spawn_point_id: Uuid,
    pub spawned_entity: SpawnedEntity,
    pub conditions_met: Vec<SpawnCondition>,
    pub actual_weight: f32,
}

#[derive(Debug, Clone)]
pub enum SpawnedEntity {
    Pokemon {
        species_id: SpeciesId,
        level: u8,
        individual_id: Uuid,
    },
    NPC {
        npc_type: String,
        entity_id: Uuid,
    },
    Item {
        item_id: u32,
        quantity: u32,
        entity_id: Uuid,
    },
    Event {
        event_id: String,
        data: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct SpawnStats {
    pub total_spawns: u64,
    pub successful_spawns: u64,
    pub failed_spawns: u64,
    pub average_spawn_time_ms: f32,
    pub active_spawn_points: u32,
    pub memory_usage_mb: f32,
}

impl SpawnManager {
    pub fn new(global_config: GlobalSpawnConfig) -> Self {
        Self {
            spawn_points: HashMap::new(),
            global_config,
            active_regions: Vec::new(),
            spawn_history: Vec::new(),
            performance_stats: SpawnStats::default(),
        }
    }

    /// 添加生成点
    pub fn add_spawn_point(&mut self, spawn_point: SpawnPoint) {
        self.spawn_points.insert(spawn_point.id, spawn_point);
    }

    /// 移除生成点
    pub fn remove_spawn_point(&mut self, id: Uuid) -> bool {
        self.spawn_points.remove(&id).is_some()
    }

    /// 添加生成区域
    pub fn add_spawn_region(&mut self, region: SpawnRegion) {
        self.active_regions.push(region);
    }

    /// 更新生成系统
    pub fn update(
        &mut self,
        current_time: f64,
        player_position: Vec2,
        player_level: u8,
        current_weather: WeatherCondition,
        current_time_of_day: TimeOfDay,
        current_season: Season,
        rng: &mut RandomGenerator,
    ) -> GameResult<Vec<SpawnResult>> {
        let mut spawn_results = Vec::new();
        let frame_start_time = std::time::Instant::now();

        // 重置帧统计
        let mut spawns_this_frame = 0;

        // 更新每个生成点
        let spawn_point_ids: Vec<Uuid> = self.spawn_points.keys().copied().collect();
        
        for spawn_point_id in spawn_point_ids {
            // 检查性能限制
            if spawns_this_frame >= self.global_config.performance_limits.max_spawns_per_frame {
                break;
            }
            
            if frame_start_time.elapsed().as_millis() as f32 > 
                self.global_config.performance_limits.max_computation_time_ms {
                break;
            }

            if let Some(spawn_point) = self.spawn_points.get_mut(&spawn_point_id) {
                if !spawn_point.is_active {
                    continue;
                }

                // 检查冷却时间
                if let Some(last_spawn) = spawn_point.last_spawn_time {
                    let cooldown_duration = self.calculate_cooldown_duration(spawn_point, rng);
                    if current_time - last_spawn < cooldown_duration as f64 {
                        continue;
                    }
                }

                // 检查玩家距离
                let distance_to_player = spawn_point.position.distance(player_position);
                if distance_to_player > spawn_point.spawn_rules.spawn_radius {
                    continue;
                }

                // 尝试生成
                if let Some(result) = self.attempt_spawn(
                    spawn_point,
                    player_position,
                    player_level,
                    current_weather,
                    current_time_of_day,
                    current_season,
                    current_time,
                    rng,
                )? {
                    spawn_results.push(result);
                    spawns_this_frame += 1;
                    spawn_point.last_spawn_time = Some(current_time);
                    spawn_point.spawn_count += 1;
                }
            }
        }

        // 更新统计
        self.performance_stats.total_spawns += spawns_this_frame as u64;
        self.performance_stats.active_spawn_points = self.spawn_points.values()
            .filter(|sp| sp.is_active).count() as u32;

        Ok(spawn_results)
    }

    fn attempt_spawn(
        &mut self,
        spawn_point: &SpawnPoint,
        player_position: Vec2,
        player_level: u8,
        weather: WeatherCondition,
        time_of_day: TimeOfDay,
        season: Season,
        current_time: f64,
        rng: &mut RandomGenerator,
    ) -> GameResult<Option<SpawnResult>> {
        // 检查全局条件
        if !self.check_global_conditions(player_level, weather, time_of_day, season)? {
            return Ok(None);
        }

        // 选择要生成的条目
        let selected_entry = self.select_spawn_entry(
            &spawn_point.spawn_rules,
            player_level,
            weather,
            time_of_day,
            season,
            rng,
        )?;

        let selected_entry = match selected_entry {
            Some(entry) => entry,
            None => return Ok(None),
        };

        // 检查条目特定条件
        if !self.check_entry_conditions(&selected_entry, player_level, weather, time_of_day, season)? {
            return Ok(None);
        }

        // 生成实体
        let spawned_entity = self.spawn_entity(
            &selected_entry,
            spawn_point.position + selected_entry.data.position_offset,
            rng,
        )?;

        // 记录生成历史
        let spawn_record = SpawnRecord {
            timestamp: current_time,
            spawn_point_id: spawn_point.id,
            spawned_entity: spawned_entity.clone(),
            conditions_met: selected_entry.conditions.clone(),
            actual_weight: selected_entry.weight,
        };
        
        self.spawn_history.push(spawn_record);
        
        // 限制历史记录长度
        if self.spawn_history.len() > 1000 {
            self.spawn_history.drain(0..500);
        }

        Ok(Some(SpawnResult {
            spawn_point_id: spawn_point.id,
            spawned_entity,
            position: spawn_point.position + selected_entry.data.position_offset,
            effects: selected_entry.data.spawn_effects.clone(),
        }))
    }

    fn select_spawn_entry(
        &self,
        spawn_rules: &SpawnRules,
        player_level: u8,
        weather: WeatherCondition,
        time_of_day: TimeOfDay,
        season: Season,
        rng: &mut RandomGenerator,
    ) -> GameResult<Option<SpawnEntry>> {
        let mut weighted_entries = Vec::new();
        let mut total_weight = 0.0f32;

        for entry in &spawn_rules.spawn_table {
            // 检查基础条件
            if !self.check_entry_conditions(entry, player_level, weather, time_of_day, season)? {
                continue;
            }

            // 计算最终权重
            let mut final_weight = entry.weight;
            
            // 应用权重修正器
            for modifier in &spawn_rules.weight_modifiers {
                if self.check_single_condition(&modifier.condition, player_level, weather, time_of_day, season)? {
                    final_weight = final_weight * modifier.multiplier + modifier.additive;
                }
            }

            // 应用稀有度修正
            final_weight *= self.get_rarity_weight_modifier(entry.rarity);

            if final_weight > 0.0 {
                weighted_entries.push((entry.clone(), final_weight));
                total_weight += final_weight;
            }
        }

        if weighted_entries.is_empty() || total_weight <= 0.0 {
            return Ok(None);
        }

        // 加权随机选择
        let mut random_value = rng.range_f32(0.0, total_weight);
        
        for (entry, weight) in weighted_entries {
            random_value -= weight;
            if random_value <= 0.0 {
                return Ok(Some(entry));
            }
        }

        // fallback: 返回最后一个条目
        Ok(weighted_entries.last().map(|(entry, _)| entry.clone()))
    }

    fn spawn_entity(
        &self,
        entry: &SpawnEntry,
        position: Vec2,
        rng: &mut RandomGenerator,
    ) -> GameResult<SpawnedEntity> {
        let quantity = rng.range(entry.data.quantity_range.0, entry.data.quantity_range.1 + 1);
        
        match &entry.entry_type {
            SpawnEntryType::Pokemon(species_id) => {
                let level = if let Some((min, max)) = entry.level_range {
                    rng.range(min, max + 1)
                } else {
                    5 // 默认等级
                };

                Ok(SpawnedEntity::Pokemon {
                    species_id: *species_id,
                    level,
                    individual_id: Uuid::new_v4(),
                })
            },
            SpawnEntryType::NPC(npc_type) => {
                Ok(SpawnedEntity::NPC {
                    npc_type: npc_type.clone(),
                    entity_id: Uuid::new_v4(),
                })
            },
            SpawnEntryType::Item(item_id) => {
                Ok(SpawnedEntity::Item {
                    item_id: *item_id,
                    quantity,
                    entity_id: Uuid::new_v4(),
                })
            },
            SpawnEntryType::Trainer(trainer_type) => {
                Ok(SpawnedEntity::NPC {
                    npc_type: format!("trainer_{}", trainer_type),
                    entity_id: Uuid::new_v4(),
                })
            },
            SpawnEntryType::Event(event_type) => {
                let mut event_data = HashMap::new();
                event_data.insert("type".to_string(), event_type.clone());
                event_data.insert("position".to_string(), format!("{},{}", position.x, position.y));
                
                Ok(SpawnedEntity::Event {
                    event_id: event_type.clone(),
                    data: event_data,
                })
            },
        }
    }

    fn check_global_conditions(
        &self,
        player_level: u8,
        weather: WeatherCondition,
        time_of_day: TimeOfDay,
        season: Season,
    ) -> GameResult<bool> {
        // 检查全局生成率
        if self.global_config.global_spawn_rate <= 0.0 {
            return Ok(false);
        }

        // 检查最大并发生成数
        let current_active = self.performance_stats.active_spawn_points;
        if current_active >= self.global_config.max_concurrent_spawns {
            return Ok(false);
        }

        Ok(true)
    }

    fn check_entry_conditions(
        &self,
        entry: &SpawnEntry,
        player_level: u8,
        weather: WeatherCondition,
        time_of_day: TimeOfDay,
        season: Season,
    ) -> GameResult<bool> {
        for condition in &entry.conditions {
            if !self.check_single_condition(condition, player_level, weather, time_of_day, season)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn check_single_condition(
        &self,
        condition: &SpawnCondition,
        player_level: u8,
        weather: WeatherCondition,
        time_of_day: TimeOfDay,
        season: Season,
    ) -> GameResult<bool> {
        match condition {
            SpawnCondition::TimeOfDay(required_time) => {
                Ok(*required_time == time_of_day)
            },
            SpawnCondition::Weather(required_weather) => {
                Ok(*required_weather == weather)
            },
            SpawnCondition::Season(required_season) => {
                Ok(*required_season == season)
            },
            SpawnCondition::PlayerLevel(min, max) => {
                Ok(player_level >= *min && player_level <= *max)
            },
            SpawnCondition::Probability(prob) => {
                // 这里需要随机数生成器，简化实现
                Ok(true) // 应该基于概率判断
            },
            SpawnCondition::TerrainType(_terrain) => {
                // 需要地形信息，简化实现
                Ok(true)
            },
            SpawnCondition::ItemHeld(_item_id) => {
                // 需要玩家物品信息，简化实现
                Ok(true)
            },
            SpawnCondition::QuestActive(_quest_name) => {
                // 需要任务系统，简化实现
                Ok(true)
            },
            SpawnCondition::PokemonInParty(_species_id) => {
                // 需要玩家队伍信息，简化实现
                Ok(true)
            },
            SpawnCondition::DateRange(_start, _end) => {
                // 需要日期系统，简化实现
                Ok(true)
            },
            SpawnCondition::CustomCondition(_name, _params) => {
                // 自定义条件，简化实现
                Ok(true)
            },
        }
    }

    fn get_rarity_weight_modifier(&self, rarity: SpawnRarity) -> f32 {
        match rarity {
            SpawnRarity::Common => 1.0,
            SpawnRarity::Uncommon => 0.4,
            SpawnRarity::Rare => 0.15,
            SpawnRarity::VeryRare => 0.05,
            SpawnRarity::Legendary => 0.01,
            SpawnRarity::Mythical => 0.001,
        }
    }

    fn calculate_cooldown_duration(&self, spawn_point: &SpawnPoint, rng: &mut RandomGenerator) -> f32 {
        let mut duration = spawn_point.cooldown.duration;

        match spawn_point.cooldown.cooldown_type {
            CooldownType::Fixed => {},
            CooldownType::Scaling => {
                // 基于最近生成的稀有度调整
                duration *= 1.0; // 简化实现
            },
            CooldownType::Adaptive => {
                // 基于生成数量调整
                let spawn_factor = (spawn_point.spawn_count as f32 / 10.0).min(2.0);
                duration *= 1.0 + spawn_factor;
            },
            CooldownType::PlayerBased => {
                // 基于玩家行为调整，简化实现
                duration *= 1.0;
            },
        }

        if spawn_point.cooldown.randomize {
            let range = spawn_point.cooldown.randomize_range;
            let min = duration * (1.0 - range);
            let max = duration * (1.0 + range);
            duration = rng.range_f32(min, max);
        }

        duration
    }

    /// 获取指定位置附近的生成点
    pub fn get_nearby_spawn_points(&self, position: Vec2, radius: f32) -> Vec<&SpawnPoint> {
        self.spawn_points
            .values()
            .filter(|sp| sp.position.distance(position) <= radius)
            .collect()
    }

    /// 激活/停用生成点
    pub fn set_spawn_point_active(&mut self, id: Uuid, active: bool) -> GameResult<()> {
        if let Some(spawn_point) = self.spawn_points.get_mut(&id) {
            spawn_point.is_active = active;
            Ok(())
        } else {
            Err(GameError::WorldError(format!("找不到生成点: {}", id)))
        }
    }

    /// 清理旧的生成记录
    pub fn cleanup_old_records(&mut self, max_age_seconds: f64, current_time: f64) {
        let cutoff_time = current_time - max_age_seconds;
        self.spawn_history.retain(|record| record.timestamp > cutoff_time);
    }

    /// 获取生成统计
    pub fn get_spawn_stats(&self) -> &SpawnStats {
        &self.performance_stats
    }

    /// 重置生成点冷却
    pub fn reset_spawn_point_cooldown(&mut self, id: Uuid) -> GameResult<()> {
        if let Some(spawn_point) = self.spawn_points.get_mut(&id) {
            spawn_point.last_spawn_time = None;
            Ok(())
        } else {
            Err(GameError::WorldError(format!("找不到生成点: {}", id)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpawnResult {
    pub spawn_point_id: Uuid,
    pub spawned_entity: SpawnedEntity,
    pub position: Vec2,
    pub effects: Vec<SpawnEffect>,
}

impl Default for GlobalSpawnConfig {
    fn default() -> Self {
        Self {
            global_spawn_rate: 1.0,
            max_concurrent_spawns: 100,
            update_interval: 1.0,
            dynamic_spawning: true,
            player_proximity_priority: true,
            performance_limits: PerformanceLimits::default(),
        }
    }
}

impl Default for PerformanceLimits {
    fn default() -> Self {
        Self {
            max_spawns_per_frame: 5,
            max_computation_time_ms: 2.0,
            max_memory_usage_mb: 50.0,
        }
    }
}

impl Default for SpawnCooldown {
    fn default() -> Self {
        Self {
            duration: 30.0,
            cooldown_type: CooldownType::Fixed,
            randomize: true,
            randomize_range: 0.2,
        }
    }
}

impl SpawnRarity {
    /// 获取稀有度的基础权重
    pub fn base_weight(&self) -> f32 {
        match self {
            SpawnRarity::Common => 100.0,
            SpawnRarity::Uncommon => 30.0,
            SpawnRarity::Rare => 10.0,
            SpawnRarity::VeryRare => 3.0,
            SpawnRarity::Legendary => 1.0,
            SpawnRarity::Mythical => 0.1,
        }
    }
}

impl SpawnBounds {
    /// 检查位置是否在边界内
    pub fn contains(&self, position: Vec2) -> bool {
        match self {
            SpawnBounds::Circle { center, radius } => {
                center.distance(position) <= *radius
            },
            SpawnBounds::Rectangle { min, max } => {
                position.x >= min.x && position.x <= max.x &&
                position.y >= min.y && position.y <= max.y
            },
            SpawnBounds::Polygon { vertices } => {
                // 简化的点在多边形内检测
                // 实际实现应该使用射线投射算法
                false
            },
        }
    }
}

// 预设生成配置
impl SpawnManager {
    /// 创建草地生成点
    pub fn create_grassland_spawn_point(position: Vec2) -> SpawnPoint {
        let mut spawn_rules = SpawnRules {
            spawn_table: vec![
                SpawnEntry {
                    id: "pidgey".to_string(),
                    entry_type: SpawnEntryType::Pokemon(1), // Pidgey
                    weight: 50.0,
                    level_range: Some((2, 5)),
                    rarity: SpawnRarity::Common,
                    conditions: vec![],
                    data: SpawnData {
                        specific_data: HashMap::new(),
                        quantity_range: (1, 1),
                        position_offset: Vec2::ZERO,
                        spawn_effects: vec![],
                    },
                },
                SpawnEntry {
                    id: "rattata".to_string(),
                    entry_type: SpawnEntryType::Pokemon(19), // Rattata
                    weight: 40.0,
                    level_range: Some((2, 4)),
                    rarity: SpawnRarity::Common,
                    conditions: vec![],
                    data: SpawnData {
                        specific_data: HashMap::new(),
                        quantity_range: (1, 1),
                        position_offset: Vec2::ZERO,
                        spawn_effects: vec![],
                    },
                },
            ],
            conditions: vec![
                SpawnCondition::TerrainType(TerrainType::Grass),
            ],
            max_concurrent: 3,
            spawn_radius: 100.0,
            weight_modifiers: vec![],
        };

        SpawnPoint {
            id: Uuid::new_v4(),
            position,
            spawn_type: SpawnType::WildPokemon,
            spawn_rules,
            cooldown: SpawnCooldown::default(),
            last_spawn_time: None,
            spawn_count: 0,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    /// 创建水域生成点
    pub fn create_water_spawn_point(position: Vec2) -> SpawnPoint {
        let spawn_rules = SpawnRules {
            spawn_table: vec![
                SpawnEntry {
                    id: "magikarp".to_string(),
                    entry_type: SpawnEntryType::Pokemon(129), // Magikarp
                    weight: 70.0,
                    level_range: Some((5, 15)),
                    rarity: SpawnRarity::Common,
                    conditions: vec![],
                    data: SpawnData {
                        specific_data: HashMap::new(),
                        quantity_range: (1, 1),
                        position_offset: Vec2::ZERO,
                        spawn_effects: vec![],
                    },
                },
            ],
            conditions: vec![
                SpawnCondition::TerrainType(TerrainType::Water),
            ],
            max_concurrent: 2,
            spawn_radius: 50.0,
            weight_modifiers: vec![],
        };

        SpawnPoint {
            id: Uuid::new_v4(),
            position,
            spawn_type: SpawnType::WildPokemon,
            spawn_rules,
            cooldown: SpawnCooldown {
                duration: 45.0,
                ..Default::default()
            },
            last_spawn_time: None,
            spawn_count: 0,
            is_active: true,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_manager_creation() {
        let config = GlobalSpawnConfig::default();
        let manager = SpawnManager::new(config);
        
        assert_eq!(manager.spawn_points.len(), 0);
        assert_eq!(manager.active_regions.len(), 0);
    }

    #[test]
    fn test_spawn_point_creation() {
        let spawn_point = SpawnManager::create_grassland_spawn_point(Vec2::new(100.0, 100.0));
        
        assert_eq!(spawn_point.spawn_type, SpawnType::WildPokemon);
        assert_eq!(spawn_point.position, Vec2::new(100.0, 100.0));
        assert!(spawn_point.is_active);
        assert!(!spawn_point.spawn_rules.spawn_table.is_empty());
    }

    #[test]
    fn test_spawn_rarity_weights() {
        assert_eq!(SpawnRarity::Common.base_weight(), 100.0);
        assert_eq!(SpawnRarity::Legendary.base_weight(), 1.0);
        assert!(SpawnRarity::Mythical.base_weight() < SpawnRarity::Legendary.base_weight());
    }

    #[test]
    fn test_spawn_bounds() {
        let circle_bounds = SpawnBounds::Circle {
            center: Vec2::ZERO,
            radius: 10.0,
        };
        
        assert!(circle_bounds.contains(Vec2::new(5.0, 0.0)));
        assert!(!circle_bounds.contains(Vec2::new(15.0, 0.0)));
        
        let rect_bounds = SpawnBounds::Rectangle {
            min: Vec2::new(-10.0, -10.0),
            max: Vec2::new(10.0, 10.0),
        };
        
        assert!(rect_bounds.contains(Vec2::ZERO));
        assert!(!rect_bounds.contains(Vec2::new(15.0, 15.0)));
    }

    #[test]
    fn test_spawn_manager_operations() {
        let mut manager = SpawnManager::new(GlobalSpawnConfig::default());
        let spawn_point = SpawnManager::create_grassland_spawn_point(Vec2::new(50.0, 50.0));
        let spawn_id = spawn_point.id;
        
        manager.add_spawn_point(spawn_point);
        assert_eq!(manager.spawn_points.len(), 1);
        
        manager.set_spawn_point_active(spawn_id, false).unwrap();
        assert!(!manager.spawn_points.get(&spawn_id).unwrap().is_active);
        
        assert!(manager.remove_spawn_point(spawn_id));
        assert_eq!(manager.spawn_points.len(), 0);
    }
}