// 核心模块 - 游戏引擎基础系统
// 开发心理：建立稳固的基础架构，为上层功能提供可靠的底层支持
// 包含错误处理、配置管理、数学工具、时间系统等核心功能

pub mod app;
pub mod error;
pub mod config;
pub mod math;
pub mod time;

// 重新导出核心类型
pub use app::App;
pub use error::{GameError, Result};
pub use config::GameConfig;
pub use math::{Vector2, Vector3, Matrix4};
pub use time::{GameTime, Timer};

// 核心系统初始化
pub fn init() -> Result<()> {
    log::info!("初始化核心系统");
    
    // 初始化时间系统
    time::init()?;
    
    // 初始化数学库
    math::init()?;
    
    log::info!("核心系统初始化完成");
    Ok(())
}

// 核心系统清理
pub fn cleanup() {
    log::info!("清理核心系统");
    
    time::cleanup();
    math::cleanup();
    
    log::info!("核心系统清理完成");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_core_init() {
        // 测试核心系统初始化
        assert!(init().is_ok());
        cleanup();
    }
}