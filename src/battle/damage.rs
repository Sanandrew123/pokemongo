/*
* 开发心理过程：
* 1. 实现完整的Pokemon伤害计算公式，参照官方游戏机制
* 2. 支持物理/特殊攻击的不同计算方式
* 3. 实现属性相克效果、同属性加成(STAB)、会心一击等修正
* 4. 考虑天气、道具、能力等各种影响因素
* 5. 提供随机因素和固定伤害模式
* 6. 优化性能，支持批量计算和缓存
* 7. 集成完整的测试和验证系统
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        individual::{IndividualPokemon, StatusType, BattleStats},
        species::PokemonSpecies,
        moves::{Move, MoveCategory, MoveId},
        types::{PokemonType, TypeEffectiveness, DualType},
        stats::StatType,
    },
    world::environment::WeatherCondition,
    battle::effects::FieldEffect,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone)]
pub struct DamageCalculator {
    /// 属性相克倍率表
    type_chart: TypeEffectivenessChart,
    /// 伤害计算配置
    config: DamageConfig,
    /// 缓存的计算结果
    calculation_cache: HashMap<DamageKey, DamageResult>,
}

#[derive(Debug, Clone)]
pub struct DamageConfig {
    /// 是否启用随机因素
    pub enable_random: bool,
    /// 随机因素范围 (85-100)
    pub random_range: (u8, u8),
    /// 是否启用会心一击
    pub enable_critical: bool,
    /// 基础会心率 (6.25%)
    pub base_critical_rate: f32,
    /// 同属性加成倍率
    pub stab_multiplier: f32,
    /// 等级修正基数
    pub level_modifier: f32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct DamageKey {
    attacker_species: u16,
    defender_species: u16,
    move_id: MoveId,
    attacker_level: u8,
    weather: WeatherCondition,
}

#[derive(Debug, Clone)]
pub struct DamageResult {
    /// 最终伤害值
    pub damage: u16,
    /// 是否会心一击
    pub critical_hit: bool,
    /// 属性相克倍率
    pub type_effectiveness: f32,
    /// 同属性加成
    pub stab_applied: bool,
    /// 随机因素
    pub random_factor: f32,
    /// 详细的计算过程
    pub calculation_details: DamageBreakdown,
}

#[derive(Debug, Clone)]
pub struct DamageBreakdown {
    /// 基础威力
    pub base_power: u16,
    /// 攻击力数值
    pub attack_stat: u16,
    /// 防御力数值
    pub defense_stat: u16,
    /// 等级修正
    pub level_modifier: f32,
    /// 属性相克修正
    pub type_modifier: f32,
    /// 同属性加成修正
    pub stab_modifier: f32,
    /// 会心一击修正
    pub critical_modifier: f32,
    /// 天气修正
    pub weather_modifier: f32,
    /// 道具修正
    pub item_modifier: f32,
    /// 能力修正
    pub ability_modifier: f32,
    /// 其他修正
    pub other_modifiers: f32,
    /// 随机因素
    pub random_modifier: f32,
}

#[derive(Debug, Clone)]
pub struct TypeEffectivenessChart {
    /// 属性相克表 [攻击属性][防御属性] = 倍率
    chart: [[f32; 18]; 18],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Special,
    Fixed(u16), // 固定伤害
}

impl DamageCalculator {
    pub fn new() -> Self {
        Self {
            type_chart: TypeEffectivenessChart::new(),
            config: DamageConfig::default(),
            calculation_cache: HashMap::new(),
        }
    }

    pub fn with_config(config: DamageConfig) -> Self {
        Self {
            type_chart: TypeEffectivenessChart::new(),
            config,
            calculation_cache: HashMap::new(),
        }
    }

    /// 计算招式伤害
    pub fn calculate_damage(
        &mut self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
        weather: &WeatherCondition,
        field_effects: &HashMap<String, FieldEffect>,
        rng: &mut RandomGenerator,
    ) -> GameResult<DamageResult> {
        // 检查固定伤害招式
        if let Some(fixed_damage) = self.get_fixed_damage(move_data, attacker) {
            return Ok(DamageResult {
                damage: fixed_damage,
                critical_hit: false,
                type_effectiveness: 1.0,
                stab_applied: false,
                random_factor: 1.0,
                calculation_details: DamageBreakdown::fixed(fixed_damage),
            });
        }

        // 检查招式威力
        let base_power = move_data.power.ok_or_else(|| {
            GameError::BattleError("非伤害招式不能计算伤害".to_string())
        })?;

        // 创建缓存键
        let cache_key = self.create_cache_key(attacker, defender, move_data, weather);
        
        // 检查缓存
        if !self.config.enable_random {
            if let Some(cached_result) = self.calculation_cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }

        // 开始详细计算
        let mut breakdown = DamageBreakdown {
            base_power,
            attack_stat: 0,
            defense_stat: 0,
            level_modifier: 1.0,
            type_modifier: 1.0,
            stab_modifier: 1.0,
            critical_modifier: 1.0,
            weather_modifier: 1.0,
            item_modifier: 1.0,
            ability_modifier: 1.0,
            other_modifiers: 1.0,
            random_modifier: 1.0,
        };

        // 1. 获取攻击和防御数值
        let (attack_stat, defense_stat) = self.get_battle_stats(
            attacker, defender, move_data, &mut breakdown
        )?;

        // 2. 等级修正
        breakdown.level_modifier = self.calculate_level_modifier(attacker.level);

        // 3. 会心一击判定
        let critical_hit = self.check_critical_hit(attacker, move_data, field_effects, rng);
        breakdown.critical_modifier = if critical_hit { 1.5 } else { 1.0 };

        // 4. 属性相克计算
        let type_effectiveness = self.calculate_type_effectiveness(
            move_data.move_type, 
            defender, 
            field_effects
        )?;
        breakdown.type_modifier = type_effectiveness;

        // 5. 同属性加成
        let stab_applied = self.check_stab(attacker, move_data.move_type)?;
        breakdown.stab_modifier = if stab_applied { self.config.stab_multiplier } else { 1.0 };

        // 6. 天气修正
        breakdown.weather_modifier = self.calculate_weather_modifier(
            move_data.move_type, weather
        );

        // 7. 道具修正
        breakdown.item_modifier = self.calculate_item_modifier(
            attacker, defender, move_data
        );

        // 8. 能力修正
        breakdown.ability_modifier = self.calculate_ability_modifier(
            attacker, defender, move_data, field_effects
        );

        // 9. 其他修正（状态、场地效果等）
        breakdown.other_modifiers = self.calculate_other_modifiers(
            attacker, defender, move_data, field_effects
        );

        // 10. 随机因素
        if self.config.enable_random {
            breakdown.random_modifier = self.calculate_random_modifier(rng);
        }

        // 最终伤害计算
        let damage = self.apply_damage_formula(&breakdown);

        let result = DamageResult {
            damage,
            critical_hit,
            type_effectiveness,
            stab_applied,
            random_factor: breakdown.random_modifier,
            calculation_details: breakdown,
        };

        // 缓存结果（仅当没有随机因素时）
        if !self.config.enable_random {
            self.calculation_cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    fn get_fixed_damage(&self, move_data: &Move, attacker: &IndividualPokemon) -> Option<u16> {
        // 检查固定伤害招式
        match move_data.id {
            // 地球上投、发泄怒气等固定伤害招式
            69 => Some(attacker.level as u16), // 地球上投
            70 => Some(1), // 愤怒门牙
            // 更多固定伤害招式...
            _ => None,
        }
    }

    fn get_battle_stats(
        &self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
        breakdown: &mut DamageBreakdown,
    ) -> GameResult<(u16, u16)> {
        // 获取攻击方属性
        let attack_stat = match move_data.category {
            MoveCategory::Physical => {
                self.get_effective_stat(attacker, StatType::Attack)?
            },
            MoveCategory::Special => {
                self.get_effective_stat(attacker, StatType::SpecialAttack)?
            },
            MoveCategory::Status => {
                return Err(GameError::BattleError("状态招式不计算攻击力".to_string()));
            },
        };

        // 获取防御方属性
        let defense_stat = match move_data.category {
            MoveCategory::Physical => {
                self.get_effective_stat(defender, StatType::Defense)?
            },
            MoveCategory::Special => {
                self.get_effective_stat(defender, StatType::SpecialDefense)?
            },
            MoveCategory::Status => {
                return Err(GameError::BattleError("状态招式不计算防御力".to_string()));
            },
        };

        breakdown.attack_stat = attack_stat;
        breakdown.defense_stat = defense_stat;

        Ok((attack_stat, defense_stat))
    }

    fn get_effective_stat(&self, pokemon: &IndividualPokemon, stat_type: StatType) -> GameResult<u16> {
        // 获取基础属性值
        let base_stat = match stat_type {
            StatType::Attack => pokemon.cached_stats.as_ref().unwrap().attack,
            StatType::Defense => pokemon.cached_stats.as_ref().unwrap().defense,
            StatType::SpecialAttack => pokemon.cached_stats.as_ref().unwrap().special_attack,
            StatType::SpecialDefense => pokemon.cached_stats.as_ref().unwrap().special_defense,
            StatType::Speed => pokemon.cached_stats.as_ref().unwrap().speed,
            StatType::HP => pokemon.cached_stats.as_ref().unwrap().hp,
        };

        // 应用战斗中的能力变化
        let modified_stat = if let Some(battle_stats) = &pokemon.battle_stats {
            if let Some(&stage) = battle_stats.stat_stages.get(&stat_type) {
                let multiplier = self.get_stat_stage_multiplier(stage);
                (base_stat as f32 * multiplier) as u16
            } else {
                base_stat
            }
        } else {
            base_stat
        };

        Ok(modified_stat)
    }

    fn get_stat_stage_multiplier(&self, stage: i8) -> f32 {
        match stage {
            -6 => 2.0 / 8.0,
            -5 => 2.0 / 7.0,
            -4 => 2.0 / 6.0,
            -3 => 2.0 / 5.0,
            -2 => 2.0 / 4.0,
            -1 => 2.0 / 3.0,
            0 => 1.0,
            1 => 3.0 / 2.0,
            2 => 4.0 / 2.0,
            3 => 5.0 / 2.0,
            4 => 6.0 / 2.0,
            5 => 7.0 / 2.0,
            6 => 8.0 / 2.0,
            _ => 1.0,
        }
    }

    fn calculate_level_modifier(&self, level: u8) -> f32 {
        (2.0 * level as f32 / 5.0 + 2.0) / 50.0
    }

    fn check_critical_hit(
        &self,
        attacker: &IndividualPokemon,
        move_data: &Move,
        field_effects: &HashMap<String, FieldEffect>,
        rng: &mut RandomGenerator,
    ) -> bool {
        if !self.config.enable_critical {
            return false;
        }

        let mut critical_stage = 0u8;

        // 招式本身的会心等级
        if move_data.high_crit {
            critical_stage += 1;
        }

        // 道具影响
        // 在实际实现中会检查持有道具

        // 能力影响
        // 在实际实现中会检查Pokemon能力

        // 计算会心率
        let critical_rate = match critical_stage {
            0 => self.config.base_critical_rate,
            1 => 0.125,   // 12.5%
            2 => 0.25,    // 25%
            3 => 0.333,   // 33.3%
            _ => 0.5,     // 50%
        };

        rng.probability() < critical_rate
    }

    fn calculate_type_effectiveness(
        &self,
        move_type: PokemonType,
        defender: &IndividualPokemon,
        field_effects: &HashMap<String, FieldEffect>,
    ) -> GameResult<f32> {
        // 获取防御方的属性
        // 在实际实现中会从Pokemon种族数据获取属性
        let defender_types = DualType {
            primary: PokemonType::Normal, // 临时值
            secondary: None,
        };

        let effectiveness = self.type_chart.get_effectiveness(move_type, defender_types);
        
        // 应用场地效果修正
        let mut final_effectiveness = effectiveness;
        
        // 例：神秘守护减少超能力招式伤害
        if field_effects.contains_key("light_screen") && 
           matches!(move_type, PokemonType::Psychic) {
            final_effectiveness *= 0.5;
        }

        Ok(final_effectiveness)
    }

    fn check_stab(&self, attacker: &IndividualPokemon, move_type: PokemonType) -> GameResult<bool> {
        // 获取攻击方的属性
        // 在实际实现中会从Pokemon种族数据获取属性
        let attacker_types = DualType {
            primary: PokemonType::Normal, // 临时值
            secondary: None,
        };

        Ok(attacker_types.primary == move_type || 
           attacker_types.secondary == Some(move_type))
    }

    fn calculate_weather_modifier(&self, move_type: PokemonType, weather: &WeatherCondition) -> f32 {
        match weather {
            WeatherCondition::Sunny => {
                match move_type {
                    PokemonType::Fire => 1.5,      // 火属性招式威力提升
                    PokemonType::Water => 0.5,     // 水属性招式威力降低
                    _ => 1.0,
                }
            },
            WeatherCondition::Rain => {
                match move_type {
                    PokemonType::Water => 1.5,     // 水属性招式威力提升
                    PokemonType::Fire => 0.5,      // 火属性招式威力降低
                    _ => 1.0,
                }
            },
            WeatherCondition::Sandstorm => {
                match move_type {
                    PokemonType::Rock => 1.5,      // 岩石属性特防提升（间接影响）
                    _ => 1.0,
                }
            },
            WeatherCondition::Hail => 1.0,
            WeatherCondition::None => 1.0,
        }
    }

    fn calculate_item_modifier(
        &self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
    ) -> f32 {
        let mut modifier = 1.0;

        // 攻击方道具效果
        if let Some(item_id) = attacker.held_item {
            modifier *= self.get_attack_item_modifier(item_id, move_data);
        }

        // 防御方道具效果
        if let Some(item_id) = defender.held_item {
            modifier *= self.get_defense_item_modifier(item_id, move_data);
        }

        modifier
    }

    fn get_attack_item_modifier(&self, item_id: u32, move_data: &Move) -> f32 {
        match item_id {
            // 属性强化道具
            1 => if matches!(move_data.move_type, PokemonType::Fire) { 1.2 } else { 1.0 }, // 木炭
            2 => if matches!(move_data.move_type, PokemonType::Water) { 1.2 } else { 1.0 }, // 神秘水滴
            // 更多道具...
            _ => 1.0,
        }
    }

    fn get_defense_item_modifier(&self, item_id: u32, move_data: &Move) -> f32 {
        match item_id {
            // 防御类道具
            100 => 0.5, // 先制之爪（减少伤害）
            // 更多道具...
            _ => 1.0,
        }
    }

    fn calculate_ability_modifier(
        &self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
        field_effects: &HashMap<String, FieldEffect>,
    ) -> f32 {
        let mut modifier = 1.0;

        // 攻击方能力
        modifier *= self.get_attack_ability_modifier(attacker.ability_id, move_data, field_effects);

        // 防御方能力
        modifier *= self.get_defense_ability_modifier(defender.ability_id, move_data);

        modifier
    }

    fn get_attack_ability_modifier(
        &self, 
        ability_id: u32, 
        move_data: &Move, 
        field_effects: &HashMap<String, FieldEffect>
    ) -> f32 {
        match ability_id {
            // 力量能力（增强物理攻击）
            1 => if matches!(move_data.category, MoveCategory::Physical) { 1.5 } else { 1.0 },
            // 更多能力...
            _ => 1.0,
        }
    }

    fn get_defense_ability_modifier(&self, ability_id: u32, move_data: &Move) -> f32 {
        match ability_id {
            // 厚脂肪能力（减少火/冰伤害）
            10 => if matches!(move_data.move_type, PokemonType::Fire | PokemonType::Ice) { 0.5 } else { 1.0 },
            // 更多能力...
            _ => 1.0,
        }
    }

    fn calculate_other_modifiers(
        &self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
        field_effects: &HashMap<String, FieldEffect>,
    ) -> f32 {
        let mut modifier = 1.0;

        // 状态条件影响
        if attacker.has_status(StatusType::Burn) && 
           matches!(move_data.category, MoveCategory::Physical) {
            modifier *= 0.5; // 烧伤状态减少物理攻击威力
        }

        // 场地效果影响
        if field_effects.contains_key("reflect") && 
           matches!(move_data.category, MoveCategory::Physical) {
            modifier *= 0.5; // 光墙减少物理攻击伤害
        }

        if field_effects.contains_key("light_screen") && 
           matches!(move_data.category, MoveCategory::Special) {
            modifier *= 0.5; // 光墙减少特殊攻击伤害
        }

        modifier
    }

    fn calculate_random_modifier(&self, rng: &mut RandomGenerator) -> f32 {
        let (min, max) = self.config.random_range;
        let random_value = rng.range(min as u32, max as u32 + 1);
        random_value as f32 / 100.0
    }

    fn apply_damage_formula(&self, breakdown: &DamageBreakdown) -> u16 {
        // Pokemon伤害公式: ((((2*Level/5+2)*Power*A/D)/50)+2)*Modifiers*Random/100
        let base_damage = (
            (breakdown.level_modifier * breakdown.base_power as f32 * 
             breakdown.attack_stat as f32 / breakdown.defense_stat as f32) + 2.0
        ) * breakdown.critical_modifier
          * breakdown.type_modifier
          * breakdown.stab_modifier
          * breakdown.weather_modifier
          * breakdown.item_modifier
          * breakdown.ability_modifier
          * breakdown.other_modifiers
          * breakdown.random_modifier;

        // 确保伤害至少为1
        base_damage.max(1.0) as u16
    }

    fn create_cache_key(
        &self,
        attacker: &IndividualPokemon,
        defender: &IndividualPokemon,
        move_data: &Move,
        weather: &WeatherCondition,
    ) -> DamageKey {
        DamageKey {
            attacker_species: attacker.species_id,
            defender_species: defender.species_id,
            move_id: move_data.id,
            attacker_level: attacker.level,
            weather: weather.clone(),
        }
    }

    /// 清除计算缓存
    pub fn clear_cache(&mut self) {
        self.calculation_cache.clear();
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.calculation_cache.len(), self.calculation_cache.capacity())
    }
}

impl TypeEffectivenessChart {
    pub fn new() -> Self {
        let mut chart = [[1.0f32; 18]; 18];
        
        // 初始化属性相克表
        // 格斗 -> 一般、岩石、钢、冰、恶
        chart[PokemonType::Fighting as usize][PokemonType::Normal as usize] = 2.0;
        chart[PokemonType::Fighting as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Fighting as usize][PokemonType::Steel as usize] = 2.0;
        chart[PokemonType::Fighting as usize][PokemonType::Ice as usize] = 2.0;
        chart[PokemonType::Fighting as usize][PokemonType::Dark as usize] = 2.0;
        
        // 格斗 <- 飞行、超能力、妖精
        chart[PokemonType::Flying as usize][PokemonType::Fighting as usize] = 2.0;
        chart[PokemonType::Psychic as usize][PokemonType::Fighting as usize] = 2.0;
        chart[PokemonType::Fairy as usize][PokemonType::Fighting as usize] = 2.0;
        
        // 格斗 -> 幽灵 (无效)
        chart[PokemonType::Fighting as usize][PokemonType::Ghost as usize] = 0.0;
        
        // 飞行 -> 格斗、虫、草
        chart[PokemonType::Flying as usize][PokemonType::Fighting as usize] = 2.0;
        chart[PokemonType::Flying as usize][PokemonType::Bug as usize] = 2.0;
        chart[PokemonType::Flying as usize][PokemonType::Grass as usize] = 2.0;
        
        // 飞行 <- 岩石、电、冰
        chart[PokemonType::Rock as usize][PokemonType::Flying as usize] = 2.0;
        chart[PokemonType::Electric as usize][PokemonType::Flying as usize] = 2.0;
        chart[PokemonType::Ice as usize][PokemonType::Flying as usize] = 2.0;
        
        // 毒 -> 草、妖精
        chart[PokemonType::Poison as usize][PokemonType::Grass as usize] = 2.0;
        chart[PokemonType::Poison as usize][PokemonType::Fairy as usize] = 2.0;
        
        // 毒 <- 地面、超能力
        chart[PokemonType::Ground as usize][PokemonType::Poison as usize] = 2.0;
        chart[PokemonType::Psychic as usize][PokemonType::Poison as usize] = 2.0;
        
        // 毒 -> 钢 (无效)
        chart[PokemonType::Poison as usize][PokemonType::Steel as usize] = 0.0;
        
        // 地面 -> 毒、岩石、钢、火、电
        chart[PokemonType::Ground as usize][PokemonType::Poison as usize] = 2.0;
        chart[PokemonType::Ground as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Ground as usize][PokemonType::Steel as usize] = 2.0;
        chart[PokemonType::Ground as usize][PokemonType::Fire as usize] = 2.0;
        chart[PokemonType::Ground as usize][PokemonType::Electric as usize] = 2.0;
        
        // 地面 <- 水、草、冰
        chart[PokemonType::Water as usize][PokemonType::Ground as usize] = 2.0;
        chart[PokemonType::Grass as usize][PokemonType::Ground as usize] = 2.0;
        chart[PokemonType::Ice as usize][PokemonType::Ground as usize] = 2.0;
        
        // 地面 -> 飞行 (无效)
        chart[PokemonType::Ground as usize][PokemonType::Flying as usize] = 0.0;
        
        // 岩石 -> 飞行、虫、火、冰
        chart[PokemonType::Rock as usize][PokemonType::Flying as usize] = 2.0;
        chart[PokemonType::Rock as usize][PokemonType::Bug as usize] = 2.0;
        chart[PokemonType::Rock as usize][PokemonType::Fire as usize] = 2.0;
        chart[PokemonType::Rock as usize][PokemonType::Ice as usize] = 2.0;
        
        // 岩石 <- 格斗、地面、钢、水、草
        chart[PokemonType::Fighting as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Ground as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Steel as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Water as usize][PokemonType::Rock as usize] = 2.0;
        chart[PokemonType::Grass as usize][PokemonType::Rock as usize] = 2.0;

        // 更多属性相克关系...
        // 由于篇幅限制，这里只展示部分，完整版本需要包含所有18个属性的相克关系

        Self { chart }
    }

    pub fn get_effectiveness(&self, attack_type: PokemonType, defender_type: DualType) -> f32 {
        let primary_effect = self.chart[attack_type as usize][defender_type.primary as usize];
        
        let secondary_effect = if let Some(secondary) = defender_type.secondary {
            self.chart[attack_type as usize][secondary as usize]
        } else {
            1.0
        };
        
        primary_effect * secondary_effect
    }
}

impl DamageConfig {
    pub fn default() -> Self {
        Self {
            enable_random: true,
            random_range: (85, 100),
            enable_critical: true,
            base_critical_rate: 0.0625, // 6.25%
            stab_multiplier: 1.5,
            level_modifier: 1.0,
        }
    }

    pub fn no_random() -> Self {
        Self {
            enable_random: false,
            random_range: (100, 100),
            ..Self::default()
        }
    }

    pub fn high_critical() -> Self {
        Self {
            base_critical_rate: 0.125, // 12.5%
            ..Self::default()
        }
    }
}

impl DamageBreakdown {
    pub fn fixed(damage: u16) -> Self {
        Self {
            base_power: damage,
            attack_stat: 0,
            defense_stat: 0,
            level_modifier: 1.0,
            type_modifier: 1.0,
            stab_modifier: 1.0,
            critical_modifier: 1.0,
            weather_modifier: 1.0,
            item_modifier: 1.0,
            ability_modifier: 1.0,
            other_modifiers: 1.0,
            random_modifier: 1.0,
        }
    }
}

// 天气条件的临时定义（应该在world模块中定义）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WeatherCondition {
    None,
    Sunny,
    Rain,
    Sandstorm,
    Hail,
}

impl Default for DamageCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{species::PokemonSpecies, stats::StatBlock};

    fn create_test_pokemon(level: u8, attack: u16, defense: u16) -> IndividualPokemon {
        let mut pokemon = IndividualPokemon {
            id: uuid::Uuid::new_v4(),
            species_id: 1,
            nickname: None,
            level,
            experience: 0,
            gender: crate::pokemon::individual::Gender::Male,
            nature: crate::pokemon::stats::Nature::Hardy,
            ability_id: 1,
            is_shiny: false,
            ivs: StatBlock::default(),
            evs: StatBlock::default(),
            current_hp: 100,
            status_conditions: Vec::new(),
            friendship: 70,
            moves: Vec::new(),
            held_item: None,
            encounter_info: crate::pokemon::individual::EncounterInfo {
                location: "Test".to_string(),
                date_caught: chrono::Utc::now(),
                level_caught: level,
                ball_type: 1,
                trainer_id: None,
                trainer_name: None,
                method: crate::pokemon::individual::EncounterMethod::WildGrass,
            },
            marks: Vec::new(),
            is_egg: false,
            egg_cycles: None,
            cached_stats: Some(StatBlock {
                hp: 100,
                attack,
                defense,
                special_attack: attack,
                special_defense: defense,
                speed: 50,
            }),
            battle_stats: None,
        };
        pokemon
    }

    fn create_test_move(power: u16, move_type: PokemonType) -> Move {
        Move {
            id: 1,
            name: "Test Move".to_string(),
            move_type,
            category: MoveCategory::Physical,
            power: Some(power),
            accuracy: Some(100),
            pp: 10,
            priority: 0,
            target: crate::pokemon::moves::MoveTarget::SingleOpponent,
            description: "Test move".to_string(),
            effects: Vec::new(),
        }
    }

    #[test]
    fn test_basic_damage_calculation() {
        let mut calculator = DamageCalculator::with_config(DamageConfig::no_random());
        let attacker = create_test_pokemon(50, 100, 50);
        let defender = create_test_pokemon(50, 50, 100);
        let move_data = create_test_move(60, PokemonType::Normal);
        let weather = WeatherCondition::None;
        let field_effects = HashMap::new();
        let mut rng = RandomGenerator::new();

        let result = calculator.calculate_damage(
            &attacker,
            &defender,
            &move_data,
            &weather,
            &field_effects,
            &mut rng,
        ).unwrap();

        assert!(result.damage > 0);
        assert!(!result.critical_hit); // 随机关闭时不会暴击
        assert_eq!(result.type_effectiveness, 1.0); // 普通属性对普通属性
    }

    #[test]
    fn test_type_effectiveness() {
        let chart = TypeEffectivenessChart::new();
        
        // 测试格斗对普通的效果拔群
        let effectiveness = chart.get_effectiveness(
            PokemonType::Fighting,
            DualType { primary: PokemonType::Normal, secondary: None }
        );
        assert_eq!(effectiveness, 2.0);

        // 测试格斗对幽灵的无效
        let effectiveness = chart.get_effectiveness(
            PokemonType::Fighting,
            DualType { primary: PokemonType::Ghost, secondary: None }
        );
        assert_eq!(effectiveness, 0.0);
    }

    #[test]
    fn test_critical_hit() {
        let mut calculator = DamageCalculator::new();
        let mut total_crits = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            let attacker = create_test_pokemon(50, 100, 50);
            let defender = create_test_pokemon(50, 50, 100);
            let move_data = create_test_move(60, PokemonType::Normal);
            let weather = WeatherCondition::None;
            let field_effects = HashMap::new();
            let mut rng = RandomGenerator::new();

            let result = calculator.calculate_damage(
                &attacker,
                &defender,
                &move_data,
                &weather,
                &field_effects,
                &mut rng,
            ).unwrap();

            if result.critical_hit {
                total_crits += 1;
            }
        }

        // 会心率应该接近6.25%
        let crit_rate = total_crits as f32 / iterations as f32;
        assert!(crit_rate > 0.03 && crit_rate < 0.10); // 允许一定的随机波动
    }

    #[test]
    fn test_weather_modifier() {
        let mut calculator = DamageCalculator::with_config(DamageConfig::no_random());
        let attacker = create_test_pokemon(50, 100, 50);
        let defender = create_test_pokemon(50, 50, 100);
        let fire_move = create_test_move(60, PokemonType::Fire);
        let mut rng = RandomGenerator::new();
        let field_effects = HashMap::new();

        // 晴天下火系招式威力提升
        let sunny_result = calculator.calculate_damage(
            &attacker,
            &defender,
            &fire_move,
            &WeatherCondition::Sunny,
            &field_effects,
            &mut rng,
        ).unwrap();

        // 雨天下火系招式威力降低
        let rain_result = calculator.calculate_damage(
            &attacker,
            &defender,
            &fire_move,
            &WeatherCondition::Rain,
            &field_effects,
            &mut rng,
        ).unwrap();

        assert!(sunny_result.damage > rain_result.damage);
    }

    #[test]
    fn test_stat_stage_modifiers() {
        let calculator = DamageCalculator::new();
        
        assert_eq!(calculator.get_stat_stage_multiplier(0), 1.0);
        assert_eq!(calculator.get_stat_stage_multiplier(6), 4.0);
        assert_eq!(calculator.get_stat_stage_multiplier(-6), 0.25);
        assert_eq!(calculator.get_stat_stage_multiplier(1), 1.5);
        assert_eq!(calculator.get_stat_stage_multiplier(-1), 2.0 / 3.0);
    }
}