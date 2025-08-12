// 资源管理器 - 统一管理游戏资源的加载、缓存和释放
// 开发心理：资源是游戏的生命线，需要高效的内存管理和异步加载
// 设计原则：智能缓存、预加载、内存池管理、资源热重载

use crate::core::{GameError, Result};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use std::any::{Any, TypeId};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use log::{info, warn, debug, error};

pub type ResourceId = u64;

// 资源状态
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
}

// 资源类型
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceType {
    Texture,
    Audio,
    Model,
    Shader,
    Font,
    Data,
    Config,
    Custom(String),
}

// 资源元数据
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    pub id: ResourceId,
    pub name: String,
    pub path: PathBuf,
    pub resource_type: ResourceType,
    pub size: u64,
    pub state: ResourceState,
    pub last_used: Instant,
    pub ref_count: u32,
    pub priority: ResourcePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

// 资源句柄
#[derive(Debug, Clone)]
pub struct ResourceHandle<T> {
    id: ResourceId,
    resource: Arc<RwLock<Option<T>>>,
    metadata: Arc<RwLock<ResourceMetadata>>,
}

impl<T> ResourceHandle<T> {
    pub fn get(&self) -> Option<Arc<RwLock<T>>> {
        let resource = self.resource.read().unwrap();
        resource.as_ref().map(|r| Arc::new(RwLock::new(r.clone())))
    }
    
    pub fn is_loaded(&self) -> bool {
        let metadata = self.metadata.read().unwrap();
        metadata.state == ResourceState::Loaded
    }
    
    pub fn get_id(&self) -> ResourceId {
        self.id
    }
}

// 资源加载器特征
pub trait ResourceLoader<T>: Send + Sync + std::fmt::Debug {
    fn load(&self, path: &Path) -> Result<T>;
    fn get_supported_extensions(&self) -> Vec<&'static str>;
}

// 纹理加载器
#[derive(Debug)]
pub struct TextureLoader;

impl ResourceLoader<Vec<u8>> for TextureLoader {
    fn load(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).map_err(|e| GameError::ResourceNotFound(format!("纹理文件读取失败: {}", e)))
    }
    
    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "bmp", "tga"]
    }
}

// 音频加载器
#[derive(Debug)]
pub struct AudioLoader;

impl ResourceLoader<Vec<u8>> for AudioLoader {
    fn load(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).map_err(|e| GameError::ResourceNotFound(format!("音频文件读取失败: {}", e)))
    }
    
    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav", "mp3", "ogg", "flac"]
    }
}

// 数据加载器
#[derive(Debug)]
pub struct DataLoader;

impl ResourceLoader<String> for DataLoader {
    fn load(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| GameError::ResourceNotFound(format!("数据文件读取失败: {}", e)))
    }
    
    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["json", "toml", "yaml", "xml", "txt"]
    }
}

// 资源缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_memory_mb: u64,
    pub max_items: usize,
    pub auto_cleanup: bool,
    pub cleanup_interval_secs: u64,
    pub lru_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_items: 10000,
            auto_cleanup: true,
            cleanup_interval_secs: 60,
            lru_enabled: true,
        }
    }
}

// 资源管理器
pub struct ResourceManager {
    resources: RwLock<HashMap<ResourceId, Box<dyn Any + Send + Sync>>>,
    metadata: RwLock<HashMap<ResourceId, Arc<RwLock<ResourceMetadata>>>>,
    loaders: RwLock<HashMap<ResourceType, Box<dyn Any + Send + Sync>>>,
    name_to_id: RwLock<HashMap<String, ResourceId>>,
    path_to_id: RwLock<HashMap<PathBuf, ResourceId>>,
    
    config: CacheConfig,
    next_id: Mutex<ResourceId>,
    stats: RwLock<ResourceStats>,
    
    // 异步加载支持
    loading_queue: Mutex<Vec<ResourceId>>,
    preload_sets: RwLock<HashMap<String, Vec<ResourceId>>>,
}

#[derive(Debug, Default)]
struct ResourceStats {
    total_resources: usize,
    loaded_resources: usize,
    memory_used: u64,
    cache_hits: u64,
    cache_misses: u64,
    load_times_ms: Vec<u64>,
}

impl ResourceManager {
    pub fn new(config: CacheConfig) -> Self {
        let mut manager = Self {
            resources: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            loaders: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            path_to_id: RwLock::new(HashMap::new()),
            
            config,
            next_id: Mutex::new(1),
            stats: RwLock::new(ResourceStats::default()),
            
            loading_queue: Mutex::new(Vec::new()),
            preload_sets: RwLock::new(HashMap::new()),
        };
        
        // 注册默认加载器
        manager.register_loader(ResourceType::Texture, TextureLoader);
        manager.register_loader(ResourceType::Audio, AudioLoader);
        manager.register_loader(ResourceType::Data, DataLoader);
        
        manager
    }
    
    // 注册资源加载器
    pub fn register_loader<T: 'static, L: ResourceLoader<T> + 'static>(
        &self,
        resource_type: ResourceType,
        loader: L,
    ) {
        let mut loaders = self.loaders.write().unwrap();
        loaders.insert(resource_type, Box::new(loader));
    }
    
    // 生成新的资源ID
    fn generate_id(&self) -> ResourceId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    }
    
    // 加载资源
    pub fn load<T: 'static + Clone + Send + Sync>(
        &self,
        name: &str,
        path: &Path,
        resource_type: ResourceType,
        priority: ResourcePriority,
    ) -> Result<ResourceHandle<T>> {
        let start_time = Instant::now();
        
        // 检查是否已经加载
        if let Some(&id) = self.name_to_id.read().unwrap().get(name) {
            let mut stats = self.stats.write().unwrap();
            stats.cache_hits += 1;
            
            if let Some(handle) = self.get_handle::<T>(id) {
                return Ok(handle);
            }
        }
        
        // 创建新资源
        let id = self.generate_id();
        let metadata = ResourceMetadata {
            id,
            name: name.to_string(),
            path: path.to_path_buf(),
            resource_type: resource_type.clone(),
            size: 0,
            state: ResourceState::Loading,
            last_used: Instant::now(),
            ref_count: 1,
            priority,
        };
        
        let metadata_arc = Arc::new(RwLock::new(metadata));
        
        // 尝试加载资源
        let loaded_resource = self.load_resource::<T>(&resource_type, path)?;
        
        // 计算资源大小
        let size = std::mem::size_of_val(&loaded_resource) as u64;
        
        // 更新元数据
        {
            let mut meta = metadata_arc.write().unwrap();
            meta.size = size;
            meta.state = ResourceState::Loaded;
        }
        
        // 创建句柄
        let handle = ResourceHandle {
            id,
            resource: Arc::new(RwLock::new(Some(loaded_resource.clone()))),
            metadata: metadata_arc.clone(),
        };
        
        // 存储资源
        self.resources.write().unwrap().insert(id, Box::new(loaded_resource));
        self.metadata.write().unwrap().insert(id, metadata_arc);
        self.name_to_id.write().unwrap().insert(name.to_string(), id);
        self.path_to_id.write().unwrap().insert(path.to_path_buf(), id);
        
        // 更新统计
        let load_time = start_time.elapsed().as_millis() as u64;
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_resources += 1;
            stats.loaded_resources += 1;
            stats.memory_used += size;
            stats.cache_misses += 1;
            stats.load_times_ms.push(load_time);
        }
        
        debug!("资源加载完成: {} ({}ms)", name, load_time);
        Ok(handle)
    }
    
    // 使用加载器加载资源
    fn load_resource<T: 'static>(
        &self,
        resource_type: &ResourceType,
        path: &Path,
    ) -> Result<T> {
        let loaders = self.loaders.read().unwrap();
        if let Some(loader_any) = loaders.get(resource_type) {
            if let Some(loader) = loader_any.downcast_ref::<dyn ResourceLoader<T>>() {
                return loader.load(path);
            }
        }
        Err(GameError::ResourceNotFound(format!(
            "未找到资源类型 {:?} 的加载器",
            resource_type
        )))
    }
    
    // 获取资源句柄
    pub fn get_handle<T: 'static + Clone>(&self, id: ResourceId) -> Option<ResourceHandle<T>> {
        let resources = self.resources.read().unwrap();
        let metadata = self.metadata.read().unwrap();
        
        if let (Some(resource_any), Some(metadata_arc)) = (resources.get(&id), metadata.get(&id)) {
            if let Some(resource) = resource_any.downcast_ref::<T>() {
                // 更新最后使用时间
                {
                    let mut meta = metadata_arc.write().unwrap();
                    meta.last_used = Instant::now();
                    meta.ref_count += 1;
                }
                
                return Some(ResourceHandle {
                    id,
                    resource: Arc::new(RwLock::new(Some(resource.clone()))),
                    metadata: metadata_arc.clone(),
                });
            }
        }
        
        None
    }
    
    // 根据名称获取资源
    pub fn get<T: 'static + Clone>(&self, name: &str) -> Option<ResourceHandle<T>> {
        if let Some(&id) = self.name_to_id.read().unwrap().get(name) {
            self.get_handle(id)
        } else {
            None
        }
    }

    // 存储资源到管理器中
    pub fn store_resource<T: 'static + Clone + Send + Sync>(
        &self,
        name: String,
        resource: T,
    ) -> ResourceHandle<T> {
        let id = self.generate_id();
        let metadata = ResourceMetadata {
            id,
            name: name.clone(),
            path: PathBuf::new(), // 内存资源没有路径
            resource_type: ResourceType::Custom("memory".to_string()),
            size: std::mem::size_of_val(&resource) as u64,
            state: ResourceState::Loaded,
            last_used: Instant::now(),
            ref_count: 1,
            priority: ResourcePriority::Normal,
        };

        let metadata_arc = Arc::new(RwLock::new(metadata));
        
        // 创建句柄
        let handle = ResourceHandle {
            id,
            resource: Arc::new(RwLock::new(Some(resource.clone()))),
            metadata: metadata_arc.clone(),
        };

        // 存储资源
        self.resources.write().unwrap().insert(id, Box::new(resource));
        self.metadata.write().unwrap().insert(id, metadata_arc);
        self.name_to_id.write().unwrap().insert(name, id);

        // 更新统计
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_resources += 1;
            stats.loaded_resources += 1;
            stats.memory_used += std::mem::size_of_val(&handle.resource) as u64;
        }

        handle
    }

    // 根据名称获取资源数据
    pub fn get_resource<T: 'static + Clone>(&self, name: &str) -> Option<T> {
        if let Some(handle) = self.get::<T>(name) {
            if let Some(resource_arc) = handle.get() {
                return Some(resource_arc.read().unwrap().clone());
            }
        }
        None
    }
    
    // 卸载资源
    pub fn unload(&self, id: ResourceId) -> Result<()> {
        let mut resources = self.resources.write().unwrap();
        let mut metadata = self.metadata.write().unwrap();
        let mut name_to_id = self.name_to_id.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();
        
        if let Some(meta_arc) = metadata.get(&id) {
            let meta = meta_arc.read().unwrap();
            let size = meta.size;
            let name = meta.name.clone();
            let path = meta.path.clone();
            
            resources.remove(&id);
            metadata.remove(&id);
            name_to_id.remove(&name);
            path_to_id.remove(&path);
            
            // 更新统计
            let mut stats = self.stats.write().unwrap();
            stats.loaded_resources = stats.loaded_resources.saturating_sub(1);
            stats.memory_used = stats.memory_used.saturating_sub(size);
            
            debug!("资源已卸载: {}", name);
        }
        
        Ok(())
    }
    
    // 创建预加载集合
    pub fn create_preload_set(&self, name: &str, resource_names: Vec<&str>) {
        let name_to_id = self.name_to_id.read().unwrap();
        let ids: Vec<ResourceId> = resource_names
            .iter()
            .filter_map(|&name| name_to_id.get(name).copied())
            .collect();
        
        self.preload_sets.write().unwrap().insert(name.to_string(), ids);
        info!("创建预加载集合: {} ({} 个资源)", name, ids.len());
    }
    
    // 预加载资源集合
    pub fn preload_set(&self, set_name: &str) -> Result<()> {
        let preload_sets = self.preload_sets.read().unwrap();
        if let Some(ids) = preload_sets.get(set_name) {
            info!("开始预加载集合: {} ({} 个资源)", set_name, ids.len());
            
            for &id in ids {
                // 这里可以添加异步预加载逻辑
                if let Some(meta_arc) = self.metadata.read().unwrap().get(&id) {
                    let mut meta = meta_arc.write().unwrap();
                    meta.last_used = Instant::now();
                }
            }
            
            info!("预加载集合完成: {}", set_name);
            Ok(())
        } else {
            Err(GameError::ResourceNotFound(format!(
                "预加载集合不存在: {}",
                set_name
            )))
        }
    }
    
    // 清理未使用的资源
    pub fn cleanup_unused(&self, max_unused_time_secs: u64) -> usize {
        let now = Instant::now();
        let threshold = std::time::Duration::from_secs(max_unused_time_secs);
        
        let metadata = self.metadata.read().unwrap();
        let mut to_unload = Vec::new();
        
        for (&id, meta_arc) in metadata.iter() {
            let meta = meta_arc.read().unwrap();
            if meta.ref_count == 0 && now.duration_since(meta.last_used) > threshold {
                to_unload.push(id);
            }
        }
        drop(metadata);
        
        let count = to_unload.len();
        for id in to_unload {
            if let Err(e) = self.unload(id) {
                warn!("清理资源失败: {}", e);
            }
        }
        
        if count > 0 {
            info!("清理了 {} 个未使用的资源", count);
        }
        
        count
    }
    
    // 内存压力下的紧急清理
    pub fn emergency_cleanup(&self) -> u64 {
        let mut freed_memory = 0u64;
        
        // 按优先级和最后使用时间排序
        let metadata = self.metadata.read().unwrap();
        let mut candidates: Vec<_> = metadata
            .iter()
            .map(|(&id, meta_arc)| {
                let meta = meta_arc.read().unwrap();
                (id, meta.priority, meta.last_used, meta.size)
            })
            .collect();
        
        // 优先级低的先清理，同优先级的按最后使用时间排序
        candidates.sort_by(|a, b| {
            a.1.cmp(&b.1).then(a.2.cmp(&b.2))
        });
        drop(metadata);
        
        // 清理一半的低优先级资源
        let to_clean = candidates.len() / 2;
        for (id, _, _, size) in candidates.into_iter().take(to_clean) {
            if self.unload(id).is_ok() {
                freed_memory += size;
            }
        }
        
        warn!("紧急清理完成，释放内存: {} MB", freed_memory / (1024 * 1024));
        freed_memory
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> ResourceStats {
        self.stats.read().unwrap().clone()
    }
    
    // 获取内存使用情况
    pub fn get_memory_usage(&self) -> (u64, u64) {
        let stats = self.stats.read().unwrap();
        let max_memory = self.config.max_memory_mb * 1024 * 1024;
        (stats.memory_used, max_memory)
    }
    
    // 检查内存压力
    pub fn is_memory_pressure(&self) -> bool {
        let (used, max) = self.get_memory_usage();
        used as f64 / max as f64 > 0.8
    }
    
    // 热重载资源
    pub fn hot_reload(&self, name: &str) -> Result<()> {
        if let Some(&id) = self.name_to_id.read().unwrap().get(name) {
            if let Some(meta_arc) = self.metadata.read().unwrap().get(&id) {
                let meta = meta_arc.read().unwrap();
                let path = meta.path.clone();
                let resource_type = meta.resource_type.clone();
                let priority = meta.priority;
                drop(meta);
                
                // 卸载旧资源
                self.unload(id)?;
                
                // 重新加载
                info!("热重载资源: {}", name);
                // 这里需要知道具体的类型，实际实现中可能需要类型注册
                
                Ok(())
            } else {
                Err(GameError::ResourceNotFound(format!("资源不存在: {}", name)))
            }
        } else {
            Err(GameError::ResourceNotFound(format!("资源名称不存在: {}", name)))
        }
    }
    
    // 清理所有资源
    pub fn clear(&self) {
        self.resources.write().unwrap().clear();
        self.metadata.write().unwrap().clear();
        self.name_to_id.write().unwrap().clear();
        self.path_to_id.write().unwrap().clear();
        self.preload_sets.write().unwrap().clear();
        
        *self.stats.write().unwrap() = ResourceStats::default();
        
        info!("所有资源已清理");
    }
}

// 克隆统计信息
impl Clone for ResourceStats {
    fn clone(&self) -> Self {
        Self {
            total_resources: self.total_resources,
            loaded_resources: self.loaded_resources,
            memory_used: self.memory_used,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            load_times_ms: self.load_times_ms.clone(),
        }
    }
}

// 全局资源管理器
static mut RESOURCE_MANAGER: Option<ResourceManager> = None;
static RESOURCE_INIT: std::sync::Once = std::sync::Once::new();

impl ResourceManager {
    pub fn init() -> Result<()> {
        unsafe {
            RESOURCE_INIT.call_once(|| {
                RESOURCE_MANAGER = Some(ResourceManager::new(CacheConfig::default()));
            });
        }
        Ok(())
    }
    
    pub fn instance() -> &'static ResourceManager {
        unsafe {
            RESOURCE_MANAGER.as_ref().expect("资源管理器未初始化")
        }
    }
    
    pub fn cleanup() {
        unsafe {
            if let Some(ref manager) = RESOURCE_MANAGER {
                manager.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_resource_manager_creation() {
        let manager = ResourceManager::new(CacheConfig::default());
        let stats = manager.get_stats();
        assert_eq!(stats.total_resources, 0);
    }
    
    #[test]
    fn test_load_text_resource() {
        let manager = ResourceManager::new(CacheConfig::default());
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        
        let handle = manager.load::<String>(
            "test_text",
            temp_file.path(),
            ResourceType::Data,
            ResourcePriority::Normal,
        ).unwrap();
        
        assert!(handle.is_loaded());
        assert_eq!(handle.get_id(), 1);
    }
    
    #[test]
    fn test_resource_caching() {
        let manager = ResourceManager::new(CacheConfig::default());
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Cached content").unwrap();
        
        // 第一次加载
        let _handle1 = manager.load::<String>(
            "cached_resource",
            temp_file.path(),
            ResourceType::Data,
            ResourcePriority::Normal,
        ).unwrap();
        
        // 第二次加载应该从缓存返回
        let _handle2 = manager.load::<String>(
            "cached_resource",
            temp_file.path(),
            ResourceType::Data,
            ResourcePriority::Normal,
        ).unwrap();
        
        let stats = manager.get_stats();
        assert!(stats.cache_hits > 0);
    }
    
    #[test]
    fn test_resource_unloading() {
        let manager = ResourceManager::new(CacheConfig::default());
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();
        
        let handle = manager.load::<String>(
            "unload_test",
            temp_file.path(),
            ResourceType::Data,
            ResourcePriority::Normal,
        ).unwrap();
        
        let id = handle.get_id();
        manager.unload(id).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.loaded_resources, 0);
    }
}