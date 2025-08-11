/*
* 开发心理过程：
* 1. 设计完整的训练师系统，包括玩家和NPC训练师
* 2. 实现等级系统、经验值计算、徽章收集
* 3. 支持训练师个性化设置，包括外观、偏好等
* 4. 集成金钱系统、背包管理、成就系统
* 5. 提供训练师AI和行为模式系统
* 6. 实现训练师战斗记录和统计系统
* 7. 支持多人游戏的训练师交互功能
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        individual::IndividualPokemon,
        species::SpeciesId,
    },
    world::location::LocationId,
    utils::random::RandomGenerator,
};

pub type TrainerId = Uuid;

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Trainer {
    pub id: TrainerId,
    pub name: String,
    pub trainer_class: TrainerClass,
    pub level: TrainerLevel,
    pub experience: u64,
    pub money: u32,
    pub position: Vec2,
    pub current_location: Option<LocationId>,
    pub appearance: TrainerAppearance,
    pub personality: TrainerPersonality,
    pub battle_stats: BattleStatistics,
    pub badges: Vec<Badge>,
    pub achievements: Vec<Achievement>,
    pub items: HashMap<u32, u32>, // item_id -> quantity
    pub key_items: Vec<u32>,
    pub settings: TrainerSettings,
    pub save_data: TrainerSaveData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainerClass {
    // 玩家类型
    Player,
    Rival,
    
    // 普通训练师
    Youngster,
    Lass,
    BugCatcher,
    Fisherman,
    Picnicker,
    Hiker,
    Beauty,
    Psychic,
    Blackbelt,
    Biker,
    Rocker,
    Juggler,
    Tamer,
    Birdkeeper,
    
    // 特殊训练师
    GymLeader,
    EliteFour,
    Champion,
    TeamRocket,
    Scientist,
    Gentleman,
    Lady,
    
    // 专业训练师
    Breeder,
    Coordinator,
    Ranger,
    
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrainerLevel(pub u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerAppearance {
    pub sprite_id: u32,
    pub gender: Gender,
    pub hair_color: Color,
    pub eye_color: Color,
    pub skin_tone: Color,
    pub outfit: OutfitSet,
    pub accessories: Vec<Accessory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    NonBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutfitSet {
    pub hat: Option<u32>,
    pub top: u32,
    pub bottom: u32,
    pub shoes: u32,
    pub bag: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accessory {
    pub accessory_type: AccessoryType,
    pub item_id: u32,
    pub position: AccessoryPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessoryType {
    Hat,
    Glasses,
    Necklace,
    Bracelet,
    Ring,
    Watch,
    Badge,
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessoryPosition {
    Head,
    Face,
    Neck,
    Wrist,
    Finger,
    Chest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerPersonality {
    pub personality_type: PersonalityType,
    pub favorite_type: Option<crate::pokemon::types::PokemonType>,
    pub battle_style: BattleStyle,
    pub catchphrase: String,
    pub bio: String,
    pub traits: Vec<PersonalityTrait>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonalityType {
    Aggressive,     // 激进型
    Defensive,      // 防守型
    Balanced,       // 平衡型
    Technical,      // 技术型
    Casual,         // 休闲型
    Competitive,    // 竞技型
    Collector,      // 收集型
    Explorer,       // 探险型
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStyle {
    Aggressive,     // 攻击导向
    Defensive,      // 防守导向
    StatusBased,    // 状态导向
    TypeSpecialist, // 属性专精
    Balanced,       // 平衡战术
    Unpredictable,  // 不可预测
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Brave,          // 勇敢
    Careful,        // 谨慎
    Friendly,       // 友善
    Competitive,    // 好胜
    Curious,        // 好奇
    Patient,        // 耐心
    Impulsive,      // 冲动
    Analytical,     // 分析性
    Creative,       // 创造性
    Loyal,          // 忠诚
    Custom(String), // 自定义特质
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattleStatistics {
    pub total_battles: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub wild_pokemon_caught: u32,
    pub pokemon_released: u32,
    pub steps_walked: u64,
    pub time_played_seconds: u64,
    pub money_earned: u64,
    pub money_spent: u64,
    pub items_used: HashMap<u32, u32>,
    pub pokemon_evolved: u32,
    pub eggs_hatched: u32,
    pub shinies_encountered: u32,
    pub legendary_encounters: u32,
    pub gym_battles_won: u32,
    pub tournament_wins: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub gym_leader: String,
    pub location: LocationId,
    pub obtained_date: String,
    pub level_requirement: u8,
    pub badge_type: BadgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeType {
    Gym,        // 道馆徽章
    Contest,    // 比赛徽章
    Special,    // 特殊徽章
    Event,      // 活动徽章
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub progress: u32,
    pub target: u32,
    pub is_completed: bool,
    pub completion_date: Option<String>,
    pub reward: Option<AchievementReward>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementCategory {
    Battle,     // 战斗相关
    Collection, // 收集相关
    Exploration,// 探索相关
    Social,     // 社交相关
    Special,    // 特殊成就
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    Money(u32),
    Item(u32, u32),
    Pokemon(SpeciesId, u8),
    Title(String),
    Cosmetic(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerSettings {
    pub battle_animations: bool,
    pub battle_text_speed: TextSpeed,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub auto_save: bool,
    pub nickname_pokemon: bool,
    pub confirm_actions: bool,
    pub battle_mode: BattleMode,
    pub ui_theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextSpeed {
    Slow,
    Normal,
    Fast,
    Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleMode {
    Set,    // SET模式：对手换Pokemon时不提示
    Switch, // SWITCH模式：对手换Pokemon时可选择换Pokemon
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerSaveData {
    pub save_version: u32,
    pub creation_date: String,
    pub last_save_date: String,
    pub play_time: PlayTime,
    pub game_progress: GameProgress,
    pub flags: HashMap<String, bool>,
    pub variables: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayTime {
    pub hours: u32,
    pub minutes: u8,
    pub seconds: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProgress {
    pub story_flags: HashMap<String, bool>,
    pub completed_quests: Vec<String>,
    pub unlocked_locations: Vec<LocationId>,
    pub pokedex_seen: Vec<SpeciesId>,
    pub pokedex_caught: Vec<SpeciesId>,
    pub last_location: Option<LocationId>,
    pub current_objective: Option<String>,
}

impl Trainer {
    pub fn new_player(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trainer_class: TrainerClass::Player,
            level: TrainerLevel(1),
            experience: 0,
            money: 3000, // 初始金钱
            position: Vec2::ZERO,
            current_location: None,
            appearance: TrainerAppearance::default_player(),
            personality: TrainerPersonality::default(),
            battle_stats: BattleStatistics::default(),
            badges: Vec::new(),
            achievements: Vec::new(),
            items: HashMap::new(),
            key_items: Vec::new(),
            settings: TrainerSettings::default(),
            save_data: TrainerSaveData::new(),
        }
    }

    pub fn new_npc(
        name: String,
        trainer_class: TrainerClass,
        level: u8,
        position: Vec2,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trainer_class,
            level: TrainerLevel(level),
            experience: Self::level_to_experience(level),
            money: 1000 + (level as u32 * 100),
            position,
            current_location: None,
            appearance: TrainerAppearance::default_npc(trainer_class),
            personality: TrainerPersonality::from_trainer_class(trainer_class),
            battle_stats: BattleStatistics::default(),
            badges: Vec::new(),
            achievements: Vec::new(),
            items: HashMap::new(),
            key_items: Vec::new(),
            settings: TrainerSettings::default(),
            save_data: TrainerSaveData::new(),
        }
    }

    /// 获取经验值对应的等级
    pub fn experience_to_level(experience: u64) -> u8 {
        // 简化的等级计算公式
        ((experience as f64).sqrt() / 10.0).floor().min(100.0) as u8 + 1
    }

    /// 获取等级对应的经验值
    pub fn level_to_experience(level: u8) -> u64 {
        let level = level.max(1) as f64;
        ((level - 1.0) * 10.0).powi(2) as u64
    }

    /// 获取到下一级所需的经验值
    pub fn experience_to_next_level(&self) -> u64 {
        let current_level_exp = Self::level_to_experience(self.level.0);
        let next_level_exp = Self::level_to_experience(self.level.0 + 1);
        next_level_exp - self.experience
    }

    /// 添加经验值
    pub fn add_experience(&mut self, amount: u64) -> Vec<LevelUpResult> {
        let old_level = self.level.0;
        self.experience += amount;
        
        let new_level = Self::experience_to_level(self.experience);
        let mut level_up_results = Vec::new();
        
        for level in (old_level + 1)..=new_level {
            self.level = TrainerLevel(level);
            level_up_results.push(LevelUpResult {
                old_level: level - 1,
                new_level: level,
                stat_increases: self.calculate_level_up_bonuses(level),
                new_abilities: self.get_level_abilities(level),
            });
            
            // 检查成就
            self.check_level_achievements();
        }
        
        level_up_results
    }

    fn calculate_level_up_bonuses(&self, _level: u8) -> HashMap<String, u32> {
        // 简化实现：每次升级获得一些奖励
        let mut bonuses = HashMap::new();
        bonuses.insert("max_party_size".to_string(), 1);
        bonuses.insert("item_slots".to_string(), 2);
        bonuses
    }

    fn get_level_abilities(&self, level: u8) -> Vec<String> {
        let mut abilities = Vec::new();
        
        match level {
            5 => abilities.push("使用Pokemon中心".to_string()),
            10 => abilities.push("参与道馆挑战".to_string()),
            15 => abilities.push("使用PC系统".to_string()),
            20 => abilities.push("参与联盟挑战".to_string()),
            _ => {},
        }
        
        abilities
    }

    fn check_level_achievements(&mut self) {
        // 检查等级相关成就
        for achievement in &mut self.achievements {
            if achievement.category == AchievementCategory::Battle && 
               !achievement.is_completed &&
               achievement.id == 1001 { // 假设1001是等级成就
                achievement.progress = self.level.0 as u32;
                if achievement.progress >= achievement.target {
                    achievement.is_completed = true;
                    achievement.completion_date = Some(chrono::Utc::now().to_rfc3339());
                }
            }
        }
    }

    /// 添加金钱
    pub fn add_money(&mut self, amount: u32) {
        self.money = self.money.saturating_add(amount);
        self.battle_stats.money_earned += amount as u64;
    }

    /// 花费金钱
    pub fn spend_money(&mut self, amount: u32) -> bool {
        if self.money >= amount {
            self.money -= amount;
            self.battle_stats.money_spent += amount as u64;
            true
        } else {
            false
        }
    }

    /// 添加物品
    pub fn add_item(&mut self, item_id: u32, quantity: u32) {
        *self.items.entry(item_id).or_insert(0) += quantity;
    }

    /// 使用物品
    pub fn use_item(&mut self, item_id: u32, quantity: u32) -> bool {
        if let Some(current) = self.items.get_mut(&item_id) {
            if *current >= quantity {
                *current -= quantity;
                if *current == 0 {
                    self.items.remove(&item_id);
                }
                
                // 更新统计
                *self.battle_stats.items_used.entry(item_id).or_insert(0) += quantity;
                return true;
            }
        }
        false
    }

    /// 检查是否有物品
    pub fn has_item(&self, item_id: u32, quantity: u32) -> bool {
        self.items.get(&item_id).map_or(false, |&count| count >= quantity)
    }

    /// 获取物品数量
    pub fn get_item_count(&self, item_id: u32) -> u32 {
        self.items.get(&item_id).copied().unwrap_or(0)
    }

    /// 添加徽章
    pub fn add_badge(&mut self, badge: Badge) {
        if !self.badges.iter().any(|b| b.id == badge.id) {
            self.badges.push(badge);
            self.check_badge_achievements();
        }
    }

    fn check_badge_achievements(&mut self) {
        // 检查徽章相关成就
        let badge_count = self.badges.len() as u32;
        
        for achievement in &mut self.achievements {
            if achievement.id == 2001 && !achievement.is_completed { // 假设2001是徽章收集成就
                achievement.progress = badge_count;
                if achievement.progress >= achievement.target {
                    achievement.is_completed = true;
                    achievement.completion_date = Some(chrono::Utc::now().to_rfc3339());
                }
            }
        }
    }

    /// 记录战斗结果
    pub fn record_battle_result(&mut self, result: BattleResult) {
        self.battle_stats.total_battles += 1;
        
        match result {
            BattleResult::Won => {
                self.battle_stats.wins += 1;
                self.add_experience(100); // 胜利奖励经验
            },
            BattleResult::Lost => {
                self.battle_stats.losses += 1;
                self.add_experience(25); // 失败也有少量经验
            },
            BattleResult::Draw => {
                self.battle_stats.draws += 1;
                self.add_experience(50); // 平局中等经验
            },
        }
        
        self.check_battle_achievements();
    }

    fn check_battle_achievements(&mut self) {
        for achievement in &mut self.achievements {
            match achievement.id {
                3001 => { // 战斗胜利数成就
                    achievement.progress = self.battle_stats.wins;
                },
                3002 => { // 总战斗数成就
                    achievement.progress = self.battle_stats.total_battles;
                },
                _ => continue,
            }
            
            if !achievement.is_completed && achievement.progress >= achievement.target {
                achievement.is_completed = true;
                achievement.completion_date = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }

    /// 记录Pokemon捕获
    pub fn record_pokemon_caught(&mut self, species_id: SpeciesId) {
        self.battle_stats.wild_pokemon_caught += 1;
        
        // 更新图鉴
        if !self.save_data.game_progress.pokedex_caught.contains(&species_id) {
            self.save_data.game_progress.pokedex_caught.push(species_id);
        }
        
        if !self.save_data.game_progress.pokedex_seen.contains(&species_id) {
            self.save_data.game_progress.pokedex_seen.push(species_id);
        }
        
        self.check_collection_achievements();
    }

    /// 记录Pokemon目击
    pub fn record_pokemon_seen(&mut self, species_id: SpeciesId) {
        if !self.save_data.game_progress.pokedex_seen.contains(&species_id) {
            self.save_data.game_progress.pokedex_seen.push(species_id);
            self.check_collection_achievements();
        }
    }

    fn check_collection_achievements(&mut self) {
        let caught_count = self.save_data.game_progress.pokedex_caught.len() as u32;
        let seen_count = self.save_data.game_progress.pokedex_seen.len() as u32;
        
        for achievement in &mut self.achievements {
            match achievement.id {
                4001 => achievement.progress = caught_count,  // 捕获数成就
                4002 => achievement.progress = seen_count,    // 目击数成就
                _ => continue,
            }
            
            if !achievement.is_completed && achievement.progress >= achievement.target {
                achievement.is_completed = true;
                achievement.completion_date = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }

    /// 更新位置
    pub fn update_position(&mut self, new_position: Vec2) {
        let distance = self.position.distance(new_position);
        self.position = new_position;
        self.battle_stats.steps_walked += distance as u64;
    }

    /// 更新游戏时间
    pub fn update_play_time(&mut self, delta_seconds: f32) {
        self.battle_stats.time_played_seconds += delta_seconds as u64;
        
        let total_seconds = self.battle_stats.time_played_seconds;
        self.save_data.play_time.hours = (total_seconds / 3600) as u32;
        self.save_data.play_time.minutes = ((total_seconds % 3600) / 60) as u8;
        self.save_data.play_time.seconds = (total_seconds % 60) as u8;
    }

    /// 获取胜率
    pub fn win_rate(&self) -> f32 {
        if self.battle_stats.total_battles > 0 {
            self.battle_stats.wins as f32 / self.battle_stats.total_battles as f32
        } else {
            0.0
        }
    }

    /// 获取已完成成就数量
    pub fn completed_achievements_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.is_completed).count()
    }

    /// 获取图鉴完成度
    pub fn pokedex_completion(&self, total_pokemon: usize) -> f32 {
        if total_pokemon > 0 {
            self.save_data.game_progress.pokedex_caught.len() as f32 / total_pokemon as f32
        } else {
            0.0
        }
    }

    /// 是否可以参与道馆挑战
    pub fn can_challenge_gyms(&self) -> bool {
        self.level.0 >= 10
    }

    /// 是否可以参与联盟挑战
    pub fn can_challenge_league(&self) -> bool {
        self.badges.iter().filter(|b| b.badge_type == BadgeType::Gym).count() >= 8
    }
}

#[derive(Debug, Clone)]
pub struct LevelUpResult {
    pub old_level: u8,
    pub new_level: u8,
    pub stat_increases: HashMap<String, u32>,
    pub new_abilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Won,
    Lost,
    Draw,
}

impl TrainerAppearance {
    pub fn default_player() -> Self {
        Self {
            sprite_id: 1,
            gender: Gender::Male,
            hair_color: Color::rgb(0.4, 0.2, 0.1),
            eye_color: Color::rgb(0.3, 0.6, 0.8),
            skin_tone: Color::rgb(0.9, 0.8, 0.7),
            outfit: OutfitSet {
                hat: None,
                top: 1,
                bottom: 1,
                shoes: 1,
                bag: Some(1),
            },
            accessories: Vec::new(),
        }
    }

    pub fn default_npc(trainer_class: TrainerClass) -> Self {
        let mut appearance = Self::default_player();
        
        match trainer_class {
            TrainerClass::GymLeader => {
                appearance.outfit.top = 10;
                appearance.outfit.bottom = 10;
            },
            TrainerClass::Youngster => {
                appearance.outfit.hat = Some(2);
                appearance.outfit.top = 2;
            },
            TrainerClass::Lass => {
                appearance.gender = Gender::Female;
                appearance.outfit.top = 3;
            },
            _ => {},
        }
        
        appearance
    }
}

impl TrainerPersonality {
    pub fn default() -> Self {
        Self {
            personality_type: PersonalityType::Balanced,
            favorite_type: None,
            battle_style: BattleStyle::Balanced,
            catchphrase: "Let's go!".to_string(),
            bio: "A Pokemon trainer".to_string(),
            traits: vec![PersonalityTrait::Friendly, PersonalityTrait::Curious],
        }
    }

    pub fn from_trainer_class(trainer_class: TrainerClass) -> Self {
        let mut personality = Self::default();
        
        match trainer_class {
            TrainerClass::GymLeader => {
                personality.personality_type = PersonalityType::Competitive;
                personality.battle_style = BattleStyle::TypeSpecialist;
                personality.catchphrase = "I'll show you my true power!".to_string();
                personality.traits = vec![PersonalityTrait::Competitive, PersonalityTrait::Loyal];
            },
            TrainerClass::BugCatcher => {
                personality.favorite_type = Some(crate::pokemon::types::PokemonType::Bug);
                personality.personality_type = PersonalityType::Collector;
                personality.catchphrase = "Bugs are the best!".to_string();
            },
            TrainerClass::Fisherman => {
                personality.favorite_type = Some(crate::pokemon::types::PokemonType::Water);
                personality.personality_type = PersonalityType::Patient;
                personality.traits = vec![PersonalityTrait::Patient, PersonalityTrait::Careful];
            },
            _ => {},
        }
        
        personality
    }
}

impl Default for TrainerSettings {
    fn default() -> Self {
        Self {
            battle_animations: true,
            battle_text_speed: TextSpeed::Normal,
            music_volume: 0.7,
            sfx_volume: 0.8,
            auto_save: true,
            nickname_pokemon: true,
            confirm_actions: true,
            battle_mode: BattleMode::Switch,
            ui_theme: "default".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

impl TrainerSaveData {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            save_version: 1,
            creation_date: now.clone(),
            last_save_date: now,
            play_time: PlayTime { hours: 0, minutes: 0, seconds: 0 },
            game_progress: GameProgress {
                story_flags: HashMap::new(),
                completed_quests: Vec::new(),
                unlocked_locations: Vec::new(),
                pokedex_seen: Vec::new(),
                pokedex_caught: Vec::new(),
                last_location: None,
                current_objective: None,
            },
            flags: HashMap::new(),
            variables: HashMap::new(),
        }
    }
}

/// 训练师管理器
#[derive(Debug, Component)]
pub struct TrainerManager {
    pub trainers: HashMap<TrainerId, Trainer>,
    pub player_id: Option<TrainerId>,
}

impl TrainerManager {
    pub fn new() -> Self {
        Self {
            trainers: HashMap::new(),
            player_id: None,
        }
    }

    pub fn create_player(&mut self, name: String) -> TrainerId {
        let trainer = Trainer::new_player(name);
        let id = trainer.id;
        
        self.trainers.insert(id, trainer);
        self.player_id = Some(id);
        
        id
    }

    pub fn add_trainer(&mut self, trainer: Trainer) -> TrainerId {
        let id = trainer.id;
        self.trainers.insert(id, trainer);
        id
    }

    pub fn get_trainer(&self, id: TrainerId) -> Option<&Trainer> {
        self.trainers.get(&id)
    }

    pub fn get_trainer_mut(&mut self, id: TrainerId) -> Option<&mut Trainer> {
        self.trainers.get_mut(&id)
    }

    pub fn get_player(&self) -> Option<&Trainer> {
        self.player_id.and_then(|id| self.get_trainer(id))
    }

    pub fn get_player_mut(&mut self) -> Option<&mut Trainer> {
        self.player_id.and_then(|id| self.get_trainer_mut(id))
    }

    pub fn remove_trainer(&mut self, id: TrainerId) -> bool {
        self.trainers.remove(&id).is_some()
    }

    pub fn get_trainers_by_class(&self, trainer_class: TrainerClass) -> Vec<&Trainer> {
        self.trainers
            .values()
            .filter(|t| t.trainer_class == trainer_class)
            .collect()
    }

    pub fn get_trainers_in_area(&self, center: Vec2, radius: f32) -> Vec<&Trainer> {
        self.trainers
            .values()
            .filter(|t| t.position.distance(center) <= radius)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trainer_creation() {
        let trainer = Trainer::new_player("Ash".to_string());
        
        assert_eq!(trainer.name, "Ash");
        assert_eq!(trainer.trainer_class, TrainerClass::Player);
        assert_eq!(trainer.level.0, 1);
        assert_eq!(trainer.money, 3000);
    }

    #[test]
    fn test_experience_system() {
        let mut trainer = Trainer::new_player("Test".to_string());
        
        assert_eq!(trainer.level.0, 1);
        
        let level_ups = trainer.add_experience(1000);
        assert!(trainer.level.0 > 1);
        assert!(!level_ups.is_empty());
    }

    #[test]
    fn test_money_system() {
        let mut trainer = Trainer::new_player("Test".to_string());
        let initial_money = trainer.money;
        
        trainer.add_money(500);
        assert_eq!(trainer.money, initial_money + 500);
        
        assert!(trainer.spend_money(100));
        assert_eq!(trainer.money, initial_money + 400);
        
        assert!(!trainer.spend_money(10000)); // 金钱不足
    }

    #[test]
    fn test_item_system() {
        let mut trainer = Trainer::new_player("Test".to_string());
        
        trainer.add_item(1, 5); // 添加5个物品1
        assert_eq!(trainer.get_item_count(1), 5);
        assert!(trainer.has_item(1, 3));
        
        assert!(trainer.use_item(1, 2)); // 使用2个
        assert_eq!(trainer.get_item_count(1), 3);
        
        assert!(!trainer.use_item(1, 10)); // 使用超过拥有数量
    }

    #[test]
    fn test_battle_stats() {
        let mut trainer = Trainer::new_player("Test".to_string());
        
        trainer.record_battle_result(BattleResult::Won);
        assert_eq!(trainer.battle_stats.total_battles, 1);
        assert_eq!(trainer.battle_stats.wins, 1);
        assert_eq!(trainer.win_rate(), 1.0);
        
        trainer.record_battle_result(BattleResult::Lost);
        assert_eq!(trainer.battle_stats.total_battles, 2);
        assert_eq!(trainer.win_rate(), 0.5);
    }

    #[test]
    fn test_trainer_manager() {
        let mut manager = TrainerManager::new();
        
        let player_id = manager.create_player("Player1".to_string());
        assert_eq!(manager.player_id, Some(player_id));
        
        let player = manager.get_player().unwrap();
        assert_eq!(player.name, "Player1");
        
        let npc = Trainer::new_npc(
            "Brock".to_string(),
            TrainerClass::GymLeader,
            15,
            Vec2::new(100.0, 200.0),
        );
        let npc_id = manager.add_trainer(npc);
        
        let gym_leaders = manager.get_trainers_by_class(TrainerClass::GymLeader);
        assert_eq!(gym_leaders.len(), 1);
    }
}