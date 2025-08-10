// 战斗状态效果系统
// 开发心理：状态效果是战斗深度的核心，需要精确计算、堆叠管理、时机判定
// 设计原则：模块化设计、优先级管理、持续时间控制、效果叠加

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;
use crate::pokemon::moves::MoveId;

// 状态效果类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectType {
    // 主要异常状态
    Burn,           // 燃烧
    Freeze,         // 冰冻
    Paralysis,      // 麻痹
    Poison,         // 中毒
    BadPoison,      // 剧毒
    Sleep,          // 睡眠
    
    // 次要状态
    Confusion,      // 混乱
    Flinch,         // 畏缩
    Infatuation,    // 迷恋
    Trap,           // 束缚
    
    // 能力变化
    AttackUp(i8),      // 攻击力提升/下降
    DefenseUp(i8),     // 防御力提升/下降
    SpAttackUp(i8),    // 特攻提升/下降
    SpDefenseUp(i8),   // 特防提升/下降
    SpeedUp(i8),       // 速度提升/下降
    AccuracyUp(i8),    // 命中率提升/下降
    EvasionUp(i8),     // 回避率提升/下降
    CriticalUp(i8),    // 会心率提升/下降
    
    // 场地效果
    Reflect,        // 光墙
    LightScreen,    // 光屏
    Safeguard,      // 神秘守护
    Mist,          // 白雾
    Spikes,        // 撒菱
    StealthRock,   // 隐形岩
    ToxicSpikes,   // 毒菱
    
    // 天气效果
    Sunny,         // 大晴天
    Rain,          // 下雨
    Sandstorm,     // 沙暴
    Hail,          // 冰雹
    
    // 特殊状态
    Substitute,    // 替身
    Encore,        // 再来一次
    Taunt,         // 挑衅
    Torment,       // 无理取闹
    Disable,       // 封印
    Heal,          // 治愈
    Leech,         // 吸取
    Curse,         // 诅咒
    
    // 自定义状态
    Custom(String),
}

// 状态效果触发时机
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectTrigger {
    OnTurnStart,        // 回合开始时
    OnTurnEnd,          // 回合结束时
    OnMoveUsed,         // 使用技能时
    OnTakeDamage,       // 受到伤害时
    OnDealDamage,       // 造成伤害时
    OnSwitch,           // 切换时
    OnStatusApply,      // 状态施加时
    OnStatusRemove,     // 状态移除时
    OnWeatherChange,    // 天气变化时
    OnFieldChange,      // 场地变化时
    OnStatChange,       // 能力变化时
    OnHeal,             // 恢复时
    OnFaint,            // 濒死时
    OnBattleStart,      // 战斗开始时
    OnBattleEnd,        // 战斗结束时
    Custom(String),     // 自定义触发
}

// 效果强度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectIntensity {
    Weak,       // 弱效果
    Normal,     // 普通效果
    Strong,     // 强效果
    Extreme,    // 极强效果
}

// 状态效果数据
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub id: u64,
    pub effect_type: StatusEffectType,
    pub intensity: EffectIntensity,
    pub duration: Option<u32>,     // 持续回合数，None为永久
    pub remaining_turns: u32,      // 剩余回合数
    pub stack_count: u32,          // 叠加层数
    pub max_stacks: u32,           // 最大叠加数
    pub source_move: Option<MoveId>, // 来源技能
    pub source_pokemon: Option<u32>, // 来源宝可梦
    pub power: f32,                // 效果强度值
    pub accuracy: f32,             // 成功率
    pub can_stack: bool,           // 是否可叠加
    pub removable: bool,           // 是否可移除
    pub blockable: bool,           // 是否可阻挡
    pub metadata: HashMap<String, f32>, // 额外数据
    pub triggers: Vec<EffectTrigger>, // 触发条件
}

// 状态效果结果
#[derive(Debug, Clone)]
pub struct EffectResult {
    pub success: bool,
    pub damage: i32,               // 造成的伤害
    pub healing: i32,              // 恢复的HP
    pub stat_changes: HashMap<String, i8>, // 能力变化
    pub new_effects: Vec<StatusEffect>, // 产生的新状态
    pub removed_effects: Vec<u64>, // 移除的状态ID
    pub messages: Vec<String>,     // 消息文本
    pub prevent_action: bool,      // 是否阻止行动
    pub forced_move: Option<MoveId>, // 强制使用的技能
    pub animation_triggers: Vec<String>, // 动画触发
}

// 状态效果管理器
pub struct StatusEffectManager {
    // 活跃效果
    active_effects: HashMap<u64, StatusEffect>,
    
    // 效果处理器
    effect_handlers: HashMap<StatusEffectType, Box<dyn EffectHandler>>,
    
    // 配置
    enable_stacking: bool,
    max_effects_per_pokemon: usize,
    debug_mode: bool,
    
    // 统计
    next_effect_id: u64,
    total_effects_applied: u64,
    
    // 效果历史
    effect_history: Vec<EffectHistoryEntry>,
    max_history_size: usize,
}

// 效果历史记录
#[derive(Debug, Clone)]
struct EffectHistoryEntry {
    effect_id: u64,
    pokemon_id: u32,
    effect_type: StatusEffectType,
    action: EffectAction,
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectAction {
    Applied,
    Triggered,
    Removed,
    Expired,
}

impl StatusEffectManager {
    pub fn new() -> Self {
        let mut manager = Self {
            active_effects: HashMap::new(),
            effect_handlers: HashMap::new(),
            enable_stacking: true,
            max_effects_per_pokemon: 20,
            debug_mode: false,
            next_effect_id: 1,
            total_effects_applied: 0,
            effect_history: Vec::new(),
            max_history_size: 1000,
        };
        
        manager.register_default_handlers();
        manager
    }
    
    // 应用状态效果
    pub fn apply_effect(
        &mut self,
        pokemon_id: u32,
        effect_type: StatusEffectType,
        duration: Option<u32>,
        source_move: Option<MoveId>,
        source_pokemon: Option<u32>,
    ) -> Result<u64, GameError> {
        // 检查是否可以应用该效果
        if !self.can_apply_effect(pokemon_id, &effect_type) {
            return Err(GameError::StatusEffect("无法应用状态效果".to_string()));
        }
        
        let effect_id = self.next_effect_id;
        self.next_effect_id += 1;
        
        let mut effect = StatusEffect {
            id: effect_id,
            effect_type: effect_type.clone(),
            intensity: EffectIntensity::Normal,
            duration,
            remaining_turns: duration.unwrap_or(0),
            stack_count: 1,
            max_stacks: self.get_max_stacks(&effect_type),
            source_move,
            source_pokemon,
            power: 1.0,
            accuracy: 1.0,
            can_stack: self.is_stackable(&effect_type),
            removable: self.is_removable(&effect_type),
            blockable: self.is_blockable(&effect_type),
            metadata: HashMap::new(),
            triggers: self.get_default_triggers(&effect_type),
        };
        
        // 检查叠加
        if let Some(existing_effect) = self.find_existing_effect(pokemon_id, &effect_type) {
            if effect.can_stack && existing_effect.stack_count < existing_effect.max_stacks {
                // 叠加效果
                let existing_id = existing_effect.id;
                let existing = self.active_effects.get_mut(&existing_id).unwrap();
                existing.stack_count += 1;
                existing.remaining_turns = existing.remaining_turns.max(effect.remaining_turns);
                
                debug!("叠加状态效果: {:?} 层数: {}/{}", 
                       effect_type, existing.stack_count, existing.max_stacks);
                
                return Ok(existing_id);
            } else {
                // 刷新效果
                let existing_id = existing_effect.id;
                let existing = self.active_effects.get_mut(&existing_id).unwrap();
                existing.remaining_turns = effect.remaining_turns;
                existing.power = existing.power.max(effect.power);
                
                debug!("刷新状态效果: {:?} 剩余回合: {}", effect_type, existing.remaining_turns);
                return Ok(existing_id);
            }
        }
        
        // 应用新效果
        self.active_effects.insert(effect_id, effect);
        self.total_effects_applied += 1;
        
        // 记录历史
        self.add_history_entry(effect_id, pokemon_id, effect_type.clone(), EffectAction::Applied);
        
        debug!("应用新状态效果: {:?} ID: {} 目标: {}", effect_type, effect_id, pokemon_id);
        Ok(effect_id)
    }
    
    // 移除状态效果
    pub fn remove_effect(&mut self, effect_id: u64) -> Result<(), GameError> {
        if let Some(effect) = self.active_effects.remove(&effect_id) {
            debug!("移除状态效果: {:?} ID: {}", effect.effect_type, effect_id);
            
            // 记录历史
            self.add_history_entry(
                effect_id,
                0, // pokemon_id 在这里不可用，使用0作为占位符
                effect.effect_type,
                EffectAction::Removed,
            );
            
            Ok(())
        } else {
            Err(GameError::StatusEffect(format!("状态效果不存在: {}", effect_id)))
        }
    }
    
    // 处理回合结束时的状态效果
    pub fn process_turn_end_effects(&mut self, pokemon_id: u32) -> Vec<EffectResult> {
        let mut results = Vec::new();
        let mut effects_to_remove = Vec::new();
        
        for (&effect_id, effect) in &mut self.active_effects {
            if effect.triggers.contains(&EffectTrigger::OnTurnEnd) {
                // 处理效果
                if let Some(handler) = self.effect_handlers.get(&effect.effect_type) {
                    let result = handler.process_effect(effect, pokemon_id);
                    results.push(result);
                }
                
                // 减少持续时间
                if effect.duration.is_some() && effect.remaining_turns > 0 {
                    effect.remaining_turns -= 1;
                    if effect.remaining_turns == 0 {
                        effects_to_remove.push(effect_id);
                    }
                }
                
                // 记录历史
                self.add_history_entry(
                    effect_id,
                    pokemon_id,
                    effect.effect_type.clone(),
                    EffectAction::Triggered,
                );
            }
        }
        
        // 移除过期效果
        for effect_id in effects_to_remove {
            self.remove_effect(effect_id).ok();
        }
        
        results
    }
    
    // 处理回合开始时的状态效果
    pub fn process_turn_start_effects(&mut self, pokemon_id: u32) -> Vec<EffectResult> {
        let mut results = Vec::new();
        
        for (&effect_id, effect) in &self.active_effects {
            if effect.triggers.contains(&EffectTrigger::OnTurnStart) {
                if let Some(handler) = self.effect_handlers.get(&effect.effect_type) {
                    let result = handler.process_effect(effect, pokemon_id);
                    results.push(result);
                }
                
                // 记录历史
                self.add_history_entry(
                    effect_id,
                    pokemon_id,
                    effect.effect_type.clone(),
                    EffectAction::Triggered,
                );
            }
        }
        
        results
    }
    
    // 处理特定触发的状态效果
    pub fn process_triggered_effects(
        &mut self,
        pokemon_id: u32,
        trigger: EffectTrigger,
        context: Option<&EffectContext>,
    ) -> Vec<EffectResult> {
        let mut results = Vec::new();
        
        for (&effect_id, effect) in &self.active_effects {
            if effect.triggers.contains(&trigger) {
                if let Some(handler) = self.effect_handlers.get(&effect.effect_type) {
                    let mut result = handler.process_effect(effect, pokemon_id);
                    
                    // 应用上下文修正
                    if let Some(ctx) = context {
                        result = handler.apply_context(result, ctx);
                    }
                    
                    results.push(result);
                }
                
                // 记录历史
                self.add_history_entry(
                    effect_id,
                    pokemon_id,
                    effect.effect_type.clone(),
                    EffectAction::Triggered,
                );
            }
        }
        
        results
    }
    
    // 获取宝可梦的所有状态效果
    pub fn get_pokemon_effects(&self, pokemon_id: u32) -> Vec<&StatusEffect> {
        // 注意：这个实现是简化的，实际需要维护pokemon_id到effect的映射
        self.active_effects.values()
            .filter(|effect| {
                // 这里需要根据实际的数据结构来过滤
                true // 临时实现
            })
            .collect()
    }
    
    // 检查是否有特定状态效果
    pub fn has_effect(&self, pokemon_id: u32, effect_type: &StatusEffectType) -> bool {
        self.active_effects.values().any(|effect| {
            effect.effect_type == *effect_type
            // && effect.target_pokemon == pokemon_id // 需要添加目标字段
        })
    }
    
    // 获取状态效果的剩余时间
    pub fn get_effect_remaining_turns(&self, effect_id: u64) -> Option<u32> {
        self.active_effects.get(&effect_id).map(|e| e.remaining_turns)
    }
    
    // 清除宝可梦的所有状态效果
    pub fn clear_pokemon_effects(&mut self, pokemon_id: u32) {
        let effect_ids: Vec<u64> = self.active_effects
            .iter()
            .filter(|(_, effect)| {
                // 这里需要检查effect是否属于该pokemon
                true // 临时实现
            })
            .map(|(&id, _)| id)
            .collect();
        
        for effect_id in effect_ids {
            self.remove_effect(effect_id).ok();
        }
        
        debug!("清除宝可梦所有状态效果: {}", pokemon_id);
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> StatusEffectStats {
        StatusEffectStats {
            active_effects: self.active_effects.len(),
            total_applied: self.total_effects_applied,
            history_entries: self.effect_history.len(),
        }
    }
    
    // 私有方法
    fn register_default_handlers(&mut self) {
        // 注册基础状态效果处理器
        self.effect_handlers.insert(
            StatusEffectType::Burn,
            Box::new(BurnHandler::new()),
        );
        self.effect_handlers.insert(
            StatusEffectType::Poison,
            Box::new(PoisonHandler::new()),
        );
        self.effect_handlers.insert(
            StatusEffectType::BadPoison,
            Box::new(BadPoisonHandler::new()),
        );
        self.effect_handlers.insert(
            StatusEffectType::Sleep,
            Box::new(SleepHandler::new()),
        );
        self.effect_handlers.insert(
            StatusEffectType::Paralysis,
            Box::new(ParalysisHandler::new()),
        );
        self.effect_handlers.insert(
            StatusEffectType::Freeze,
            Box::new(FreezeHandler::new()),
        );
        
        // 能力变化处理器
        for i in -6..=6i8 {
            if i != 0 {
                self.effect_handlers.insert(
                    StatusEffectType::AttackUp(i),
                    Box::new(StatChangeHandler::new("attack".to_string(), i)),
                );
                self.effect_handlers.insert(
                    StatusEffectType::DefenseUp(i),
                    Box::new(StatChangeHandler::new("defense".to_string(), i)),
                );
                self.effect_handlers.insert(
                    StatusEffectType::SpeedUp(i),
                    Box::new(StatChangeHandler::new("speed".to_string(), i)),
                );
            }
        }
    }
    
    fn can_apply_effect(&self, pokemon_id: u32, effect_type: &StatusEffectType) -> bool {
        // 检查是否已达到最大效果数量
        let current_effect_count = self.get_pokemon_effects(pokemon_id).len();
        if current_effect_count >= self.max_effects_per_pokemon {
            return false;
        }
        
        // 检查互斥效果
        match effect_type {
            StatusEffectType::Burn | StatusEffectType::Freeze | 
            StatusEffectType::Paralysis | StatusEffectType::Sleep => {
                // 主要异常状态互斥
                let has_major_status = self.active_effects.values().any(|e| {
                    matches!(e.effect_type, 
                        StatusEffectType::Burn | StatusEffectType::Freeze |
                        StatusEffectType::Paralysis | StatusEffectType::Sleep)
                });
                !has_major_status
            }
            _ => true,
        }
    }
    
    fn find_existing_effect(&self, pokemon_id: u32, effect_type: &StatusEffectType) -> Option<&StatusEffect> {
        self.active_effects.values().find(|effect| {
            effect.effect_type == *effect_type
            // && effect.target_pokemon == pokemon_id
        })
    }
    
    fn get_max_stacks(&self, effect_type: &StatusEffectType) -> u32 {
        match effect_type {
            StatusEffectType::AttackUp(_) | StatusEffectType::DefenseUp(_) |
            StatusEffectType::SpAttackUp(_) | StatusEffectType::SpDefenseUp(_) |
            StatusEffectType::SpeedUp(_) | StatusEffectType::AccuracyUp(_) |
            StatusEffectType::EvasionUp(_) => 6,
            StatusEffectType::Spikes => 3,
            StatusEffectType::ToxicSpikes => 2,
            _ => 1,
        }
    }
    
    fn is_stackable(&self, effect_type: &StatusEffectType) -> bool {
        matches!(effect_type,
            StatusEffectType::AttackUp(_) | StatusEffectType::DefenseUp(_) |
            StatusEffectType::SpAttackUp(_) | StatusEffectType::SpDefenseUp(_) |
            StatusEffectType::SpeedUp(_) | StatusEffectType::AccuracyUp(_) |
            StatusEffectType::EvasionUp(_) | StatusEffectType::Spikes |
            StatusEffectType::ToxicSpikes | StatusEffectType::BadPoison
        )
    }
    
    fn is_removable(&self, effect_type: &StatusEffectType) -> bool {
        !matches!(effect_type,
            StatusEffectType::Sunny | StatusEffectType::Rain |
            StatusEffectType::Sandstorm | StatusEffectType::Hail
        )
    }
    
    fn is_blockable(&self, effect_type: &StatusEffectType) -> bool {
        !matches!(effect_type,
            StatusEffectType::Flinch | StatusEffectType::Confusion
        )
    }
    
    fn get_default_triggers(&self, effect_type: &StatusEffectType) -> Vec<EffectTrigger> {
        match effect_type {
            StatusEffectType::Burn | StatusEffectType::Poison | StatusEffectType::BadPoison => {
                vec![EffectTrigger::OnTurnEnd]
            }
            StatusEffectType::Sleep | StatusEffectType::Freeze => {
                vec![EffectTrigger::OnTurnStart, EffectTrigger::OnMoveUsed]
            }
            StatusEffectType::Paralysis => {
                vec![EffectTrigger::OnMoveUsed]
            }
            StatusEffectType::AttackUp(_) | StatusEffectType::DefenseUp(_) |
            StatusEffectType::SpAttackUp(_) | StatusEffectType::SpDefenseUp(_) |
            StatusEffectType::SpeedUp(_) => {
                vec![EffectTrigger::OnStatChange]
            }
            _ => vec![],
        }
    }
    
    fn add_history_entry(
        &mut self,
        effect_id: u64,
        pokemon_id: u32,
        effect_type: StatusEffectType,
        action: EffectAction,
    ) {
        let entry = EffectHistoryEntry {
            effect_id,
            pokemon_id,
            effect_type,
            action,
            timestamp: std::time::Instant::now(),
        };
        
        self.effect_history.push(entry);
        
        // 限制历史大小
        if self.effect_history.len() > self.max_history_size {
            self.effect_history.remove(0);
        }
    }
}

// 效果上下文
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub attacker_id: Option<u32>,
    pub defender_id: Option<u32>,
    pub move_id: Option<MoveId>,
    pub damage_amount: Option<i32>,
    pub weather: Option<StatusEffectType>,
    pub field_effects: Vec<StatusEffectType>,
    pub turn_number: u32,
}

// 效果处理器接口
pub trait EffectHandler: Send + Sync {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult;
    fn apply_context(&self, result: EffectResult, context: &EffectContext) -> EffectResult {
        result // 默认不修改
    }
}

// 燃烧状态处理器
struct BurnHandler;

impl BurnHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for BurnHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let damage = 100; // HP的1/16
        
        EffectResult {
            success: true,
            damage,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects: Vec::new(),
            messages: vec![format!("宝可梦 {} 受到燃烧伤害！", pokemon_id)],
            prevent_action: false,
            forced_move: None,
            animation_triggers: vec!["burn_damage".to_string()],
        }
    }
}

// 中毒状态处理器
struct PoisonHandler;

impl PoisonHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for PoisonHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let damage = 125; // HP的1/8
        
        EffectResult {
            success: true,
            damage,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects: Vec::new(),
            messages: vec![format!("宝可梦 {} 受到毒素伤害！", pokemon_id)],
            prevent_action: false,
            forced_move: None,
            animation_triggers: vec!["poison_damage".to_string()],
        }
    }
}

// 剧毒状态处理器
struct BadPoisonHandler;

impl BadPoisonHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for BadPoisonHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        // 剧毒伤害随回合数增加
        let turn_multiplier = effect.stack_count;
        let damage = 125 * turn_multiplier as i32 / 16; // HP的n/16
        
        EffectResult {
            success: true,
            damage,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects: Vec::new(),
            messages: vec![format!("宝可梦 {} 受到剧毒伤害！", pokemon_id)],
            prevent_action: false,
            forced_move: None,
            animation_triggers: vec!["bad_poison_damage".to_string()],
        }
    }
}

// 睡眠状态处理器
struct SleepHandler;

impl SleepHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for SleepHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let wake_up = effect.remaining_turns <= 1 || fastrand::f32() < 0.33;
        let mut removed_effects = Vec::new();
        let mut messages = Vec::new();
        
        if wake_up {
            removed_effects.push(effect.id);
            messages.push(format!("宝可梦 {} 醒来了！", pokemon_id));
        } else {
            messages.push(format!("宝可梦 {} 在睡觉。", pokemon_id));
        }
        
        EffectResult {
            success: true,
            damage: 0,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects,
            messages,
            prevent_action: !wake_up,
            forced_move: None,
            animation_triggers: if wake_up { 
                vec!["wake_up".to_string()] 
            } else { 
                vec!["sleeping".to_string()] 
            },
        }
    }
}

// 麻痹状态处理器
struct ParalysisHandler;

impl ParalysisHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for ParalysisHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let paralyzed = fastrand::f32() < 0.25;
        
        EffectResult {
            success: true,
            damage: 0,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects: Vec::new(),
            messages: if paralyzed {
                vec![format!("宝可梦 {} 因麻痹而无法行动！", pokemon_id)]
            } else {
                Vec::new()
            },
            prevent_action: paralyzed,
            forced_move: None,
            animation_triggers: if paralyzed {
                vec!["paralysis_effect".to_string()]
            } else {
                Vec::new()
            },
        }
    }
}

// 冰冻状态处理器
struct FreezeHandler;

impl FreezeHandler {
    fn new() -> Self {
        Self
    }
}

impl EffectHandler for FreezeHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let thaw = fastrand::f32() < 0.2;
        let mut removed_effects = Vec::new();
        let mut messages = Vec::new();
        
        if thaw {
            removed_effects.push(effect.id);
            messages.push(format!("宝可梦 {} 解冻了！", pokemon_id));
        } else {
            messages.push(format!("宝可梦 {} 被冰冻无法行动！", pokemon_id));
        }
        
        EffectResult {
            success: true,
            damage: 0,
            healing: 0,
            stat_changes: HashMap::new(),
            new_effects: Vec::new(),
            removed_effects,
            messages,
            prevent_action: !thaw,
            forced_move: None,
            animation_triggers: if thaw {
                vec!["thaw".to_string()]
            } else {
                vec!["frozen".to_string()]
            },
        }
    }
}

// 能力变化处理器
struct StatChangeHandler {
    stat_name: String,
    change_amount: i8,
}

impl StatChangeHandler {
    fn new(stat_name: String, change_amount: i8) -> Self {
        Self {
            stat_name,
            change_amount,
        }
    }
}

impl EffectHandler for StatChangeHandler {
    fn process_effect(&self, effect: &StatusEffect, pokemon_id: u32) -> EffectResult {
        let mut stat_changes = HashMap::new();
        stat_changes.insert(self.stat_name.clone(), self.change_amount);
        
        EffectResult {
            success: true,
            damage: 0,
            healing: 0,
            stat_changes,
            new_effects: Vec::new(),
            removed_effects: Vec::new(),
            messages: Vec::new(),
            prevent_action: false,
            forced_move: None,
            animation_triggers: Vec::new(),
        }
    }
}

// 统计信息
#[derive(Debug, Clone)]
pub struct StatusEffectStats {
    pub active_effects: usize,
    pub total_applied: u64,
    pub history_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_status_effect_manager_creation() {
        let manager = StatusEffectManager::new();
        assert_eq!(manager.active_effects.len(), 0);
        assert!(manager.effect_handlers.contains_key(&StatusEffectType::Burn));
    }
    
    #[test]
    fn test_apply_burn_effect() {
        let mut manager = StatusEffectManager::new();
        
        let effect_id = manager.apply_effect(
            1,
            StatusEffectType::Burn,
            Some(5),
            None,
            None,
        ).unwrap();
        
        assert!(manager.active_effects.contains_key(&effect_id));
        let effect = manager.active_effects.get(&effect_id).unwrap();
        assert_eq!(effect.effect_type, StatusEffectType::Burn);
        assert_eq!(effect.remaining_turns, 5);
    }
    
    #[test]
    fn test_process_turn_effects() {
        let mut manager = StatusEffectManager::new();
        
        manager.apply_effect(1, StatusEffectType::Burn, Some(3), None, None).unwrap();
        
        let results = manager.process_turn_end_effects(1);
        assert!(!results.is_empty());
        assert!(results[0].damage > 0);
    }
    
    #[test]
    fn test_effect_stacking() {
        let mut manager = StatusEffectManager::new();
        
        // 测试能力变化的叠加
        let effect_id1 = manager.apply_effect(
            1,
            StatusEffectType::AttackUp(1),
            Some(5),
            None,
            None,
        ).unwrap();
        
        let effect_id2 = manager.apply_effect(
            1,
            StatusEffectType::AttackUp(1),
            Some(5),
            None,
            None,
        );
        
        // 应该叠加到同一个效果上
        assert_eq!(effect_id1, effect_id2.unwrap());
        
        let effect = manager.active_effects.get(&effect_id1).unwrap();
        assert_eq!(effect.stack_count, 2);
    }
}