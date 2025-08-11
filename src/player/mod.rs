// 玩家系统
// 开发心理：玩家是游戏核心实体，需要完整数据管理、状态同步、持久化存储
// 设计原则：数据完整性、状态一致性、性能优化、安全性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
#[cfg(feature = "pokemon-wip")]
use crate::pokemon::stats::PokemonStats;
#[cfg(feature = "pokemon-wip")]
use crate::pokemon::types::DualType;

// 临时类型定义，直到pokemon模块可用
#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonStats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub special_attack: u32,
    pub special_defense: u32,
    pub speed: u32,
}

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualType {
    pub primary: u32,
    pub secondary: Option<u32>,
}

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nature {
    pub id: u32,
    pub name: String,
}

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualValues {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortValues {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}
use glam::Vec2;

pub mod inventory;
pub mod profile;
pub mod progress;

// 玩家ID类型
pub type PlayerId = u64;

// 玩家状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerStatus {
    Active,         // 活跃
    Away,           // 离开
    Offline,        // 离线
    Banned,         // 封禁
}

// 玩家等级信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLevel {
    pub level: u32,
    pub experience: u64,
    pub experience_to_next: u64,
    pub total_experience: u64,
}

// 玩家位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLocation {
    pub map_id: String,
    pub position: Vec2,
    pub facing_direction: Vec2,
    pub last_updated: std::time::SystemTime,
}

// 玩家Pokemon队伍
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonTeam {
    pub active_team: Vec<u64>,      // 战斗队伍 (最多6只)
    pub storage: HashMap<u64, PokemonInstance>, // 存储系统中的Pokemon
    pub next_pokemon_id: u64,
}

// Pokemon实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonInstance {
    pub id: u64,
    pub species_id: u32,
    pub nickname: Option<String>,
    pub level: u8,
    pub experience: u32,
    pub stats: PokemonStats,
    pub types: DualType,
    pub moves: Vec<u32>,
    pub ability: u32,
    #[cfg(feature = "pokemon-wip")]
    pub nature: crate::pokemon::stats::Nature,
    #[cfg(not(feature = "pokemon-wip"))]
    pub nature: Nature,
    #[cfg(feature = "pokemon-wip")]
    pub individual_values: crate::pokemon::stats::IndividualValues,
    #[cfg(not(feature = "pokemon-wip"))]
    pub individual_values: IndividualValues,
    #[cfg(feature = "pokemon-wip")]
    pub effort_values: crate::pokemon::stats::EffortValues,
    #[cfg(not(feature = "pokemon-wip"))]
    pub effort_values: EffortValues,
    pub friendship: u8,
    pub original_trainer: String,
    pub catch_date: std::time::SystemTime,
    pub pokeball_type: u32,
    pub status_condition: Option<u32>,
    pub held_item: Option<u32>,
    pub is_shiny: bool,
}

// 玩家统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub pokemon_caught: u32,
    pub pokemon_seen: u32,
    pub battles_won: u32,
    pub battles_lost: u32,
    pub distance_walked: f64,        // 米
    pub playtime: u64,               // 秒
    pub items_used: u32,
    pub pokemon_evolved: u32,
    pub trades_completed: u32,
    pub gyms_defeated: u32,
}

// 玩家数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    // 基本信息
    pub id: PlayerId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    pub status: PlayerStatus,
    
    // 等级系统
    pub level_info: PlayerLevel,
    
    // 位置信息
    pub location: PlayerLocation,
    
    // Pokemon相关
    pub pokemon_team: PokemonTeam,
    pub pokedex: HashMap<u32, PokedexEntry>, // species_id -> entry
    
    // 背包系统
    pub inventory: inventory::Inventory,
    
    // 游戏进度
    pub progress: progress::GameProgress,
    
    // 统计信息
    pub stats: PlayerStats,
    
    // 设置
    pub settings: HashMap<String, String>,
    
    // 时间戳
    pub created_at: std::time::SystemTime,
    pub last_login: std::time::SystemTime,
    pub last_save: std::time::SystemTime,
}

// 图鉴条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokedexEntry {
    pub species_id: u32,
    pub seen: bool,
    pub caught: bool,
    pub first_seen_date: Option<std::time::SystemTime>,
    pub first_caught_date: Option<std::time::SystemTime>,
    pub times_encountered: u32,
    pub times_caught: u32,
}

// 玩家管理器
pub struct PlayerManager {
    current_player: Option<Player>,
    player_cache: HashMap<PlayerId, Player>,
    save_timer: f32,
    auto_save_interval: f32,
    
    // 统计
    total_saves: u64,
    last_save_time: std::time::Instant,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            current_player: None,
            player_cache: HashMap::new(),
            save_timer: 0.0,
            auto_save_interval: 300.0, // 5分钟自动保存
            total_saves: 0,
            last_save_time: std::time::Instant::now(),
        }
    }
    
    // 创建新玩家
    pub fn create_player(
        &mut self,
        username: String,
        display_name: String,
    ) -> Result<PlayerId, GameError> {
        let player_id = self.generate_player_id();
        
        let player = Player {
            id: player_id,
            username: username.clone(),
            display_name,
            avatar: "default".to_string(),
            status: PlayerStatus::Active,
            level_info: PlayerLevel {
                level: 1,
                experience: 0,
                experience_to_next: 1000,
                total_experience: 0,
            },
            location: PlayerLocation {
                map_id: "starting_town".to_string(),
                position: Vec2::new(100.0, 100.0),
                facing_direction: Vec2::new(0.0, -1.0),
                last_updated: std::time::SystemTime::now(),
            },
            pokemon_team: PokemonTeam {
                active_team: Vec::new(),
                storage: HashMap::new(),
                next_pokemon_id: 1,
            },
            pokedex: HashMap::new(),
            inventory: inventory::Inventory::new(),
            progress: progress::GameProgress::new(),
            stats: PlayerStats {
                pokemon_caught: 0,
                pokemon_seen: 0,
                battles_won: 0,
                battles_lost: 0,
                distance_walked: 0.0,
                playtime: 0,
                items_used: 0,
                pokemon_evolved: 0,
                trades_completed: 0,
                gyms_defeated: 0,
            },
            settings: HashMap::new(),
            created_at: std::time::SystemTime::now(),
            last_login: std::time::SystemTime::now(),
            last_save: std::time::SystemTime::now(),
        };
        
        self.current_player = Some(player.clone());
        self.player_cache.insert(player_id, player);
        
        debug!("创建新玩家: {} (ID: {})", username, player_id);
        Ok(player_id)
    }
    
    // 加载玩家
    pub fn load_player(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        // 尝试从缓存加载
        if let Some(player) = self.player_cache.get(&player_id).cloned() {
            self.current_player = Some(player);
            return Ok(());
        }
        
        // 从文件系统加载
        match self.load_player_from_file(player_id) {
            Ok(player) => {
                self.current_player = Some(player.clone());
                self.player_cache.insert(player_id, player);
                debug!("加载玩家: ID {}", player_id);
                Ok(())
            },
            Err(e) => {
                error!("加载玩家失败: {}", e);
                Err(e)
            }
        }
    }
    
    // 保存当前玩家
    pub fn save_current_player(&mut self) -> Result<(), GameError> {
        if let Some(ref mut player) = self.current_player {
            player.last_save = std::time::SystemTime::now();
            self.save_player_to_file(player)?;
            self.total_saves += 1;
            self.last_save_time = std::time::Instant::now();
            debug!("保存玩家数据: {}", player.username);
        }
        Ok(())
    }
    
    // 获取当前玩家
    pub fn get_current_player(&self) -> Option<&Player> {
        self.current_player.as_ref()
    }
    
    // 获取当前玩家(可变)
    pub fn get_current_player_mut(&mut self) -> Option<&mut Player> {
        self.current_player.as_mut()
    }
    
    // 添加Pokemon到队伍
    pub fn add_pokemon_to_team(&mut self, pokemon: PokemonInstance) -> Result<u64, GameError> {
        if let Some(ref mut player) = self.current_player {
            let pokemon_id = pokemon.id;
            
            // 添加到存储
            player.pokemon_team.storage.insert(pokemon_id, pokemon);
            
            // 如果队伍未满，添加到战斗队伍
            if player.pokemon_team.active_team.len() < 6 {
                player.pokemon_team.active_team.push(pokemon_id);
            }
            
            player.stats.pokemon_caught += 1;
            debug!("添加Pokemon到队伍: ID {}", pokemon_id);
            Ok(pokemon_id)
        } else {
            Err(GameError::Player("没有当前玩家".to_string()))
        }
    }
    
    // 获得经验值
    pub fn gain_experience(&mut self, amount: u64) -> Result<Vec<u32>, GameError> {
        if let Some(ref mut player) = self.current_player {
            player.level_info.experience += amount;
            player.level_info.total_experience += amount;
            
            let mut levels_gained = Vec::new();
            
            // 检查升级
            while player.level_info.experience >= player.level_info.experience_to_next {
                player.level_info.experience -= player.level_info.experience_to_next;
                player.level_info.level += 1;
                levels_gained.push(player.level_info.level);
                
                // 计算下一级所需经验
                player.level_info.experience_to_next = self.calculate_experience_to_next_level(player.level_info.level);
                
                debug!("玩家升级到 {} 级!", player.level_info.level);
            }
            
            Ok(levels_gained)
        } else {
            Err(GameError::Player("没有当前玩家".to_string()))
        }
    }
    
    // 更新玩家位置
    pub fn update_location(&mut self, map_id: String, position: Vec2) -> Result<(), GameError> {
        if let Some(ref mut player) = self.current_player {
            // 计算移动距离
            let distance = (position - player.location.position).length();
            player.stats.distance_walked += distance as f64;
            
            player.location.map_id = map_id;
            player.location.position = position;
            player.location.last_updated = std::time::SystemTime::now();
            
            Ok(())
        } else {
            Err(GameError::Player("没有当前玩家".to_string()))
        }
    }
    
    // 更新Pokedex
    pub fn update_pokedex(&mut self, species_id: u32, seen: bool, caught: bool) -> Result<(), GameError> {
        if let Some(ref mut player) = self.current_player {
            let entry = player.pokedex.entry(species_id).or_insert(PokedexEntry {
                species_id,
                seen: false,
                caught: false,
                first_seen_date: None,
                first_caught_date: None,
                times_encountered: 0,
                times_caught: 0,
            });
            
            if seen && !entry.seen {
                entry.seen = true;
                entry.first_seen_date = Some(std::time::SystemTime::now());
                player.stats.pokemon_seen += 1;
                debug!("首次发现Pokemon: species_id {}", species_id);
            }
            
            if caught {
                entry.times_caught += 1;
                if !entry.caught {
                    entry.caught = true;
                    entry.first_caught_date = Some(std::time::SystemTime::now());
                    debug!("首次捕获Pokemon: species_id {}", species_id);
                }
            }
            
            if seen {
                entry.times_encountered += 1;
            }
            
            Ok(())
        } else {
            Err(GameError::Player("没有当前玩家".to_string()))
        }
    }
    
    // 更新游戏时间
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        if let Some(ref mut player) = self.current_player {
            player.stats.playtime += delta_time as u64;
        }
        
        // 自动保存检查
        self.save_timer += delta_time;
        if self.save_timer >= self.auto_save_interval {
            self.save_current_player()?;
            self.save_timer = 0.0;
        }
        
        Ok(())
    }
    
    // 私有方法
    fn generate_player_id(&self) -> PlayerId {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // 简单的ID生成，实际可能需要更复杂的算法
        timestamp + fastrand::u64(0..1000)
    }
    
    fn calculate_experience_to_next_level(&self, level: u32) -> u64 {
        // 简化的经验公式
        (level as u64 * 1000) + (level as u64 * level as u64 * 50)
    }
    
    fn load_player_from_file(&self, player_id: PlayerId) -> Result<Player, GameError> {
        let filename = format!("saves/player_{}.json", player_id);
        
        match std::fs::read_to_string(&filename) {
            Ok(data) => {
                match serde_json::from_str::<Player>(&data) {
                    Ok(mut player) => {
                        player.last_login = std::time::SystemTime::now();
                        Ok(player)
                    },
                    Err(e) => Err(GameError::Player(format!("反序列化失败: {}", e))),
                }
            },
            Err(e) => Err(GameError::Player(format!("读取文件失败: {}", e))),
        }
    }
    
    fn save_player_to_file(&self, player: &Player) -> Result<(), GameError> {
        // 确保保存目录存在
        std::fs::create_dir_all("saves").ok();
        
        let filename = format!("saves/player_{}.json", player.id);
        
        match serde_json::to_string_pretty(player) {
            Ok(data) => {
                match std::fs::write(&filename, data) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(GameError::Player(format!("写入文件失败: {}", e))),
                }
            },
            Err(e) => Err(GameError::Player(format!("序列化失败: {}", e))),
        }
    }
}

impl Player {
    // 获取战斗队伍中的Pokemon
    pub fn get_active_pokemon(&self) -> Vec<&PokemonInstance> {
        self.pokemon_team
            .active_team
            .iter()
            .filter_map(|&id| self.pokemon_team.storage.get(&id))
            .collect()
    }
    
    // 检查Pokemon是否在战斗队伍中
    pub fn is_pokemon_in_active_team(&self, pokemon_id: u64) -> bool {
        self.pokemon_team.active_team.contains(&pokemon_id)
    }
    
    // 交换Pokemon在队伍中的位置
    pub fn swap_pokemon_in_team(&mut self, index1: usize, index2: usize) -> Result<(), GameError> {
        if index1 < self.pokemon_team.active_team.len() && 
           index2 < self.pokemon_team.active_team.len() {
            self.pokemon_team.active_team.swap(index1, index2);
            Ok(())
        } else {
            Err(GameError::Player("索引超出范围".to_string()))
        }
    }
    
    // 计算图鉴完成度
    pub fn calculate_pokedex_completion(&self) -> (u32, u32, f32) {
        let total_species = 151; // 简化为第一代Pokemon数量
        let seen = self.pokedex.values().filter(|e| e.seen).count() as u32;
        let caught = self.pokedex.values().filter(|e| e.caught).count() as u32;
        let completion_rate = (caught as f32 / total_species as f32) * 100.0;
        
        (caught, total_species, completion_rate)
    }
    
    // 获取设置值
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
    
    // 设置设置值
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            pokemon_caught: 0,
            pokemon_seen: 0,
            battles_won: 0,
            battles_lost: 0,
            distance_walked: 0.0,
            playtime: 0,
            items_used: 0,
            pokemon_evolved: 0,
            trades_completed: 0,
            gyms_defeated: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_player_manager_creation() {
        let manager = PlayerManager::new();
        assert!(manager.current_player.is_none());
        assert_eq!(manager.player_cache.len(), 0);
    }
    
    #[test]
    fn test_player_creation() {
        let mut manager = PlayerManager::new();
        
        let player_id = manager.create_player(
            "testuser".to_string(),
            "Test User".to_string(),
        ).unwrap();
        
        assert!(player_id > 0);
        assert!(manager.current_player.is_some());
        
        let player = manager.get_current_player().unwrap();
        assert_eq!(player.username, "testuser");
        assert_eq!(player.display_name, "Test User");
        assert_eq!(player.level_info.level, 1);
    }
    
    #[test]
    fn test_experience_gain() {
        let mut manager = PlayerManager::new();
        manager.create_player("test".to_string(), "Test".to_string()).unwrap();
        
        let levels_gained = manager.gain_experience(1500).unwrap();
        
        let player = manager.get_current_player().unwrap();
        assert_eq!(player.level_info.level, 2);
        assert_eq!(levels_gained, vec![2]);
        assert_eq!(player.level_info.total_experience, 1500);
    }
    
    #[test]
    fn test_pokedex_update() {
        let mut manager = PlayerManager::new();
        manager.create_player("test".to_string(), "Test".to_string()).unwrap();
        
        // 首次发现
        manager.update_pokedex(1, true, false).unwrap();
        let player = manager.get_current_player().unwrap();
        assert_eq!(player.stats.pokemon_seen, 1);
        
        // 首次捕获
        manager.update_pokedex(1, true, true).unwrap();
        let player = manager.get_current_player().unwrap();
        assert_eq!(player.stats.pokemon_caught, 1);
        
        let entry = player.pokedex.get(&1).unwrap();
        assert!(entry.seen);
        assert!(entry.caught);
        assert_eq!(entry.times_encountered, 2);
        assert_eq!(entry.times_caught, 1);
    }
}