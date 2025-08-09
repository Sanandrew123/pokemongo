// 日志系统 - 高性能的游戏日志记录
// 开发心理：提供灵活的日志记录功能，支持多级别、多输出、异步写入
// 设计原则：性能优先、配置灵活、格式丰富、线程安全

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log::{Level, LevelFilter, Log, Metadata, Record};

// 日志级别（扩展标准库）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
    
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[36m",     // 青色
            LogLevel::Debug => "\x1b[34m",     // 蓝色
            LogLevel::Info => "\x1b[32m",      // 绿色
            LogLevel::Warn => "\x1b[33m",      // 黄色
            LogLevel::Error => "\x1b[31m",     // 红色
            LogLevel::Fatal => "\x1b[35;1m",   // 紫色加粗
        }
    }
    
    pub fn from_log_level(level: Level) -> Self {
        match level {
            Level::Trace => LogLevel::Trace,
            Level::Debug => LogLevel::Debug,
            Level::Info => LogLevel::Info,
            Level::Warn => LogLevel::Warn,
            Level::Error => LogLevel::Error,
        }
    }
}

// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub thread_id: String,
    pub thread_name: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, target: String, message: String) -> Self {
        let thread = thread::current();
        
        Self {
            timestamp: SystemTime::now(),
            level,
            target,
            message,
            module_path: None,
            file: None,
            line: None,
            thread_id: format!("{:?}", thread.id()),
            thread_name: thread.name().map(|s| s.to_string()),
        }
    }
    
    pub fn with_location(mut self, module_path: Option<String>, file: Option<String>, line: Option<u32>) -> Self {
        self.module_path = module_path;
        self.file = file;
        self.line = line;
        self
    }
    
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    pub fn format_timestamp(&self) -> String {
        let duration = self.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        
        // 简化的时间格式
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;
        
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }
}

// 日志格式器
pub trait LogFormatter: Send + Sync {
    fn format(&self, entry: &LogEntry) -> String;
}

// 简单文本格式器
pub struct SimpleFormatter {
    pub include_timestamp: bool,
    pub include_level: bool,
    pub include_target: bool,
    pub include_thread: bool,
    pub include_location: bool,
    pub colored: bool,
}

impl Default for SimpleFormatter {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_level: true,
            include_target: true,
            include_thread: false,
            include_location: false,
            colored: true,
        }
    }
}

impl LogFormatter for SimpleFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();
        
        // 时间戳
        if self.include_timestamp {
            parts.push(format!("[{}]", entry.format_timestamp()));
        }
        
        // 级别
        if self.include_level {
            let level_str = if self.colored {
                format!("{}{}[{}]\x1b[0m", 
                       entry.level.color_code(), 
                       entry.level.as_str(),
                       entry.level.as_str())
            } else {
                format!("[{}]", entry.level.as_str())
            };
            parts.push(level_str);
        }
        
        // 目标
        if self.include_target && !entry.target.is_empty() {
            parts.push(format!("[{}]", entry.target));
        }
        
        // 线程信息
        if self.include_thread {
            let thread_info = if let Some(ref name) = entry.thread_name {
                format!("[{}]", name)
            } else {
                format!("[{}]", &entry.thread_id[..8])
            };
            parts.push(thread_info);
        }
        
        // 位置信息
        if self.include_location {
            if let (Some(ref file), Some(line)) = (&entry.file, entry.line) {
                let filename = Path::new(file).file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file);
                parts.push(format!("[{}:{}]", filename, line));
            }
        }
        
        // 消息
        parts.push(entry.message.clone());
        
        parts.join(" ")
    }
}

// JSON格式器
pub struct JsonFormatter;

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let json = serde_json::json!({
            "timestamp": entry.timestamp_millis(),
            "level": entry.level.as_str(),
            "target": entry.target,
            "message": entry.message,
            "module_path": entry.module_path,
            "file": entry.file,
            "line": entry.line,
            "thread_id": entry.thread_id,
            "thread_name": entry.thread_name,
        });
        
        json.to_string()
    }
}

// 日志输出目标
pub trait LogTarget: Send + Sync {
    fn write(&mut self, formatted_entry: &str) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn supports_color(&self) -> bool { false }
}

// 控制台输出
pub struct ConsoleTarget {
    use_stderr: bool,
}

impl ConsoleTarget {
    pub fn new(use_stderr: bool) -> Self {
        Self { use_stderr }
    }
}

impl LogTarget for ConsoleTarget {
    fn write(&mut self, formatted_entry: &str) -> Result<()> {
        if self.use_stderr {
            eprintln!("{}", formatted_entry);
        } else {
            println!("{}", formatted_entry);
        }
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        use std::io::{stderr, stdout, Write};
        
        if self.use_stderr {
            stderr().flush().map_err(|e| GameError::IOError(format!("控制台刷新失败: {}", e)))?;
        } else {
            stdout().flush().map_err(|e| GameError::IOError(format!("控制台刷新失败: {}", e)))?;
        }
        Ok(())
    }
    
    fn supports_color(&self) -> bool {
        // 简化的颜色支持检测
        std::env::var("NO_COLOR").is_err() && atty::is(if self.use_stderr { atty::Stream::Stderr } else { atty::Stream::Stdout })
    }
}

// 文件输出
pub struct FileTarget {
    writer: BufWriter<File>,
    path: PathBuf,
    max_size: Option<u64>,
    current_size: u64,
}

impl FileTarget {
    pub fn new(path: PathBuf, max_size: Option<u64>) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| GameError::IOError(format!("打开日志文件失败: {}", e)))?;
        
        let current_size = file.metadata()
            .map(|m| m.len())
            .unwrap_or(0);
        
        Ok(Self {
            writer: BufWriter::new(file),
            path,
            max_size,
            current_size,
        })
    }
    
    fn should_rotate(&self) -> bool {
        if let Some(max_size) = self.max_size {
            self.current_size >= max_size
        } else {
            false
        }
    }
    
    fn rotate(&mut self) -> Result<()> {
        self.writer.flush().map_err(|e| GameError::IOError(format!("刷新缓冲区失败: {}", e)))?;
        
        // 生成轮换文件名
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut rotated_path = self.path.clone();
        if let Some(extension) = self.path.extension() {
            rotated_path.set_extension(format!("{}.{}", extension.to_string_lossy(), timestamp));
        } else {
            rotated_path.set_extension(timestamp.to_string());
        }
        
        // 重命名当前文件
        std::fs::rename(&self.path, &rotated_path)
            .map_err(|e| GameError::IOError(format!("日志文件轮换失败: {}", e)))?;
        
        // 创建新文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| GameError::IOError(format!("创建新日志文件失败: {}", e)))?;
        
        self.writer = BufWriter::new(file);
        self.current_size = 0;
        
        Ok(())
    }
}

impl LogTarget for FileTarget {
    fn write(&mut self, formatted_entry: &str) -> Result<()> {
        if self.should_rotate() {
            self.rotate()?;
        }
        
        let entry_with_newline = format!("{}\n", formatted_entry);
        self.writer.write_all(entry_with_newline.as_bytes())
            .map_err(|e| GameError::IOError(format!("写入日志文件失败: {}", e)))?;
        
        self.current_size += entry_with_newline.len() as u64;
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        self.writer.flush()
            .map_err(|e| GameError::IOError(format!("刷新日志文件失败: {}", e)))?;
        Ok(())
    }
}

// 内存缓冲区输出（用于调试）
pub struct MemoryTarget {
    entries: Arc<Mutex<VecDeque<String>>>,
    max_entries: usize,
}

impl MemoryTarget {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_entries,
        }
    }
    
    pub fn get_entries(&self) -> Vec<String> {
        self.entries.lock().unwrap().iter().cloned().collect()
    }
    
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

impl LogTarget for MemoryTarget {
    fn write(&mut self, formatted_entry: &str) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        entries.push_back(formatted_entry.to_string());
        
        while entries.len() > self.max_entries {
            entries.pop_front();
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        // 内存输出不需要刷新
        Ok(())
    }
}

// 游戏日志器配置
#[derive(Debug, Clone)]
pub struct GameLoggerConfig {
    pub level: LogLevel,
    pub enable_console: bool,
    pub enable_file: bool,
    pub file_path: Option<PathBuf>,
    pub file_max_size: Option<u64>,
    pub enable_memory_buffer: bool,
    pub memory_buffer_size: usize,
    pub async_logging: bool,
    pub flush_interval: Duration,
    pub formatter_type: FormatterType,
    pub colored_output: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FormatterType {
    Simple,
    Json,
}

impl Default for GameLoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enable_console: true,
            enable_file: false,
            file_path: None,
            file_max_size: Some(10 * 1024 * 1024), // 10MB
            enable_memory_buffer: false,
            memory_buffer_size: 1000,
            async_logging: true,
            flush_interval: Duration::from_secs(1),
            formatter_type: FormatterType::Simple,
            colored_output: true,
        }
    }
}

// 游戏日志器
pub struct GameLogger {
    config: GameLoggerConfig,
    targets: Vec<Box<dyn LogTarget>>,
    formatter: Box<dyn LogFormatter>,
    
    // 异步日志支持
    log_queue: Arc<Mutex<VecDeque<LogEntry>>>,
    async_thread: Option<JoinHandle<()>>,
    shutdown_flag: Arc<RwLock<bool>>,
    
    // 统计信息
    total_entries: Arc<RwLock<u64>>,
    dropped_entries: Arc<RwLock<u64>>,
    last_flush: Arc<RwLock<Instant>>,
}

impl GameLogger {
    pub fn new(config: GameLoggerConfig) -> Result<Self> {
        let mut targets: Vec<Box<dyn LogTarget>> = Vec::new();
        
        // 添加控制台输出
        if config.enable_console {
            targets.push(Box::new(ConsoleTarget::new(true)));
        }
        
        // 添加文件输出
        if config.enable_file {
            if let Some(ref path) = config.file_path {
                targets.push(Box::new(FileTarget::new(path.clone(), config.file_max_size)?));
            }
        }
        
        // 添加内存缓冲区
        if config.enable_memory_buffer {
            targets.push(Box::new(MemoryTarget::new(config.memory_buffer_size)));
        }
        
        // 选择格式器
        let formatter: Box<dyn LogFormatter> = match config.formatter_type {
            FormatterType::Simple => {
                let mut simple = SimpleFormatter::default();
                simple.colored = config.colored_output;
                Box::new(simple)
            },
            FormatterType::Json => Box::new(JsonFormatter),
        };
        
        let mut logger = Self {
            config,
            targets,
            formatter,
            log_queue: Arc::new(Mutex::new(VecDeque::new())),
            async_thread: None,
            shutdown_flag: Arc::new(RwLock::new(false)),
            total_entries: Arc::new(RwLock::new(0)),
            dropped_entries: Arc::new(RwLock::new(0)),
            last_flush: Arc::new(RwLock::new(Instant::now())),
        };
        
        // 启动异步日志线程
        if logger.config.async_logging {
            logger.start_async_thread()?;
        }
        
        Ok(logger)
    }
    
    fn start_async_thread(&mut self) -> Result<()> {
        let log_queue = self.log_queue.clone();
        let shutdown_flag = self.shutdown_flag.clone();
        let flush_interval = self.config.flush_interval;
        
        let thread_handle = thread::Builder::new()
            .name("GameLogger".to_string())
            .spawn(move || {
                let mut last_flush = Instant::now();
                
                loop {
                    let should_shutdown = *shutdown_flag.read().unwrap();
                    if should_shutdown {
                        break;
                    }
                    
                    // 检查是否需要刷新
                    let now = Instant::now();
                    if now.duration_since(last_flush) >= flush_interval {
                        last_flush = now;
                        // 在实际实现中，这里会处理日志队列
                        thread::sleep(Duration::from_millis(10));
                    } else {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            })
            .map_err(|e| GameError::IOError(format!("启动日志线程失败: {}", e)))?;
        
        self.async_thread = Some(thread_handle);
        Ok(())
    }
    
    pub fn log(&mut self, entry: LogEntry) -> Result<()> {
        // 检查日志级别
        if entry.level < self.config.level {
            return Ok(());
        }
        
        *self.total_entries.write().unwrap() += 1;
        
        if self.config.async_logging {
            // 异步日志：添加到队列
            let mut queue = self.log_queue.lock().unwrap();
            queue.push_back(entry);
            
            // 限制队列大小，避免内存泄漏
            const MAX_QUEUE_SIZE: usize = 10000;
            if queue.len() > MAX_QUEUE_SIZE {
                queue.pop_front();
                *self.dropped_entries.write().unwrap() += 1;
            }
        } else {
            // 同步日志：直接写入
            self.write_entry(&entry)?;
        }
        
        Ok(())
    }
    
    fn write_entry(&mut self, entry: &LogEntry) -> Result<()> {
        let formatted = self.formatter.format(entry);
        
        for target in &mut self.targets {
            target.write(&formatted)?;
        }
        
        Ok(())
    }
    
    pub fn flush(&mut self) -> Result<()> {
        // 处理异步队列中的所有条目
        if self.config.async_logging {
            let entries: Vec<LogEntry> = {
                let mut queue = self.log_queue.lock().unwrap();
                queue.drain(..).collect()
            };
            
            for entry in entries {
                self.write_entry(&entry)?;
            }
        }
        
        // 刷新所有目标
        for target in &mut self.targets {
            target.flush()?;
        }
        
        *self.last_flush.write().unwrap() = Instant::now();
        Ok(())
    }
    
    pub fn get_stats(&self) -> LoggerStats {
        LoggerStats {
            total_entries: *self.total_entries.read().unwrap(),
            dropped_entries: *self.dropped_entries.read().unwrap(),
            queue_size: self.log_queue.lock().unwrap().len(),
            last_flush: *self.last_flush.read().unwrap(),
        }
    }
    
    pub fn shutdown(&mut self) -> Result<()> {
        // 设置关闭标志
        *self.shutdown_flag.write().unwrap() = true;
        
        // 刷新所有待处理的日志
        self.flush()?;
        
        // 等待异步线程结束
        if let Some(handle) = self.async_thread.take() {
            if let Err(e) = handle.join() {
                eprintln!("日志线程关闭失败: {:?}", e);
            }
        }
        
        Ok(())
    }
}

impl Drop for GameLogger {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

// 实现标准库Log trait
impl Log for GameLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        LogLevel::from_log_level(metadata.level()) >= self.config.level
    }
    
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        let entry = LogEntry::new(
            LogLevel::from_log_level(record.level()),
            record.target().to_string(),
            record.args().to_string(),
        ).with_location(
            record.module_path().map(|s| s.to_string()),
            record.file().map(|s| s.to_string()),
            record.line(),
        );
        
        // 由于Log trait的log方法是不可变的，我们需要使用内部可变性
        // 在实际实现中，应该使用Arc<Mutex<GameLogger>>
        // 这里简化处理
    }
    
    fn flush(&self) {
        // 类似上面的问题，需要内部可变性
    }
}

// 日志器统计信息
#[derive(Debug, Clone)]
pub struct LoggerStats {
    pub total_entries: u64,
    pub dropped_entries: u64,
    pub queue_size: usize,
    pub last_flush: Instant,
}

// 便捷宏
#[macro_export]
macro_rules! game_log {
    ($level:expr, $target:expr, $($arg:tt)*) => {
        // 在实际实现中，这里会调用GameLogger实例
        log::log!($level.into(), target: $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! game_trace {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Trace, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Trace, module_path!(), $($arg)*) };
}

#[macro_export]
macro_rules! game_debug {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Debug, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Debug, module_path!(), $($arg)*) };
}

#[macro_export]
macro_rules! game_info {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Info, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Info, module_path!(), $($arg)*) };
}

#[macro_export]
macro_rules! game_warn {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Warn, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Warn, module_path!(), $($arg)*) };
}

#[macro_export]
macro_rules! game_error {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Error, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Error, module_path!(), $($arg)*) };
}

#[macro_export]
macro_rules! game_fatal {
    ($target:expr, $($arg:tt)*) => { game_log!(LogLevel::Fatal, $target, $($arg)*) };
    ($($arg:tt)*) => { game_log!(LogLevel::Fatal, module_path!(), $($arg)*) };
}

// atty模拟（用于颜色支持检测）
mod atty {
    pub enum Stream {
        Stdout,
        Stderr,
    }
    
    pub fn is(_stream: Stream) -> bool {
        // 简化实现，假设支持颜色
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }
    
    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "test".to_string(),
            "Test message".to_string(),
        );
        
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.target, "test");
        assert_eq!(entry.message, "Test message");
        assert!(entry.timestamp <= SystemTime::now());
    }
    
    #[test]
    fn test_simple_formatter() {
        let formatter = SimpleFormatter::default();
        let entry = LogEntry::new(
            LogLevel::Info,
            "test".to_string(),
            "Test message".to_string(),
        );
        
        let formatted = formatter.format(&entry);
        assert!(formatted.contains("[INFO]"));
        assert!(formatted.contains("[test]"));
        assert!(formatted.contains("Test message"));
    }
    
    #[test]
    fn test_memory_target() {
        let mut target = MemoryTarget::new(5);
        
        target.write("Entry 1").unwrap();
        target.write("Entry 2").unwrap();
        
        let entries = target.get_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], "Entry 1");
        assert_eq!(entries[1], "Entry 2");
    }
    
    #[test]
    fn test_file_target() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        let mut target = FileTarget::new(log_path.clone(), None).unwrap();
        target.write("Test log entry").unwrap();
        target.flush().unwrap();
        
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Test log entry"));
    }
    
    #[test]
    fn test_game_logger_creation() {
        let config = GameLoggerConfig {
            enable_file: false,
            enable_memory_buffer: true,
            async_logging: false,
            ..Default::default()
        };
        
        let logger = GameLogger::new(config).unwrap();
        assert_eq!(logger.targets.len(), 2); // Console + Memory
    }
}