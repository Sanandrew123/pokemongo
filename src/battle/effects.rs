/*
* 开发心理过程：
* 1. 设计灵活的战斗效果系统，支持各种状态条件和场地效果
* 2. 实现效果的叠加、覆盖、免疫等复杂交互逻辑
* 3. 支持回合计时、触发条件、持续时间管理
* 4. 集成天气系统和特殊战斗环境
* 5. 提供可扩展的效果插件系统
* 6. 优化性能，支持大量效果同时存在
* 7. 实现完整的效果优先级和解决冲突的机制
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        individual::{IndividualPokemon, StatusType, StatusCondition},
        moves::{Move, MoveId},
        types::PokemonType,
        stats::StatType,
    },
    battle::{
        engine::BattleContext,
        turn::{TurnPhase, EffectApplication, EffectType},
    },
    world::environment::WeatherCondition,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone)]
pub struct EffectProcessor {
    /// 活跃的场地效果
    pub field_effects: HashMap<String, FieldEffect>,
    /// 天气效果
    pub weather: WeatherCondition,
    /// 效果处理器配置
    pub config: EffectConfig,
    /// 效果处理历史
    pub effect_history: Vec<EffectEvent>,
}

#[derive(Debug, Clone)]
pub struct EffectConfig {
    /// 最大同时存在的效果数量
    pub max_concurrent_effects: usize,
    /// 是否启用效果叠加
    pub allow_effect_stacking: bool,
    /// 效果优先级模式
    pub priority_mode: EffectPriorityMode,
    /// 自动清理过期效果
    pub auto_cleanup: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectPriorityMode {
    FirstIn,    // 先进先出
    LastIn,     // 后进先出
    ByPriority, // 按优先级排序
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEffect {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effect_type: FieldEffectType,
    pub source_id: Option<Uuid>,
    pub target_side: EffectSide,
    pub duration: EffectDuration,
    pub intensity: f32,
    pub priority: i8,
    pub created_turn: u32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldEffectType {
    // 伤害相关
    DamageModifier,     // 伤害修正
    TypeBoost,          // 属性加成
    
    // 防御相关
    Barrier,            // 屏障效果（光墙、反射壁等）
    Immunity,           // 免疫效果
    
    // 状态相关
    StatusPrevention,   // 状态预防
    StatusCure,         // 状态治愈
    
    // 环境相关
    Weather,            // 天气效果
    Terrain,            // 场地效果
    
    // 特殊效果
    Trap,               // 束缚效果
    Entry,              // 入场效果
    Priority,           // 优先度修正
    
    // 自定义效果
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSide {
    All,        // 影响全场
    User,       // 影响使用者一方
    Target,     // 影响目标一方
    Individual(Uuid), // 影响特定Pokemon
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectDuration {
    Permanent,           // 永久效果
    Turns(u8),          // 固定回合数
    UntilSwitch,        // 直到换Pokemon
    UntilKO,            // 直到濒死
    Conditional(String), // 条件触发结束
}

#[derive(Debug, Clone)]
pub struct EffectEvent {
    pub turn: u32,
    pub phase: TurnPhase,
    pub event_type: EffectEventType,
    pub effect_id: String,
    pub source_id: Option<Uuid>,
    pub target_id: Option<Uuid>,
    pub result: EffectResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectEventType {
    Applied,    // 效果被应用
    Triggered,  // 效果被触发
    Expired,    // 效果过期
    Removed,    // 效果被移除
    Blocked,    // 效果被阻挡
    Modified,   // 效果被修改
}

#[derive(Debug, Clone)]
pub enum EffectResult {
    Success(String),
    Failed(String),
    Partial(String, f32), // 部分成功，附带效果强度
    Blocked(String),
}

impl EffectProcessor {
    pub fn new() -> Self {
        Self {
            field_effects: HashMap::new(),
            weather: WeatherCondition::None,
            config: EffectConfig::default(),
            effect_history: Vec::new(),
        }
    }

    pub fn with_config(config: EffectConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// 应用场地效果
    pub fn apply_field_effect(&mut self, effect: FieldEffect, turn: u32) -> GameResult<()> {
        // 检查效果数量限制
        if self.field_effects.len() >= self.config.max_concurrent_effects {
            self.cleanup_expired_effects(turn)?;
            
            if self.field_effects.len() >= self.config.max_concurrent_effects {
                return Err(GameError::BattleError("场地效果数量已达上限".to_string()));
            }
        }

        // 检查冲突效果
        if let Some(existing) = self.check_conflicting_effect(&effect) {
            match self.resolve_effect_conflict(&effect, existing, turn)? {
                ConflictResolution::ReplaceExisting => {
                    self.remove_field_effect(&existing.id, turn)?;
                },
                ConflictResolution::Block => {
                    self.record_effect_event(EffectEvent {
                        turn,
                        phase: TurnPhase::ApplyEffects,
                        event_type: EffectEventType::Blocked,
                        effect_id: effect.id.clone(),
                        source_id: effect.source_id,
                        target_id: None,
                        result: EffectResult::Blocked(
                            format!("与{}冲突", existing.name)
                        ),
                    });
                    return Ok(());
                },
                ConflictResolution::Stack => {
                    // 允许叠加，继续处理
                },
            }
        }

        // 应用效果
        let effect_id = effect.id.clone();
        self.field_effects.insert(effect_id.clone(), effect.clone());

        self.record_effect_event(EffectEvent {
            turn,
            phase: TurnPhase::ApplyEffects,
            event_type: EffectEventType::Applied,
            effect_id,
            source_id: effect.source_id,
            target_id: None,
            result: EffectResult::Success(format!("{}生效", effect.name)),
        });

        // 触发效果的立即影响
        self.trigger_immediate_effect(&effect, turn)?;

        Ok(())
    }

    /// 移除场地效果
    pub fn remove_field_effect(&mut self, effect_id: &str, turn: u32) -> GameResult<()> {
        if let Some(effect) = self.field_effects.remove(effect_id) {
            self.record_effect_event(EffectEvent {
                turn,
                phase: TurnPhase::ApplyEffects,
                event_type: EffectEventType::Removed,
                effect_id: effect_id.to_string(),
                source_id: effect.source_id,
                target_id: None,
                result: EffectResult::Success(format!("{}消失", effect.name)),
            });

            // 触发移除时的效果
            self.trigger_removal_effect(&effect, turn)?;
        }

        Ok(())
    }

    /// 处理Pokemon状态条件
    pub fn apply_status_condition(
        &mut self,
        pokemon: &mut IndividualPokemon,
        status: StatusCondition,
        context: &BattleContext,
        turn: u32,
    ) -> GameResult<bool> {
        // 检查状态免疫
        if self.is_status_immune(pokemon, status.condition_type, context)? {
            return Ok(false);
        }

        // 检查已有状态
        if pokemon.has_status(status.condition_type) {
            return Ok(false);
        }

        // 应用状态
        let success = pokemon.apply_status(status.clone());
        
        if success {
            self.record_effect_event(EffectEvent {
                turn,
                phase: TurnPhase::ApplyEffects,
                event_type: EffectEventType::Applied,
                effect_id: format!("status_{:?}", status.condition_type),
                source_id: None,
                target_id: Some(pokemon.id),
                result: EffectResult::Success(
                    format!("{}进入了{:?}状态", pokemon.get_display_name(), status.condition_type)
                ),
            });

            // 触发状态应用时的效果
            self.trigger_status_effect(pokemon, status.condition_type, turn)?;
        }

        Ok(success)
    }

    /// 处理回合开始时的效果
    pub fn process_turn_start_effects(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();

        // 处理天气效果
        applications.extend(self.process_weather_effects(context, turn)?);

        // 处理场地效果
        applications.extend(self.process_field_effects(context, turn)?);

        // 处理Pokemon状态效果
        applications.extend(self.process_status_effects(context, turn)?);

        // 更新效果持续时间
        self.update_effect_durations(turn)?;

        Ok(applications)
    }

    /// 处理回合结束时的效果
    pub fn process_turn_end_effects(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();

        // 处理持续伤害效果
        applications.extend(self.process_damage_over_time(context, turn)?);

        // 处理持续恢复效果
        applications.extend(self.process_heal_over_time(context, turn)?);

        // 清理过期效果
        self.cleanup_expired_effects(turn)?;

        Ok(applications)
    }

    fn process_weather_effects(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();

        match self.weather {
            WeatherCondition::Sandstorm => {
                // 沙暴伤害非岩石/地面/钢系Pokemon
                for participant in &mut context.participants {
                    if let Some(pokemon) = &mut participant.active_pokemon {
                        if !self.is_weather_immune(pokemon, WeatherCondition::Sandstorm)? {
                            let damage = pokemon.cached_stats.as_ref().unwrap().hp / 16;
                            pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                            
                            applications.push(EffectApplication {
                                effect_id: "sandstorm_damage".to_string(),
                                source_id: Uuid::nil(),
                                target_id: pokemon.id,
                                effect_type: EffectType::Damage,
                                duration: None,
                                intensity: damage as f32,
                            });
                        }
                    }
                }
            },
            WeatherCondition::Hail => {
                // 冰雹伤害非冰系Pokemon
                for participant in &mut context.participants {
                    if let Some(pokemon) = &mut participant.active_pokemon {
                        if !self.is_weather_immune(pokemon, WeatherCondition::Hail)? {
                            let damage = pokemon.cached_stats.as_ref().unwrap().hp / 16;
                            pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                            
                            applications.push(EffectApplication {
                                effect_id: "hail_damage".to_string(),
                                source_id: Uuid::nil(),
                                target_id: pokemon.id,
                                effect_type: EffectType::Damage,
                                duration: None,
                                intensity: damage as f32,
                            });
                        }
                    }
                }
            },
            _ => {}, // 其他天气暂无每回合效果
        }

        Ok(applications)
    }

    fn process_field_effects(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();

        for (effect_id, effect) in &self.field_effects {
            match effect.effect_type {
                FieldEffectType::Trap => {
                    // 处理束缚效果（如火焰漩涡、束缚等）
                    applications.extend(self.process_trap_effect(effect, context, turn)?);
                },
                FieldEffectType::Terrain => {
                    // 处理场地效果（如电气场地、草木场地等）
                    applications.extend(self.process_terrain_effect(effect, context, turn)?);
                },
                _ => {}, // 其他效果在相应时机处理
            }
        }

        Ok(applications)
    }

    fn process_status_effects(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();

        for participant in &mut context.participants {
            if let Some(pokemon) = &mut participant.active_pokemon {
                let mut status_to_remove = Vec::new();

                for (index, status) in pokemon.status_conditions.iter_mut().enumerate() {
                    match status.condition_type {
                        StatusType::Burn => {
                            // 烧伤每回合造成最大HP 1/16 的伤害
                            let damage = pokemon.cached_stats.as_ref().unwrap().hp / 16;
                            pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                            
                            applications.push(EffectApplication {
                                effect_id: "burn_damage".to_string(),
                                source_id: Uuid::nil(),
                                target_id: pokemon.id,
                                effect_type: EffectType::Damage,
                                duration: None,
                                intensity: damage as f32,
                            });
                        },
                        StatusType::Poison => {
                            // 中毒每回合造成最大HP 1/8 的伤害
                            let damage = pokemon.cached_stats.as_ref().unwrap().hp / 8;
                            pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                            
                            applications.push(EffectApplication {
                                effect_id: "poison_damage".to_string(),
                                source_id: Uuid::nil(),
                                target_id: pokemon.id,
                                effect_type: EffectType::Damage,
                                duration: None,
                                intensity: damage as f32,
                            });
                        },
                        StatusType::BadlyPoisoned => {
                            // 剧毒每回合造成递增伤害
                            let turns_poisoned = turn.saturating_sub(status.applied_turn) + 1;
                            let damage = pokemon.cached_stats.as_ref().unwrap().hp * turns_poisoned as u16 / 16;
                            pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                            
                            applications.push(EffectApplication {
                                effect_id: "badly_poison_damage".to_string(),
                                source_id: Uuid::nil(),
                                target_id: pokemon.id,
                                effect_type: EffectType::Damage,
                                duration: None,
                                intensity: damage as f32,
                            });
                        },
                        StatusType::Sleep => {
                            // 睡眠状态持续时间减少
                            if let Some(duration) = &mut status.duration {
                                *duration = duration.saturating_sub(1);
                                if *duration == 0 {
                                    status_to_remove.push(index);
                                }
                            }
                        },
                        _ => {}, // 其他状态效果
                    }
                }

                // 移除过期状态
                for &index in status_to_remove.iter().rev() {
                    pokemon.status_conditions.remove(index);
                }
            }
        }

        Ok(applications)
    }

    fn process_damage_over_time(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();
        
        // 这里处理各种持续伤害效果，如束缚、诅咒等
        
        Ok(applications)
    }

    fn process_heal_over_time(
        &mut self,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();
        
        // 这里处理各种持续恢复效果，如许愿、水滴恢复等
        
        Ok(applications)
    }

    fn process_trap_effect(
        &self,
        effect: &FieldEffect,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();
        
        // 处理束缚类效果
        for participant in &mut context.participants {
            if let Some(pokemon) = &mut participant.active_pokemon {
                if self.is_affected_by_effect(pokemon, effect) {
                    let damage = pokemon.cached_stats.as_ref().unwrap().hp / 8;
                    pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                    
                    applications.push(EffectApplication {
                        effect_id: effect.id.clone(),
                        source_id: effect.source_id.unwrap_or(Uuid::nil()),
                        target_id: pokemon.id,
                        effect_type: EffectType::Damage,
                        duration: None,
                        intensity: damage as f32,
                    });
                }
            }
        }
        
        Ok(applications)
    }

    fn process_terrain_effect(
        &self,
        effect: &FieldEffect,
        context: &mut BattleContext,
        turn: u32,
    ) -> GameResult<Vec<EffectApplication>> {
        let mut applications = Vec::new();
        
        // 处理场地效果，如电气场地的每回合恢复
        match effect.id.as_str() {
            "grassy_terrain" => {
                // 草木场地：接触地面的Pokemon每回合恢复HP
                for participant in &mut context.participants {
                    if let Some(pokemon) = &mut participant.active_pokemon {
                        let heal = pokemon.cached_stats.as_ref().unwrap().hp / 16;
                        let max_hp = pokemon.cached_stats.as_ref().unwrap().hp;
                        pokemon.current_hp = (pokemon.current_hp + heal).min(max_hp);
                        
                        applications.push(EffectApplication {
                            effect_id: effect.id.clone(),
                            source_id: effect.source_id.unwrap_or(Uuid::nil()),
                            target_id: pokemon.id,
                            effect_type: EffectType::Heal,
                            duration: None,
                            intensity: heal as f32,
                        });
                    }
                }
            },
            _ => {},
        }
        
        Ok(applications)
    }

    fn is_status_immune(
        &self,
        pokemon: &IndividualPokemon,
        status_type: StatusType,
        context: &BattleContext,
    ) -> GameResult<bool> {
        // 检查特性免疫
        match pokemon.ability_id {
            1 => matches!(status_type, StatusType::Burn), // 水幕能力免疫烧伤
            2 => matches!(status_type, StatusType::Freeze), // 岩浆铠甲免疫冰冻
            // 更多能力免疫...
            _ => Ok(false),
        }
    }

    fn is_weather_immune(
        &self,
        pokemon: &IndividualPokemon,
        weather: WeatherCondition,
    ) -> GameResult<bool> {
        match weather {
            WeatherCondition::Sandstorm => {
                // 岩石、地面、钢系免疫沙暴伤害
                // 在实际实现中会检查Pokemon属性
                Ok(false) // 临时返回
            },
            WeatherCondition::Hail => {
                // 冰系免疫冰雹伤害
                Ok(false) // 临时返回
            },
            _ => Ok(false),
        }
    }

    fn is_affected_by_effect(&self, pokemon: &IndividualPokemon, effect: &FieldEffect) -> bool {
        match effect.target_side {
            EffectSide::All => true,
            EffectSide::Individual(id) => pokemon.id == id,
            _ => false, // 简化实现
        }
    }

    fn check_conflicting_effect(&self, effect: &FieldEffect) -> Option<&FieldEffect> {
        // 检查是否有冲突的效果
        for existing in self.field_effects.values() {
            if self.effects_conflict(effect, existing) {
                return Some(existing);
            }
        }
        None
    }

    fn effects_conflict(&self, effect1: &FieldEffect, effect2: &FieldEffect) -> bool {
        // 定义效果冲突规则
        match (&effect1.effect_type, &effect2.effect_type) {
            (FieldEffectType::Weather, FieldEffectType::Weather) => true, // 天气效果互斥
            (FieldEffectType::Terrain, FieldEffectType::Terrain) => true, // 场地效果互斥
            _ => effect1.id == effect2.id, // 相同ID的效果冲突
        }
    }

    fn resolve_effect_conflict(
        &self,
        new_effect: &FieldEffect,
        existing_effect: &FieldEffect,
        turn: u32,
    ) -> GameResult<ConflictResolution> {
        match self.config.priority_mode {
            EffectPriorityMode::FirstIn => Ok(ConflictResolution::Block),
            EffectPriorityMode::LastIn => Ok(ConflictResolution::ReplaceExisting),
            EffectPriorityMode::ByPriority => {
                if new_effect.priority > existing_effect.priority {
                    Ok(ConflictResolution::ReplaceExisting)
                } else if new_effect.priority == existing_effect.priority {
                    Ok(ConflictResolution::Block)
                } else {
                    Ok(ConflictResolution::Block)
                }
            },
        }
    }

    fn trigger_immediate_effect(&mut self, effect: &FieldEffect, turn: u32) -> GameResult<()> {
        // 触发效果的立即影响
        match &effect.effect_type {
            FieldEffectType::Weather => {
                // 天气变化时的立即影响
            },
            FieldEffectType::Terrain => {
                // 场地效果的立即影响
            },
            _ => {},
        }
        Ok(())
    }

    fn trigger_removal_effect(&mut self, effect: &FieldEffect, turn: u32) -> GameResult<()> {
        // 效果移除时的影响
        Ok(())
    }

    fn trigger_status_effect(
        &mut self,
        pokemon: &mut IndividualPokemon,
        status_type: StatusType,
        turn: u32,
    ) -> GameResult<()> {
        // 状态效果应用时的立即影响
        match status_type {
            StatusType::Paralysis => {
                // 麻痹时立即检查是否无法行动
            },
            StatusType::Freeze => {
                // 冰冻时立即检查是否解冻
            },
            _ => {},
        }
        Ok(())
    }

    fn update_effect_durations(&mut self, turn: u32) -> GameResult<()> {
        let mut effects_to_remove = Vec::new();

        for (id, effect) in &mut self.field_effects {
            match &mut effect.duration {
                EffectDuration::Turns(turns) => {
                    let turns_passed = turn.saturating_sub(effect.created_turn);
                    if turns_passed >= *turns as u32 {
                        effects_to_remove.push(id.clone());
                    }
                },
                _ => {}, // 其他持续时间类型的处理
            }
        }

        for id in effects_to_remove {
            self.remove_field_effect(&id, turn)?;
        }

        Ok(())
    }

    fn cleanup_expired_effects(&mut self, turn: u32) -> GameResult<()> {
        if !self.config.auto_cleanup {
            return Ok(());
        }

        self.update_effect_durations(turn)?;
        Ok(())
    }

    fn record_effect_event(&mut self, event: EffectEvent) {
        self.effect_history.push(event);
        
        // 限制历史记录长度
        if self.effect_history.len() > 1000 {
            self.effect_history.drain(0..500);
        }
    }

    /// 获取活跃效果统计
    pub fn get_active_effects_count(&self) -> usize {
        self.field_effects.len()
    }

    /// 清除所有效果
    pub fn clear_all_effects(&mut self, turn: u32) {
        let effect_ids: Vec<String> = self.field_effects.keys().cloned().collect();
        for id in effect_ids {
            let _ = self.remove_field_effect(&id, turn);
        }
    }

    /// 获取特定类型的效果
    pub fn get_effects_by_type(&self, effect_type: FieldEffectType) -> Vec<&FieldEffect> {
        self.field_effects
            .values()
            .filter(|e| std::mem::discriminant(&e.effect_type) == std::mem::discriminant(&effect_type))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictResolution {
    ReplaceExisting,
    Block,
    Stack,
}

impl EffectConfig {
    pub fn default() -> Self {
        Self {
            max_concurrent_effects: 20,
            allow_effect_stacking: false,
            priority_mode: EffectPriorityMode::ByPriority,
            auto_cleanup: true,
        }
    }

    pub fn permissive() -> Self {
        Self {
            max_concurrent_effects: 50,
            allow_effect_stacking: true,
            priority_mode: EffectPriorityMode::FirstIn,
            auto_cleanup: true,
        }
    }

    pub fn strict() -> Self {
        Self {
            max_concurrent_effects: 10,
            allow_effect_stacking: false,
            priority_mode: EffectPriorityMode::ByPriority,
            auto_cleanup: true,
        }
    }
}

impl Default for EffectProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// 一些预定义的常用效果
impl FieldEffect {
    pub fn light_screen(source_id: Uuid, turns: u8) -> Self {
        Self {
            id: "light_screen".to_string(),
            name: "光墙".to_string(),
            description: "减少特殊攻击伤害".to_string(),
            effect_type: FieldEffectType::Barrier,
            source_id: Some(source_id),
            target_side: EffectSide::User,
            duration: EffectDuration::Turns(turns),
            intensity: 0.5,
            priority: 0,
            created_turn: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn reflect(source_id: Uuid, turns: u8) -> Self {
        Self {
            id: "reflect".to_string(),
            name: "反射壁".to_string(),
            description: "减少物理攻击伤害".to_string(),
            effect_type: FieldEffectType::Barrier,
            source_id: Some(source_id),
            target_side: EffectSide::User,
            duration: EffectDuration::Turns(turns),
            intensity: 0.5,
            priority: 0,
            created_turn: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn grassy_terrain(turns: u8) -> Self {
        Self {
            id: "grassy_terrain".to_string(),
            name: "草木场地".to_string(),
            description: "强化草系招式，每回合恢复HP".to_string(),
            effect_type: FieldEffectType::Terrain,
            source_id: None,
            target_side: EffectSide::All,
            duration: EffectDuration::Turns(turns),
            intensity: 1.3,
            priority: 0,
            created_turn: 0,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_processor_creation() {
        let processor = EffectProcessor::new();
        assert_eq!(processor.get_active_effects_count(), 0);
        assert_eq!(processor.weather, WeatherCondition::None);
    }

    #[test]
    fn test_field_effect_application() {
        let mut processor = EffectProcessor::new();
        let effect = FieldEffect::light_screen(Uuid::new_v4(), 5);
        
        let result = processor.apply_field_effect(effect, 1);
        assert!(result.is_ok());
        assert_eq!(processor.get_active_effects_count(), 1);
    }

    #[test]
    fn test_effect_conflict_resolution() {
        let mut processor = EffectProcessor::with_config(
            EffectConfig { priority_mode: EffectPriorityMode::LastIn, ..EffectConfig::default() }
        );
        
        let effect1 = FieldEffect::grassy_terrain(8);
        let effect2 = FieldEffect {
            id: "electric_terrain".to_string(),
            name: "电气场地".to_string(),
            description: "强化电系招式".to_string(),
            effect_type: FieldEffectType::Terrain,
            source_id: None,
            target_side: EffectSide::All,
            duration: EffectDuration::Turns(8),
            intensity: 1.3,
            priority: 0,
            created_turn: 0,
            metadata: HashMap::new(),
        };
        
        processor.apply_field_effect(effect1, 1).unwrap();
        processor.apply_field_effect(effect2, 2).unwrap();
        
        // 应该只有电气场地生效（后进先出）
        assert_eq!(processor.get_active_effects_count(), 1);
        assert!(processor.field_effects.contains_key("electric_terrain"));
        assert!(!processor.field_effects.contains_key("grassy_terrain"));
    }

    #[test]
    fn test_effect_duration() {
        let mut processor = EffectProcessor::new();
        let mut effect = FieldEffect::light_screen(Uuid::new_v4(), 3);
        effect.created_turn = 1;
        
        processor.apply_field_effect(effect, 1).unwrap();
        assert_eq!(processor.get_active_effects_count(), 1);
        
        // 3回合后效果应该消失
        processor.update_effect_durations(4).unwrap();
        assert_eq!(processor.get_active_effects_count(), 0);
    }

    #[test]
    fn test_weather_effects() {
        let mut processor = EffectProcessor::new();
        processor.weather = WeatherCondition::Sandstorm;
        
        // 测试天气效果处理
        // 这里需要mock BattleContext，简化测试
        let applications = processor.process_weather_effects(
            &mut BattleContext::default(),
            1
        ).unwrap();
        
        // 沙暴应该产生伤害效果
        assert!(!applications.is_empty());
    }
}