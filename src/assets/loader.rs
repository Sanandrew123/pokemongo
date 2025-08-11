// 资源加载器 - 异步资源加载和格式处理
// 开发心理：提供统一的资源加载接口，支持多种格式、异步加载、进度跟踪
// 设计原则：模块化、可扩展、错误处理、性能优化

use crate::core::{GameError, Result};
use crate::assets::{AssetType, AssetMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use log::{info, debug, warn, error};

// 加载进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProgress {
    pub asset_id: String,
    pub current_bytes: u64,
    pub total_bytes: u64,
    pub stage: LoadStage,
    pub elapsed_time: Duration,
    pub estimated_remaining: Option<Duration>,
}

impl LoadProgress {
    pub fn progress_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.current_bytes as f64 / self.total_bytes as f64 * 100.0).min(100.0)
        }
    }
    
    pub fn bytes_per_second(&self) -> f64 {
        if self.elapsed_time.is_zero() {
            0.0
        } else {
            self.current_bytes as f64 / self.elapsed_time.as_secs_f64()
        }
    }
}

// 加载阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadStage {
    Reading,      // 读取文件
    Parsing,      // 解析格式
    Processing,   // 后处理
    Completed,    // 完成
    Failed,       // 失败
}

// 加载选项
#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub async_loading: bool,
    pub progress_callback: Option<Arc<dyn Fn(LoadProgress) + Send + Sync>>,
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub compression: bool,
    pub validation: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            async_loading: true,
            progress_callback: None,
            timeout: Some(Duration::from_secs(30)),
            retry_count: 3,
            compression: false,
            validation: true,
        }
    }
}

// 资源解析器特征
pub trait AssetParser: Send + Sync {
    fn can_parse(&self, asset_type: AssetType, data: &[u8]) -> bool;
    fn parse(&self, data: &[u8], options: &LoadOptions) -> Result<Vec<u8>>;
    fn get_metadata(&self, data: &[u8]) -> Result<HashMap<String, String>>;
}

// 通用二进制解析器
pub struct BinaryParser;

impl AssetParser for BinaryParser {
    fn can_parse(&self, _asset_type: AssetType, _data: &[u8]) -> bool {
        true // 可以处理所有类型的原始数据
    }
    
    fn parse(&self, data: &[u8], _options: &LoadOptions) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
    
    fn get_metadata(&self, data: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), data.len().to_string());
        metadata.insert("type".to_string(), "binary".to_string());
        Ok(metadata)
    }
}

// JSON解析器
pub struct JsonParser;

impl AssetParser for JsonParser {
    fn can_parse(&self, asset_type: AssetType, data: &[u8]) -> bool {
        if asset_type == AssetType::Data {
            // 尝试解析JSON
            serde_json::from_slice::<serde_json::Value>(data).is_ok()
        } else {
            false
        }
    }
    
    fn parse(&self, data: &[u8], options: &LoadOptions) -> Result<Vec<u8>> {
        // 验证JSON格式
        if options.validation {
            serde_json::from_slice::<serde_json::Value>(data)
                .map_err(|e| GameError::ParseError(format!("JSON解析失败: {}", e)))?;
        }
        
        // 如果需要压缩，可以在这里处理
        if options.compression {
            // TODO: 实现JSON压缩
        }
        
        Ok(data.to_vec())
    }
    
    fn get_metadata(&self, data: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), data.len().to_string());
        metadata.insert("type".to_string(), "json".to_string());
        
        // 尝试获取JSON结构信息
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
            match json {
                serde_json::Value::Object(obj) => {
                    metadata.insert("keys".to_string(), obj.len().to_string());
                },
                serde_json::Value::Array(arr) => {
                    metadata.insert("elements".to_string(), arr.len().to_string());
                },
                _ => {}
            }
        }
        
        Ok(metadata)
    }
}

// 图片解析器（简化版）
pub struct ImageParser;

impl AssetParser for ImageParser {
    fn can_parse(&self, asset_type: AssetType, data: &[u8]) -> bool {
        asset_type == AssetType::Texture && self.detect_image_format(data).is_some()
    }
    
    fn parse(&self, data: &[u8], options: &LoadOptions) -> Result<Vec<u8>> {
        let format = self.detect_image_format(data)
            .ok_or_else(|| GameError::ParseError("无法识别图片格式".to_string()))?;
        
        debug!("解析图片格式: {:?}", format);
        
        // 这里可以进行图片格式转换、压缩等处理
        if options.compression {
            // TODO: 实现图片压缩
        }
        
        // 目前直接返回原始数据
        Ok(data.to_vec())
    }
    
    fn get_metadata(&self, data: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), data.len().to_string());
        
        if let Some(format) = self.detect_image_format(data) {
            metadata.insert("format".to_string(), format);
            
            // 尝试获取图片尺寸（简化实现）
            let (width, height) = self.get_image_dimensions(data, &format)?;
            metadata.insert("width".to_string(), width.to_string());
            metadata.insert("height".to_string(), height.to_string());
        }
        
        Ok(metadata)
    }
}

impl ImageParser {
    fn detect_image_format(&self, data: &[u8]) -> Option<String> {
        if data.len() < 8 {
            return None;
        }
        
        // PNG签名
        if &data[0..8] == &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Some("PNG".to_string());
        }
        
        // JPEG签名
        if data.len() >= 2 && &data[0..2] == &[0xFF, 0xD8] {
            return Some("JPEG".to_string());
        }
        
        // BMP签名
        if data.len() >= 2 && &data[0..2] == b"BM" {
            return Some("BMP".to_string());
        }
        
        // GIF签名
        if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            return Some("GIF".to_string());
        }
        
        None
    }
    
    fn get_image_dimensions(&self, data: &[u8], format: &str) -> Result<(u32, u32)> {
        match format {
            "PNG" => self.get_png_dimensions(data),
            "JPEG" => self.get_jpeg_dimensions(data),
            "BMP" => self.get_bmp_dimensions(data),
            _ => Ok((0, 0)), // 未知格式
        }
    }
    
    fn get_png_dimensions(&self, data: &[u8]) -> Result<(u32, u32)> {
        if data.len() < 24 {
            return Ok((0, 0));
        }
        
        // PNG IHDR chunk在偏移16处
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        
        Ok((width, height))
    }
    
    fn get_jpeg_dimensions(&self, _data: &[u8]) -> Result<(u32, u32)> {
        // JPEG尺寸解析比较复杂，这里简化处理
        Ok((0, 0))
    }
    
    fn get_bmp_dimensions(&self, data: &[u8]) -> Result<(u32, u32)> {
        if data.len() < 26 {
            return Ok((0, 0));
        }
        
        // BMP尺寸信息在偏移18和22处
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        
        Ok((width, height))
    }
}

// 音频解析器
pub struct AudioParser;

impl AssetParser for AudioParser {
    fn can_parse(&self, asset_type: AssetType, data: &[u8]) -> bool {
        asset_type == AssetType::Audio && self.detect_audio_format(data).is_some()
    }
    
    fn parse(&self, data: &[u8], options: &LoadOptions) -> Result<Vec<u8>> {
        let format = self.detect_audio_format(data)
            .ok_or_else(|| GameError::ParseError("无法识别音频格式".to_string()))?;
        
        debug!("解析音频格式: {:?}", format);
        
        // 这里可以进行音频格式转换、压缩等处理
        if options.compression && format != "OGG" {
            // TODO: 转换为压缩格式
        }
        
        Ok(data.to_vec())
    }
    
    fn get_metadata(&self, data: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), data.len().to_string());
        
        if let Some(format) = self.detect_audio_format(data) {
            metadata.insert("format".to_string(), format);
            
            // 获取音频参数（简化实现）
            if let Ok((sample_rate, channels, duration)) = self.get_audio_info(data, &format) {
                metadata.insert("sample_rate".to_string(), sample_rate.to_string());
                metadata.insert("channels".to_string(), channels.to_string());
                metadata.insert("duration".to_string(), duration.as_secs().to_string());
            }
        }
        
        Ok(metadata)
    }
}

impl AudioParser {
    fn detect_audio_format(&self, data: &[u8]) -> Option<String> {
        if data.len() < 12 {
            return None;
        }
        
        // WAV格式
        if &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
            return Some("WAV".to_string());
        }
        
        // OGG格式
        if &data[0..4] == b"OggS" {
            return Some("OGG".to_string());
        }
        
        // MP3格式（ID3v2标签）
        if &data[0..3] == b"ID3" {
            return Some("MP3".to_string());
        }
        
        // MP3格式（无标签）
        if data.len() >= 2 && (data[0] == 0xFF && (data[1] & 0xE0) == 0xE0) {
            return Some("MP3".to_string());
        }
        
        None
    }
    
    fn get_audio_info(&self, _data: &[u8], _format: &str) -> Result<(u32, u16, Duration)> {
        // 简化实现，返回默认值
        Ok((44100, 2, Duration::from_secs(0)))
    }
}

// 资源加载器
pub struct AssetLoader {
    parsers: Vec<Box<dyn AssetParser>>,
    active_loads: Arc<Mutex<HashMap<String, LoadProgress>>>,
    load_stats: Arc<Mutex<LoadStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub total_bytes_loaded: u64,
    pub total_load_time: Duration,
    pub cache_hits: u64,
}

impl LoadStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_loads == 0 {
            0.0
        } else {
            self.successful_loads as f64 / self.total_loads as f64
        }
    }
    
    pub fn average_load_time(&self) -> Duration {
        if self.successful_loads == 0 {
            Duration::ZERO
        } else {
            self.total_load_time / self.successful_loads as u32
        }
    }
    
    pub fn throughput_mbps(&self) -> f64 {
        if self.total_load_time.is_zero() {
            0.0
        } else {
            let mb_loaded = self.total_bytes_loaded as f64 / (1024.0 * 1024.0);
            mb_loaded / self.total_load_time.as_secs_f64()
        }
    }
}

impl std::fmt::Debug for AssetLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetLoader")
            .field("parsers_count", &self.parsers.len())
            .field("load_stats", &self.load_stats)
            .finish()
    }
}

impl AssetLoader {
    pub fn new() -> Self {
        let mut loader = Self {
            parsers: Vec::new(),
            active_loads: Arc::new(Mutex::new(HashMap::new())),
            load_stats: Arc::new(Mutex::new(LoadStats::default())),
        };
        
        // 注册默认解析器
        loader.register_parser(Box::new(BinaryParser));
        loader.register_parser(Box::new(JsonParser));
        loader.register_parser(Box::new(ImageParser));
        loader.register_parser(Box::new(AudioParser));
        
        loader
    }
    
    // 注册解析器
    pub fn register_parser(&mut self, parser: Box<dyn AssetParser>) {
        self.parsers.push(parser);
        debug!("注册解析器");
    }
    
    // 加载资源
    pub fn load_asset(&self, path: &Path) -> Result<Vec<u8>> {
        self.load_asset_with_options(path, &LoadOptions::default())
    }
    
    pub fn load_asset_with_options(&self, path: &Path, options: &LoadOptions) -> Result<Vec<u8>> {
        let asset_id = path.to_string_lossy().to_string();
        let start_time = Instant::now();
        
        // 更新统计信息
        {
            let mut stats = self.load_stats.lock().unwrap();
            stats.total_loads += 1;
        }
        
        // 初始化进度
        let mut progress = LoadProgress {
            asset_id: asset_id.clone(),
            current_bytes: 0,
            total_bytes: 0,
            stage: LoadStage::Reading,
            elapsed_time: Duration::ZERO,
            estimated_remaining: None,
        };
        
        // 获取文件大小
        let file_size = std::fs::metadata(path)
            .map_err(|e| GameError::IOError(format!("获取文件信息失败: {}", e)))?
            .len();
        
        progress.total_bytes = file_size;
        
        // 记录活跃加载
        {
            let mut active_loads = self.active_loads.lock().unwrap();
            active_loads.insert(asset_id.clone(), progress.clone());
        }
        
        let result = self.load_with_retries(path, options, &mut progress);
        
        // 移除活跃加载记录
        {
            let mut active_loads = self.active_loads.lock().unwrap();
            active_loads.remove(&asset_id);
        }
        
        // 更新统计信息
        let load_time = start_time.elapsed();
        {
            let mut stats = self.load_stats.lock().unwrap();
            stats.total_load_time += load_time;
            
            match result {
                Ok(ref data) => {
                    stats.successful_loads += 1;
                    stats.total_bytes_loaded += data.len() as u64;
                }
                Err(_) => {
                    stats.failed_loads += 1;
                }
            }
        }
        
        result
    }
    
    fn load_with_retries(&self, path: &Path, options: &LoadOptions, progress: &mut LoadProgress) -> Result<Vec<u8>> {
        let mut last_error = GameError::IOError("未知错误".to_string());
        
        for attempt in 0..=options.retry_count {
            if attempt > 0 {
                debug!("重试加载资源 ({}/{}): {:?}", attempt, options.retry_count, path);
                thread::sleep(Duration::from_millis(100 * attempt as u64));
            }
            
            match self.load_file_internal(path, options, progress) {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = e;
                    warn!("加载失败 (尝试 {}/{}): {:?} - {}", attempt + 1, options.retry_count + 1, path, last_error);
                }
            }
        }
        
        progress.stage = LoadStage::Failed;
        Err(last_error)
    }
    
    fn load_file_internal(&self, path: &Path, options: &LoadOptions, progress: &mut LoadProgress) -> Result<Vec<u8>> {
        let start_time = Instant::now();
        
        // 阶段1: 读取文件
        progress.stage = LoadStage::Reading;
        self.notify_progress(progress, options);
        
        let mut file = std::fs::File::open(path)
            .map_err(|e| GameError::IOError(format!("打开文件失败: {}", e)))?;
        
        let mut buffer = Vec::new();
        let mut temp_buffer = vec![0u8; 8192]; // 8KB临时缓冲区
        
        loop {
            match file.read(&mut temp_buffer) {
                Ok(0) => break, // EOF
                Ok(bytes_read) => {
                    buffer.extend_from_slice(&temp_buffer[..bytes_read]);
                    progress.current_bytes = buffer.len() as u64;
                    progress.elapsed_time = start_time.elapsed();
                    
                    // 估算剩余时间
                    if progress.current_bytes > 0 {
                        let bytes_per_sec = progress.bytes_per_second();
                        if bytes_per_sec > 0.0 {
                            let remaining_bytes = progress.total_bytes - progress.current_bytes;
                            progress.estimated_remaining = Some(Duration::from_secs_f64(
                                remaining_bytes as f64 / bytes_per_sec
                            ));
                        }
                    }
                    
                    self.notify_progress(progress, options);
                    
                    // 检查超时
                    if let Some(timeout) = options.timeout {
                        if progress.elapsed_time > timeout {
                            return Err(GameError::IOError("加载超时".to_string()));
                        }
                    }
                },
                Err(e) => return Err(GameError::IOError(format!("读取文件失败: {}", e))),
            }
        }
        
        // 阶段2: 解析格式
        progress.stage = LoadStage::Parsing;
        self.notify_progress(progress, options);
        
        let asset_type = AssetType::from_extension(
            path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ).unwrap_or(AssetType::Data);
        
        let parsed_data = self.parse_asset_data(&buffer, asset_type, options)?;
        
        // 阶段3: 后处理
        progress.stage = LoadStage::Processing;
        self.notify_progress(progress, options);
        
        let processed_data = self.post_process_data(parsed_data, options)?;
        
        // 完成
        progress.stage = LoadStage::Completed;
        progress.current_bytes = processed_data.len() as u64;
        progress.elapsed_time = start_time.elapsed();
        progress.estimated_remaining = Some(Duration::ZERO);
        self.notify_progress(progress, options);
        
        debug!("资源加载完成: {:?} (大小: {} bytes, 耗时: {:?})", 
               path, processed_data.len(), progress.elapsed_time);
        
        Ok(processed_data)
    }
    
    fn parse_asset_data(&self, data: &[u8], asset_type: AssetType, options: &LoadOptions) -> Result<Vec<u8>> {
        // 查找合适的解析器
        for parser in &self.parsers {
            if parser.can_parse(asset_type, data) {
                return parser.parse(data, options);
            }
        }
        
        // 如果没有找到特定解析器，使用二进制解析器
        Ok(data.to_vec())
    }
    
    fn post_process_data(&self, data: Vec<u8>, _options: &LoadOptions) -> Result<Vec<u8>> {
        // 这里可以进行额外的后处理，如压缩、加密等
        Ok(data)
    }
    
    fn notify_progress(&self, progress: &LoadProgress, options: &LoadOptions) {
        if let Some(ref callback) = options.progress_callback {
            callback(progress.clone());
        }
        
        // 更新活跃加载记录
        {
            let mut active_loads = self.active_loads.lock().unwrap();
            active_loads.insert(progress.asset_id.clone(), progress.clone());
        }
    }
    
    // 异步加载资源
    pub fn load_asset_async<F>(&self, path: &Path, options: LoadOptions, callback: F) 
    where 
        F: FnOnce(Result<Vec<u8>>) + Send + 'static
    {
        let path = path.to_path_buf();
        let parsers_count = self.parsers.len(); // 为了检查解析器是否可用
        
        thread::spawn(move || {
            // 创建新的加载器实例用于线程
            let loader = AssetLoader::new();
            let result = loader.load_asset_with_options(&path, &options);
            callback(result);
        });
    }
    
    // 获取活跃加载信息
    pub fn get_active_loads(&self) -> Vec<LoadProgress> {
        let active_loads = self.active_loads.lock().unwrap();
        active_loads.values().cloned().collect()
    }
    
    // 获取加载统计信息
    pub fn get_stats(&self) -> LoadStats {
        self.load_stats.lock().unwrap().clone()
    }
    
    // 清除统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.load_stats.lock().unwrap();
        *stats = LoadStats::default();
    }
    
    // 获取资源元数据
    pub fn get_asset_metadata(&self, path: &Path) -> Result<HashMap<String, String>> {
        // 读取少量数据用于元数据提取
        let mut file = std::fs::File::open(path)
            .map_err(|e| GameError::IOError(format!("打开文件失败: {}", e)))?;
        
        let mut buffer = vec![0u8; 1024]; // 读取前1KB
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| GameError::IOError(format!("读取文件失败: {}", e)))?;
        
        buffer.truncate(bytes_read);
        
        let asset_type = AssetType::from_extension(
            path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ).unwrap_or(AssetType::Data);
        
        // 查找合适的解析器
        for parser in &self.parsers {
            if parser.can_parse(asset_type, &buffer) {
                return parser.get_metadata(&buffer);
            }
        }
        
        // 默认元数据
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "unknown".to_string());
        Ok(metadata)
    }
    
    // 预加载资源列表
    pub fn preload_assets(&self, paths: &[&Path], options: &LoadOptions) -> Vec<Result<Vec<u8>>> {
        let mut results = Vec::new();
        
        for path in paths {
            debug!("预加载资源: {:?}", path);
            let result = self.load_asset_with_options(path, options);
            results.push(result);
        }
        
        info!("预加载完成，成功: {}, 失败: {}", 
              results.iter().filter(|r| r.is_ok()).count(),
              results.iter().filter(|r| r.is_err()).count());
        
        results
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_binary_parser() {
        let parser = BinaryParser;
        let data = vec![1, 2, 3, 4, 5];
        let options = LoadOptions::default();
        
        assert!(parser.can_parse(AssetType::Data, &data));
        assert_eq!(parser.parse(&data, &options).unwrap(), data);
        
        let metadata = parser.get_metadata(&data).unwrap();
        assert_eq!(metadata.get("size"), Some(&"5".to_string()));
    }
    
    #[test]
    fn test_json_parser() {
        let parser = JsonParser;
        let json_data = br#"{"key": "value", "number": 42}"#;
        let options = LoadOptions::default();
        
        assert!(parser.can_parse(AssetType::Data, json_data));
        assert_eq!(parser.parse(json_data, &options).unwrap(), json_data.to_vec());
        
        let metadata = parser.get_metadata(json_data).unwrap();
        assert_eq!(metadata.get("type"), Some(&"json".to_string()));
        assert_eq!(metadata.get("keys"), Some(&"2".to_string()));
    }
    
    #[test]
    fn test_image_parser() {
        let parser = ImageParser;
        
        // PNG签名
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(parser.detect_image_format(&png_data), Some("PNG".to_string()));
        
        // JPEG签名
        let jpeg_data = vec![0xFF, 0xD8, 0xFF];
        assert_eq!(parser.detect_image_format(&jpeg_data), Some("JPEG".to_string()));
    }
    
    #[test]
    fn test_asset_loader() {
        let loader = AssetLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        fs::write(&file_path, b"Hello, World!").unwrap();
        
        let data = loader.load_asset(&file_path).unwrap();
        assert_eq!(data, b"Hello, World!");
        
        let stats = loader.get_stats();
        assert_eq!(stats.total_loads, 1);
        assert_eq!(stats.successful_loads, 1);
    }
    
    #[test]
    fn test_load_progress() {
        let progress = LoadProgress {
            asset_id: "test".to_string(),
            current_bytes: 50,
            total_bytes: 100,
            stage: LoadStage::Reading,
            elapsed_time: Duration::from_secs(1),
            estimated_remaining: None,
        };
        
        assert_eq!(progress.progress_percent(), 50.0);
        assert_eq!(progress.bytes_per_second(), 50.0);
    }
}