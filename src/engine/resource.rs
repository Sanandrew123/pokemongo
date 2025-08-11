/*
 * 资源管理系统 - Resource Manager System
 * 
 * 开发心理过程：
 * 设计统一的资源管理系统，支持异步加载、缓存管理、内存优化等功能
 * 需要考虑资源的生命周期管理、依赖关系和加载优先级
 * 重点关注性能优化和内存使用效率
 */

use bevy::prelude::*;
use bevy::asset::*;
use bevy::utils::HashMap;
use std::collections::{VecDeque, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use crate::core::error::{GameResult, GameError};

// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Texture,        // 纹理
    Audio,          // 音频
    Mesh,           // 网格
    Material,       // 材质
    Shader,         // 着色器
    Font,           // 字体
    Animation,      // 动画
    Data,           // 数据文件
    Scene,          // 场景
    Script,         // 脚本
}

// 资源状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceStatus {
    NotLoaded,      // 未加载
    Loading,        // 正在加载
    Loaded,         // 已加载
    Failed,         // 加载失败
    Unloading,      // 正在卸载
}

// 加载优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Critical = 4,   // 关键资源
    High = 3,       // 高优先级
    Normal = 2,     // 普通优先级
    Low = 1,        // 低优先级
    Background = 0, // 后台加载
}

// 资源信息
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub id: String,
    pub resource_type: ResourceType,
    pub path: PathBuf,
    pub status: ResourceStatus,
    pub priority: LoadPriority,
    pub size: usize,
    pub last_accessed: f64,
    pub ref_count: u32,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub tags: HashSet<String>,
    pub metadata: HashMap<String, String>,
}

impl ResourceInfo {
    pub fn new(id: String, resource_type: ResourceType, path: PathBuf) -> Self {
        Self {
            id,
            resource_type,
            path,
            status: ResourceStatus::NotLoaded,
            priority: LoadPriority::Normal,
            size: 0,
            last_accessed: 0.0,
            ref_count: 0,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            tags: HashSet::new(),
            metadata: HashMap::new(),
        }
    }
}

// 资源句柄包装器
#[derive(Debug, Clone)]
pub struct ResourceHandle<T: Asset> {
    pub handle: Handle<T>,
    pub info: Arc<RwLock<ResourceInfo>>,
}

// 加载请求
#[derive(Debug)]
pub struct LoadRequest {
    pub resource_id: String,
    pub priority: LoadPriority,
    pub callback: Option<Box<dyn Fn(GameResult<()>) + Send + Sync>>,
    pub timestamp: f64,
}

// 资源缓存策略
#[derive(Debug, Clone)]
pub struct CacheStrategy {
    pub max_memory_usage: usize,      // 最大内存使用量 (bytes)
    pub max_cached_resources: usize,  // 最大缓存资源数
    pub ttl_seconds: f64,            // 缓存生存时间
    pub auto_cleanup_interval: f64,   // 自动清理间隔
    pub preload_distance: f32,       // 预加载距离
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self {
            max_memory_usage: 512 * 1024 * 1024, // 512MB
            max_cached_resources: 1000,
            ttl_seconds: 300.0, // 5分钟
            auto_cleanup_interval: 60.0, // 1分钟
            preload_distance: 100.0,
        }
    }
}

// 资源事件
#[derive(Debug, Clone)]
pub enum ResourceEvent {
    LoadStarted(String),
    LoadCompleted(String),
    LoadFailed(String, String),
    Unloaded(String),
    CacheCleared,
    MemoryWarning(usize),
}

// 资源管理器主结构
pub struct ResourceManager {
    asset_path: PathBuf,
    
    // 资源注册表
    resources: HashMap<String, Arc<RwLock<ResourceInfo>>>,
    loaded_handles: HashMap<String, HandleUntyped>,
    
    // 加载队列
    load_queue: VecDeque<LoadRequest>,
    loading_resources: HashSet<String>,
    
    // 缓存管理
    cache_strategy: CacheStrategy,
    memory_usage: usize,
    last_cleanup: f64,
    
    // 统计信息
    total_resources: usize,
    loaded_resources: usize,
    failed_resources: usize,
    
    // 异步加载控制
    max_concurrent_loads: usize,
    current_loads: usize,
}

impl ResourceManager {
    // 创建新的资源管理器
    pub fn new(asset_path: &str) -> GameResult<Self> {
        Ok(Self {
            asset_path: PathBuf::from(asset_path),
            resources: HashMap::new(),
            loaded_handles: HashMap::new(),
            load_queue: VecDeque::new(),
            loading_resources: HashSet::new(),
            cache_strategy: CacheStrategy::default(),
            memory_usage: 0,
            last_cleanup: 0.0,
            total_resources: 0,
            loaded_resources: 0,
            failed_resources: 0,
            max_concurrent_loads: 4,
            current_loads: 0,
        })
    }

    // 初始化资源管理器
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化资源管理器...");
        
        // 确保资源路径存在
        if !self.asset_path.exists() {
            warn!("资源路径不存在: {:?}", self.asset_path);
        }
        
        // 预留容量
        self.resources.reserve(1000);
        self.loaded_handles.reserve(1000);
        
        info!("资源管理器初始化完成");
        Ok(())
    }

    // 关闭资源管理器
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("关闭资源管理器...");
        
        // 清理所有资源
        self.unload_all()?;
        
        self.resources.clear();
        self.loaded_handles.clear();
        self.load_queue.clear();
        self.loading_resources.clear();
        
        info!("资源管理器已关闭");
        Ok(())
    }

    // 更新资源管理器
    pub fn update(&mut self) -> GameResult<()> {
        let current_time = self.get_current_time();
        
        // 处理加载队列
        self.process_load_queue()?;
        
        // 自动清理缓存
        if current_time - self.last_cleanup > self.cache_strategy.auto_cleanup_interval {
            self.cleanup_cache(current_time)?;
            self.last_cleanup = current_time;
        }
        
        Ok(())
    }

    // 注册资源
    pub fn register_resource(&mut self, 
        id: String, 
        resource_type: ResourceType, 
        path: &str
    ) -> GameResult<()> {
        
        if self.resources.contains_key(&id) {
            return Err(GameError::Resource(format!("资源已注册: {}", id)));
        }

        let full_path = self.asset_path.join(path);
        let resource_info = ResourceInfo::new(id.clone(), resource_type, full_path);
        
        self.resources.insert(id.clone(), Arc::new(RwLock::new(resource_info)));
        self.total_resources += 1;

        debug!("注册资源: {} -> {}", id, path);
        Ok(())
    }

    // 异步加载资源
    pub fn load_async<T: Asset>(&mut self, 
        asset_server: &AssetServer,
        resource_id: &str, 
        priority: LoadPriority
    ) -> GameResult<ResourceHandle<T>> {
        
        let resource_info = self.resources.get(resource_id)
            .ok_or_else(|| GameError::Resource(format!("资源未注册: {}", resource_id)))?
            .clone();

        // 检查是否已加载
        if let Some(handle) = self.loaded_handles.get(resource_id) {
            if let Ok(typed_handle) = handle.clone().typed::<T>() {
                return Ok(ResourceHandle {
                    handle: typed_handle,
                    info: resource_info,
                });
            }
        }

        // 添加到加载队列
        let load_request = LoadRequest {
            resource_id: resource_id.to_string(),
            priority,
            callback: None,
            timestamp: self.get_current_time(),
        };

        // 按优先级插入队列
        let insert_pos = self.load_queue
            .iter()
            .position(|req| req.priority < priority)
            .unwrap_or(self.load_queue.len());
        
        self.load_queue.insert(insert_pos, load_request);

        // 立即开始加载（如果可能）
        self.try_start_load(asset_server, resource_id)?;

        // 创建句柄占位符
        let path = {
            let info = resource_info.blocking_read();
            info.path.to_str().unwrap_or("").to_string()
        };
        
        let handle: Handle<T> = asset_server.load(&path);
        let untyped_handle = handle.clone().untyped();
        
        self.loaded_handles.insert(resource_id.to_string(), untyped_handle);

        Ok(ResourceHandle {
            handle,
            info: resource_info,
        })
    }

    // 同步加载资源
    pub fn load_sync<T: Asset>(&mut self, 
        asset_server: &AssetServer,
        resource_id: &str
    ) -> GameResult<ResourceHandle<T>> {
        
        self.load_async(asset_server, resource_id, LoadPriority::Critical)
    }

    // 卸载资源
    pub fn unload(&mut self, resource_id: &str) -> GameResult<()> {
        if let Some(resource_info) = self.resources.get(resource_id) {
            {
                let mut info = resource_info.blocking_write();
                if info.ref_count > 0 {
                    info.ref_count -= 1;
                    if info.ref_count > 0 {
                        return Ok(()); // 还有其他引用，不卸载
                    }
                }
                info.status = ResourceStatus::Unloading;
            }

            // 移除句柄
            if let Some(handle) = self.loaded_handles.remove(resource_id) {
                // 在实际实现中，这里需要通知AssetServer释放资源
                drop(handle);
                self.loaded_resources = self.loaded_resources.saturating_sub(1);
            }

            {
                let mut info = resource_info.blocking_write();
                info.status = ResourceStatus::NotLoaded;
                info.ref_count = 0;
            }

            info!("卸载资源: {}", resource_id);
        }

        Ok(())
    }

    // 卸载所有资源
    pub fn unload_all(&mut self) -> GameResult<()> {
        let resource_ids: Vec<String> = self.resources.keys().cloned().collect();
        
        for resource_id in resource_ids {
            self.unload(&resource_id)?;
        }
        
        self.memory_usage = 0;
        info!("已卸载所有资源");
        Ok(())
    }

    // 预加载资源批次
    pub fn preload_batch(&mut self, 
        asset_server: &AssetServer,
        resource_ids: &[String], 
        priority: LoadPriority
    ) -> GameResult<()> {
        
        for resource_id in resource_ids {
            if let Ok(resource_info) = self.resources.get(resource_id)
                .ok_or_else(|| GameError::Resource(format!("资源未注册: {}", resource_id)))
            {
                let resource_type = {
                    let info = resource_info.blocking_read();
                    info.resource_type
                };

                // 根据类型加载不同的资源
                match resource_type {
                    ResourceType::Texture => {
                        let _ = self.load_async::<Image>(asset_server, resource_id, priority);
                    },
                    ResourceType::Audio => {
                        let _ = self.load_async::<AudioSource>(asset_server, resource_id, priority);
                    },
                    ResourceType::Font => {
                        let _ = self.load_async::<Font>(asset_server, resource_id, priority);
                    },
                    _ => {
                        warn!("不支持的资源类型预加载: {:?}", resource_type);
                    }
                }
            }
        }
        
        info!("开始预加载 {} 个资源", resource_ids.len());
        Ok(())
    }

    // 设置资源依赖关系
    pub fn set_dependencies(&mut self, 
        resource_id: &str, 
        dependencies: Vec<String>
    ) -> GameResult<()> {
        
        if let Some(resource_info) = self.resources.get(resource_id) {
            let mut info = resource_info.blocking_write();
            info.dependencies = dependencies.clone();
            
            // 更新被依赖资源的dependents列表
            for dep_id in dependencies {
                if let Some(dep_info) = self.resources.get(&dep_id) {
                    let mut dep = dep_info.blocking_write();
                    if !dep.dependents.contains(&resource_id.to_string()) {
                        dep.dependents.push(resource_id.to_string());
                    }
                }
            }
        }
        
        Ok(())
    }

    // 获取资源信息
    pub fn get_resource_info(&self, resource_id: &str) -> Option<Arc<RwLock<ResourceInfo>>> {
        self.resources.get(resource_id).cloned()
    }

    // 检查资源是否已加载
    pub fn is_loaded(&self, resource_id: &str) -> bool {
        if let Some(resource_info) = self.resources.get(resource_id) {
            let info = resource_info.blocking_read();
            info.status == ResourceStatus::Loaded
        } else {
            false
        }
    }

    // 设置缓存策略
    pub fn set_cache_strategy(&mut self, strategy: CacheStrategy) {
        self.cache_strategy = strategy;
        info!("更新缓存策略: 最大内存 {}MB", strategy.max_memory_usage / 1024 / 1024);
    }

    // 添加资源标签
    pub fn add_resource_tag(&mut self, resource_id: &str, tag: String) -> GameResult<()> {
        if let Some(resource_info) = self.resources.get(resource_id) {
            let mut info = resource_info.blocking_write();
            info.tags.insert(tag);
            Ok(())
        } else {
            Err(GameError::Resource(format!("资源不存在: {}", resource_id)))
        }
    }

    // 按标签查找资源
    pub fn find_resources_by_tag(&self, tag: &str) -> Vec<String> {
        self.resources
            .iter()
            .filter_map(|(id, info)| {
                let resource_info = info.blocking_read();
                if resource_info.tags.contains(tag) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    // 获取内存使用统计
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage
    }

    // 获取加载统计
    pub fn get_load_stats(&self) -> (usize, usize, usize) {
        (self.total_resources, self.loaded_resources, self.failed_resources)
    }

    // 私有辅助方法

    // 处理加载队列
    fn process_load_queue(&mut self) -> GameResult<()> {
        while self.current_loads < self.max_concurrent_loads && !self.load_queue.is_empty() {
            if let Some(request) = self.load_queue.pop_front() {
                if !self.loading_resources.contains(&request.resource_id) {
                    // 实际加载逻辑
                    self.loading_resources.insert(request.resource_id.clone());
                    self.current_loads += 1;
                    
                    // 更新资源状态
                    if let Some(resource_info) = self.resources.get(&request.resource_id) {
                        let mut info = resource_info.blocking_write();
                        info.status = ResourceStatus::Loading;
                    }
                }
            }
        }
        Ok(())
    }

    // 尝试开始加载
    fn try_start_load(&mut self, _asset_server: &AssetServer, resource_id: &str) -> GameResult<()> {
        if !self.loading_resources.contains(resource_id) && self.current_loads < self.max_concurrent_loads {
            self.loading_resources.insert(resource_id.to_string());
            self.current_loads += 1;
            
            // 这里应该启动异步加载任务
            // 简化实现
        }
        Ok(())
    }

    // 清理缓存
    fn cleanup_cache(&mut self, current_time: f64) -> GameResult<()> {
        let mut resources_to_unload = Vec::new();
        
        // 查找可以卸载的资源
        for (id, resource_info) in &self.resources {
            let info = resource_info.blocking_read();
            
            // 检查TTL
            if current_time - info.last_accessed > self.cache_strategy.ttl_seconds {
                if info.ref_count == 0 && info.status == ResourceStatus::Loaded {
                    resources_to_unload.push(id.clone());
                }
            }
        }
        
        // 如果内存使用过多，按LRU清理
        if self.memory_usage > self.cache_strategy.max_memory_usage {
            let mut lru_candidates: Vec<_> = self.resources
                .iter()
                .filter(|(_, info)| {
                    let resource_info = info.blocking_read();
                    resource_info.ref_count == 0 && resource_info.status == ResourceStatus::Loaded
                })
                .map(|(id, info)| {
                    let resource_info = info.blocking_read();
                    (id.clone(), resource_info.last_accessed)
                })
                .collect();
                
            lru_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            for (id, _) in lru_candidates {
                resources_to_unload.push(id);
                
                // 预估清理后的内存使用
                if let Some(resource_info) = self.resources.get(&id) {
                    let info = resource_info.blocking_read();
                    if self.memory_usage.saturating_sub(info.size) < self.cache_strategy.max_memory_usage {
                        break;
                    }
                }
            }
        }
        
        // 执行卸载
        for resource_id in resources_to_unload {
            self.unload(&resource_id)?;
        }
        
        if self.memory_usage > self.cache_strategy.max_memory_usage {
            warn!("内存使用仍然过高: {}MB", self.memory_usage / 1024 / 1024);
        }
        
        Ok(())
    }

    // 获取当前时间
    fn get_current_time(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
}

// 便捷函数
impl ResourceManager {
    // 加载纹理
    pub fn load_texture(&mut self, 
        asset_server: &AssetServer,
        resource_id: &str
    ) -> GameResult<ResourceHandle<Image>> {
        self.load_sync(asset_server, resource_id)
    }

    // 加载音频
    pub fn load_audio(&mut self, 
        asset_server: &AssetServer,
        resource_id: &str
    ) -> GameResult<ResourceHandle<AudioSource>> {
        self.load_sync(asset_server, resource_id)
    }

    // 加载字体
    pub fn load_font(&mut self, 
        asset_server: &AssetServer,
        resource_id: &str
    ) -> GameResult<ResourceHandle<Font>> {
        self.load_sync(asset_server, resource_id)
    }

    // 批量注册资源
    pub fn register_resource_batch(&mut self, 
        resources: Vec<(String, ResourceType, String)>
    ) -> GameResult<()> {
        
        for (id, resource_type, path) in resources {
            self.register_resource(id, resource_type, &path)?;
        }
        
        Ok(())
    }

    // 按类型卸载资源
    pub fn unload_by_type(&mut self, resource_type: ResourceType) -> GameResult<()> {
        let resources_to_unload: Vec<String> = self.resources
            .iter()
            .filter_map(|(id, info)| {
                let resource_info = info.blocking_read();
                if resource_info.resource_type == resource_type {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for resource_id in resources_to_unload {
            self.unload(&resource_id)?;
        }

        Ok(())
    }

    // 强制垃圾回收
    pub fn force_garbage_collect(&mut self) -> GameResult<()> {
        self.cleanup_cache(self.get_current_time())?;
        info!("强制垃圾回收完成");
        Ok(())
    }

    // 获取详细统计信息
    pub fn get_detailed_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        stats.insert("total_resources".to_string(), self.total_resources);
        stats.insert("loaded_resources".to_string(), self.loaded_resources);
        stats.insert("failed_resources".to_string(), self.failed_resources);
        stats.insert("memory_usage_mb".to_string(), self.memory_usage / 1024 / 1024);
        stats.insert("loading_queue_size".to_string(), self.load_queue.len());
        stats.insert("current_loads".to_string(), self.current_loads);
        
        // 按类型统计
        let mut type_counts = HashMap::new();
        for resource_info in self.resources.values() {
            let info = resource_info.blocking_read();
            *type_counts.entry(format!("{:?}", info.resource_type)).or_insert(0) += 1;
        }
        
        stats.extend(type_counts);
        stats
    }
}

// Bevy系统实现
pub fn resource_system(
    mut resource_manager: ResMut<ResourceManager>,
) {
    let _ = resource_manager.update();
}

// 资源事件处理系统
pub fn resource_events_system(
    mut resource_events: EventReader<ResourceEvent>,
    mut resource_manager: ResMut<ResourceManager>,
) {
    for event in resource_events.iter() {
        match event {
            ResourceEvent::LoadCompleted(resource_id) => {
                if let Some(resource_info) = resource_manager.resources.get(resource_id) {
                    let mut info = resource_info.blocking_write();
                    info.status = ResourceStatus::Loaded;
                    info.last_accessed = resource_manager.get_current_time();
                }
                resource_manager.loaded_resources += 1;
                resource_manager.current_loads = resource_manager.current_loads.saturating_sub(1);
                resource_manager.loading_resources.remove(resource_id);
            },
            ResourceEvent::LoadFailed(resource_id, error) => {
                if let Some(resource_info) = resource_manager.resources.get(resource_id) {
                    let mut info = resource_info.blocking_write();
                    info.status = ResourceStatus::Failed;
                }
                resource_manager.failed_resources += 1;
                resource_manager.current_loads = resource_manager.current_loads.saturating_sub(1);
                resource_manager.loading_resources.remove(resource_id);
                error!("资源加载失败: {} - {}", resource_id, error);
            },
            _ => {
                debug!("资源事件: {:?}", event);
            }
        }
    }
}