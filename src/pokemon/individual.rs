/*
* 开发心理过程：
* 1. 创建个体Pokemon实例系统，每个Pokemon都有独特的属性
* 2. 实现IV（Individual Values）系统，决定Pokemon的个体差异
* 3. 支持性格系统，影响属性成长
* 4. 实现经验值和等级系统
* 5. 支持状态条件（中毒、烧伤等）
* 6. 提供Pokemon个性化数据（昵称、遇见信息等）
* 7. 集成持久化和序列化支持
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        species::{PokemonSpecies, SpeciesId},
        stats::{StatType, StatBlock, Nature},
        moves::{Move, MoveId, LearnedMove},
        types::PokemonType,
    },
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct IndividualPokemon {
    pub id: Uuid,
    pub species_id: SpeciesId,
    pub nickname: Option<String>,
    
    // 基础属性
    pub level: u8,
    pub experience: u32,
    pub gender: Gender,
    pub nature: Nature,
    pub ability_id: u32,
    pub is_shiny: bool,
    
    // 个体值 (Individual Values)
    pub ivs: StatBlock,
    
    // 努力值 (Effort Values)  
    pub evs: StatBlock,
    
    // 当前状态
    pub current_hp: u16,
    pub status_conditions: Vec<StatusCondition>,
    pub friendship: u8,
    
    // 技能
    pub moves: Vec<LearnedMove>,
    
    // 道具
    pub held_item: Option<u32>,
    
    // 遇见信息
    pub encounter_info: EncounterInfo,
    
    // 标记和特殊状态
    pub marks: Vec<PokemonMark>,
    pub is_egg: bool,
    pub egg_cycles: Option<u16>,
    
    // 缓存计算的属性值
    #[serde(skip)]
    pub cached_stats: Option<StatBlock>,
    
    // 战斗状态
    #[serde(skip)]
    pub battle_stats: Option<BattleStats>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Genderless,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCondition {
    pub condition_type: StatusType,
    pub duration: Option<u8>, // None表示永久
    pub severity: u8,
    pub applied_turn: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    // 主要状态 (只能有一个)
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned,
    Sleep,
    
    // 次要状态 (可以同时存在多个)
    Confusion,
    Attraction,
    Curse,
    Flinch,
    Leech,
    Nightmare,
    Perish,
    Trap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterInfo {
    pub location: String,
    pub date_caught: DateTime<Utc>,
    pub level_caught: u8,
    pub ball_type: u8,
    pub trainer_id: Option<u32>,
    pub trainer_name: Option<String>,
    pub method: EncounterMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterMethod {
    WildGrass,
    WildWater,
    WildCave,
    Trade,
    Gift,
    Egg,
    Raid,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokemonMark {
    Circle,
    Triangle,
    Square,
    Heart,
    Star,
    Diamond,
    Shiny,
    Origin,
    Partner,
}

#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    pub stat_stages: HashMap<StatType, i8>, // -6 to +6
    pub accuracy_stage: i8,
    pub evasion_stage: i8,
    pub critical_hit_stage: u8,
    pub type_changes: Vec<PokemonType>,
    pub ability_suppressed: bool,
    pub last_move_used: Option<MoveId>,
    pub consecutive_turns: u8,
}

impl IndividualPokemon {
    pub fn new(
        species: &PokemonSpecies,
        level: u8,
        rng: &mut RandomGenerator,
    ) -> GameResult<Self> {
        if level == 0 || level > 100 {
            return Err(GameError::InvalidPokemonData("Level must be 1-100".to_string()));
        }

        let id = Uuid::new_v4();
        let gender = Self::determine_gender(species, rng)?;
        let nature = Nature::random(rng);
        let is_shiny = rng.chance(species.shiny_rate);
        
        // 生成随机IV值 (0-31)
        let ivs = StatBlock {
            hp: rng.range(0, 32) as u16,
            attack: rng.range(0, 32) as u16,
            defense: rng.range(0, 32) as u16,
            special_attack: rng.range(0, 32) as u16,
            special_defense: rng.range(0, 32) as u16,
            speed: rng.range(0, 32) as u16,
        };

        // 初始化EV值为0
        let evs = StatBlock::default();
        
        // 计算经验值
        let experience = Self::calculate_experience_for_level(level, &species.growth_rate);
        
        // 选择随机能力
        let ability_id = if rng.probability() < 0.8 {
            species.abilities.ability1
        } else if let Some(ability2) = species.abilities.ability2 {
            ability2
        } else {
            species.abilities.ability1
        };

        // 生成初始技能
        let moves = Self::generate_initial_moves(species, level, rng)?;
        
        // 计算当前HP
        let max_hp = Self::calculate_hp_stat(&ivs, &evs, species.base_stats.hp, level);
        
        let encounter_info = EncounterInfo {
            location: "Unknown".to_string(),
            date_caught: Utc::now(),
            level_caught: level,
            ball_type: 1, // 普通精灵球
            trainer_id: None,
            trainer_name: None,
            method: EncounterMethod::WildGrass,
        };

        Ok(IndividualPokemon {
            id,
            species_id: species.id,
            nickname: None,
            level,
            experience,
            gender,
            nature,
            ability_id,
            is_shiny,
            ivs,
            evs,
            current_hp: max_hp,
            status_conditions: Vec::new(),
            friendship: species.base_friendship,
            moves,
            held_item: None,
            encounter_info,
            marks: Vec::new(),
            is_egg: false,
            egg_cycles: None,
            cached_stats: None,
            battle_stats: None,
        })
    }

    pub fn from_egg(
        species: &PokemonSpecies,
        parent1: &IndividualPokemon,
        parent2: Option<&IndividualPokemon>,
        rng: &mut RandomGenerator,
    ) -> GameResult<Self> {
        let mut pokemon = Self::new(species, 1, rng)?;
        
        // 蛋的特殊处理
        pokemon.is_egg = true;
        pokemon.egg_cycles = Some(species.egg_cycles);
        pokemon.current_hp = 0; // 蛋不能战斗
        
        // 遗传IV值
        pokemon.ivs = Self::inherit_ivs(&parent1.ivs, parent2.map(|p| &p.ivs), rng);
        
        // 遗传性格 (有概率)
        if rng.probability() < 0.5 {
            pokemon.nature = parent1.nature;
        } else if let Some(parent2) = parent2 {
            if rng.probability() < 0.5 {
                pokemon.nature = parent2.nature;
            }
        }
        
        // 遗传能力 (隐藏能力有概率遗传)
        if let Some(hidden_ability) = species.abilities.hidden_ability {
            if parent1.ability_id == hidden_ability || 
               parent2.map_or(false, |p| p.ability_id == hidden_ability) {
                if rng.probability() < 0.6 {
                    pokemon.ability_id = hidden_ability;
                }
            }
        }
        
        pokemon.encounter_info.method = EncounterMethod::Egg;
        
        Ok(pokemon)
    }

    fn determine_gender(species: &PokemonSpecies, rng: &mut RandomGenerator) -> GameResult<Gender> {
        match species.gender_ratio {
            -1 => Ok(Gender::Genderless),
            0 => Ok(Gender::Male),
            8 => Ok(Gender::Female),
            ratio => {
                if rng.range(0, 8) < ratio {
                    Ok(Gender::Female)
                } else {
                    Ok(Gender::Male)
                }
            }
        }
    }

    fn generate_initial_moves(
        species: &PokemonSpecies,
        level: u8,
        rng: &mut RandomGenerator,
    ) -> GameResult<Vec<LearnedMove>> {
        let mut moves = Vec::new();
        
        // 获取可学习的招式
        let available_moves: Vec<_> = species.level_up_moves.iter()
            .filter(|(learn_level, _)| *learn_level <= level)
            .collect();
        
        if available_moves.is_empty() {
            return Err(GameError::InvalidPokemonData("No available moves for level".to_string()));
        }
        
        // 从最近学会的招式中选择最多4个
        let mut selected_moves = available_moves;
        selected_moves.sort_by_key(|(level, _)| *level);
        selected_moves.reverse(); // 最新的招式优先
        
        for (_, move_id) in selected_moves.iter().take(4) {
            moves.push(LearnedMove {
                move_id: *move_id,
                current_pp: 20, // 默认PP，应该从招式数据中获取
                max_pp: 20,
                pp_ups: 0,
            });
        }
        
        // 如果没有足够的招式，添加基础招式
        if moves.is_empty() {
            moves.push(LearnedMove {
                move_id: 1, // 撞击
                current_pp: 35,
                max_pp: 35,
                pp_ups: 0,
            });
        }
        
        Ok(moves)
    }

    fn inherit_ivs(
        parent1_ivs: &StatBlock,
        parent2_ivs: Option<&StatBlock>,
        rng: &mut RandomGenerator,
    ) -> StatBlock {
        let mut inherited_ivs = StatBlock::default();
        
        // 每个属性有50%几率继承父母之一，50%随机
        let stats = [
            (&mut inherited_ivs.hp, parent1_ivs.hp, parent2_ivs.map_or(0, |p| p.hp)),
            (&mut inherited_ivs.attack, parent1_ivs.attack, parent2_ivs.map_or(0, |p| p.attack)),
            (&mut inherited_ivs.defense, parent1_ivs.defense, parent2_ivs.map_or(0, |p| p.defense)),
            (&mut inherited_ivs.special_attack, parent1_ivs.special_attack, parent2_ivs.map_or(0, |p| p.special_attack)),
            (&mut inherited_ivs.special_defense, parent1_ivs.special_defense, parent2_ivs.map_or(0, |p| p.special_defense)),
            (&mut inherited_ivs.speed, parent1_ivs.speed, parent2_ivs.map_or(0, |p| p.speed)),
        ];
        
        for (target, parent1_val, parent2_val) in stats {
            if rng.probability() < 0.5 {
                // 继承父母
                if parent2_val > 0 && rng.probability() < 0.5 {
                    *target = parent2_val;
                } else {
                    *target = parent1_val;
                }
            } else {
                // 随机生成
                *target = rng.range(0, 32) as u16;
            }
        }
        
        inherited_ivs
    }

    pub fn calculate_stats(&self, species: &PokemonSpecies) -> StatBlock {
        StatBlock {
            hp: Self::calculate_hp_stat(&self.ivs, &self.evs, species.base_stats.hp, self.level),
            attack: Self::calculate_stat(&self.ivs, &self.evs, species.base_stats.attack, self.level, StatType::Attack, self.nature),
            defense: Self::calculate_stat(&self.ivs, &self.evs, species.base_stats.defense, self.level, StatType::Defense, self.nature),
            special_attack: Self::calculate_stat(&self.ivs, &self.evs, species.base_stats.special_attack, self.level, StatType::SpecialAttack, self.nature),
            special_defense: Self::calculate_stat(&self.ivs, &self.evs, species.base_stats.special_defense, self.level, StatType::SpecialDefense, self.nature),
            speed: Self::calculate_stat(&self.ivs, &self.evs, species.base_stats.speed, self.level, StatType::Speed, self.nature),
        }
    }

    fn calculate_hp_stat(ivs: &StatBlock, evs: &StatBlock, base: u16, level: u8) -> u16 {
        if base == 1 { // Shedinja特殊情况
            return 1;
        }
        
        let iv = ivs.hp as u32;
        let ev = (evs.hp / 4) as u32; // EV值除以4
        let base = base as u32;
        let level = level as u32;
        
        ((((2 * base + iv + ev) * level) / 100) + level + 10) as u16
    }

    fn calculate_stat(
        ivs: &StatBlock,
        evs: &StatBlock,
        base: u16,
        level: u8,
        stat_type: StatType,
        nature: Nature,
    ) -> u16 {
        let iv = match stat_type {
            StatType::Attack => ivs.attack,
            StatType::Defense => ivs.defense,
            StatType::SpecialAttack => ivs.special_attack,
            StatType::SpecialDefense => ivs.special_defense,
            StatType::Speed => ivs.speed,
            StatType::HP => ivs.hp,
        } as u32;
        
        let ev = (match stat_type {
            StatType::Attack => evs.attack,
            StatType::Defense => evs.defense,
            StatType::SpecialAttack => evs.special_attack,
            StatType::SpecialDefense => evs.special_defense,
            StatType::Speed => evs.speed,
            StatType::HP => evs.hp,
        } / 4) as u32;
        
        let base = base as u32;
        let level = level as u32;
        
        let base_stat = ((((2 * base + iv + ev) * level) / 100) + 5) as f32;
        let nature_modifier = nature.get_stat_modifier(stat_type);
        
        (base_stat * nature_modifier) as u16
    }

    fn calculate_experience_for_level(level: u8, growth_rate: &str) -> u32 {
        let level = level as u32;
        
        match growth_rate {
            "fast" => (4 * level.pow(3)) / 5,
            "medium" => level.pow(3),
            "slow" => (5 * level.pow(3)) / 4,
            "medium_slow" => {
                if level <= 50 {
                    (level.pow(3) * (100 - level)) / 50
                } else if level <= 68 {
                    (level.pow(3) * (150 - level)) / 100
                } else if level <= 98 {
                    level.pow(3) * ((1911 - 10 * level) / 3) / 500
                } else {
                    (level.pow(3) * (160 - level)) / 100
                }
            },
            "fluctuating" => {
                if level <= 15 {
                    level.pow(3) * ((level + 1) / 3 + 24) / 50
                } else if level <= 36 {
                    level.pow(3) * (level + 14) / 50
                } else {
                    level.pow(3) * (level / 2 + 32) / 50
                }
            },
            "erratic" => {
                if level <= 50 {
                    level.pow(3) * (100 - level) / 50
                } else if level <= 68 {
                    level.pow(3) * (150 - level) / 100
                } else if level <= 98 {
                    level.pow(3) * (1911 - 10 * level) / 3 / 500
                } else {
                    level.pow(3) * (160 - level) / 100
                }
            },
            _ => level.pow(3), // 默认medium
        }
    }

    // 实用方法
    pub fn get_display_name(&self) -> &str {
        self.nickname.as_deref().unwrap_or("Pokemon")
    }

    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }

    pub fn get_hp_percentage(&self, species: &PokemonSpecies) -> f32 {
        if self.is_egg {
            return 0.0;
        }
        
        let max_hp = self.get_cached_stats(species).hp;
        if max_hp == 0 {
            0.0
        } else {
            (self.current_hp as f32) / (max_hp as f32)
        }
    }

    pub fn get_cached_stats(&self, species: &PokemonSpecies) -> StatBlock {
        // 在实际实现中，这里会使用缓存
        self.calculate_stats(species)
    }

    pub fn add_experience(&mut self, amount: u32, species: &PokemonSpecies) -> bool {
        if self.level >= 100 {
            return false;
        }
        
        self.experience += amount;
        let new_level = self.calculate_level_from_experience(&species.growth_rate);
        
        if new_level > self.level {
            self.level_up_to(new_level, species);
            true
        } else {
            false
        }
    }

    fn calculate_level_from_experience(&self, growth_rate: &str) -> u8 {
        for level in 1..=100 {
            if Self::calculate_experience_for_level(level, growth_rate) > self.experience {
                return level - 1;
            }
        }
        100
    }

    fn level_up_to(&mut self, new_level: u8, species: &PokemonSpecies) {
        let old_level = self.level;
        self.level = new_level;
        
        // 学习新招式
        for level in (old_level + 1)..=new_level {
            if let Some(moves) = species.level_up_moves.get(&level) {
                for &move_id in moves {
                    self.try_learn_move(move_id);
                }
            }
        }
        
        // 重新计算HP
        let new_max_hp = Self::calculate_hp_stat(&self.ivs, &self.evs, species.base_stats.hp, self.level);
        let hp_increase = new_max_hp.saturating_sub(self.current_hp);
        self.current_hp += hp_increase;
        
        // 清除缓存的属性值
        self.cached_stats = None;
    }

    fn try_learn_move(&mut self, move_id: MoveId) -> bool {
        // 检查是否已经学会
        if self.moves.iter().any(|m| m.move_id == move_id) {
            return false;
        }
        
        let learned_move = LearnedMove {
            move_id,
            current_pp: 20, // 应该从招式数据获取
            max_pp: 20,
            pp_ups: 0,
        };
        
        if self.moves.len() < 4 {
            self.moves.push(learned_move);
            true
        } else {
            // 需要替换招式，在实际游戏中会询问玩家
            // 这里简单替换第一个招式
            self.moves[0] = learned_move;
            true
        }
    }

    pub fn has_status(&self, status_type: StatusType) -> bool {
        self.status_conditions.iter().any(|s| s.condition_type == status_type)
    }

    pub fn apply_status(&mut self, status: StatusCondition) -> bool {
        // 检查是否已有相同状态
        if self.has_status(status.condition_type) {
            return false;
        }
        
        // 主要状态只能有一个
        if self.is_major_status(status.condition_type) {
            self.status_conditions.retain(|s| !self.is_major_status(s.condition_type));
        }
        
        self.status_conditions.push(status);
        true
    }

    fn is_major_status(&self, status_type: StatusType) -> bool {
        matches!(status_type, 
            StatusType::Burn | StatusType::Freeze | StatusType::Paralysis |
            StatusType::Poison | StatusType::BadlyPoisoned | StatusType::Sleep
        )
    }

    pub fn hatch_egg(&mut self, species: &PokemonSpecies) -> GameResult<()> {
        if !self.is_egg {
            return Err(GameError::InvalidPokemonData("Pokemon is not an egg".to_string()));
        }
        
        self.is_egg = false;
        self.egg_cycles = None;
        self.level = 1;
        
        // 重新计算HP
        let max_hp = Self::calculate_hp_stat(&self.ivs, &self.evs, species.base_stats.hp, 1);
        self.current_hp = max_hp;
        
        // 设置初始友好度
        self.friendship = species.base_friendship;
        
        Ok(())
    }
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Genderless
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::species::PokemonSpecies;

    #[test]
    fn test_pokemon_creation() {
        let species = PokemonSpecies::default();
        let mut rng = RandomGenerator::new();
        
        let pokemon = IndividualPokemon::new(&species, 5, &mut rng).unwrap();
        
        assert_eq!(pokemon.level, 5);
        assert!(!pokemon.is_egg);
        assert!(!pokemon.moves.is_empty());
    }

    #[test]
    fn test_stat_calculation() {
        let species = PokemonSpecies::default();
        let mut rng = RandomGenerator::new();
        let pokemon = IndividualPokemon::new(&species, 50, &mut rng).unwrap();
        
        let stats = pokemon.calculate_stats(&species);
        assert!(stats.hp > 0);
        assert!(stats.attack > 0);
    }

    #[test]
    fn test_experience_and_leveling() {
        let species = PokemonSpecies::default();
        let mut rng = RandomGenerator::new();
        let mut pokemon = IndividualPokemon::new(&species, 1, &mut rng).unwrap();
        
        let leveled_up = pokemon.add_experience(1000, &species);
        assert!(leveled_up);
        assert!(pokemon.level > 1);
    }

    #[test]
    fn test_status_conditions() {
        let species = PokemonSpecies::default();
        let mut rng = RandomGenerator::new();
        let mut pokemon = IndividualPokemon::new(&species, 10, &mut rng).unwrap();
        
        let status = StatusCondition {
            condition_type: StatusType::Burn,
            duration: None,
            severity: 1,
            applied_turn: 1,
        };
        
        assert!(pokemon.apply_status(status));
        assert!(pokemon.has_status(StatusType::Burn));
        assert!(!pokemon.apply_status(StatusCondition {
            condition_type: StatusType::Burn,
            duration: None,
            severity: 1,
            applied_turn: 2,
        })); // 不能重复添加相同状态
    }
}