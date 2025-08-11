/*
* 开发心理过程：
* 1. 设计分层的AI决策系统，从新手到专家不同难度级别
* 2. 实现基于评分的招式选择算法，考虑威力、命中、效果等因素
* 3. 支持Pokemon换位策略，分析优劣势匹配
* 4. 集成预测系统，模拟未来几回合的战斗走向
* 5. 实现学习机制，AI可以从失败中改进策略
* 6. 提供可配置的AI个性和风格系统
* 7. 优化性能，支持大量AI同时运算而不影响游戏流畅度
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        individual::{IndividualPokemon, StatusType},
        moves::{Move, MoveId, MoveCategory},
        types::{PokemonType, TypeEffectiveness},
        stats::StatType,
        species::PokemonSpecies,
    },
    battle::{
        engine::{BattleContext, BattleParticipant},
        turn::{TurnAction, ActionType, ActionTarget, TurnManager},
        damage::{DamageCalculator, WeatherCondition},
        effects::{EffectProcessor, FieldEffect},
    },
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone)]
pub struct BattleAI {
    /// AI配置
    pub config: AIConfig,
    /// 决策历史
    pub decision_history: Vec<AIDecision>,
    /// 学习数据
    pub learning_data: AILearningData,
    /// 招式评价缓存
    move_cache: HashMap<MoveEvaluationKey, f32>,
}

#[derive(Debug, Clone)]
pub struct AIConfig {
    /// AI难度级别
    pub difficulty: AIDifficulty,
    /// AI个性
    pub personality: AIPersonality,
    /// 随机性程度 (0.0 = 完全理性，1.0 = 完全随机)
    pub randomness: f32,
    /// 前瞻深度（模拟未来回合数）
    pub lookahead_depth: u8,
    /// 是否启用学习
    pub enable_learning: bool,
    /// 决策时间限制（毫秒）
    pub time_limit_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIDifficulty {
    Beginner,    // 新手：基础策略，偶有失误
    Novice,      // 初学者：合理选择，较少失误
    Intermediate,// 中级：良好判断，考虑类型相克
    Advanced,    // 高级：深度分析，预测对手
    Expert,      // 专家：完美决策，复杂策略
    Master,      // 大师：人类顶级水平
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIPersonality {
    Aggressive,   // 激进：偏爱高伤害招式
    Defensive,    // 防守：优先防御和恢复
    Balanced,     // 平衡：综合考虑各种因素
    Tactical,     // 战术：重视状态和场地控制
    Unpredictable,// 不可预测：行为较为随机
    Analytical,   // 分析型：基于数据做决策
}

#[derive(Debug, Clone)]
pub struct AIDecision {
    pub turn: u32,
    pub situation: BattleSituation,
    pub action: TurnAction,
    pub evaluation_score: f32,
    pub reasoning: String,
    pub alternative_actions: Vec<(TurnAction, f32)>, // 其他考虑的行动及其评分
}

#[derive(Debug, Clone)]
pub struct AILearningData {
    /// 招式使用成功率统计
    pub move_success_rates: HashMap<MoveId, (u32, u32)>, // (成功次数, 总次数)
    /// 换Pokemon策略效果
    pub switch_effectiveness: HashMap<String, f32>,
    /// 对手行为模式学习
    pub opponent_patterns: HashMap<String, f32>,
    /// 总战斗统计
    pub battle_stats: BattleStats,
}

#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    pub total_battles: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub average_turns: f32,
}

#[derive(Debug, Clone)]
pub struct BattleSituation {
    pub my_pokemon: PokemonState,
    pub opponent_pokemon: PokemonState,
    pub field_conditions: Vec<String>,
    pub weather: WeatherCondition,
    pub turn_number: u32,
    pub my_team_status: TeamStatus,
    pub opponent_team_status: TeamStatus,
}

#[derive(Debug, Clone)]
pub struct PokemonState {
    pub id: Uuid,
    pub species_id: u16,
    pub level: u8,
    pub current_hp_percent: f32,
    pub status_conditions: Vec<StatusType>,
    pub available_moves: Vec<MoveId>,
    pub stat_stages: HashMap<StatType, i8>,
    pub ability_id: u32,
    pub held_item: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TeamStatus {
    pub active_pokemon: PokemonState,
    pub remaining_pokemon: u8,
    pub healthy_pokemon: u8,
    pub fainted_pokemon: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct MoveEvaluationKey {
    my_species: u16,
    opponent_species: u16,
    move_id: MoveId,
    hp_range: u8, // HP百分比范围 (0-10)
}

impl BattleAI {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            decision_history: Vec::new(),
            learning_data: AILearningData::new(),
            move_cache: HashMap::new(),
        }
    }

    /// 主要的决策入口点
    pub fn make_decision(
        &mut self,
        context: &BattleContext,
        my_participant_id: Uuid,
        damage_calculator: &DamageCalculator,
        effect_processor: &EffectProcessor,
        rng: &mut RandomGenerator,
    ) -> GameResult<TurnAction> {
        let start_time = std::time::Instant::now();

        // 分析当前战况
        let situation = self.analyze_battle_situation(context, my_participant_id)?;

        // 获取可能的行动
        let possible_actions = self.generate_possible_actions(&situation, context, my_participant_id)?;

        if possible_actions.is_empty() {
            return Err(GameError::BattleError("没有可用的行动".to_string()));
        }

        // 评估每个行动
        let mut action_evaluations = Vec::new();
        for action in possible_actions {
            let score = self.evaluate_action(
                &action,
                &situation,
                context,
                damage_calculator,
                effect_processor,
                rng,
            )?;
            
            action_evaluations.push((action, score));
        }

        // 根据难度和个性调整评分
        self.apply_difficulty_adjustments(&mut action_evaluations, rng);

        // 选择最佳行动
        let best_action = self.select_action(action_evaluations, rng)?;

        // 应用随机性
        let final_action = self.apply_randomness(best_action.0, &situation, rng);

        // 记录决策
        self.record_decision(AIDecision {
            turn: situation.turn_number,
            situation: situation.clone(),
            action: final_action.clone(),
            evaluation_score: best_action.1,
            reasoning: self.generate_reasoning(&final_action, &situation),
            alternative_actions: action_evaluations.into_iter()
                .filter(|(a, _)| a.action_id != final_action.action_id)
                .take(3)
                .collect(),
        });

        // 检查时间限制
        if start_time.elapsed().as_millis() as u64 > self.config.time_limit_ms {
            warn!("AI决策超时: {}ms", start_time.elapsed().as_millis());
        }

        Ok(final_action)
    }

    fn analyze_battle_situation(
        &self,
        context: &BattleContext,
        my_participant_id: Uuid,
    ) -> GameResult<BattleSituation> {
        // 找到我的参与者
        let my_participant = context.participants.iter()
            .find(|p| p.id == my_participant_id)
            .ok_or_else(|| GameError::BattleError("找不到AI参与者".to_string()))?;

        // 找到对手参与者（简化为第一个不是我的参与者）
        let opponent_participant = context.participants.iter()
            .find(|p| p.id != my_participant_id)
            .ok_or_else(|| GameError::BattleError("找不到对手参与者".to_string()))?;

        let my_pokemon = my_participant.active_pokemon.as_ref()
            .ok_or_else(|| GameError::BattleError("没有活跃的Pokemon".to_string()))?;

        let opponent_pokemon = opponent_participant.active_pokemon.as_ref()
            .ok_or_else(|| GameError::BattleError("对手没有活跃的Pokemon".to_string()))?;

        Ok(BattleSituation {
            my_pokemon: self.create_pokemon_state(my_pokemon)?,
            opponent_pokemon: self.create_pokemon_state(opponent_pokemon)?,
            field_conditions: context.field_effects.keys().cloned().collect(),
            weather: context.weather.clone(),
            turn_number: context.current_turn,
            my_team_status: self.analyze_team_status(my_participant)?,
            opponent_team_status: self.analyze_team_status(opponent_participant)?,
        })
    }

    fn create_pokemon_state(&self, pokemon: &IndividualPokemon) -> GameResult<PokemonState> {
        let max_hp = pokemon.cached_stats.as_ref().unwrap().hp;
        let hp_percent = if max_hp > 0 {
            pokemon.current_hp as f32 / max_hp as f32
        } else {
            0.0
        };

        let available_moves: Vec<MoveId> = pokemon.moves.iter()
            .filter(|m| m.current_pp > 0)
            .map(|m| m.move_id)
            .collect();

        let stat_stages = pokemon.battle_stats.as_ref()
            .map(|bs| bs.stat_stages.clone())
            .unwrap_or_default();

        Ok(PokemonState {
            id: pokemon.id,
            species_id: pokemon.species_id,
            level: pokemon.level,
            current_hp_percent: hp_percent,
            status_conditions: pokemon.status_conditions.iter()
                .map(|s| s.condition_type)
                .collect(),
            available_moves,
            stat_stages,
            ability_id: pokemon.ability_id,
            held_item: pokemon.held_item,
        })
    }

    fn analyze_team_status(&self, participant: &BattleParticipant) -> GameResult<TeamStatus> {
        let active_pokemon = participant.active_pokemon.as_ref()
            .ok_or_else(|| GameError::BattleError("没有活跃Pokemon".to_string()))?;

        let total_pokemon = participant.party.len();
        let fainted_count = participant.party.iter()
            .filter(|p| p.current_hp == 0)
            .count();
        let healthy_count = participant.party.iter()
            .filter(|p| p.current_hp > p.cached_stats.as_ref().unwrap().hp / 2)
            .count();

        Ok(TeamStatus {
            active_pokemon: self.create_pokemon_state(active_pokemon)?,
            remaining_pokemon: (total_pokemon - fainted_count) as u8,
            healthy_pokemon: healthy_count as u8,
            fainted_pokemon: fainted_count as u8,
        })
    }

    fn generate_possible_actions(
        &self,
        situation: &BattleSituation,
        context: &BattleContext,
        my_participant_id: Uuid,
    ) -> GameResult<Vec<TurnAction>> {
        let mut actions = Vec::new();

        // 生成攻击行动
        for (index, &move_id) in situation.my_pokemon.available_moves.iter().enumerate() {
            actions.push(TurnAction {
                action_id: Uuid::new_v4(),
                participant_id: my_participant_id,
                pokemon_id: situation.my_pokemon.id,
                action_type: ActionType::UseMove { move_id, move_slot: index },
                target: Some(ActionTarget::Opponent(situation.opponent_pokemon.id)),
                priority: 0, // 将在后续计算
                speed: 0,    // 将在后续计算
                selected_at: 0.0,
            });
        }

        // 生成换Pokemon行动（如果有可用的Pokemon）
        if situation.my_team_status.remaining_pokemon > 1 {
            let my_participant = context.participants.iter()
                .find(|p| p.id == my_participant_id)
                .unwrap();

            for pokemon in &my_participant.party {
                if pokemon.id != situation.my_pokemon.id && pokemon.current_hp > 0 {
                    actions.push(TurnAction {
                        action_id: Uuid::new_v4(),
                        participant_id: my_participant_id,
                        pokemon_id: situation.my_pokemon.id,
                        action_type: ActionType::SwitchPokemon { target_pokemon_id: pokemon.id },
                        target: None,
                        priority: 6, // 换Pokemon优先级高
                        speed: 0,
                        selected_at: 0.0,
                    });
                }
            }
        }

        // 如果没有可用招式，生成挣扎行动
        if situation.my_pokemon.available_moves.is_empty() {
            actions.push(TurnAction {
                action_id: Uuid::new_v4(),
                participant_id: my_participant_id,
                pokemon_id: situation.my_pokemon.id,
                action_type: ActionType::Struggle,
                target: Some(ActionTarget::Opponent(situation.opponent_pokemon.id)),
                priority: 0,
                speed: 0,
                selected_at: 0.0,
            });
        }

        Ok(actions)
    }

    fn evaluate_action(
        &mut self,
        action: &TurnAction,
        situation: &BattleSituation,
        context: &BattleContext,
        damage_calculator: &DamageCalculator,
        effect_processor: &EffectProcessor,
        rng: &mut RandomGenerator,
    ) -> GameResult<f32> {
        match &action.action_type {
            ActionType::UseMove { move_id, .. } => {
                self.evaluate_move_action(*move_id, situation, context, damage_calculator, rng)
            },
            ActionType::SwitchPokemon { target_pokemon_id } => {
                self.evaluate_switch_action(*target_pokemon_id, situation, context)
            },
            ActionType::Struggle => Ok(10.0), // 挣扎的基础评分很低
            _ => Ok(0.0), // 其他行动暂不评估
        }
    }

    fn evaluate_move_action(
        &mut self,
        move_id: MoveId,
        situation: &BattleSituation,
        context: &BattleContext,
        damage_calculator: &DamageCalculator,
        rng: &mut RandomGenerator,
    ) -> GameResult<f32> {
        // 检查缓存
        let cache_key = MoveEvaluationKey {
            my_species: situation.my_pokemon.species_id,
            opponent_species: situation.opponent_pokemon.species_id,
            move_id,
            hp_range: (situation.my_pokemon.current_hp_percent * 10.0) as u8,
        };

        if let Some(&cached_score) = self.move_cache.get(&cache_key) {
            return Ok(cached_score);
        }

        let mut score = 0.0f32;

        // 获取招式数据（简化实现）
        let move_data = self.get_move_data(move_id)?;

        // 基础威力评分
        if let Some(power) = move_data.power {
            score += power as f32 * 0.5;
        }

        // 命中率评分
        if let Some(accuracy) = move_data.accuracy {
            score *= accuracy as f32 / 100.0;
        }

        // 属性相克评分
        let type_effectiveness = self.calculate_type_effectiveness(
            move_data.move_type,
            situation.opponent_pokemon.species_id,
        )?;
        score *= type_effectiveness;

        // 根据AI难度调整评分深度
        match self.config.difficulty {
            AIDifficulty::Beginner => {
                // 新手AI主要看威力
                score += if move_data.power.unwrap_or(0) > 80 { 20.0 } else { 0.0 };
            },
            AIDifficulty::Expert | AIDifficulty::Master => {
                // 专家AI考虑复杂因素
                score += self.evaluate_move_effects(&move_data, situation)?;
                score += self.evaluate_situational_factors(&move_data, situation)?;
                score += self.predict_opponent_response(&move_data, situation)?;
            },
            _ => {
                // 中级AI适度考虑各种因素
                score += self.evaluate_move_effects(&move_data, situation)? * 0.5;
            },
        }

        // 根据个性调整评分
        score = self.apply_personality_bias(score, &move_data, situation);

        // 缓存结果
        self.move_cache.insert(cache_key, score);

        Ok(score)
    }

    fn evaluate_switch_action(
        &self,
        target_pokemon_id: Uuid,
        situation: &BattleSituation,
        context: &BattleContext,
    ) -> GameResult<f32> {
        let mut score = 50.0; // 换Pokemon的基础评分

        // 如果当前Pokemon处于不利状态，换Pokemon的价值更高
        if situation.my_pokemon.current_hp_percent < 0.3 {
            score += 30.0;
        }

        // 如果有不利的属性相克，换Pokemon价值更高
        // 这里需要更详细的实现来检查新Pokemon的类型优势

        // 如果有有害状态条件，换Pokemon可以清除
        if !situation.my_pokemon.status_conditions.is_empty() {
            score += 20.0;
        }

        Ok(score)
    }

    fn evaluate_move_effects(&self, move_data: &Move, situation: &BattleSituation) -> GameResult<f32> {
        let mut effect_score = 0.0;

        // 评估状态效果
        match move_data.move_type {
            PokemonType::Electric => {
                // 电系招式可能造成麻痹
                if !situation.opponent_pokemon.status_conditions.contains(&StatusType::Paralysis) {
                    effect_score += 15.0;
                }
            },
            PokemonType::Fire => {
                // 火系招式可能造成烧伤
                if !situation.opponent_pokemon.status_conditions.contains(&StatusType::Burn) {
                    effect_score += 12.0;
                }
            },
            _ => {},
        }

        // 评估招式特殊效果
        // 这里需要根据具体招式数据来实现

        Ok(effect_score)
    }

    fn evaluate_situational_factors(&self, move_data: &Move, situation: &BattleSituation) -> GameResult<f32> {
        let mut situational_score = 0.0;

        // 天气因素
        match situation.weather {
            WeatherCondition::Sunny => {
                if matches!(move_data.move_type, PokemonType::Fire) {
                    situational_score += 20.0;
                } else if matches!(move_data.move_type, PokemonType::Water) {
                    situational_score -= 20.0;
                }
            },
            WeatherCondition::Rain => {
                if matches!(move_data.move_type, PokemonType::Water) {
                    situational_score += 20.0;
                } else if matches!(move_data.move_type, PokemonType::Fire) {
                    situational_score -= 20.0;
                }
            },
            _ => {},
        }

        // 场地效果
        for condition in &situation.field_conditions {
            match condition.as_str() {
                "light_screen" => {
                    if matches!(move_data.category, MoveCategory::Special) {
                        situational_score -= 25.0;
                    }
                },
                "reflect" => {
                    if matches!(move_data.category, MoveCategory::Physical) {
                        situational_score -= 25.0;
                    }
                },
                _ => {},
            }
        }

        Ok(situational_score)
    }

    fn predict_opponent_response(&self, move_data: &Move, situation: &BattleSituation) -> GameResult<f32> {
        // 简化的对手行为预测
        let mut prediction_score = 0.0;

        // 如果招式可能让对手濒死，评分很高
        if let Some(power) = move_data.power {
            if power > 80 && situation.opponent_pokemon.current_hp_percent < 0.4 {
                prediction_score += 50.0;
            }
        }

        // 预测对手可能的换Pokemon行为
        if situation.opponent_team_status.remaining_pokemon > 1 {
            // 如果对手很可能换Pokemon，状态招式价值降低
            if matches!(move_data.category, MoveCategory::Status) {
                prediction_score -= 15.0;
            }
        }

        Ok(prediction_score)
    }

    fn apply_personality_bias(&self, base_score: f32, move_data: &Move, situation: &BattleSituation) -> f32 {
        let mut adjusted_score = base_score;

        match self.config.personality {
            AIPersonality::Aggressive => {
                if let Some(power) = move_data.power {
                    if power > 100 {
                        adjusted_score *= 1.3; // 偏爱高威力招式
                    }
                }
                if matches!(move_data.category, MoveCategory::Status) {
                    adjusted_score *= 0.7; // 不喜欢状态招式
                }
            },
            AIPersonality::Defensive => {
                if matches!(move_data.category, MoveCategory::Status) {
                    adjusted_score *= 1.2; // 偏爱状态招式
                }
                if situation.my_pokemon.current_hp_percent < 0.5 {
                    // 生命值低时更保守
                    adjusted_score *= 0.8;
                }
            },
            AIPersonality::Tactical => {
                // 战术型AI重视招式的附加效果
                adjusted_score += 10.0; // 所有招式都有基础加成，体现思考深度
            },
            _ => {}, // 其他个性暂不调整
        }

        adjusted_score
    }

    fn apply_difficulty_adjustments(&self, evaluations: &mut Vec<(TurnAction, f32)>, rng: &mut RandomGenerator) {
        match self.config.difficulty {
            AIDifficulty::Beginner => {
                // 新手AI有概率做出错误选择
                for (_, score) in evaluations.iter_mut() {
                    if rng.probability() < 0.2 {
                        *score *= rng.range_f32(0.3, 0.8); // 随机降低评分
                    }
                }
            },
            AIDifficulty::Expert | AIDifficulty::Master => {
                // 专家AI会进行更深度的分析
                for (action, score) in evaluations.iter_mut() {
                    if let ActionType::UseMove { move_id, .. } = &action.action_type {
                        // 专家AI会考虑学习数据
                        if let Some((success, total)) = self.learning_data.move_success_rates.get(move_id) {
                            let success_rate = *success as f32 / *total as f32;
                            *score *= (0.8 + success_rate * 0.4); // 根据历史成功率调整
                        }
                    }
                }
            },
            _ => {}, // 中级AI不做特殊调整
        }
    }

    fn select_action(&self, mut evaluations: Vec<(TurnAction, f32)>, rng: &mut RandomGenerator) -> GameResult<(TurnAction, f32)> {
        if evaluations.is_empty() {
            return Err(GameError::BattleError("没有可评估的行动".to_string()));
        }

        // 按评分排序
        evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 根据难度选择行动
        let selected_index = match self.config.difficulty {
            AIDifficulty::Beginner => {
                // 新手有时会选择非最优选项
                if rng.probability() < 0.3 {
                    rng.range(0, evaluations.len().min(3))
                } else {
                    0
                }
            },
            AIDifficulty::Expert | AIDifficulty::Master => {
                // 专家几乎总是选择最优选项
                if rng.probability() < 0.95 {
                    0
                } else {
                    rng.range(0, evaluations.len().min(2))
                }
            },
            _ => {
                // 中级AI大多选择最优，偶尔选择次优
                if rng.probability() < 0.8 {
                    0
                } else {
                    rng.range(0, evaluations.len().min(2))
                }
            },
        };

        Ok(evaluations.into_iter().nth(selected_index).unwrap())
    }

    fn apply_randomness(&self, action: TurnAction, situation: &BattleSituation, rng: &mut RandomGenerator) -> TurnAction {
        if self.config.randomness > rng.probability() {
            // 在随机性影响下，可能选择其他行动
            // 这里简化实现，实际可以重新生成一个随机行动
        }
        action
    }

    fn generate_reasoning(&self, action: &TurnAction, situation: &BattleSituation) -> String {
        match &action.action_type {
            ActionType::UseMove { move_id, .. } => {
                format!("选择使用招式 {} 基于威力和属性相克分析", move_id)
            },
            ActionType::SwitchPokemon { .. } => {
                format!("选择换Pokemon基于当前不利局面")
            },
            ActionType::Struggle => {
                format!("没有可用招式，被迫使用挣扎")
            },
            _ => "未知行动".to_string(),
        }
    }

    fn record_decision(&mut self, decision: AIDecision) {
        self.decision_history.push(decision);
        
        // 限制历史记录长度
        if self.decision_history.len() > 100 {
            self.decision_history.drain(0..50);
        }
    }

    // 辅助方法
    fn get_move_data(&self, move_id: MoveId) -> GameResult<Move> {
        // 简化实现，实际应该从数据加载器获取
        Ok(Move {
            id: move_id,
            name: "Test Move".to_string(),
            move_type: PokemonType::Normal,
            category: MoveCategory::Physical,
            power: Some(60),
            accuracy: Some(100),
            pp: 20,
            priority: 0,
            target: crate::pokemon::moves::MoveTarget::SingleOpponent,
            description: "Test move".to_string(),
            effects: Vec::new(),
        })
    }

    fn calculate_type_effectiveness(&self, move_type: PokemonType, defender_species: u16) -> GameResult<f32> {
        // 简化实现，实际需要查询属性相克表
        Ok(1.0)
    }

    /// 学习方法：从战斗结果中更新学习数据
    pub fn learn_from_battle_result(&mut self, won: bool, turn_count: u32) {
        self.learning_data.battle_stats.total_battles += 1;
        if won {
            self.learning_data.battle_stats.wins += 1;
        } else {
            self.learning_data.battle_stats.losses += 1;
        }

        // 更新平均回合数
        let total = self.learning_data.battle_stats.total_battles as f32;
        self.learning_data.battle_stats.average_turns = 
            (self.learning_data.battle_stats.average_turns * (total - 1.0) + turn_count as f32) / total;

        // 根据战斗结果调整招式成功率统计
        for decision in &self.decision_history {
            if let ActionType::UseMove { move_id, .. } = &decision.action.action_type {
                let entry = self.learning_data.move_success_rates.entry(*move_id).or_insert((0, 0));
                entry.1 += 1; // 总次数
                if won {
                    entry.0 += 1; // 成功次数（简化判断）
                }
            }
        }
    }

    /// 重置AI状态（新战斗开始时调用）
    pub fn reset_for_new_battle(&mut self) {
        self.decision_history.clear();
        self.move_cache.clear();
    }

    /// 获取AI统计信息
    pub fn get_stats(&self) -> &BattleStats {
        &self.learning_data.battle_stats
    }
}

impl AILearningData {
    pub fn new() -> Self {
        Self {
            move_success_rates: HashMap::new(),
            switch_effectiveness: HashMap::new(),
            opponent_patterns: HashMap::new(),
            battle_stats: BattleStats::default(),
        }
    }
}

impl AIConfig {
    pub fn beginner() -> Self {
        Self {
            difficulty: AIDifficulty::Beginner,
            personality: AIPersonality::Balanced,
            randomness: 0.3,
            lookahead_depth: 1,
            enable_learning: false,
            time_limit_ms: 1000,
        }
    }

    pub fn expert() -> Self {
        Self {
            difficulty: AIDifficulty::Expert,
            personality: AIPersonality::Analytical,
            randomness: 0.05,
            lookahead_depth: 3,
            enable_learning: true,
            time_limit_ms: 5000,
        }
    }

    pub fn custom(difficulty: AIDifficulty, personality: AIPersonality) -> Self {
        Self {
            difficulty,
            personality,
            randomness: match difficulty {
                AIDifficulty::Beginner => 0.4,
                AIDifficulty::Novice => 0.25,
                AIDifficulty::Intermediate => 0.15,
                AIDifficulty::Advanced => 0.1,
                AIDifficulty::Expert => 0.05,
                AIDifficulty::Master => 0.02,
            },
            lookahead_depth: match difficulty {
                AIDifficulty::Beginner | AIDifficulty::Novice => 1,
                AIDifficulty::Intermediate => 2,
                AIDifficulty::Advanced => 3,
                AIDifficulty::Expert | AIDifficulty::Master => 4,
            },
            enable_learning: matches!(difficulty, AIDifficulty::Advanced | AIDifficulty::Expert | AIDifficulty::Master),
            time_limit_ms: match difficulty {
                AIDifficulty::Beginner => 500,
                AIDifficulty::Novice => 1000,
                AIDifficulty::Intermediate => 2000,
                AIDifficulty::Advanced => 3000,
                AIDifficulty::Expert => 5000,
                AIDifficulty::Master => 8000,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_creation() {
        let ai = BattleAI::new(AIConfig::beginner());
        assert_eq!(ai.config.difficulty, AIDifficulty::Beginner);
        assert!(ai.decision_history.is_empty());
    }

    #[test]
    fn test_ai_config_presets() {
        let beginner = AIConfig::beginner();
        assert_eq!(beginner.difficulty, AIDifficulty::Beginner);
        assert_eq!(beginner.lookahead_depth, 1);
        
        let expert = AIConfig::expert();
        assert_eq!(expert.difficulty, AIDifficulty::Expert);
        assert!(expert.enable_learning);
    }

    #[test]
    fn test_battle_stats_tracking() {
        let mut ai = BattleAI::new(AIConfig::beginner());
        
        ai.learn_from_battle_result(true, 15);
        assert_eq!(ai.get_stats().wins, 1);
        assert_eq!(ai.get_stats().total_battles, 1);
        
        ai.learn_from_battle_result(false, 20);
        assert_eq!(ai.get_stats().losses, 1);
        assert_eq!(ai.get_stats().total_battles, 2);
        assert_eq!(ai.get_stats().average_turns, 17.5);
    }

    #[test]
    fn test_difficulty_randomness() {
        let beginner = AIConfig::custom(AIDifficulty::Beginner, AIPersonality::Balanced);
        let expert = AIConfig::custom(AIDifficulty::Expert, AIPersonality::Analytical);
        
        assert!(beginner.randomness > expert.randomness);
        assert!(beginner.lookahead_depth < expert.lookahead_depth);
    }
}