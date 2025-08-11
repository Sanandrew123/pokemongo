// Player Progression System - 玩家进度系统
//
// 开发心理过程：
// 1. 这是管理玩家整体游戏进度的核心系统，包括剧情推进、成就系统、图鉴等
// 2. 需要追踪各种游戏事件和里程碑，为玩家提供成就感和目标感
// 3. 实现灵活的任务系统，支持主线任务、支线任务和日常任务
// 4. 图鉴系统需要记录Pokemon的发现状态和详细信息
// 5. 统计系统帮助玩家了解游戏进展和个人表现

use std::collections::{HashMap, HashSet, BTreeMap};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use crate::pokemon::{PokemonId, Type, Individual};
use crate::world::{LocationId, WeatherType};
use crate::battle::{BattleOutcome};
use crate::data::DatabaseError;

pub type QuestId = u32;
pub type AchievementId = u32;
pub type BadgeId = u32;

/// 游戏进度状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    pub player_id: Uuid,
    pub game_version: String,
    pub start_date: DateTime<Utc>,
    pub last_save: DateTime<Utc>,
    pub total_playtime: Duration,
    
    // 主要进度
    pub story_progress: StoryProgress,
    pub pokedex: Pokedex,
    pub achievements: AchievementTracker,
    pub quest_log: QuestLog,
    pub badges: BadgeCollection,
    
    // 统计信息
    pub statistics: PlayerStatistics,
    pub milestones: Vec<Milestone>,
    pub preferences: PlayerPreferences,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerProgression {
    pub fn new() -> Self {
        Self {
            player_id: Uuid::new_v4(),
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            start_date: Utc::now(),
            last_save: Utc::now(),
            total_playtime: Duration::zero(),
            
            story_progress: StoryProgress::new(),
            pokedex: Pokedex::new(),
            achievements: AchievementTracker::new(),
            quest_log: QuestLog::new(),
            badges: BadgeCollection::new(),
            
            statistics: PlayerStatistics::new(),
            milestones: Vec::new(),
            preferences: PlayerPreferences::default(),
        }
    }

    pub fn update_playtime(&mut self, session_duration: Duration) {
        self.total_playtime = self.total_playtime + session_duration;
        self.last_save = Utc::now();
    }

    pub fn check_milestones(&mut self) -> Vec<Milestone> {
        let mut new_milestones = Vec::new();

        // 检查游戏时间里程碑
        let hours_played = self.total_playtime.num_hours();
        for &hour_milestone in &[1, 10, 50, 100, 500, 1000] {
            if hours_played >= hour_milestone && 
               !self.milestones.iter().any(|m| matches!(m.milestone_type, MilestoneType::PlaytimeHours(h) if h == hour_milestone)) {
                let milestone = Milestone {
                    id: Uuid::new_v4(),
                    milestone_type: MilestoneType::PlaytimeHours(hour_milestone),
                    achieved_date: Utc::now(),
                    description: format!("Played for {} hours!", hour_milestone),
                    reward: Some(MilestoneReward::Money(hour_milestone * 100)),
                };
                new_milestones.push(milestone.clone());
                self.milestones.push(milestone);
            }
        }

        // 检查图鉴里程碑
        let pokedex_completion = self.pokedex.get_completion_percentage();
        for &completion in &[10.0, 25.0, 50.0, 75.0, 90.0, 100.0] {
            if pokedex_completion >= completion &&
               !self.milestones.iter().any(|m| matches!(m.milestone_type, MilestoneType::PokedexCompletion(p) if (p - completion).abs() < 0.1)) {
                let milestone = Milestone {
                    id: Uuid::new_v4(),
                    milestone_type: MilestoneType::PokedexCompletion(completion),
                    achieved_date: Utc::now(),
                    description: format!("Pokedex {}% complete!", completion),
                    reward: Some(MilestoneReward::Item("Rare Candy".to_string(), 1)),
                };
                new_milestones.push(milestone.clone());
                self.milestones.push(milestone);
            }
        }

        new_milestones
    }

    pub fn get_completion_summary(&self) -> ProgressionSummary {
        ProgressionSummary {
            story_completion: self.story_progress.get_completion_percentage(),
            pokedex_completion: self.pokedex.get_completion_percentage(),
            achievement_completion: self.achievements.get_completion_percentage(),
            badges_earned: self.badges.get_earned_count(),
            total_pokemon_caught: self.statistics.pokemon_caught,
            total_battles_won: self.statistics.battles_won,
            playtime_hours: self.total_playtime.num_hours() as u32,
            active_quests: self.quest_log.get_active_quest_count(),
        }
    }
}

/// 剧情进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryProgress {
    pub current_chapter: u32,
    pub completed_chapters: HashSet<u32>,
    pub story_flags: HashMap<String, bool>,
    pub cutscene_viewed: HashSet<String>,
    pub dialogue_history: Vec<DialogueEntry>,
    pub region_access: HashMap<String, bool>,
}

impl StoryProgress {
    pub fn new() -> Self {
        Self {
            current_chapter: 1,
            completed_chapters: HashSet::new(),
            story_flags: HashMap::new(),
            cutscene_viewed: HashSet::new(),
            dialogue_history: Vec::new(),
            region_access: HashMap::new(),
        }
    }

    pub fn advance_chapter(&mut self, chapter: u32) -> bool {
        if chapter > self.current_chapter {
            self.completed_chapters.insert(self.current_chapter);
            self.current_chapter = chapter;
            true
        } else {
            false
        }
    }

    pub fn set_story_flag(&mut self, flag: String, value: bool) {
        self.story_flags.insert(flag, value);
    }

    pub fn get_story_flag(&self, flag: &str) -> bool {
        self.story_flags.get(flag).copied().unwrap_or(false)
    }

    pub fn mark_cutscene_viewed(&mut self, cutscene_id: String) {
        self.cutscene_viewed.insert(cutscene_id);
    }

    pub fn has_viewed_cutscene(&self, cutscene_id: &str) -> bool {
        self.cutscene_viewed.contains(cutscene_id)
    }

    pub fn add_dialogue(&mut self, npc_name: String, dialogue: String) {
        self.dialogue_history.push(DialogueEntry {
            npc_name,
            dialogue,
            timestamp: Utc::now(),
            location: None,
        });

        // 保留最近1000条对话记录
        if self.dialogue_history.len() > 1000 {
            self.dialogue_history.remove(0);
        }
    }

    pub fn unlock_region(&mut self, region_name: String) {
        self.region_access.insert(region_name, true);
    }

    pub fn is_region_unlocked(&self, region_name: &str) -> bool {
        self.region_access.get(region_name).copied().unwrap_or(false)
    }

    pub fn get_completion_percentage(&self) -> f32 {
        // 假设总共有20章
        const TOTAL_CHAPTERS: f32 = 20.0;
        (self.completed_chapters.len() as f32 / TOTAL_CHAPTERS) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEntry {
    pub npc_name: String,
    pub dialogue: String,
    pub timestamp: DateTime<Utc>,
    pub location: Option<LocationId>,
}

/// Pokemon图鉴系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokedex {
    pub entries: HashMap<PokemonId, PokedexEntry>,
    pub total_species: usize,
    pub regional_dexes: HashMap<String, RegionalDex>,
    pub research_tasks: HashMap<PokemonId, Vec<ResearchTask>>,
}

impl Pokedex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            total_species: 1010, // 假设总共有1010种Pokemon
            regional_dexes: HashMap::new(),
            research_tasks: HashMap::new(),
        }
    }

    pub fn register_sighting(&mut self, pokemon_id: PokemonId, location: LocationId) {
        let entry = self.entries.entry(pokemon_id).or_insert_with(|| PokedexEntry::new(pokemon_id));
        
        if entry.status == PokedexStatus::Unknown {
            entry.status = PokedexStatus::Seen;
            entry.first_seen_date = Some(Utc::now());
            entry.first_seen_location = Some(location);
        }
        
        entry.sightings.push(PokemonSighting {
            location,
            timestamp: Utc::now(),
            weather: None,
        });
    }

    pub fn register_capture(&mut self, pokemon: &Individual, location: LocationId, method: CaptureMethod) {
        let entry = self.entries.entry(pokemon.species.id).or_insert_with(|| PokedexEntry::new(pokemon.species.id));
        
        entry.status = PokedexStatus::Caught;
        entry.times_caught += 1;
        
        if entry.first_caught_date.is_none() {
            entry.first_caught_date = Some(Utc::now());
            entry.first_caught_location = Some(location);
            entry.first_caught_method = Some(method);
        }

        // 更新统计信息
        entry.min_level = entry.min_level.map(|min| min.min(pokemon.level)).or(Some(pokemon.level));
        entry.max_level = entry.max_level.map(|max| max.max(pokemon.level)).or(Some(pokemon.level));

        // 记录发现的形态
        if !entry.forms_seen.contains(&pokemon.form_id.unwrap_or(0)) {
            entry.forms_seen.insert(pokemon.form_id.unwrap_or(0));
        }

        // 记录能力
        for ability in &pokemon.abilities {
            entry.abilities_seen.insert(ability.clone());
        }
    }

    pub fn get_entry(&self, pokemon_id: PokemonId) -> Option<&PokedexEntry> {
        self.entries.get(&pokemon_id)
    }

    pub fn get_seen_count(&self) -> usize {
        self.entries.values()
            .filter(|entry| entry.status != PokedexStatus::Unknown)
            .count()
    }

    pub fn get_caught_count(&self) -> usize {
        self.entries.values()
            .filter(|entry| entry.status == PokedexStatus::Caught)
            .count()
    }

    pub fn get_completion_percentage(&self) -> f32 {
        (self.get_caught_count() as f32 / self.total_species as f32) * 100.0
    }

    pub fn get_type_statistics(&self) -> HashMap<Type, TypeStats> {
        let mut stats = HashMap::new();

        for entry in self.entries.values() {
            if entry.status == PokedexStatus::Caught {
                for pokemon_type in &entry.types {
                    let type_stats = stats.entry(*pokemon_type).or_insert_with(TypeStats::default);
                    type_stats.caught_count += 1;
                    type_stats.total_caught += entry.times_caught;
                }
            }
        }

        stats
    }

    pub fn create_regional_dex(&mut self, region_name: String, pokemon_ids: Vec<PokemonId>) {
        let regional_dex = RegionalDex {
            name: region_name.clone(),
            pokemon_list: pokemon_ids.into_iter().collect(),
            completion_percentage: 0.0,
        };
        self.regional_dexes.insert(region_name, regional_dex);
        self.update_regional_dex_completion();
    }

    pub fn update_regional_dex_completion(&mut self) {
        for regional_dex in self.regional_dexes.values_mut() {
            let total = regional_dex.pokemon_list.len();
            let caught = regional_dex.pokemon_list.iter()
                .filter(|&&id| self.entries.get(&id)
                    .map(|entry| entry.status == PokedexStatus::Caught)
                    .unwrap_or(false))
                .count();
            
            regional_dex.completion_percentage = if total > 0 {
                (caught as f32 / total as f32) * 100.0
            } else {
                0.0
            };
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokedexEntry {
    pub pokemon_id: PokemonId,
    pub status: PokedexStatus,
    pub types: Vec<Type>,
    pub first_seen_date: Option<DateTime<Utc>>,
    pub first_seen_location: Option<LocationId>,
    pub first_caught_date: Option<DateTime<Utc>>,
    pub first_caught_location: Option<LocationId>,
    pub first_caught_method: Option<CaptureMethod>,
    pub times_caught: u32,
    pub times_evolved: u32,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    pub forms_seen: HashSet<u32>,
    pub abilities_seen: HashSet<String>,
    pub sightings: Vec<PokemonSighting>,
    pub notes: String,
}

impl PokedexEntry {
    pub fn new(pokemon_id: PokemonId) -> Self {
        Self {
            pokemon_id,
            status: PokedexStatus::Unknown,
            types: Vec::new(),
            first_seen_date: None,
            first_seen_location: None,
            first_caught_date: None,
            first_caught_location: None,
            first_caught_method: None,
            times_caught: 0,
            times_evolved: 0,
            min_level: None,
            max_level: None,
            forms_seen: HashSet::new(),
            abilities_seen: HashSet::new(),
            sightings: Vec::new(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokedexStatus {
    Unknown,
    Seen,
    Caught,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMethod {
    PokeBall,
    Fishing,
    Surfing,
    HeadbuttTree,
    RockSmash,
    Gift,
    Trade,
    Evolution,
    Egg,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSighting {
    pub location: LocationId,
    pub timestamp: DateTime<Utc>,
    pub weather: Option<WeatherType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalDex {
    pub name: String,
    pub pokemon_list: HashSet<PokemonId>,
    pub completion_percentage: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeStats {
    pub caught_count: u32,
    pub total_caught: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    pub task_id: Uuid,
    pub description: String,
    pub progress: u32,
    pub required: u32,
    pub reward: TaskReward,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskReward {
    Experience(u32),
    Item(String, u32),
    Money(u32),
    PokedexData,
}

/// 成就系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementTracker {
    pub achievements: HashMap<AchievementId, Achievement>,
    pub earned_achievements: HashSet<AchievementId>,
    pub progress_data: HashMap<AchievementId, u32>,
    pub categories: HashMap<String, Vec<AchievementId>>,
}

impl AchievementTracker {
    pub fn new() -> Self {
        let mut tracker = Self {
            achievements: HashMap::new(),
            earned_achievements: HashSet::new(),
            progress_data: HashMap::new(),
            categories: HashMap::new(),
        };
        
        tracker.initialize_achievements();
        tracker
    }

    fn initialize_achievements(&mut self) {
        // 战斗相关成就
        self.add_achievement(Achievement {
            id: 1,
            name: "First Victory".to_string(),
            description: "Win your first Pokemon battle".to_string(),
            category: "Battle".to_string(),
            requirement: AchievementRequirement::BattlesWon(1),
            reward: AchievementReward::Money(500),
            icon: "trophy_bronze".to_string(),
            rarity: AchievementRarity::Common,
        });

        self.add_achievement(Achievement {
            id: 2,
            name: "Champion".to_string(),
            description: "Win 100 Pokemon battles".to_string(),
            category: "Battle".to_string(),
            requirement: AchievementRequirement::BattlesWon(100),
            reward: AchievementReward::Item("Master Ball".to_string(), 1),
            icon: "trophy_gold".to_string(),
            rarity: AchievementRarity::Legendary,
        });

        // 收集相关成就
        self.add_achievement(Achievement {
            id: 3,
            name: "Collector".to_string(),
            description: "Catch 50 different Pokemon species".to_string(),
            category: "Collection".to_string(),
            requirement: AchievementRequirement::PokemonCaught(50),
            reward: AchievementReward::Item("Ultra Ball".to_string(), 10),
            icon: "pokeball_master".to_string(),
            rarity: AchievementRarity::Rare,
        });

        // 探索相关成就
        self.add_achievement(Achievement {
            id: 4,
            name: "Explorer".to_string(),
            description: "Visit 20 different locations".to_string(),
            category: "Exploration".to_string(),
            requirement: AchievementRequirement::LocationsVisited(20),
            reward: AchievementReward::Money(2000),
            icon: "map".to_string(),
            rarity: AchievementRarity::Uncommon,
        });
    }

    fn add_achievement(&mut self, achievement: Achievement) {
        let category_achievements = self.categories.entry(achievement.category.clone()).or_insert_with(Vec::new);
        category_achievements.push(achievement.id);
        self.achievements.insert(achievement.id, achievement);
    }

    pub fn update_progress(&mut self, event: GameEvent) -> Vec<AchievementId> {
        let mut newly_earned = Vec::new();

        for (&achievement_id, achievement) in &self.achievements {
            if self.earned_achievements.contains(&achievement_id) {
                continue;
            }

            let current_progress = self.progress_data.get(&achievement_id).copied().unwrap_or(0);
            let new_progress = achievement.requirement.update_progress(current_progress, &event);

            if new_progress != current_progress {
                self.progress_data.insert(achievement_id, new_progress);

                if achievement.requirement.is_completed(new_progress) {
                    self.earned_achievements.insert(achievement_id);
                    newly_earned.push(achievement_id);
                }
            }
        }

        newly_earned
    }

    pub fn get_achievement(&self, achievement_id: AchievementId) -> Option<&Achievement> {
        self.achievements.get(&achievement_id)
    }

    pub fn is_earned(&self, achievement_id: AchievementId) -> bool {
        self.earned_achievements.contains(&achievement_id)
    }

    pub fn get_progress(&self, achievement_id: AchievementId) -> u32 {
        self.progress_data.get(&achievement_id).copied().unwrap_or(0)
    }

    pub fn get_completion_percentage(&self) -> f32 {
        if self.achievements.is_empty() {
            return 0.0;
        }
        (self.earned_achievements.len() as f32 / self.achievements.len() as f32) * 100.0
    }

    pub fn get_achievements_by_category(&self, category: &str) -> Vec<&Achievement> {
        if let Some(achievement_ids) = self.categories.get(category) {
            achievement_ids.iter()
                .filter_map(|&id| self.achievements.get(&id))
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: AchievementId,
    pub name: String,
    pub description: String,
    pub category: String,
    pub requirement: AchievementRequirement,
    pub reward: AchievementReward,
    pub icon: String,
    pub rarity: AchievementRarity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRequirement {
    BattlesWon(u32),
    PokemonCaught(u32),
    LocationsVisited(u32),
    PlaytimeHours(u32),
    LevelReached(u32),
    ItemsUsed(u32),
    MoneyEarned(u32),
    EvolutionsPerformed(u32),
    TypesCaught(Vec<Type>),
    Custom(String, u32), // 自定义成就条件
}

impl AchievementRequirement {
    pub fn update_progress(&self, current: u32, event: &GameEvent) -> u32 {
        match (self, event) {
            (AchievementRequirement::BattlesWon(_), GameEvent::BattleWon) => current + 1,
            (AchievementRequirement::PokemonCaught(_), GameEvent::PokemonCaught(_)) => current + 1,
            (AchievementRequirement::LocationsVisited(_), GameEvent::LocationVisited(_)) => current + 1,
            (AchievementRequirement::PlaytimeHours(_), GameEvent::PlaytimeUpdate(hours)) => *hours,
            (AchievementRequirement::LevelReached(_), GameEvent::LevelUp(level)) => (*level as u32).max(current),
            (AchievementRequirement::ItemsUsed(_), GameEvent::ItemUsed(_)) => current + 1,
            (AchievementRequirement::MoneyEarned(_), GameEvent::MoneyGained(amount)) => current + amount,
            (AchievementRequirement::EvolutionsPerformed(_), GameEvent::PokemonEvolved(_)) => current + 1,
            _ => current,
        }
    }

    pub fn is_completed(&self, progress: u32) -> bool {
        match self {
            AchievementRequirement::BattlesWon(required) => progress >= *required,
            AchievementRequirement::PokemonCaught(required) => progress >= *required,
            AchievementRequirement::LocationsVisited(required) => progress >= *required,
            AchievementRequirement::PlaytimeHours(required) => progress >= *required,
            AchievementRequirement::LevelReached(required) => progress >= *required,
            AchievementRequirement::ItemsUsed(required) => progress >= *required,
            AchievementRequirement::MoneyEarned(required) => progress >= *required,
            AchievementRequirement::EvolutionsPerformed(required) => progress >= *required,
            AchievementRequirement::TypesCaught(_) => false, // 需要特殊处理
            AchievementRequirement::Custom(_, required) => progress >= *required,
        }
    }

    pub fn get_target(&self) -> u32 {
        match self {
            AchievementRequirement::BattlesWon(target) |
            AchievementRequirement::PokemonCaught(target) |
            AchievementRequirement::LocationsVisited(target) |
            AchievementRequirement::PlaytimeHours(target) |
            AchievementRequirement::LevelReached(target) |
            AchievementRequirement::ItemsUsed(target) |
            AchievementRequirement::MoneyEarned(target) |
            AchievementRequirement::EvolutionsPerformed(target) |
            AchievementRequirement::Custom(_, target) => *target,
            AchievementRequirement::TypesCaught(types) => types.len() as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    Money(u32),
    Item(String, u32),
    Title(String),
    Cosmetic(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// 游戏事件用于触发成就进度更新
#[derive(Debug, Clone)]
pub enum GameEvent {
    BattleWon,
    BattleLost,
    PokemonCaught(PokemonId),
    PokemonEvolved(PokemonId),
    LocationVisited(LocationId),
    LevelUp(u8),
    ItemUsed(String),
    MoneyGained(u32),
    QuestCompleted(QuestId),
    PlaytimeUpdate(u32),
}

/// 任务系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestLog {
    pub quests: HashMap<QuestId, Quest>,
    pub active_quests: HashSet<QuestId>,
    pub completed_quests: HashSet<QuestId>,
    pub available_quests: HashSet<QuestId>,
    pub next_quest_id: QuestId,
    pub daily_quest_reset: DateTime<Utc>,
}

impl QuestLog {
    pub fn new() -> Self {
        let mut quest_log = Self {
            quests: HashMap::new(),
            active_quests: HashSet::new(),
            completed_quests: HashSet::new(),
            available_quests: HashSet::new(),
            next_quest_id: 1,
            daily_quest_reset: Utc::now() + Duration::days(1),
        };
        
        quest_log.initialize_quests();
        quest_log
    }

    fn initialize_quests(&mut self) {
        // 主线任务
        self.add_quest(Quest {
            id: 1,
            title: "First Steps".to_string(),
            description: "Catch your first Pokemon".to_string(),
            quest_type: QuestType::Story,
            objectives: vec![
                QuestObjective {
                    id: 1,
                    description: "Catch a Pokemon".to_string(),
                    objective_type: ObjectiveType::CatchPokemon { species_id: None, count: 1 },
                    progress: 0,
                    completed: false,
                }
            ],
            rewards: vec![
                QuestReward::Money(1000),
                QuestReward::Item("Poke Ball".to_string(), 5),
            ],
            prerequisites: Vec::new(),
            time_limit: None,
            repeatable: false,
            is_active: false,
        });

        // 每日任务
        self.add_quest(Quest {
            id: 2,
            title: "Daily Training".to_string(),
            description: "Win 3 Pokemon battles today".to_string(),
            quest_type: QuestType::Daily,
            objectives: vec![
                QuestObjective {
                    id: 1,
                    description: "Win battles".to_string(),
                    objective_type: ObjectiveType::WinBattles { count: 3 },
                    progress: 0,
                    completed: false,
                }
            ],
            rewards: vec![
                QuestReward::Money(500),
                QuestReward::Experience(200),
            ],
            prerequisites: Vec::new(),
            time_limit: Some(Duration::days(1)),
            repeatable: true,
            is_active: false,
        });
    }

    fn add_quest(&mut self, quest: Quest) {
        self.available_quests.insert(quest.id);
        self.quests.insert(quest.id, quest);
        self.next_quest_id = self.next_quest_id.max(quest.id + 1);
    }

    pub fn start_quest(&mut self, quest_id: QuestId) -> Result<(), QuestError> {
        if !self.available_quests.contains(&quest_id) {
            return Err(QuestError::QuestNotAvailable);
        }

        if self.active_quests.contains(&quest_id) {
            return Err(QuestError::QuestAlreadyActive);
        }

        // 检查前置条件
        if let Some(quest) = self.quests.get(&quest_id) {
            for prerequisite in &quest.prerequisites {
                if !self.completed_quests.contains(prerequisite) {
                    return Err(QuestError::PrerequisiteNotMet);
                }
            }
        }

        self.active_quests.insert(quest_id);
        self.available_quests.remove(&quest_id);

        if let Some(quest) = self.quests.get_mut(&quest_id) {
            quest.is_active = true;
        }

        Ok(())
    }

    pub fn update_quest_progress(&mut self, event: &GameEvent) -> Vec<QuestId> {
        let mut completed_quests = Vec::new();

        for &quest_id in &self.active_quests.clone() {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                let mut all_objectives_completed = true;

                for objective in &mut quest.objectives {
                    if !objective.completed {
                        let old_progress = objective.progress;
                        objective.update_progress(event);

                        if objective.progress != old_progress && objective.is_completed() {
                            objective.completed = true;
                        }

                        if !objective.completed {
                            all_objectives_completed = false;
                        }
                    }
                }

                if all_objectives_completed {
                    self.complete_quest(quest_id);
                    completed_quests.push(quest_id);
                }
            }
        }

        completed_quests
    }

    fn complete_quest(&mut self, quest_id: QuestId) {
        self.active_quests.remove(&quest_id);
        self.completed_quests.insert(quest_id);

        if let Some(quest) = self.quests.get_mut(&quest_id) {
            quest.is_active = false;

            // 如果是可重复任务，重新加入可用任务
            if quest.repeatable {
                quest.reset_objectives();
                self.available_quests.insert(quest_id);
            }
        }
    }

    pub fn get_active_quest_count(&self) -> usize {
        self.active_quests.len()
    }

    pub fn get_quest(&self, quest_id: QuestId) -> Option<&Quest> {
        self.quests.get(&quest_id)
    }

    pub fn get_active_quests(&self) -> Vec<&Quest> {
        self.active_quests.iter()
            .filter_map(|&id| self.quests.get(&id))
            .collect()
    }

    pub fn check_daily_reset(&mut self) {
        if Utc::now() >= self.daily_quest_reset {
            self.reset_daily_quests();
            self.daily_quest_reset = Utc::now() + Duration::days(1);
        }
    }

    fn reset_daily_quests(&mut self) {
        for quest in self.quests.values_mut() {
            if quest.quest_type == QuestType::Daily {
                quest.reset_objectives();
                self.available_quests.insert(quest.id);
                self.active_quests.remove(&quest.id);
                self.completed_quests.remove(&quest.id);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: QuestId,
    pub title: String,
    pub description: String,
    pub quest_type: QuestType,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub prerequisites: Vec<QuestId>,
    pub time_limit: Option<Duration>,
    pub repeatable: bool,
    pub is_active: bool,
}

impl Quest {
    pub fn is_completed(&self) -> bool {
        self.objectives.iter().all(|obj| obj.completed)
    }

    pub fn get_completion_percentage(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }

        let completed_count = self.objectives.iter().filter(|obj| obj.completed).count();
        (completed_count as f32 / self.objectives.len() as f32) * 100.0
    }

    pub fn reset_objectives(&mut self) {
        for objective in &mut self.objectives {
            objective.progress = 0;
            objective.completed = false;
        }
        self.is_active = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestType {
    Story,
    Side,
    Daily,
    Weekly,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub id: u32,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub progress: u32,
    pub completed: bool,
}

impl QuestObjective {
    pub fn update_progress(&mut self, event: &GameEvent) {
        match (&self.objective_type, event) {
            (ObjectiveType::CatchPokemon { species_id: None, count: _ }, GameEvent::PokemonCaught(_)) => {
                self.progress += 1;
            }
            (ObjectiveType::CatchPokemon { species_id: Some(target_id), count: _ }, GameEvent::PokemonCaught(caught_id)) => {
                if target_id == caught_id {
                    self.progress += 1;
                }
            }
            (ObjectiveType::WinBattles { count: _ }, GameEvent::BattleWon) => {
                self.progress += 1;
            }
            (ObjectiveType::VisitLocation { location_id }, GameEvent::LocationVisited(visited_id)) => {
                if location_id == visited_id {
                    self.progress = 1;
                }
            }
            (ObjectiveType::ReachLevel { level }, GameEvent::LevelUp(current_level)) => {
                if *current_level >= *level {
                    self.progress = 1;
                }
            }
            (ObjectiveType::UseItem { item_name }, GameEvent::ItemUsed(used_item)) => {
                if item_name == used_item {
                    self.progress += 1;
                }
            }
            _ => {}
        }
    }

    pub fn is_completed(&self) -> bool {
        match &self.objective_type {
            ObjectiveType::CatchPokemon { count, .. } => self.progress >= *count,
            ObjectiveType::WinBattles { count } => self.progress >= *count,
            ObjectiveType::VisitLocation { .. } => self.progress >= 1,
            ObjectiveType::ReachLevel { .. } => self.progress >= 1,
            ObjectiveType::UseItem { .. } => self.progress >= 1,
            ObjectiveType::Custom { target } => self.progress >= *target,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    CatchPokemon { species_id: Option<PokemonId>, count: u32 },
    WinBattles { count: u32 },
    VisitLocation { location_id: LocationId },
    ReachLevel { level: u8 },
    UseItem { item_name: String },
    Custom { description: String, target: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestReward {
    Money(u32),
    Experience(u32),
    Item(String, u32),
    Pokemon(PokemonId),
    Badge(BadgeId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestError {
    QuestNotFound,
    QuestNotAvailable,
    QuestAlreadyActive,
    PrerequisiteNotMet,
    QuestExpired,
}

/// 徽章收集系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeCollection {
    pub badges: HashMap<BadgeId, Badge>,
    pub earned_badges: HashSet<BadgeId>,
    pub gym_progress: HashMap<String, GymProgress>,
}

impl BadgeCollection {
    pub fn new() -> Self {
        let mut collection = Self {
            badges: HashMap::new(),
            earned_badges: HashSet::new(),
            gym_progress: HashMap::new(),
        };
        
        collection.initialize_badges();
        collection
    }

    fn initialize_badges(&mut self) {
        // 道馆徽章
        let gym_badges = [
            ("Boulder Badge", "Pewter City Gym", Type::Rock),
            ("Cascade Badge", "Cerulean City Gym", Type::Water),
            ("Thunder Badge", "Vermilion City Gym", Type::Electric),
            ("Rainbow Badge", "Celadon City Gym", Type::Grass),
            ("Soul Badge", "Fuchsia City Gym", Type::Poison),
            ("Marsh Badge", "Saffron City Gym", Type::Psychic),
            ("Volcano Badge", "Cinnabar Island Gym", Type::Fire),
            ("Earth Badge", "Viridian City Gym", Type::Ground),
        ];

        for (i, (name, gym, badge_type)) in gym_badges.iter().enumerate() {
            self.badges.insert(i as u32 + 1, Badge {
                id: i as u32 + 1,
                name: name.to_string(),
                description: format!("Defeat the {} leader", gym),
                badge_type: BadgeType::Gym(*badge_type),
                icon: format!("badge_{}", name.to_lowercase().replace(' ', "_")),
                earned_date: None,
            });
        }

        // 特殊徽章
        self.badges.insert(100, Badge {
            id: 100,
            name: "Champion Badge".to_string(),
            description: "Become the Pokemon League Champion".to_string(),
            badge_type: BadgeType::Special,
            icon: "badge_champion".to_string(),
            earned_date: None,
        });
    }

    pub fn earn_badge(&mut self, badge_id: BadgeId) -> bool {
        if self.earned_badges.contains(&badge_id) {
            return false;
        }

        self.earned_badges.insert(badge_id);
        if let Some(badge) = self.badges.get_mut(&badge_id) {
            badge.earned_date = Some(Utc::now());
        }
        
        true
    }

    pub fn get_earned_count(&self) -> usize {
        self.earned_badges.len()
    }

    pub fn get_badge(&self, badge_id: BadgeId) -> Option<&Badge> {
        self.badges.get(&badge_id)
    }

    pub fn is_earned(&self, badge_id: BadgeId) -> bool {
        self.earned_badges.contains(&badge_id)
    }

    pub fn get_gym_badges(&self) -> Vec<&Badge> {
        self.badges.values()
            .filter(|badge| matches!(badge.badge_type, BadgeType::Gym(_)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: BadgeId,
    pub name: String,
    pub description: String,
    pub badge_type: BadgeType,
    pub icon: String,
    pub earned_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeType {
    Gym(Type),
    Elite4,
    Champion,
    Contest,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GymProgress {
    pub gym_name: String,
    pub leader_defeated: bool,
    pub attempts: u32,
    pub best_time: Option<Duration>,
}

/// 玩家统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    // 基础统计
    pub pokemon_caught: u32,
    pub pokemon_seen: u32,
    pub battles_won: u32,
    pub battles_lost: u32,
    pub steps_taken: u32,
    pub money_earned: u32,
    pub money_spent: u32,
    
    // 高级统计
    pub shiny_pokemon_found: u32,
    pub legendary_pokemon_caught: u32,
    pub pokemon_evolved: u32,
    pub eggs_hatched: u32,
    pub trades_made: u32,
    pub items_found: u32,
    pub berries_picked: u32,
    
    // 时间统计
    pub first_play_date: DateTime<Utc>,
    pub last_play_date: DateTime<Utc>,
    pub longest_session: Duration,
    pub total_sessions: u32,
    
    // 特殊记录
    pub highest_level_pokemon: u8,
    pub fastest_battle_win: Option<Duration>,
    pub locations_discovered: u32,
    pub gym_leaders_defeated: u32,
}

impl PlayerStatistics {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            pokemon_caught: 0,
            pokemon_seen: 0,
            battles_won: 0,
            battles_lost: 0,
            steps_taken: 0,
            money_earned: 0,
            money_spent: 0,
            shiny_pokemon_found: 0,
            legendary_pokemon_caught: 0,
            pokemon_evolved: 0,
            eggs_hatched: 0,
            trades_made: 0,
            items_found: 0,
            berries_picked: 0,
            first_play_date: now,
            last_play_date: now,
            longest_session: Duration::zero(),
            total_sessions: 0,
            highest_level_pokemon: 1,
            fastest_battle_win: None,
            locations_discovered: 0,
            gym_leaders_defeated: 0,
        }
    }

    pub fn update_battle_outcome(&mut self, outcome: BattleOutcome, duration: Duration) {
        match outcome {
            BattleOutcome::Victory => {
                self.battles_won += 1;
                if self.fastest_battle_win.is_none() || duration < self.fastest_battle_win.unwrap() {
                    self.fastest_battle_win = Some(duration);
                }
            }
            BattleOutcome::Defeat => {
                self.battles_lost += 1;
            }
            _ => {}
        }
        
        self.last_play_date = Utc::now();
    }

    pub fn get_win_rate(&self) -> f32 {
        let total_battles = self.battles_won + self.battles_lost;
        if total_battles == 0 {
            0.0
        } else {
            (self.battles_won as f32 / total_battles as f32) * 100.0
        }
    }

    pub fn get_catch_rate(&self) -> f32 {
        if self.pokemon_seen == 0 {
            0.0
        } else {
            (self.pokemon_caught as f32 / self.pokemon_seen as f32) * 100.0
        }
    }
}

/// 里程碑系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub milestone_type: MilestoneType,
    pub achieved_date: DateTime<Utc>,
    pub description: String,
    pub reward: Option<MilestoneReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneType {
    PlaytimeHours(i64),
    PokemonCaught(u32),
    BattlesWon(u32),
    PokedexCompletion(f32),
    LocationsVisited(u32),
    LevelReached(u8),
    MoneyEarned(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneReward {
    Money(i64),
    Item(String, u32),
    Title(String),
    Achievement(AchievementId),
}

/// 玩家偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPreferences {
    pub auto_save: bool,
    pub battle_animations: bool,
    pub encounter_notifications: bool,
    pub achievement_notifications: bool,
    pub quest_reminders: bool,
    pub favorite_pokemon_types: Vec<Type>,
    pub preferred_language: String,
    pub difficulty_level: DifficultyLevel,
}

impl Default for PlayerPreferences {
    fn default() -> Self {
        Self {
            auto_save: true,
            battle_animations: true,
            encounter_notifications: true,
            achievement_notifications: true,
            quest_reminders: true,
            favorite_pokemon_types: Vec::new(),
            preferred_language: "English".to_string(),
            difficulty_level: DifficultyLevel::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Normal,
    Hard,
    Expert,
}

/// 进度摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionSummary {
    pub story_completion: f32,
    pub pokedex_completion: f32,
    pub achievement_completion: f32,
    pub badges_earned: usize,
    pub total_pokemon_caught: u32,
    pub total_battles_won: u32,
    pub playtime_hours: u32,
    pub active_quests: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_progression_creation() {
        let progression = PlayerProgression::new();
        assert!(!progression.player_id.to_string().is_empty());
        assert_eq!(progression.story_progress.current_chapter, 1);
        assert_eq!(progression.pokedex.get_seen_count(), 0);
    }

    #[test]
    fn test_achievement_tracking() {
        let mut tracker = AchievementTracker::new();
        let event = GameEvent::BattleWon;
        
        let earned = tracker.update_progress(event);
        assert_eq!(tracker.get_progress(1), 1); // First Victory achievement
        assert!(earned.contains(&1));
    }

    #[test]
    fn test_quest_system() {
        let mut quest_log = QuestLog::new();
        
        quest_log.start_quest(1).unwrap();
        assert!(quest_log.active_quests.contains(&1));
        
        let event = GameEvent::PokemonCaught(PokemonId(1));
        let completed = quest_log.update_quest_progress(&event);
        assert!(completed.contains(&1));
        assert!(quest_log.completed_quests.contains(&1));
    }

    #[test]
    fn test_pokedex_registration() {
        let mut pokedex = Pokedex::new();
        let pokemon_id = PokemonId(25);
        let location = LocationId(1);
        
        pokedex.register_sighting(pokemon_id, location);
        assert_eq!(pokedex.get_seen_count(), 1);
        
        let entry = pokedex.get_entry(pokemon_id).unwrap();
        assert_eq!(entry.status, PokedexStatus::Seen);
    }

    #[test]
    fn test_badge_collection() {
        let mut badges = BadgeCollection::new();
        
        assert!(badges.earn_badge(1));
        assert!(badges.is_earned(1));
        assert_eq!(badges.get_earned_count(), 1);
        
        // 不能重复获得同一徽章
        assert!(!badges.earn_badge(1));
    }

    #[test]
    fn test_milestone_checking() {
        let mut progression = PlayerProgression::new();
        progression.total_playtime = Duration::hours(10);
        
        let milestones = progression.check_milestones();
        assert!(!milestones.is_empty());
        
        // 应该有1小时和10小时的里程碑
        assert!(milestones.iter().any(|m| matches!(m.milestone_type, MilestoneType::PlaytimeHours(1))));
        assert!(milestones.iter().any(|m| matches!(m.milestone_type, MilestoneType::PlaytimeHours(10))));
    }
}