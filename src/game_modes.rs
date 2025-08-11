// 游戏模式系统 - 管理不同的游戏玩法模式
// 开发心理：宝可梦游戏有多种玩法模式，需要统一的状态管理和模式切换机制
// 设计原则：状态机驱动、可扩展的模式系统、流畅的模式切换

use crate::core::{GameError, Result};
#[cfg(feature = "pokemon-wip")]
use crate::pokemon::{Pokemon, PokemonSpecies, SpeciesId};

// 临时类型定义，直到pokemon模块可用
#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone)]
pub struct Pokemon {
    pub id: u64,
    pub species_id: u32,
    pub level: u8,
}

#[cfg(not(feature = "pokemon-wip"))]
#[derive(Debug, Clone)]
pub struct PokemonSpecies {
    pub id: u32,
    pub name: String,
}

#[cfg(not(feature = "pokemon-wip"))]
pub type SpeciesId = u32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};

// 游戏模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    MainStory,      // 主线剧情
    FreeRoam,       // 自由探索
    Battle,         // 战斗模式
    Training,       // 训练模式
    Breeding,       // 繁育模式
    Contest,        // 华丽大赛
    BattleTower,    // 对战塔
    Safari,         // 狩猎区
    Gym,            // 道馆挑战
    EliteFour,      // 四天王
    Champion,       // 冠军赛
    Tournament,     // 锦标赛
    Online,         // 在线模式
    Debug,          // 调试模式
}

// 游戏状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Loading,
    MainMenu,
    InGame,
    Paused,
    BattleTransition,
    Dialogue,
    Inventory,
    Settings,
    Saving,
}

// 游戏模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub mode: GameMode,
    pub allow_save: bool,
    pub allow_pause: bool,
    pub time_limit: Option<Duration>,
    pub entry_requirements: Vec<Requirement>,
    pub rewards: Vec<Reward>,
    pub difficulty_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    MinLevel(u8),
    BadgeCount(u8),
    CompletedStory(String),
    PokemonCount(usize),
    ItemPossession(u32, u32), // item_id, count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reward {
    Experience(u32),
    Money(u32),
    Item(u32, u32), // item_id, count
    Pokemon(SpeciesId, u8), // species_id, level
    Badge(String),
    Title(String),
}

// 游戏模式管理器
pub struct GameModeManager {
    current_mode: GameMode,
    current_state: GameState,
    mode_configs: HashMap<GameMode, ModeConfig>,
    mode_start_time: Instant,
    session_stats: SessionStats,
    transition_stack: Vec<GameMode>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub mode_play_time: HashMap<GameMode, Duration>,
    pub battles_won: u32,
    pub battles_lost: u32,
    pub pokemon_caught: u32,
    pub steps_taken: u32,
    pub items_used: u32,
    pub story_progress: f32,
    pub achievements_unlocked: Vec<String>,
}

impl GameModeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: GameMode::MainStory,
            current_state: GameState::MainMenu,
            mode_configs: HashMap::new(),
            mode_start_time: Instant::now(),
            session_stats: SessionStats::default(),
            transition_stack: Vec::new(),
        };
        
        manager.initialize_default_configs();
        manager
    }
    
    // 初始化默认配置
    fn initialize_default_configs(&mut self) {
        // 主线剧情模式
        self.mode_configs.insert(GameMode::MainStory, ModeConfig {
            mode: GameMode::MainStory,
            allow_save: true,
            allow_pause: true,
            time_limit: None,
            entry_requirements: vec![],
            rewards: vec![],
            difficulty_scaling: true,
        });
        
        // 自由探索模式
        self.mode_configs.insert(GameMode::FreeRoam, ModeConfig {
            mode: GameMode::FreeRoam,
            allow_save: true,
            allow_pause: true,
            time_limit: None,
            entry_requirements: vec![],
            rewards: vec![],
            difficulty_scaling: false,
        });
        
        // 战斗模式
        self.mode_configs.insert(GameMode::Battle, ModeConfig {
            mode: GameMode::Battle,
            allow_save: false,
            allow_pause: true,
            time_limit: Some(Duration::from_secs(1800)), // 30分钟
            entry_requirements: vec![Requirement::PokemonCount(1)],
            rewards: vec![Reward::Experience(100)],
            difficulty_scaling: true,
        });
        
        // 训练模式
        self.mode_configs.insert(GameMode::Training, ModeConfig {
            mode: GameMode::Training,
            allow_save: true,
            allow_pause: true,
            time_limit: None,
            entry_requirements: vec![],
            rewards: vec![],
            difficulty_scaling: false,
        });
        
        // 对战塔
        self.mode_configs.insert(GameMode::BattleTower, ModeConfig {
            mode: GameMode::BattleTower,
            allow_save: false,
            allow_pause: false,
            time_limit: Some(Duration::from_secs(3600)), // 1小时
            entry_requirements: vec![
                Requirement::MinLevel(50),
                Requirement::PokemonCount(6),
            ],
            rewards: vec![
                Reward::Money(10000),
                Reward::Item(1, 1), // 大师球
            ],
            difficulty_scaling: true,
        });
        
        info!("游戏模式配置初始化完成");
    }
    
    // 切换游戏模式
    pub fn switch_mode(&mut self, new_mode: GameMode) -> Result<()> {
        // 检查切换条件
        self.validate_mode_switch(new_mode)?;
        
        // 保存当前模式的游戏时间
        let elapsed = self.mode_start_time.elapsed();
        self.session_stats.mode_play_time
            .entry(self.current_mode)
            .and_modify(|t| *t += elapsed)
            .or_insert(elapsed);
        
        let old_mode = self.current_mode;
        self.current_mode = new_mode;
        self.mode_start_time = Instant::now();
        
        info!("游戏模式切换: {:?} -> {:?}", old_mode, new_mode);
        
        // 执行模式切换逻辑
        self.on_mode_enter(new_mode)?;
        
        Ok(())
    }
    
    // 验证模式切换
    fn validate_mode_switch(&self, new_mode: GameMode) -> Result<()> {
        if let Some(config) = self.mode_configs.get(&new_mode) {
            // 检查进入要求
            for requirement in &config.entry_requirements {
                if !self.check_requirement(requirement) {
                    return Err(GameError::GameModeError(
                        format!("不满足模式 {:?} 的进入要求: {:?}", new_mode, requirement)
                    ));
                }
            }
            
            // 检查当前状态是否允许切换
            if self.current_state == GameState::Saving {
                return Err(GameError::GameModeError("保存中无法切换模式".to_string()));
            }
            
            Ok(())
        } else {
            Err(GameError::GameModeError(format!("未知的游戏模式: {:?}", new_mode)))
        }
    }
    
    // 检查要求
    fn check_requirement(&self, requirement: &Requirement) -> bool {
        match requirement {
            Requirement::MinLevel(level) => {
                // TODO: 检查玩家队伍最高等级
                true
            },
            Requirement::BadgeCount(count) => {
                // TODO: 检查徽章数量
                true
            },
            Requirement::CompletedStory(story) => {
                // TODO: 检查剧情完成度
                true
            },
            Requirement::PokemonCount(count) => {
                // TODO: 检查宝可梦数量
                *count <= 6 // 简单检查
            },
            Requirement::ItemPossession(item_id, count) => {
                // TODO: 检查道具拥有量
                true
            },
        }
    }
    
    // 模式进入处理
    fn on_mode_enter(&mut self, mode: GameMode) -> Result<()> {
        match mode {
            GameMode::Battle => {
                self.current_state = GameState::BattleTransition;
                debug!("进入战斗模式");
            },
            GameMode::MainStory => {
                self.current_state = GameState::InGame;
                debug!("进入主线剧情模式");
            },
            GameMode::FreeRoam => {
                self.current_state = GameState::InGame;
                debug!("进入自由探索模式");
            },
            GameMode::Training => {
                self.current_state = GameState::InGame;
                debug!("进入训练模式");
            },
            GameMode::BattleTower => {
                self.current_state = GameState::BattleTransition;
                debug!("进入对战塔模式");
            },
            _ => {
                self.current_state = GameState::InGame;
                debug!("进入模式: {:?}", mode);
            }
        }
        
        Ok(())
    }
    
    // 推入临时模式
    pub fn push_temporary_mode(&mut self, temp_mode: GameMode) -> Result<()> {
        self.transition_stack.push(self.current_mode);
        self.switch_mode(temp_mode)
    }
    
    // 返回上一个模式
    pub fn pop_mode(&mut self) -> Result<()> {
        if let Some(previous_mode) = self.transition_stack.pop() {
            self.switch_mode(previous_mode)
        } else {
            Err(GameError::GameModeError("没有可返回的模式".to_string()))
        }
    }
    
    // 更新游戏状态
    pub fn set_state(&mut self, new_state: GameState) {
        if self.current_state != new_state {
            debug!("游戏状态切换: {:?} -> {:?}", self.current_state, new_state);
            self.current_state = new_state;
        }
    }
    
    // 获取当前模式
    pub fn current_mode(&self) -> GameMode {
        self.current_mode
    }
    
    // 获取当前状态
    pub fn current_state(&self) -> GameState {
        self.current_state
    }
    
    // 获取模式配置
    pub fn get_config(&self, mode: GameMode) -> Option<&ModeConfig> {
        self.mode_configs.get(&mode)
    }
    
    // 检查是否允许保存
    pub fn can_save(&self) -> bool {
        if let Some(config) = self.mode_configs.get(&self.current_mode) {
            config.allow_save && self.current_state != GameState::BattleTransition
        } else {
            false
        }
    }
    
    // 检查是否允许暂停
    pub fn can_pause(&self) -> bool {
        if let Some(config) = self.mode_configs.get(&self.current_mode) {
            config.allow_pause
        } else {
            false
        }
    }
    
    // 检查时间限制
    pub fn check_time_limit(&self) -> Option<Duration> {
        if let Some(config) = self.mode_configs.get(&self.current_mode) {
            if let Some(time_limit) = config.time_limit {
                let elapsed = self.mode_start_time.elapsed();
                if elapsed >= time_limit {
                    Some(Duration::ZERO)
                } else {
                    Some(time_limit - elapsed)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    // 更新统计数据
    pub fn update_stats(&mut self, stat_type: StatType, value: u32) {
        match stat_type {
            StatType::BattleWon => self.session_stats.battles_won += value,
            StatType::BattleLost => self.session_stats.battles_lost += value,
            StatType::PokemonCaught => self.session_stats.pokemon_caught += value,
            StatType::StepsTaken => self.session_stats.steps_taken += value,
            StatType::ItemUsed => self.session_stats.items_used += value,
        }
    }
    
    // 设置剧情进度
    pub fn set_story_progress(&mut self, progress: f32) {
        self.session_stats.story_progress = progress.clamp(0.0, 100.0);
        debug!("剧情进度更新: {:.1}%", self.session_stats.story_progress);
    }
    
    // 解锁成就
    pub fn unlock_achievement(&mut self, achievement: String) {
        if !self.session_stats.achievements_unlocked.contains(&achievement) {
            self.session_stats.achievements_unlocked.push(achievement.clone());
            info!("解锁成就: {}", achievement);
        }
    }
    
    // 获取统计数据
    pub fn get_stats(&self) -> &SessionStats {
        &self.session_stats
    }
    
    // 重置统计数据
    pub fn reset_stats(&mut self) {
        self.session_stats = SessionStats::default();
        info!("游戏统计数据已重置");
    }
    
    // 获取总游戏时间
    pub fn get_total_play_time(&self) -> Duration {
        let mut total = self.session_stats.mode_play_time.values().sum::<Duration>();
        total += self.mode_start_time.elapsed();
        total
    }
    
    // 获取当前模式游戏时间
    pub fn get_current_mode_time(&self) -> Duration {
        let previous_time = self.session_stats.mode_play_time
            .get(&self.current_mode)
            .copied()
            .unwrap_or(Duration::ZERO);
        previous_time + self.mode_start_time.elapsed()
    }
}

// 统计类型
pub enum StatType {
    BattleWon,
    BattleLost,
    PokemonCaught,
    StepsTaken,
    ItemUsed,
}

// 模式切换事件
#[derive(Debug, Clone)]
pub struct ModeChangeEvent {
    pub from_mode: GameMode,
    pub to_mode: GameMode,
    pub timestamp: Instant,
}

impl crate::core::event_system::Event for ModeChangeEvent {
    fn event_type(&self) -> &'static str {
        "ModeChange"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 状态切换事件
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub from_state: GameState,
    pub to_state: GameState,
    pub timestamp: Instant,
}

impl crate::core::event_system::Event for StateChangeEvent {
    fn event_type(&self) -> &'static str {
        "StateChange"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 成就解锁事件
#[derive(Debug, Clone)]
pub struct AchievementEvent {
    pub achievement_name: String,
    pub description: String,
    pub timestamp: Instant,
}

impl crate::core::event_system::Event for AchievementEvent {
    fn event_type(&self) -> &'static str {
        "Achievement"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_mode_manager_creation() {
        let manager = GameModeManager::new();
        assert_eq!(manager.current_mode(), GameMode::MainStory);
        assert_eq!(manager.current_state(), GameState::MainMenu);
    }
    
    #[test]
    fn test_mode_switching() {
        let mut manager = GameModeManager::new();
        
        // 切换到自由探索模式
        assert!(manager.switch_mode(GameMode::FreeRoam).is_ok());
        assert_eq!(manager.current_mode(), GameMode::FreeRoam);
    }
    
    #[test]
    fn test_temporary_mode() {
        let mut manager = GameModeManager::new();
        let original_mode = manager.current_mode();
        
        // 推入临时模式
        assert!(manager.push_temporary_mode(GameMode::Battle).is_ok());
        assert_eq!(manager.current_mode(), GameMode::Battle);
        
        // 返回原模式
        assert!(manager.pop_mode().is_ok());
        assert_eq!(manager.current_mode(), original_mode);
    }
    
    #[test]
    fn test_stats_update() {
        let mut manager = GameModeManager::new();
        
        manager.update_stats(StatType::BattleWon, 5);
        manager.update_stats(StatType::PokemonCaught, 3);
        
        assert_eq!(manager.get_stats().battles_won, 5);
        assert_eq!(manager.get_stats().pokemon_caught, 3);
    }
    
    #[test]
    fn test_achievements() {
        let mut manager = GameModeManager::new();
        
        manager.unlock_achievement("首次胜利".to_string());
        manager.unlock_achievement("收集家".to_string());
        
        assert_eq!(manager.get_stats().achievements_unlocked.len(), 2);
        assert!(manager.get_stats().achievements_unlocked.contains(&"首次胜利".to_string()));
    }
}