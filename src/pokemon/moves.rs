// 宝可梦技能系统
// 开发心理：技能是战斗系统的核心，需要完整的数据模型和效果系统
// 设计原则：数据驱动、可扩展的效果系统、支持自定义技能

use crate::core::{GameError, Result};
use crate::pokemon::{PokemonType, SpeciesId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lazy_static::lazy_static;
use log::{debug, info};

pub type MoveId = u16;

// 技能基础数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub id: MoveId,
    pub name: String,
    pub description: String,
    pub move_type: PokemonType,
    pub category: MoveCategory,
    pub power: Option<u16>,        // None表示非伤害技能
    pub accuracy: Option<u8>,      // None表示必中
    pub pp: u8,
    pub priority: i8,
    pub target: MoveTarget,
    pub contact: bool,             // 是否接触类技能
    pub sound: bool,               // 是否声音类技能
    pub bullet: bool,              // 是否子弹类技能
    pub bite: bool,                // 是否咬类技能
    pub punch: bool,               // 是否拳类技能
    pub dance: bool,               // 是否舞蹈类技能
    pub wind: bool,                // 是否风类技能
    pub heal: bool,                // 是否回复类技能
    pub substitute_bypass: bool,   // 是否无视替身
    pub protect_bypass: bool,      // 是否无视守护
    pub mirror_move_bypass: bool,  // 是否无视鹦鹉学舌
    pub king_rock_affected: bool,  // 是否受王者之证影响
    pub high_crit: bool,           // 是否高爆击率
    pub effects: Vec<MoveEffect>,
    pub secondary_effects: Vec<SecondaryEffect>,
    pub flavor_text: String,
    pub introduced_generation: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,   // 物理攻击
    Special,    // 特殊攻击
    Status,     // 变化技能
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveTarget {
    SingleTarget,           // 单体目标
    SingleOpponent,         // 单个对手
    AllOpponents,          // 所有对手
    AllAllies,             // 所有同伴
    AllPokemon,            // 所有宝可梦
    User,                  // 使用者自己
    UserAndAllies,         // 使用者和同伴
    OpponentField,         // 对手场地
    UserField,             // 己方场地
    EntireField,           // 整个场地
    RandomOpponent,        // 随机对手
    Adjacent,              // 相邻宝可梦
}

// 技能效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveEffect {
    // 基础效果
    Damage {
        formula: DamageFormula,
        type_effectiveness: bool,
    },
    Heal {
        formula: HealFormula,
        target: EffectTarget,
    },
    StatusChange {
        target: EffectTarget,
        status: StatusEffect,
        chance: f32,
    },
    StatChange {
        target: EffectTarget,
        stat: StatType,
        stages: i8,
        chance: f32,
    },
    
    // 特殊效果
    SwitchOut { force: bool },
    Flinch { chance: f32 },
    Confusion { chance: f32 },
    Trap { turns: u8 },
    Recoil { damage_ratio: f32 },
    Drain { drain_ratio: f32 },
    Weather { weather: WeatherType, turns: u8 },
    FieldEffect { effect: FieldEffectType, turns: u8 },
    TypeChange { new_type: PokemonType },
    AbilityChange { new_ability: u16 },
    ItemRemove,
    ItemGive { item_id: u32 },
    
    // 复合效果
    MultiHit { min_hits: u8, max_hits: u8 },
    TwoTurnMove { charge_turn: String },
    OHKO,                    // 一击必杀
    FixedDamage { damage: u16 },
    LevelDamage,             // 等级伤害
    WeightDamage,            // 重量伤害
    RandomDamage { min: u16, max: u16 },
    
    // 条件效果
    ConditionalEffect {
        condition: MoveCondition,
        effect: Box<MoveEffect>,
    },
    Custom { effect_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub effect: MoveEffect,
    pub chance: f32,
    pub condition: Option<MoveCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageFormula {
    Standard,               // 标准伤害公式
    Fixed(u16),            // 固定伤害
    Level,                 // 等级伤害
    Psywave,              // 精神波动
    NightShade,           // 黑夜魔影
    DragonRage,           // 龙之怒
    SonicBoom,            // 音爆
    Counter,              // 双倍奉还
    MirrorCoat,           // 镜面反射
    MetalBurst,           // 金属爆炸
    Custom(String),       // 自定义公式
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealFormula {
    Fixed(u16),           // 固定回复
    Percentage(f32),      // 百分比回复
    WeatherBased,         // 基于天气
    DamageBased(f32),     // 基于造成伤害
    Custom(String),       // 自定义公式
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectTarget {
    User,
    Target,
    AllOpponents,
    AllAllies,
    All,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffect {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned,
    Sleep,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatType {
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Sun,
    Rain,
    Sandstorm,
    Hail,
    Fog,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldEffectType {
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
    Reflect,
    LightScreen,
    Mist,
    Safeguard,
    TailWind,
    TrickRoom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveCondition {
    UserHPBelow(f32),        // 用户HP低于百分比
    UserHPAbove(f32),        // 用户HP高于百分比
    TargetHPBelow(f32),      // 目标HP低于百分比
    WeatherIs(WeatherType),  // 特定天气
    HasStatus(StatusEffect), // 有特定状态
    FirstTurn,               // 首回合
    ConsecutiveUse(u8),      // 连续使用
    TypeIs(PokemonType),     // 特定属性
    AbilityIs(u16),         // 特定能力
    ItemIs(u32),            // 特定道具
    Custom(String),         // 自定义条件
}

// 技能学习方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnMethod {
    LevelUp,                 // 升级学会
    TM(u8),                 // 招式学习器
    TR(u8),                 // 招式记录
    Tutor,                  // 技能导师
    Egg,                    // 蛋技能
    Event,                  // 活动技能
    Transfer,               // 传授技能
    Sketch,                 // 写生
    Mimic,                  // 模仿
    Custom(String),         // 自定义方式
}

// 可学习技能数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnableMove {
    pub move_id: MoveId,
    pub learn_method: LearnMethod,
    pub level: Option<u8>,     // 升级学会时的等级
    pub machine_id: Option<u8>, // TM/TR编号
    pub generation: Option<u8>, // 首次可学会的世代
}

// 技能威力计算器
pub struct MovePowerCalculator;

impl MovePowerCalculator {
    // 计算技能基础威力
    pub fn calculate_base_power(move_data: &Move, context: &BattleContext) -> u16 {
        match move_data.power {
            Some(power) => {
                // 根据特殊条件调整威力
                let mut final_power = power;
                
                // 检查技能效果中的威力修正
                for effect in &move_data.effects {
                    final_power = Self::apply_power_modifier(final_power, effect, context);
                }
                
                final_power
            },
            None => 0, // 非伤害技能
        }
    }
    
    fn apply_power_modifier(power: u16, effect: &MoveEffect, context: &BattleContext) -> u16 {
        match effect {
            MoveEffect::Damage { formula, .. } => {
                match formula {
                    DamageFormula::Fixed(dmg) => *dmg,
                    DamageFormula::Level => context.user_level as u16,
                    _ => power,
                }
            },
            MoveEffect::ConditionalEffect { condition, effect } => {
                if Self::check_condition(condition, context) {
                    Self::apply_power_modifier(power, effect, context)
                } else {
                    power
                }
            },
            _ => power,
        }
    }
    
    fn check_condition(condition: &MoveCondition, context: &BattleContext) -> bool {
        match condition {
            MoveCondition::UserHPBelow(threshold) => {
                context.user_hp_ratio < *threshold
            },
            MoveCondition::WeatherIs(weather) => {
                context.weather == Some(*weather)
            },
            MoveCondition::FirstTurn => {
                context.turn_count == 1
            },
            _ => true, // 简化实现
        }
    }
}

// 战斗上下文（用于威力计算）
#[derive(Debug)]
pub struct BattleContext {
    pub user_level: u8,
    pub user_hp_ratio: f32,
    pub target_hp_ratio: f32,
    pub weather: Option<WeatherType>,
    pub turn_count: u32,
    pub consecutive_uses: u8,
}

// 全局技能数据库
lazy_static! {
    static ref MOVE_DATABASE: HashMap<MoveId, Move> = {
        let mut db = HashMap::new();
        load_basic_moves(&mut db);
        debug!("技能数据库初始化完成，共加载了{}个技能", db.len());
        db
    };
}

// 根据技能ID获取技能数据
pub fn get_move(move_id: MoveId) -> Option<&'static Move> {
    MOVE_DATABASE.get(&move_id)
}

// 获取所有技能数据
pub fn get_all_moves() -> &'static HashMap<MoveId, Move> {
    &MOVE_DATABASE
}

// 根据名称查找技能
pub fn get_move_by_name(name: &str) -> Option<&'static Move> {
    MOVE_DATABASE.values()
        .find(|move_data| move_data.name.eq_ignore_ascii_case(name))
}

// 获取特定属性的所有技能
pub fn get_moves_by_type(move_type: PokemonType) -> Vec<&'static Move> {
    MOVE_DATABASE.values()
        .filter(|move_data| move_data.move_type == move_type)
        .collect()
}

// 获取特定类别的所有技能
pub fn get_moves_by_category(category: MoveCategory) -> Vec<&'static Move> {
    MOVE_DATABASE.values()
        .filter(|move_data| move_data.category == category)
        .collect()
}

impl Move {
    // 静态方法：根据ID获取技能
    pub fn get(move_id: MoveId) -> Option<&'static Self> {
        get_move(move_id)
    }
    
    // 检查技能是否命中
    pub fn check_accuracy(&self, context: &BattleContext) -> bool {
        match self.accuracy {
            Some(acc) => {
                let random = fastrand::u8(1..=100);
                random <= acc
            },
            None => true, // 必中技能
        }
    }
    
    // 获取技能的实际威力
    pub fn get_effective_power(&self, context: &BattleContext) -> u16 {
        MovePowerCalculator::calculate_base_power(self, context)
    }
    
    // 检查技能是否有特定效果
    pub fn has_effect(&self, effect_type: &str) -> bool {
        self.effects.iter().any(|effect| {
            match effect {
                MoveEffect::StatusChange { .. } if effect_type == "status" => true,
                MoveEffect::StatChange { .. } if effect_type == "stat" => true,
                MoveEffect::Heal { .. } if effect_type == "heal" => true,
                _ => false,
            }
        })
    }
    
    // 获取技能的所有次要效果
    pub fn get_secondary_effects(&self) -> &Vec<SecondaryEffect> {
        &self.secondary_effects
    }
    
    // 检查技能是否为接触类
    pub fn makes_contact(&self) -> bool {
        self.contact
    }
    
    // 检查技能是否受特定能力影响
    pub fn affected_by_ability(&self, ability_name: &str) -> bool {
        match ability_name {
            "soundproof" => self.sound,
            "bulletproof" => self.bullet,
            "iron_fist" => self.punch,
            "strong_jaw" => self.bite,
            "dancer" => self.dance,
            "wind_rider" => self.wind,
            _ => false,
        }
    }
}

// 加载基础技能数据
fn load_basic_moves(db: &mut HashMap<MoveId, Move>) {
    // 撞击 - 基础物理攻击
    db.insert(1, Move {
        id: 1,
        name: "撞击".to_string(),
        description: "用整个身体撞击对手进行攻击。".to_string(),
        move_type: PokemonType::Normal,
        category: MoveCategory::Physical,
        power: Some(40),
        accuracy: Some(100),
        pp: 35,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: true,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![],
        flavor_text: "最基础的攻击技能。".to_string(),
        introduced_generation: 1,
    });
    
    // 叫声 - 降低攻击力
    db.insert(2, Move {
        id: 2,
        name: "叫声".to_string(),
        description: "可爱的叫声，使对手疏忽大意，降低对手的攻击力。".to_string(),
        move_type: PokemonType::Normal,
        category: MoveCategory::Status,
        power: None,
        accuracy: Some(100),
        pp: 40,
        priority: 0,
        target: MoveTarget::AllOpponents,
        contact: false,
        sound: true,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: false,
        high_crit: false,
        effects: vec![
            MoveEffect::StatChange {
                target: EffectTarget::AllOpponents,
                stat: StatType::Attack,
                stages: -1,
                chance: 1.0,
            }
        ],
        secondary_effects: vec![],
        flavor_text: "用叫声降低对手的攻击力。".to_string(),
        introduced_generation: 1,
    });
    
    // 藤鞭 - 草系攻击
    db.insert(3, Move {
        id: 3,
        name: "藤鞭".to_string(),
        description: "用藤鞭抽打对手进行攻击。".to_string(),
        move_type: PokemonType::Grass,
        category: MoveCategory::Physical,
        power: Some(45),
        accuracy: Some(100),
        pp: 25,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: true,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![],
        flavor_text: "用长长的藤鞭抽打对手。".to_string(),
        introduced_generation: 1,
    });
    
    // 火花 - 火系攻击
    db.insert(52, Move {
        id: 52,
        name: "火花".to_string(),
        description: "向对手攻击火焰。有时会让对手陷入灼伤状态。".to_string(),
        move_type: PokemonType::Fire,
        category: MoveCategory::Special,
        power: Some(40),
        accuracy: Some(100),
        pp: 25,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: false,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![
            SecondaryEffect {
                effect: MoveEffect::StatusChange {
                    target: EffectTarget::Target,
                    status: StatusEffect::Burn,
                    chance: 1.0,
                },
                chance: 0.1, // 10%几率
                condition: None,
            }
        ],
        flavor_text: "小火焰攻击，可能造成灼伤。".to_string(),
        introduced_generation: 1,
    });
    
    // 水枪 - 水系攻击
    db.insert(55, Move {
        id: 55,
        name: "水枪".to_string(),
        description: "向对手喷射水流进行攻击。".to_string(),
        move_type: PokemonType::Water,
        category: MoveCategory::Special,
        power: Some(40),
        accuracy: Some(100),
        pp: 25,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: false,
        sound: false,
        bullet: true,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![],
        flavor_text: "喷射水流攻击对手。".to_string(),
        introduced_generation: 1,
    });
    
    // 电击 - 电系攻击
    db.insert(84, Move {
        id: 84,
        name: "电击".to_string(),
        description: "发出电击攻击对手。有时会让对手陷入麻痹状态。".to_string(),
        move_type: PokemonType::Electric,
        category: MoveCategory::Special,
        power: Some(40),
        accuracy: Some(100),
        pp: 30,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: false,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![
            SecondaryEffect {
                effect: MoveEffect::StatusChange {
                    target: EffectTarget::Target,
                    status: StatusEffect::Paralysis,
                    chance: 1.0,
                },
                chance: 0.1, // 10%几率
                condition: None,
            }
        ],
        flavor_text: "电击攻击，可能造成麻痹。".to_string(),
        introduced_generation: 1,
    });
    
    // 十万伏特 - 强力电系攻击
    db.insert(86, Move {
        id: 86,
        name: "十万伏特".to_string(),
        description: "向对手发出强力电击。有时会让对手陷入麻痹状态。".to_string(),
        move_type: PokemonType::Electric,
        category: MoveCategory::Special,
        power: Some(90),
        accuracy: Some(100),
        pp: 15,
        priority: 0,
        target: MoveTarget::SingleOpponent,
        contact: false,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: true,
        high_crit: false,
        effects: vec![
            MoveEffect::Damage {
                formula: DamageFormula::Standard,
                type_effectiveness: true,
            }
        ],
        secondary_effects: vec![
            SecondaryEffect {
                effect: MoveEffect::StatusChange {
                    target: EffectTarget::Target,
                    status: StatusEffect::Paralysis,
                    chance: 1.0,
                },
                chance: 0.1, // 10%几率
                condition: None,
            }
        ],
        flavor_text: "强力的电击攻击，有麻痹效果。".to_string(),
        introduced_generation: 1,
    });
    
    // 尾巴摇摆 - 降低防御力
    db.insert(39, Move {
        id: 39,
        name: "尾巴摇摆".to_string(),
        description: "可爱地摇摆尾巴，使对手疏忽大意，降低对手的防御力。".to_string(),
        move_type: PokemonType::Normal,
        category: MoveCategory::Status,
        power: None,
        accuracy: Some(100),
        pp: 30,
        priority: 0,
        target: MoveTarget::AllOpponents,
        contact: false,
        sound: false,
        bullet: false,
        bite: false,
        punch: false,
        dance: false,
        wind: false,
        heal: false,
        substitute_bypass: false,
        protect_bypass: false,
        mirror_move_bypass: false,
        king_rock_affected: false,
        high_crit: false,
        effects: vec![
            MoveEffect::StatChange {
                target: EffectTarget::AllOpponents,
                stat: StatType::Defense,
                stages: -1,
                chance: 1.0,
            }
        ],
        secondary_effects: vec![],
        flavor_text: "摇尾巴降低对手防御力。".to_string(),
        introduced_generation: 1,
    });
}

// 技能效果处理器
pub struct MoveEffectProcessor;

impl MoveEffectProcessor {
    // 应用技能效果
    pub fn apply_effect(effect: &MoveEffect, context: &BattleContext) -> Result<EffectResult> {
        match effect {
            MoveEffect::Damage { .. } => {
                // 伤害计算将在战斗引擎中处理
                Ok(EffectResult::Damage)
            },
            MoveEffect::Heal { formula, .. } => {
                let heal_amount = Self::calculate_heal_amount(formula, context);
                Ok(EffectResult::Heal(heal_amount))
            },
            MoveEffect::StatusChange { status, chance, .. } => {
                if fastrand::f32() < *chance {
                    Ok(EffectResult::StatusChange(*status))
                } else {
                    Ok(EffectResult::NoEffect)
                }
            },
            MoveEffect::StatChange { stat, stages, chance, .. } => {
                if fastrand::f32() < *chance {
                    Ok(EffectResult::StatChange(*stat, *stages))
                } else {
                    Ok(EffectResult::NoEffect)
                }
            },
            _ => Ok(EffectResult::NoEffect),
        }
    }
    
    fn calculate_heal_amount(formula: &HealFormula, context: &BattleContext) -> u16 {
        match formula {
            HealFormula::Fixed(amount) => *amount,
            HealFormula::Percentage(percent) => {
                // 这里需要知道最大HP，简化为固定值
                (100.0 * percent) as u16
            },
            _ => 50, // 默认回复量
        }
    }
}

// 技能效果结果
#[derive(Debug, Clone)]
pub enum EffectResult {
    NoEffect,
    Damage,
    Heal(u16),
    StatusChange(StatusEffect),
    StatChange(StatType, i8),
    SwitchOut,
    Flinch,
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_move_database() {
        let tackle = Move::get(1).unwrap();
        assert_eq!(tackle.name, "撞击");
        assert_eq!(tackle.move_type, PokemonType::Normal);
        assert_eq!(tackle.power, Some(40));
    }
    
    #[test]
    fn test_move_accuracy() {
        let tackle = Move::get(1).unwrap();
        let context = BattleContext {
            user_level: 50,
            user_hp_ratio: 1.0,
            target_hp_ratio: 1.0,
            weather: None,
            turn_count: 1,
            consecutive_uses: 1,
        };
        
        // 撞击技能应该是100%命中
        assert!(tackle.check_accuracy(&context));
    }
    
    #[test]
    fn test_move_effects() {
        let growl = Move::get(2).unwrap();
        assert!(growl.has_effect("stat"));
        assert!(!growl.has_effect("heal"));
    }
    
    #[test]
    fn test_secondary_effects() {
        let ember = Move::get(52).unwrap();
        assert!(!ember.secondary_effects.is_empty());
        
        let secondary = &ember.secondary_effects[0];
        assert_eq!(secondary.chance, 0.1);
    }
    
    #[test]
    fn test_move_categories() {
        let physical_moves = get_moves_by_category(MoveCategory::Physical);
        let status_moves = get_moves_by_category(MoveCategory::Status);
        
        assert!(!physical_moves.is_empty());
        assert!(!status_moves.is_empty());
    }
}