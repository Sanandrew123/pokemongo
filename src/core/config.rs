// 配置管理系统
// 开发心理：统一的配置管理，支持环境变量、配置文件和命令行参数
// 提供类型安全的配置访问和热重载功能，便于开发和部署

use crate::core::error::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};

// 主要游戏配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub graphics: GraphicsConfig,
    pub audio: AudioConfig,
    pub network: NetworkConfig,
    pub gameplay: GameplayConfig,
    pub debug_mode: bool,
}

// 图形配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub max_fps: u32,
    pub graphics_quality: GraphicsQuality,
    pub render_scale: f32,
    pub shadows: bool,
    pub anti_aliasing: AntiAliasing,
}

// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub audio_quality: AudioQuality,
}

// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub server_url: String,
    pub port: u16,
    pub timeout_ms: u32,
    pub max_players: usize,
    pub enable_p2p: bool,
    pub compression: bool,
}

// 游戏玩法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
    pub battle_animations: bool,
    pub battle_speed: f32,
    pub auto_save: bool,
    pub auto_save_interval: u32,
    pub difficulty: Difficulty,
    pub language: String,
}

// 图形质量等级
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
    Ultra,
}

// 抗锯齿设置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AntiAliasing {
    None,
    FXAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
}

// 音频质量等级
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AudioQuality {
    Low,      // 22kHz
    Medium,   // 44kHz
    High,     // 48kHz
    Studio,   // 96kHz
}

// 游戏难度
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            network: NetworkConfig::default(),
            gameplay: GameplayConfig::default(),
            debug_mode: false,
        }
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            max_fps: 60,
            graphics_quality: GraphicsQuality::Medium,
            render_scale: 1.0,
            shadows: true,
            anti_aliasing: AntiAliasing::FXAA,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.9,
            voice_volume: 1.0,
            audio_quality: AudioQuality::High,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_url: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
            max_players: 100,
            enable_p2p: false,
            compression: true,
        }
    }
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            battle_animations: true,
            battle_speed: 1.0,
            auto_save: true,
            auto_save_interval: 300, // 5分钟
            difficulty: Difficulty::Normal,
            language: "zh-CN".to_string(),
        }
    }
}

impl GameConfig {
    // 从配置文件加载
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| GameError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: GameConfig = toml::from_str(&content)
            .map_err(|e| GameError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        Ok(config)
    }
    
    // 保存到配置文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| GameError::ConfigError(format!("序列化配置失败: {}", e)))?;
        
        fs::write(path, content)
            .map_err(|e| GameError::ConfigError(format!("写入配置文件失败: {}", e)))?;
        
        Ok(())
    }
    
    // 从环境变量覆盖配置
    pub fn apply_env_overrides(&mut self) -> Result<()> {
        if let Ok(width) = env::var("POKEMON_WIDTH") {
            self.graphics.width = width.parse()
                .map_err(|_| GameError::ConfigError("POKEMON_WIDTH 不是有效数字".to_string()))?;
        }
        
        if let Ok(height) = env::var("POKEMON_HEIGHT") {
            self.graphics.height = height.parse()
                .map_err(|_| GameError::ConfigError("POKEMON_HEIGHT 不是有效数字".to_string()))?;
        }
        
        if let Ok(fullscreen) = env::var("POKEMON_FULLSCREEN") {
            self.graphics.fullscreen = fullscreen.parse()
                .map_err(|_| GameError::ConfigError("POKEMON_FULLSCREEN 不是有效布尔值".to_string()))?;
        }
        
        if let Ok(server_url) = env::var("POKEMON_SERVER") {
            self.network.server_url = server_url;
        }
        
        if let Ok(port) = env::var("POKEMON_PORT") {
            self.network.port = port.parse()
                .map_err(|_| GameError::ConfigError("POKEMON_PORT 不是有效端口号".to_string()))?;
        }
        
        if let Ok(debug) = env::var("POKEMON_DEBUG") {
            self.debug_mode = debug.parse()
                .map_err(|_| GameError::ConfigError("POKEMON_DEBUG 不是有效布尔值".to_string()))?;
        }
        
        Ok(())
    }
    
    // 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        if self.graphics.width < 640 || self.graphics.height < 480 {
            return Err(GameError::ConfigError("分辨率太小，最小支持640x480".to_string()));
        }
        
        if self.graphics.max_fps < 30 || self.graphics.max_fps > 240 {
            return Err(GameError::ConfigError("FPS必须在30-240之间".to_string()));
        }
        
        if !(0.0..=1.0).contains(&self.audio.master_volume) {
            return Err(GameError::ConfigError("主音量必须在0.0-1.0之间".to_string()));
        }
        
        if self.network.port == 0 {
            return Err(GameError::ConfigError("网络端口不能为0".to_string()));
        }
        
        if self.network.max_players > 1000 {
            return Err(GameError::ConfigError("最大玩家数不能超过1000".to_string()));
        }
        
        Ok(())
    }
    
    // 获取配置路径
    pub fn get_config_path() -> Result<std::path::PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| GameError::ConfigError("无法获取配置目录".to_string()))?;
        
        path.push("pokemongo");
        
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| GameError::ConfigError(format!("创建配置目录失败: {}", e)))?;
        }
        
        path.push("config.toml");
        Ok(path)
    }
    
    // 加载或创建默认配置
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        let mut config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let default_config = Self::default();
            default_config.save_to_file(&config_path)?;
            log::info!("已创建默认配置文件: {}", config_path.display());
            default_config
        };
        
        // 应用环境变量覆盖
        config.apply_env_overrides()?;
        
        // 验证配置
        config.validate()?;
        
        Ok(config)
    }
}

// 配置管理器 - 支持热重载
pub struct ConfigManager {
    config: GameConfig,
    config_path: std::path::PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = GameConfig::get_config_path()?;
        let config = GameConfig::load_or_default()?;
        
        let last_modified = if config_path.exists() {
            fs::metadata(&config_path)
                .ok()
                .and_then(|m| m.modified().ok())
        } else {
            None
        };
        
        Ok(Self {
            config,
            config_path,
            last_modified,
        })
    }
    
    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }
    
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut GameConfig),
    {
        updater(&mut self.config);
        self.config.validate()?;
        self.config.save_to_file(&self.config_path)?;
        Ok(())
    }
    
    // 检查并重载配置文件
    pub fn check_reload(&mut self) -> Result<bool> {
        if !self.config_path.exists() {
            return Ok(false);
        }
        
        let metadata = fs::metadata(&self.config_path)
            .map_err(|e| GameError::ConfigError(format!("读取配置文件元数据失败: {}", e)))?;
        
        let modified = metadata.modified()
            .map_err(|e| GameError::ConfigError(format!("获取文件修改时间失败: {}", e)))?;
        
        if Some(modified) != self.last_modified {
            let new_config = GameConfig::load_from_file(&self.config_path)?;
            new_config.validate()?;
            
            self.config = new_config;
            self.last_modified = Some(modified);
            
            log::info!("配置文件已重载");
            return Ok(true);
        }
        
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.graphics.width, 1280);
        assert_eq!(config.graphics.height, 720);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = GameConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: GameConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.graphics.width, parsed.graphics.width);
        assert_eq!(config.graphics.height, parsed.graphics.height);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = GameConfig::default();
        
        // 测试无效分辨率
        config.graphics.width = 100;
        assert!(config.validate().is_err());
        
        // 测试无效音量
        config.graphics.width = 1280; // 重置
        config.audio.master_volume = 2.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let original_config = GameConfig::default();
        original_config.save_to_file(&config_path).unwrap();
        
        let loaded_config = GameConfig::load_from_file(&config_path).unwrap();
        assert_eq!(original_config.graphics.width, loaded_config.graphics.width);
    }
}