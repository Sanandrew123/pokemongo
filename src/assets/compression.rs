// 资源压缩系统 - 高效的数据压缩和解压缩
// 开发心理：通过压缩减少内存使用和加载时间，支持多种压缩算法
// 设计原则：算法选择、压缩率优化、解压速度、内存友好

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write, Cursor};
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};

// 压缩算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionType {
    None,       // 无压缩
    LZ4,        // 快速压缩/解压
    Zlib,       // 平衡的压缩率和速度
    Zstd,       // 现代高效压缩
    Brotli,     // 高压缩率（适合文本）
    Snappy,     // Google的快速压缩
}

impl CompressionType {
    // 获取算法特性
    pub fn characteristics(&self) -> CompressionCharacteristics {
        match self {
            CompressionType::None => CompressionCharacteristics {
                compression_speed: 10,
                decompression_speed: 10,
                compression_ratio: 1.0,
                memory_usage: 1,
                best_for: "Raw data, already compressed content".to_string(),
            },
            CompressionType::LZ4 => CompressionCharacteristics {
                compression_speed: 9,
                decompression_speed: 10,
                compression_ratio: 2.5,
                memory_usage: 2,
                best_for: "Real-time compression, game assets".to_string(),
            },
            CompressionType::Zlib => CompressionCharacteristics {
                compression_speed: 6,
                decompression_speed: 8,
                compression_ratio: 3.5,
                memory_usage: 3,
                best_for: "General purpose, network data".to_string(),
            },
            CompressionType::Zstd => CompressionCharacteristics {
                compression_speed: 7,
                decompression_speed: 9,
                compression_ratio: 4.0,
                memory_usage: 3,
                best_for: "Modern applications, large datasets".to_string(),
            },
            CompressionType::Brotli => CompressionCharacteristics {
                compression_speed: 4,
                decompression_speed: 7,
                compression_ratio: 4.5,
                memory_usage: 4,
                best_for: "Text files, web content, JSON".to_string(),
            },
            CompressionType::Snappy => CompressionCharacteristics {
                compression_speed: 8,
                decompression_speed: 9,
                compression_ratio: 2.8,
                memory_usage: 2,
                best_for: "High-frequency compression, logs".to_string(),
            },
        }
    }
    
    // 根据数据类型推荐压缩算法
    pub fn recommend_for_data(data: &[u8]) -> Self {
        let size = data.len();
        
        // 小文件使用快速算法
        if size < 1024 {
            return CompressionType::LZ4;
        }
        
        // 检测数据类型
        let entropy = calculate_entropy(data);
        let text_ratio = detect_text_ratio(data);
        
        if text_ratio > 0.8 {
            // 文本数据，使用Brotli
            CompressionType::Brotli
        } else if entropy < 6.0 {
            // 低熵数据（重复性高），使用高压缩率算法
            CompressionType::Zstd
        } else if size > 10 * 1024 * 1024 {
            // 大文件，平衡压缩率和速度
            CompressionType::Zstd
        } else {
            // 默认使用LZ4
            CompressionType::LZ4
        }
    }
}

// 压缩算法特性
#[derive(Debug, Clone)]
pub struct CompressionCharacteristics {
    pub compression_speed: u8,    // 1-10, 10最快
    pub decompression_speed: u8,  // 1-10, 10最快
    pub compression_ratio: f64,   // 典型压缩比
    pub memory_usage: u8,         // 1-5, 内存使用量
    pub best_for: String,         // 适用场景
}

// 压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub algorithm: CompressionType,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compression_time: Duration,
    pub decompression_time: Option<Duration>,
    pub checksum: u32,
}

impl CompressionResult {
    pub fn space_savings(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            let saved = self.original_size - self.compressed_size;
            saved as f64 / self.original_size as f64 * 100.0
        }
    }
    
    pub fn compression_speed_mbps(&self) -> f64 {
        if self.compression_time.is_zero() {
            0.0
        } else {
            let mb = self.original_size as f64 / (1024.0 * 1024.0);
            mb / self.compression_time.as_secs_f64()
        }
    }
}

// 压缩配置
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub level: i32,               // 压缩等级（算法相关）
    pub window_size: Option<i32>, // 窗口大小
    pub dictionary: Option<Vec<u8>>, // 预定义字典
    pub verify_integrity: bool,   // 验证数据完整性
    pub enable_checksum: bool,    // 启用校验和
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 6, // 中等压缩级别
            window_size: None,
            dictionary: None,
            verify_integrity: true,
            enable_checksum: true,
        }
    }
}

// 压缩器特征
pub trait Compressor: Send + Sync {
    fn compress(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>>;
    fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>>;
    fn get_algorithm(&self) -> CompressionType;
    fn estimate_compressed_size(&self, original_size: usize) -> usize;
}

// LZ4压缩器（简化实现）
pub struct LZ4Compressor;

impl Compressor for LZ4Compressor {
    fn compress(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>> {
        // 简化的LZ4实现 - 实际项目中应使用lz4库
        let mut compressed = Vec::new();
        
        // LZ4头部
        compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
        
        // 简单的重复字符压缩
        let mut i = 0;
        while i < data.len() {
            let byte = data[i];
            let mut count = 1;
            
            // 计算重复次数
            while i + count < data.len() && data[i + count] == byte && count < 255 {
                count += 1;
            }
            
            if count > 3 {
                // 压缩重复数据
                compressed.push(0xFF); // 特殊标记
                compressed.push(byte);
                compressed.push(count as u8);
            } else {
                // 直接存储
                for _ in 0..count {
                    compressed.push(byte);
                }
            }
            
            i += count;
        }
        
        Ok(compressed)
    }
    
    fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        if compressed.len() < 4 {
            return Err(GameError::CompressionError("LZ4数据头部不完整".to_string()));
        }
        
        // 读取原始大小
        let original_size = u32::from_le_bytes([
            compressed[0], compressed[1], compressed[2], compressed[3]
        ]) as usize;
        
        let mut decompressed = Vec::with_capacity(original_size);
        let mut i = 4;
        
        while i < compressed.len() {
            if compressed[i] == 0xFF && i + 2 < compressed.len() {
                // 解压缩重复数据
                let byte = compressed[i + 1];
                let count = compressed[i + 2] as usize;
                
                for _ in 0..count {
                    decompressed.push(byte);
                }
                
                i += 3;
            } else {
                // 直接复制
                decompressed.push(compressed[i]);
                i += 1;
            }
        }
        
        Ok(decompressed)
    }
    
    fn get_algorithm(&self) -> CompressionType {
        CompressionType::LZ4
    }
    
    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        // LZ4通常压缩率较低，但速度快
        (original_size as f64 * 0.6) as usize
    }
}

// Zlib压缩器（简化实现）
pub struct ZlibCompressor;

impl Compressor for ZlibCompressor {
    fn compress(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>> {
        // 简化的Zlib实现 - 实际项目中应使用flate2库
        use std::io::Write;
        
        let mut compressed = Vec::new();
        
        // Zlib头部（简化）
        compressed.write_all(&[0x78, 0x9C]).unwrap();
        
        // 简单的字节频率压缩
        let mut frequency = [0u32; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }
        
        // 存储频率表（简化）
        for &freq in &frequency {
            compressed.write_all(&(freq as u16).to_le_bytes()).unwrap();
        }
        
        // 使用最频繁字节的索引替代
        let most_frequent_byte = frequency
            .iter()
            .enumerate()
            .max_by_key(|(_, &freq)| freq)
            .map(|(idx, _)| idx as u8)
            .unwrap_or(0);
        
        compressed.push(most_frequent_byte);
        
        // 简单替换编码
        for &byte in data {
            if byte == most_frequent_byte {
                compressed.push(0xFF); // 特殊标记
            } else {
                compressed.push(byte);
            }
        }
        
        Ok(compressed)
    }
    
    fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        if compressed.len() < 514 { // 2 + 256*2
            return Err(GameError::CompressionError("Zlib数据不完整".to_string()));
        }
        
        // 跳过头部
        let mut i = 2;
        
        // 读取频率表
        let mut frequency = [0u32; 256];
        for freq in &mut frequency {
            *freq = u16::from_le_bytes([compressed[i], compressed[i + 1]]) as u32;
            i += 2;
        }
        
        let most_frequent_byte = compressed[i];
        i += 1;
        
        // 解压缩数据
        let mut decompressed = Vec::new();
        while i < compressed.len() {
            if compressed[i] == 0xFF {
                decompressed.push(most_frequent_byte);
            } else {
                decompressed.push(compressed[i]);
            }
            i += 1;
        }
        
        Ok(decompressed)
    }
    
    fn get_algorithm(&self) -> CompressionType {
        CompressionType::Zlib
    }
    
    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        // Zlib通常有较好的压缩率
        (original_size as f64 * 0.4) as usize
    }
}

// 无压缩器
pub struct NoCompressor;

impl Compressor for NoCompressor {
    fn compress(&self, data: &[u8], _config: &CompressionConfig) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
    
    fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        Ok(compressed.to_vec())
    }
    
    fn get_algorithm(&self) -> CompressionType {
        CompressionType::None
    }
    
    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        original_size
    }
}

// 压缩管理器
pub struct CompressionManager {
    compressors: HashMap<CompressionType, Box<dyn Compressor>>,
    stats: CompressionStats,
    default_config: CompressionConfig,
}

#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    pub total_compressions: u64,
    pub total_decompressions: u64,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
    pub total_compression_time: Duration,
    pub total_decompression_time: Duration,
    pub algorithm_usage: HashMap<CompressionType, u64>,
}

impl CompressionStats {
    pub fn overall_compression_ratio(&self) -> f64 {
        if self.total_original_bytes == 0 {
            1.0
        } else {
            self.total_compressed_bytes as f64 / self.total_original_bytes as f64
        }
    }
    
    pub fn space_savings(&self) -> f64 {
        (1.0 - self.overall_compression_ratio()) * 100.0
    }
    
    pub fn average_compression_speed(&self) -> f64 {
        if self.total_compression_time.is_zero() {
            0.0
        } else {
            let mb = self.total_original_bytes as f64 / (1024.0 * 1024.0);
            mb / self.total_compression_time.as_secs_f64()
        }
    }
    
    pub fn average_decompression_speed(&self) -> f64 {
        if self.total_decompression_time.is_zero() {
            0.0
        } else {
            let mb = self.total_compressed_bytes as f64 / (1024.0 * 1024.0);
            mb / self.total_decompression_time.as_secs_f64()
        }
    }
}

impl CompressionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            compressors: HashMap::new(),
            stats: CompressionStats::default(),
            default_config: CompressionConfig::default(),
        };
        
        // 注册默认压缩器
        manager.register_compressor(Box::new(NoCompressor));
        manager.register_compressor(Box::new(LZ4Compressor));
        manager.register_compressor(Box::new(ZlibCompressor));
        
        manager
    }
    
    // 注册压缩器
    pub fn register_compressor(&mut self, compressor: Box<dyn Compressor>) {
        let algorithm = compressor.get_algorithm();
        self.compressors.insert(algorithm, compressor);
        self.stats.algorithm_usage.insert(algorithm, 0);
        debug!("注册压缩器: {:?}", algorithm);
    }
    
    // 压缩数据
    pub fn compress(&mut self, data: &[u8], algorithm: CompressionType) -> Result<CompressionResult> {
        self.compress_with_config(data, algorithm, &self.default_config.clone())
    }
    
    pub fn compress_with_config(
        &mut self, 
        data: &[u8], 
        algorithm: CompressionType, 
        config: &CompressionConfig
    ) -> Result<CompressionResult> {
        let compressor = self.compressors.get(&algorithm)
            .ok_or_else(|| GameError::CompressionError(format!("不支持的压缩算法: {:?}", algorithm)))?;
        
        let start_time = Instant::now();
        let compressed = compressor.compress(data, config)?;
        let compression_time = start_time.elapsed();
        
        let checksum = if config.enable_checksum {
            calculate_checksum(data)
        } else {
            0
        };
        
        let result = CompressionResult {
            algorithm,
            original_size: data.len(),
            compressed_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / data.len() as f64,
            compression_time,
            decompression_time: None,
            checksum,
        };
        
        // 更新统计信息
        self.update_compression_stats(&result, &compressed);
        
        debug!("压缩完成: {:?}, 原始: {} bytes, 压缩后: {} bytes, 压缩率: {:.2}%, 耗时: {:?}",
               algorithm, result.original_size, result.compressed_size, 
               result.space_savings(), result.compression_time);
        
        Ok(result)
    }
    
    // 解压数据
    pub fn decompress(&mut self, compressed_data: &[u8], algorithm: CompressionType) -> Result<Vec<u8>> {
        let compressor = self.compressors.get(&algorithm)
            .ok_or_else(|| GameError::CompressionError(format!("不支持的压缩算法: {:?}", algorithm)))?;
        
        let start_time = Instant::now();
        let decompressed = compressor.decompress(compressed_data)?;
        let decompression_time = start_time.elapsed();
        
        // 更新统计信息
        self.stats.total_decompressions += 1;
        self.stats.total_decompression_time += decompression_time;
        *self.stats.algorithm_usage.get_mut(&algorithm).unwrap() += 1;
        
        debug!("解压完成: {:?}, 压缩数据: {} bytes, 解压后: {} bytes, 耗时: {:?}",
               algorithm, compressed_data.len(), decompressed.len(), decompression_time);
        
        Ok(decompressed)
    }
    
    // 自动选择最佳压缩算法
    pub fn compress_auto(&mut self, data: &[u8]) -> Result<(Vec<u8>, CompressionResult)> {
        let recommended_algorithm = CompressionType::recommend_for_data(data);
        let result = self.compress(data, recommended_algorithm)?;
        let compressed = self.get_last_compressed_data()?;
        
        Ok((compressed, result))
    }
    
    // 基准测试不同算法
    pub fn benchmark_algorithms(&mut self, data: &[u8]) -> Vec<CompressionResult> {
        let mut results = Vec::new();
        
        for &algorithm in self.compressors.keys() {
            if let Ok(result) = self.compress(data, algorithm) {
                results.push(result);
            }
        }
        
        // 按压缩率排序
        results.sort_by(|a, b| a.compression_ratio.partial_cmp(&b.compression_ratio).unwrap());
        
        info!("压缩基准测试完成，测试了 {} 种算法", results.len());
        for result in &results {
            info!("  {:?}: {:.1}% 空间节省, {:.2} MB/s",
                  result.algorithm, result.space_savings(), result.compression_speed_mbps());
        }
        
        results
    }
    
    // 批量压缩
    pub fn compress_batch(
        &mut self, 
        data_list: &[(&str, &[u8])], 
        algorithm: CompressionType
    ) -> Vec<(String, Result<CompressionResult>)> {
        let mut results = Vec::new();
        
        for (name, data) in data_list {
            let result = self.compress(data, algorithm);
            results.push((name.to_string(), result));
        }
        
        let successful = results.iter().filter(|(_, r)| r.is_ok()).count();
        info!("批量压缩完成: {}/{} 成功", successful, results.len());
        
        results
    }
    
    // 验证压缩数据完整性
    pub fn verify_integrity(
        &mut self, 
        original: &[u8], 
        compressed: &[u8], 
        algorithm: CompressionType
    ) -> Result<bool> {
        let decompressed = self.decompress(compressed, algorithm)?;
        Ok(original == decompressed.as_slice())
    }
    
    // 估算压缩后大小
    pub fn estimate_compressed_size(&self, data: &[u8], algorithm: CompressionType) -> Option<usize> {
        self.compressors.get(&algorithm)
            .map(|compressor| compressor.estimate_compressed_size(data.len()))
    }
    
    // 获取支持的算法列表
    pub fn get_supported_algorithms(&self) -> Vec<CompressionType> {
        self.compressors.keys().copied().collect()
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }
    
    // 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = CompressionStats::default();
        for &algorithm in self.compressors.keys() {
            self.stats.algorithm_usage.insert(algorithm, 0);
        }
    }
    
    // 设置默认配置
    pub fn set_default_config(&mut self, config: CompressionConfig) {
        self.default_config = config;
    }
    
    fn update_compression_stats(&mut self, result: &CompressionResult, compressed_data: &[u8]) {
        self.stats.total_compressions += 1;
        self.stats.total_original_bytes += result.original_size as u64;
        self.stats.total_compressed_bytes += compressed_data.len() as u64;
        self.stats.total_compression_time += result.compression_time;
        *self.stats.algorithm_usage.get_mut(&result.algorithm).unwrap() += 1;
    }
    
    // 简化实现：获取最后压缩的数据
    fn get_last_compressed_data(&self) -> Result<Vec<u8>> {
        // 实际实现中应该保存压缩结果
        Err(GameError::CompressionError("未实现".to_string()))
    }
    
    // 导出压缩报告
    pub fn export_report(&self) -> String {
        let stats = &self.stats;
        
        format!(
            "=== 压缩系统报告 ===\n\
             总压缩次数: {}\n\
             总解压次数: {}\n\
             处理数据量: {:.2} MB (原始) -> {:.2} MB (压缩)\n\
             整体压缩率: {:.3}\n\
             空间节省: {:.1}%\n\
             平均压缩速度: {:.2} MB/s\n\
             平均解压速度: {:.2} MB/s\n\
             \n算法使用统计:\n{}",
            stats.total_compressions,
            stats.total_decompressions,
            stats.total_original_bytes as f64 / (1024.0 * 1024.0),
            stats.total_compressed_bytes as f64 / (1024.0 * 1024.0),
            stats.overall_compression_ratio(),
            stats.space_savings(),
            stats.average_compression_speed(),
            stats.average_decompression_speed(),
            self.format_algorithm_usage()
        )
    }
    
    fn format_algorithm_usage(&self) -> String {
        let mut usage_report = String::new();
        let total_usage: u64 = self.stats.algorithm_usage.values().sum();
        
        for (&algorithm, &count) in &self.stats.algorithm_usage {
            if count > 0 {
                let percentage = if total_usage > 0 {
                    count as f64 / total_usage as f64 * 100.0
                } else {
                    0.0
                };
                
                usage_report.push_str(&format!(
                    "  {:?}: {} 次 ({:.1}%)\n",
                    algorithm, count, percentage
                ));
            }
        }
        
        usage_report
    }
}

// 工具函数

// 计算数据熵（简化版）
fn calculate_entropy(data: &[u8]) -> f64 {
    let mut frequency = [0u32; 256];
    for &byte in data {
        frequency[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &freq in &frequency {
        if freq > 0 {
            let p = freq as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

// 检测文本比例
fn detect_text_ratio(data: &[u8]) -> f64 {
    let text_chars = data.iter()
        .filter(|&&b| (b >= 32 && b <= 126) || b == 9 || b == 10 || b == 13)
        .count();
    
    text_chars as f64 / data.len() as f64
}

// 计算简单校验和
fn calculate_checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32))
}

impl Default for CompressionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_type_recommendation() {
        let text_data = b"Hello, world! This is a text string with repetitive patterns.";
        let binary_data = vec![0u8, 1, 2, 3, 4, 5, 255, 254, 253];
        
        let text_rec = CompressionType::recommend_for_data(text_data);
        let binary_rec = CompressionType::recommend_for_data(&binary_data);
        
        // 文本数据应该推荐Brotli
        assert_eq!(text_rec, CompressionType::Brotli);
        
        // 小的二进制数据应该推荐LZ4
        assert_eq!(binary_rec, CompressionType::LZ4);
    }
    
    #[test]
    fn test_lz4_compressor() {
        let compressor = LZ4Compressor;
        let config = CompressionConfig::default();
        let data = b"AAAAABBBBBCCCCC";
        
        let compressed = compressor.compress(data, &config).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(data.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < data.len()); // 应该有压缩效果
    }
    
    #[test]
    fn test_compression_manager() {
        let mut manager = CompressionManager::new();
        let data = b"This is test data with some repetitive content. This is test data.";
        
        let result = manager.compress(data, CompressionType::LZ4).unwrap();
        assert!(result.original_size > 0);
        assert!(result.compressed_size > 0);
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_compressions, 1);
    }
    
    #[test]
    fn test_entropy_calculation() {
        let uniform_data = (0..=255u8).collect::<Vec<_>>();
        let repetitive_data = vec![65u8; 100];
        
        let uniform_entropy = calculate_entropy(&uniform_data);
        let repetitive_entropy = calculate_entropy(&repetitive_data);
        
        assert!(uniform_entropy > repetitive_entropy);
        assert!(uniform_entropy > 7.0); // 接近8.0（最大熵）
        assert!(repetitive_entropy < 1.0); // 接近0.0（最小熵）
    }
    
    #[test]
    fn test_text_detection() {
        let text_data = b"Hello, world! This is readable text.";
        let binary_data = vec![0, 1, 2, 255, 254, 253, 128, 129];
        
        let text_ratio = detect_text_ratio(text_data);
        let binary_ratio = detect_text_ratio(&binary_data);
        
        assert!(text_ratio > 0.9);
        assert!(binary_ratio < 0.1);
    }
}