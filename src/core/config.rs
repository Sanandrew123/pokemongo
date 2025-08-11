/*
* 开发心理过程：
* 1. 创建游戏配置管理系统，支持多种配置来源
* 2. 实现配置的加载、保存、验证和热重载功能
* 3. 提供类型安全的配置访问接口
* 4. 支持配置继承和覆盖机制
* 5. 集成环境变量和命令行参数支持
* 6. 提供配置变更监听和回调机制
* 7. 确保高性能的配置访问和内存效率
*/

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
    env,
};
use bevy::prelude::*;
use tracing::{info, warn, error, debug};

use crate::core::error::{GameError, GameResult};

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct GameConfig {
    pub general: GeneralConfig,
    pub graphics: GraphicsConfig,
    pub audio: AudioConfig,
    pub input: InputConfig,
    pub network: NetworkConfig,
    pub performance: PerformanceConfig,
    pub debug: DebugConfig,
    pub pokemon: PokemonConfig,
    pub battle: BattleConfig,
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub game_title: String,
    pub version: String,
    pub language: String,
    pub region: String,
    pub save_directory: PathBuf,
    pub auto_save_interval: Duration,
    pub max_save_files: u32,
    pub debug_mode: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub keyboard_repeat_delay: Duration,
    pub keyboard_repeat_rate: Duration,
    pub gamepad_deadzone: f32,
    pub gamepad_sensitivity: f32,
    pub touch_sensitivity: f32,
    pub gesture_threshold: f32,
    pub double_click_time: Duration,
    pub key_bindings: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub target_fps: u32,
    pub frame_time_smoothing: f32,
    pub memory_pool_size: usize,
    pub thread_pool_size: Option<usize>,
    pub asset_streaming: bool,
    pub texture_compression: bool,
    pub audio_compression: bool,
    pub gc_interval: Duration,
    pub profiling_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub show_fps: bool,
    pub show_memory: bool,
    pub show_render_stats: bool,
    pub show_collision_boxes: bool,
    pub show_ai_debug: bool,
    pub wireframe_mode: bool,
    pub god_mode: bool,
    pub infinite_resources: bool,
    pub skip_intro: bool,
    pub dev_console: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonConfig {
    pub max_party_size: u32,
    pub max_box_storage: u32,
    pub shiny_rate: f32,
    pub experience_multiplier: f32,
    pub catch_rate_multiplier: f32,
    pub evolution_animation_speed: f32,
    pub stats_calculation_method: String,
    pub iv_generation_method: String,
    pub nature_effects_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleConfig {
    pub max_battle_time: Duration,
    pub animation_speed: f32,
    pub auto_battle: bool,
    pub skip_animations: bool,
    pub damage_numbers_visible: bool,
    pub type_effectiveness_hints: bool,
    pub ai_difficulty: AIDifficulty,
    pub weather_effects_enabled: bool,
    pub status_effects_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub max_fps: u32,
    pub render_scale: f32,
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
    pub effects_quality: EffectsQuality,
    pub ui_scale: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub audio_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub spatial_audio: bool,
    pub reverb_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub enabled: bool,
    pub server_address: String,
    pub server_port: u16,
    pub timeout: Duration,
    pub max_retries: u32,
    pub connection_pool_size: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EffectsQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AIDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    Nightmare,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            input: InputConfig::default(),
            network: NetworkConfig::default(),
            performance: PerformanceConfig::default(),
            debug: DebugConfig::default(),
            pokemon: PokemonConfig::default(),
            battle: BattleConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            game_title: "Pokemon Adventure".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            language: "en".to_string(),
            region: "US".to_string(),
            save_directory: PathBuf::from("saves"),
            auto_save_interval: Duration::from_secs(300), // 5分钟
            max_save_files: 10,
            debug_mode: cfg!(debug_assertions),
            log_level: if cfg!(debug_assertions) { "debug".to_string() } else { "info".to_string() },
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
            msaa_samples: 4,
            max_fps: 60,
            render_scale: 1.0,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::High,
            effects_quality: EffectsQuality::Medium,
            ui_scale: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            voice_volume: 0.9,
            audio_device: None,
            sample_rate: 44100,
            buffer_size: 1024,
            spatial_audio: true,
            reverb_enabled: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_address: "127.0.0.1".to_string(),
            server_port: 7777,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            connection_pool_size: 10,
            compression_enabled: true,
            encryption_enabled: true,
            rate_limit_requests: 100,
            rate_limit_window: Duration::from_secs(60),
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();
        key_bindings.insert("move_up".to_string(), vec!["KeyW".to_string(), "ArrowUp".to_string()]);
        key_bindings.insert("move_down".to_string(), vec!["KeyS".to_string(), "ArrowDown".to_string()]);
        key_bindings.insert("move_left".to_string(), vec!["KeyA".to_string(), "ArrowLeft".to_string()]);
        key_bindings.insert("move_right".to_string(), vec!["KeyD".to_string(), "ArrowRight".to_string()]);
        key_bindings.insert("confirm".to_string(), vec!["Enter".to_string(), "Space".to_string()]);
        key_bindings.insert("cancel".to_string(), vec!["Escape".to_string()]);
        key_bindings.insert("menu".to_string(), vec!["Tab".to_string()]);
        
        Self {
            mouse_sensitivity: 1.0,
            keyboard_repeat_delay: Duration::from_millis(500),
            keyboard_repeat_rate: Duration::from_millis(50),
            gamepad_deadzone: 0.1,
            gamepad_sensitivity: 1.0,
            touch_sensitivity: 1.0,
            gesture_threshold: 0.3,
            double_click_time: Duration::from_millis(300),
            key_bindings,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            frame_time_smoothing: 0.9,
            memory_pool_size: 64 * 1024 * 1024, // 64MB
            thread_pool_size: None, // 自动检测
            asset_streaming: true,
            texture_compression: true,
            audio_compression: true,
            gc_interval: Duration::from_secs(10),
            profiling_enabled: cfg!(debug_assertions),
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        let debug_mode = cfg!(debug_assertions);
        Self {
            show_fps: debug_mode,
            show_memory: debug_mode,
            show_render_stats: debug_mode,
            show_collision_boxes: false,
            show_ai_debug: false,
            wireframe_mode: false,
            god_mode: false,
            infinite_resources: false,
            skip_intro: debug_mode,
            dev_console: debug_mode,
        }
    }
}

impl Default for PokemonConfig {
    fn default() -> Self {
        Self {
            max_party_size: 6,
            max_box_storage: 720, // 24盒子 * 30个
            shiny_rate: 1.0 / 4096.0,
            experience_multiplier: 1.0,
            catch_rate_multiplier: 1.0,
            evolution_animation_speed: 1.0,
            stats_calculation_method: "standard".to_string(),
            iv_generation_method: "random".to_string(),
            nature_effects_enabled: true,
        }
    }
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            max_battle_time: Duration::from_secs(600), // 10分钟
            animation_speed: 1.0,
            auto_battle: false,
            skip_animations: false,
            damage_numbers_visible: true,
            type_effectiveness_hints: true,
            ai_difficulty: AIDifficulty::Normal,
            weather_effects_enabled: true,
            status_effects_enabled: true,
        }
    }
}

#[derive(Resource)]
pub struct ConfigManager {
    config: Arc<RwLock<GameConfig>>,
    config_path: PathBuf,
    last_modified: SystemTime,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

pub trait ConfigWatcher: Send + Sync {
    fn on_config_changed(&self, old_config: &GameConfig, new_config: &GameConfig);
}

impl ConfigManager {
    pub fn new() -> GameResult<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_from_file(&config_path)?;
        let last_modified = fs::metadata(&config_path)
            .map(|m| m.modified().unwrap_or_else(|_| SystemTime::now()))
            .unwrap_or_else(|_| SystemTime::now());

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            last_modified,
            watchers: Vec::new(),
        })
    }

    pub fn get_config(&self) -> GameConfig {
        self.config.read().unwrap().clone()
    }

    pub fn update_config<F>(&mut self, updater: F) -> GameResult<()>
    where
        F: FnOnce(&mut GameConfig),
    {
        let old_config = self.get_config();
        
        {
            let mut config = self.config.write().unwrap();
            updater(&mut *config);
        }
        
        let new_config = self.get_config();
        
        // 通知观察者
        for watcher in &self.watchers {
            watcher.on_config_changed(&old_config, &new_config);
        }
        
        // 保存到文件
        self.save_to_file()?;
        
        info!("配置已更新并保存");
        Ok(())
    }

    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        self.watchers.push(watcher);
    }

    pub fn check_reload(&mut self) -> GameResult<bool> {
        let metadata = fs::metadata(&self.config_path)?;
        let modified = metadata.modified()?;
        
        if modified > self.last_modified {
            info!("检测到配置文件更改，重新加载");
            let old_config = self.get_config();
            let new_config = Self::load_from_file(&self.config_path)?;
            
            *self.config.write().unwrap() = new_config.clone();
            self.last_modified = modified;
            
            // 通知观察者
            for watcher in &self.watchers {
                watcher.on_config_changed(&old_config, &new_config);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_config_path() -> GameResult<PathBuf> {
        let mut path = std::env::current_dir()?;
        path.push("config");
        
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        
        path.push("game.toml");
        Ok(path)
    }

    fn load_from_file(path: &Path) -> GameResult<GameConfig> {
        if !path.exists() {
            info!("配置文件不存在，创建默认配置: {:?}", path);
            let default_config = GameConfig::default();
            Self::save_config_to_file(&default_config, path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(path)?;
        let config: GameConfig = toml::from_str(&content).map_err(|e| {
            GameError::ConfigError(format!("解析配置文件失败: {}", e))
        })?;

        Self::validate_config(&config)?;
        info!("成功加载配置文件: {:?}", path);
        Ok(config)
    }

    fn save_to_file(&self) -> GameResult<()> {
        let config = self.config.read().unwrap();
        Self::save_config_to_file(&*config, &self.config_path)
    }

    fn save_config_to_file(config: &GameConfig, path: &Path) -> GameResult<()> {
        let content = toml::to_string_pretty(config).map_err(|e| {
            GameError::ConfigError(format!("序列化配置失败: {}", e))
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        debug!("配置已保存到: {:?}", path);
        Ok(())
    }

    fn validate_config(config: &GameConfig) -> GameResult<()> {
        // 验证图形配置
        if config.graphics.width == 0 || config.graphics.height == 0 {
            return Err(GameError::ConfigError("无效的屏幕分辨率".to_string()));
        }

        if config.graphics.max_fps == 0 {
            return Err(GameError::ConfigError("无效的最大FPS设置".to_string()));
        }

        // 验证音频配置
        if !(0.0..=1.0).contains(&config.audio.master_volume) {
            return Err(GameError::ConfigError("音量必须在0.0-1.0之间".to_string()));
        }

        // 验证Pokemon配置
        if config.pokemon.max_party_size == 0 || config.pokemon.max_party_size > 6 {
            return Err(GameError::ConfigError("队伍大小必须在1-6之间".to_string()));
        }

        Ok(())
    }

    pub fn load_from_args(args: &[String]) -> GameResult<GameConfig> {
        let mut config = GameConfig::default();

        for (i, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--config" | "-c" => {
                    if let Some(config_path) = args.get(i + 1) {
                        let path = PathBuf::from(config_path);
                        config = Self::load_from_file(&path)?;
                    }
                }
                "--width" => {
                    if let Some(width) = args.get(i + 1) {
                        config.graphics.width = width.parse().unwrap_or(1280);
                    }
                }
                "--height" => {
                    if let Some(height) = args.get(i + 1) {
                        config.graphics.height = height.parse().unwrap_or(720);
                    }
                }
                "--fullscreen" => config.graphics.fullscreen = true,
                "--windowed" => config.graphics.fullscreen = false,
                "--debug" => config.debug = DebugConfig {
                    show_fps: true,
                    dev_console: true,
                    ..Default::default()
                },
                _ => {}
            }
        }

        Self::validate_config(&config)?;
        Ok(config)
    }
}
// Bevy系统插件
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config_manager = ConfigManager::new().expect("Failed to initialize config manager");
        let config = config_manager.get_config();

        app.insert_resource(config_manager)
           .insert_resource(config)
           .add_systems(Update, config_hot_reload_system);
    }
}

fn config_hot_reload_system(
    mut config_manager: ResMut<ConfigManager>,
    mut config: ResMut<GameConfig>,
) {
    if let Ok(reloaded) = config_manager.check_reload() {
        if reloaded {
            *config = config_manager.get_config();
            info!("配置已热重载");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.graphics.width, 1280);
        assert_eq!(config.graphics.height, 720);
        assert_eq!(config.pokemon.max_party_size, 6);
    }

    #[test]
    fn test_config_serialization() {
        let config = GameConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: GameConfig = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config.graphics.width, deserialized.graphics.width);
        assert_eq!(config.audio.master_volume, deserialized.audio.master_volume);
    }

    #[test]
    fn test_config_validation() {
        let mut config = GameConfig::default();
        assert!(ConfigManager::validate_config(&config).is_ok());
        
        config.graphics.width = 0;
        assert!(ConfigManager::validate_config(&config).is_err());
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let config = GameConfig::default();
        ConfigManager::save_config_to_file(&config, &config_path).unwrap();
        
        assert!(config_path.exists());
        
        let loaded_config = ConfigManager::load_from_file(&config_path).unwrap();
        assert_eq!(config.graphics.width, loaded_config.graphics.width);
    }
}