// 错误处理系统
// 开发心理：统一的错误类型系统，提供清晰的错误信息和恢复机制
// 使用Rust的Result类型确保错误处理的安全性和一致性

use std::{fmt, error::Error as StdError, io};
use serde::{Serialize, Deserialize};

// 游戏主要错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    // 系统级错误
    InitializationFailed(String),
    ConfigError(String),
    ResourceNotFound(String),
    
    // 图形相关错误
    RenderError(String),
    ShaderError(String),
    TextureError(String),
    
    // 音频相关错误
    AudioError(String),
    SoundNotFound(String),
    
    // 网络相关错误
    NetworkError(String),
    ConnectionFailed(String),
    
    // 游戏逻辑错误
    BattleError(String),
    PokemonError(String),
    SaveError(String),
    PlayerError(String),
    UIError(String),
    AssetError(String),
    GameModeError(String),
    
    // 新增错误类型
    Data(String),
    Database(String),
    ECS(String),
    World(String),
    Progress(String),
    NPC(String),
    Map(String),
    Network(String),
    
    // IO错误
    IOError(String),
    
    // 状态错误
    State(String),
    
    // 系统错误
    SystemError(String),
    
    // 压缩错误
    CompressionError(String),
    
    // 序列化错误
    SerializationError(String),
    
    // 玩家错误
    Player(String),
    
    // 物品错误
    Inventory(String),
    
    // 泛型错误
    GenericError(String),
    
    // I/O错误
    FileError(String),
    ParseError(String),
    
    // 通用错误
    InvalidInput(String),
    NotImplemented(String),
    Unknown(String),
}

// Result类型别名
pub type Result<T> = std::result::Result<T, GameError>;
pub type GameResult<T> = std::result::Result<T, GameError>;

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            GameError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            GameError::ResourceNotFound(msg) => write!(f, "资源未找到: {}", msg),
            
            GameError::RenderError(msg) => write!(f, "渲染错误: {}", msg),
            GameError::ShaderError(msg) => write!(f, "着色器错误: {}", msg),
            GameError::TextureError(msg) => write!(f, "纹理错误: {}", msg),
            
            GameError::AudioError(msg) => write!(f, "音频错误: {}", msg),
            GameError::SoundNotFound(msg) => write!(f, "音效未找到: {}", msg),
            
            GameError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            GameError::ConnectionFailed(msg) => write!(f, "连接失败: {}", msg),
            
            GameError::BattleError(msg) => write!(f, "战斗错误: {}", msg),
            GameError::PokemonError(msg) => write!(f, "宝可梦错误: {}", msg),
            GameError::SaveError(msg) => write!(f, "存档错误: {}", msg),
            GameError::PlayerError(msg) => write!(f, "玩家错误: {}", msg),
            GameError::UIError(msg) => write!(f, "UI错误: {}", msg),
            GameError::AssetError(msg) => write!(f, "资源错误: {}", msg),
            GameError::GameModeError(msg) => write!(f, "游戏模式错误: {}", msg),
            
            GameError::Data(msg) => write!(f, "数据错误: {}", msg),
            GameError::Database(msg) => write!(f, "数据库错误: {}", msg),
            GameError::ECS(msg) => write!(f, "ECS错误: {}", msg),
            GameError::World(msg) => write!(f, "世界错误: {}", msg),
            GameError::Progress(msg) => write!(f, "进度错误: {}", msg),
            GameError::NPC(msg) => write!(f, "NPC错误: {}", msg),
            GameError::Map(msg) => write!(f, "地图错误: {}", msg),
            GameError::Network(msg) => write!(f, "网络错误: {}", msg),
            
            GameError::GenericError(msg) => write!(f, "错误: {}", msg),
            
            GameError::FileError(msg) => write!(f, "文件错误: {}", msg),
            GameError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            
            GameError::InvalidInput(msg) => write!(f, "输入无效: {}", msg),
            GameError::NotImplemented(msg) => write!(f, "功能未实现: {}", msg),
            GameError::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl StdError for GameError {}

// 便捷的Result类型别名
pub type Result<T> = std::result::Result<T, GameError>;

// 错误转换实现
impl From<io::Error> for GameError {
    fn from(error: io::Error) -> Self {
        GameError::FileError(error.to_string())
    }
}

impl From<serde_json::Error> for GameError {
    fn from(error: serde_json::Error) -> Self {
        GameError::ParseError(error.to_string())
    }
}

impl From<toml::de::Error> for GameError {
    fn from(error: toml::de::Error) -> Self {
        GameError::ConfigError(error.to_string())
    }
}

impl From<rusqlite::Error> for GameError {
    fn from(error: rusqlite::Error) -> Self {
        GameError::Database(error.to_string())
    }
}

impl From<std::time::SystemTimeError> for GameError {
    fn from(error: std::time::SystemTimeError) -> Self {
        GameError::SystemError(error.to_string())
    }
}

// 错误创建辅助宏
#[macro_export]
macro_rules! game_error {
    ($variant:ident, $msg:expr) => {
        GameError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        GameError::$variant(format!($fmt, $($arg)*))
    };
}

// 错误恢复策略
#[derive(Debug, Clone)]
pub enum ErrorRecovery {
    Retry,
    UseDefault,
    Skip,
    Abort,
}

impl GameError {
    // 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            GameError::InitializationFailed(_) => ErrorSeverity::Critical,
            GameError::NetworkError(_) => ErrorSeverity::High,
            GameError::ResourceNotFound(_) => ErrorSeverity::Medium,
            GameError::InvalidInput(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
    
    // 获取推荐的恢复策略
    pub fn recovery_strategy(&self) -> ErrorRecovery {
        match self {
            GameError::InitializationFailed(_) => ErrorRecovery::Abort,
            GameError::ResourceNotFound(_) => ErrorRecovery::UseDefault,
            GameError::NetworkError(_) => ErrorRecovery::Retry,
            GameError::InvalidInput(_) => ErrorRecovery::Skip,
            _ => ErrorRecovery::UseDefault,
        }
    }
    
    // 检查是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, GameError::InitializationFailed(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = GameError::ResourceNotFound("test.png".to_string());
        assert_eq!(error.to_string(), "资源未找到: test.png");
    }
    
    #[test]
    fn test_error_severity() {
        let error = GameError::InitializationFailed("test".to_string());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
    }
    
    #[test]
    fn test_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let game_error: GameError = io_error.into();
        
        match game_error {
            GameError::FileError(_) => {},
            _ => panic!("Expected FileError"),
        }
    }
    
    #[test]
    fn test_game_error_macro() {
        let error = game_error!(InvalidInput, "test input: {}", 42);
        match error {
            GameError::InvalidInput(msg) => assert_eq!(msg, "test input: 42"),
            _ => panic!("Expected InvalidInput"),
        }
    }
}