// 宝可梦种族数据模块
// 开发心理：定义宝可梦的基础属性和种族特征
// 设计原则：数据驱动、可扩展、支持模组化

use super::{BaseStats, AbilityId, MoveId, EvolutionChain, SpeciesId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lazy_static::lazy_static;
use log::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSpecies {
    pub id: SpeciesId,
    pub name: String,
    pub base_stats: BaseStats,
    pub types: Vec<PokemonType>,
    pub abilities: Vec<AbilityId>,
    pub hidden_ability: Option<AbilityId>,
    pub catch_rate: u8,
    pub base_experience: u32,
    pub base_friendship: u8,
    pub growth_rate: GrowthRate,
    pub egg_groups: Vec<EggGroup>,
    pub gender_ratio: GenderRatio,
    pub height: u16, // cm
    pub weight: u16, // kg
    pub color: Color,
    pub shape: Shape,
    pub habitat: Option<Habitat>,
    pub generation: u8,
    pub is_legendary: bool,
    pub is_mythical: bool,
    pub evolution_chain: Option<EvolutionChain>,
    pub learnable_moves: Vec<LearnableMove>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokemonType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthRate {
    Fast,       // 800,000 exp to level 100
    MediumFast, // 1,000,000 exp to level 100
    MediumSlow, // 1,059,860 exp to level 100
    Slow,       // 1,250,000 exp to level 100
    Erratic,    // 600,000 exp to level 100
    Fluctuating, // 1,640,000 exp to level 100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EggGroup {
    Monster,
    Water1,
    Water2,
    Water3,
    Bug,
    Flying,
    Field,
    Fairy,
    Grass,
    HumanLike,
    Mineral,
    Amorphous,
    Ditto,
    Dragon,
    Undiscovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenderRatio {
    AlwaysMale,
    SevenEighthsMale,    // 87.5% male
    ThreeQuartersMale,   // 75% male
    Equal,               // 50% male, 50% female
    OneQuarterMale,      // 25% male
    OneEighthMale,       // 12.5% male
    AlwaysFemale,
    Genderless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Blue,
    Yellow,
    Green,
    Black,
    Brown,
    Purple,
    Gray,
    White,
    Pink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Ball,
    Squiggle,
    Fish,
    Arms,
    Blob,
    Upright,
    Legs,
    Quadruped,
    Wings,
    Tentacles,
    Heads,
    Humanoid,
    BugWings,
    Armor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Habitat {
    Cave,
    Forest,
    Grassland,
    Mountain,
    Rare,
    RoughTerrain,
    Sea,
    Urban,
    WatersEdge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnableMove {
    pub move_id: MoveId,
    pub learn_method: LearnMethod,
    pub level: Option<u8>,
    pub machine_id: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnMethod {
    LevelUp,
    TM,
    HM,
    Tutor,
    Egg,
    Special,
}

impl PokemonSpecies {
    // 根据ID获取种族（静态方法）
    pub fn get(species_id: SpeciesId) -> Option<&'static Self> {
        get_species(species_id)
    }
    
    // 获取所有种族
    pub fn get_all() -> &'static HashMap<SpeciesId, PokemonSpecies> {
        get_all_species()
    }
    
    pub fn get_by_name(name: &str) -> Option<&'static Self> {
        SPECIES_DATABASE.values().find(|species| species.name.eq_ignore_ascii_case(name))
    }
    
    pub fn generate_gender(&self) -> crate::pokemon::Gender {
        use crate::pokemon::Gender;
        use fastrand;
        
        match self.gender_ratio {
            GenderRatio::AlwaysMale => Gender::Male,
            GenderRatio::AlwaysFemale => Gender::Female,
            GenderRatio::Genderless => Gender::Genderless,
            GenderRatio::Equal => {
                if fastrand::bool() { Gender::Male } else { Gender::Female }
            },
            GenderRatio::SevenEighthsMale => {
                if fastrand::u8(1..=8) <= 7 { Gender::Male } else { Gender::Female }
            },
            GenderRatio::ThreeQuartersMale => {
                if fastrand::u8(1..=4) <= 3 { Gender::Male } else { Gender::Female }
            },
            GenderRatio::OneQuarterMale => {
                if fastrand::u8(1..=4) == 1 { Gender::Male } else { Gender::Female }
            },
            GenderRatio::OneEighthMale => {
                if fastrand::u8(1..=8) == 1 { Gender::Male } else { Gender::Female }
            },
        }
    }
    
    pub fn experience_for_level(&self, level: u8) -> u32 {
        if level <= 1 {
            return 0;
        }
        
        let n = level as u32;
        match self.growth_rate {
            GrowthRate::Fast => (4 * n.pow(3)) / 5,
            GrowthRate::MediumFast => n.pow(3),
            GrowthRate::MediumSlow => {
                (6 * n.pow(3)) / 5 - 15 * n.pow(2) + 100 * n - 140
            },
            GrowthRate::Slow => (5 * n.pow(3)) / 4,
            GrowthRate::Erratic => {
                if n <= 50 {
                    (n.pow(3) * (100 - n)) / 50
                } else if n <= 68 {
                    (n.pow(3) * (150 - n)) / 100
                } else if n <= 98 {
                    (n.pow(3) * ((1911 - 10 * n) / 3)) / 500
                } else {
                    (n.pow(3) * (160 - n)) / 100
                }
            },
            GrowthRate::Fluctuating => {
                if n <= 15 {
                    n.pow(3) * ((((n + 1) / 3) + 24) / 50)
                } else if n <= 36 {
                    n.pow(3) * ((n + 14) / 50)
                } else {
                    n.pow(3) * (((n / 2) + 32) / 50)
                }
            },
        }
    }
    
    pub fn get_learnable_moves_at_level(&self, level: u8) -> Vec<MoveId> {
        self.learnable_moves
            .iter()
            .filter(|lm| {
                matches!(lm.learn_method, LearnMethod::LevelUp) &&
                lm.level == Some(level)
            })
            .map(|lm| lm.move_id)
            .collect()
    }
    
    pub fn get_random_ability(&self) -> AbilityId {
        if self.abilities.is_empty() {
            return 0; // 默认能力
        }
        
        let idx = fastrand::usize(..self.abilities.len());
        self.abilities[idx]
    }
    
    pub fn get_evolution_chains(&self) -> Vec<EvolutionChain> {
        self.evolution_chain.as_ref().map(|ec| vec![ec.clone()]).unwrap_or_default()
    }
    
    pub fn is_compatible_for_breeding(&self, other: &PokemonSpecies) -> bool {
        if self.egg_groups.contains(&EggGroup::Undiscovered) ||
           other.egg_groups.contains(&EggGroup::Undiscovered) {
            return false;
        }
        
        if self.egg_groups.contains(&EggGroup::Ditto) ||
           other.egg_groups.contains(&EggGroup::Ditto) {
            return true;
        }
        
        self.egg_groups.iter().any(|group| other.egg_groups.contains(group))
    }
}

impl GrowthRate {
    pub fn max_experience(&self) -> u32 {
        match self {
            GrowthRate::Fast => 800_000,
            GrowthRate::MediumFast => 1_000_000,
            GrowthRate::MediumSlow => 1_059_860,
            GrowthRate::Slow => 1_250_000,
            GrowthRate::Erratic => 600_000,
            GrowthRate::Fluctuating => 1_640_000,
        }
    }
}

// 第一个SPECIES_DATABASE定义已移除，保留下面的定义

// 全局种族数据库
lazy_static! {
    static ref SPECIES_DATABASE: HashMap<SpeciesId, PokemonSpecies> = {
        let mut db = HashMap::new();
        add_gen1_pokemon(&mut db);
        debug!("宝可梦种族数据库初始化完成，共加载了{}个种族", db.len());
        db
    };
}

// 根据种族ID获取种族数据
pub fn get_species(species_id: SpeciesId) -> Option<&'static PokemonSpecies> {
    SPECIES_DATABASE.get(&species_id)
}

// 获取所有种族数据
pub fn get_all_species() -> &'static HashMap<SpeciesId, PokemonSpecies> {
    &SPECIES_DATABASE
}

fn add_gen1_pokemon(db: &mut HashMap<SpeciesId, PokemonSpecies>) {
    // 妙蛙种子 #001
    db.insert(1, PokemonSpecies {
        id: 1,
        name: "妙蛙种子".to_string(),
        base_stats: BaseStats {
            hp: 45,
            attack: 49,
            defense: 49,
            special_attack: 65,
            special_defense: 65,
            speed: 45,
        },
        types: vec![PokemonType::Grass, PokemonType::Poison],
        abilities: vec![1], // 茂盛
        hidden_ability: Some(2), // 叶绿素
        catch_rate: 45,
        base_experience: 64,
        base_friendship: 70,
        growth_rate: GrowthRate::MediumSlow,
        egg_groups: vec![EggGroup::Monster, EggGroup::Grass],
        gender_ratio: GenderRatio::SevenEighthsMale,
        height: 70,
        weight: 69,
        color: Color::Green,
        shape: Shape::Quadruped,
        habitat: Some(Habitat::Grassland),
        generation: 1,
        is_legendary: false,
        is_mythical: false,
        evolution_chain: None, // 简化，实际应该包含进化链
        learnable_moves: vec![
            LearnableMove {
                move_id: 1, // 撞击
                learn_method: LearnMethod::LevelUp,
                level: Some(1),
                machine_id: None,
            },
            LearnableMove {
                move_id: 2, // 叫声
                learn_method: LearnMethod::LevelUp,
                level: Some(3),
                machine_id: None,
            },
            LearnableMove {
                move_id: 3, // 藤鞭
                learn_method: LearnMethod::LevelUp,
                level: Some(7),
                machine_id: None,
            },
        ],
    });
    
    // 小火龙 #004
    db.insert(4, PokemonSpecies {
        id: 4,
        name: "小火龙".to_string(),
        base_stats: BaseStats {
            hp: 39,
            attack: 52,
            defense: 43,
            special_attack: 60,
            special_defense: 50,
            speed: 65,
        },
        types: vec![PokemonType::Fire],
        abilities: vec![3], // 猛火
        hidden_ability: Some(4), // 太阳之力
        catch_rate: 45,
        base_experience: 62,
        base_friendship: 70,
        growth_rate: GrowthRate::MediumSlow,
        egg_groups: vec![EggGroup::Monster, EggGroup::Dragon],
        gender_ratio: GenderRatio::SevenEighthsMale,
        height: 60,
        weight: 85,
        color: Color::Red,
        shape: Shape::Upright,
        habitat: Some(Habitat::Mountain),
        generation: 1,
        is_legendary: false,
        is_mythical: false,
        evolution_chain: None,
        learnable_moves: vec![
            LearnableMove {
                move_id: 1, // 撞击
                learn_method: LearnMethod::LevelUp,
                level: Some(1),
                machine_id: None,
            },
            LearnableMove {
                move_id: 2, // 叫声
                learn_method: LearnMethod::LevelUp,
                level: Some(3),
                machine_id: None,
            },
            LearnableMove {
                move_id: 52, // 火花
                learn_method: LearnMethod::LevelUp,
                level: Some(7),
                machine_id: None,
            },
        ],
    });
    
    // 杰尼龟 #007
    db.insert(7, PokemonSpecies {
        id: 7,
        name: "杰尼龟".to_string(),
        base_stats: BaseStats {
            hp: 44,
            attack: 48,
            defense: 65,
            special_attack: 50,
            special_defense: 64,
            speed: 43,
        },
        types: vec![PokemonType::Water],
        abilities: vec![5], // 激流
        hidden_ability: Some(6), // 雨盘
        catch_rate: 45,
        base_experience: 63,
        base_friendship: 70,
        growth_rate: GrowthRate::MediumSlow,
        egg_groups: vec![EggGroup::Monster, EggGroup::Water1],
        gender_ratio: GenderRatio::SevenEighthsMale,
        height: 50,
        weight: 90,
        color: Color::Blue,
        shape: Shape::Upright,
        habitat: Some(Habitat::WatersEdge),
        generation: 1,
        is_legendary: false,
        is_mythical: false,
        evolution_chain: None,
        learnable_moves: vec![
            LearnableMove {
                move_id: 1, // 撞击
                learn_method: LearnMethod::LevelUp,
                level: Some(1),
                machine_id: None,
            },
            LearnableMove {
                move_id: 39, // 尾巴摇摆
                learn_method: LearnMethod::LevelUp,
                level: Some(4),
                machine_id: None,
            },
            LearnableMove {
                move_id: 55, // 水枪
                learn_method: LearnMethod::LevelUp,
                level: Some(7),
                machine_id: None,
            },
        ],
    });
    
    // 皮卡丘 #025
    db.insert(25, PokemonSpecies {
        id: 25,
        name: "皮卡丘".to_string(),
        base_stats: BaseStats {
            hp: 35,
            attack: 55,
            defense: 40,
            special_attack: 50,
            special_defense: 50,
            speed: 90,
        },
        types: vec![PokemonType::Electric],
        abilities: vec![7], // 静电
        hidden_ability: Some(8), // 避雷针
        catch_rate: 190,
        base_experience: 112,
        base_friendship: 70,
        growth_rate: GrowthRate::MediumFast,
        egg_groups: vec![EggGroup::Field, EggGroup::Fairy],
        gender_ratio: GenderRatio::Equal,
        height: 40,
        weight: 60,
        color: Color::Yellow,
        shape: Shape::Quadruped,
        habitat: Some(Habitat::Forest),
        generation: 1,
        is_legendary: false,
        is_mythical: false,
        evolution_chain: None,
        learnable_moves: vec![
            LearnableMove {
                move_id: 84, // 电击
                learn_method: LearnMethod::LevelUp,
                level: Some(1),
                machine_id: None,
            },
            LearnableMove {
                move_id: 39, // 尾巴摇摆
                learn_method: LearnMethod::LevelUp,
                level: Some(5),
                machine_id: None,
            },
            LearnableMove {
                move_id: 86, // 十万伏特
                learn_method: LearnMethod::LevelUp,
                level: Some(15),
                machine_id: None,
            },
        ],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_species_database() {
        let pikachu = PokemonSpecies::get(25).unwrap();
        assert_eq!(pikachu.name, "皮卡丘");
        assert_eq!(pikachu.types, vec![PokemonType::Electric]);
        assert_eq!(pikachu.base_stats.speed, 90);
    }
    
    #[test]
    fn test_gender_generation() {
        let pikachu = PokemonSpecies::get(25).unwrap();
        let gender = pikachu.generate_gender();
        assert!(matches!(gender, crate::pokemon::Gender::Male | crate::pokemon::Gender::Female));
    }
    
    #[test]
    fn test_experience_calculation() {
        let pikachu = PokemonSpecies::get(25).unwrap();
        let exp_50 = pikachu.experience_for_level(50);
        let exp_100 = pikachu.experience_for_level(100);
        
        assert!(exp_50 < exp_100);
        assert!(exp_100 <= 1_000_000); // MediumFast growth rate max
    }
    
    #[test]
    fn test_learnable_moves() {
        let pikachu = PokemonSpecies::get(25).unwrap();
        let level_1_moves = pikachu.get_learnable_moves_at_level(1);
        assert!(!level_1_moves.is_empty());
    }
}