// 战斗系统模块 - 宝可梦对战核心机制
// 开发心理：战斗是宝可梦游戏的核心体验，需要精确的回合制逻辑和丰富的策略性
// 设计原则：状态机驱动、事件响应式、可扩展的技能系统、公平的随机性

// 逐步实现子模块
pub mod engine;
pub mod turn_manager;
pub mod damage_calculator;
// pub mod status_effects;
// pub mod animation;

// 重新导出已实现的类型
pub use engine::{BattleEngine, BattleLogEntry, BattleActionResult};
pub use turn_manager::{TurnManager as NewTurnManager, BattleAction, ActionResult, TurnResult, ParticipantId};
pub use damage_calculator::{DamageCalculator as NewDamageCalculator, DamageResult as NewDamageResult, DamageContext};
// pub use status_effects::{StatusEffect, StatusManager, EffectTrigger};
// pub use animation::{BattleAnimator, AnimationType, AnimationQueue};

use crate::core::{GameError, Result};
use crate::pokemon::{Pokemon, Move, MoveId};
use crate::core::event_system::{Event, EventSystem};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use log::{info, debug, warn};

// 临时类型定义，避免编译错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    ActionSelection,
    ExecuteActions,
    EndTurn,
}

pub struct TurnManager;
pub struct DamageCalculator;
pub struct StatusManager;
pub struct BattleAnimator;

// 临时结构定义
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage: u16,
    pub is_critical: bool,
    pub effectiveness: f32,
}

#[derive(Debug, Clone)]
pub struct SecondaryEffect {
    pub effect_type: String,
    pub chance: f32,
}

impl TurnManager {
    pub fn new() -> Self { Self }
    pub fn add_action(&mut self, _trainer_id: u64, _action: BattleAction) -> Result<()> { Ok(()) }
    pub fn all_actions_submitted(&self, _participants: &[BattleParticipant]) -> bool { true }
    pub fn get_sorted_actions(&self, _participants: &[BattleParticipant]) -> Result<Vec<(u64, BattleAction)>> { Ok(vec![]) }
    pub fn clear_actions(&mut self) {}
}

impl DamageCalculator {
    pub fn new() -> Self { Self }
    pub fn calculate_damage(
        &self, 
        _user: &Pokemon, 
        _target: &Pokemon, 
        _move_data: &Move, 
        _env: &BattleEnvironment
    ) -> Result<DamageResult> {
        Ok(DamageResult {
            damage: 50,
            hit: true,
            critical: false,
            type_effectiveness: 1.0,
        })
    }
}

// DamageResult重复定义已移除，使用第一个定义

impl StatusManager {
    pub fn new() -> Self { Self }
    pub fn apply_effect(&mut self, _target_id: u64, _effect: SecondaryEffect) -> Result<()> { Ok(()) }
    pub fn process_end_turn_effects(&mut self, _participants: &mut [BattleParticipant]) -> Result<()> { Ok(()) }
}

// SecondaryEffect重复定义已移除，使用第一个定义

impl BattleAnimator {
    pub fn new() -> Self { Self }
    pub fn start_move_animation(&mut self, _trainer_id: u64, _pokemon_index: usize, _move_id: MoveId) -> Result<()> { Ok(()) }
}

// 战斗类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleType {
    Single,      // 1vs1
    Double,      // 2vs2
    Multi,       // 2vs2 with 4 trainers
    Horde,       // 1vs5
    Raid,        // Multiple vs 1 boss
}

// 战斗格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleFormat {
    Wild,        // 野生宝可梦
    Trainer,     // 训练师对战
    Gym,         // 道馆挑战
    Elite4,      // 四天王
    Champion,    // 冠军赛
    Tournament,  // 锦标赛
    Online,      // 在线对战
}

// 战斗状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStatus {
    Initializing,
    WaitingForAction,
    ProcessingTurn,
    AnimatingMove,
    CheckingFaint,
    SwitchingPokemon,
    BattleEnd,
}

// 新的战斗状态结构（用于turn_manager）
#[derive(Debug, Clone)]
pub struct BattleState {
    participants: Vec<BattleParticipant>,
    battle_ended: bool,
}

impl BattleState {
    pub fn new(participants: Vec<BattleParticipant>) -> Self {
        Self {
            participants,
            battle_ended: false,
        }
    }
    
    pub fn get_participant_count(&self) -> usize {
        self.participants.len()
    }
    
    pub fn get_participants(&self) -> &[BattleParticipant] {
        &self.participants
    }
    
    pub fn get_active_pokemon(&self, participant_id: usize) -> Option<&Pokemon> {
        self.participants.get(participant_id)?
            .active_pokemon.get(0)?
            .map(|&idx| &self.participants[participant_id].pokemon[idx])
    }
    
    pub fn get_active_pokemon_mut(&mut self, participant_id: usize) -> Option<&mut Pokemon> {
        let active_idx = self.participants.get(participant_id)?
            .active_pokemon.get(0)?
            .clone()?;
        Some(&mut self.participants[participant_id].pokemon[active_idx])
    }
    
    pub fn get_active_pokemon_index(&self, participant_id: usize) -> usize {
        self.participants.get(participant_id)
            .and_then(|p| p.active_pokemon.get(0))
            .and_then(|&idx| idx)
            .unwrap_or(0)
    }
    
    pub fn switch_pokemon(&mut self, participant_id: usize, new_index: usize) -> Result<bool> {
        if let Some(participant) = self.participants.get_mut(participant_id) {
            if new_index < participant.pokemon.len() && !participant.pokemon[new_index].is_fainted() {
                if let Some(active_slot) = participant.active_pokemon.get_mut(0) {
                    *active_slot = Some(new_index);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    
    pub fn is_battle_ended(&self) -> bool {
        self.battle_ended
    }
    
    pub fn set_battle_ended(&mut self, ended: bool) {
        self.battle_ended = ended;
    }
}

// 战斗参与者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleParticipant {
    pub trainer_id: u64,
    pub trainer_name: String,
    pub pokemon: Vec<Pokemon>, // 原名为team，但为兼容性改为pokemon
    pub active_pokemon_index: usize,
    pub active_pokemon: Vec<Option<usize>>, // 场上宝可梦索引
    pub is_ai: bool,
    pub ai_difficulty: AIDifficulty,
}

impl BattleParticipant {
    pub fn new(pokemon: Vec<Pokemon>) -> Self {
        Self {
            trainer_id: fastrand::u64(1..),
            trainer_name: "Trainer".to_string(),
            pokemon,
            active_pokemon_index: 0,
            active_pokemon: vec![Some(0)],
            is_ai: false,
            ai_difficulty: AIDifficulty::Normal,
        }
    }
    
    pub fn team(&self) -> &[Pokemon] {
        &self.pokemon
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

// 战斗行动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleAction {
    UseMove {
        pokemon_index: usize,
        move_index: usize,
        target: BattleTarget,
    },
    SwitchPokemon {
        from_index: usize,
        to_index: usize,
    },
    UseItem {
        item_id: u32,
        target: Option<usize>,
    },
    Run,
    Forfeit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleTarget {
    Self_,
    Opponent(usize),
    Ally(usize),
    All,
    AllOpponents,
    AllAllies,
    Random,
    User,
}

// 战斗事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleTurnStartEvent {
    pub turn_number: u32,
    pub phase: TurnPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonMoveEvent {
    pub user_id: u64,
    pub pokemon_index: usize,
    pub move_id: MoveId,
    pub target: BattleTarget,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageDealtEvent {
    pub attacker_id: u64,
    pub defender_id: u64,
    pub damage: u16,
    pub critical_hit: bool,
    pub type_effectiveness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonFaintedEvent {
    pub trainer_id: u64,
    pub pokemon_index: usize,
    pub pokemon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEndEvent {
    pub winner_id: Option<u64>,
    pub battle_type: BattleType,
    pub total_turns: u32,
    pub duration: Duration,
}

// 实现Event特征
impl Event for BattleTurnStartEvent {
    fn event_type(&self) -> &'static str { "BattleTurnStart" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for PokemonMoveEvent {
    fn event_type(&self) -> &'static str { "PokemonMove" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for DamageDealtEvent {
    fn event_type(&self) -> &'static str { "DamageDealt" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for PokemonFaintedEvent {
    fn event_type(&self) -> &'static str { "PokemonFainted" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl Event for BattleEndEvent {
    fn event_type(&self) -> &'static str { "BattleEnd" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// 战斗环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEnvironment {
    pub weather: Option<crate::pokemon::moves::WeatherType>,
    pub weather_turns: Option<u8>,
    pub terrain: TerrainType,
    pub field_effects: Vec<FieldEffect>,
    pub trick_room: bool,
    pub gravity: bool,
    pub magic_room: bool,
    pub wonder_room: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    None,
    Sun,
    Rain,
    Sandstorm,
    Hail,
    Fog,
    HarshSun,    // 大晴天
    HeavyRain,   // 大雨
    StrongWinds, // 乱流
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    None,
    Electric,    // 电气场地
    Grassy,      // 青草场地
    Misty,       // 薄雾场地
    Psychic,     // 精神场地
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEffect {
    pub effect_type: FieldEffectType,
    pub duration: u8,
    pub source: Option<u64>, // 施展者ID
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldEffectType {
    Spikes,           // 撒菱
    ToxicSpikes,      // 毒菱
    StealthRock,      // 隐形岩
    StickyWeb,        // 蛛网
    LightScreen,      // 光之壁
    Reflect,          // 反射壁
    Aurora_Veil,      // 极光幕
    Tailwind,         // 顺风
    TrickRoom,        // 戏法空间
    WonderRoom,       // 奇迹空间
    MagicRoom,        // 魔法空间
}

impl Default for BattleEnvironment {
    fn default() -> Self {
        Self {
            weather: None,
            weather_turns: None,
            terrain: TerrainType::None,
            field_effects: Vec::new(),
            trick_room: false,
            gravity: false,
            magic_room: false,
            wonder_room: false,
        }
    }
}

// 战斗配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleConfig {
    pub battle_type: BattleType,
    pub battle_format: BattleFormat,
    pub time_limit_seconds: Option<u32>,
    pub level_cap: Option<u8>,
    pub item_clause: bool,
    pub sleep_clause: bool,
    pub species_clause: bool,
    pub enable_z_moves: bool,
    pub enable_mega_evolution: bool,
    pub enable_dynamax: bool,
    pub terrain_turns: u8,
    pub weather_turns: u8,
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            battle_type: BattleType::Single,
            battle_format: BattleFormat::Trainer,
            time_limit_seconds: Some(300), // 5分钟
            level_cap: None,
            item_clause: false,
            sleep_clause: false,
            species_clause: false,
            enable_z_moves: true,
            enable_mega_evolution: true,
            enable_dynamax: true,
            terrain_turns: 5,
            weather_turns: 5,
        }
    }
}

// 战斗上下文
pub struct BattleContext {
    pub battle_id: u64,
    pub config: BattleConfig,
    pub participants: Vec<BattleParticipant>,
    pub environment: BattleEnvironment,
    pub state: BattleStatus,
    
    pub turn_number: u32,
    pub start_time: Instant,
    pub last_action_time: Instant,
    
    // 战斗统计
    pub stats: BattleStats,
    
    // 子系统
    pub turn_manager: TurnManager,
    pub damage_calculator: DamageCalculator,
    pub status_manager: StatusManager,
    pub animator: BattleAnimator,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BattleStats {
    pub total_damage_dealt: HashMap<u64, u32>,
    pub moves_used: HashMap<MoveId, u32>,
    pub critical_hits: u32,
    pub status_conditions_applied: u32,
    pub pokemon_fainted: u32,
    pub switches_made: u32,
    pub items_used: u32,
}

impl BattleContext {
    pub fn new(
        battle_id: u64,
        config: BattleConfig,
        participants: Vec<BattleParticipant>,
    ) -> Result<Self> {
        if participants.len() < 2 {
            return Err(GameError::BattleError("至少需要两个参与者".to_string()));
        }
        
        // 验证参与者队伍
        for participant in &participants {
            if participant.pokemon.is_empty() {
                return Err(GameError::BattleError("参与者队伍不能为空".to_string()));
            }
        }
        
        Ok(Self {
            battle_id,
            config,
            participants,
            environment: BattleEnvironment::default(),
            state: BattleStatus::Initializing,
            
            turn_number: 0,
            start_time: Instant::now(),
            last_action_time: Instant::now(),
            
            stats: BattleStats::default(),
            
            turn_manager: TurnManager::new(),
            damage_calculator: DamageCalculator::new(),
            status_manager: StatusManager::new(),
            animator: BattleAnimator::new(),
        })
    }
    
    // 开始战斗
    pub fn start_battle(&mut self) -> Result<()> {
        info!("开始战斗 #{}", self.battle_id);
        
        // 初始化参与者的活跃宝可梦
        for participant in &mut self.participants {
            let active_count = match self.config.battle_type {
                BattleType::Single => 1,
                BattleType::Double => 2,
                _ => 1,
            };
            
            participant.active_pokemon = vec![None; active_count];
            
            // 自动选择前几只健康的宝可梦上场
            let mut active_index = 0;
            for (i, pokemon) in participant.pokemon.iter().enumerate() {
                if !pokemon.is_fainted() && active_index < active_count {
                    participant.active_pokemon[active_index] = Some(i);
                    active_index += 1;
                }
            }
        }
        
        self.state = BattleStatus::WaitingForAction;
        self.turn_number = 1;
        
        // 发送战斗开始事件
        EventSystem::dispatch(BattleTurnStartEvent {
            turn_number: self.turn_number,
            phase: TurnPhase::ActionSelection,
        })?;
        
        Ok(())
    }
    
    // 提交行动
    pub fn submit_action(&mut self, trainer_id: u64, action: BattleAction) -> Result<()> {
        if self.state != BattleStatus::WaitingForAction {
            return Err(GameError::BattleError("当前不是行动选择阶段".to_string()));
        }
        
        // 验证行动合法性
        self.validate_action(trainer_id, &action)?;
        
        // 添加到行动队列
        self.turn_manager.add_action(trainer_id, action)?;
        
        // 检查是否所有参与者都提交了行动
        if self.turn_manager.all_actions_submitted(&self.participants) {
            self.process_turn()?;
        }
        
        Ok(())
    }
    
    // 处理回合
    fn process_turn(&mut self) -> Result<()> {
        self.state = BattleStatus::ProcessingTurn;
        
        debug!("处理回合 #{}", self.turn_number);
        
        // 按优先级排序行动
        let actions = self.turn_manager.get_sorted_actions(&self.participants)?;
        
        // 执行每个行动
        for (trainer_id, action) in actions {
            if self.is_battle_ended() {
                break;
            }
            
            self.execute_action(trainer_id, action)?;
        }
        
        // 回合结束处理
        self.end_turn_effects()?;
        
        // 检查战斗是否结束
        if self.is_battle_ended() {
            self.end_battle()?;
        } else {
            // 准备下一回合
            self.turn_number += 1;
            self.state = BattleStatus::WaitingForAction;
            self.turn_manager.clear_actions();
            
            EventSystem::dispatch(BattleTurnStartEvent {
                turn_number: self.turn_number,
                phase: TurnPhase::ActionSelection,
            })?;
        }
        
        Ok(())
    }
    
    // 执行行动
    fn execute_action(&mut self, trainer_id: u64, action: BattleAction) -> Result<()> {
        match action {
            BattleAction::UseMove { pokemon_index, move_index, target } => {
                self.execute_move(trainer_id, pokemon_index, move_index, target)?;
            },
            BattleAction::SwitchPokemon { from_index, to_index } => {
                self.execute_switch(trainer_id, from_index, to_index)?;
            },
            BattleAction::UseItem { item_id, target } => {
                self.execute_item_use(trainer_id, item_id, target)?;
            },
            BattleAction::Run => {
                self.execute_run(trainer_id)?;
            },
            BattleAction::Forfeit => {
                self.execute_forfeit(trainer_id)?;
            },
        }
        
        Ok(())
    }
    
    // 执行技能使用
    fn execute_move(
        &mut self,
        trainer_id: u64,
        pokemon_index: usize,
        move_index: usize,
        target: BattleTarget,
    ) -> Result<()> {
        // 获取使用者信息
        let participant = self.get_participant_mut(trainer_id)?;
        let active_slot = participant.active_pokemon
            .iter()
            .position(|&slot| slot == Some(pokemon_index))
            .ok_or_else(|| GameError::BattleError("宝可梦不在场上".to_string()))?;
        
        let pokemon = &mut participant.pokemon[pokemon_index];
        
        // 检查宝可梦状态
        if pokemon.is_fainted() {
            return Err(GameError::BattleError("濒死的宝可梦无法使用技能".to_string()));
        }
        
        if move_index >= pokemon.moves.len() {
            return Err(GameError::BattleError("无效的技能索引".to_string()));
        }
        
        let move_slot = &mut pokemon.moves[move_index];
        if move_slot.current_pp == 0 {
            return Err(GameError::BattleError("技能PP不足".to_string()));
        }
        
        // 获取技能信息
        let move_data = crate::pokemon::Move::get(move_slot.move_id)
            .ok_or_else(|| GameError::BattleError("技能数据不存在".to_string()))?;
        
        // 消耗PP
        move_slot.current_pp -= 1;
        
        // 动画开始
        self.state = BattleStatus::AnimatingMove;
        self.animator.start_move_animation(trainer_id, pokemon_index, move_slot.move_id)?;
        
        // 计算伤害和效果
        let targets = self.resolve_targets(trainer_id, target)?;
        let mut move_success = false;
        
        for target_id in targets {
            let damage_result = self.damage_calculator.calculate_damage(
                pokemon,
                &self.get_target_pokemon(target_id)?,
                move_data,
                &self.environment,
            )?;
            
            if damage_result.hit {
                move_success = true;
                
                // 应用伤害
                self.apply_damage(target_id, damage_result.damage)?;
                
                // 发送伤害事件
                EventSystem::dispatch(DamageDealtEvent {
                    attacker_id: trainer_id,
                    defender_id: target_id,
                    damage: damage_result.damage,
                    critical_hit: damage_result.critical,
                    type_effectiveness: damage_result.type_effectiveness,
                })?;
                
                // 应用附加效果
                if let Some(effect) = &move_data.secondary_effect {
                    if fastrand::f32() < effect.chance {
                        self.status_manager.apply_effect(target_id, effect.clone())?;
                    }
                }
                
                // 更新统计
                self.stats.total_damage_dealt
                    .entry(trainer_id)
                    .and_modify(|d| *d += damage_result.damage as u32)
                    .or_insert(damage_result.damage as u32);
                
                if damage_result.critical {
                    self.stats.critical_hits += 1;
                }
            }
        }
        
        // 更新技能使用统计
        self.stats.moves_used
            .entry(move_slot.move_id)
            .and_modify(|c| *c += 1)
            .or_insert(1);
        
        // 发送技能使用事件
        EventSystem::dispatch(PokemonMoveEvent {
            user_id: trainer_id,
            pokemon_index,
            move_id: move_slot.move_id,
            target,
            success: move_success,
        })?;
        
        // 检查濒死
        self.check_and_handle_faints()?;
        
        Ok(())
    }
    
    // 执行宝可梦切换
    fn execute_switch(&mut self, trainer_id: u64, from_index: usize, to_index: usize) -> Result<()> {
        let participant = self.get_participant_mut(trainer_id)?;
        
        // 验证切换的合法性
        if participant.pokemon[to_index].is_fainted() {
            return Err(GameError::BattleError("无法切换到濒死的宝可梦".to_string()));
        }
        
        // 执行切换
        for active_slot in &mut participant.active_pokemon {
            if *active_slot == Some(from_index) {
                *active_slot = Some(to_index);
                break;
            }
        }
        
        self.stats.switches_made += 1;
        
        info!("{}切换宝可梦: {} -> {}", 
              participant.trainer_name,
              participant.pokemon[from_index].get_display_name(),
              participant.pokemon[to_index].get_display_name());
        
        Ok(())
    }
    
    // 执行道具使用
    fn execute_item_use(&mut self, trainer_id: u64, item_id: u32, target: Option<usize>) -> Result<()> {
        // TODO: 实现道具使用逻辑
        self.stats.items_used += 1;
        debug!("训练师 {} 使用道具 {}", trainer_id, item_id);
        Ok(())
    }
    
    // 执行逃跑
    fn execute_run(&mut self, trainer_id: u64) -> Result<()> {
        if self.config.battle_format != BattleFormat::Wild {
            return Err(GameError::BattleError("无法从训练师对战中逃跑".to_string()));
        }
        
        // 计算逃跑成功率
        let escape_chance = self.calculate_escape_chance(trainer_id)?;
        
        if fastrand::f32() < escape_chance {
            info!("逃跑成功!");
            self.end_battle_with_result(None)?;
        } else {
            info!("逃跑失败!");
        }
        
        Ok(())
    }
    
    // 执行认输
    fn execute_forfeit(&mut self, trainer_id: u64) -> Result<()> {
        info!("训练师 {} 认输", trainer_id);
        
        // 找到获胜者
        let winner_id = self.participants
            .iter()
            .find(|p| p.trainer_id != trainer_id)
            .map(|p| p.trainer_id);
        
        self.end_battle_with_result(winner_id)?;
        Ok(())
    }
    
    // 辅助方法
    fn validate_action(&self, trainer_id: u64, action: &BattleAction) -> Result<()> {
        let participant = self.get_participant(trainer_id)?;
        
        match action {
            BattleAction::UseMove { pokemon_index, move_index, .. } => {
                if *pokemon_index >= participant.pokemon.len() {
                    return Err(GameError::BattleError("无效的宝可梦索引".to_string()));
                }
                
                let pokemon = &participant.pokemon[*pokemon_index];
                if pokemon.is_fainted() {
                    return Err(GameError::BattleError("濒死宝可梦无法行动".to_string()));
                }
                
                if *move_index >= pokemon.moves.len() {
                    return Err(GameError::BattleError("无效的技能索引".to_string()));
                }
                
                if pokemon.moves[*move_index].current_pp == 0 {
                    return Err(GameError::BattleError("技能PP不足".to_string()));
                }
            },
            BattleAction::SwitchPokemon { to_index, .. } => {
                if *to_index >= participant.pokemon.len() {
                    return Err(GameError::BattleError("无效的宝可梦索引".to_string()));
                }
                
                if participant.pokemon[*to_index].is_fainted() {
                    return Err(GameError::BattleError("无法切换到濒死宝可梦".to_string()));
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn get_participant(&self, trainer_id: u64) -> Result<&BattleParticipant> {
        self.participants
            .iter()
            .find(|p| p.trainer_id == trainer_id)
            .ok_or_else(|| GameError::BattleError("参与者不存在".to_string()))
    }
    
    fn get_participant_mut(&mut self, trainer_id: u64) -> Result<&mut BattleParticipant> {
        self.participants
            .iter_mut()
            .find(|p| p.trainer_id == trainer_id)
            .ok_or_else(|| GameError::BattleError("参与者不存在".to_string()))
    }
    
    fn resolve_targets(&self, user_id: u64, target: BattleTarget) -> Result<Vec<u64>> {
        // TODO: 实现目标解析逻辑
        match target {
            BattleTarget::Opponent(_) => {
                // 返回对手ID
                Ok(self.participants
                    .iter()
                    .filter(|p| p.trainer_id != user_id)
                    .map(|p| p.trainer_id)
                    .collect())
            },
            _ => Ok(vec![user_id]),
        }
    }
    
    fn get_target_pokemon(&self, target_id: u64) -> Result<&Pokemon> {
        let participant = self.get_participant(target_id)?;
        let active_index = participant.active_pokemon[0]
            .ok_or_else(|| GameError::BattleError("目标没有活跃宝可梦".to_string()))?;
        Ok(&participant.pokemon[active_index])
    }
    
    fn apply_damage(&mut self, target_id: u64, damage: u16) -> Result<()> {
        let participant = self.get_participant_mut(target_id)?;
        let active_index = participant.active_pokemon[0]
            .ok_or_else(|| GameError::BattleError("目标没有活跃宝可梦".to_string()))?;
        
        let pokemon = &mut participant.pokemon[active_index];
        let fainted = pokemon.take_damage(damage);
        
        if fainted {
            EventSystem::dispatch(PokemonFaintedEvent {
                trainer_id: target_id,
                pokemon_index: active_index,
                pokemon_name: pokemon.get_display_name(),
            })?;
            
            self.stats.pokemon_fainted += 1;
        }
        
        Ok(())
    }
    
    fn check_and_handle_faints(&mut self) -> Result<()> {
        self.state = BattleStatus::CheckingFaint;
        
        for participant in &mut self.participants {
            for (i, active_slot) in participant.active_pokemon.iter_mut().enumerate() {
                if let Some(pokemon_index) = *active_slot {
                    if participant.pokemon[pokemon_index].is_fainted() {
                        *active_slot = None;
                        
                        // 寻找替补宝可梦
                        let replacement = participant.team
                            .iter()
                            .enumerate()
                            .find(|(_, p)| !p.is_fainted())
                            .map(|(idx, _)| idx);
                        
                        if let Some(new_index) = replacement {
                            *active_slot = Some(new_index);
                            self.state = BattleStatus::SwitchingPokemon;
                            info!("自动切换宝可梦: {}", participant.pokemon[new_index].get_display_name());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn end_turn_effects(&mut self) -> Result<()> {
        // 处理天气伤害
        self.apply_weather_effects()?;
        
        // 处理状态异常
        self.status_manager.process_end_turn_effects(&mut self.participants)?;
        
        // 处理场地效果
        self.process_field_effects()?;
        
        // 更新环境效果持续时间
        self.update_environment_durations();
        
        Ok(())
    }
    
    fn apply_weather_effects(&mut self) -> Result<()> {
        match self.environment.weather {
            WeatherCondition::Sandstorm => {
                // 沙暴伤害
                for participant in &mut self.participants {
                    for &active_index in &participant.active_pokemon {
                        if let Some(pokemon_index) = active_index {
                            let pokemon = &mut participant.pokemon[pokemon_index];
                            if !pokemon.get_species().unwrap().types.contains(&crate::pokemon::PokemonType::Rock) &&
                               !pokemon.get_species().unwrap().types.contains(&crate::pokemon::PokemonType::Ground) &&
                               !pokemon.get_species().unwrap().types.contains(&crate::pokemon::PokemonType::Steel) {
                                let damage = pokemon.get_stats().unwrap().hp / 16;
                                pokemon.take_damage(damage);
                                debug!("{} 受到沙暴伤害: {}", pokemon.get_display_name(), damage);
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn process_field_effects(&mut self) -> Result<()> {
        // TODO: 实现场地效果处理
        Ok(())
    }
    
    fn update_environment_durations(&mut self) {
        // 更新场地效果持续时间
        self.environment.field_effects.retain_mut(|effect| {
            effect.duration = effect.duration.saturating_sub(1);
            effect.duration > 0
        });
    }
    
    fn is_battle_ended(&self) -> bool {
        // 检查是否有参与者失去所有宝可梦
        self.participants.iter().any(|p| {
            p.pokemon.iter().all(|pokemon| pokemon.is_fainted())
        })
    }
    
    fn end_battle(&mut self) -> Result<()> {
        let winner_id = self.participants
            .iter()
            .find(|p| p.pokemon.iter().any(|pokemon| !pokemon.is_fainted()))
            .map(|p| p.trainer_id);
        
        self.end_battle_with_result(winner_id)
    }
    
    fn end_battle_with_result(&mut self, winner_id: Option<u64>) -> Result<()> {
        self.state = BattleStatus::BattleEnd;
        let duration = self.start_time.elapsed();
        
        info!("战斗结束! 获胜者: {:?}, 持续时间: {:?}", winner_id, duration);
        
        EventSystem::dispatch(BattleEndEvent {
            winner_id,
            battle_type: self.config.battle_type,
            total_turns: self.turn_number,
            duration,
        })?;
        
        Ok(())
    }
    
    fn calculate_escape_chance(&self, trainer_id: u64) -> Result<f32> {
        // 简单的逃跑成功率计算
        let participant = self.get_participant(trainer_id)?;
        let active_index = participant.active_pokemon[0]
            .ok_or_else(|| GameError::BattleError("没有活跃宝可梦".to_string()))?;
        
        let player_speed = participant.pokemon[active_index].get_stats()?.speed;
        
        // 基础逃跑率，可以根据速度、等级等调整
        let base_chance = 0.5f32;
        let speed_bonus = (player_speed as f32 / 200.0).min(0.3);
        
        Ok((base_chance + speed_bonus).min(0.95))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battle_context_creation() {
        let config = BattleConfig::default();
        
        // 这里需要创建测试用的参与者
        let participants = vec![
            // TODO: 创建测试参与者
        ];
        
        if !participants.is_empty() {
            let battle = BattleContext::new(1, config, participants);
            assert!(battle.is_ok());
        }
    }
    
    #[test]
    fn test_battle_target_resolution() {
        // TODO: 测试目标解析逻辑
    }
    
    #[test]
    fn test_action_validation() {
        // TODO: 测试行动验证
    }
}