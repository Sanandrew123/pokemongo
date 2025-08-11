// 存档系统 - 游戏进度保存和加载
// 开发心理：存档是玩家投入时间的保障，需要可靠的持久化和版本兼容性
// 设计原则：数据完整性、向后兼容、多存档支持、云同步准备

use crate::core::{GameError, Result};
use crate::player::Player;
use crate::game_modes::{GameMode, GameState};
#[cfg(feature = "pokemon-wip")]
use crate::pokemon::Pokemon;

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pokemon {
    pub id: u64,
    pub species_id: u32,
    pub level: u8,
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::{info, debug, warn, error};

// 存档版本
pub const SAVE_VERSION: u32 = 1;

// 游戏存档数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSave {
    pub version: u32,
    pub created_at: u64,
    pub last_saved: u64,
    pub save_count: u32,
    pub playtime: Duration,
    
    // 玩家数据
    pub player: Player,
    
    // 游戏状态
    pub current_mode: GameMode,
    pub current_state: GameState,
    
    // 世界状态
    pub world_data: WorldSaveData,
    
    // 游戏设置
    pub game_settings: GameSettings,
    
    // 校验和
    pub checksum: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveData {
    // 地图状态
    pub visited_areas: Vec<String>,
    pub unlocked_areas: Vec<String>,
    pub current_map: String,
    pub player_position: (f32, f32, f32),
    
    // NPC状态
    pub npc_interactions: HashMap<String, NpcSaveData>,
    
    // 物品状态
    pub collected_items: Vec<CollectedItem>,
    pub hidden_items: Vec<HiddenItem>,
    
    // 事件状态
    pub triggered_events: Vec<String>,
    pub event_flags: HashMap<String, bool>,
    pub event_variables: HashMap<String, i32>,
    
    // 时间状态
    pub game_time: GameTime,
    pub weather_state: WeatherState,
    
    // 特殊状态
    pub legendary_encounters: HashMap<u32, bool>,
    pub fossil_revivals: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSaveData {
    pub npc_id: String,
    pub last_interaction: u64,
    pub dialogue_progress: u32,
    pub has_battled: bool,
    pub custom_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedItem {
    pub item_id: u32,
    pub location: String,
    pub collected_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenItem {
    pub item_id: u32,
    pub location: (f32, f32),
    pub map_id: u32,
    pub respawn_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub season: Season,
    pub total_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub current_weather: WeatherType,
    pub duration_remaining: Duration,
    pub region_weather: HashMap<String, WeatherType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Rain,
    Snow,
    Fog,
    Sandstorm,
    Sunny,
    Cloudy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub difficulty: Difficulty,
    pub battle_style: BattleStyle,
    pub text_speed: TextSpeed,
    pub sound_effects: bool,
    pub music: bool,
    pub auto_run: bool,
    pub unit_system: UnitSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStyle {
    Switch,  // 切换模式
    Set,     // 设定模式
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextSpeed {
    Slow,
    Normal,
    Fast,
    Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitSystem {
    Metric,    // 公制
    Imperial,  // 英制
}

// 存档管理器
pub struct SaveManager {
    save_directory: PathBuf,
    current_save: Option<GameSave>,
    backup_count: usize,
    auto_save_interval: Duration,
    last_auto_save: SystemTime,
}

impl SaveManager {
    pub fn new<P: AsRef<Path>>(save_directory: P) -> Result<Self> {
        let save_dir = save_directory.as_ref().to_path_buf();
        
        // 创建存档目录
        if !save_dir.exists() {
            create_dir_all(&save_dir)?;
            info!("创建存档目录: {:?}", save_dir);
        }
        
        Ok(Self {
            save_directory: save_dir,
            current_save: None,
            backup_count: 5,
            auto_save_interval: Duration::from_secs(300), // 5分钟
            last_auto_save: SystemTime::now(),
        })
    }
    
    // 创建新存档
    pub fn create_new_save(&mut self, player: Player) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let save = GameSave {
            version: SAVE_VERSION,
            created_at: now,
            last_saved: now,
            save_count: 0,
            playtime: Duration::ZERO,
            
            player,
            
            current_mode: GameMode::MainStory,
            current_state: GameState::InGame,
            
            world_data: WorldSaveData::default(),
            game_settings: GameSettings::default(),
            
            checksum: 0,
        };
        
        self.current_save = Some(save);
        info!("创建新存档");
        Ok(())
    }
    
    // 保存游戏
    pub fn save_game(&mut self, slot: u8) -> Result<()> {
        let save = self.current_save.as_mut()
            .ok_or_else(|| GameError::SaveError("没有当前存档".to_string()))?;
        
        // 更新保存信息
        save.last_saved = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        save.save_count += 1;
        
        // 计算校验和
        save.checksum = self.calculate_checksum(save);
        
        // 创建备份
        self.create_backup(slot)?;
        
        // 保存到文件
        let save_path = self.get_save_path(slot);
        self.write_save_file(&save_path, save)?;
        
        info!("游戏已保存到存档槽 {}", slot);
        Ok(())
    }
    
    // 加载游戏
    pub fn load_game(&mut self, slot: u8) -> Result<()> {
        let save_path = self.get_save_path(slot);
        
        if !save_path.exists() {
            return Err(GameError::SaveError(format!("存档槽 {} 不存在", slot)));
        }
        
        let save = self.read_save_file(&save_path)?;
        
        // 验证存档
        self.validate_save(&save)?;
        
        self.current_save = Some(save);
        info!("从存档槽 {} 加载游戏", slot);
        Ok(())
    }
    
    // 删除存档
    pub fn delete_save(&self, slot: u8) -> Result<()> {
        let save_path = self.get_save_path(slot);
        
        if save_path.exists() {
            std::fs::remove_file(save_path)?;
            info!("删除存档槽 {}", slot);
        }
        
        Ok(())
    }
    
    // 检查存档是否存在
    pub fn save_exists(&self, slot: u8) -> bool {
        self.get_save_path(slot).exists()
    }
    
    // 获取存档信息
    pub fn get_save_info(&self, slot: u8) -> Result<Option<SaveInfo>> {
        let save_path = self.get_save_path(slot);
        
        if !save_path.exists() {
            return Ok(None);
        }
        
        let save = self.read_save_file(&save_path)?;
        
        Ok(Some(SaveInfo {
            slot,
            player_name: save.player.display_name.clone(),
            player_level: save.player.level_info.level,
            playtime: save.playtime,
            last_saved: save.last_saved,
            badges: save.player.progress.badges.len() as u8,
            pokedex_count: save.player.pokedex.len() as u16,
            location: save.player.location.map_id.clone(),
        }))
    }
    
    // 自动保存检查
    pub fn check_auto_save(&mut self, current_slot: u8) -> Result<bool> {
        if self.last_auto_save.elapsed().unwrap_or(Duration::ZERO) >= self.auto_save_interval {
            if self.current_save.is_some() {
                self.save_game(current_slot)?;
                self.last_auto_save = SystemTime::now();
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    // 快速保存
    pub fn quick_save(&mut self) -> Result<()> {
        self.save_game(0) // 使用槽位0作为快速保存
    }
    
    // 快速加载
    pub fn quick_load(&mut self) -> Result<()> {
        self.load_game(0)
    }
    
    // 导出存档
    pub fn export_save(&self, slot: u8, export_path: &Path) -> Result<()> {
        let save_path = self.get_save_path(slot);
        std::fs::copy(save_path, export_path)?;
        info!("存档已导出到: {:?}", export_path);
        Ok(())
    }
    
    // 导入存档
    pub fn import_save(&mut self, import_path: &Path, slot: u8) -> Result<()> {
        // 验证导入的存档
        let save = self.read_save_file(import_path)?;
        self.validate_save(&save)?;
        
        let save_path = self.get_save_path(slot);
        std::fs::copy(import_path, save_path)?;
        info!("存档已导入到槽位 {}", slot);
        Ok(())
    }
    
    // 获取当前存档
    pub fn get_current_save(&self) -> Option<&GameSave> {
        self.current_save.as_ref()
    }
    
    // 获取当前存档（可变）
    pub fn get_current_save_mut(&mut self) -> Option<&mut GameSave> {
        self.current_save.as_mut()
    }
    
    // 更新游戏时间
    pub fn update_playtime(&mut self, delta: Duration) {
        if let Some(save) = &mut self.current_save {
            save.playtime += delta;
            save.player.stats.playtime += delta;
        }
    }
    
    // 辅助方法
    fn get_save_path(&self, slot: u8) -> PathBuf {
        self.save_directory.join(format!("save_{:02}.dat", slot))
    }
    
    fn get_backup_path(&self, slot: u8, backup_index: usize) -> PathBuf {
        self.save_directory.join(format!("save_{:02}_backup_{}.dat", slot, backup_index))
    }
    
    fn write_save_file(&self, path: &Path, save: &GameSave) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        // 使用 bincode 进行序列化
        let encoded = bincode::serialize(save)
            .map_err(|e| GameError::SaveError(format!("序列化失败: {}", e)))?;
        
        writer.write_all(&encoded)?;
        writer.flush()?;
        
        Ok(())
    }
    
    fn read_save_file(&self, path: &Path) -> Result<GameSave> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // 读取所有数据
        let mut buffer = Vec::new();
        use std::io::Read;
        let mut reader = reader;
        reader.read_to_end(&mut buffer)?;
        
        // 使用 bincode 进行反序列化
        let save = bincode::deserialize(&buffer)
            .map_err(|e| GameError::SaveError(format!("反序列化失败: {}", e)))?;
        
        Ok(save)
    }
    
    fn validate_save(&self, save: &GameSave) -> Result<()> {
        // 版本检查
        if save.version > SAVE_VERSION {
            return Err(GameError::SaveError(
                "存档版本过高，请更新游戏".to_string()
            ));
        }
        
        // 校验和检查
        let calculated_checksum = self.calculate_checksum(save);
        if calculated_checksum != save.checksum {
            warn!("存档校验和不匹配，可能已损坏");
            // 不直接报错，给用户选择是否继续加载
        }
        
        // 基本数据完整性检查
        if save.player.display_name.is_empty() {
            return Err(GameError::SaveError("玩家名称为空".to_string()));
        }
        
        if save.player.pokemon_team.party.is_empty() {
            warn!("玩家队伍为空");
        }
        
        Ok(())
    }
    
    fn calculate_checksum(&self, save: &GameSave) -> u64 {
        // 简单的校验和计算
        let mut checksum = 0u64;
        
        checksum = checksum.wrapping_add(save.version as u64);
        checksum = checksum.wrapping_add(save.created_at);
        checksum = checksum.wrapping_add(save.player.id);
        
        // 可以添加更多字段用于校验和计算
        
        checksum
    }
    
    fn create_backup(&self, slot: u8) -> Result<()> {
        let save_path = self.get_save_path(slot);
        
        if save_path.exists() {
            // 轮转备份文件
            for i in (1..self.backup_count).rev() {
                let old_backup = self.get_backup_path(slot, i - 1);
                let new_backup = self.get_backup_path(slot, i);
                
                if old_backup.exists() {
                    if new_backup.exists() {
                        std::fs::remove_file(&new_backup)?;
                    }
                    std::fs::rename(old_backup, new_backup)?;
                }
            }
            
            // 创建新的备份
            let backup_path = self.get_backup_path(slot, 0);
            std::fs::copy(&save_path, backup_path)?;
        }
        
        Ok(())
    }
}

// 存档信息
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub slot: u8,
    pub player_name: String,
    pub player_level: u32,
    pub playtime: Duration,
    pub last_saved: u64,
    pub badges: u8,
    pub pokedex_count: u16,
    pub location: String,
}

// 默认实现
impl Default for WorldSaveData {
    fn default() -> Self {
        Self {
            visited_areas: vec!["真新镇".to_string()],
            unlocked_areas: vec!["真新镇".to_string()],
            current_map: "真新镇".to_string(),
            player_position: (0.0, 0.0, 0.0),
            
            npc_interactions: HashMap::new(),
            
            collected_items: Vec::new(),
            hidden_items: Vec::new(),
            
            triggered_events: Vec::new(),
            event_flags: HashMap::new(),
            event_variables: HashMap::new(),
            
            game_time: GameTime::default(),
            weather_state: WeatherState::default(),
            
            legendary_encounters: HashMap::new(),
            fossil_revivals: Vec::new(),
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            day: 1,
            hour: 12,
            minute: 0,
            season: Season::Spring,
            total_seconds: 0,
        }
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            duration_remaining: Duration::from_secs(3600),
            region_weather: HashMap::new(),
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            battle_style: BattleStyle::Switch,
            text_speed: TextSpeed::Normal,
            sound_effects: true,
            music: true,
            auto_run: false,
            unit_system: UnitSystem::Metric,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Player, PlayerGender};
    use tempfile::TempDir;
    
    #[test]
    fn test_save_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path());
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_create_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SaveManager::new(temp_dir.path()).unwrap();
        
        let player = Player::new("测试玩家".to_string(), PlayerGender::Male);
        assert!(manager.create_new_save(player).is_ok());
        assert!(manager.save_game(1).is_ok());
        assert!(manager.save_exists(1));
    }
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SaveManager::new(temp_dir.path()).unwrap();
        
        let player = Player::new("测试玩家".to_string(), PlayerGender::Male);
        manager.create_new_save(player).unwrap();
        manager.save_game(1).unwrap();
        
        // 清空当前存档
        manager.current_save = None;
        
        // 重新加载
        assert!(manager.load_game(1).is_ok());
        assert!(manager.get_current_save().is_some());
        assert_eq!(manager.get_current_save().unwrap().player.name, "测试玩家");
    }
    
    #[test]
    fn test_save_info() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SaveManager::new(temp_dir.path()).unwrap();
        
        let player = Player::new("信息测试".to_string(), PlayerGender::Female);
        manager.create_new_save(player).unwrap();
        manager.save_game(2).unwrap();
        
        let info = manager.get_save_info(2).unwrap();
        assert!(info.is_some());
        
        let info = info.unwrap();
        assert_eq!(info.slot, 2);
        assert_eq!(info.player_name, "信息测试");
    }
}