// 资源管理系统 - 统一的游戏资源加载和管理
// 开发心理：提供高效的资源管理，支持异步加载、缓存、压缩、热重载
// 设计原则：内存高效、支持多种格式、异步IO、智能缓存

pub mod cache;
pub mod compression;
pub mod loader;

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceManager, ResourceHandle, ResourceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use log::{info, debug, warn, error};

pub use cache::*;
pub use compression::*;
pub use loader::*;

// 资源类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Audio,
    Model,
    Shader,
    Font,
    Data,
    Map,
    Animation,
    Script,
    Config,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" => Some(AssetType::Texture),
            "wav" | "mp3" | "ogg" | "flac" | "m4a" => Some(AssetType::Audio),
            "obj" | "fbx" | "gltf" | "glb" | "dae" => Some(AssetType::Model),
            "vert" | "frag" | "geom" | "comp" | "glsl" => Some(AssetType::Shader),
            "ttf" | "otf" | "woff" | "woff2" => Some(AssetType::Font),
            "json" | "toml" | "yaml" | "yml" | "xml" => Some(AssetType::Data),
            "tmx" | "tsx" | "map" => Some(AssetType::Map),
            "anim" | "skeleton" | "atlas" => Some(AssetType::Animation),
            "lua" | "js" | "py" | "cs" => Some(AssetType::Script),
            "cfg" | "conf" | "ini" => Some(AssetType::Config),
            _ => None,
        }
    }
    
    pub fn default_extensions(&self) -> &[&str] {
        match self {
            AssetType::Texture => &["png", "jpg", "jpeg"],
            AssetType::Audio => &["ogg", "wav", "mp3"],
            AssetType::Model => &["gltf", "glb", "obj"],
            AssetType::Shader => &["glsl", "vert", "frag"],
            AssetType::Font => &["ttf", "otf"],
            AssetType::Data => &["json", "toml"],
            AssetType::Map => &["json", "tmx"],
            AssetType::Animation => &["json", "anim"],
            AssetType::Script => &["lua", "js"],
            AssetType::Config => &["toml", "json"],
        }
    }
}

// 资源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size: u64,
    pub checksum: String,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl AssetMetadata {
    pub fn from_path(path: &Path, id: String) -> Result<Self> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| GameError::IOError(format!("读取文件元数据失败: {}", e)))?;
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let asset_type = AssetType::from_extension(extension)
            .unwrap_or(AssetType::Data);
        
        // 计算文件校验和（简化版）
        let checksum = Self::calculate_checksum(path)?;
        
        Ok(Self {
            id,
            path: path.to_path_buf(),
            asset_type,
            size: metadata.len(),
            checksum,
            created_at: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
            modified_at: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            dependencies: Vec::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
        })
    }
    
    fn calculate_checksum(path: &Path) -> Result<String> {
        // 简化的校验和计算（实际项目中应使用更好的哈希算法）
        let data = std::fs::read(path)
            .map_err(|e| GameError::IOError(format!("读取文件失败: {}", e)))?;
        
        let mut hash = 0u64;
        for byte in data.iter() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        
        Ok(format!("{:x}", hash))
    }
    
    pub fn is_modified_since(&self, time: SystemTime) -> bool {
        self.modified_at > time
    }
    
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
    
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

// 资源加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
}

// 资源条目
#[derive(Debug)]
pub struct AssetEntry {
    pub metadata: AssetMetadata,
    pub state: AssetLoadState,
    pub data: Option<Vec<u8>>,
    pub handle: Option<ResourceHandle<Vec<u8>>>,
    pub load_time: Option<Instant>,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

impl AssetEntry {
    pub fn new(metadata: AssetMetadata) -> Self {
        Self {
            metadata,
            state: AssetLoadState::NotLoaded,
            data: None,
            handle: None,
            load_time: None,
            last_accessed: Instant::now(),
            access_count: 0,
            dependencies: Vec::new(),
            dependents: Vec::new(),
        }
    }
    
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
    
    pub fn is_loaded(&self) -> bool {
        self.state == AssetLoadState::Loaded && self.data.is_some()
    }
    
    pub fn get_load_time(&self) -> Option<Duration> {
        self.load_time.map(|start| start.elapsed())
    }
}

// 资源注册表
#[derive(Debug)]
pub struct AssetRegistry {
    assets: RwLock<HashMap<String, AssetEntry>>,
    base_paths: Vec<PathBuf>,
    loader: AssetLoader,
    cache: AssetCache,
    
    // 统计信息
    total_loads: Arc<RwLock<u64>>,
    total_load_time: Arc<RwLock<Duration>>,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            base_paths: vec![
                PathBuf::from("assets"),
                PathBuf::from("data"),
                PathBuf::from("resources"),
            ],
            loader: AssetLoader::new(),
            cache: AssetCache::new(1024 * 1024 * 256), // 256MB cache
            
            total_loads: Arc::new(RwLock::new(0)),
            total_load_time: Arc::new(RwLock::new(Duration::ZERO)),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
        }
    }
    
    // 添加资源搜索路径
    pub fn add_base_path<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref().to_path_buf();
        if !self.base_paths.contains(&path) {
            self.base_paths.push(path);
            info!("添加资源路径: {:?}", path);
        }
    }
    
    // 扫描并注册所有资源
    pub fn scan_assets(&mut self) -> Result<()> {
        info!("开始扫描资源文件...");
        let mut total_assets = 0;
        
        for base_path in &self.base_paths.clone() {
            if base_path.exists() {
                total_assets += self.scan_directory(base_path, base_path)?;
            }
        }
        
        info!("资源扫描完成，共找到 {} 个资源", total_assets);
        Ok(())
    }
    
    fn scan_directory(&mut self, dir: &Path, base_path: &Path) -> Result<usize> {
        let mut count = 0;
        
        let entries = std::fs::read_dir(dir)
            .map_err(|e| GameError::IOError(format!("读取目录失败: {:?}: {}", dir, e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| GameError::IOError(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                count += self.scan_directory(&path, base_path)?;
            } else if path.is_file() {
                // 生成资源ID（相对于基础路径）
                let relative_path = path.strip_prefix(base_path)
                    .unwrap_or(&path);
                let asset_id = relative_path.to_string_lossy().replace('\\', "/");
                
                // 创建资源元数据
                match AssetMetadata::from_path(&path, asset_id.clone()) {
                    Ok(metadata) => {
                        let entry = AssetEntry::new(metadata);
                        
                        let mut assets = self.assets.write().unwrap();
                        assets.insert(asset_id.clone(), entry);
                        count += 1;
                        
                        debug!("注册资源: {} -> {:?}", asset_id, path);
                    }
                    Err(e) => {
                        warn!("跳过资源 {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    // 预加载指定资源
    pub fn preload_asset(&mut self, asset_id: &str) -> Result<()> {
        debug!("预加载资源: {}", asset_id);
        self.load_asset_internal(asset_id, false)
    }
    
    // 异步加载资源
    pub fn load_asset(&mut self, asset_id: &str) -> Result<ResourceHandle<Vec<u8>>> {
        self.load_asset_internal(asset_id, true)?;
        
        let assets = self.assets.read().unwrap();
        if let Some(entry) = assets.get(asset_id) {
            if let Some(ref handle) = entry.handle {
                Ok(handle.clone())
            } else {
                Err(GameError::AssetError(format!("资源句柄未创建: {}", asset_id)))
            }
        } else {
            Err(GameError::AssetError(format!("资源不存在: {}", asset_id)))
        }
    }
    
    fn load_asset_internal(&mut self, asset_id: &str, create_handle: bool) -> Result<()> {
        let start_time = Instant::now();
        
        // 检查缓存
        if let Some(cached_data) = self.cache.get(asset_id) {
            *self.cache_hits.write().unwrap() += 1;
            
            if create_handle {
                let handle = ResourceManager::instance().store_resource(
                    asset_id.to_string(),
                    cached_data,
                );
                
                let mut assets = self.assets.write().unwrap();
                if let Some(entry) = assets.get_mut(asset_id) {
                    entry.handle = Some(handle);
                    entry.state = AssetLoadState::Loaded;
                    entry.mark_accessed();
                }
            }
            
            return Ok(());
        }
        
        *self.cache_misses.write().unwrap() += 1;
        
        // 查找资源条目
        let asset_path = {
            let assets = self.assets.read().unwrap();
            let entry = assets.get(asset_id)
                .ok_or_else(|| GameError::AssetError(format!("资源不存在: {}", asset_id)))?;
            entry.metadata.path.clone()
        };
        
        // 标记为加载中
        {
            let mut assets = self.assets.write().unwrap();
            if let Some(entry) = assets.get_mut(asset_id) {
                entry.state = AssetLoadState::Loading;
                entry.load_time = Some(start_time);
            }
        }
        
        // 加载资源数据
        let data = match self.loader.load_asset(&asset_path) {
            Ok(data) => data,
            Err(e) => {
                // 标记为失败
                let mut assets = self.assets.write().unwrap();
                if let Some(entry) = assets.get_mut(asset_id) {
                    entry.state = AssetLoadState::Failed;
                }
                return Err(e);
            }
        };
        
        // 缓存数据
        self.cache.insert(asset_id.to_string(), data.clone());
        
        // 创建资源句柄（如果需要）
        let handle = if create_handle {
            Some(ResourceManager::instance().store_resource(
                asset_id.to_string(),
                data.clone(),
            ))
        } else {
            None
        };
        
        // 更新资源条目
        {
            let mut assets = self.assets.write().unwrap();
            if let Some(entry) = assets.get_mut(asset_id) {
                entry.data = Some(data);
                entry.handle = handle;
                entry.state = AssetLoadState::Loaded;
                entry.mark_accessed();
            }
        }
        
        // 更新统计信息
        let load_time = start_time.elapsed();
        *self.total_loads.write().unwrap() += 1;
        *self.total_load_time.write().unwrap() += load_time;
        
        debug!("资源加载完成: {} (耗时: {:?})", asset_id, load_time);
        Ok(())
    }
    
    // 卸载资源
    pub fn unload_asset(&mut self, asset_id: &str) -> Result<()> {
        let mut assets = self.assets.write().unwrap();
        
        if let Some(entry) = assets.get_mut(asset_id) {
            entry.data = None;
            entry.handle = None;
            entry.state = AssetLoadState::NotLoaded;
            
            // 从缓存中移除
            self.cache.remove(asset_id);
            
            debug!("卸载资源: {}", asset_id);
        }
        
        Ok(())
    }
    
    // 重新加载资源（用于热重载）
    pub fn reload_asset(&mut self, asset_id: &str) -> Result<()> {
        info!("重新加载资源: {}", asset_id);
        
        // 先卸载
        self.unload_asset(asset_id)?;
        
        // 重新扫描文件元数据
        let assets_guard = self.assets.read().unwrap();
        let path = if let Some(entry) = assets_guard.get(asset_id) {
            entry.metadata.path.clone()
        } else {
            return Err(GameError::AssetError(format!("资源不存在: {}", asset_id)));
        };
        drop(assets_guard);
        
        // 更新元数据
        let new_metadata = AssetMetadata::from_path(&path, asset_id.to_string())?;
        {
            let mut assets = self.assets.write().unwrap();
            if let Some(entry) = assets.get_mut(asset_id) {
                entry.metadata = new_metadata;
            }
        }
        
        // 重新加载
        self.preload_asset(asset_id)?;
        
        Ok(())
    }
    
    // 检查资源是否已加载
    pub fn is_asset_loaded(&self, asset_id: &str) -> bool {
        let assets = self.assets.read().unwrap();
        assets.get(asset_id)
            .map(|entry| entry.is_loaded())
            .unwrap_or(false)
    }
    
    // 获取资源元数据
    pub fn get_asset_metadata(&self, asset_id: &str) -> Option<AssetMetadata> {
        let assets = self.assets.read().unwrap();
        assets.get(asset_id).map(|entry| entry.metadata.clone())
    }
    
    // 获取指定类型的所有资源ID
    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<String> {
        let assets = self.assets.read().unwrap();
        assets.iter()
            .filter(|(_, entry)| entry.metadata.asset_type == asset_type)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    // 搜索资源
    pub fn search_assets(&self, query: &str) -> Vec<String> {
        let assets = self.assets.read().unwrap();
        let query = query.to_lowercase();
        
        assets.iter()
            .filter(|(id, entry)| {
                id.to_lowercase().contains(&query) ||
                entry.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    // 清理未使用的资源
    pub fn cleanup_unused_assets(&mut self, max_age: Duration) -> usize {
        let now = Instant::now();
        let mut removed_count = 0;
        
        let mut assets = self.assets.write().unwrap();
        let mut to_remove = Vec::new();
        
        for (id, entry) in assets.iter() {
            if entry.state == AssetLoadState::Loaded &&
               now.duration_since(entry.last_accessed) > max_age &&
               entry.dependents.is_empty() {
                to_remove.push(id.clone());
            }
        }
        
        for id in to_remove {
            if let Some(mut entry) = assets.remove(&id) {
                entry.data = None;
                entry.handle = None;
                self.cache.remove(&id);
                removed_count += 1;
                debug!("清理未使用资源: {}", id);
            }
        }
        
        if removed_count > 0 {
            info!("清理了 {} 个未使用的资源", removed_count);
        }
        
        removed_count
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> AssetStats {
        let assets = self.assets.read().unwrap();
        let loaded_count = assets.values()
            .filter(|entry| entry.state == AssetLoadState::Loaded)
            .count();
        let failed_count = assets.values()
            .filter(|entry| entry.state == AssetLoadState::Failed)
            .count();
        
        AssetStats {
            total_assets: assets.len(),
            loaded_assets: loaded_count,
            failed_assets: failed_count,
            cache_size: self.cache.get_memory_usage(),
            total_loads: *self.total_loads.read().unwrap(),
            total_load_time: *self.total_load_time.read().unwrap(),
            cache_hits: *self.cache_hits.read().unwrap(),
            cache_misses: *self.cache_misses.read().unwrap(),
        }
    }
    
    // 导出资源清单
    pub fn export_manifest(&self) -> Result<String> {
        let assets = self.assets.read().unwrap();
        let metadata: Vec<&AssetMetadata> = assets.values()
            .map(|entry| &entry.metadata)
            .collect();
        
        serde_json::to_string_pretty(&metadata)
            .map_err(|e| GameError::SerializationError(format!("导出清单失败: {}", e)))
    }
    
    // 更新资源依赖关系
    pub fn update_dependencies(&mut self, asset_id: &str, dependencies: Vec<String>) {
        let mut assets = self.assets.write().unwrap();
        
        if let Some(entry) = assets.get_mut(asset_id) {
            // 移除旧的依赖关系
            for old_dep in &entry.dependencies {
                if let Some(dep_entry) = assets.get_mut(old_dep) {
                    dep_entry.dependents.retain(|dep| dep != asset_id);
                }
            }
            
            // 设置新的依赖关系
            entry.dependencies = dependencies.clone();
            
            // 建立新的依赖关系
            for dep in dependencies {
                if let Some(dep_entry) = assets.get_mut(&dep) {
                    if !dep_entry.dependents.contains(&asset_id.to_string()) {
                        dep_entry.dependents.push(asset_id.to_string());
                    }
                }
            }
        }
    }
}

// 统计信息结构
#[derive(Debug, Clone)]
pub struct AssetStats {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: usize,
    pub cache_size: usize,
    pub total_loads: u64,
    pub total_load_time: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl AssetStats {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
    
    pub fn average_load_time(&self) -> Duration {
        if self.total_loads == 0 {
            Duration::ZERO
        } else {
            self.total_load_time / self.total_loads as u32
        }
    }
}

// 全局资源管理器实例
static mut ASSET_REGISTRY: Option<AssetRegistry> = None;
static INIT: std::sync::Once = std::sync::Once::new();

impl AssetRegistry {
    pub fn instance() -> &'static mut AssetRegistry {
        unsafe {
            INIT.call_once(|| {
                ASSET_REGISTRY = Some(AssetRegistry::new());
            });
            ASSET_REGISTRY.as_mut().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("png"), Some(AssetType::Texture));
        assert_eq!(AssetType::from_extension("ogg"), Some(AssetType::Audio));
        assert_eq!(AssetType::from_extension("json"), Some(AssetType::Data));
        assert_eq!(AssetType::from_extension("unknown"), None);
    }
    
    #[test]
    fn test_asset_registry_creation() {
        let registry = AssetRegistry::new();
        assert_eq!(registry.base_paths.len(), 3);
        assert!(registry.base_paths.contains(&PathBuf::from("assets")));
    }
    
    #[test]
    fn test_asset_metadata_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"fake png data").unwrap();
        
        let metadata = AssetMetadata::from_path(&file_path, "test.png".to_string()).unwrap();
        assert_eq!(metadata.id, "test.png");
        assert_eq!(metadata.asset_type, AssetType::Texture);
        assert_eq!(metadata.size, 13);
    }
}