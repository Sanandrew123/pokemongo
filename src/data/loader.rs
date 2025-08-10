// 数据加载器
// 开发心理：负责从各种数据源加载游戏数据，支持多种格式和异步加载
// 设计原则：格式兼容、错误恢复、性能优化、可扩展性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use log::{debug, warn, error, info};
use crate::core::error::GameError;
use super::DataType;

// 数据加载器
pub struct DataLoader {
    // 数据路径配置
    data_paths: HashMap<DataType, PathBuf>,
    
    // 支持的文件格式
    supported_formats: Vec<DataFormat>,
    
    // 加载器配置
    config: LoaderConfig,
    
    // 文件监视器
    file_watchers: HashMap<String, FileWatcher>,
    
    // 加载统计
    load_statistics: LoadStatistics,
}

// 数据格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    JSON,           // JSON格式
    YAML,           // YAML格式
    TOML,           // TOML格式
    Binary,         // 二进制格式
    MessagePack,    // MessagePack格式
    CSV,            // CSV格式
}

// 加载器配置
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub default_format: DataFormat,
    pub encoding: String,
    pub buffer_size: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub validate_checksums: bool,
    pub watch_file_changes: bool,
}

// 文件监视器
#[derive(Debug, Clone)]
pub struct FileWatcher {
    pub file_path: PathBuf,
    pub last_modified: std::time::SystemTime,
    pub checksum: String,
    pub auto_reload: bool,
}

// 加载统计
#[derive(Debug, Clone, Default)]
pub struct LoadStatistics {
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub total_load_time: f64,
    pub bytes_loaded: u64,
    pub cache_hits: u64,
}

// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    pub name: String,
    pub source_type: DataSourceType,
    pub connection_string: Option<String>,
    pub credentials: Option<HashMap<String, String>>,
    pub options: HashMap<String, String>,
}

// 数据源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    FileSystem,     // 文件系统
    Database,       // 数据库
    HTTP,           // HTTP API
    FTP,            // FTP服务器
    Memory,         // 内存数据
}

impl DataLoader {
    pub fn new() -> Result<Self, GameError> {
        let mut data_paths = HashMap::new();
        
        // 设置默认数据路径
        data_paths.insert(DataType::Pokemon, PathBuf::from("data/pokemon"));
        data_paths.insert(DataType::Moves, PathBuf::from("data/moves"));
        data_paths.insert(DataType::Items, PathBuf::from("data/items"));
        data_paths.insert(DataType::Maps, PathBuf::from("data/maps"));
        data_paths.insert(DataType::NPCs, PathBuf::from("data/npcs"));
        data_paths.insert(DataType::Quests, PathBuf::from("data/quests"));
        data_paths.insert(DataType::Audio, PathBuf::from("assets/audio"));
        data_paths.insert(DataType::Textures, PathBuf::from("assets/textures"));
        data_paths.insert(DataType::Translations, PathBuf::from("data/translations"));
        data_paths.insert(DataType::Config, PathBuf::from("config"));
        
        // 确保数据目录存在
        for path in data_paths.values() {
            if let Err(e) = fs::create_dir_all(path) {
                warn!("创建数据目录失败: {} - {}", path.display(), e);
            }
        }
        
        let config = LoaderConfig {
            default_format: DataFormat::JSON,
            encoding: "utf-8".to_string(),
            buffer_size: 8192,
            timeout_seconds: 30,
            retry_attempts: 3,
            validate_checksums: true,
            watch_file_changes: false,
        };
        
        Ok(Self {
            data_paths,
            supported_formats: vec![
                DataFormat::JSON,
                DataFormat::YAML,
                DataFormat::TOML,
                DataFormat::Binary,
                DataFormat::CSV,
            ],
            config,
            file_watchers: HashMap::new(),
            load_statistics: LoadStatistics::default(),
        })
    }
    
    // 加载数据
    pub fn load_data<T>(&mut self, data_type: DataType, id: &str) -> Result<T, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let start_time = std::time::Instant::now();
        
        // 构建文件路径
        let file_path = self.build_file_path(data_type, id)?;
        
        // 检查文件是否存在
        if !file_path.exists() {
            self.load_statistics.failed_loads += 1;
            return Err(GameError::Data(format!("数据文件不存在: {}", file_path.display())));
        }
        
        // 尝试多种格式
        let mut last_error = None;
        
        for format in &self.supported_formats {
            let format_path = self.try_format_extension(&file_path, *format);
            
            if format_path.exists() {
                match self.load_file_with_format::<T>(&format_path, *format) {
                    Ok(data) => {
                        let load_time = start_time.elapsed().as_secs_f64();
                        self.load_statistics.total_loads += 1;
                        self.load_statistics.successful_loads += 1;
                        self.load_statistics.total_load_time += load_time;
                        
                        debug!("加载数据成功: {} ({:?}) 耗时: {:.3}s", 
                            format_path.display(), format, load_time);
                        
                        // 添加文件监视器
                        if self.config.watch_file_changes {
                            self.add_file_watcher(format_path, false);
                        }
                        
                        return Ok(data);
                    },
                    Err(e) => {
                        last_error = Some(e);
                        debug!("格式 {:?} 加载失败: {}", format, last_error.as_ref().unwrap());
                    }
                }
            }
        }
        
        self.load_statistics.failed_loads += 1;
        Err(last_error.unwrap_or_else(|| 
            GameError::Data(format!("无法加载数据: {}", file_path.display()))
        ))
    }
    
    // 保存数据
    pub fn save_data<T>(&mut self, data_type: DataType, id: &str, data: &T) -> Result<(), GameError>
    where
        T: Serialize,
    {
        let file_path = self.build_file_path(data_type, id)?;
        
        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GameError::Data(format!("创建目录失败: {}", e)))?;
        }
        
        // 使用默认格式保存
        let format_path = self.try_format_extension(&file_path, self.config.default_format);
        self.save_file_with_format(&format_path, data, self.config.default_format)?;
        
        debug!("保存数据成功: {}", format_path.display());
        Ok(())
    }
    
    // 批量加载数据
    pub fn load_batch<T>(&mut self, data_type: DataType, ids: &[String]) -> Result<HashMap<String, T>, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut results = HashMap::new();
        let mut errors = Vec::new();
        
        for id in ids {
            match self.load_data::<T>(data_type, id) {
                Ok(data) => {
                    results.insert(id.clone(), data);
                },
                Err(e) => {
                    errors.push(format!("{}: {}", id, e));
                }
            }
        }
        
        if !errors.is_empty() {
            warn!("批量加载部分失败: {}", errors.join(", "));
        }
        
        info!("批量加载完成: {}/{} 成功", results.len(), ids.len());
        Ok(results)
    }
    
    // 检查文件是否发生变化
    pub fn check_file_changes(&mut self) -> Vec<String> {
        let mut changed_files = Vec::new();
        
        for (file_path, watcher) in &mut self.file_watchers {
            if let Ok(metadata) = fs::metadata(&watcher.file_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified != watcher.last_modified {
                        watcher.last_modified = modified;
                        changed_files.push(file_path.clone());
                        
                        if watcher.auto_reload {
                            debug!("文件已变化，准备重新加载: {}", watcher.file_path.display());
                        }
                    }
                }
            }
        }
        
        changed_files
    }
    
    // 预加载数据目录
    pub fn preload_directory(&mut self, data_type: DataType) -> Result<Vec<String>, GameError> {
        let data_path = self.data_paths.get(&data_type)
            .ok_or_else(|| GameError::Data("未知的数据类型".to_string()))?;
        
        if !data_path.exists() {
            return Err(GameError::Data(format!("数据目录不存在: {}", data_path.display())));
        }
        
        let mut loaded_files = Vec::new();
        
        for entry in fs::read_dir(data_path)
            .map_err(|e| GameError::Data(format!("读取目录失败: {}", e)))? {
            
            let entry = entry.map_err(|e| GameError::Data(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if let Some(stem_str) = stem.to_str() {
                        loaded_files.push(stem_str.to_string());
                    }
                }
            }
        }
        
        info!("预加载目录: {} 发现 {} 个文件", data_path.display(), loaded_files.len());
        Ok(loaded_files)
    }
    
    // 验证文件完整性
    pub fn validate_file_integrity(&self, file_path: &Path) -> Result<bool, GameError> {
        if !file_path.exists() {
            return Ok(false);
        }
        
        // 简单的文件大小检查
        let metadata = fs::metadata(file_path)
            .map_err(|e| GameError::Data(format!("获取文件元数据失败: {}", e)))?;
        
        if metadata.len() == 0 {
            warn!("文件为空: {}", file_path.display());
            return Ok(false);
        }
        
        // 这里可以添加更复杂的校验逻辑，如MD5校验
        Ok(true)
    }
    
    // 获取数据源信息
    pub fn get_data_source_info(&self, data_type: DataType) -> Option<DataSourceInfo> {
        self.data_paths.get(&data_type).map(|path| {
            DataSourceInfo {
                data_type,
                path: path.clone(),
                exists: path.exists(),
                file_count: self.count_files_in_directory(path),
                total_size: self.calculate_directory_size(path),
            }
        })
    }
    
    // 获取加载统计
    pub fn get_load_statistics(&self) -> &LoadStatistics {
        &self.load_statistics
    }
    
    // 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.load_statistics = LoadStatistics::default();
        debug!("加载统计信息已重置");
    }
    
    // 私有方法
    fn build_file_path(&self, data_type: DataType, id: &str) -> Result<PathBuf, GameError> {
        let base_path = self.data_paths.get(&data_type)
            .ok_or_else(|| GameError::Data("未支持的数据类型".to_string()))?;
        
        Ok(base_path.join(id))
    }
    
    fn try_format_extension(&self, base_path: &Path, format: DataFormat) -> PathBuf {
        let extension = match format {
            DataFormat::JSON => "json",
            DataFormat::YAML => "yaml",
            DataFormat::TOML => "toml",
            DataFormat::Binary => "bin",
            DataFormat::MessagePack => "msgpack",
            DataFormat::CSV => "csv",
        };
        
        base_path.with_extension(extension)
    }
    
    fn load_file_with_format<T>(&self, file_path: &Path, format: DataFormat) -> Result<T, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let content = fs::read_to_string(file_path)
            .map_err(|e| GameError::Data(format!("读取文件失败: {}", e)))?;
        
        match format {
            DataFormat::JSON => {
                serde_json::from_str(&content)
                    .map_err(|e| GameError::Data(format!("JSON解析失败: {}", e)))
            },
            DataFormat::YAML => {
                // 需要添加yaml依赖
                Err(GameError::Data("YAML格式暂不支持".to_string()))
            },
            DataFormat::TOML => {
                // 需要添加toml依赖
                Err(GameError::Data("TOML格式暂不支持".to_string()))
            },
            DataFormat::Binary => {
                let bytes = fs::read(file_path)
                    .map_err(|e| GameError::Data(format!("读取二进制文件失败: {}", e)))?;
                
                bincode::deserialize(&bytes)
                    .map_err(|e| GameError::Data(format!("二进制反序列化失败: {}", e)))
            },
            DataFormat::MessagePack => {
                Err(GameError::Data("MessagePack格式暂不支持".to_string()))
            },
            DataFormat::CSV => {
                Err(GameError::Data("CSV格式暂不支持".to_string()))
            },
        }
    }
    
    fn save_file_with_format<T>(&self, file_path: &Path, data: &T, format: DataFormat) -> Result<(), GameError>
    where
        T: Serialize,
    {
        match format {
            DataFormat::JSON => {
                let content = serde_json::to_string_pretty(data)
                    .map_err(|e| GameError::Data(format!("JSON序列化失败: {}", e)))?;
                
                fs::write(file_path, content)
                    .map_err(|e| GameError::Data(format!("写入文件失败: {}", e)))
            },
            DataFormat::Binary => {
                let bytes = bincode::serialize(data)
                    .map_err(|e| GameError::Data(format!("二进制序列化失败: {}", e)))?;
                
                fs::write(file_path, bytes)
                    .map_err(|e| GameError::Data(format!("写入二进制文件失败: {}", e)))
            },
            _ => Err(GameError::Data(format!("不支持的保存格式: {:?}", format))),
        }
    }
    
    fn add_file_watcher(&mut self, file_path: PathBuf, auto_reload: bool) {
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                let watcher = FileWatcher {
                    file_path: file_path.clone(),
                    last_modified: modified,
                    checksum: String::new(), // 简化实现
                    auto_reload,
                };
                
                self.file_watchers.insert(file_path.to_string_lossy().to_string(), watcher);
            }
        }
    }
    
    fn count_files_in_directory(&self, path: &Path) -> usize {
        if let Ok(entries) = fs::read_dir(path) {
            entries.filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().is_file() { Some(()) } else { None }
                })
            }).count()
        } else {
            0
        }
    }
    
    fn calculate_directory_size(&self, path: &Path) -> u64 {
        if let Ok(entries) = fs::read_dir(path) {
            entries.filter_map(|entry| {
                entry.ok().and_then(|e| {
                    fs::metadata(e.path()).ok().map(|m| m.len())
                })
            }).sum()
        } else {
            0
        }
    }
}

// 数据源信息
#[derive(Debug, Clone)]
pub struct DataSourceInfo {
    pub data_type: DataType,
    pub path: PathBuf,
    pub exists: bool,
    pub file_count: usize,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;
    
    #[test]
    fn test_data_loader_creation() {
        let loader = DataLoader::new();
        assert!(loader.is_ok());
    }
    
    #[test]
    fn test_file_path_building() {
        let loader = DataLoader::new().unwrap();
        let path = loader.build_file_path(DataType::Pokemon, "pikachu").unwrap();
        assert!(path.to_string_lossy().contains("pokemon"));
        assert!(path.to_string_lossy().contains("pikachu"));
    }
    
    #[test]
    fn test_format_extension() {
        let loader = DataLoader::new().unwrap();
        let base_path = PathBuf::from("test");
        
        let json_path = loader.try_format_extension(&base_path, DataFormat::JSON);
        assert_eq!(json_path.extension().unwrap(), "json");
        
        let binary_path = loader.try_format_extension(&base_path, DataFormat::Binary);
        assert_eq!(binary_path.extension().unwrap(), "bin");
    }
    
    #[test]
    fn test_load_save_data() {
        let dir = tempdir().unwrap();
        let mut loader = DataLoader::new().unwrap();
        
        // 修改Pokemon数据路径到临时目录
        loader.data_paths.insert(DataType::Pokemon, dir.path().to_path_buf());
        
        // 创建测试数据
        let test_data = json!({
            "name": "Pikachu",
            "type": "Electric",
            "level": 25
        });
        
        // 保存数据
        let result = loader.save_data(DataType::Pokemon, "pikachu", &test_data);
        assert!(result.is_ok());
        
        // 加载数据
        let loaded_data: serde_json::Value = loader.load_data(DataType::Pokemon, "pikachu").unwrap();
        assert_eq!(loaded_data["name"], "Pikachu");
        assert_eq!(loaded_data["type"], "Electric");
    }
}