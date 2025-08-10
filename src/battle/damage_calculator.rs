// 战斗伤害计算器
// 开发心理：精确的伤害计算是公平战斗的基础，需要考虑所有影响因子
// 设计原则：数学精确性、性能优化、可扩展的修正系统

use crate::core::{GameError, Result};
use crate::pokemon::{Pokemon, PokemonType, Move, MoveCategory};
use crate::battle::{BattleEnvironment, WeatherType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};

// 伤害计算器主结构
pub struct DamageCalculator {
    type_chart: TypeEffectivenessChart,
    critical_hit_multipliers: HashMap<u8, f32>,
    weather_modifiers: HashMap<WeatherType, HashMap<PokemonType, f32>>,
    ability_modifiers: HashMap<String, DamageModifier>,
    item_modifiers: HashMap<u32, DamageModifier>,
}

// 类型相性表
#[derive(Debug, Clone)]
pub struct TypeEffectivenessChart {
    effectiveness: HashMap<(PokemonType, PokemonType), f32>,
}

// 伤害修正器
#[derive(Debug, Clone)]
pub struct DamageModifier {
    pub multiplier: f32,
    pub condition: Option<ModifierCondition>,
    pub stage: ModifierStage,
}

#[derive(Debug, Clone)]
pub enum ModifierCondition {
    WeatherIs(WeatherType),
    TypeMatches(PokemonType),
    MoveCategory(MoveCategory),
    HPBelow(f32),
    StatusIs(String),
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierStage {
    BeforeTypeEffectiveness,
    AfterTypeEffectiveness,
    Final,
}

// 伤害计算上下文
#[derive(Debug, Clone)]
pub struct DamageContext<'a> {
    pub attacker: &'a Pokemon,
    pub defender: &'a Pokemon,
    pub move_data: &'a Move,
    pub environment: &'a BattleEnvironment,
    pub critical_hit: bool,
    pub random_factor: f32,        // 0.85 - 1.0
    pub stab_bonus: bool,         // 本系技能加成
    pub multi_target: bool,       // 多目标技能
    pub weather_boost: bool,      // 天气加成
    pub ability_effects: Vec<String>,
    pub item_effects: Vec<u32>,
    pub field_effects: Vec<String>,
}

// 伤害计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageResult {
    pub base_damage: u32,
    pub final_damage: u32,
    pub is_critical: bool,
    pub type_effectiveness: f32,
    pub modifiers: Vec<AppliedModifier>,
    pub damage_range: (u32, u32),
    pub percentage: f32,           // 占目标最大HP的百分比
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedModifier {
    pub name: String,
    pub multiplier: f32,
    pub description: String,
}

impl DamageCalculator {
    pub fn new() -> Self {
        Self {
            type_chart: TypeEffectivenessChart::new(),
            critical_hit_multipliers: Self::init_critical_multipliers(),
            weather_modifiers: Self::init_weather_modifiers(),
            ability_modifiers: Self::init_ability_modifiers(),
            item_modifiers: Self::init_item_modifiers(),
        }
    }
    
    // 主要伤害计算函数
    pub fn calculate_damage(&self, context: &DamageContext) -> Result<DamageResult> {
        let mut modifiers = Vec::new();
        
        // 1. 检查技能威力
        let base_power = self.get_move_power(context)?;
        if base_power == 0 {
            return Ok(DamageResult {
                base_damage: 0,
                final_damage: 0,
                is_critical: false,
                type_effectiveness: 1.0,
                modifiers: vec![],
                damage_range: (0, 0),
                percentage: 0.0,
            });
        }
        
        // 2. 获取攻击和防御能力值
        let (attack_stat, defense_stat) = self.get_battle_stats(context)?;
        
        // 3. 计算基础伤害 (Gen 3+ 公式)
        let level_factor = (2.0 * context.attacker.level as f32 / 5.0 + 2.0) / 50.0;
        let base_damage = level_factor * base_power as f32 * attack_stat / defense_stat + 2.0;
        
        debug!("基础伤害计算: level_factor={}, power={}, attack={}, defense={}, base={}",
               level_factor, base_power, attack_stat, defense_stat, base_damage);
        
        let mut final_damage = base_damage;
        
        // 4. 应用各种修正
        // 4.1 暴击修正
        if context.critical_hit {
            let crit_multiplier = self.get_critical_multiplier(context)?;
            final_damage *= crit_multiplier;
            modifiers.push(AppliedModifier {
                name: "暴击".to_string(),
                multiplier: crit_multiplier,
                description: format!("暴击伤害 x{}", crit_multiplier),
            });
        }
        
        // 4.2 随机因子 (85%-100%)
        final_damage *= context.random_factor;
        
        // 4.3 本系加成 (STAB)
        if context.stab_bonus {
            final_damage *= 1.5;
            modifiers.push(AppliedModifier {
                name: "本系加成".to_string(),
                multiplier: 1.5,
                description: "同属性技能加成".to_string(),
            });
        }
        
        // 4.4 类型相性
        let type_effectiveness = self.calculate_type_effectiveness(context)?;
        final_damage *= type_effectiveness;
        if type_effectiveness != 1.0 {
            let effectiveness_text = match type_effectiveness {
                x if x > 1.0 => format!("效果拔群 x{}", x),
                x if x < 1.0 => format!("效果不理想 x{}", x),
                _ => "普通效果".to_string(),
            };
            modifiers.push(AppliedModifier {
                name: "属性相性".to_string(),
                multiplier: type_effectiveness,
                description: effectiveness_text,
            });
        }
        
        // 4.5 天气修正
        let weather_multiplier = self.calculate_weather_modifier(context)?;
        if weather_multiplier != 1.0 {
            final_damage *= weather_multiplier;
            modifiers.push(AppliedModifier {
                name: "天气效果".to_string(),
                multiplier: weather_multiplier,
                description: format!("天气修正 x{}", weather_multiplier),
            });
        }
        
        // 4.6 能力修正
        let ability_multiplier = self.calculate_ability_modifier(context)?;
        if ability_multiplier != 1.0 {
            final_damage *= ability_multiplier;
            modifiers.push(AppliedModifier {
                name: "特性效果".to_string(),
                multiplier: ability_multiplier,
                description: format!("特性修正 x{}", ability_multiplier),
            });
        }
        
        // 4.7 道具修正
        let item_multiplier = self.calculate_item_modifier(context)?;
        if item_multiplier != 1.0 {
            final_damage *= item_multiplier;
            modifiers.push(AppliedModifier {
                name: "道具效果".to_string(),
                multiplier: item_multiplier,
                description: format!("道具修正 x{}", item_multiplier),
            });
        }
        
        // 4.8 多目标修正
        if context.multi_target {
            final_damage *= 0.75;
            modifiers.push(AppliedModifier {
                name: "多目标".to_string(),
                multiplier: 0.75,
                description: "多目标技能伤害降低".to_string(),
            });
        }
        
        // 5. 计算伤害范围 (考虑随机因子)
        let min_damage = (base_damage * 0.85 * 
            modifiers.iter().map(|m| m.multiplier).product::<f32>()).round() as u32;
        let max_damage = (base_damage * 
            modifiers.iter().map(|m| m.multiplier).product::<f32>()).round() as u32;
        
        let final_damage_int = final_damage.round() as u32;
        
        // 6. 计算伤害百分比
        let defender_max_hp = context.defender.get_stats()?.hp as f32;
        let damage_percentage = (final_damage_int as f32 / defender_max_hp) * 100.0;
        
        Ok(DamageResult {
            base_damage: base_damage as u32,
            final_damage: final_damage_int.max(1), // 最少造成1点伤害
            is_critical: context.critical_hit,
            type_effectiveness,
            modifiers,
            damage_range: (min_damage, max_damage),
            percentage: damage_percentage,
        })
    }
    
    // 计算一击必杀成功率
    pub fn calculate_ohko_chance(&self, context: &DamageContext) -> f32 {
        let level_diff = context.attacker.level as i16 - context.defender.level as i16;
        if level_diff < 0 {
            return 0.0; // 等级低于对手时无法成功
        }
        
        let base_accuracy = context.move_data.accuracy.unwrap_or(30) as f32;
        (base_accuracy + level_diff as f32).min(100.0) / 100.0
    }
    
    // 计算固定伤害
    pub fn calculate_fixed_damage(&self, context: &DamageContext, damage: u16) -> DamageResult {
        DamageResult {
            base_damage: damage as u32,
            final_damage: damage as u32,
            is_critical: false,
            type_effectiveness: 1.0,
            modifiers: vec![AppliedModifier {
                name: "固定伤害".to_string(),
                multiplier: 1.0,
                description: "技能造成固定伤害".to_string(),
            }],
            damage_range: (damage as u32, damage as u32),
            percentage: if let Ok(stats) = context.defender.get_stats() {
                (damage as f32 / stats.hp as f32) * 100.0
            } else {
                0.0
            },
        }
    }
    
    // 私有辅助方法
    fn get_move_power(&self, context: &DamageContext) -> Result<u16> {
        match context.move_data.power {
            Some(power) => Ok(power),
            None => Ok(0), // 非伤害技能
        }
    }
    
    fn get_battle_stats(&self, context: &DamageContext) -> Result<(f32, f32)> {
        let attacker_stats = context.attacker.get_stats()?;
        let defender_stats = context.defender.get_stats()?;
        
        let (attack_stat, defense_stat) = match context.move_data.category {
            MoveCategory::Physical => (
                attacker_stats.attack as f32,
                defender_stats.defense as f32
            ),
            MoveCategory::Special => (
                attacker_stats.special_attack as f32,
                defender_stats.special_defense as f32
            ),
            MoveCategory::Status => (0.0, 1.0), // 状态技能不造成伤害
        };
        
        Ok((attack_stat, defense_stat))
    }
    
    fn get_critical_multiplier(&self, context: &DamageContext) -> Result<f32> {
        // 根据游戏世代返回不同的暴击倍率
        // Gen 6+: 1.5倍, Gen 2-5: 2倍, Gen 1: 2倍
        Ok(1.5)
    }
    
    fn calculate_type_effectiveness(&self, context: &DamageContext) -> Result<f32> {
        let move_type = context.move_data.move_type;
        let defender_species = context.defender.get_species()?;
        
        let mut effectiveness = 1.0;
        
        for defender_type in &defender_species.types {
            let type_modifier = self.type_chart.get_effectiveness(move_type, *defender_type);
            effectiveness *= type_modifier;
        }
        
        Ok(effectiveness)
    }
    
    fn calculate_weather_modifier(&self, context: &DamageContext) -> Result<f32> {
        match context.environment.weather {
            Some(weather) => {
                if let Some(weather_mods) = self.weather_modifiers.get(&weather) {
                    if let Some(&modifier) = weather_mods.get(&context.move_data.move_type) {
                        return Ok(modifier);
                    }
                }
                Ok(1.0)
            },
            None => Ok(1.0),
        }
    }
    
    fn calculate_ability_modifier(&self, context: &DamageContext) -> Result<f32> {
        let mut multiplier = 1.0;
        
        for ability in &context.ability_effects {
            if let Some(modifier) = self.ability_modifiers.get(ability) {
                if self.check_modifier_condition(&modifier.condition, context) {
                    multiplier *= modifier.multiplier;
                }
            }
        }
        
        Ok(multiplier)
    }
    
    fn calculate_item_modifier(&self, context: &DamageContext) -> Result<f32> {
        let mut multiplier = 1.0;
        
        for &item_id in &context.item_effects {
            if let Some(modifier) = self.item_modifiers.get(&item_id) {
                if self.check_modifier_condition(&modifier.condition, context) {
                    multiplier *= modifier.multiplier;
                }
            }
        }
        
        Ok(multiplier)
    }
    
    fn check_modifier_condition(&self, condition: &Option<ModifierCondition>, context: &DamageContext) -> bool {
        match condition {
            Some(ModifierCondition::WeatherIs(weather)) => {
                context.environment.weather == Some(*weather)
            },
            Some(ModifierCondition::TypeMatches(move_type)) => {
                context.move_data.move_type == *move_type
            },
            Some(ModifierCondition::MoveCategory(category)) => {
                context.move_data.category == *category
            },
            Some(ModifierCondition::HPBelow(threshold)) => {
                let current_hp = context.attacker.current_hp as f32;
                let max_hp = context.attacker.get_stats().map_or(1.0, |s| s.hp as f32);
                (current_hp / max_hp) < *threshold
            },
            None => true,
            _ => true, // 简化实现
        }
    }
    
    // 初始化各种修正数据
    fn init_critical_multipliers() -> HashMap<u8, f32> {
        let mut multipliers = HashMap::new();
        multipliers.insert(1, 1.5);  // Gen 6+
        multipliers.insert(2, 2.0);  // Gen 2-5
        multipliers
    }
    
    fn init_weather_modifiers() -> HashMap<WeatherType, HashMap<PokemonType, f32>> {
        let mut weather_mods = HashMap::new();
        
        // 晴天
        let mut sunny_mods = HashMap::new();
        sunny_mods.insert(PokemonType::Fire, 1.5);
        sunny_mods.insert(PokemonType::Water, 0.5);
        weather_mods.insert(WeatherType::Sun, sunny_mods);
        
        // 雨天
        let mut rainy_mods = HashMap::new();
        rainy_mods.insert(PokemonType::Water, 1.5);
        rainy_mods.insert(PokemonType::Fire, 0.5);
        weather_mods.insert(WeatherType::Rain, rainy_mods);
        
        weather_mods
    }
    
    fn init_ability_modifiers() -> HashMap<String, DamageModifier> {
        let mut modifiers = HashMap::new();
        
        // 技师特性 - 威力60以下的技能威力提升50%
        modifiers.insert("technician".to_string(), DamageModifier {
            multiplier: 1.5,
            condition: None, // 这里应该检查技能威力
            stage: ModifierStage::BeforeTypeEffectiveness,
        });
        
        // 适应力特性 - 本系加成从1.5倍提升到2倍
        modifiers.insert("adaptability".to_string(), DamageModifier {
            multiplier: 2.0 / 1.5, // 补充到2倍
            condition: None,
            stage: ModifierStage::AfterTypeEffectiveness,
        });
        
        modifiers
    }
    
    fn init_item_modifiers() -> HashMap<u32, DamageModifier> {
        let mut modifiers = HashMap::new();
        
        // 生命宝珠 - 技能威力提升30%，但自己受伤
        modifiers.insert(201, DamageModifier {
            multiplier: 1.3,
            condition: None,
            stage: ModifierStage::Final,
        });
        
        // 专爱眼镜 - 特攻技能威力提升50%
        modifiers.insert(202, DamageModifier {
            multiplier: 1.5,
            condition: Some(ModifierCondition::MoveCategory(MoveCategory::Special)),
            stage: ModifierStage::Final,
        });
        
        modifiers
    }
}

impl TypeEffectivenessChart {
    pub fn new() -> Self {
        let mut chart = Self {
            effectiveness: HashMap::new(),
        };
        chart.load_type_chart();
        chart
    }
    
    pub fn get_effectiveness(&self, attacking_type: PokemonType, defending_type: PokemonType) -> f32 {
        self.effectiveness.get(&(attacking_type, defending_type))
            .copied()
            .unwrap_or(1.0)
    }
    
    fn load_type_chart(&mut self) {
        use PokemonType::*;
        
        // 格斗系
        self.add_effectiveness(Fighting, Normal, 2.0);
        self.add_effectiveness(Fighting, Ice, 2.0);
        self.add_effectiveness(Fighting, Rock, 2.0);
        self.add_effectiveness(Fighting, Steel, 2.0);
        self.add_effectiveness(Fighting, Flying, 0.5);
        self.add_effectiveness(Fighting, Poison, 0.5);
        self.add_effectiveness(Fighting, Psychic, 0.5);
        self.add_effectiveness(Fighting, Bug, 0.5);
        self.add_effectiveness(Fighting, Fairy, 0.5);
        self.add_effectiveness(Fighting, Ghost, 0.0);
        
        // 飞行系
        self.add_effectiveness(Flying, Electric, 0.5);
        self.add_effectiveness(Flying, Ice, 0.5);
        self.add_effectiveness(Flying, Rock, 0.5);
        self.add_effectiveness(Flying, Fighting, 2.0);
        self.add_effectiveness(Flying, Ground, 2.0);
        self.add_effectiveness(Flying, Bug, 2.0);
        self.add_effectiveness(Flying, Grass, 2.0);
        
        // 毒系
        self.add_effectiveness(Poison, Fighting, 0.5);
        self.add_effectiveness(Poison, Poison, 0.5);
        self.add_effectiveness(Poison, Bug, 0.5);
        self.add_effectiveness(Poison, Grass, 2.0);
        self.add_effectiveness(Poison, Fairy, 2.0);
        self.add_effectiveness(Poison, Rock, 0.5);
        self.add_effectiveness(Poison, Ghost, 0.5);
        self.add_effectiveness(Poison, Ground, 0.5);
        self.add_effectiveness(Poison, Steel, 0.0);
        
        // 地面系
        self.add_effectiveness(Ground, Fire, 2.0);
        self.add_effectiveness(Ground, Electric, 2.0);
        self.add_effectiveness(Ground, Poison, 2.0);
        self.add_effectiveness(Ground, Rock, 2.0);
        self.add_effectiveness(Ground, Steel, 2.0);
        self.add_effectiveness(Ground, Flying, 0.0);
        self.add_effectiveness(Ground, Bug, 0.5);
        self.add_effectiveness(Ground, Grass, 0.5);
        
        // 岩石系
        self.add_effectiveness(Rock, Fire, 2.0);
        self.add_effectiveness(Rock, Ice, 2.0);
        self.add_effectiveness(Rock, Flying, 2.0);
        self.add_effectiveness(Rock, Bug, 2.0);
        self.add_effectiveness(Rock, Fighting, 0.5);
        self.add_effectiveness(Rock, Ground, 0.5);
        self.add_effectiveness(Rock, Steel, 0.5);
        
        // 虫系
        self.add_effectiveness(Bug, Grass, 2.0);
        self.add_effectiveness(Bug, Psychic, 2.0);
        self.add_effectiveness(Bug, Dark, 2.0);
        self.add_effectiveness(Bug, Fire, 0.5);
        self.add_effectiveness(Bug, Fighting, 0.5);
        self.add_effectiveness(Bug, Poison, 0.5);
        self.add_effectiveness(Bug, Flying, 0.5);
        self.add_effectiveness(Bug, Ghost, 0.5);
        self.add_effectiveness(Bug, Steel, 0.5);
        self.add_effectiveness(Bug, Fairy, 0.5);
        
        // 幽灵系
        self.add_effectiveness(Ghost, Ghost, 2.0);
        self.add_effectiveness(Ghost, Psychic, 2.0);
        self.add_effectiveness(Ghost, Normal, 0.0);
        self.add_effectiveness(Ghost, Dark, 0.5);
        
        // 钢系
        self.add_effectiveness(Steel, Ice, 2.0);
        self.add_effectiveness(Steel, Rock, 2.0);
        self.add_effectiveness(Steel, Fairy, 2.0);
        self.add_effectiveness(Steel, Steel, 0.5);
        self.add_effectiveness(Steel, Fire, 0.5);
        self.add_effectiveness(Steel, Water, 0.5);
        self.add_effectiveness(Steel, Electric, 0.5);
        
        // 火系
        self.add_effectiveness(Fire, Bug, 2.0);
        self.add_effectiveness(Fire, Steel, 2.0);
        self.add_effectiveness(Fire, Grass, 2.0);
        self.add_effectiveness(Fire, Ice, 2.0);
        self.add_effectiveness(Fire, Rock, 0.5);
        self.add_effectiveness(Fire, Fire, 0.5);
        self.add_effectiveness(Fire, Water, 0.5);
        self.add_effectiveness(Fire, Dragon, 0.5);
        
        // 水系
        self.add_effectiveness(Water, Ground, 2.0);
        self.add_effectiveness(Water, Rock, 2.0);
        self.add_effectiveness(Water, Fire, 2.0);
        self.add_effectiveness(Water, Water, 0.5);
        self.add_effectiveness(Water, Grass, 0.5);
        self.add_effectiveness(Water, Dragon, 0.5);
        
        // 草系
        self.add_effectiveness(Grass, Ground, 2.0);
        self.add_effectiveness(Grass, Rock, 2.0);
        self.add_effectiveness(Grass, Water, 2.0);
        self.add_effectiveness(Grass, Flying, 0.5);
        self.add_effectiveness(Grass, Poison, 0.5);
        self.add_effectiveness(Grass, Bug, 0.5);
        self.add_effectiveness(Grass, Steel, 0.5);
        self.add_effectiveness(Grass, Fire, 0.5);
        self.add_effectiveness(Grass, Grass, 0.5);
        self.add_effectiveness(Grass, Dragon, 0.5);
        
        // 电系
        self.add_effectiveness(Electric, Flying, 2.0);
        self.add_effectiveness(Electric, Water, 2.0);
        self.add_effectiveness(Electric, Grass, 0.5);
        self.add_effectiveness(Electric, Electric, 0.5);
        self.add_effectiveness(Electric, Dragon, 0.5);
        self.add_effectiveness(Electric, Ground, 0.0);
        
        // 超能力系
        self.add_effectiveness(Psychic, Fighting, 2.0);
        self.add_effectiveness(Psychic, Poison, 2.0);
        self.add_effectiveness(Psychic, Steel, 0.5);
        self.add_effectiveness(Psychic, Psychic, 0.5);
        self.add_effectiveness(Psychic, Dark, 0.0);
        
        // 冰系
        self.add_effectiveness(Ice, Flying, 2.0);
        self.add_effectiveness(Ice, Ground, 2.0);
        self.add_effectiveness(Ice, Grass, 2.0);
        self.add_effectiveness(Ice, Dragon, 2.0);
        self.add_effectiveness(Ice, Steel, 0.5);
        self.add_effectiveness(Ice, Fire, 0.5);
        self.add_effectiveness(Ice, Water, 0.5);
        self.add_effectiveness(Ice, Ice, 0.5);
        
        // 龙系
        self.add_effectiveness(Dragon, Dragon, 2.0);
        self.add_effectiveness(Dragon, Steel, 0.5);
        self.add_effectiveness(Dragon, Fairy, 0.0);
        
        // 恶系
        self.add_effectiveness(Dark, Fighting, 0.5);
        self.add_effectiveness(Dark, Ghost, 2.0);
        self.add_effectiveness(Dark, Psychic, 2.0);
        self.add_effectiveness(Dark, Dark, 0.5);
        self.add_effectiveness(Dark, Fairy, 0.5);
        
        // 妖精系
        self.add_effectiveness(Fairy, Fire, 0.5);
        self.add_effectiveness(Fairy, Poison, 0.5);
        self.add_effectiveness(Fairy, Steel, 0.5);
        self.add_effectiveness(Fairy, Fighting, 2.0);
        self.add_effectiveness(Fairy, Dragon, 2.0);
        self.add_effectiveness(Fairy, Dark, 2.0);
    }
    
    fn add_effectiveness(&mut self, attacking: PokemonType, defending: PokemonType, multiplier: f32) {
        self.effectiveness.insert((attacking, defending), multiplier);
    }
}

// 辅助函数：创建伤害计算上下文
pub fn create_damage_context<'a>(
    attacker: &'a Pokemon,
    defender: &'a Pokemon,
    move_data: &'a Move,
    environment: &'a BattleEnvironment,
    critical_hit: bool,
) -> DamageContext<'a> {
    // 检查本系加成
    let attacker_species = attacker.get_species().ok();
    let stab_bonus = attacker_species.map_or(false, |species| {
        species.types.contains(&move_data.move_type)
    });
    
    // 生成随机因子
    let random_factor = fastrand::f32() * 0.15 + 0.85; // 0.85 - 1.0
    
    DamageContext {
        attacker,
        defender,
        move_data,
        environment,
        critical_hit,
        random_factor,
        stab_bonus,
        multi_target: false,
        weather_boost: false,
        ability_effects: vec![],
        item_effects: vec![],
        field_effects: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{Pokemon, PokemonType, Gender, Nature};
    
    #[test]
    fn test_type_effectiveness() {
        let chart = TypeEffectivenessChart::new();
        
        // 火克草
        assert_eq!(chart.get_effectiveness(PokemonType::Fire, PokemonType::Grass), 2.0);
        
        // 水克火
        assert_eq!(chart.get_effectiveness(PokemonType::Water, PokemonType::Fire), 2.0);
        
        // 草克水
        assert_eq!(chart.get_effectiveness(PokemonType::Grass, PokemonType::Water), 2.0);
        
        // 一般对幽灵无效
        assert_eq!(chart.get_effectiveness(PokemonType::Normal, PokemonType::Ghost), 0.0);
    }
    
    #[test]
    fn test_damage_calculator_creation() {
        let calculator = DamageCalculator::new();
        assert!(!calculator.type_chart.effectiveness.is_empty());
    }
    
    // 注意：完整的伤害计算测试需要创建完整的Pokemon和Move实例
    // 这里只是基本的结构测试
}