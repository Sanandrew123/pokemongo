// Player Party System - 玩家队伍管理系统
//
// 开发心理过程：
// 1. 这是Pokemon游戏的核心系统之一，管理玩家携带的Pokemon队伍
// 2. 需要支持队伍编辑、Pokemon状态管理、经验分配、能力值计算等
// 3. 考虑到战斗系统，需要维护HP、PP、状态异常等动态数据
// 4. 实现队伍排序、替换、存储等功能，支持不同的队伍配置策略
// 5. 为UI系统和战斗系统提供数据接口，确保数据一致性

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;

use crate::pokemon::{PokemonId, PokemonSpecies, Individual, Stats, StatusCondition};
use crate::battle::{BattleStats, Move, MoveId};
use crate::data::DatabaseError;

pub type PartySlot = usize;
pub const MAX_PARTY_SIZE: usize = 6;
pub const MIN_PARTY_SIZE: usize = 1;

/// Pokemon在队伍中的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyPokemon {
    pub pokemon: Individual,
    pub current_hp: u32,
    pub current_pp: Vec<u32>,
    pub status_condition: Option<StatusCondition>,
    pub experience_pending: u64,
    pub friendship: u8,
    pub is_fainted: bool,
    pub battle_stats: Option<BattleStats>,
    pub position: PartySlot,
    pub nickname: Option<String>,
    pub held_item: Option<ItemId>,
    pub ribbon_count: u8,
    pub contest_stats: ContestStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestStats {
    pub beauty: u8,
    pub cute: u8,
    pub smart: u8,
    pub tough: u8,
    pub cool: u8,
}

impl Default for ContestStats {
    fn default() -> Self {
        Self {
            beauty: 0,
            cute: 0,
            smart: 0,
            tough: 0,
            cool: 0,
        }
    }
}

impl PartyPokemon {
    pub fn new(pokemon: Individual, position: PartySlot) -> Self {
        let max_hp = pokemon.calculate_stat(crate::pokemon::StatType::HP);
        let current_pp = pokemon.moves.iter()
            .map(|move_data| move_data.move_info.pp)
            .collect();

        Self {
            pokemon,
            current_hp: max_hp,
            current_pp,
            status_condition: None,
            experience_pending: 0,
            friendship: 70, // 默认亲密度
            is_fainted: false,
            battle_stats: None,
            position,
            nickname: None,
            held_item: None,
            ribbon_count: 0,
            contest_stats: ContestStats::default(),
        }
    }

    pub fn is_able_to_battle(&self) -> bool {
        !self.is_fainted && self.current_hp > 0 && 
        !matches!(self.status_condition, Some(StatusCondition::Fainted))
    }

    pub fn get_effective_stats(&self) -> Stats {
        let mut base_stats = self.pokemon.calculate_all_stats();
        
        // 应用状态异常对能力值的影响
        if let Some(ref condition) = self.status_condition {
            base_stats = condition.apply_stat_modifications(base_stats);
        }

        // 应用道具效果
        if let Some(item_id) = self.held_item {
            base_stats = self.apply_item_stat_boost(base_stats, item_id);
        }

        base_stats
    }

    fn apply_item_stat_boost(&self, mut stats: Stats, item_id: ItemId) -> Stats {
        // 根据道具ID应用不同的能力值提升
        match item_id.0 {
            // 攻击类道具
            1001 => { stats.attack = (stats.attack as f32 * 1.1) as u32; }, // 力量头带
            1002 => { stats.special_attack = (stats.special_attack as f32 * 1.1) as u32; }, // 智慧眼镜
            1003 => { stats.defense = (stats.defense as f32 * 1.1) as u32; }, // 防御背心
            1004 => { stats.special_defense = (stats.special_defense as f32 * 1.1) as u32; }, // 特防围巾
            1005 => { stats.speed = (stats.speed as f32 * 1.1) as u32; }, // 速度药草
            
            // 回复类道具效果在其他地方处理
            _ => {}
        }
        
        stats
    }

    pub fn heal(&mut self, amount: u32) -> u32 {
        let max_hp = self.pokemon.calculate_stat(crate::pokemon::StatType::HP);
        let old_hp = self.current_hp;
        self.current_hp = (self.current_hp + amount).min(max_hp);
        
        if self.current_hp > 0 {
            self.is_fainted = false;
            if matches!(self.status_condition, Some(StatusCondition::Fainted)) {
                self.status_condition = None;
            }
        }
        
        self.current_hp - old_hp
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        if damage >= self.current_hp {
            self.current_hp = 0;
            self.is_fainted = true;
            self.status_condition = Some(StatusCondition::Fainted);
            true // Pokemon倒下了
        } else {
            self.current_hp -= damage;
            false
        }
    }

    pub fn restore_pp(&mut self, move_slot: usize, amount: u32) -> bool {
        if move_slot >= self.current_pp.len() {
            return false;
        }

        let max_pp = self.pokemon.moves[move_slot].move_info.pp;
        self.current_pp[move_slot] = (self.current_pp[move_slot] + amount).min(max_pp);
        true
    }

    pub fn use_move(&mut self, move_slot: usize) -> bool {
        if move_slot >= self.current_pp.len() || self.current_pp[move_slot] == 0 {
            return false;
        }

        self.current_pp[move_slot] -= 1;
        true
    }

    pub fn gain_experience(&mut self, exp: u64) -> bool {
        let old_level = self.pokemon.level;
        self.pokemon.gain_experience(exp);
        
        // 升级时恢复HP和PP
        if self.pokemon.level > old_level {
            self.level_up_heal();
            return true;
        }
        
        false
    }

    fn level_up_heal(&mut self) {
        let max_hp = self.pokemon.calculate_stat(crate::pokemon::StatType::HP);
        self.current_hp = max_hp;
        
        for (i, move_data) in self.pokemon.moves.iter().enumerate() {
            if i < self.current_pp.len() {
                self.current_pp[i] = move_data.move_info.pp;
            }
        }
        
        self.is_fainted = false;
        if matches!(self.status_condition, Some(StatusCondition::Fainted)) {
            self.status_condition = None;
        }
    }

    pub fn apply_status_condition(&mut self, condition: StatusCondition) -> bool {
        // 检查是否可以应用状态异常
        if let Some(ref current) = self.status_condition {
            if current.priority() >= condition.priority() {
                return false; // 当前状态异常优先级更高
            }
        }

        self.status_condition = Some(condition);
        
        // 特殊处理某些状态异常
        match condition {
            StatusCondition::Fainted => {
                self.is_fainted = true;
                self.current_hp = 0;
            }
            _ => {}
        }
        
        true
    }

    pub fn cure_status_condition(&mut self) {
        self.status_condition = None;
    }

    pub fn increase_friendship(&mut self, amount: u8) {
        self.friendship = self.friendship.saturating_add(amount).min(255);
    }

    pub fn decrease_friendship(&mut self, amount: u8) {
        self.friendship = self.friendship.saturating_sub(amount);
    }

    pub fn get_happiness_level(&self) -> HappinessLevel {
        match self.friendship {
            0..=49 => HappinessLevel::Unhappy,
            50..=149 => HappinessLevel::Neutral,
            150..=219 => HappinessLevel::Happy,
            220..=255 => HappinessLevel::VeryHappy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HappinessLevel {
    Unhappy,
    Neutral,
    Happy,
    VeryHappy,
}

/// 玩家队伍管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub pokemon: Vec<Option<PartyPokemon>>,
    pub active_pokemon: Option<PartySlot>,
    pub last_battle_participant: Option<PartySlot>,
    pub experience_share: bool,
    pub auto_sort: bool,
    pub formation: PartyFormation,
    pub battle_box_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyFormation {
    Standard,      // 标准队形
    OffensiveFocus, // 攻击导向
    DefensiveFocus, // 防御导向
    SpeedFocus,     // 速度导向
    Balanced,       // 平衡队形
    Custom,         // 自定义
}

impl Default for Party {
    fn default() -> Self {
        Self::new()
    }
}

impl Party {
    pub fn new() -> Self {
        Self {
            pokemon: vec![None; MAX_PARTY_SIZE],
            active_pokemon: None,
            last_battle_participant: None,
            experience_share: false,
            auto_sort: false,
            formation: PartyFormation::Standard,
            battle_box_mode: false,
        }
    }

    pub fn add_pokemon(&mut self, pokemon: Individual) -> Result<PartySlot, PartyError> {
        if self.is_full() {
            return Err(PartyError::PartyFull);
        }

        for (i, slot) in self.pokemon.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(PartyPokemon::new(pokemon, i));
                
                // 如果是第一只Pokemon，设为活跃Pokemon
                if self.active_pokemon.is_none() {
                    self.active_pokemon = Some(i);
                }
                
                return Ok(i);
            }
        }

        Err(PartyError::PartyFull)
    }

    pub fn remove_pokemon(&mut self, slot: PartySlot) -> Result<Individual, PartyError> {
        if slot >= MAX_PARTY_SIZE {
            return Err(PartyError::InvalidSlot);
        }

        if let Some(party_pokemon) = self.pokemon[slot].take() {
            // 检查是否是最后一只可战斗的Pokemon
            if self.get_battle_ready_count() == 0 {
                // 重新放回，不能移除最后一只可战斗的Pokemon
                self.pokemon[slot] = Some(party_pokemon);
                return Err(PartyError::CannotRemoveLastBattler);
            }

            // 更新活跃Pokemon
            if self.active_pokemon == Some(slot) {
                self.active_pokemon = self.find_first_battle_ready();
            }

            Ok(party_pokemon.pokemon)
        } else {
            Err(PartyError::EmptySlot)
        }
    }

    pub fn swap_pokemon(&mut self, slot_a: PartySlot, slot_b: PartySlot) -> Result<(), PartyError> {
        if slot_a >= MAX_PARTY_SIZE || slot_b >= MAX_PARTY_SIZE {
            return Err(PartyError::InvalidSlot);
        }

        if slot_a == slot_b {
            return Ok(());
        }

        // 交换Pokemon
        let temp = self.pokemon[slot_a].take();
        self.pokemon[slot_a] = self.pokemon[slot_b].take();
        self.pokemon[slot_b] = temp;

        // 更新位置信息
        if let Some(ref mut pokemon) = self.pokemon[slot_a] {
            pokemon.position = slot_a;
        }
        if let Some(ref mut pokemon) = self.pokemon[slot_b] {
            pokemon.position = slot_b;
        }

        // 更新活跃Pokemon引用
        if self.active_pokemon == Some(slot_a) {
            self.active_pokemon = Some(slot_b);
        } else if self.active_pokemon == Some(slot_b) {
            self.active_pokemon = Some(slot_a);
        }

        Ok(())
    }

    pub fn get_pokemon(&self, slot: PartySlot) -> Option<&PartyPokemon> {
        if slot < MAX_PARTY_SIZE {
            self.pokemon[slot].as_ref()
        } else {
            None
        }
    }

    pub fn get_pokemon_mut(&mut self, slot: PartySlot) -> Option<&mut PartyPokemon> {
        if slot < MAX_PARTY_SIZE {
            self.pokemon[slot].as_mut()
        } else {
            None
        }
    }

    pub fn get_active_pokemon(&self) -> Option<&PartyPokemon> {
        if let Some(slot) = self.active_pokemon {
            self.get_pokemon(slot)
        } else {
            None
        }
    }

    pub fn get_active_pokemon_mut(&mut self) -> Option<&mut PartyPokemon> {
        if let Some(slot) = self.active_pokemon {
            self.get_pokemon_mut(slot)
        } else {
            None
        }
    }

    pub fn set_active_pokemon(&mut self, slot: PartySlot) -> Result<(), PartyError> {
        if slot >= MAX_PARTY_SIZE {
            return Err(PartyError::InvalidSlot);
        }

        if let Some(ref pokemon) = self.pokemon[slot] {
            if pokemon.is_able_to_battle() {
                self.active_pokemon = Some(slot);
                Ok(())
            } else {
                Err(PartyError::PokemonNotBattleReady)
            }
        } else {
            Err(PartyError::EmptySlot)
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

    pub fn get_battle_ready_count(&self) -> usize {
        self.pokemon.iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|pokemon| pokemon.is_able_to_battle())
            .count()
    }

    pub fn find_first_battle_ready(&self) -> Option<PartySlot> {
        for (i, pokemon) in self.pokemon.iter().enumerate() {
            if let Some(ref pokemon) = pokemon {
                if pokemon.is_able_to_battle() {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn get_all_battle_ready(&self) -> Vec<PartySlot> {
        self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                if let Some(ref pokemon) = slot {
                    if pokemon.is_able_to_battle() {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn heal_all(&mut self) {
        for slot in &mut self.pokemon {
            if let Some(ref mut pokemon) = slot {
                let max_hp = pokemon.pokemon.calculate_stat(crate::pokemon::StatType::HP);
                pokemon.current_hp = max_hp;
                pokemon.is_fainted = false;
                pokemon.status_condition = None;
                
                // 恢复所有PP
                for (i, move_data) in pokemon.pokemon.moves.iter().enumerate() {
                    if i < pokemon.current_pp.len() {
                        pokemon.current_pp[i] = move_data.move_info.pp;
                    }
                }
            }
        }
    }

    pub fn apply_experience_share(&mut self, total_exp: u64, battle_participants: &[PartySlot]) {
        if self.experience_share {
            // 经验学习装置模式：所有Pokemon分享经验
            let per_pokemon_exp = total_exp / self.count() as u64;
            
            for slot in &mut self.pokemon {
                if let Some(ref mut pokemon) = slot {
                    pokemon.gain_experience(per_pokemon_exp);
                }
            }
        } else {
            // 正常模式：只有战斗参与者获得经验
            let per_participant_exp = total_exp / battle_participants.len() as u64;
            
            for &slot in battle_participants {
                if let Some(ref mut pokemon) = self.pokemon[slot] {
                    pokemon.gain_experience(per_participant_exp);
                }
            }
        }
    }

    pub fn auto_arrange_by_formation(&mut self) {
        if !self.auto_sort {
            return;
        }

        match self.formation {
            PartyFormation::Standard => {
                // 按等级排序
                self.sort_by_level();
            }
            PartyFormation::OffensiveFocus => {
                // 按攻击力排序
                self.sort_by_attack();
            }
            PartyFormation::DefensiveFocus => {
                // 按防御力排序
                self.sort_by_defense();
            }
            PartyFormation::SpeedFocus => {
                // 按速度排序
                self.sort_by_speed();
            }
            PartyFormation::Balanced => {
                // 按综合能力值排序
                self.sort_by_total_stats();
            }
            PartyFormation::Custom => {
                // 自定义排序，不自动调整
            }
        }
    }

    fn sort_by_level(&mut self) {
        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        pokemon_with_slots.sort_by(|(_, a), (_, b)| b.pokemon.level.cmp(&a.pokemon.level));

        self.rearrange_from_sorted(pokemon_with_slots);
    }

    fn sort_by_attack(&mut self) {
        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        pokemon_with_slots.sort_by(|(_, a), (_, b)| {
            let attack_a = a.get_effective_stats().attack;
            let attack_b = b.get_effective_stats().attack;
            attack_b.cmp(&attack_a)
        });

        self.rearrange_from_sorted(pokemon_with_slots);
    }

    fn sort_by_defense(&mut self) {
        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        pokemon_with_slots.sort_by(|(_, a), (_, b)| {
            let defense_a = a.get_effective_stats().defense;
            let defense_b = b.get_effective_stats().defense;
            defense_b.cmp(&defense_a)
        });

        self.rearrange_from_sorted(pokemon_with_slots);
    }

    fn sort_by_speed(&mut self) {
        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        pokemon_with_slots.sort_by(|(_, a), (_, b)| {
            let speed_a = a.get_effective_stats().speed;
            let speed_b = b.get_effective_stats().speed;
            speed_b.cmp(&speed_a)
        });

        self.rearrange_from_sorted(pokemon_with_slots);
    }

    fn sort_by_total_stats(&mut self) {
        let mut pokemon_with_slots: Vec<_> = self.pokemon.iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|p| (i, p.clone())))
            .collect();

        pokemon_with_slots.sort_by(|(_, a), (_, b)| {
            let total_a = a.get_effective_stats().total();
            let total_b = b.get_effective_stats().total();
            total_b.cmp(&total_a)
        });

        self.rearrange_from_sorted(pokemon_with_slots);
    }

    fn rearrange_from_sorted(&mut self, sorted_pokemon: Vec<(PartySlot, PartyPokemon)>) {
        // 清空当前队伍
        for slot in &mut self.pokemon {
            *slot = None;
        }

        // 重新排列Pokemon
        for (new_position, (old_position, mut pokemon)) in sorted_pokemon.into_iter().enumerate() {
            pokemon.position = new_position;
            self.pokemon[new_position] = Some(pokemon);
            
            // 更新活跃Pokemon引用
            if self.active_pokemon == Some(old_position) {
                self.active_pokemon = Some(new_position);
            }
        }
    }

    pub fn get_type_coverage(&self) -> HashMap<crate::pokemon::Type, usize> {
        let mut coverage = HashMap::new();
        
        for slot in &self.pokemon {
            if let Some(ref pokemon) = slot {
                for pokemon_type in &pokemon.pokemon.species.types {
                    *coverage.entry(*pokemon_type).or_insert(0) += 1;
                }
            }
        }
        
        coverage
    }

    pub fn get_party_summary(&self) -> PartySummary {
        let mut summary = PartySummary::default();
        
        for slot in &self.pokemon {
            if let Some(ref pokemon) = slot {
                summary.total_level += pokemon.pokemon.level as u32;
                summary.total_pokemon += 1;
                summary.battle_ready += if pokemon.is_able_to_battle() { 1 } else { 0 };
                
                let stats = pokemon.get_effective_stats();
                summary.average_attack += stats.attack;
                summary.average_defense += stats.defense;
                summary.average_speed += stats.speed;
                
                if pokemon.friendship >= 220 {
                    summary.high_friendship_count += 1;
                }
            }
        }
        
        if summary.total_pokemon > 0 {
            summary.average_level = summary.total_level as f32 / summary.total_pokemon as f32;
            summary.average_attack /= summary.total_pokemon;
            summary.average_defense /= summary.total_pokemon;
            summary.average_speed /= summary.total_pokemon;
        }
        
        summary
    }

    pub fn validate_party(&self) -> Vec<PartyValidationError> {
        let mut errors = Vec::new();
        
        if self.count() == 0 {
            errors.push(PartyValidationError::EmptyParty);
            return errors;
        }
        
        if self.get_battle_ready_count() == 0 {
            errors.push(PartyValidationError::NoBattleReadyPokemon);
        }
        
        if self.active_pokemon.is_none() {
            errors.push(PartyValidationError::NoActivePokemon);
        } else if let Some(slot) = self.active_pokemon {
            if let Some(ref pokemon) = self.pokemon[slot] {
                if !pokemon.is_able_to_battle() {
                    errors.push(PartyValidationError::ActivePokemonNotBattleReady);
                }
            } else {
                errors.push(PartyValidationError::ActiveSlotEmpty);
            }
        }
        
        // 检查重复的Pokemon
        let mut species_count = HashMap::new();
        for slot in &self.pokemon {
            if let Some(ref pokemon) = slot {
                let species_id = pokemon.pokemon.species.id;
                *species_count.entry(species_id).or_insert(0) += 1;
            }
        }
        
        if self.battle_box_mode {
            // 对战盒模式下不允许重复Pokemon
            for (species_id, count) in species_count {
                if count > 1 {
                    errors.push(PartyValidationError::DuplicateSpeciesInBattleBox(species_id));
                }
            }
        }
        
        errors
    }
}

#[derive(Debug, Clone, Default)]
pub struct PartySummary {
    pub total_pokemon: usize,
    pub battle_ready: usize,
    pub total_level: u32,
    pub average_level: f32,
    pub average_attack: u32,
    pub average_defense: u32,
    pub average_speed: u32,
    pub high_friendship_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyError {
    PartyFull,
    EmptySlot,
    InvalidSlot,
    PokemonNotBattleReady,
    CannotRemoveLastBattler,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyValidationError {
    EmptyParty,
    NoBattleReadyPokemon,
    NoActivePokemon,
    ActivePokemonNotBattleReady,
    ActiveSlotEmpty,
    DuplicateSpeciesInBattleBox(PokemonId),
}

/// 队伍管理器 - 提供高级队伍操作功能
pub struct PartyManager {
    pub current_party: Party,
    pub saved_formations: HashMap<String, Vec<Option<Individual>>>,
    pub battle_history: Vec<BattleRecord>,
    pub auto_healing_items: HashMap<ItemId, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub participants: Vec<PartySlot>,
    pub outcome: BattleOutcome,
    pub experience_gained: u64,
    pub location: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleOutcome {
    Victory,
    Defeat,
    Draw,
    Fled,
}

impl PartyManager {
    pub fn new() -> Self {
        Self {
            current_party: Party::new(),
            saved_formations: HashMap::new(),
            battle_history: Vec::new(),
            auto_healing_items: HashMap::new(),
        }
    }

    pub fn save_formation(&mut self, name: String) -> Result<(), PartyError> {
        let formation: Vec<Option<Individual>> = self.current_party.pokemon
            .iter()
            .map(|slot| slot.as_ref().map(|p| p.pokemon.clone()))
            .collect();
        
        self.saved_formations.insert(name, formation);
        Ok(())
    }

    pub fn load_formation(&mut self, name: &str) -> Result<(), PartyError> {
        if let Some(formation) = self.saved_formations.get(name).cloned() {
            let mut new_party = Party::new();
            
            for (i, pokemon_opt) in formation.into_iter().enumerate() {
                if let Some(pokemon) = pokemon_opt {
                    new_party.pokemon[i] = Some(PartyPokemon::new(pokemon, i));
                }
            }
            
            // 设置第一个有效Pokemon为活跃Pokemon
            new_party.active_pokemon = new_party.find_first_battle_ready();
            
            self.current_party = new_party;
            Ok(())
        } else {
            Err(PartyError::InvalidSlot) // 使用现有错误类型
        }
    }

    pub fn auto_use_healing_items(&mut self) -> Vec<(PartySlot, ItemId)> {
        let mut items_used = Vec::new();
        
        for (slot, pokemon_opt) in self.current_party.pokemon.iter_mut().enumerate() {
            if let Some(ref mut pokemon) = pokemon_opt {
                // 检查HP状态
                let max_hp = pokemon.pokemon.calculate_stat(crate::pokemon::StatType::HP);
                let hp_ratio = pokemon.current_hp as f32 / max_hp as f32;
                
                if hp_ratio < 0.3 { // HP低于30%时自动使用回复药
                    if let Some(&item_count) = self.auto_healing_items.get(&ItemId(2001)) {
                        if item_count > 0 {
                            pokemon.heal(50); // 回复药回复50HP
                            *self.auto_healing_items.get_mut(&ItemId(2001)).unwrap() -= 1;
                            items_used.push((slot, ItemId(2001)));
                        }
                    }
                }
                
                // 检查状态异常
                if pokemon.status_condition.is_some() {
                    if let Some(&item_count) = self.auto_healing_items.get(&ItemId(2002)) {
                        if item_count > 0 {
                            pokemon.cure_status_condition();
                            *self.auto_healing_items.get_mut(&ItemId(2002)).unwrap() -= 1;
                            items_used.push((slot, ItemId(2002)));
                        }
                    }
                }
            }
        }
        
        items_used
    }

    pub fn record_battle(&mut self, participants: Vec<PartySlot>, outcome: BattleOutcome, 
                        experience_gained: u64, location: String) {
        let record = BattleRecord {
            timestamp: chrono::Utc::now(),
            participants,
            outcome,
            experience_gained,
            location,
        };
        
        self.battle_history.push(record);
        
        // 保留最近的1000场战斗记录
        if self.battle_history.len() > 1000 {
            self.battle_history.remove(0);
        }
    }

    pub fn get_battle_statistics(&self) -> BattleStatistics {
        let mut stats = BattleStatistics::default();
        
        for record in &self.battle_history {
            stats.total_battles += 1;
            
            match record.outcome {
                BattleOutcome::Victory => stats.victories += 1,
                BattleOutcome::Defeat => stats.defeats += 1,
                BattleOutcome::Draw => stats.draws += 1,
                BattleOutcome::Fled => stats.fled += 1,
            }
            
            stats.total_experience += record.experience_gained;
        }
        
        if stats.total_battles > 0 {
            stats.win_rate = stats.victories as f32 / stats.total_battles as f32;
        }
        
        stats
    }

    pub fn optimize_party_for_battle(&mut self, opponent_types: &[crate::pokemon::Type]) {
        // 基于对手类型优化队伍排列
        let mut scored_pokemon: Vec<(PartySlot, f32)> = Vec::new();
        
        for (slot, pokemon_opt) in self.current_party.pokemon.iter().enumerate() {
            if let Some(ref pokemon) = pokemon_opt {
                if !pokemon.is_able_to_battle() {
                    continue;
                }
                
                let mut score = 0.0;
                
                // 基于类型相性计算得分
                for move_data in &pokemon.pokemon.moves {
                    for &opponent_type in opponent_types {
                        let effectiveness = move_data.move_info.move_type.effectiveness_against(opponent_type);
                        score += effectiveness;
                    }
                }
                
                // 基于Pokemon等级和能力值
                score += pokemon.pokemon.level as f32 * 0.1;
                score += pokemon.get_effective_stats().total() as f32 * 0.001;
                
                scored_pokemon.push((slot, score));
            }
        }
        
        // 按得分排序
        scored_pokemon.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 重新排列Pokemon（只移动前3名到前面）
        for (new_pos, (old_slot, _)) in scored_pokemon.iter().take(3).enumerate() {
            if new_pos != *old_slot {
                let _ = self.current_party.swap_pokemon(new_pos, *old_slot);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BattleStatistics {
    pub total_battles: u32,
    pub victories: u32,
    pub defeats: u32,
    pub draws: u32,
    pub fled: u32,
    pub win_rate: f32,
    pub total_experience: u64,
}

impl Default for PartyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{PokemonSpecies, Nature, Type};

    fn create_test_pokemon() -> Individual {
        let species = PokemonSpecies {
            id: PokemonId(25), // Pikachu
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
    fn test_party_creation() {
        let party = Party::new();
        assert!(party.is_empty());
        assert_eq!(party.count(), 0);
        assert_eq!(party.get_battle_ready_count(), 0);
    }

    #[test]
    fn test_add_pokemon_to_party() {
        let mut party = Party::new();
        let pokemon = create_test_pokemon();
        
        let slot = party.add_pokemon(pokemon).unwrap();
        assert_eq!(slot, 0);
        assert_eq!(party.count(), 1);
        assert_eq!(party.get_battle_ready_count(), 1);
        assert_eq!(party.active_pokemon, Some(0));
    }

    #[test]
    fn test_party_full_error() {
        let mut party = Party::new();
        
        // 添加满6只Pokemon
        for _ in 0..MAX_PARTY_SIZE {
            let pokemon = create_test_pokemon();
            party.add_pokemon(pokemon).unwrap();
        }
        
        // 尝试添加第7只Pokemon应该失败
        let pokemon = create_test_pokemon();
        let result = party.add_pokemon(pokemon);
        assert!(matches!(result, Err(PartyError::PartyFull)));
    }

    #[test]
    fn test_pokemon_healing() {
        let mut party_pokemon = PartyPokemon::new(create_test_pokemon(), 0);
        
        // 造成伤害
        party_pokemon.take_damage(20);
        let hp_before = party_pokemon.current_hp;
        
        // 治疗
        let healed = party_pokemon.heal(15);
        assert_eq!(healed, 15);
        assert_eq!(party_pokemon.current_hp, hp_before + 15);
    }

    #[test]
    fn test_experience_share() {
        let mut party = Party::new();
        
        for _ in 0..3 {
            let pokemon = create_test_pokemon();
            party.add_pokemon(pokemon).unwrap();
        }
        
        party.experience_share = true;
        party.apply_experience_share(300, &[0]);
        
        // 所有Pokemon应该获得100经验
        for i in 0..3 {
            if let Some(ref pokemon) = party.pokemon[i] {
                assert!(pokemon.pokemon.experience > 0);
            }
        }
    }

    #[test]
    fn test_party_validation() {
        let party = Party::new();
        let errors = party.validate_party();
        assert!(errors.contains(&PartyValidationError::EmptyParty));
        
        let mut party_with_pokemon = Party::new();
        let pokemon = create_test_pokemon();
        party_with_pokemon.add_pokemon(pokemon).unwrap();
        
        let errors = party_with_pokemon.validate_party();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_party_manager_formation_save_load() {
        let mut manager = PartyManager::new();
        let pokemon = create_test_pokemon();
        
        manager.current_party.add_pokemon(pokemon).unwrap();
        manager.save_formation("test_formation".to_string()).unwrap();
        
        // 清空当前队伍
        let mut empty_party = Party::new();
        std::mem::swap(&mut manager.current_party, &mut empty_party);
        
        // 加载队形
        manager.load_formation("test_formation").unwrap();
        assert_eq!(manager.current_party.count(), 1);
    }
}