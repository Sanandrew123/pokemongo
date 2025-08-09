// 高性能宝可梦游戏引擎 - 项目根库文件
// 开发心理：建立清晰的模块架构，分离关注点，确保代码可维护性
// 采用Rust + C++混合架构，Rust负责安全性和并发，C++负责性能关键路径

pub mod core;
pub mod engine;
pub mod pokemon;
pub mod battle;
pub mod creature_engine;
pub mod game_modes;
pub mod network;
pub mod world;
pub mod player;
pub mod ui;
pub mod utils;
pub mod save;

// 重新导出核心类型，简化API使用
pub use core::{
    app::App,
    error::{GameError, Result},
    config::GameConfig,
};

pub use pokemon::{
    species::PokemonSpecies,
    individual::Pokemon,
    types::PokemonType,
};

pub use battle::engine::BattleEngine;
pub use creature_engine::generator::CreatureGenerator;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

// 游戏常量
pub mod constants {
    pub const MAX_POKEMON_PER_TEAM: usize = 6;
    pub const MAX_LEVEL: u8 = 100;
    pub const MIN_LEVEL: u8 = 1;
    pub const MAX_POKEMON_TYPES: usize = 2;
    pub const MAX_MOVES_PER_POKEMON: usize = 4;
}

// C++互操作性声明
extern "C" {
    // 数学计算函数（来自C++模块）
    fn native_damage_calculation(
        attack_power: f32,
        defense: f32,
        level: u8,
        type_effectiveness: f32,
    ) -> f32;
    
    // 音频处理函数
    fn native_play_sound(sound_id: u32, volume: f32);
    
    // 图形处理函数
    fn native_render_sprite(sprite_id: u32, x: f32, y: f32, scale: f32);
}

// 安全的C++函数包装器
pub mod ffi {
    use super::*;
    
    pub fn calculate_damage(attack: f32, defense: f32, level: u8, effectiveness: f32) -> f32 {
        unsafe {
            native_damage_calculation(attack, defense, level, effectiveness)
        }
    }
    
    pub fn play_sound(id: u32, volume: f32) {
        unsafe {
            native_play_sound(id, volume);
        }
    }
    
    pub fn render_sprite(id: u32, x: f32, y: f32, scale: f32) {
        unsafe {
            native_render_sprite(id, x, y, scale);
        }
    }
}

// 初始化函数
pub fn init() -> Result<()> {
    log::info!("初始化宝可梦游戏引擎 v{}", VERSION);
    
    // 初始化各个子系统
    core::init()?;
    
    Ok(())
}

// 清理函数
pub fn cleanup() {
    log::info!("清理游戏引擎资源");
    core::cleanup();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        assert_eq!(MAX_POKEMON_PER_TEAM, 6);
        assert_eq!(MAX_LEVEL, 100);
        assert_eq!(MIN_LEVEL, 1);
    }
    
    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
}