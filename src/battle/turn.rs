/*
* 开发心理过程：
* 1. 实现回合制战斗的核心逻辑，管理回合顺序和行动
* 2. 处理速度计算和优先级系统
* 3. 支持多种行动类型（攻击、道具、逃跑、换Pokemon等）
* 4. 实现状态效果的回合处理
* 5. 管理战斗阶段和时机
* 6. 支持双打和多人战斗扩展
* 7. 集成AI决策和玩家输入系统
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        individual::{IndividualPokemon, StatusType, BattleStats},
        moves::{Move, MoveId, MoveCategory, MoveTarget},
        species::PokemonSpecies,
        stats::StatType,
    },
    battle::{
        engine::{BattleContext, BattleParticipant, BattleResult},
        damage::DamageCalculator,
        effects::EffectProcessor,
    },
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    SelectActions,      // 选择行动阶段
    ProcessActions,     // 执行行动阶段
    ApplyEffects,       // 应用效果阶段
    CheckFaint,         // 检查濒死阶段
    EndTurn,           // 回合结束阶段
}

#[derive(Debug, Clone)]
pub struct TurnManager {
    pub current_turn: u32,
    pub current_phase: TurnPhase,
    pub action_queue: VecDeque<TurnAction>,
    pub participant_speeds: HashMap<Uuid, u16>,
    pub turn_history: Vec<TurnRecord>,
    pub effects_queue: VecDeque<EffectApplication>,
    pub fainted_pokemon: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnAction {
    pub action_id: Uuid,
    pub participant_id: Uuid,
    pub pokemon_id: Uuid,
    pub action_type: ActionType,
    pub target: Option<ActionTarget>,
    pub priority: i8,
    pub speed: u16,
    pub selected_at: f64, // 用于同优先级同速度的随机处理
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    UseMove {
        move_id: MoveId,
        move_slot: usize,
    },
    UseItem {
        item_id: u32,
    },
    SwitchPokemon {
        target_pokemon_id: Uuid,
    },
    Run {
        success_chance: f32,
    },
    Struggle, // 没有PP时的挣扎
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTarget {
    Opponent(Uuid),
    Ally(Uuid),
    Self_,
    AllOpponents,
    AllAllies,
    AllOthers,
    Field,
}

#[derive(Debug, Clone)]
pub struct TurnRecord {
    pub turn_number: u32,
    pub actions: Vec<CompletedAction>,
    pub effects_applied: Vec<EffectApplication>,
    pub damage_dealt: HashMap<Uuid, u16>,
    pub status_changes: Vec<StatusChange>,
    pub fainted_pokemon: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CompletedAction {
    pub action: TurnAction,
    pub success: bool,
    pub result: ActionResult,
    pub critical_hit: bool,
    pub effectiveness: f32,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    MoveDamage {
        damage: u16,
        target_id: Uuid,
    },
    MoveStatus {
        status_applied: Vec<StatusType>,
        target_id: Uuid,
    },
    ItemUsed {
        item_consumed: bool,
        effect_description: String,
    },
    SwitchSuccessful {
        old_pokemon_id: Uuid,
        new_pokemon_id: Uuid,
    },
    RunSuccessful,
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct StatusChange {
    pub pokemon_id: Uuid,
    pub status_type: StatusType,
    pub added: bool, // true表示添加，false表示移除
}

#[derive(Debug, Clone)]
pub struct EffectApplication {
    pub effect_id: String,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub effect_type: EffectType,
    pub duration: Option<u8>,
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Damage,
    Heal,
    StatusInfliction,
    StatModification,
    WeatherChange,
    FieldEffect,
}

impl TurnManager {
    pub fn new() -> Self {
        Self {
            current_turn: 1,
            current_phase: TurnPhase::SelectActions,
            action_queue: VecDeque::new(),
            participant_speeds: HashMap::new(),
            turn_history: Vec::new(),
            effects_queue: VecDeque::new(),
            fainted_pokemon: Vec::new(),
        }
    }

    pub fn start_turn(&mut self, context: &BattleContext) -> GameResult<()> {
        self.current_phase = TurnPhase::SelectActions;
        self.action_queue.clear();
        self.effects_queue.clear();
        self.fainted_pokemon.clear();

        // 更新所有参与者的速度
        self.update_participant_speeds(context)?;

        // 处理回合开始效果
        self.apply_turn_start_effects(context)?;

        Ok(())
    }

    fn update_participant_speeds(&mut self, context: &BattleContext) -> GameResult<()> {
        self.participant_speeds.clear();

        for participant in &context.participants {
            if let Some(active_pokemon) = participant.active_pokemon.as_ref() {
                if active_pokemon.current_hp > 0 {
                    let speed = self.calculate_effective_speed(
                        active_pokemon,
                        &context.weather,
                        &context.field_effects,
                    )?;
                    self.participant_speeds.insert(participant.id, speed);
                }
            }
        }

        Ok(())
    }

    fn calculate_effective_speed(
        &self,
        pokemon: &IndividualPokemon,
        weather: &crate::world::weather::WeatherCondition,
        field_effects: &HashMap<String, crate::battle::effects::FieldEffect>,
    ) -> GameResult<u16> {
        // 基础速度值
        let base_speed = if let Some(battle_stats) = &pokemon.battle_stats {
            let stage = battle_stats.stat_stages.get(&StatType::Speed).copied().unwrap_or(0);
            let multiplier = Self::get_stat_stage_multiplier(stage);
            (pokemon.cached_stats.as_ref().unwrap().speed as f32 * multiplier) as u16
        } else {
            pokemon.cached_stats.as_ref().unwrap().speed
        };

        let mut effective_speed = base_speed as f32;

        // 状态效果修正
        if pokemon.has_status(StatusType::Paralysis) {
            effective_speed *= 0.25; // 麻痹减少75%速度
        }

        // 天气效果
        // 在实际实现中会根据天气类型调整

        // 道具效果
        // 在实际实现中会根据持有道具调整

        // 能力效果
        // 在实际实现中会根据Pokemon能力调整

        Ok(effective_speed.max(1.0) as u16)
    }

    fn get_stat_stage_multiplier(stage: i8) -> f32 {
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

    pub fn submit_action(&mut self, action: TurnAction) -> GameResult<()> {
        if self.current_phase != TurnPhase::SelectActions {
            return Err(GameError::BattleError("不在行动选择阶段".to_string()));
        }

        // 验证行动有效性
        self.validate_action(&action)?;

        // 计算行动优先级和速度
        let processed_action = self.process_action_priority(action)?;

        self.action_queue.push_back(processed_action);
        Ok(())
    }

    fn validate_action(&self, action: &TurnAction) -> GameResult<()> {
        match &action.action_type {
            ActionType::UseMove { move_id, move_slot } => {
                // 验证招式是否可用
                if *move_slot >= 4 {
                    return Err(GameError::BattleError("无效的招式位置".to_string()));
                }
                // 检查PP是否足够
                // 在实际实现中会检查Pokemon的招式PP
            },
            ActionType::UseItem { item_id } => {
                // 验证道具是否可用
                // 在实际实现中会检查背包中是否有该道具
            },
            ActionType::SwitchPokemon { target_pokemon_id } => {
                // 验证目标Pokemon是否可用
                // 在实际实现中会检查目标Pokemon的状态
            },
            ActionType::Run { .. } => {
                // 验证是否可以逃跑
                // 在实际实现中会根据战斗类型检查
            },
            ActionType::Struggle => {
                // 挣扎总是可用的
            },
        }

        Ok(())
    }

    fn process_action_priority(&self, mut action: TurnAction) -> GameResult<TurnAction> {
        // 获取行动基础优先级
        action.priority = match &action.action_type {
            ActionType::UseMove { move_id, .. } => {
                // 在实际实现中会从招式数据获取优先级
                0
            },
            ActionType::UseItem { .. } => 6, // 道具使用优先级最高
            ActionType::SwitchPokemon { .. } => 6, // 换Pokemon优先级最高
            ActionType::Run { .. } => 6, // 逃跑优先级最高
            ActionType::Struggle => 0,
        };

        // 获取Pokemon速度
        action.speed = self.participant_speeds.get(&action.participant_id).copied().unwrap_or(0);

        // 设置选择时间用于打破平局
        action.selected_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Ok(action)
    }

    pub fn can_proceed_to_next_phase(&self, total_participants: usize) -> bool {
        match self.current_phase {
            TurnPhase::SelectActions => {
                // 所有存活的参与者都已提交行动
                self.action_queue.len() >= total_participants.saturating_sub(self.fainted_pokemon.len())
            },
            _ => true,
        }
    }

    pub fn proceed_to_next_phase(&mut self) -> TurnPhase {
        self.current_phase = match self.current_phase {
            TurnPhase::SelectActions => {
                // 排序行动队列
                self.sort_action_queue();
                TurnPhase::ProcessActions
            },
            TurnPhase::ProcessActions => TurnPhase::ApplyEffects,
            TurnPhase::ApplyEffects => TurnPhase::CheckFaint,
            TurnPhase::CheckFaint => TurnPhase::EndTurn,
            TurnPhase::EndTurn => {
                // 开始新回合
                self.current_turn += 1;
                TurnPhase::SelectActions
            },
        };

        self.current_phase
    }

    fn sort_action_queue(&mut self) {
        // 将VecDeque转换为Vec进行排序
        let mut actions: Vec<_> = self.action_queue.drain(..).collect();
        
        actions.sort_by(|a, b| {
            // 首先按优先级排序（高优先级在前）
            let priority_cmp = b.priority.cmp(&a.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // 然后按速度排序（高速度在前）
            let speed_cmp = b.speed.cmp(&a.speed);
            if speed_cmp != std::cmp::Ordering::Equal {
                return speed_cmp;
            }

            // 最后按选择时间排序（先选择的在前）
            a.selected_at.partial_cmp(&b.selected_at).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 转换回VecDeque
        self.action_queue = actions.into();
    }

    pub fn get_next_action(&mut self) -> Option<TurnAction> {
        self.action_queue.pop_front()
    }

    pub fn execute_action(
        &mut self,
        action: TurnAction,
        context: &mut BattleContext,
        damage_calculator: &DamageCalculator,
        effect_processor: &EffectProcessor,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        let result = match &action.action_type {
            ActionType::UseMove { move_id, move_slot } => {
                self.execute_move_action(&action, *move_id, *move_slot, context, damage_calculator, rng)?
            },
            ActionType::UseItem { item_id } => {
                self.execute_item_action(&action, *item_id, context)?
            },
            ActionType::SwitchPokemon { target_pokemon_id } => {
                self.execute_switch_action(&action, *target_pokemon_id, context)?
            },
            ActionType::Run { success_chance } => {
                self.execute_run_action(&action, *success_chance, rng)?
            },
            ActionType::Struggle => {
                self.execute_struggle_action(&action, context, damage_calculator, rng)?
            },
        };

        Ok(result)
    }

    fn execute_move_action(
        &self,
        action: &TurnAction,
        move_id: MoveId,
        move_slot: usize,
        context: &mut BattleContext,
        damage_calculator: &DamageCalculator,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        // 查找使用者Pokemon
        let user_pokemon = self.find_pokemon_by_id(action.pokemon_id, context)
            .ok_or_else(|| GameError::BattleError("找不到使用者Pokemon".to_string()))?;

        // 检查PP
        if move_slot >= user_pokemon.moves.len() {
            return Ok(CompletedAction {
                action: action.clone(),
                success: false,
                result: ActionResult::Failed { reason: "无效的招式位置".to_string() },
                critical_hit: false,
                effectiveness: 0.0,
            });
        }

        let learned_move = &user_pokemon.moves[move_slot];
        if learned_move.current_pp == 0 {
            return Ok(CompletedAction {
                action: action.clone(),
                success: false,
                result: ActionResult::Failed { reason: "PP不足".to_string() },
                critical_hit: false,
                effectiveness: 0.0,
            });
        }

        // 获取招式数据
        // 在实际实现中会从数据库获取招式信息
        let move_data = self.get_move_data(move_id)?;

        // 检查命中率
        if !self.check_move_accuracy(&move_data, user_pokemon, action.target.as_ref(), rng) {
            return Ok(CompletedAction {
                action: action.clone(),
                success: false,
                result: ActionResult::Failed { reason: "招式未命中".to_string() },
                critical_hit: false,
                effectiveness: 0.0,
            });
        }

        // 执行招式效果
        match move_data.category {
            MoveCategory::Physical | MoveCategory::Special => {
                self.execute_damage_move(action, &move_data, user_pokemon, context, damage_calculator, rng)
            },
            MoveCategory::Status => {
                self.execute_status_move(action, &move_data, user_pokemon, context, rng)
            },
        }
    }

    fn execute_damage_move(
        &self,
        action: &TurnAction,
        move_data: &Move,
        user_pokemon: &IndividualPokemon,
        context: &BattleContext,
        damage_calculator: &DamageCalculator,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        // 选择目标
        let targets = self.select_move_targets(action.target.as_ref(), &move_data.target, context)?;
        
        if targets.is_empty() {
            return Ok(CompletedAction {
                action: action.clone(),
                success: false,
                result: ActionResult::Failed { reason: "没有有效目标".to_string() },
                critical_hit: false,
                effectiveness: 0.0,
            });
        }

        // 对每个目标计算伤害
        let target = &targets[0]; // 简化处理，只处理第一个目标
        let target_pokemon = self.find_pokemon_by_id(target.pokemon_id, context)
            .ok_or_else(|| GameError::BattleError("找不到目标Pokemon".to_string()))?;

        // 计算伤害
        let damage_result = damage_calculator.calculate_damage(
            user_pokemon,
            target_pokemon,
            move_data,
            &context.weather,
            &context.field_effects,
            rng,
        )?;

        Ok(CompletedAction {
            action: action.clone(),
            success: true,
            result: ActionResult::MoveDamage {
                damage: damage_result.damage,
                target_id: target.pokemon_id,
            },
            critical_hit: damage_result.critical_hit,
            effectiveness: damage_result.type_effectiveness,
        })
    }

    fn execute_status_move(
        &self,
        action: &TurnAction,
        move_data: &Move,
        user_pokemon: &IndividualPokemon,
        context: &BattleContext,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        // 状态招式的实现
        // 在实际实现中会根据具体招式效果处理

        Ok(CompletedAction {
            action: action.clone(),
            success: true,
            result: ActionResult::MoveStatus {
                status_applied: vec![],
                target_id: action.pokemon_id,
            },
            critical_hit: false,
            effectiveness: 1.0,
        })
    }

    fn execute_item_action(
        &self,
        action: &TurnAction,
        item_id: u32,
        context: &BattleContext,
    ) -> GameResult<CompletedAction> {
        // 道具使用的实现
        Ok(CompletedAction {
            action: action.clone(),
            success: true,
            result: ActionResult::ItemUsed {
                item_consumed: true,
                effect_description: "道具使用成功".to_string(),
            },
            critical_hit: false,
            effectiveness: 1.0,
        })
    }

    fn execute_switch_action(
        &self,
        action: &TurnAction,
        target_pokemon_id: Uuid,
        context: &BattleContext,
    ) -> GameResult<CompletedAction> {
        // Pokemon切换的实现
        Ok(CompletedAction {
            action: action.clone(),
            success: true,
            result: ActionResult::SwitchSuccessful {
                old_pokemon_id: action.pokemon_id,
                new_pokemon_id: target_pokemon_id,
            },
            critical_hit: false,
            effectiveness: 1.0,
        })
    }

    fn execute_run_action(
        &self,
        action: &TurnAction,
        success_chance: f32,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        let success = rng.probability() < success_chance;
        
        if success {
            Ok(CompletedAction {
                action: action.clone(),
                success: true,
                result: ActionResult::RunSuccessful,
                critical_hit: false,
                effectiveness: 1.0,
            })
        } else {
            Ok(CompletedAction {
                action: action.clone(),
                success: false,
                result: ActionResult::Failed { reason: "逃跑失败".to_string() },
                critical_hit: false,
                effectiveness: 0.0,
            })
        }
    }

    fn execute_struggle_action(
        &self,
        action: &TurnAction,
        context: &BattleContext,
        damage_calculator: &DamageCalculator,
        rng: &mut RandomGenerator,
    ) -> GameResult<CompletedAction> {
        // 挣扎招式的实现
        // 挣扎会对使用者造成最大HP 1/4 的反作用力伤害
        
        Ok(CompletedAction {
            action: action.clone(),
            success: true,
            result: ActionResult::MoveDamage {
                damage: 50, // 固定伤害
                target_id: action.target.as_ref()
                    .and_then(|t| match t {
                        ActionTarget::Opponent(id) => Some(*id),
                        _ => None,
                    })
                    .unwrap_or(action.pokemon_id),
            },
            critical_hit: false,
            effectiveness: 1.0,
        })
    }

    // 辅助方法
    fn find_pokemon_by_id(&self, pokemon_id: Uuid, context: &BattleContext) -> Option<&IndividualPokemon> {
        for participant in &context.participants {
            if let Some(pokemon) = &participant.active_pokemon {
                if pokemon.id == pokemon_id {
                    return Some(pokemon);
                }
            }
            for pokemon in &participant.party {
                if pokemon.id == pokemon_id {
                    return Some(pokemon);
                }
            }
        }
        None
    }

    fn get_move_data(&self, move_id: MoveId) -> GameResult<Move> {
        // 在实际实现中会从数据加载器获取招式数据
        Ok(Move {
            id: move_id,
            name: "Tackle".to_string(),
            move_type: crate::pokemon::types::PokemonType::Normal,
            category: MoveCategory::Physical,
            power: Some(40),
            accuracy: 100,
            pp: 35,
            target: MoveTarget::SingleOpponent,
            priority: 0,
            description: "基础物理攻击".to_string(),
            effects: vec![],
        })
    }

    fn check_move_accuracy(
        &self,
        move_data: &Move,
        user_pokemon: &IndividualPokemon,
        target: Option<&ActionTarget>,
        rng: &mut RandomGenerator,
    ) -> bool {
        if move_data.accuracy == 0 {
            return true; // 必中招式
        }

        let accuracy_chance = move_data.accuracy as f32 / 100.0;
        
        // 考虑命中率和闪避率的等级修正
        // 在实际实现中会计算详细的命中率修正
        
        rng.probability() < accuracy_chance
    }

    fn select_move_targets(
        &self,
        action_target: Option<&ActionTarget>,
        move_target: &MoveTarget,
        context: &BattleContext,
    ) -> GameResult<Vec<BattleTarget>> {
        // 根据招式目标类型选择实际目标
        let mut targets = Vec::new();

        match move_target {
            MoveTarget::SingleOpponent => {
                if let Some(ActionTarget::Opponent(id)) = action_target {
                    targets.push(BattleTarget { pokemon_id: *id });
                }
            },
            MoveTarget::AllOpponents => {
                // 选择所有对手
                for participant in &context.participants {
                    if let Some(pokemon) = &participant.active_pokemon {
                        if pokemon.current_hp > 0 {
                            targets.push(BattleTarget { pokemon_id: pokemon.id });
                        }
                    }
                }
            },
            MoveTarget::Self_ => {
                if let Some(ActionTarget::Self_) = action_target {
                    // 目标是自己
                }
            },
            _ => {
                // 其他目标类型的处理
            },
        }

        Ok(targets)
    }

    fn apply_turn_start_effects(&mut self, context: &BattleContext) -> GameResult<()> {
        // 应用回合开始时的效果，如天气伤害、状态效果等
        Ok(())
    }

    pub fn record_completed_action(&mut self, action: CompletedAction) {
        // 记录完成的行动到历史中
        if let Some(current_record) = self.turn_history.last_mut() {
            if current_record.turn_number == self.current_turn {
                current_record.actions.push(action);
                return;
            }
        }

        // 创建新的回合记录
        let mut new_record = TurnRecord {
            turn_number: self.current_turn,
            actions: vec![action],
            effects_applied: vec![],
            damage_dealt: HashMap::new(),
            status_changes: vec![],
            fainted_pokemon: vec![],
        };

        self.turn_history.push(new_record);
    }
}

#[derive(Debug, Clone)]
struct BattleTarget {
    pokemon_id: Uuid,
}

impl Default for TurnManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_manager_creation() {
        let manager = TurnManager::new();
        assert_eq!(manager.current_turn, 1);
        assert_eq!(manager.current_phase, TurnPhase::SelectActions);
        assert!(manager.action_queue.is_empty());
    }

    #[test]
    fn test_action_priority_sorting() {
        let mut manager = TurnManager::new();
        
        let action1 = TurnAction {
            action_id: Uuid::new_v4(),
            participant_id: Uuid::new_v4(),
            pokemon_id: Uuid::new_v4(),
            action_type: ActionType::UseMove { move_id: 1, move_slot: 0 },
            target: None,
            priority: 0,
            speed: 100,
            selected_at: 1.0,
        };
        
        let action2 = TurnAction {
            action_id: Uuid::new_v4(),
            participant_id: Uuid::new_v4(),
            pokemon_id: Uuid::new_v4(),
            action_type: ActionType::UseItem { item_id: 1 },
            target: None,
            priority: 6,
            speed: 50,
            selected_at: 2.0,
        };
        
        manager.action_queue.push_back(action1);
        manager.action_queue.push_back(action2);
        
        manager.sort_action_queue();
        
        // 道具使用(priority=6)应该排在招式使用(priority=0)前面
        let first_action = manager.action_queue.front().unwrap();
        assert_eq!(first_action.priority, 6);
    }

    #[test]
    fn test_stat_stage_multiplier() {
        assert_eq!(TurnManager::get_stat_stage_multiplier(0), 1.0);
        assert_eq!(TurnManager::get_stat_stage_multiplier(1), 1.5);
        assert_eq!(TurnManager::get_stat_stage_multiplier(-1), 2.0 / 3.0);
        assert_eq!(TurnManager::get_stat_stage_multiplier(6), 4.0);
        assert_eq!(TurnManager::get_stat_stage_multiplier(-6), 0.25);
    }
}