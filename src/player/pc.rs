// Pokemon PC Storage System - Pokemon电脑存储系统
//
// 开发心理过程：
// 1. Pokemon PC是游戏中重要的存储系统，需要管理大量Pokemon数据
// 2. 设计盒子系统，支持多个存储盒，每个盒子可存放30只Pokemon
// 3. 实现搜索、排序、标记等功能，方便玩家管理大量Pokemon
// 4. 支持盒子主题自定义和Pokemon快速访问
// 5. 与队伍系统集成，支持Pokemon在队伍和PC间的转移

use std::collections::{HashMap, BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::pokemon::{Individual, PokemonId, Type, Nature, Ability};
use crate::player::{Party, PartyError, PartySlot};
use crate::data::DatabaseError;

pub type BoxId = u32;
pub type BoxSlot = usize;

pub const POKEMON_PER_BOX: usize = 30;
pub const DEFAULT_BOX_COUNT: usize = 8;
pub const MAX_BOX_COUNT: usize = 32;

/// PC盒子主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoxTheme {
    Forest,
    Ocean,
    Desert,
    Mountain,
    City,
    Space,
    Volcano,
    Ice,
    Custom { name: String, background_id: u32 },
}

impl Default for BoxTheme {
    fn default() -> Self {
        BoxTheme::Forest
    }
}

/// PC存储盒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonBox {
    pub id: BoxId,
    pub name: String,
    pub theme: BoxTheme,
    pub pokemon: Vec<Option<StoredPokemon>>,
    pub is_locked: bool,
    pub created_date: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub tags: HashSet<String>,
    pub sort_order: BoxSortOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoxSortOrder {
    None,
    ByLevel,
    BySpecies,
    ByType,
    ByNature,
    ByDateCaught,
    ByName,
    Custom,
}

impl Default for BoxSortOrder {
    fn default() -> Self {
        BoxSortOrder::None
    }
}

impl PokemonBox {
    pub fn new(id: BoxId, name: String) -> Self {
        Self {
            id,
            name,
            theme: BoxTheme::default(),
            pokemon: vec![None; POKEMON_PER_BOX],
            is_locked: false,
            created_date: Utc::now(),
            last_accessed: Utc::now(),
            tags: HashSet::new(),
            sort_order: BoxSortOrder::default(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.pokemon.iter().all(|slot| slot.is_some())
    }

    pub fn is_empty(&self) -> bool {
        self.pokemon.iter().all(|slot| slot.is_none())
    }

    pub fn count(&self) -> usize {
        self.pokemon.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn available_slots(&self) -> Vec<BoxSlot> {
        self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| if slot.is_none() { Some(i) } else { None })
            .collect()
    }

    pub fn find_first_empty_slot(&self) -> Option<BoxSlot> {
        self.pokemon.iter()
            .enumerate()
            .find_map(|(i, slot)| if slot.is_none() { Some(i) } else { None })
    }

    pub fn store_pokemon(&mut self, pokemon: Individual) -> Result<BoxSlot, PCError> {
        if self.is_locked {
            return Err(PCError::BoxLocked);
        }

        if let Some(slot) = self.find_first_empty_slot() {
            let stored_pokemon = StoredPokemon::new(pokemon);
            self.pokemon[slot] = Some(stored_pokemon);
            self.last_accessed = Utc::now();
            Ok(slot)
        } else {
            Err(PCError::BoxFull)
        }
    }

    pub fn retrieve_pokemon(&mut self, slot: BoxSlot) -> Result<Individual, PCError> {
        if self.is_locked {
            return Err(PCError::BoxLocked);
        }

        if slot >= POKEMON_PER_BOX {
            return Err(PCError::InvalidSlot);
        }

        if let Some(stored_pokemon) = self.pokemon[slot].take() {
            self.last_accessed = Utc::now();
            Ok(stored_pokemon.pokemon)
        } else {
            Err(PCError::EmptySlot)
        }
    }

    pub fn get_pokemon(&self, slot: BoxSlot) -> Option<&StoredPokemon> {
        if slot < POKEMON_PER_BOX {
            self.pokemon[slot].as_ref()
        } else {
            None
        }
    }

    pub fn swap_pokemon(&mut self, slot_a: BoxSlot, slot_b: BoxSlot) -> Result<(), PCError> {
        if self.is_locked {
            return Err(PCError::BoxLocked);
        }

        if slot_a >= POKEMON_PER_BOX || slot_b >= POKEMON_PER_BOX {
            return Err(PCError::InvalidSlot);
        }

        self.pokemon.swap(slot_a, slot_b);
        self.last_accessed = Utc::now();
        Ok(())
    }

    pub fn auto_sort(&mut self) {
        if self.is_locked || self.sort_order == BoxSortOrder::None {
            return;
        }

        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        match self.sort_order {
            BoxSortOrder::ByLevel => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| b.pokemon.level.cmp(&a.pokemon.level));
            }
            BoxSortOrder::BySpecies => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| a.pokemon.species.id.0.cmp(&b.pokemon.species.id.0));
            }
            BoxSortOrder::ByType => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| {
                    let type_a = a.pokemon.species.types.first().unwrap_or(&Type::Normal);
                    let type_b = b.pokemon.species.types.first().unwrap_or(&Type::Normal);
                    type_a.cmp(type_b)
                });
            }
            BoxSortOrder::ByNature => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| a.pokemon.nature.cmp(&b.pokemon.nature));
            }
            BoxSortOrder::ByDateCaught => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| b.stored_date.cmp(&a.stored_date));
            }
            BoxSortOrder::ByName => {
                pokemon_with_slots.sort_by(|(_, a), (_, b)| {
                    let name_a = a.nickname.as_ref().unwrap_or(&a.pokemon.species.name);
                    let name_b = b.nickname.as_ref().unwrap_or(&b.pokemon.species.name);
                    name_a.cmp(name_b)
                });
            }
            _ => {}
        }

        // 清空盒子并重新排列Pokemon
        for slot in &mut self.pokemon {
            *slot = None;
        }

        for (new_position, (_, pokemon)) in pokemon_with_slots.into_iter().enumerate() {
            self.pokemon[new_position] = Some(pokemon);
        }

        self.last_accessed = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

/// 存储在PC中的Pokemon数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPokemon {
    pub pokemon: Individual,
    pub stored_date: DateTime<Utc>,
    pub nickname: Option<String>,
    pub original_trainer: String,
    pub location_caught: Option<String>,
    pub ball_type: PokeBallType,
    pub is_favorite: bool,
    pub markings: PokemonMarkings,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokeBallType {
    PokeBall,
    GreatBall,
    UltraBall,
    MasterBall,
    TimerBall,
    RepeatBall,
    LureBall,
    HeavyBall,
    LoveBall,
    FriendBall,
    MoonBall,
    SportBall,
    NetBall,
    NestBall,
    DiveBall,
    LuxuryBall,
    HealBall,
    QuickBall,
    DuskBall,
    CherishBall,
}

impl Default for PokeBallType {
    fn default() -> Self {
        PokeBallType::PokeBall
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonMarkings {
    pub circle: bool,
    pub triangle: bool,
    pub square: bool,
    pub heart: bool,
    pub star: bool,
    pub diamond: bool,
}

impl Default for PokemonMarkings {
    fn default() -> Self {
        Self {
            circle: false,
            triangle: false,
            square: false,
            heart: false,
            star: false,
            diamond: false,
        }
    }
}

impl StoredPokemon {
    pub fn new(pokemon: Individual) -> Self {
        Self {
            pokemon,
            stored_date: Utc::now(),
            nickname: None,
            original_trainer: "Player".to_string(),
            location_caught: None,
            ball_type: PokeBallType::default(),
            is_favorite: false,
            markings: PokemonMarkings::default(),
            notes: String::new(),
        }
    }

    pub fn with_details(pokemon: Individual, trainer: String, location: Option<String>, 
                       ball_type: PokeBallType) -> Self {
        Self {
            pokemon,
            stored_date: Utc::now(),
            nickname: None,
            original_trainer: trainer,
            location_caught: location,
            ball_type,
            is_favorite: false,
            markings: PokemonMarkings::default(),
            notes: String::new(),
        }
    }

    pub fn get_display_name(&self) -> &str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.species.name)
    }

    pub fn set_nickname(&mut self, nickname: Option<String>) {
        self.nickname = nickname;
    }

    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    pub fn add_marking(&mut self, marking: MarkingType) {
        match marking {
            MarkingType::Circle => self.markings.circle = true,
            MarkingType::Triangle => self.markings.triangle = true,
            MarkingType::Square => self.markings.square = true,
            MarkingType::Heart => self.markings.heart = true,
            MarkingType::Star => self.markings.star = true,
            MarkingType::Diamond => self.markings.diamond = true,
        }
    }

    pub fn remove_marking(&mut self, marking: MarkingType) {
        match marking {
            MarkingType::Circle => self.markings.circle = false,
            MarkingType::Triangle => self.markings.triangle = false,
            MarkingType::Square => self.markings.square = false,
            MarkingType::Heart => self.markings.heart = false,
            MarkingType::Star => self.markings.star = false,
            MarkingType::Diamond => self.markings.diamond = false,
        }
    }

    pub fn has_marking(&self, marking: MarkingType) -> bool {
        match marking {
            MarkingType::Circle => self.markings.circle,
            MarkingType::Triangle => self.markings.triangle,
            MarkingType::Square => self.markings.square,
            MarkingType::Heart => self.markings.heart,
            MarkingType::Star => self.markings.star,
            MarkingType::Diamond => self.markings.diamond,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkingType {
    Circle,
    Triangle,
    Square,
    Heart,
    Star,
    Diamond,
}

/// Pokemon PC存储系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonPC {
    pub boxes: HashMap<BoxId, PokemonBox>,
    pub current_box: BoxId,
    pub next_box_id: BoxId,
    pub wallpaper: String,
    pub music_theme: String,
    pub auto_sort_enabled: bool,
    pub favorites_filter: bool,
    pub search_history: Vec<String>,
}

impl Default for PokemonPC {
    fn default() -> Self {
        Self::new()
    }
}

impl PokemonPC {
    pub fn new() -> Self {
        let mut pc = Self {
            boxes: HashMap::new(),
            current_box: 1,
            next_box_id: 1,
            wallpaper: "default".to_string(),
            music_theme: "default".to_string(),
            auto_sort_enabled: false,
            favorites_filter: false,
            search_history: Vec::new(),
        };

        // 创建默认盒子
        for i in 0..DEFAULT_BOX_COUNT {
            let box_name = format!("Box {}", i + 1);
            pc.create_box(box_name).unwrap();
        }

        pc
    }

    pub fn create_box(&mut self, name: String) -> Result<BoxId, PCError> {
        if self.boxes.len() >= MAX_BOX_COUNT {
            return Err(PCError::MaxBoxesReached);
        }

        let box_id = self.next_box_id;
        let pokemon_box = PokemonBox::new(box_id, name);
        self.boxes.insert(box_id, pokemon_box);
        self.next_box_id += 1;

        Ok(box_id)
    }

    pub fn delete_box(&mut self, box_id: BoxId) -> Result<(), PCError> {
        if self.boxes.len() <= 1 {
            return Err(PCError::CannotDeleteLastBox);
        }

        if let Some(pokemon_box) = self.boxes.get(&box_id) {
            if !pokemon_box.is_empty() {
                return Err(PCError::BoxNotEmpty);
            }
        }

        self.boxes.remove(&box_id);

        // 如果删除的是当前盒子，切换到第一个可用盒子
        if self.current_box == box_id {
            self.current_box = *self.boxes.keys().next().unwrap();
        }

        Ok(())
    }

    pub fn get_box(&self, box_id: BoxId) -> Option<&PokemonBox> {
        self.boxes.get(&box_id)
    }

    pub fn get_box_mut(&mut self, box_id: BoxId) -> Option<&mut PokemonBox> {
        self.boxes.get_mut(&box_id)
    }

    pub fn get_current_box(&self) -> Option<&PokemonBox> {
        self.get_box(self.current_box)
    }

    pub fn get_current_box_mut(&mut self) -> Option<&mut PokemonBox> {
        self.get_box_mut(self.current_box)
    }

    pub fn switch_box(&mut self, box_id: BoxId) -> Result<(), PCError> {
        if !self.boxes.contains_key(&box_id) {
            return Err(PCError::BoxNotFound);
        }

        self.current_box = box_id;
        
        if let Some(current_box) = self.get_box_mut(self.current_box) {
            current_box.last_accessed = Utc::now();
        }

        Ok(())
    }

    pub fn store_pokemon(&mut self, pokemon: Individual) -> Result<(BoxId, BoxSlot), PCError> {
        // 尝试在当前盒子存储
        if let Some(current_box) = self.get_box_mut(self.current_box) {
            if let Ok(slot) = current_box.store_pokemon(pokemon.clone()) {
                return Ok((self.current_box, slot));
            }
        }

        // 当前盒子满了，找第一个可用盒子
        for (&box_id, pokemon_box) in &mut self.boxes {
            if !pokemon_box.is_full() {
                if let Ok(slot) = pokemon_box.store_pokemon(pokemon) {
                    return Ok((box_id, slot));
                }
            }
        }

        Err(PCError::AllBoxesFull)
    }

    pub fn retrieve_pokemon(&mut self, box_id: BoxId, slot: BoxSlot) -> Result<Individual, PCError> {
        if let Some(pokemon_box) = self.get_box_mut(box_id) {
            pokemon_box.retrieve_pokemon(slot)
        } else {
            Err(PCError::BoxNotFound)
        }
    }

    pub fn move_pokemon(&mut self, from_box: BoxId, from_slot: BoxSlot, 
                       to_box: BoxId, to_slot: BoxSlot) -> Result<(), PCError> {
        if from_box == to_box {
            // 同一盒子内移动
            if let Some(pokemon_box) = self.get_box_mut(from_box) {
                return pokemon_box.swap_pokemon(from_slot, to_slot);
            }
        } else {
            // 跨盒子移动
            let pokemon = self.retrieve_pokemon(from_box, from_slot)?;
            
            if let Some(to_box_ref) = self.get_box_mut(to_box) {
                if to_slot >= POKEMON_PER_BOX {
                    return Err(PCError::InvalidSlot);
                }
                
                if to_box_ref.pokemon[to_slot].is_some() {
                    return Err(PCError::SlotOccupied);
                }
                
                let stored_pokemon = StoredPokemon::new(pokemon);
                to_box_ref.pokemon[to_slot] = Some(stored_pokemon);
                to_box_ref.last_accessed = Utc::now();
            } else {
                return Err(PCError::BoxNotFound);
            }
        }

        Ok(())
    }

    pub fn search_pokemon(&mut self, criteria: &SearchCriteria) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for (&box_id, pokemon_box) in &self.boxes {
            for (slot, stored_pokemon_opt) in pokemon_box.pokemon.iter().enumerate() {
                if let Some(ref stored_pokemon) = stored_pokemon_opt {
                    if criteria.matches(stored_pokemon) {
                        results.push(SearchResult {
                            box_id,
                            slot,
                            pokemon: stored_pokemon.clone(),
                        });
                    }
                }
            }
        }

        // 保存搜索记录
        if let Some(query) = &criteria.name_query {
            self.search_history.push(query.clone());
            self.search_history.truncate(20); // 保留最近20次搜索
        }

        results
    }

    pub fn get_total_pokemon_count(&self) -> usize {
        self.boxes.values().map(|pokemon_box| pokemon_box.count()).sum()
    }

    pub fn get_box_usage_stats(&self) -> HashMap<BoxId, BoxStats> {
        let mut stats = HashMap::new();

        for (&box_id, pokemon_box) in &self.boxes {
            let count = pokemon_box.count();
            let percentage = (count as f32 / POKEMON_PER_BOX as f32) * 100.0;

            stats.insert(box_id, BoxStats {
                total_pokemon: count,
                usage_percentage: percentage,
                last_accessed: pokemon_box.last_accessed,
                is_full: pokemon_box.is_full(),
            });
        }

        stats
    }

    pub fn get_type_distribution(&self) -> HashMap<Type, usize> {
        let mut distribution = HashMap::new();

        for pokemon_box in self.boxes.values() {
            for stored_pokemon_opt in &pokemon_box.pokemon {
                if let Some(ref stored_pokemon) = stored_pokemon_opt {
                    for pokemon_type in &stored_pokemon.pokemon.species.types {
                        *distribution.entry(*pokemon_type).or_insert(0) += 1;
                    }
                }
            }
        }

        distribution
    }

    pub fn backup_to_data(&self) -> PCBackupData {
        PCBackupData {
            boxes: self.boxes.clone(),
            current_box: self.current_box,
            wallpaper: self.wallpaper.clone(),
            music_theme: self.music_theme.clone(),
            backup_date: Utc::now(),
        }
    }

    pub fn restore_from_backup(&mut self, backup: PCBackupData) -> Result<(), PCError> {
        self.boxes = backup.boxes;
        self.current_box = backup.current_box;
        self.wallpaper = backup.wallpaper;
        self.music_theme = backup.music_theme;

        // 重新计算next_box_id
        self.next_box_id = self.boxes.keys().max().unwrap_or(&0) + 1;

        Ok(())
    }
}

/// 搜索条件
#[derive(Debug, Clone, Default)]
pub struct SearchCriteria {
    pub name_query: Option<String>,
    pub species_id: Option<PokemonId>,
    pub types: Vec<Type>,
    pub nature: Option<Nature>,
    pub level_min: Option<u8>,
    pub level_max: Option<u8>,
    pub is_favorite: Option<bool>,
    pub has_nickname: Option<bool>,
    pub ball_type: Option<PokeBallType>,
    pub markings: Vec<MarkingType>,
    pub trainer: Option<String>,
    pub location_caught: Option<String>,
}

impl SearchCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name_query = Some(name);
        self
    }

    pub fn with_species(mut self, species_id: PokemonId) -> Self {
        self.species_id = Some(species_id);
        self
    }

    pub fn with_type(mut self, pokemon_type: Type) -> Self {
        self.types.push(pokemon_type);
        self
    }

    pub fn with_level_range(mut self, min: u8, max: u8) -> Self {
        self.level_min = Some(min);
        self.level_max = Some(max);
        self
    }

    pub fn favorites_only(mut self) -> Self {
        self.is_favorite = Some(true);
        self
    }

    pub fn matches(&self, stored_pokemon: &StoredPokemon) -> bool {
        // 名称匹配
        if let Some(ref query) = self.name_query {
            let pokemon_name = stored_pokemon.get_display_name().to_lowercase();
            let species_name = stored_pokemon.pokemon.species.name.to_lowercase();
            let query_lower = query.to_lowercase();
            
            if !pokemon_name.contains(&query_lower) && !species_name.contains(&query_lower) {
                return false;
            }
        }

        // 种族匹配
        if let Some(species_id) = self.species_id {
            if stored_pokemon.pokemon.species.id != species_id {
                return false;
            }
        }

        // 类型匹配
        if !self.types.is_empty() {
            let has_matching_type = self.types.iter()
                .any(|&search_type| stored_pokemon.pokemon.species.types.contains(&search_type));
            if !has_matching_type {
                return false;
            }
        }

        // 性格匹配
        if let Some(nature) = self.nature {
            if stored_pokemon.pokemon.nature != nature {
                return false;
            }
        }

        // 等级范围匹配
        if let Some(min_level) = self.level_min {
            if stored_pokemon.pokemon.level < min_level {
                return false;
            }
        }
        if let Some(max_level) = self.level_max {
            if stored_pokemon.pokemon.level > max_level {
                return false;
            }
        }

        // 收藏状态匹配
        if let Some(is_favorite) = self.is_favorite {
            if stored_pokemon.is_favorite != is_favorite {
                return false;
            }
        }

        // 昵称状态匹配
        if let Some(has_nickname) = self.has_nickname {
            let pokemon_has_nickname = stored_pokemon.nickname.is_some();
            if pokemon_has_nickname != has_nickname {
                return false;
            }
        }

        // 精灵球类型匹配
        if let Some(ball_type) = self.ball_type {
            if stored_pokemon.ball_type != ball_type {
                return false;
            }
        }

        // 标记匹配
        if !self.markings.is_empty() {
            let has_matching_marking = self.markings.iter()
                .any(|&marking| stored_pokemon.has_marking(marking));
            if !has_matching_marking {
                return false;
            }
        }

        // 训练师匹配
        if let Some(ref trainer) = self.trainer {
            if stored_pokemon.original_trainer.to_lowercase() != trainer.to_lowercase() {
                return false;
            }
        }

        // 捕获地点匹配
        if let Some(ref location) = self.location_caught {
            match &stored_pokemon.location_caught {
                Some(pokemon_location) => {
                    if pokemon_location.to_lowercase() != location.to_lowercase() {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub box_id: BoxId,
    pub slot: BoxSlot,
    pub pokemon: StoredPokemon,
}

/// 盒子统计信息
#[derive(Debug, Clone)]
pub struct BoxStats {
    pub total_pokemon: usize,
    pub usage_percentage: f32,
    pub last_accessed: DateTime<Utc>,
    pub is_full: bool,
}

/// PC备份数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCBackupData {
    pub boxes: HashMap<BoxId, PokemonBox>,
    pub current_box: BoxId,
    pub wallpaper: String,
    pub music_theme: String,
    pub backup_date: DateTime<Utc>,
}

/// PC与队伍系统的集成接口
pub struct PCPartyInterface<'a> {
    pub pc: &'a mut PokemonPC,
    pub party: &'a mut Party,
}

impl<'a> PCPartyInterface<'a> {
    pub fn new(pc: &'a mut PokemonPC, party: &'a mut Party) -> Self {
        Self { pc, party }
    }

    pub fn move_to_party(&mut self, box_id: BoxId, box_slot: BoxSlot) -> Result<PartySlot, PCError> {
        if self.party.is_full() {
            return Err(PCError::PartyFull);
        }

        let pokemon = self.pc.retrieve_pokemon(box_id, box_slot)?;
        
        match self.party.add_pokemon(pokemon) {
            Ok(party_slot) => Ok(party_slot),
            Err(PartyError::PartyFull) => Err(PCError::PartyFull),
            Err(_) => Err(PCError::TransferFailed),
        }
    }

    pub fn move_to_pc(&mut self, party_slot: PartySlot) -> Result<(BoxId, BoxSlot), PCError> {
        match self.party.remove_pokemon(party_slot) {
            Ok(pokemon) => self.pc.store_pokemon(pokemon),
            Err(PartyError::CannotRemoveLastBattler) => Err(PCError::CannotStoreLastBattler),
            Err(PartyError::EmptySlot) => Err(PCError::EmptySlot),
            Err(_) => Err(PCError::TransferFailed),
        }
    }

    pub fn swap_party_pc(&mut self, party_slot: PartySlot, box_id: BoxId, 
                        box_slot: BoxSlot) -> Result<(), PCError> {
        // 先从队伍取出Pokemon
        let party_pokemon = match self.party.remove_pokemon(party_slot) {
            Ok(pokemon) => pokemon,
            Err(PartyError::CannotRemoveLastBattler) => return Err(PCError::CannotStoreLastBattler),
            Err(_) => return Err(PCError::TransferFailed),
        };

        // 从PC取出Pokemon
        let pc_pokemon = match self.pc.retrieve_pokemon(box_id, box_slot) {
            Ok(pokemon) => pokemon,
            Err(e) => {
                // 交换失败，将Pokemon重新放回队伍
                let _ = self.party.add_pokemon(party_pokemon);
                return Err(e);
            }
        };

        // 将PC的Pokemon加入队伍
        match self.party.add_pokemon(pc_pokemon) {
            Ok(_) => {
                // 将队伍的Pokemon存入PC
                match self.pc.store_pokemon(party_pokemon) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            Err(_) => {
                // 如果添加到队伍失败，将PC Pokemon重新存回
                let _ = self.pc.store_pokemon(pc_pokemon);
                let _ = self.party.add_pokemon(party_pokemon);
                Err(PCError::TransferFailed)
            }
        }
    }
}

/// PC系统错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PCError {
    BoxFull,
    BoxNotFound,
    BoxLocked,
    BoxNotEmpty,
    EmptySlot,
    InvalidSlot,
    SlotOccupied,
    MaxBoxesReached,
    CannotDeleteLastBox,
    AllBoxesFull,
    PartyFull,
    CannotStoreLastBattler,
    TransferFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{PokemonSpecies, Stats};

    fn create_test_pokemon() -> Individual {
        let species = PokemonSpecies {
            id: PokemonId(25),
            name: "Pikachu".to_string(),
            types: vec![Type::Electric],
            base_stats: Stats {
                hp: 35,
                attack: 55,
                defense: 40,
                special_attack: 50,
                special_defense: 50,
                speed: 90,
            },
            abilities: vec![],
            moves_learned: vec![],
            evolution_chain: None,
        };

        Individual::new(species, 5, Nature::Hardy)
    }

    #[test]
    fn test_pc_creation() {
        let pc = PokemonPC::new();
        assert_eq!(pc.boxes.len(), DEFAULT_BOX_COUNT);
        assert!(!pc.boxes.is_empty());
    }

    #[test]
    fn test_store_pokemon() {
        let mut pc = PokemonPC::new();
        let pokemon = create_test_pokemon();
        
        let result = pc.store_pokemon(pokemon);
        assert!(result.is_ok());
        
        let (box_id, slot) = result.unwrap();
        assert_eq!(pc.get_total_pokemon_count(), 1);
        
        let stored_pokemon = pc.get_box(box_id).unwrap().get_pokemon(slot);
        assert!(stored_pokemon.is_some());
    }

    #[test]
    fn test_search_pokemon() {
        let mut pc = PokemonPC::new();
        let pokemon = create_test_pokemon();
        
        pc.store_pokemon(pokemon).unwrap();
        
        let criteria = SearchCriteria::new()
            .with_name("Pikachu".to_string())
            .with_type(Type::Electric);
        
        let results = pc.search_pokemon(&criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].pokemon.pokemon.species.name, "Pikachu");
    }

    #[test]
    fn test_box_management() {
        let mut pc = PokemonPC::new();
        
        let box_id = pc.create_box("Test Box".to_string()).unwrap();
        assert!(pc.get_box(box_id).is_some());
        
        let switch_result = pc.switch_box(box_id);
        assert!(switch_result.is_ok());
        assert_eq!(pc.current_box, box_id);
        
        // 测试删除空盒子
        let delete_result = pc.delete_box(box_id);
        assert!(delete_result.is_ok());
        assert!(pc.get_box(box_id).is_none());
    }

    #[test]
    fn test_pokemon_markings() {
        let mut stored_pokemon = StoredPokemon::new(create_test_pokemon());
        
        assert!(!stored_pokemon.has_marking(MarkingType::Heart));
        
        stored_pokemon.add_marking(MarkingType::Heart);
        assert!(stored_pokemon.has_marking(MarkingType::Heart));
        
        stored_pokemon.remove_marking(MarkingType::Heart);
        assert!(!stored_pokemon.has_marking(MarkingType::Heart));
    }

    #[test]
    fn test_pc_party_interface() {
        let mut pc = PokemonPC::new();
        let mut party = Party::new();
        let pokemon = create_test_pokemon();
        
        let (box_id, slot) = pc.store_pokemon(pokemon).unwrap();
        
        let mut interface = PCPartyInterface::new(&mut pc, &mut party);
        let party_slot = interface.move_to_party(box_id, slot).unwrap();
        
        assert_eq!(party.count(), 1);
        assert_eq!(pc.get_total_pokemon_count(), 0);
        
        let (new_box_id, new_slot) = interface.move_to_pc(party_slot).unwrap();
        assert_eq!(party.count(), 0);
        assert_eq!(pc.get_total_pokemon_count(), 1);
    }
}