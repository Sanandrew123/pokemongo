// 战斗回合管理器
// 开发心理：回合制战斗是宝可梦的核心，需要精确的行动顺序和状态管理
// 设计原则：事件驱动、可预测的行动顺序、支持各种战斗机制

use crate::core::{GameError, Result};
use crate::pokemon::{Pokemon, Move, StatusCondition, MoveId};
use crate::battle::{BattleState, BattleParticipant, DamageCalculator, DamageContext, BattleEnvironment};
use serde::{Deserialize, Serialize};
use std::collections::{VecDeque, HashMap};
use log::{info, debug, warn};

// 回合管理器主结构
pub struct TurnManager {
    current_turn: u32,
    action_queue: VecDeque<BattleAction>,
    turn_order: Vec<ParticipantId>,
    battle_state: BattleState,
    damage_calculator: DamageCalculator,
    environment: BattleEnvironment,
    turn_history: Vec<TurnResult>,
    speed_modifiers: HashMap<ParticipantId, f32>,
    priority_modifiers: HashMap<ParticipantId, i8>,
}

pub type ParticipantId = usize;

// 战斗行动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleAction {
    pub participant_id: ParticipantId,
    pub action_type: ActionType,
    pub priority: i8,
    pub speed: u16,
    pub turn_number: u32,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    UseMove {
        move_id: MoveId,
        target_id: Option<ParticipantId>,
        targets: Vec<ParticipantId>,
    },
    SwitchPokemon {
        pokemon_index: usize,
    },
    UseItem {
        item_id: u32,
        target_id: Option<ParticipantId>,
    },
    Flee,
    Wait,
    // 系统行动
    StatusEffect {
        effect: StatusCondition,
        source: Option<ParticipantId>,
    },
    WeatherEffect,
    FieldEffect {
        effect_name: String,
    },
}

// 回合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub turn_number: u32,
    pub actions: Vec<ActionResult>,
    pub turn_start_state: BattleSnapshot,
    pub turn_end_state: BattleSnapshot,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: BattleAction,
    pub success: bool,
    pub effects: Vec<ActionEffect>,
    pub messages: Vec<String>,
    pub damage_dealt: Option<u16>,
    pub healing_done: Option<u16>,
    pub critical_hit: bool,
    pub type_effectiveness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionEffect {
    Damage { amount: u16, target: ParticipantId },
    Heal { amount: u16, target: ParticipantId },
    StatusChange { status: StatusCondition, target: ParticipantId },
    StatChange { stat: String, stages: i8, target: ParticipantId },
    Switch { old_pokemon: usize, new_pokemon: usize, participant: ParticipantId },
    Faint { target: ParticipantId },
    Miss { target: ParticipantId },
    Critical { target: ParticipantId },
    TypeEffectiveness { effectiveness: f32, target: ParticipantId },
    Weather { new_weather: String },
    Field { effect_name: String, turns_remaining: u8 },
}

// 战斗快照（用于回滚和记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSnapshot {
    pub participants: Vec<ParticipantSnapshot>,
    pub environment: BattleEnvironment,
    pub turn_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantSnapshot {
    pub pokemon: Vec<PokemonSnapshot>,
    pub active_pokemon_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSnapshot {
    pub current_hp: u16,
    pub status_conditions: Vec<StatusCondition>,
    pub stat_stages: HashMap<String, i8>,
    pub pp_remaining: Vec<u8>,
}

impl TurnManager {
    pub fn new(participants: Vec<BattleParticipant>, environment: BattleEnvironment) -> Self {
        let battle_state = BattleState::new(participants);
        
        Self {
            current_turn: 0,
            action_queue: VecDeque::new(),
            turn_order: Vec::new(),
            battle_state,
            damage_calculator: DamageCalculator::new(),
            environment,
            turn_history: Vec::new(),
            speed_modifiers: HashMap::new(),
            priority_modifiers: HashMap::new(),
        }
    }
    
    // 添加行动到队列
    pub fn queue_action(&mut self, action: BattleAction) -> Result<()> {
        debug!("添加行动到队列: {:?}", action.action_type);
        
        // 验证行动有效性
        self.validate_action(&action)?;
        
        // 计算行动优先级
        let priority = self.calculate_action_priority(&action)?;
        let mut action = action;
        action.priority = priority;
        
        // 插入到正确的位置（按优先级和速度排序）
        let insert_pos = self.find_insert_position(&action);
        self.action_queue.insert(insert_pos, action);
        
        Ok(())
    }
    
    // 处理一个完整回合
    pub fn process_turn(&mut self) -> Result<TurnResult> {
        let turn_start_time = std::time::Instant::now();
        self.current_turn += 1;
        
        info!("开始处理第{}回合", self.current_turn);
        
        // 记录回合开始状态
        let turn_start_state = self.create_battle_snapshot();
        
        // 回合开始事件
        self.handle_turn_start_events()?;
        
        // 处理所有排队的行动
        let mut action_results = Vec::new();
        while let Some(action) = self.action_queue.pop_front() {
            if self.is_battle_over() {
                break;
            }
            
            let result = self.execute_action(action)?;
            action_results.push(result);
        }
        
        // 回合结束事件
        self.handle_turn_end_events()?;
        
        // 记录回合结束状态
        let turn_end_state = self.create_battle_snapshot();
        
        let turn_result = TurnResult {
            turn_number: self.current_turn,
            actions: action_results,
            turn_start_state,
            turn_end_state,
            duration_ms: turn_start_time.elapsed().as_millis() as u64,
        };
        
        self.turn_history.push(turn_result.clone());
        
        debug!("第{}回合处理完成，用时{}ms", self.current_turn, turn_result.duration_ms);
        Ok(turn_result)
    }
    
    // 执行单个行动
    fn execute_action(&mut self, action: BattleAction) -> Result<ActionResult> {
        debug!("执行行动: {:?}", action.action_type);
        
        let mut result = ActionResult {
            action: action.clone(),
            success: false,
            effects: Vec::new(),
            messages: Vec::new(),
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        match &action.action_type {
            ActionType::UseMove { move_id, target_id, targets } => {
                result = self.execute_move_action(&action, *move_id, target_id, targets)?;
            },
            ActionType::SwitchPokemon { pokemon_index } => {
                result = self.execute_switch_action(&action, *pokemon_index)?;
            },
            ActionType::UseItem { item_id, target_id } => {
                result = self.execute_item_action(&action, *item_id, target_id)?;
            },
            ActionType::Flee => {
                result = self.execute_flee_action(&action)?;
            },
            ActionType::StatusEffect { effect, source } => {
                result = self.execute_status_effect(&action, effect, source)?;
            },
            ActionType::WeatherEffect => {
                result = self.execute_weather_effect(&action)?;
            },
            ActionType::FieldEffect { effect_name } => {
                result = self.execute_field_effect(&action, effect_name)?;
            },
            ActionType::Wait => {
                result.success = true;
                result.messages.push("等待中...".to_string());
            },
        }
        
        Ok(result)
    }
    
    // 执行技能行动
    fn execute_move_action(
        &mut self, 
        action: &BattleAction, 
        move_id: MoveId, 
        target_id: &Option<ParticipantId>,
        targets: &[ParticipantId]
    ) -> Result<ActionResult> {
        let mut result = ActionResult {
            action: action.clone(),
            success: false,
            effects: Vec::new(),
            messages: Vec::new(),
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        // 获取使用者的宝可梦
        let user_pokemon = self.battle_state.get_active_pokemon(action.participant_id)
            .ok_or_else(|| GameError::BattleError("无效的参与者ID".to_string()))?;
        
        // 获取技能数据
        let move_data = crate::pokemon::moves::get_move(move_id)
            .ok_or_else(|| GameError::BattleError("无效的技能ID".to_string()))?;
        
        // 检查PP
        if !self.check_move_pp(action.participant_id, move_id)? {
            result.messages.push(format!("{}的PP不足！", move_data.name));
            return Ok(result);
        }
        
        // 消耗PP
        self.consume_move_pp(action.participant_id, move_id)?;
        
        // 检查命中率
        let battle_context = self.create_battle_context(action.participant_id);
        if !move_data.check_accuracy(&battle_context) {
            result.messages.push(format!("{}的{}没有命中！", 
                user_pokemon.get_display_name(), move_data.name));
            result.effects.push(ActionEffect::Miss { 
                target: target_id.unwrap_or(action.participant_id) 
            });
            return Ok(result);
        }
        
        result.messages.push(format!("{}使用了{}！", 
            user_pokemon.get_display_name(), move_data.name));
        
        // 根据技能目标类型处理
        let actual_targets = self.resolve_move_targets(action.participant_id, move_data, targets)?;
        
        // 对每个目标执行技能效果
        for target_id in actual_targets {
            let target_effects = self.apply_move_to_target(
                action.participant_id, 
                target_id, 
                move_data,
                &battle_context
            )?;
            
            result.effects.extend(target_effects);
        }
        
        result.success = true;
        Ok(result)
    }
    
    // 对目标应用技能效果
    fn apply_move_to_target(
        &mut self,
        user_id: ParticipantId,
        target_id: ParticipantId,
        move_data: &Move,
        context: &crate::pokemon::moves::BattleContext
    ) -> Result<Vec<ActionEffect>> {
        let mut effects = Vec::new();
        
        // 获取用户和目标宝可梦
        let user_pokemon = self.battle_state.get_active_pokemon(user_id)
            .ok_or_else(|| GameError::BattleError("用户宝可梦不存在".to_string()))?;
        
        let target_pokemon = self.battle_state.get_active_pokemon_mut(target_id)
            .ok_or_else(|| GameError::BattleError("目标宝可梦不存在".to_string()))?;
        
        // 处理伤害技能
        if let Some(_power) = move_data.power {
            let damage_context = crate::battle::damage_calculator::create_damage_context(
                user_pokemon,
                target_pokemon,
                move_data,
                &self.environment,
                fastrand::f32() < 0.0625, // 1/16概率暴击
            );
            
            let damage_result = self.damage_calculator.calculate_damage(&damage_context)?;
            
            if damage_result.final_damage > 0 {
                let fainted = target_pokemon.take_damage(damage_result.final_damage as u16);
                
                effects.push(ActionEffect::Damage {
                    amount: damage_result.final_damage as u16,
                    target: target_id,
                });
                
                if damage_result.is_critical {
                    effects.push(ActionEffect::Critical { target: target_id });
                }
                
                if damage_result.type_effectiveness != 1.0 {
                    effects.push(ActionEffect::TypeEffectiveness {
                        effectiveness: damage_result.type_effectiveness,
                        target: target_id,
                    });
                }
                
                if fainted {
                    effects.push(ActionEffect::Faint { target: target_id });
                }
            }
        }
        
        // 处理次要效果
        for secondary_effect in &move_data.secondary_effects {
            if fastrand::f32() < secondary_effect.chance {
                let effect_results = self.apply_move_effect(&secondary_effect.effect, target_id)?;
                effects.extend(effect_results);
            }
        }
        
        Ok(effects)
    }
    
    // 应用技能效果
    fn apply_move_effect(&mut self, effect: &crate::pokemon::moves::MoveEffect, target_id: ParticipantId) -> Result<Vec<ActionEffect>> {
        let mut effects = Vec::new();
        
        match effect {
            crate::pokemon::moves::MoveEffect::StatusChange { status, .. } => {
                let target_pokemon = self.battle_state.get_active_pokemon_mut(target_id)
                    .ok_or_else(|| GameError::BattleError("目标宝可梦不存在".to_string()))?;
                
                let status_condition = match status {
                    crate::pokemon::moves::StatusEffect::Burn => StatusCondition::Burn,
                    crate::pokemon::moves::StatusEffect::Paralysis => StatusCondition::Paralysis,
                    crate::pokemon::moves::StatusEffect::Poison => StatusCondition::Poison,
                    crate::pokemon::moves::StatusEffect::Sleep => StatusCondition::Sleep { turns_remaining: 3 },
                    _ => StatusCondition::None,
                };
                
                if !status_condition.eq(&StatusCondition::None) {
                    target_pokemon.apply_status(status_condition.clone());
                    effects.push(ActionEffect::StatusChange {
                        status: status_condition,
                        target: target_id,
                    });
                }
            },
            crate::pokemon::moves::MoveEffect::StatChange { stat, stages, .. } => {
                // 简化的能力值变化实现
                let stat_name = match stat {
                    crate::pokemon::moves::StatType::Attack => "attack",
                    crate::pokemon::moves::StatType::Defense => "defense",
                    crate::pokemon::moves::StatType::Speed => "speed",
                    _ => "unknown",
                };
                
                effects.push(ActionEffect::StatChange {
                    stat: stat_name.to_string(),
                    stages: *stages,
                    target: target_id,
                });
            },
            _ => {
                // 其他效果的简化处理
                debug!("未实现的技能效果: {:?}", effect);
            }
        }
        
        Ok(effects)
    }
    
    // 执行换宝可梦行动
    fn execute_switch_action(&mut self, action: &BattleAction, pokemon_index: usize) -> Result<ActionResult> {
        let mut result = ActionResult {
            action: action.clone(),
            success: false,
            effects: Vec::new(),
            messages: Vec::new(),
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        let old_index = self.battle_state.get_active_pokemon_index(action.participant_id);
        
        if self.battle_state.switch_pokemon(action.participant_id, pokemon_index)? {
            let new_pokemon = self.battle_state.get_active_pokemon(action.participant_id)
                .ok_or_else(|| GameError::BattleError("切换后的宝可梦不存在".to_string()))?;
            
            result.success = true;
            result.messages.push(format!("切换到了{}！", new_pokemon.get_display_name()));
            result.effects.push(ActionEffect::Switch {
                old_pokemon: old_index,
                new_pokemon: pokemon_index,
                participant: action.participant_id,
            });
        } else {
            result.messages.push("无法切换宝可梦！".to_string());
        }
        
        Ok(result)
    }
    
    // 执行道具使用行动
    fn execute_item_action(&mut self, action: &BattleAction, _item_id: u32, _target_id: &Option<ParticipantId>) -> Result<ActionResult> {
        let result = ActionResult {
            action: action.clone(),
            success: false,
            effects: Vec::new(),
            messages: vec!["道具系统尚未实现".to_string()],
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        Ok(result)
    }
    
    // 执行逃跑行动
    fn execute_flee_action(&mut self, action: &BattleAction) -> Result<ActionResult> {
        let result = ActionResult {
            action: action.clone(),
            success: true,
            effects: Vec::new(),
            messages: vec!["成功逃跑了！".to_string()],
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        // 设置战斗结束状态
        self.battle_state.set_battle_ended(true);
        
        Ok(result)
    }
    
    // 执行状态效果
    fn execute_status_effect(&mut self, action: &BattleAction, effect: &StatusCondition, _source: &Option<ParticipantId>) -> Result<ActionResult> {
        let mut result = ActionResult {
            action: action.clone(),
            success: false,
            effects: Vec::new(),
            messages: Vec::new(),
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        if let Some(pokemon) = self.battle_state.get_active_pokemon_mut(action.participant_id) {
            // 处理状态异常效果
            match effect {
                StatusCondition::Burn => {
                    let damage = pokemon.get_stats()?.hp / 16; // 最大HP的1/16
                    let fainted = pokemon.take_damage(damage);
                    result.messages.push(format!("{}因灼伤受到了伤害！", pokemon.get_display_name()));
                    result.effects.push(ActionEffect::Damage {
                        amount: damage,
                        target: action.participant_id,
                    });
                    if fainted {
                        result.effects.push(ActionEffect::Faint { target: action.participant_id });
                    }
                },
                StatusCondition::Poison => {
                    let damage = pokemon.get_stats()?.hp / 8; // 最大HP的1/8
                    let fainted = pokemon.take_damage(damage);
                    result.messages.push(format!("{}因中毒受到了伤害！", pokemon.get_display_name()));
                    result.effects.push(ActionEffect::Damage {
                        amount: damage,
                        target: action.participant_id,
                    });
                    if fainted {
                        result.effects.push(ActionEffect::Faint { target: action.participant_id });
                    }
                },
                _ => {
                    result.messages.push(format!("{}受到状态异常影响", pokemon.get_display_name()));
                }
            }
            result.success = true;
        }
        
        Ok(result)
    }
    
    // 执行天气效果
    fn execute_weather_effect(&mut self, action: &BattleAction) -> Result<ActionResult> {
        let result = ActionResult {
            action: action.clone(),
            success: true,
            effects: Vec::new(),
            messages: vec!["天气效果生效".to_string()],
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        // 简化的天气效果处理
        Ok(result)
    }
    
    // 执行场地效果
    fn execute_field_effect(&mut self, action: &BattleAction, effect_name: &str) -> Result<ActionResult> {
        let result = ActionResult {
            action: action.clone(),
            success: true,
            effects: vec![ActionEffect::Field {
                effect_name: effect_name.to_string(),
                turns_remaining: 5,
            }],
            messages: vec![format!("{}效果生效", effect_name)],
            damage_dealt: None,
            healing_done: None,
            critical_hit: false,
            type_effectiveness: 1.0,
        };
        
        Ok(result)
    }
    
    // 处理回合开始事件
    fn handle_turn_start_events(&mut self) -> Result<()> {
        debug!("处理回合开始事件");
        
        // 处理状态异常
        for participant_id in 0..self.battle_state.get_participant_count() {
            if let Some(pokemon) = self.battle_state.get_active_pokemon(participant_id) {
                for status in &pokemon.status_conditions.clone() {
                    if !matches!(status, StatusCondition::None) {
                        let action = BattleAction {
                            participant_id,
                            action_type: ActionType::StatusEffect {
                                effect: status.clone(),
                                source: None,
                            },
                            priority: 0,
                            speed: 0,
                            turn_number: self.current_turn,
                            timestamp: std::time::Instant::now(),
                        };
                        self.action_queue.push_back(action);
                    }
                }
            }
        }
        
        // 处理天气效果
        if self.environment.weather.is_some() {
            let action = BattleAction {
                participant_id: 0,
                action_type: ActionType::WeatherEffect,
                priority: 0,
                speed: 0,
                turn_number: self.current_turn,
                timestamp: std::time::Instant::now(),
            };
            self.action_queue.push_back(action);
        }
        
        Ok(())
    }
    
    // 处理回合结束事件
    fn handle_turn_end_events(&mut self) -> Result<()> {
        debug!("处理回合结束事件");
        
        // 减少天气持续时间
        if let Some(weather) = &mut self.environment.weather {
            if let Some(ref mut turns) = self.environment.weather_turns {
                *turns = turns.saturating_sub(1);
                if *turns == 0 {
                    self.environment.weather = None;
                    self.environment.weather_turns = None;
                    info!("天气效果结束了");
                }
            }
        }
        
        // 处理场地效果衰减
        // 这里可以添加更多回合结束逻辑
        
        Ok(())
    }
    
    // 辅助方法
    fn validate_action(&self, action: &BattleAction) -> Result<()> {
        // 检查参与者ID是否有效
        if action.participant_id >= self.battle_state.get_participant_count() {
            return Err(GameError::BattleError("无效的参与者ID".to_string()));
        }
        
        // 检查宝可梦是否濒死
        if let Some(pokemon) = self.battle_state.get_active_pokemon(action.participant_id) {
            if pokemon.is_fainted() && !matches!(action.action_type, ActionType::SwitchPokemon { .. }) {
                return Err(GameError::BattleError("濒死的宝可梦无法行动".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn calculate_action_priority(&self, action: &BattleAction) -> Result<i8> {
        let base_priority = match &action.action_type {
            ActionType::UseMove { move_id, .. } => {
                if let Some(move_data) = crate::pokemon::moves::get_move(*move_id) {
                    move_data.priority
                } else {
                    0
                }
            },
            ActionType::SwitchPokemon { .. } => 6, // 换宝可梦优先级最高
            ActionType::UseItem { .. } => 6,      // 道具使用优先级也很高
            ActionType::Flee => -7,               // 逃跑优先级最低
            _ => 0,
        };
        
        // 应用优先级修正
        let modifier = self.priority_modifiers.get(&action.participant_id).unwrap_or(&0);
        Ok(base_priority + modifier)
    }
    
    fn find_insert_position(&self, action: &BattleAction) -> usize {
        for (i, existing_action) in self.action_queue.iter().enumerate() {
            if action.priority > existing_action.priority {
                return i;
            } else if action.priority == existing_action.priority {
                if action.speed > existing_action.speed {
                    return i;
                }
            }
        }
        self.action_queue.len()
    }
    
    fn is_battle_over(&self) -> bool {
        self.battle_state.is_battle_ended()
    }
    
    fn create_battle_snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            participants: self.battle_state.get_participants().iter()
                .map(|p| ParticipantSnapshot {
                    pokemon: p.pokemon.iter().map(|pokemon| PokemonSnapshot {
                        current_hp: pokemon.current_hp,
                        status_conditions: pokemon.status_conditions.clone(),
                        stat_stages: HashMap::new(), // 简化实现
                        pp_remaining: pokemon.moves.iter().map(|m| m.current_pp).collect(),
                    }).collect(),
                    active_pokemon_index: p.active_pokemon_index,
                })
                .collect(),
            environment: self.environment.clone(),
            turn_number: self.current_turn,
        }
    }
    
    fn create_battle_context(&self, participant_id: ParticipantId) -> crate::pokemon::moves::BattleContext {
        let pokemon = self.battle_state.get_active_pokemon(participant_id);
        
        crate::pokemon::moves::BattleContext {
            user_level: pokemon.map_or(50, |p| p.level),
            user_hp_ratio: pokemon.map_or(1.0, |p| {
                if let Ok(stats) = p.get_stats() {
                    p.current_hp as f32 / stats.hp as f32
                } else {
                    1.0
                }
            }),
            target_hp_ratio: 1.0, // 简化实现
            weather: self.environment.weather,
            turn_count: self.current_turn,
            consecutive_uses: 1, // 简化实现
        }
    }
    
    fn check_move_pp(&self, participant_id: ParticipantId, move_id: MoveId) -> Result<bool> {
        if let Some(pokemon) = self.battle_state.get_active_pokemon(participant_id) {
            for move_slot in &pokemon.moves {
                if move_slot.move_id == move_id {
                    return Ok(move_slot.current_pp > 0);
                }
            }
        }
        Ok(false)
    }
    
    fn consume_move_pp(&mut self, participant_id: ParticipantId, move_id: MoveId) -> Result<()> {
        if let Some(pokemon) = self.battle_state.get_active_pokemon_mut(participant_id) {
            for move_slot in &mut pokemon.moves {
                if move_slot.move_id == move_id && move_slot.current_pp > 0 {
                    move_slot.current_pp -= 1;
                    return Ok(());
                }
            }
        }
        Err(GameError::BattleError("无法消耗PP".to_string()))
    }
    
    fn resolve_move_targets(&self, user_id: ParticipantId, move_data: &Move, targets: &[ParticipantId]) -> Result<Vec<ParticipantId>> {
        match move_data.target {
            crate::pokemon::moves::MoveTarget::SingleOpponent => {
                if targets.is_empty() {
                    // 自动选择第一个可用对手
                    for i in 0..self.battle_state.get_participant_count() {
                        if i != user_id && !self.battle_state.get_active_pokemon(i).map_or(true, |p| p.is_fainted()) {
                            return Ok(vec![i]);
                        }
                    }
                    Ok(vec![])
                } else {
                    Ok(targets.to_vec())
                }
            },
            crate::pokemon::moves::MoveTarget::AllOpponents => {
                let mut opponents = Vec::new();
                for i in 0..self.battle_state.get_participant_count() {
                    if i != user_id && !self.battle_state.get_active_pokemon(i).map_or(true, |p| p.is_fainted()) {
                        opponents.push(i);
                    }
                }
                Ok(opponents)
            },
            crate::pokemon::moves::MoveTarget::User => {
                Ok(vec![user_id])
            },
            _ => {
                // 简化处理其他目标类型
                Ok(targets.to_vec())
            }
        }
    }
    
    // 公共访问方法
    pub fn get_current_turn(&self) -> u32 {
        self.current_turn
    }
    
    pub fn get_battle_state(&self) -> &BattleState {
        &self.battle_state
    }
    
    pub fn get_turn_history(&self) -> &[TurnResult] {
        &self.turn_history
    }
    
    pub fn is_action_queue_empty(&self) -> bool {
        self.action_queue.is_empty()
    }
    
    pub fn get_environment(&self) -> &BattleEnvironment {
        &self.environment
    }
    
    pub fn set_speed_modifier(&mut self, participant_id: ParticipantId, modifier: f32) {
        self.speed_modifiers.insert(participant_id, modifier);
    }
    
    pub fn set_priority_modifier(&mut self, participant_id: ParticipantId, modifier: i8) {
        self.priority_modifiers.insert(participant_id, modifier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{Pokemon, PokemonSpecies};
    
    fn create_test_pokemon() -> Pokemon {
        Pokemon::new(25, 50, None, "Test".to_string(), "Test Location".to_string()).unwrap()
    }
    
    fn create_test_battle_participant() -> BattleParticipant {
        BattleParticipant::new(vec![create_test_pokemon()])
    }
    
    #[test]
    fn test_turn_manager_creation() {
        let participants = vec![
            create_test_battle_participant(),
            create_test_battle_participant(),
        ];
        let environment = BattleEnvironment::default();
        
        let turn_manager = TurnManager::new(participants, environment);
        assert_eq!(turn_manager.current_turn, 0);
        assert!(turn_manager.action_queue.is_empty());
    }
    
    #[test]
    fn test_action_queuing() {
        let participants = vec![
            create_test_battle_participant(),
            create_test_battle_participant(),
        ];
        let environment = BattleEnvironment::default();
        let mut turn_manager = TurnManager::new(participants, environment);
        
        let action = BattleAction {
            participant_id: 0,
            action_type: ActionType::UseMove {
                move_id: 1,
                target_id: Some(1),
                targets: vec![1],
            },
            priority: 0,
            speed: 100,
            turn_number: 1,
            timestamp: std::time::Instant::now(),
        };
        
        assert!(turn_manager.queue_action(action).is_ok());
        assert!(!turn_manager.is_action_queue_empty());
    }
    
    #[test]
    fn test_priority_ordering() {
        let participants = vec![
            create_test_battle_participant(),
            create_test_battle_participant(),
        ];
        let environment = BattleEnvironment::default();
        let mut turn_manager = TurnManager::new(participants, environment);
        
        // 添加低优先级行动
        let low_priority = BattleAction {
            participant_id: 0,
            action_type: ActionType::UseMove {
                move_id: 1, // 撞击，优先级0
                target_id: Some(1),
                targets: vec![1],
            },
            priority: 0,
            speed: 50,
            turn_number: 1,
            timestamp: std::time::Instant::now(),
        };
        
        // 添加高优先级行动
        let high_priority = BattleAction {
            participant_id: 1,
            action_type: ActionType::SwitchPokemon { pokemon_index: 0 },
            priority: 6,
            speed: 30,
            turn_number: 1,
            timestamp: std::time::Instant::now(),
        };
        
        turn_manager.queue_action(low_priority).unwrap();
        turn_manager.queue_action(high_priority).unwrap();
        
        // 高优先级行动应该在前面
        assert_eq!(turn_manager.action_queue[0].priority, 6);
        assert_eq!(turn_manager.action_queue[1].priority, 0);
    }
}