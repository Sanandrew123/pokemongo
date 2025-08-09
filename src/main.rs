// 高性能宝可梦游戏主程序入口
// 开发心理：简洁的启动流程，专注于初始化和游戏循环管理
// 使用Bevy引擎的App架构，保持代码整洁和可测试性

use pokemongo::{
    App, GameConfig, GameError, Result,
    init, cleanup,
};
use std::env;
use log::{info, error};

fn main() {
    // 初始化日志系统
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    info!("🎮 启动宝可梦游戏 v{}", pokemongo::VERSION);
    
    // 运行游戏，处理所有错误
    if let Err(e) = run_game() {
        error!("游戏运行失败: {}", e);
        std::process::exit(1);
    }
    
    // 清理资源
    cleanup();
    info!("游戏正常退出");
}

fn run_game() -> Result<()> {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;
    
    // 初始化游戏引擎
    init()?;
    
    // 创建并运行应用程序
    let mut app = App::new(config)?;
    app.run()
}

fn parse_args(args: &[String]) -> Result<GameConfig> {
    let mut config = GameConfig::default();
    
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--fullscreen" => config.graphics.fullscreen = true,
            "--windowed" => config.graphics.fullscreen = false,
            "--debug" => config.debug_mode = true,
            "--no-audio" => config.audio.enabled = false,
            "--resolution" => {
                if i + 1 < args.len() {
                    let resolution = &args[i + 1];
                    let parts: Vec<&str> = resolution.split('x').collect();
                    if parts.len() == 2 {
                        config.graphics.width = parts[0].parse().unwrap_or(1280);
                        config.graphics.height = parts[1].parse().unwrap_or(720);
                    }
                }
            },
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            },
            _ => {}
        }
    }
    
    Ok(config)
}

fn print_help() {
    println!("宝可梦游戏 v{}", pokemongo::VERSION);
    println!();
    println!("使用方法:");
    println!("  {} [选项]", env!("CARGO_BIN_NAME"));
    println!();
    println!("选项:");
    println!("  --fullscreen     全屏模式");
    println!("  --windowed       窗口模式");
    println!("  --resolution WxH 设置分辨率 (例: --resolution 1920x1080)");
    println!("  --debug          启用调试模式");
    println!("  --no-audio       禁用音频");
    println!("  --help, -h       显示帮助信息");
    println!();
    println!("示例:");
    println!("  {} --fullscreen --resolution 1920x1080", env!("CARGO_BIN_NAME"));
    println!("  {} --windowed --debug", env!("CARGO_BIN_NAME"));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_args_default() {
        let args = vec!["pokemongo".to_string()];
        let config = parse_args(&args).unwrap();
        
        assert_eq!(config.graphics.width, 1280);
        assert_eq!(config.graphics.height, 720);
        assert!(!config.graphics.fullscreen);
        assert!(config.audio.enabled);
    }
    
    #[test]
    fn test_parse_args_fullscreen() {
        let args = vec!["pokemongo".to_string(), "--fullscreen".to_string()];
        let config = parse_args(&args).unwrap();
        
        assert!(config.graphics.fullscreen);
    }
    
    #[test]
    fn test_parse_args_resolution() {
        let args = vec![
            "pokemongo".to_string(),
            "--resolution".to_string(),
            "1920x1080".to_string(),
        ];
        let config = parse_args(&args).unwrap();
        
        assert_eq!(config.graphics.width, 1920);
        assert_eq!(config.graphics.height, 1080);
    }
}