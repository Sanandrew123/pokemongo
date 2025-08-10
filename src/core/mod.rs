// 核心模块 - 游戏引擎基础系统
// 开发心理：建立稳固的基础架构，为上层功能提供可靠的底层支持
// 包含错误处理、配置管理、数学工具、时间系统等核心功能

// 基础系统 - 始终可用
pub mod error;
pub mod config;
pub mod event_system;
pub mod resource_manager;
pub mod time;

// 实验性模块 - 需要feature启用
#[cfg(feature = "custom-engine")]
pub mod app;

#[cfg(feature = "custom-engine")]
pub mod engine;

// 工具模块 - 部分已在utils模块中实现
// pub mod math; // 已在utils/math.rs中实现
// pub mod time; // 需要创建简化版本

// 重新导出核心类型
pub use error::{GameError, Result};
pub use config::GameConfig;
pub use time::{GameTime, Timer};

// 仅在相应feature启用时导出
#[cfg(feature = "custom-engine")]
pub use app::App;

// 数学类型从utils模块导出
// pub use crate::utils::{Vector2, Vector3, Matrix4};

// 时间类型需要实现后再导出
// pub use time::{GameTime, Timer};

// 核心系统初始化
pub fn init() -> Result<()> {
    log::info!("初始化核心系统");
    
    // 只初始化确实存在的系统
    // 等相应模块实现后再启用
    // time::init()?;
    // math::init()?;
    
    log::info!("核心系统初始化完成");
    Ok(())
}

// 核心系统清理
pub fn cleanup() {
    log::info!("清理核心系统");
    
    // 等相应模块实现后再启用
    // time::cleanup();
    // math::cleanup();
    
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