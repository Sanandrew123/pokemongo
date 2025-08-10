// 高性能宝可梦游戏库入口
// 开发心理：现代Rust游戏开发最佳实践，注重性能、安全性和可维护性
// 架构：模块化设计，支持功能特性开关，便于测试和部署

// 核心模块 - 始终可用
pub mod core;
pub mod utils;
pub mod assets;
pub mod audio;

// 游戏系统 - 根据feature启用
#[cfg(feature = "pokemon-wip")]
pub mod pokemon;

#[cfg(feature = "battle-wip")]
pub mod battle;

#[cfg(feature = "graphics-wip")]
pub mod graphics;

#[cfg(feature = "network-wip")]
pub mod network;

// 实验性模块
#[cfg(feature = "custom-engine")]
pub mod engine;

#[cfg(feature = "creature-designer")]
pub mod creature_engine;

// 游戏模式
pub mod game_modes;
pub mod player;
pub mod save;
pub mod ui;
pub mod world;
pub mod states;
pub mod input;
pub mod data;
pub mod ecs;

// C++集成模块 (仅在native特性开启时)
#[cfg(feature = "native")]
pub mod bindings {
    //! C++绑定模块 - 提供高性能数学运算和平台特定优化
    
    extern "C" {
        // 数学函数
        pub fn simd_vector_add(a: *const f32, b: *const f32, result: *mut f32, count: usize);
        pub fn simd_matrix_multiply(a: *const f32, b: *const f32, result: *mut f32);
        pub fn simd_dot_product(a: *const f32, b: *const f32, count: usize) -> f32;
        
        // 战斗计算
        pub fn calculate_damage_native(
            attack: f32, 
            defense: f32, 
            level: u8, 
            effectiveness: f32
        ) -> f32;
        pub fn calculate_critical_hit(base_rate: f32, luck_factor: f32) -> bool;
        
        // 性能工具
        pub fn start_profiler();
        pub fn end_profiler() -> f64;
    }
}

// 非native模式的fallback实现
#[cfg(not(feature = "native"))]
pub mod bindings {
    //! Rust fallback实现
    
    pub fn simd_vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
        for ((a_val, b_val), result_val) in a.iter().zip(b.iter()).zip(result.iter_mut()) {
            *result_val = a_val + b_val;
        }
    }
    
    pub fn calculate_damage_native(attack: f32, defense: f32, level: u8, effectiveness: f32) -> f32 {
        let base_damage = (attack / defense) * (level as f32 / 50.0) * effectiveness;
        base_damage.max(1.0)
    }
    
    pub fn calculate_critical_hit(base_rate: f32, luck_factor: f32) -> bool {
        fastrand::f32() < (base_rate * luck_factor)
    }
    
    pub fn start_profiler() {
        // Rust性能分析实现
    }
    
    pub fn end_profiler() -> f64 {
        0.0
    }
}

// 重新导出核心类型
pub use core::{GameError, Result, GameConfig, GameTime, Timer};

// 根据特性导出模块
#[cfg(feature = "custom-engine")]
pub use core::App;

#[cfg(feature = "pokemon-wip")]
pub use pokemon::{Pokemon, PokemonSpecies, PokemonType};

#[cfg(feature = "battle-wip")]
pub use battle::{BattleEngine, BattleLogEntry, BattleActionResult};

#[cfg(feature = "graphics-wip")]
pub use graphics::{Renderer2D, RenderLayer, sprite_rendering_system};

// 版本信息 - 使用默认值避免编译时环境变量依赖
pub const VERSION: &str = "0.1.0";
pub const NAME: &str = "pokemongo";

// 游戏常量
pub mod constants {
    pub const MAX_POKEMON_PER_TEAM: usize = 6;
    pub const MAX_MOVES_PER_POKEMON: usize = 4;
    pub const MAX_LEVEL: u8 = 100;
    pub const MIN_LEVEL: u8 = 1;
    
    // 战斗常量
    pub const MAX_BATTLE_PARTICIPANTS: usize = 4;
    pub const DEFAULT_BATTLE_TIMEOUT_MINUTES: u32 = 30;
    
    // 性能常量
    pub const TARGET_FPS: u32 = 60;
    pub const MAX_FRAME_TIME_MS: u32 = 33; // ~30 FPS minimum
    
    // 网络常量
    pub const DEFAULT_SERVER_PORT: u16 = 7777;
    pub const MAX_PACKET_SIZE: usize = 1024;
    pub const HEARTBEAT_INTERVAL_MS: u64 = 5000;
}

// 便利函数
pub fn init() -> Result<()> {
    // 初始化日志系统
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pokemongo=info");
    }
    
    env_logger::init();
    
    log::info!("宝可梦游戏初始化完成 v{}", VERSION);
    
    // 初始化其他系统
    #[cfg(feature = "native")]
    unsafe {
        bindings::start_profiler();
    }
    
    Ok(())
}

pub fn cleanup() {
    log::info!("清理游戏资源");
    
    #[cfg(feature = "native")]
    unsafe {
        let profiler_time = bindings::end_profiler();
        log::info!("性能分析时间: {:.2}ms", profiler_time);
    }
}

// FFI包装函数，提供安全接口
#[cfg(feature = "native")]
pub fn calculate_damage(attack: f32, defense: f32, level: u8, effectiveness: f32) -> f32 {
    unsafe {
        bindings::calculate_damage_native(attack, defense, level, effectiveness)
    }
}

#[cfg(not(feature = "native"))]
pub fn calculate_damage(attack: f32, defense: f32, level: u8, effectiveness: f32) -> f32 {
    bindings::calculate_damage_native(attack, defense, level, effectiveness)
}

// 性能分析工具
pub struct PerformanceProfiler {
    start_time: std::time::Instant,
    name: String,
}

impl PerformanceProfiler {
    pub fn new(name: &str) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            name: name.to_string(),
        }
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl Drop for PerformanceProfiler {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        if elapsed.as_millis() > 1 {
            log::debug!("性能: {} 耗时 {:.2}ms", self.name, elapsed.as_secs_f64() * 1000.0);
        }
    }
}

// 便利宏
#[macro_export]
macro_rules! profile {
    ($name:expr, $code:block) => {
        {
            let _profiler = $crate::PerformanceProfiler::new($name);
            $code
        }
    };
}

// 错误处理便利宏
#[macro_export]
macro_rules! game_bail {
    ($msg:literal $(,)?) => {
        return Err($crate::GameError::GenericError($msg.to_string()))
    };
    ($err:expr $(,)?) => {
        return Err($crate::GameError::GenericError($err.to_string()))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::GameError::GenericError(format!($fmt, $($arg)*)))
    };
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_cleanup() {
        init().unwrap();
        cleanup();
    }
    
    #[test]
    fn test_damage_calculation() {
        let damage = calculate_damage(100.0, 50.0, 50, 2.0);
        assert!(damage >= 1.0);
        assert!(damage <= 200.0);
    }
    
    #[test]
    fn test_constants() {
        assert_eq!(constants::MAX_POKEMON_PER_TEAM, 6);
        assert_eq!(constants::MAX_MOVES_PER_POKEMON, 4);
        assert!(constants::MIN_LEVEL < constants::MAX_LEVEL);
    }
    
    #[test]
    fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new("test");
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(profiler.elapsed().as_millis() >= 1);
    }
    
    #[cfg(feature = "pokemon-wip")]
    #[test]
    fn test_pokemon_species_database() {
        use crate::pokemon::PokemonSpecies;
        
        let pikachu = PokemonSpecies::get(25);
        assert!(pikachu.is_some());
        
        if let Some(pikachu) = pikachu {
            assert_eq!(pikachu.name, "皮卡丘");
            assert_eq!(pikachu.base_stats.speed, 90);
        }
    }
    
    #[test]
    fn test_version_info() {
        assert_eq!(VERSION, "0.1.0");
        assert_eq!(NAME, "pokemongo");
    }
}