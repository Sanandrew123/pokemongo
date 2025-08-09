// 宝可梦系统模块 - 游戏核心玩法系统
// 开发心理：宝可梦是游戏的灵魂，需要完整的数据模型和行为系统
// 设计原则：数据驱动、可扩展的种族系统、灵活的个体差异

pub mod species;
pub mod stats;
pub mod types;
pub mod moves;
pub mod abilities;
pub mod evolution;
pub mod ai;

// 重新导出主要类型
pub use species::{PokemonSpecies, SpeciesId};
pub use stats::{BaseStats, IndividualValues, EffortValues, PokemonStats};
pub use types::{PokemonType, TypeEffectiveness};
pub use moves::{Move, MoveId, MoveCategory, MoveTarget};
pub use abilities::{Ability, AbilityId, AbilityEffect};
pub use evolution::{EvolutionChain, EvolutionTrigger, EvolutionCondition};
pub use ai::{PokemonAI, AIBehavior, AIPersonality};

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug};

// 宝可梦个体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    // 基本信息
    pub id: u64,
    pub species_id: SpeciesId,
    pub nickname: Option<String>,
    pub level: u8,
    pub experience: u32,
    pub gender: Gender,
    pub nature: Nature,
    pub is_shiny: bool,
    
    // 能力值
    pub individual_values: IndividualValues,
    pub effort_values: EffortValues,
    pub current_hp: u16,
    
    // 技能
    pub moves: Vec<MoveSlot>,
    pub ability_id: AbilityId,
    
    // 状态
    pub status_conditions: Vec<StatusCondition>,
    pub held_item: Option<ItemId>,
    
    // 训练信息
    pub trainer_id: Option<u64>,
    pub original_trainer: String,
    pub caught_location: String,
    pub caught_level: u8,
    pub friendship: u8,
    
    // 战斗相关
    pub current_stats: Option<PokemonStats>,
    pub stat_stages: StatStages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Genderless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nature {
    Hardy, Lonely, Brave, Adamant, Naughty,
    Bold, Docile, Relaxed, Impish, Lax,
    Timid, Hasty, Serious, Jolly, Naive,
    Modest, Mild, Quiet, Bashful, Rash,
    Calm, Gentle, Sassy, Careful, Quirky,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSlot {
    pub move_id: MoveId,
    pub current_pp: u8,
    pub max_pp: u8,
    pub pp_ups: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusCondition {
    None,
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned,
    Sleep { turns_remaining: u8 },
    Confusion { turns_remaining: u8 },
    Flinch,
    Infatuation,
}

pub type ItemId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatStages {
    pub attack: i8,
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

impl Default for StatStages {
    fn default() -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        }
    }
}

impl Pokemon {
    // 创建新的宝可梦个体
    pub fn new(
        species_id: SpeciesId,
        level: u8,
        trainer_id: Option<u64>,
        original_trainer: String,
        caught_location: String,
    ) -> Result<Self> {
        let species = PokemonSpecies::get(species_id)
            .ok_or_else(|| GameError::PokemonError("无效的宝可梦种族ID".to_string()))?;
        
        // 生成随机个体值
        let individual_values = IndividualValues::random();
        
        // 初始努力值为0
        let effort_values = EffortValues::default();
        
        // 随机性别（基于种族的性别比例）
        let gender = species.generate_gender();
        
        // 随机性格
        let nature = Nature::random();
        
        // 随机判断是否为异色（1/4096概率）
        let is_shiny = fastrand::u32(1..=4096) == 1;
        
        // 计算经验值
        let experience = species.experience_for_level(level);
        
        // 计算当前能力值
        let current_stats = PokemonStats::calculate(
            &species.base_stats,
            &individual_values,
            &effort_values,
            level,
            nature,
        );
        
        let current_hp = current_stats.hp;
        
        // 学习初始技能
        let moves = species.get_learnable_moves_at_level(level)
            .into_iter()
            .take(4)
            .map(|move_id| {
                let move_data = Move::get(move_id).unwrap();
                MoveSlot {
                    move_id,
                    current_pp: move_data.pp,
                    max_pp: move_data.pp,
                    pp_ups: 0,
                }
            })
            .collect();
        
        // 随机能力
        let ability_id = species.get_random_ability();
        
        let pokemon = Pokemon {
            id: fastrand::u64(1..),
            species_id,
            nickname: None,
            level,
            experience,
            gender,
            nature,
            is_shiny,
            individual_values,
            effort_values,
            current_hp,
            moves,
            ability_id,
            status_conditions: vec![StatusCondition::None],
            held_item: None,
            trainer_id,
            original_trainer,
            caught_location,
            caught_level: level,
            friendship: species.base_friendship,
            current_stats: Some(current_stats),
            stat_stages: StatStages::default(),
        };
        
        debug!("创建新宝可梦: {} Lv.{}", species.name, level);
        Ok(pokemon)
    }
    
    // 获取种族信息
    pub fn get_species(&self) -> Result<&PokemonSpecies> {
        PokemonSpecies::get(self.species_id)
            .ok_or_else(|| GameError::PokemonError("宝可梦种族数据丢失".to_string()))
    }
    
    // 获取显示名称
    pub fn get_display_name(&self) -> String {
        if let Some(ref nickname) = self.nickname {
            nickname.clone()
        } else if let Ok(species) = self.get_species() {
            species.name.clone()
        } else {
            format!("未知宝可梦#{}", self.species_id)
        }
    }
    
    // 计算当前能力值
    pub fn calculate_stats(&mut self) -> Result<()> {
        let species = self.get_species()?;
        
        self.current_stats = Some(PokemonStats::calculate(
            &species.base_stats,
            &self.individual_values,
            &self.effort_values,
            self.level,
            self.nature,
        ));
        
        Ok(())
    }
    
    // 获取当前能力值
    pub fn get_stats(&self) -> Result<&PokemonStats> {
        self.current_stats.as_ref()
            .ok_or_else(|| GameError::PokemonError("能力值未计算".to_string()))
    }
    
    // 升级
    pub fn level_up(&mut self) -> Result<Vec<MoveId>> {
        if self.level >= 100 {
            return Err(GameError::PokemonError("已达到最高等级".to_string()));
        }
        
        let species = self.get_species()?;
        self.level += 1;
        self.experience = species.experience_for_level(self.level);
        
        // 重新计算能力值
        self.calculate_stats()?;
        
        // 恢复HP
        if let Ok(stats) = self.get_stats() {
            self.current_hp = stats.hp;
        }
        
        // 检查学习新技能
        let new_moves = species.get_learnable_moves_at_level(self.level);
        
        info!("{}升级到Lv.{}!", self.get_display_name(), self.level);
        Ok(new_moves)
    }
    
    // 学习技能
    pub fn learn_move(&mut self, move_id: MoveId, slot: Option<usize>) -> Result<Option<MoveId>> {
        let move_data = Move::get(move_id)
            .ok_or_else(|| GameError::PokemonError("无效的技能ID".to_string()))?;
        
        let new_move_slot = MoveSlot {
            move_id,
            current_pp: move_data.pp,
            max_pp: move_data.pp,
            pp_ups: 0,
        };
        
        if let Some(slot_index) = slot {
            if slot_index >= 4 {
                return Err(GameError::PokemonError("技能位置无效".to_string()));
            }
            
            let old_move = if slot_index < self.moves.len() {
                Some(self.moves[slot_index].move_id)
            } else {
                None
            };
            
            if slot_index < self.moves.len() {
                self.moves[slot_index] = new_move_slot;
            } else {
                self.moves.push(new_move_slot);
            }
            
            Ok(old_move)
        } else {
            // 如果技能位置不满，直接添加
            if self.moves.len() < 4 {
                self.moves.push(new_move_slot);
                Ok(None)
            } else {
                Err(GameError::PokemonError("技能位置已满".to_string()))
            }
        }
    }
    
    // 使用技能
    pub fn use_move(&mut self, move_index: usize) -> Result<()> {
        if move_index >= self.moves.len() {
            return Err(GameError::PokemonError("无效的技能索引".to_string()));
        }
        
        let move_slot = &mut self.moves[move_index];
        if move_slot.current_pp == 0 {
            return Err(GameError::PokemonError("技能PP已耗尽".to_string()));
        }
        
        move_slot.current_pp -= 1;
        Ok(())
    }
    
    // 恢复HP
    pub fn heal(&mut self, amount: u16) -> Result<u16> {
        let stats = self.get_stats()?;
        let max_hp = stats.hp;
        let old_hp = self.current_hp;
        
        self.current_hp = (self.current_hp + amount).min(max_hp);
        let healed = self.current_hp - old_hp;
        
        Ok(healed)
    }
    
    // 受到伤害
    pub fn take_damage(&mut self, damage: u16) -> bool {
        self.current_hp = self.current_hp.saturating_sub(damage);
        self.current_hp == 0
    }
    
    // 是否濒死
    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }
    
    // 应用状态异常
    pub fn apply_status(&mut self, status: StatusCondition) {
        // 移除之前的状态异常（某些状态可以覆盖）
        self.status_conditions.retain(|s| !s.conflicts_with(&status));
        self.status_conditions.push(status);
    }
    
    // 清除状态异常
    pub fn clear_status(&mut self, status_type: &StatusCondition) {
        self.status_conditions.retain(|s| !std::mem::discriminant(s).eq(&std::mem::discriminant(status_type)));
    }
    
    // 检查是否有特定状态
    pub fn has_status(&self, status_type: &StatusCondition) -> bool {
        self.status_conditions.iter().any(|s| std::mem::discriminant(s).eq(&std::mem::discriminant(status_type)))
    }
    
    // 检查是否可以进化
    pub fn can_evolve(&self) -> Result<Vec<EvolutionChain>> {
        let species = self.get_species()?;
        let evolution_chains = species.get_evolution_chains();
        
        let valid_evolutions: Vec<_> = evolution_chains
            .into_iter()
            .filter(|chain| chain.check_conditions(self))
            .collect();
        
        Ok(valid_evolutions)
    }
    
    // 进化
    pub fn evolve(&mut self, target_species_id: SpeciesId) -> Result<()> {
        let new_species = PokemonSpecies::get(target_species_id)
            .ok_or_else(|| GameError::PokemonError("进化目标种族不存在".to_string()))?;
        
        let old_species_name = self.get_species()?.name.clone();
        
        self.species_id = target_species_id;
        self.calculate_stats()?;
        
        info!("{}进化成{}!", old_species_name, new_species.name);
        Ok(())
    }
}

impl Nature {
    pub fn random() -> Self {
        match fastrand::u8(0..25) {
            0 => Nature::Hardy, 1 => Nature::Lonely, 2 => Nature::Brave, 3 => Nature::Adamant, 4 => Nature::Naughty,
            5 => Nature::Bold, 6 => Nature::Docile, 7 => Nature::Relaxed, 8 => Nature::Impish, 9 => Nature::Lax,
            10 => Nature::Timid, 11 => Nature::Hasty, 12 => Nature::Serious, 13 => Nature::Jolly, 14 => Nature::Naive,
            15 => Nature::Modest, 16 => Nature::Mild, 17 => Nature::Quiet, 18 => Nature::Bashful, 19 => Nature::Rash,
            20 => Nature::Calm, 21 => Nature::Gentle, 22 => Nature::Sassy, 23 => Nature::Careful, _ => Nature::Quirky,
        }
    }
    
    // 获取性格对能力值的影响
    pub fn get_stat_multiplier(&self, stat: StatType) -> f32 {
        match (self, stat) {
            // 攻击+防御-
            (Nature::Lonely, StatType::Attack) => 1.1,
            (Nature::Lonely, StatType::Defense) => 0.9,
            // 攻击+特攻-
            (Nature::Adamant, StatType::Attack) => 1.1,
            (Nature::Adamant, StatType::SpecialAttack) => 0.9,
            // 攻击+特防-
            (Nature::Naughty, StatType::Attack) => 1.1,
            (Nature::Naughty, StatType::SpecialDefense) => 0.9,
            // 攻击+速度-
            (Nature::Brave, StatType::Attack) => 1.1,
            (Nature::Brave, StatType::Speed) => 0.9,
            
            // 防御+攻击-
            (Nature::Bold, StatType::Defense) => 1.1,
            (Nature::Bold, StatType::Attack) => 0.9,
            // 防御+特攻-
            (Nature::Impish, StatType::Defense) => 1.1,
            (Nature::Impish, StatType::SpecialAttack) => 0.9,
            // 防御+特防-
            (Nature::Lax, StatType::Defense) => 1.1,
            (Nature::Lax, StatType::SpecialDefense) => 0.9,
            // 防御+速度-
            (Nature::Relaxed, StatType::Defense) => 1.1,
            (Nature::Relaxed, StatType::Speed) => 0.9,
            
            // 特攻+攻击-
            (Nature::Modest, StatType::SpecialAttack) => 1.1,
            (Nature::Modest, StatType::Attack) => 0.9,
            // 特攻+防御-
            (Nature::Mild, StatType::SpecialAttack) => 1.1,
            (Nature::Mild, StatType::Defense) => 0.9,
            // 特攻+特防-
            (Nature::Rash, StatType::SpecialAttack) => 1.1,
            (Nature::Rash, StatType::SpecialDefense) => 0.9,
            // 特攻+速度-
            (Nature::Quiet, StatType::SpecialAttack) => 1.1,
            (Nature::Quiet, StatType::Speed) => 0.9,
            
            // 特防+攻击-
            (Nature::Calm, StatType::SpecialDefense) => 1.1,
            (Nature::Calm, StatType::Attack) => 0.9,
            // 特防+防御-
            (Nature::Gentle, StatType::SpecialDefense) => 1.1,
            (Nature::Gentle, StatType::Defense) => 0.9,
            // 特防+特攻-
            (Nature::Careful, StatType::SpecialDefense) => 1.1,
            (Nature::Careful, StatType::SpecialAttack) => 0.9,
            // 特防+速度-
            (Nature::Sassy, StatType::SpecialDefense) => 1.1,
            (Nature::Sassy, StatType::Speed) => 0.9,
            
            // 速度+攻击-
            (Nature::Timid, StatType::Speed) => 1.1,
            (Nature::Timid, StatType::Attack) => 0.9,
            // 速度+防御-
            (Nature::Hasty, StatType::Speed) => 1.1,
            (Nature::Hasty, StatType::Defense) => 0.9,
            // 速度+特攻-
            (Nature::Jolly, StatType::Speed) => 1.1,
            (Nature::Jolly, StatType::SpecialAttack) => 0.9,
            // 速度+特防-
            (Nature::Naive, StatType::Speed) => 1.1,
            (Nature::Naive, StatType::SpecialDefense) => 0.9,
            
            // 平衡性格
            _ => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    HP,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

impl StatusCondition {
    // 检查状态是否冲突
    pub fn conflicts_with(&self, other: &StatusCondition) -> bool {
        use StatusCondition::*;
        match (self, other) {
            (Burn, Burn) | (Freeze, Freeze) | (Paralysis, Paralysis) => true,
            (Poison, Poison) | (BadlyPoisoned, BadlyPoisoned) => true,
            (Poison, BadlyPoisoned) | (BadlyPoisoned, Poison) => true,
            (Sleep { .. }, Sleep { .. }) => true,
            (Confusion { .. }, Confusion { .. }) => true,
            _ => false,
        }
    }
}

// 宝可梦管理器
pub struct PokemonManager {
    pokemon_storage: HashMap<u64, Pokemon>,
    next_id: u64,
}

impl PokemonManager {
    pub fn new() -> Self {
        Self {
            pokemon_storage: HashMap::new(),
            next_id: 1,
        }
    }
    
    // 创建新的宝可梦
    pub fn create_pokemon(
        &mut self,
        species_id: SpeciesId,
        level: u8,
        trainer_id: Option<u64>,
        original_trainer: String,
        caught_location: String,
    ) -> Result<u64> {
        let mut pokemon = Pokemon::new(
            species_id,
            level,
            trainer_id,
            original_trainer,
            caught_location,
        )?;
        
        pokemon.id = self.next_id;
        self.next_id += 1;
        
        let id = pokemon.id;
        self.pokemon_storage.insert(id, pokemon);
        
        Ok(id)
    }
    
    // 获取宝可梦
    pub fn get_pokemon(&self, id: u64) -> Option<&Pokemon> {
        self.pokemon_storage.get(&id)
    }
    
    // 获取可变引用
    pub fn get_pokemon_mut(&mut self, id: u64) -> Option<&mut Pokemon> {
        self.pokemon_storage.get_mut(&id)
    }
    
    // 释放宝可梦
    pub fn release_pokemon(&mut self, id: u64) -> Option<Pokemon> {
        self.pokemon_storage.remove(&id)
    }
    
    // 获取训练师的所有宝可梦
    pub fn get_trainer_pokemon(&self, trainer_id: u64) -> Vec<&Pokemon> {
        self.pokemon_storage
            .values()
            .filter(|p| p.trainer_id == Some(trainer_id))
            .collect()
    }
    
    // 统计信息
    pub fn get_total_count(&self) -> usize {
        self.pokemon_storage.len()
    }
    
    pub fn get_species_count(&self, species_id: SpeciesId) -> usize {
        self.pokemon_storage
            .values()
            .filter(|p| p.species_id == species_id)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nature_stat_multipliers() {
        let adamant = Nature::Adamant;
        assert_eq!(adamant.get_stat_multiplier(StatType::Attack), 1.1);
        assert_eq!(adamant.get_stat_multiplier(StatType::SpecialAttack), 0.9);
        assert_eq!(adamant.get_stat_multiplier(StatType::Defense), 1.0);
    }
    
    #[test]
    fn test_status_condition_conflicts() {
        let burn1 = StatusCondition::Burn;
        let burn2 = StatusCondition::Burn;
        let poison = StatusCondition::Poison;
        
        assert!(burn1.conflicts_with(&burn2));
        assert!(!burn1.conflicts_with(&poison));
    }
    
    #[test]
    fn test_pokemon_manager() {
        let mut manager = PokemonManager::new();
        assert_eq!(manager.get_total_count(), 0);
        
        // 这里需要有效的种族ID和种族数据才能测试
        // let id = manager.create_pokemon(1, 5, None, "Test".to_string(), "Test Location".to_string()).unwrap();
        // assert_eq!(manager.get_total_count(), 1);
    }
}