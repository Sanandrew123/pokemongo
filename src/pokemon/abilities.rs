// 宝可梦特性系统
// 开发心理：特性是战斗的隐性要素，需要精确触发、复杂逻辑、平衡设计
// 设计原则：事件驱动、条件判断、效果叠加、优先级管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;
use crate::pokemon::moves::MoveId;
use crate::pokemon::types::PokemonType;
use crate::battle::status_effects::{StatusEffectType, EffectTrigger};

// 特性ID
pub type AbilityId = u16;

// 特性类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    // 攻击相关
    PurePower,      // 巨力 - 攻击力2倍
    Guts,           // 毅力 - 异常状态时攻击1.5倍
    Overgrow,       // 茂盛 - HP低时草系技能威力提升
    Blaze,          // 猛火 - HP低时火系技能威力提升
    Torrent,        // 激流 - HP低时水系技能威力提升
    Swarm,          // 虫之预感 - HP低时虫系技能威力提升
    
    // 防御相关
    SturdiBody,     // 结实 - 满HP时不会被一击打倒
    WonderGuard,    // 神奇守护 - 只有效果绝佳的技能能造成伤害
    Levitate,       // 飘浮 - 免疫地面系技能
    WaterAbsorb,    // 蓄水 - 水系技能回复HP
    VoltAbsorb,     // 蓄电 - 电系技能回复HP
    FlashFire,      // 引火 - 火系技能提升火系威力
    
    // 状态相关
    NaturalCure,    // 自然回复 - 退场时治愈异常状态
    Immunity,       // 免疫 - 不会中毒
    Insomnia,       // 不眠 - 不会睡眠
    Limber,         // 柔软 - 不会麻痹
    OwnTempo,       // 我行我素 - 不会混乱
    InnerFocus,     // 精神力 - 不会畏缩
    
    // 速度相关
    Chlorophyll,    // 叶绿素 - 晴天时速度2倍
    SwiftSwim,      // 游泳高手 - 雨天时速度2倍
    SandRush,       // 拨沙 - 沙暴时速度2倍
    SlushRush,      // 拨雪 - 冰雹时速度2倍
    
    // 天气相关
    Drizzle,        // 降雨 - 出场时下雨
    Drought,        // 日照 - 出场时出太阳
    SandStream,     // 扬沙 - 出场时沙暴
    SnowWarning,    // 降雪 - 出场时冰雹
    
    // 特殊能力
    Trace,          // 复制 - 复制对手特性
    Synchronize,    // 同步 - 中异常状态时对手也中同样状态
    Pressure,       // 压迫感 - 对手技能PP消耗加倍
    Arena_Trap,     // 沙穴 - 对手无法逃跑
    MagnetPull,     // 磁力 - 钢系宝可梦无法逃跑
    ShadowTag,      // 踩影 - 对手无法逃跑(除同特性)
    
    // 能力变化
    Intimidate,     // 威吓 - 出场时降低对手攻击
    Download,       // 下载 - 根据对手能力决定提升攻击或特攻
    Adaptability,   // 适应力 - 本系技能威力由1.5倍变为2倍
    Technician,     // 技术高手 - 威力60以下技能威力1.5倍
    
    // 隐藏特性
    Protean,        // 变幻自如 - 使用技能前变成该技能属性
    Multiscale,     // 多重鳞片 - 满HP时受到的伤害减半
    Magic_Guard,    // 魔法防守 - 只受到直接攻击伤害
    Magic_Bounce,   // 魔法反射 - 反射变化技能
    
    // 自定义特性
    Custom(u16),
}

// 特性触发时机
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityTrigger {
    OnEnterBattle,      // 进入战场时
    OnExitBattle,       // 离开战场时
    OnTurnStart,        // 回合开始时
    OnTurnEnd,          // 回合结束时
    OnMoveUsed,         // 使用技能时
    OnBeforeMove,       // 技能使用前
    OnAfterMove,        // 技能使用后
    OnTakeDamage,       // 受到伤害时
    OnDealDamage,       // 造成伤害时
    OnStatusApply,      // 状态施加时
    OnStatusRemove,     // 状态移除时
    OnWeatherChange,    // 天气变化时
    OnHPChange,         // HP变化时
    OnStatChange,       // 能力变化时
    OnKnockOut,         // 被击倒时
    OnHit,              // 被命中时
    OnMiss,             // 技能失误时
    OnCritical,         // 会心一击时
    OnTypeChange,       // 属性变化时
    Passive,            // 被动效果
}

// 特性效果结果
#[derive(Debug, Clone)]
pub struct AbilityResult {
    pub success: bool,
    pub damage_multiplier: f32,     // 伤害倍率
    pub accuracy_multiplier: f32,   // 命中率倍率
    pub stat_changes: HashMap<String, i8>, // 能力变化
    pub type_changes: Vec<PokemonType>, // 属性变化
    pub status_effects: Vec<StatusEffectType>, // 添加的状态效果
    pub removed_effects: Vec<StatusEffectType>, // 移除的状态效果
    pub prevent_move: bool,         // 是否阻止技能
    pub redirect_target: Option<u32>, // 重定向目标
    pub additional_effects: Vec<String>, // 额外效果
    pub messages: Vec<String>,      // 消息文本
    pub priority_change: i8,        // 优先度变化
    pub weather_change: Option<StatusEffectType>, // 天气变化
}

// 特性数据
#[derive(Debug, Clone)]
pub struct Ability {
    pub id: AbilityId,
    pub ability_type: AbilityType,
    pub name: String,
    pub description: String,
    pub triggers: Vec<AbilityTrigger>,
    pub priority: i8,               // 触发优先级
    pub can_be_suppressed: bool,    // 是否可被压制
    pub can_be_swapped: bool,       // 是否可被交换
    pub can_be_traced: bool,        // 是否可被复制
    pub is_hidden: bool,            // 是否为隐藏特性
    pub generation: u8,             // 引入世代
    pub conditions: Vec<AbilityCondition>, // 触发条件
    pub metadata: HashMap<String, f32>, // 额外数据
}

// 特性触发条件
#[derive(Debug, Clone)]
pub struct AbilityCondition {
    pub condition_type: ConditionType,
    pub value: f32,
    pub comparison: Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionType {
    HPPercentage,       // HP百分比
    StatusCount,        // 状态数量
    TurnNumber,         // 回合数
    WeatherType,        // 天气类型
    OpponentType,       // 对手属性
    MoveType,           // 技能属性
    MovePower,          // 技能威力
    StatStage,          // 能力等级
    PartySize,          // 队伍大小
    Custom(String),     // 自定义条件
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Equal,          // 等于
    NotEqual,       // 不等于
    Greater,        // 大于
    GreaterEqual,   // 大于等于
    Less,           // 小于
    LessEqual,      // 小于等于
}

// 特性上下文
#[derive(Debug, Clone)]
pub struct AbilityContext {
    pub pokemon_id: u32,
    pub opponent_id: Option<u32>,
    pub move_id: Option<MoveId>,
    pub move_type: Option<PokemonType>,
    pub move_power: Option<u16>,
    pub damage_amount: Option<i32>,
    pub current_hp: u32,
    pub max_hp: u32,
    pub status_effects: Vec<StatusEffectType>,
    pub weather: Option<StatusEffectType>,
    pub turn_number: u32,
    pub field_effects: Vec<StatusEffectType>,
    pub stat_stages: HashMap<String, i8>,
}

// 特性管理器
pub struct AbilityManager {
    // 特性数据库
    abilities: HashMap<AbilityId, Ability>,
    
    // 特性处理器
    handlers: HashMap<AbilityType, Box<dyn AbilityHandler>>,
    
    // 活跃特性实例
    active_abilities: HashMap<u32, Vec<AbilityInstance>>, // pokemon_id -> abilities
    
    // 配置
    enable_hidden_abilities: bool,
    ability_activation_chance: f32,
    debug_mode: bool,
    
    // 统计
    total_activations: u64,
    activation_history: Vec<ActivationRecord>,
    max_history_size: usize,
}

// 特性实例
#[derive(Debug, Clone)]
pub struct AbilityInstance {
    pub ability_id: AbilityId,
    pub pokemon_id: u32,
    pub is_suppressed: bool,    // 是否被压制
    pub activation_count: u32,  // 激活次数
    pub last_activation: Option<std::time::Instant>,
    pub custom_data: HashMap<String, f32>, // 自定义数据
}

// 激活记录
#[derive(Debug, Clone)]
struct ActivationRecord {
    ability_id: AbilityId,
    pokemon_id: u32,
    trigger: AbilityTrigger,
    result: AbilityResult,
    timestamp: std::time::Instant,
}

impl AbilityManager {
    pub fn new() -> Self {
        let mut manager = Self {
            abilities: HashMap::new(),
            handlers: HashMap::new(),
            active_abilities: HashMap::new(),
            enable_hidden_abilities: true,
            ability_activation_chance: 1.0,
            debug_mode: false,
            total_activations: 0,
            activation_history: Vec::new(),
            max_history_size: 500,
        };
        
        manager.initialize_abilities();
        manager.register_handlers();
        manager
    }
    
    // 为宝可梦添加特性
    pub fn add_ability(&mut self, pokemon_id: u32, ability_id: AbilityId) -> Result<(), GameError> {
        let ability = self.abilities.get(&ability_id)
            .ok_or_else(|| GameError::Ability(format!("特性不存在: {}", ability_id)))?;
        
        if !self.enable_hidden_abilities && ability.is_hidden {
            return Err(GameError::Ability("隐藏特性未启用".to_string()));
        }
        
        let instance = AbilityInstance {
            ability_id,
            pokemon_id,
            is_suppressed: false,
            activation_count: 0,
            last_activation: None,
            custom_data: HashMap::new(),
        };
        
        self.active_abilities
            .entry(pokemon_id)
            .or_insert_with(Vec::new)
            .push(instance);
        
        debug!("为宝可梦 {} 添加特性: {} ({})", 
               pokemon_id, ability.name, ability_id);
        
        Ok(())
    }
    
    // 移除特性
    pub fn remove_ability(&mut self, pokemon_id: u32, ability_id: AbilityId) -> Result<(), GameError> {
        if let Some(abilities) = self.active_abilities.get_mut(&pokemon_id) {
            abilities.retain(|instance| instance.ability_id != ability_id);
            debug!("移除宝可梦 {} 的特性: {}", pokemon_id, ability_id);
            Ok(())
        } else {
            Err(GameError::Ability(format!("宝可梦没有特性: {}", pokemon_id)))
        }
    }
    
    // 触发特性
    pub fn trigger_abilities(
        &mut self,
        trigger: AbilityTrigger,
        context: &AbilityContext,
    ) -> Vec<AbilityResult> {
        let mut results = Vec::new();
        
        // 获取所有相关的特性实例
        let mut relevant_instances = Vec::new();
        
        for (&pokemon_id, instances) in &self.active_abilities {
            for instance in instances {
                if instance.is_suppressed {
                    continue;
                }
                
                if let Some(ability) = self.abilities.get(&instance.ability_id) {
                    if ability.triggers.contains(&trigger) {
                        relevant_instances.push((pokemon_id, instance.clone(), ability.clone()));
                    }
                }
            }
        }
        
        // 按优先级排序
        relevant_instances.sort_by_key(|(_, _, ability)| -ability.priority);
        
        // 处理每个特性
        for (pokemon_id, mut instance, ability) in relevant_instances {
            // 检查触发条件
            if !self.check_conditions(&ability.conditions, context) {
                continue;
            }
            
            // 检查激活概率
            if fastrand::f32() > self.ability_activation_chance {
                continue;
            }
            
            // 执行特性效果
            if let Some(handler) = self.handlers.get(&ability.ability_type) {
                let result = handler.execute(&ability, &instance, context);
                
                if result.success {
                    // 更新实例数据
                    instance.activation_count += 1;
                    instance.last_activation = Some(std::time::Instant::now());
                    
                    // 更新激活数据
                    if let Some(instances) = self.active_abilities.get_mut(&pokemon_id) {
                        if let Some(stored_instance) = instances.iter_mut()
                            .find(|i| i.ability_id == instance.ability_id) {
                            *stored_instance = instance.clone();
                        }
                    }
                    
                    // 记录激活
                    self.record_activation(&ability, &instance, trigger, &result);
                    
                    results.push(result);
                    
                    debug!("特性激活: {} ({})", ability.name, ability.id);
                }
            }
        }
        
        results
    }
    
    // 获取宝可梦的特性
    pub fn get_pokemon_abilities(&self, pokemon_id: u32) -> Vec<&Ability> {
        self.active_abilities
            .get(&pokemon_id)
            .map(|instances| {
                instances
                    .iter()
                    .filter_map(|instance| self.abilities.get(&instance.ability_id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    // 检查是否拥有特定特性
    pub fn has_ability(&self, pokemon_id: u32, ability_type: AbilityType) -> bool {
        self.active_abilities
            .get(&pokemon_id)
            .map(|instances| {
                instances.iter().any(|instance| {
                    !instance.is_suppressed &&
                    self.abilities.get(&instance.ability_id)
                        .map(|ability| ability.ability_type == ability_type)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }
    
    // 压制特性
    pub fn suppress_ability(&mut self, pokemon_id: u32, ability_id: Option<AbilityId>) {
        if let Some(instances) = self.active_abilities.get_mut(&pokemon_id) {
            for instance in instances {
                if ability_id.is_none() || Some(instance.ability_id) == ability_id {
                    instance.is_suppressed = true;
                }
            }
        }
        
        debug!("压制宝可梦 {} 的特性", pokemon_id);
    }
    
    // 恢复特性
    pub fn restore_ability(&mut self, pokemon_id: u32, ability_id: Option<AbilityId>) {
        if let Some(instances) = self.active_abilities.get_mut(&pokemon_id) {
            for instance in instances {
                if ability_id.is_none() || Some(instance.ability_id) == ability_id {
                    instance.is_suppressed = false;
                }
            }
        }
        
        debug!("恢复宝可梦 {} 的特性", pokemon_id);
    }
    
    // 交换特性
    pub fn swap_abilities(&mut self, pokemon1: u32, pokemon2: u32) -> Result<(), GameError> {
        let abilities1 = self.active_abilities.get(&pokemon1).cloned().unwrap_or_default();
        let abilities2 = self.active_abilities.get(&pokemon2).cloned().unwrap_or_default();
        
        // 检查是否可交换
        for instance in &abilities1 {
            if let Some(ability) = self.abilities.get(&instance.ability_id) {
                if !ability.can_be_swapped {
                    return Err(GameError::Ability("特性无法交换".to_string()));
                }
            }
        }
        
        for instance in &abilities2 {
            if let Some(ability) = self.abilities.get(&instance.ability_id) {
                if !ability.can_be_swapped {
                    return Err(GameError::Ability("特性无法交换".to_string()));
                }
            }
        }
        
        // 执行交换
        self.active_abilities.insert(pokemon1, abilities2);
        self.active_abilities.insert(pokemon2, abilities1);
        
        debug!("交换宝可梦 {} 和 {} 的特性", pokemon1, pokemon2);
        Ok(())
    }
    
    // 复制特性
    pub fn trace_ability(&mut self, tracer_id: u32, target_id: u32) -> Result<Vec<AbilityId>, GameError> {
        let target_abilities = self.active_abilities.get(&target_id).cloned().unwrap_or_default();
        let mut traced_abilities = Vec::new();
        
        for instance in target_abilities {
            if let Some(ability) = self.abilities.get(&instance.ability_id) {
                if ability.can_be_traced {
                    self.add_ability(tracer_id, ability.id)?;
                    traced_abilities.push(ability.id);
                }
            }
        }
        
        debug!("宝可梦 {} 复制了宝可梦 {} 的特性", tracer_id, target_id);
        Ok(traced_abilities)
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> AbilityStats {
        AbilityStats {
            total_abilities: self.abilities.len(),
            active_instances: self.active_abilities.values().map(|v| v.len()).sum(),
            total_activations: self.total_activations,
            history_size: self.activation_history.len(),
        }
    }
    
    // 私有方法
    fn initialize_abilities(&mut self) {
        // 初始化所有特性数据
        self.create_attack_abilities();
        self.create_defense_abilities();
        self.create_status_abilities();
        self.create_weather_abilities();
        self.create_special_abilities();
    }
    
    fn create_attack_abilities(&mut self) {
        // 巨力特性
        let pure_power = Ability {
            id: 1,
            ability_type: AbilityType::PurePower,
            name: "巨力".to_string(),
            description: "物理攻击力变为2倍。".to_string(),
            triggers: vec![AbilityTrigger::Passive],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(1, pure_power);
        
        // 毅力特性
        let guts = Ability {
            id: 2,
            ability_type: AbilityType::Guts,
            name: "毅力".to_string(),
            description: "因异常状态导致攻击下降时，攻击会提高。".to_string(),
            triggers: vec![AbilityTrigger::OnStatusApply, AbilityTrigger::Passive],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(2, guts);
        
        // 茂盛特性
        let overgrow = Ability {
            id: 3,
            ability_type: AbilityType::Overgrow,
            name: "茂盛".to_string(),
            description: "HP减少时，草属性技能威力会提高。".to_string(),
            triggers: vec![AbilityTrigger::OnMoveUsed, AbilityTrigger::Passive],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: vec![
                AbilityCondition {
                    condition_type: ConditionType::HPPercentage,
                    value: 33.0,
                    comparison: Comparison::LessEqual,
                }
            ],
            metadata: HashMap::new(),
        };
        self.abilities.insert(3, overgrow);
    }
    
    fn create_defense_abilities(&mut self) {
        // 结实特性
        let sturdy_body = Ability {
            id: 10,
            ability_type: AbilityType::SturdiBody,
            name: "结实".to_string(),
            description: "HP满时不会被一击打倒。一击必杀技能对其无效。".to_string(),
            triggers: vec![AbilityTrigger::OnTakeDamage],
            priority: 10, // 高优先级
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: vec![
                AbilityCondition {
                    condition_type: ConditionType::HPPercentage,
                    value: 100.0,
                    comparison: Comparison::Equal,
                }
            ],
            metadata: HashMap::new(),
        };
        self.abilities.insert(10, sturdy_body);
        
        // 飘浮特性
        let levitate = Ability {
            id: 11,
            ability_type: AbilityType::Levitate,
            name: "飘浮".to_string(),
            description: "从地面浮起，不会受到地面属性技能攻击。".to_string(),
            triggers: vec![AbilityTrigger::OnTakeDamage],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: false, // 不能交换
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(11, levitate);
    }
    
    fn create_status_abilities(&mut self) {
        // 自然回复特性
        let natural_cure = Ability {
            id: 20,
            ability_type: AbilityType::NaturalCure,
            name: "自然回复".to_string(),
            description: "回到同行宝可梦时，异常状态就会被治愈。".to_string(),
            triggers: vec![AbilityTrigger::OnExitBattle],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(20, natural_cure);
        
        // 免疫特性
        let immunity = Ability {
            id: 21,
            ability_type: AbilityType::Immunity,
            name: "免疫".to_string(),
            description: "因为体内拥有免疫能力，不会变成中毒状态。".to_string(),
            triggers: vec![AbilityTrigger::OnStatusApply],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(21, immunity);
    }
    
    fn create_weather_abilities(&mut self) {
        // 降雨特性
        let drizzle = Ability {
            id: 30,
            ability_type: AbilityType::Drizzle,
            name: "降雨".to_string(),
            description: "出场时，会将天气变为下雨。".to_string(),
            triggers: vec![AbilityTrigger::OnEnterBattle],
            priority: 1, // 天气优先级较高
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(30, drizzle);
        
        // 日照特性
        let drought = Ability {
            id: 31,
            ability_type: AbilityType::Drought,
            name: "日照".to_string(),
            description: "出场时，会将天气变为晴朗。".to_string(),
            triggers: vec![AbilityTrigger::OnEnterBattle],
            priority: 1,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(31, drought);
    }
    
    fn create_special_abilities(&mut self) {
        // 复制特性
        let trace = Ability {
            id: 40,
            ability_type: AbilityType::Trace,
            name: "复制".to_string(),
            description: "出场时，复制对手的特性并将其变为相同的特性。".to_string(),
            triggers: vec![AbilityTrigger::OnEnterBattle],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: false,
            can_be_traced: false, // 不能被复制
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(40, trace);
        
        // 威吓特性
        let intimidate = Ability {
            id: 41,
            ability_type: AbilityType::Intimidate,
            name: "威吓".to_string(),
            description: "出场时威吓对手，降低对手的攻击。".to_string(),
            triggers: vec![AbilityTrigger::OnEnterBattle],
            priority: 0,
            can_be_suppressed: true,
            can_be_swapped: true,
            can_be_traced: true,
            is_hidden: false,
            generation: 3,
            conditions: Vec::new(),
            metadata: HashMap::new(),
        };
        self.abilities.insert(41, intimidate);
    }
    
    fn register_handlers(&mut self) {
        // 注册特性处理器
        self.handlers.insert(AbilityType::PurePower, Box::new(PurePowerHandler));
        self.handlers.insert(AbilityType::Guts, Box::new(GutsHandler));
        self.handlers.insert(AbilityType::Overgrow, Box::new(OvergrowHandler));
        self.handlers.insert(AbilityType::SturdiBody, Box::new(SturdyBodyHandler));
        self.handlers.insert(AbilityType::Levitate, Box::new(LevitateHandler));
        self.handlers.insert(AbilityType::NaturalCure, Box::new(NaturalCureHandler));
        self.handlers.insert(AbilityType::Immunity, Box::new(ImmunityHandler));
        self.handlers.insert(AbilityType::Drizzle, Box::new(DrizzleHandler));
        self.handlers.insert(AbilityType::Drought, Box::new(DroughtHandler));
        self.handlers.insert(AbilityType::Trace, Box::new(TraceHandler));
        self.handlers.insert(AbilityType::Intimidate, Box::new(IntimidateHandler));
    }
    
    fn check_conditions(&self, conditions: &[AbilityCondition], context: &AbilityContext) -> bool {
        for condition in conditions {
            let value = match condition.condition_type {
                ConditionType::HPPercentage => {
                    (context.current_hp as f32 / context.max_hp as f32) * 100.0
                }
                ConditionType::TurnNumber => context.turn_number as f32,
                ConditionType::StatusCount => context.status_effects.len() as f32,
                ConditionType::MovePower => context.move_power.unwrap_or(0) as f32,
                _ => 0.0, // 其他条件的实现
            };
            
            let matches = match condition.comparison {
                Comparison::Equal => value == condition.value,
                Comparison::NotEqual => value != condition.value,
                Comparison::Greater => value > condition.value,
                Comparison::GreaterEqual => value >= condition.value,
                Comparison::Less => value < condition.value,
                Comparison::LessEqual => value <= condition.value,
            };
            
            if !matches {
                return false;
            }
        }
        
        true
    }
    
    fn record_activation(
        &mut self,
        ability: &Ability,
        instance: &AbilityInstance,
        trigger: AbilityTrigger,
        result: &AbilityResult,
    ) {
        let record = ActivationRecord {
            ability_id: ability.id,
            pokemon_id: instance.pokemon_id,
            trigger,
            result: result.clone(),
            timestamp: std::time::Instant::now(),
        };
        
        self.activation_history.push(record);
        self.total_activations += 1;
        
        // 限制历史大小
        if self.activation_history.len() > self.max_history_size {
            self.activation_history.remove(0);
        }
    }
}

// 特性处理器接口
pub trait AbilityHandler: Send + Sync {
    fn execute(
        &self,
        ability: &Ability,
        instance: &AbilityInstance,
        context: &AbilityContext,
    ) -> AbilityResult;
}

// 具体的特性处理器实现
struct PurePowerHandler;
impl AbilityHandler for PurePowerHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, _context: &AbilityContext) -> AbilityResult {
        AbilityResult {
            success: true,
            damage_multiplier: 2.0, // 攻击力2倍
            ..Default::default()
        }
    }
}

struct GutsHandler;
impl AbilityHandler for GutsHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let has_status = !context.status_effects.is_empty();
        AbilityResult {
            success: has_status,
            damage_multiplier: if has_status { 1.5 } else { 1.0 },
            messages: if has_status { 
                vec!["毅力特性激活！攻击力提升！".to_string()] 
            } else { 
                Vec::new() 
            },
            ..Default::default()
        }
    }
}

struct OvergrowHandler;
impl AbilityHandler for OvergrowHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let hp_percentage = (context.current_hp as f32 / context.max_hp as f32) * 100.0;
        let is_grass_move = context.move_type == Some(PokemonType::Grass);
        let low_hp = hp_percentage <= 33.0;
        
        AbilityResult {
            success: is_grass_move && low_hp,
            damage_multiplier: if is_grass_move && low_hp { 1.5 } else { 1.0 },
            messages: if is_grass_move && low_hp {
                vec!["茂盛特性激活！草系技能威力提升！".to_string()]
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }
}

struct SturdyBodyHandler;
impl AbilityHandler for SturdyBodyHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let full_hp = context.current_hp == context.max_hp;
        let would_ko = context.damage_amount.unwrap_or(0) >= context.current_hp as i32;
        
        AbilityResult {
            success: full_hp && would_ko,
            damage_multiplier: if full_hp && would_ko { 0.0 } else { 1.0 }, // 阻止击倒
            messages: if full_hp && would_ko {
                vec!["结实特性激活！撑住了攻击！".to_string()]
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }
}

struct LevitateHandler;
impl AbilityHandler for LevitateHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let is_ground_move = context.move_type == Some(PokemonType::Ground);
        
        AbilityResult {
            success: is_ground_move,
            damage_multiplier: if is_ground_move { 0.0 } else { 1.0 }, // 免疫地面系
            messages: if is_ground_move {
                vec!["飘浮特性激活！对地面系技能免疫！".to_string()]
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }
}

struct NaturalCureHandler;
impl AbilityHandler for NaturalCureHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let has_status = !context.status_effects.is_empty();
        
        AbilityResult {
            success: has_status,
            removed_effects: if has_status { context.status_effects.clone() } else { Vec::new() },
            messages: if has_status {
                vec!["自然回复特性激活！异常状态治愈了！".to_string()]
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }
}

struct ImmunityHandler;
impl AbilityHandler for ImmunityHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        let poison_status = context.status_effects.iter().any(|&effect| {
            matches!(effect, StatusEffectType::Poison | StatusEffectType::BadPoison)
        });
        
        AbilityResult {
            success: poison_status,
            removed_effects: if poison_status {
                vec![StatusEffectType::Poison, StatusEffectType::BadPoison]
            } else {
                Vec::new()
            },
            messages: if poison_status {
                vec!["免疫特性激活！不会中毒！".to_string()]
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }
}

struct DrizzleHandler;
impl AbilityHandler for DrizzleHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, _context: &AbilityContext) -> AbilityResult {
        AbilityResult {
            success: true,
            weather_change: Some(StatusEffectType::Rain),
            messages: vec!["降雨特性激活！开始下雨了！".to_string()],
            ..Default::default()
        }
    }
}

struct DroughtHandler;
impl AbilityHandler for DroughtHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, _context: &AbilityContext) -> AbilityResult {
        AbilityResult {
            success: true,
            weather_change: Some(StatusEffectType::Sunny),
            messages: vec!["日照特性激活！阳光变强了！".to_string()],
            ..Default::default()
        }
    }
}

struct TraceHandler;
impl AbilityHandler for TraceHandler {
    fn execute(&self, _ability: &Ability, instance: &AbilityInstance, context: &AbilityContext) -> AbilityResult {
        AbilityResult {
            success: context.opponent_id.is_some(),
            messages: vec!["复制特性激活！复制了对手的特性！".to_string()],
            additional_effects: vec!["trace_opponent".to_string()],
            ..Default::default()
        }
    }
}

struct IntimidateHandler;
impl AbilityHandler for IntimidateHandler {
    fn execute(&self, _ability: &Ability, _instance: &AbilityInstance, _context: &AbilityContext) -> AbilityResult {
        let mut stat_changes = HashMap::new();
        stat_changes.insert("attack".to_string(), -1); // 降低对手攻击
        
        AbilityResult {
            success: true,
            stat_changes,
            messages: vec!["威吓特性激活！降低了对手的攻击！".to_string()],
            ..Default::default()
        }
    }
}

// 默认实现
impl Default for AbilityResult {
    fn default() -> Self {
        Self {
            success: false,
            damage_multiplier: 1.0,
            accuracy_multiplier: 1.0,
            stat_changes: HashMap::new(),
            type_changes: Vec::new(),
            status_effects: Vec::new(),
            removed_effects: Vec::new(),
            prevent_move: false,
            redirect_target: None,
            additional_effects: Vec::new(),
            messages: Vec::new(),
            priority_change: 0,
            weather_change: None,
        }
    }
}

// 统计信息
#[derive(Debug, Clone)]
pub struct AbilityStats {
    pub total_abilities: usize,
    pub active_instances: usize,
    pub total_activations: u64,
    pub history_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ability_manager_creation() {
        let manager = AbilityManager::new();
        assert!(!manager.abilities.is_empty());
        assert!(!manager.handlers.is_empty());
    }
    
    #[test]
    fn test_add_ability() {
        let mut manager = AbilityManager::new();
        
        let result = manager.add_ability(1, 1); // 添加巨力特性
        assert!(result.is_ok());
        
        let abilities = manager.get_pokemon_abilities(1);
        assert_eq!(abilities.len(), 1);
        assert_eq!(abilities[0].ability_type, AbilityType::PurePower);
    }
    
    #[test]
    fn test_trigger_abilities() {
        let mut manager = AbilityManager::new();
        manager.add_ability(1, 2).unwrap(); // 添加毅力特性
        
        let context = AbilityContext {
            pokemon_id: 1,
            opponent_id: Some(2),
            move_id: None,
            move_type: None,
            move_power: None,
            damage_amount: None,
            current_hp: 100,
            max_hp: 100,
            status_effects: vec![StatusEffectType::Burn], // 有异常状态
            weather: None,
            turn_number: 1,
            field_effects: Vec::new(),
            stat_stages: HashMap::new(),
        };
        
        let results = manager.trigger_abilities(AbilityTrigger::OnStatusApply, &context);
        assert!(!results.is_empty());
        assert!(results[0].success);
    }
}