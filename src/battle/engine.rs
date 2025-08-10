// 战斗引擎核心 - 回合制战斗逻辑实现
// 开发心理：严格遵循宝可梦战斗规则，确保公平性和策略性
// 设计原则：状态机模式、可预测性、支持回放

use crate::core::{GameError, Result};
use crate::pokemon::{Pokemon, Move, MoveId};
use crate::battle::{
    BattleAction, BattleParticipant, BattleEnvironment, 
    TurnManager, DamageCalculator, StatusManager, BattleAnimator,
    TurnPhase, DamageResult, SecondaryEffect
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug, warn};

// 战斗状态结构体 (用于引擎内部状态管理)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub battle_type: BattleType,
    pub phase: TurnPhase, 
    pub is_active: bool,
    pub weather: Option<WeatherCondition>,
    pub field_effects: HashMap<String, u8>,
    pub turn_limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleType {
    Single,
    Double,
    Wild,
    Trainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    Rain,
    Sun,
    Sandstorm,
    Hail,
    Fog,
}

#[derive(Debug)]
pub struct BattleEngine {
    state: BattleState,
    participants: Vec<BattleParticipant>,
    environment: BattleEnvironment,
    turn_manager: TurnManager,
    damage_calculator: DamageCalculator,
    status_manager: StatusManager,
    animator: BattleAnimator,
    
    // 战斗统计
    turn_count: u32,
    battle_log: Vec<BattleLogEntry>,
    
    // 性能和调试
    debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleLogEntry {
    pub turn: u32,
    pub phase: TurnPhase,
    pub action: BattleAction,
    pub result: BattleActionResult,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleActionResult {
    Move {
        user_id: u64,
        target_id: u64,
        move_id: MoveId,
        damage: Option<u16>,
        hit: bool,
        critical: bool,
        effectiveness: f32,
    },
    Switch {
        trainer_id: u64,
        old_pokemon: u64,
        new_pokemon: u64,
    },
    UseItem {
        trainer_id: u64,
        item_id: u32,
        target_id: Option<u64>,
        success: bool,
    },
    StatusChange {
        pokemon_id: u64,
        status: String,
        applied: bool,
    },
    Faint {
        pokemon_id: u64,
    },
}

impl BattleEngine {
    pub fn new(
        battle_type: crate::battle::BattleType,
        participants: Vec<BattleParticipant>,
        environment: BattleEnvironment,
        debug_mode: bool,
    ) -> Result<Self> {
        info!("初始化战斗引擎: {:?}", battle_type);
        
        if participants.len() < 2 {
            return Err(GameError::BattleError("至少需要2个参战者".to_string()));
        }
        
        // 验证每个参战者都有可用的宝可梦
        for participant in &participants {
            if participant.team.is_empty() {
                return Err(GameError::BattleError(
                    format!("训练师{}没有可用的宝可梦", participant.trainer_id)
                ));
            }
            
            if !participant.team.iter().any(|id| {
                if let Some(pokemon) = participant.get_pokemon(*id) {
                    !pokemon.is_fainted()
                } else {
                    false
                }
            }) {
                return Err(GameError::BattleError(
                    format!("训练师{}的所有宝可梦都已失去战斗能力", participant.trainer_id)
                ));
            }
        }
        
        let state = BattleState {
            battle_type,
            phase: TurnPhase::ActionSelection,
            is_active: true,
            weather: None,
            field_effects: HashMap::new(),
            turn_limit: None,
        };
        
        Ok(Self {
            state,
            participants,
            environment,
            turn_manager: TurnManager::new(),
            damage_calculator: DamageCalculator::new(),
            status_manager: StatusManager::new(),
            animator: BattleAnimator::new(),
            turn_count: 0,
            battle_log: Vec::new(),
            debug_mode,
        })
    }
    
    // 主要的战斗更新循环
    pub fn update(&mut self, delta_time: std::time::Duration) -> Result<()> {
        if !self.state.is_active {
            return Ok(());
        }
        
        match self.state.phase {
            TurnPhase::ActionSelection => {
                self.handle_action_selection()?;
            },
            TurnPhase::ExecuteActions => {
                self.handle_action_execution()?;
            },
            TurnPhase::EndTurn => {
                self.handle_turn_end()?;
            },
        }
        
        // 检查战斗是否结束
        self.check_battle_end()?;
        
        // 更新动画系统
        self.animator.update(delta_time);
        
        Ok(())
    }
    
    // 处理行动选择阶段
    fn handle_action_selection(&mut self) -> Result<()> {
        if self.turn_manager.all_actions_submitted(&self.participants) {
            info!("所有训练师已选择行动，开始执行");
            self.state.phase = TurnPhase::ExecuteActions;
        }
        
        Ok(())
    }
    
    // 处理行动执行阶段
    fn handle_action_execution(&mut self) -> Result<()> {
        // 获取按速度排序的行动列表
        let actions = self.turn_manager.get_sorted_actions(&self.participants)?;
        
        for (trainer_id, action) in actions {
            if !self.state.is_active {
                break;
            }
            
            let result = self.execute_action(trainer_id, action)?;
            
            // 记录战斗日志
            self.battle_log.push(BattleLogEntry {
                turn: self.turn_count,
                phase: self.state.phase,
                action,
                result,
                timestamp: std::time::SystemTime::now(),
            });
            
            // 检查是否有宝可梦失去战斗能力
            self.handle_faints()?;
        }
        
        self.state.phase = TurnPhase::EndTurn;
        Ok(())
    }
    
    // 处理回合结束阶段
    fn handle_turn_end(&mut self) -> Result<()> {
        // 处理持续性效果（毒、燃烧等）
        self.status_manager.process_end_turn_effects(&mut self.participants)?;
        
        // 应用天气效果
        self.apply_weather_effects()?;
        
        // 应用场地效果
        self.apply_field_effects()?;
        
        // 清理回合
        self.turn_manager.clear_actions();
        self.turn_count += 1;
        self.state.phase = TurnPhase::ActionSelection;
        
        if self.debug_mode {
            debug!("回合{}结束", self.turn_count);
        }
        
        Ok(())
    }
    
    // 执行具体的战斗行动
    fn execute_action(&mut self, trainer_id: u64, action: BattleAction) -> Result<BattleActionResult> {
        match action {
            BattleAction::UseMove { pokemon_index, move_index, target_id } => {
                self.execute_move_action(trainer_id, pokemon_index, move_index, target_id)
            },
            BattleAction::SwitchPokemon { old_index, new_index } => {
                self.execute_switch_action(trainer_id, old_index, new_index)
            },
            BattleAction::UseItem { item_id, target_id } => {
                self.execute_item_action(trainer_id, item_id, target_id)
            },
            BattleAction::Run => {
                self.execute_run_action(trainer_id)
            },
        }
    }
    
    // 执行技能行动
    fn execute_move_action(
        &mut self, 
        trainer_id: u64, 
        pokemon_index: usize, 
        move_index: usize,
        target_id: Option<u64>
    ) -> Result<BattleActionResult> {
        let participant = self.participants.iter_mut()
            .find(|p| p.trainer_id == trainer_id)
            .ok_or_else(|| GameError::BattleError("找不到指定的训练师".to_string()))?;
        
        let pokemon_id = participant.active_pokemon.get(pokemon_index)
            .copied()
            .ok_or_else(|| GameError::BattleError("指定的宝可梦不在场上".to_string()))?;
        
        let pokemon = participant.get_pokemon_mut(pokemon_id)
            .ok_or_else(|| GameError::BattleError("找不到指定的宝可梦".to_string()))?;
        
        if pokemon.is_fainted() {
            return Err(GameError::BattleError("宝可梦已失去战斗能力".to_string()));
        }
        
        if move_index >= pokemon.moves.len() {
            return Err(GameError::BattleError("技能索引无效".to_string()));
        }
        
        let move_slot = &pokemon.moves[move_index];
        if move_slot.current_pp == 0 {
            return Err(GameError::BattleError("技能PP不足".to_string()));
        }
        
        let move_id = move_slot.move_id;
        let move_data = Move::get(move_id)
            .ok_or_else(|| GameError::BattleError("找不到技能数据".to_string()))?;
        
        // 消耗PP
        pokemon.use_move(move_index)?;
        
        // 启动技能动画
        self.animator.start_move_animation(trainer_id, pokemon_index, move_id)?;
        
        info!("{}使用了{}", pokemon.get_display_name(), move_data.name);
        
        // 确定目标
        let target_pokemon_id = self.determine_target(target_id, &participant.team)?;
        
        // 计算伤害
        let damage_result = if let Some(target_id) = target_pokemon_id {
            let target_participant = self.participants.iter()
                .find(|p| p.team.contains(&target_id))
                .ok_or_else(|| GameError::BattleError("找不到目标宝可梦".to_string()))?;
            
            let target_pokemon = target_participant.get_pokemon(target_id)
                .ok_or_else(|| GameError::BattleError("找不到目标宝可梦数据".to_string()))?;
            
            let damage_result = self.damage_calculator.calculate_damage(
                pokemon,
                target_pokemon,
                move_data,
                &self.environment
            )?;
            
            // 应用伤害
            if damage_result.hit && damage_result.damage > 0 {
                let target_participant = self.participants.iter_mut()
                    .find(|p| p.team.contains(&target_id))
                    .unwrap();
                let target_pokemon = target_participant.get_pokemon_mut(target_id).unwrap();
                
                let is_fainted = target_pokemon.take_damage(damage_result.damage);
                if is_fainted {
                    info!("{}失去了战斗能力！", target_pokemon.get_display_name());
                }
            }
            
            Some(damage_result)
        } else {
            None
        };
        
        // 应用附加效果
        if let Some(ref secondary_effect) = move_data.secondary_effect {
            if fastrand::f32() < secondary_effect.chance {
                if let Some(target_id) = target_pokemon_id {
                    self.status_manager.apply_effect(target_id, secondary_effect.clone())?;
                }
            }
        }
        
        Ok(BattleActionResult::Move {
            user_id: pokemon_id,
            target_id: target_pokemon_id.unwrap_or(0),
            move_id,
            damage: damage_result.as_ref().map(|r| r.damage),
            hit: damage_result.as_ref().map_or(true, |r| r.hit),
            critical: damage_result.as_ref().map_or(false, |r| r.critical),
            effectiveness: damage_result.as_ref().map_or(1.0, |r| r.type_effectiveness),
        })
    }
    
    // 执行换宝可梦行动
    fn execute_switch_action(
        &mut self,
        trainer_id: u64,
        old_index: usize,
        new_index: usize,
    ) -> Result<BattleActionResult> {
        let participant = self.participants.iter_mut()
            .find(|p| p.trainer_id == trainer_id)
            .ok_or_else(|| GameError::BattleError("找不到指定的训练师".to_string()))?;
        
        if old_index >= participant.active_pokemon.len() {
            return Err(GameError::BattleError("旧宝可梦索引无效".to_string()));
        }
        
        if new_index >= participant.team.len() {
            return Err(GameError::BattleError("新宝可梦索引无效".to_string()));
        }
        
        let old_pokemon_id = participant.active_pokemon[old_index];
        let new_pokemon_id = participant.team[new_index];
        
        // 检查新宝可梦是否可用
        let new_pokemon = participant.get_pokemon(new_pokemon_id)
            .ok_or_else(|| GameError::BattleError("找不到新宝可梦".to_string()))?;
        
        if new_pokemon.is_fainted() {
            return Err(GameError::BattleError("新宝可梦已失去战斗能力".to_string()));
        }
        
        if participant.active_pokemon.contains(&new_pokemon_id) {
            return Err(GameError::BattleError("新宝可梦已在场上".to_string()));
        }
        
        // 执行切换
        participant.active_pokemon[old_index] = new_pokemon_id;
        
        let old_name = participant.get_pokemon(old_pokemon_id)
            .map(|p| p.get_display_name())
            .unwrap_or_else(|| "未知".to_string());
        let new_name = new_pokemon.get_display_name();
        
        info!("训练师{}将{}换成了{}", trainer_id, old_name, new_name);
        
        Ok(BattleActionResult::Switch {
            trainer_id,
            old_pokemon: old_pokemon_id,
            new_pokemon: new_pokemon_id,
        })
    }
    
    // 执行使用道具行动
    fn execute_item_action(
        &mut self,
        trainer_id: u64,
        item_id: u32,
        target_id: Option<u64>,
    ) -> Result<BattleActionResult> {
        // 简化的道具系统实现
        info!("训练师{}使用了道具{}", trainer_id, item_id);
        
        Ok(BattleActionResult::UseItem {
            trainer_id,
            item_id,
            target_id,
            success: true,
        })
    }
    
    // 执行逃跑行动
    fn execute_run_action(&mut self, trainer_id: u64) -> Result<BattleActionResult> {
        // 在野生宝可梦战斗中可以逃跑
        if matches!(self.state.battle_type, crate::battle::BattleType::Wild) {
            info!("训练师{}逃跑了", trainer_id);
            self.state.is_active = false;
        }
        
        Ok(BattleActionResult::UseItem {
            trainer_id,
            item_id: 0,
            target_id: None,
            success: true,
        })
    }
    
    // 确定技能目标
    fn determine_target(&self, target_id: Option<u64>, user_team: &[u64]) -> Result<Option<u64>> {
        if let Some(id) = target_id {
            return Ok(Some(id));
        }
        
        // 自动选择第一个敌方宝可梦作为目标
        for participant in &self.participants {
            if participant.team != user_team && !participant.active_pokemon.is_empty() {
                return Ok(Some(participant.active_pokemon[0]));
            }
        }
        
        Ok(None)
    }
    
    // 处理宝可梦失去战斗能力
    fn handle_faints(&mut self) -> Result<()> {
        for participant in &mut self.participants {
            let mut to_remove = Vec::new();
            
            for (index, &pokemon_id) in participant.active_pokemon.iter().enumerate() {
                if let Some(pokemon) = participant.get_pokemon(pokemon_id) {
                    if pokemon.is_fainted() {
                        to_remove.push(index);
                        
                        // 记录失去战斗能力
                        self.battle_log.push(BattleLogEntry {
                            turn: self.turn_count,
                            phase: self.state.phase,
                            action: BattleAction::Run, // 临时用Run表示
                            result: BattleActionResult::Faint {
                                pokemon_id,
                            },
                            timestamp: std::time::SystemTime::now(),
                        });
                    }
                }
            }
            
            // 移除失去战斗能力的宝可梦
            for &index in to_remove.iter().rev() {
                participant.active_pokemon.remove(index);
            }
        }
        
        Ok(())
    }
    
    // 应用天气效果
    fn apply_weather_effects(&mut self) -> Result<()> {
        if let Some(_weather) = &self.state.weather {
            // TODO: 实现天气效果
        }
        Ok(())
    }
    
    // 应用场地效果
    fn apply_field_effects(&mut self) -> Result<()> {
        // TODO: 实现场地效果
        Ok(())
    }
    
    // 检查战斗是否结束
    fn check_battle_end(&mut self) -> Result<()> {
        let mut active_participants = 0;
        
        for participant in &self.participants {
            let has_active_pokemon = participant.team.iter().any(|&id| {
                if let Some(pokemon) = participant.get_pokemon(id) {
                    !pokemon.is_fainted()
                } else {
                    false
                }
            });
            
            if has_active_pokemon {
                active_participants += 1;
            }
        }
        
        if active_participants <= 1 {
            self.state.is_active = false;
            info!("战斗结束！剩余参战者: {}", active_participants);
        }
        
        Ok(())
    }
    
    // 添加行动到当前回合
    pub fn add_action(&mut self, trainer_id: u64, action: BattleAction) -> Result<()> {
        if self.state.phase != TurnPhase::ActionSelection {
            return Err(GameError::BattleError("当前阶段不接受行动输入".to_string()));
        }
        
        self.turn_manager.add_action(trainer_id, action)
    }
    
    // 获取当前战斗状态
    pub fn get_state(&self) -> &BattleState {
        &self.state
    }
    
    // 获取参战者信息
    pub fn get_participants(&self) -> &[BattleParticipant] {
        &self.participants
    }
    
    // 获取战斗日志
    pub fn get_battle_log(&self) -> &[BattleLogEntry] {
        &self.battle_log
    }
    
    // 检查战斗是否仍然活跃
    pub fn is_active(&self) -> bool {
        self.state.is_active
    }
    
    // 获取当前回合数
    pub fn get_turn_count(&self) -> u32 {
        self.turn_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::BattleType;
    
    #[test]
    fn test_battle_engine_creation() {
        // 这个测试需要更完整的宝可梦数据才能运行
        // 现在只测试基础结构
    }
}