// 工具模块 - 通用工具和辅助函数
// 开发心理：提供游戏开发中常用的工具函数，保持代码的可重用性和简洁性
// 设计原则：模块化、高效、易用、跨平台

pub mod logger;
// 暂时注释掉未实现的子模块，避免编译错误
// pub mod math;
// pub mod random;
// pub mod timer;

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log::{info, debug, warn, error};

// 颜色结构体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}

pub use logger::*;
// 暂时注释掉未实现的模块导出
// pub use math::*;
// pub use random::*;
// pub use timer::*;

// 版本信息结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }
    
    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
    
    pub fn to_string(&self) -> String {
        match &self.build {
            Some(build) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, build),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
    
    pub fn compare(&self, other: &Version) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
    
    pub fn is_compatible(&self, other: &Version) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// 性能监控器
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    frame_times: Vec<Duration>,
    max_samples: usize,
    start_time: Instant,
    frame_count: u64,
    
    // 性能计数器
    counters: HashMap<String, PerformanceCounter>,
}

#[derive(Debug, Clone)]
struct PerformanceCounter {
    total_time: Duration,
    count: u64,
    min_time: Duration,
    max_time: Duration,
}

impl PerformanceCounter {
    fn new() -> Self {
        Self {
            total_time: Duration::ZERO,
            count: 0,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
        }
    }
    
    fn record(&mut self, duration: Duration) {
        self.total_time += duration;
        self.count += 1;
        self.min_time = self.min_time.min(duration);
        self.max_time = self.max_time.max(duration);
    }
    
    fn average(&self) -> Duration {
        if self.count == 0 {
            Duration::ZERO
        } else {
            self.total_time / self.count as u32
        }
    }
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
            start_time: Instant::now(),
            frame_count: 0,
            counters: HashMap::new(),
        }
    }
    
    // 记录帧时间
    pub fn record_frame(&mut self, frame_time: Duration) {
        self.frame_times.push(frame_time);
        self.frame_count += 1;
        
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }
    
    // 获取当前FPS
    pub fn get_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let total_time: Duration = self.frame_times.iter().sum();
        if total_time.is_zero() {
            return 0.0;
        }
        
        let avg_frame_time = total_time.as_secs_f64() / self.frame_times.len() as f64;
        1.0 / avg_frame_time
    }
    
    // 获取平均帧时间
    pub fn get_average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }
    
    // 获取最小/最大帧时间
    pub fn get_frame_time_range(&self) -> (Duration, Duration) {
        if self.frame_times.is_empty() {
            return (Duration::ZERO, Duration::ZERO);
        }
        
        let min = *self.frame_times.iter().min().unwrap();
        let max = *self.frame_times.iter().max().unwrap();
        (min, max)
    }
    
    // 记录性能计数器
    pub fn record_counter(&mut self, name: &str, duration: Duration) {
        let counter = self.counters.entry(name.to_string()).or_insert_with(PerformanceCounter::new);
        counter.record(duration);
    }
    
    // 获取计数器统计
    pub fn get_counter_stats(&self, name: &str) -> Option<(Duration, Duration, Duration, u64)> {
        self.counters.get(name).map(|counter| {
            (counter.average(), counter.min_time, counter.max_time, counter.count)
        })
    }
    
    // 获取总运行时间
    pub fn get_total_runtime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    // 获取总帧数
    pub fn get_total_frames(&self) -> u64 {
        self.frame_count
    }
    
    // 重置统计信息
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.start_time = Instant::now();
        self.frame_count = 0;
        self.counters.clear();
    }
    
    // 生成性能报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("=== 性能监控报告 ===\n"));
        report.push_str(&format!("运行时间: {:.2}s\n", self.get_total_runtime().as_secs_f64()));
        report.push_str(&format!("总帧数: {}\n", self.frame_count));
        report.push_str(&format!("当前FPS: {:.1}\n", self.get_fps()));
        report.push_str(&format!("平均帧时间: {:.2}ms\n", self.get_average_frame_time().as_secs_f64() * 1000.0));
        
        let (min_frame, max_frame) = self.get_frame_time_range();
        report.push_str(&format!("帧时间范围: {:.2}ms - {:.2}ms\n", 
                                min_frame.as_secs_f64() * 1000.0, 
                                max_frame.as_secs_f64() * 1000.0));
        
        if !self.counters.is_empty() {
            report.push_str("\n=== 性能计数器 ===\n");
            for (name, counter) in &self.counters {
                report.push_str(&format!("{}: 平均 {:.3}ms, 最小 {:.3}ms, 最大 {:.3}ms, 次数 {}\n",
                                        name,
                                        counter.average().as_secs_f64() * 1000.0,
                                        counter.min_time.as_secs_f64() * 1000.0,
                                        counter.max_time.as_secs_f64() * 1000.0,
                                        counter.count));
            }
        }
        
        report
    }
}

// 内存使用统计
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub allocated_bytes: u64,
    pub peak_allocated: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

impl MemoryStats {
    pub fn record_allocation(&mut self, size: u64) {
        self.allocated_bytes += size;
        self.allocation_count += 1;
        self.peak_allocated = self.peak_allocated.max(self.allocated_bytes);
    }
    
    pub fn record_deallocation(&mut self, size: u64) {
        self.allocated_bytes = self.allocated_bytes.saturating_sub(size);
        self.deallocation_count += 1;
    }
    
    pub fn get_allocated_mb(&self) -> f64 {
        self.allocated_bytes as f64 / (1024.0 * 1024.0)
    }
    
    pub fn get_peak_mb(&self) -> f64 {
        self.peak_allocated as f64 / (1024.0 * 1024.0)
    }
}

// 字符串工具
pub struct StringUtils;

impl StringUtils {
    // 安全的字符串截断
    pub fn truncate(s: &str, max_chars: usize) -> &str {
        match s.char_indices().nth(max_chars) {
            None => s,
            Some((idx, _)) => &s[..idx],
        }
    }
    
    // 移除多余的空白字符
    pub fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    // 驼峰命名转蛇形命名
    pub fn camel_to_snake_case(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_lowercase() {
                        result.push('_');
                    }
                } else {
                    result.push('_');
                }
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        
        result
    }
    
    // 蛇形命名转驼峰命名
    pub fn snake_to_camel_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;
        
        for ch in s.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    // 计算字符串相似度（简化的编辑距离）
    pub fn similarity(s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        let distance = Self::edit_distance(s1, s2);
        let max_len = len1.max(len2);
        
        1.0 - (distance as f64 / max_len as f64)
    }
    
    fn edit_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<_> = s1.chars().collect();
        let chars2: Vec<_> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            dp[i][0] = i;
        }
        
        for j in 0..=len2 {
            dp[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i-1] == chars2[j-1] { 0 } else { 1 };
                
                dp[i][j] = (dp[i-1][j] + 1)
                    .min(dp[i][j-1] + 1)
                    .min(dp[i-1][j-1] + cost);
            }
        }
        
        dp[len1][len2]
    }
}

// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    // 验证数值范围
    pub fn validate_range<T: PartialOrd + Copy + std::fmt::Debug>(value: T, min: T, max: T, name: &str) -> Result<T> {
        if value < min || value > max {
            Err(GameError::ConfigError(format!("{} 超出范围 [{:?}, {:?}]", name, min, max)))
        } else {
            Ok(value)
        }
    }
    
    // 验证端口号
    pub fn validate_port(port: u16) -> Result<u16> {
        if port < 1024 || port > 65535 {
            Err(GameError::ConfigError(format!("端口号 {} 无效，应该在 1024-65535 范围内", port)))
        } else {
            Ok(port)
        }
    }
    
    // 验证文件路径
    pub fn validate_file_path(path: &str) -> Result<String> {
        let path_buf = std::path::Path::new(path);
        if !path_buf.exists() {
            Err(GameError::ConfigError(format!("文件不存在: {}", path)))
        } else if !path_buf.is_file() {
            Err(GameError::ConfigError(format!("不是文件: {}", path)))
        } else {
            Ok(path.to_string())
        }
    }
    
    // 验证目录路径
    pub fn validate_directory_path(path: &str) -> Result<String> {
        let path_buf = std::path::Path::new(path);
        if !path_buf.exists() {
            Err(GameError::ConfigError(format!("目录不存在: {}", path)))
        } else if !path_buf.is_dir() {
            Err(GameError::ConfigError(format!("不是目录: {}", path)))
        } else {
            Ok(path.to_string())
        }
    }
    
    // 验证URL格式
    pub fn validate_url(url: &str) -> Result<String> {
        // 简化的URL验证
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(url.to_string())
        } else {
            Err(GameError::ConfigError(format!("无效的URL格式: {}", url)))
        }
    }
}

// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub memory_total: u64, // bytes
    pub memory_available: u64, // bytes
    pub username: Option<String>,
    pub hostname: Option<String>,
}

impl SystemInfo {
    pub fn gather() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: num_cpus(),
            memory_total: Self::get_total_memory(),
            memory_available: Self::get_available_memory(),
            username: std::env::var("USERNAME").or_else(|_| std::env::var("USER")).ok(),
            hostname: Self::get_hostname(),
        }
    }
    
    fn get_total_memory() -> u64 {
        // 简化实现，实际项目中应使用系统API
        16 * 1024 * 1024 * 1024 // 假设16GB
    }
    
    fn get_available_memory() -> u64 {
        // 简化实现
        8 * 1024 * 1024 * 1024 // 假设8GB可用
    }
    
    fn get_hostname() -> Option<String> {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .ok()
    }
    
    pub fn memory_total_gb(&self) -> f64 {
        self.memory_total as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    pub fn memory_available_gb(&self) -> f64 {
        self.memory_available as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    pub fn memory_usage_percent(&self) -> f64 {
        if self.memory_total == 0 {
            0.0
        } else {
            let used = self.memory_total - self.memory_available;
            used as f64 / self.memory_total as f64 * 100.0
        }
    }
}

// 唯一ID生成器
pub struct IdGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
    start_time: SystemTime,
}

impl IdGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
            prefix: prefix.to_string(),
            start_time: SystemTime::now(),
        }
    }
    
    // 生成数字ID
    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    
    // 生成字符串ID
    pub fn next_string_id(&self) -> String {
        let id = self.next_id();
        format!("{}_{}", self.prefix, id)
    }
    
    // 生成时间戳ID
    pub fn next_timestamp_id(&self) -> String {
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
        let id = self.next_id();
        format!("{}_{}{}", self.prefix, timestamp, id)
    }
    
    // 生成UUID风格的ID
    pub fn next_uuid_like_id(&self) -> String {
        let id = self.next_id();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let random_part = timestamp & 0xFFFFFF;
        
        format!("{}-{:08x}-{:06x}", self.prefix, id, random_part)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new("id")
    }
}

// 缓存清理策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheEvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In First Out
    Random,
}

// 环形缓冲区
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: (0..capacity).map(|_| None).collect(),
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        }
    }
    
    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = self.buffer[self.tail].take();
        self.buffer[self.tail] = Some(item);
        
        self.tail = (self.tail + 1) % self.capacity;
        
        if self.size < self.capacity {
            self.size += 1;
        } else {
            self.head = (self.head + 1) % self.capacity;
        }
        
        old_item
    }
    
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.size -= 1;
        
        item
    }
    
    pub fn len(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }
}

// 简单的CPU核心数检测
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

// 重复的Color定义已删除

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);
        let v3 = Version::new(2, 0, 0);
        
        assert!(v1.compare(&v2) == std::cmp::Ordering::Less);
        assert!(v2.compare(&v3) == std::cmp::Ordering::Less);
        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(10);
        
        monitor.record_frame(Duration::from_millis(16)); // ~60 FPS
        monitor.record_frame(Duration::from_millis(33)); // ~30 FPS
        
        let fps = monitor.get_fps();
        assert!(fps > 40.0 && fps < 50.0); // 应该在30-60之间
    }
    
    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::camel_to_snake_case("CamelCase"), "camel_case");
        assert_eq!(StringUtils::snake_to_camel_case("snake_case"), "snakeCase");
        assert!(StringUtils::similarity("hello", "hello") == 1.0);
        assert!(StringUtils::similarity("hello", "world") < 1.0);
    }
    
    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        
        assert_eq!(buffer.push(1), None);
        assert_eq!(buffer.push(2), None);
        assert_eq!(buffer.push(3), None);
        assert_eq!(buffer.push(4), Some(1)); // 溢出，返回被覆盖的值
        
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.len(), 2);
    }
    
    #[test]
    fn test_color() {
        let red = Color::RED;
        let blue = Color::BLUE;
        let purple = red.lerp(&blue, 0.5);
        
        assert_eq!(purple.r, 0.5);
        assert_eq!(purple.b, 0.5);
        assert_eq!(purple.g, 0.0);
        
        let hex_color = Color::from_hex(0xFF0000FF); // 红色
        assert_eq!(hex_color.r, 1.0);
        assert_eq!(hex_color.g, 0.0);
        assert_eq!(hex_color.to_hex(), 0xFF0000FF);
    }
    
    #[test]
    fn test_id_generator() {
        let generator = IdGenerator::new("test");
        
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        
        assert!(id2 > id1);
        
        let str_id = generator.next_string_id();
        assert!(str_id.starts_with("test_"));
    }
}