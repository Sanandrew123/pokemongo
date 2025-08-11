// 数据管理系统
// 开发心理：数据层负责游戏数据的加载、缓存、序列化、版本管理
// 设计原则：数据分层、缓存优化、版本兼容、异步加载

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use log::{debug, warn, error, info};
use crate::core::error::GameError;

pub mod loader;
pub mod cache;
pub mod serializer;
pub mod database;

// 数据管理器
pub struct DataManager {
    // 数据加载器
    loader: loader::DataLoader,
    
    // 数据缓存
    cache: cache::DataCache,
    
    // 序列化器
    serializer: serializer::DataSerializer,
    
    // 数据库连接
    database: Option<database::GameDatabase>,
    
    // 数据版本
    data_version: String,
    
    // 加载状态
    loading_tasks: HashMap<String, LoadingTask>,
    
    // 配置
    auto_save_enabled: bool,
    compression_enabled: bool,
    encryption_enabled: bool,
    
    // 统计信息
    total_data_loaded: u64,
    cache_hit_rate: f32,
    load_time_stats: HashMap<String, f32>,
}

// 加载任务
#[derive(Debug, Clone)]
pub struct LoadingTask {
    pub task_id: String,
    pub data_type: String,
    pub progress: f32,
    pub status: LoadingStatus,
    pub start_time: std::time::Instant,
}

unsafe impl Send for LoadingTask {}
unsafe impl Sync for LoadingTask {}

// 加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingStatus {
    Pending,
    Loading,
    Completed,
    Failed,
    Cancelled,
}

// 数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Pokemon,        // Pokemon数据
    Moves,          // 技能数据
    Items,          // 道具数据
    Maps,           // 地图数据
    NPCs,           // NPC数据
    Quests,         // 任务数据
    Audio,          // 音频数据
    Textures,       // 纹理数据
    Translations,   // 翻译数据
    Config,         // 配置数据
}

// 数据元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMetadata {
    pub data_type: String,
    pub version: String,
    pub checksum: String,
    pub size: u64,
    pub last_modified: std::time::SystemTime,
    pub dependencies: Vec<String>,
    pub optional: bool,
}

unsafe impl Send for DataMetadata {}
unsafe impl Sync for DataMetadata {}

// 数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPackage {
    pub metadata: DataMetadata,
    pub data: Vec<u8>,
    pub compressed: bool,
    pub encrypted: bool,
}

unsafe impl Send for DataPackage {}
unsafe impl Sync for DataPackage {}

impl DataManager {
    pub fn new() -> Result<Self, GameError> {
        let loader = loader::DataLoader::new()?;
        let cache = cache::DataCache::new(100 * 1024 * 1024)?; // 100MB缓存
        let serializer = serializer::DataSerializer::new();
        
        Ok(Self {
            loader,
            cache,
            serializer,
            database: None,
            data_version: "1.0.0".to_string(),
            loading_tasks: HashMap::new(),
            auto_save_enabled: true,
            compression_enabled: true,
            encryption_enabled: false,
            total_data_loaded: 0,
            cache_hit_rate: 0.0,
            load_time_stats: HashMap::new(),
        })
    }
    
    // 初始化数据库
    pub fn initialize_database(&mut self, db_path: &str) -> Result<(), GameError> {
        self.database = Some(database::GameDatabase::new(db_path)?);
        info!("数据库初始化完成: {}", db_path);
        Ok(())
    }
    
    // 加载数据
    pub fn load_data<T>(&mut self, data_type: DataType, id: &str) -> Result<T, GameError> 
    where
        T: for<'de> Deserialize<'de> + Clone + 'static,
    {
        let cache_key = format!("{}:{}", data_type as u8, id);
        
        // 检查缓存
        if let Some(cached_data) = self.cache.get::<T>(&cache_key) {
            debug!("从缓存加载数据: {} ({})", cache_key, std::any::type_name::<T>());
            return Ok(cached_data);
        }
        
        // 从数据库加载
        if let Some(database) = &mut self.database {
            if let Ok(data) = database.load_data::<T>(data_type, id) {
                self.cache.set(cache_key.clone(), data.clone());
                debug!("从数据库加载数据: {} ({})", cache_key, std::any::type_name::<T>());
                self.total_data_loaded += 1;
                return Ok(data);
            }
        }
        
        // 从文件系统加载
        let start_time = std::time::Instant::now();
        let data = self.loader.load_data::<T>(data_type, id)?;
        let load_time = start_time.elapsed().as_secs_f32();
        
        // 更新统计
        let type_name = std::any::type_name::<T>();
        self.load_time_stats.insert(type_name.to_string(), load_time);
        
        // 存入缓存
        self.cache.set(cache_key.clone(), data.clone());
        
        debug!("从文件加载数据: {} ({}) 耗时: {:.3}s", cache_key, type_name, load_time);
        self.total_data_loaded += 1;
        
        Ok(data)
    }
    
    // 异步加载数据
    pub fn load_data_async<T>(&mut self, data_type: DataType, id: &str) -> Result<String, GameError>
    where
        T: for<'de> Deserialize<'de> + Clone + Send + 'static,
    {
        let task_id = format!("{}:{}:{}", data_type as u8, id, self.loading_tasks.len());
        
        let task = LoadingTask {
            task_id: task_id.clone(),
            data_type: format!("{:?}", data_type),
            progress: 0.0,
            status: LoadingStatus::Pending,
            start_time: std::time::Instant::now(),
        };
        
        self.loading_tasks.insert(task_id.clone(), task);
        
        // 实际的异步加载逻辑应该在这里实现
        // 为了简化，我们直接标记为完成
        if let Some(task) = self.loading_tasks.get_mut(&task_id) {
            task.status = LoadingStatus::Loading;
            task.progress = 50.0;
        }
        
        debug!("启动异步加载任务: {}", task_id);
        Ok(task_id)
    }
    
    // 保存数据
    pub fn save_data<T>(&mut self, data_type: DataType, id: &str, data: &T) -> Result<(), GameError>
    where
        T: Serialize + Clone + 'static,
    {
        let cache_key = format!("{}:{}", data_type as u8, id);
        
        // 更新缓存
        self.cache.set(cache_key.clone(), data.clone());
        
        // 保存到数据库
        if let Some(database) = &mut self.database {
            database.save_data(data_type, id, data)?;
            debug!("数据已保存到数据库: {}", cache_key);
        } else {
            // 保存到文件系统
            self.loader.save_data(data_type, id, data)?;
            debug!("数据已保存到文件: {}", cache_key);
        }
        
        Ok(())
    }
    
    // 批量加载数据
    pub fn load_data_batch<T>(&mut self, data_type: DataType, ids: &[String]) -> Result<HashMap<String, T>, GameError>
    where
        T: for<'de> Deserialize<'de> + Clone + 'static,
    {
        let mut results = HashMap::new();
        let mut cache_hits = 0;
        
        for id in ids {
            match self.load_data::<T>(data_type, id) {
                Ok(data) => {
                    results.insert(id.clone(), data);
                    cache_hits += 1;
                },
                Err(e) => {
                    warn!("批量加载数据失败: {} - {}", id, e);
                }
            }
        }
        
        // 更新缓存命中率
        self.cache_hit_rate = cache_hits as f32 / ids.len() as f32;
        
        debug!("批量加载数据完成: {}/{} 成功, 缓存命中率: {:.1}%", 
            results.len(), ids.len(), self.cache_hit_rate * 100.0);
        
        Ok(results)
    }
    
    // 预加载数据
    pub fn preload_data(&mut self, data_types: &[DataType]) -> Result<(), GameError> {
        for &data_type in data_types {
            match data_type {
                DataType::Pokemon => {
                    let pokemon_ids: Vec<String> = (1..152).map(|i| i.to_string()).collect();
                    let _: Result<HashMap<String, crate::pokemon::species::PokemonSpecies>, _> = 
                        self.load_data_batch(DataType::Pokemon, &pokemon_ids);
                },
                DataType::Items => {
                    let item_ids: Vec<String> = (1..501).map(|i| i.to_string()).collect();
                    // 这里应该加载道具数据，暂时跳过
                },
                DataType::Moves => {
                    let move_ids: Vec<String> = (1..722).map(|i| i.to_string()).collect();
                    // 这里应该加载技能数据，暂时跳过
                },
                _ => {
                    debug!("跳过预加载: {:?}", data_type);
                }
            }
        }
        
        info!("数据预加载完成");
        Ok(())
    }
    
    // 清理缓存
    pub fn cleanup_cache(&mut self) -> Result<(), GameError> {
        let cleaned_size = self.cache.cleanup()?;
        debug!("清理缓存: 释放 {} 字节", cleaned_size);
        Ok(())
    }
    
    // 获取加载任务状态
    pub fn get_loading_task_status(&self, task_id: &str) -> Option<&LoadingTask> {
        self.loading_tasks.get(task_id)
    }
    
    // 取消加载任务
    pub fn cancel_loading_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.loading_tasks.get_mut(task_id) {
            task.status = LoadingStatus::Cancelled;
            debug!("取消加载任务: {}", task_id);
            true
        } else {
            false
        }
    }
    
    // 验证数据完整性
    pub fn validate_data_integrity(&self) -> Result<Vec<String>, GameError> {
        let mut issues = Vec::new();
        
        // 检查必要文件是否存在
        let required_files = vec![
            "data/pokemon/species.json",
            "data/moves/moves.json",
            "data/items/items.json",
        ];
        
        for file_path in required_files {
            if !Path::new(file_path).exists() {
                issues.push(format!("缺少必要文件: {}", file_path));
            }
        }
        
        // 检查数据版本
        if let Err(e) = self.check_data_version() {
            issues.push(format!("数据版本检查失败: {}", e));
        }
        
        if issues.is_empty() {
            info!("数据完整性验证通过");
        } else {
            warn!("发现 {} 个数据问题", issues.len());
        }
        
        Ok(issues)
    }
    
    // 获取数据统计信息
    pub fn get_data_stats(&self) -> DataStats {
        DataStats {
            total_data_loaded: self.total_data_loaded,
            cache_size: self.cache.get_size(),
            cache_hit_rate: self.cache_hit_rate,
            loading_tasks_active: self.loading_tasks.len(),
            data_version: self.data_version.clone(),
            average_load_time: self.calculate_average_load_time(),
        }
    }
    
    // 私有方法
    fn check_data_version(&self) -> Result<(), GameError> {
        // 简化的版本检查
        Ok(())
    }
    
    fn calculate_average_load_time(&self) -> f32 {
        if self.load_time_stats.is_empty() {
            0.0
        } else {
            let total: f32 = self.load_time_stats.values().sum();
            total / self.load_time_stats.len() as f32
        }
    }
}

// 数据统计信息
#[derive(Debug, Clone)]
pub struct DataStats {
    pub total_data_loaded: u64,
    pub cache_size: u64,
    pub cache_hit_rate: f32,
    pub loading_tasks_active: usize,
    pub data_version: String,
    pub average_load_time: f32,
}

unsafe impl Send for DataStats {}
unsafe impl Sync for DataStats {}

// 数据配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub cache_size_mb: u32,
    pub auto_save_interval: f32,
    pub compression_level: u8,
    pub encryption_key: Option<String>,
    pub backup_enabled: bool,
    pub backup_interval: f32,
}

unsafe impl Send for DataConfig {}
unsafe impl Sync for DataConfig {}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            cache_size_mb: 100,
            auto_save_interval: 300.0,
            compression_level: 6,
            encryption_key: None,
            backup_enabled: true,
            backup_interval: 3600.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_manager_creation() {
        let manager = DataManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_data_type_enum() {
        assert_eq!(DataType::Pokemon as u8, 0);
        assert_eq!(DataType::Moves as u8, 1);
        assert_ne!(DataType::Pokemon, DataType::Moves);
    }
    
    #[test]
    fn test_loading_task() {
        let task = LoadingTask {
            task_id: "test".to_string(),
            data_type: "Pokemon".to_string(),
            progress: 0.0,
            status: LoadingStatus::Pending,
            start_time: std::time::Instant::now(),
        };
        
        assert_eq!(task.status, LoadingStatus::Pending);
        assert_eq!(task.progress, 0.0);
    }
}