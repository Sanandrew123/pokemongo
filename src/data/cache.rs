// 数据缓存系统
// 开发心理：缓存系统提供内存中的数据存储，减少磁盘IO，提升性能
// 设计原则：LRU淘汰、内存限制、线程安全、统计监控

use std::collections::{HashMap, LinkedList};
use std::hash::Hash;
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex, RwLock};
use log::{debug, warn, info};
use crate::core::error::GameError;

// 数据缓存
pub struct DataCache {
    // 缓存存储
    storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
    
    // LRU访问顺序
    access_order: Arc<Mutex<LinkedList<String>>>,
    
    // 缓存配置
    config: CacheConfig,
    
    // 统计信息
    statistics: Arc<Mutex<CacheStatistics>>,
}

// 缓存项
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Arc<dyn Any + Send + Sync>,
    type_id: TypeId,
    size: usize,
    created_at: std::time::Instant,
    last_accessed: std::time::Instant,
    access_count: u64,
    ttl: Option<std::time::Duration>,
}

// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size_bytes: u64,
    pub max_entries: usize,
    pub default_ttl: Option<std::time::Duration>,
    pub cleanup_interval: std::time::Duration,
    pub eviction_policy: EvictionPolicy,
}

// 淘汰策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    LRU,            // 最近最少使用
    LFU,            // 最不常用
    FIFO,           // 先进先出
    TTL,            // 基于过期时间
}

// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_size: u64,
    pub current_entries: usize,
    pub total_sets: u64,
    pub total_gets: u64,
    pub total_removes: u64,
}

impl DataCache {
    pub fn new(max_size_bytes: u64) -> Result<Self, GameError> {
        let config = CacheConfig {
            max_size_bytes,
            max_entries: 10000,
            default_ttl: Some(std::time::Duration::from_secs(3600)), // 1小时
            cleanup_interval: std::time::Duration::from_secs(60),    // 1分钟
            eviction_policy: EvictionPolicy::LRU,
        };
        
        Ok(Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(Mutex::new(LinkedList::new())),
            config,
            statistics: Arc::new(Mutex::new(CacheStatistics::default())),
        })
    }
    
    // 设置缓存项
    pub fn set<T>(&self, key: String, value: T) -> Result<(), GameError>
    where
        T: Clone + Send + Sync + 'static,
    {
        let size = std::mem::size_of::<T>();
        let type_id = TypeId::of::<T>();
        let now = std::time::Instant::now();
        
        let entry = CacheEntry {
            data: Arc::new(value),
            type_id,
            size,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl: self.config.default_ttl,
        };
        
        // 检查是否需要清理空间
        self.ensure_space_available(size)?;
        
        // 存储数据
        {
            let mut storage = self.storage.write().unwrap();
            let mut access_order = self.access_order.lock().unwrap();
            
            // 如果key已存在，从访问顺序中移除
            if storage.contains_key(&key) {
                access_order.retain(|k| k != &key);
            }
            
            storage.insert(key.clone(), entry);
            access_order.push_back(key.clone());
        }
        
        // 更新统计
        {
            let mut stats = self.statistics.lock().unwrap();
            stats.total_sets += 1;
            stats.current_entries = self.storage.read().unwrap().len();
            stats.current_size += size as u64;
        }
        
        debug!("缓存设置: {} ({}字节)", key, size);
        Ok(())
    }
    
    // 获取缓存项
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + 'static,
    {
        let type_id = TypeId::of::<T>();
        let now = std::time::Instant::now();
        
        let mut stats = self.statistics.lock().unwrap();
        stats.total_gets += 1;
        
        let result = {
            let mut storage = self.storage.write().unwrap();
            
            if let Some(entry) = storage.get_mut(key) {
                // 检查类型匹配
                if entry.type_id != type_id {
                    stats.misses += 1;
                    return None;
                }
                
                // 检查TTL
                if let Some(ttl) = entry.ttl {
                    if now.duration_since(entry.created_at) > ttl {
                        stats.misses += 1;
                        return None; // 已过期，但不立即删除
                    }
                }
                
                // 更新访问信息
                entry.last_accessed = now;
                entry.access_count += 1;
                
                // 更新LRU顺序
                {
                    let mut access_order = self.access_order.lock().unwrap();
                    access_order.retain(|k| k != key);
                    access_order.push_back(key.to_string());
                }
                
                // 尝试获取数据
                if let Ok(data) = entry.data.downcast_ref::<T>() {
                    stats.hits += 1;
                    Some(data.clone())
                } else {
                    stats.misses += 1;
                    None
                }
            } else {
                stats.misses += 1;
                None
            }
        };
        
        if result.is_some() {
            debug!("缓存命中: {}", key);
        } else {
            debug!("缓存未命中: {}", key);
        }
        
        result
    }
    
    // 删除缓存项
    pub fn remove(&self, key: &str) -> bool {
        let mut storage = self.storage.write().unwrap();
        let mut access_order = self.access_order.lock().unwrap();
        
        if let Some(entry) = storage.remove(key) {
            access_order.retain(|k| k != key);
            
            // 更新统计
            let mut stats = self.statistics.lock().unwrap();
            stats.total_removes += 1;
            stats.current_entries = storage.len();
            stats.current_size = stats.current_size.saturating_sub(entry.size as u64);
            
            debug!("缓存删除: {}", key);
            true
        } else {
            false
        }
    }
    
    // 检查key是否存在
    pub fn contains_key(&self, key: &str) -> bool {
        let storage = self.storage.read().unwrap();
        storage.contains_key(key)
    }
    
    // 清空缓存
    pub fn clear(&self) {
        let mut storage = self.storage.write().unwrap();
        let mut access_order = self.access_order.lock().unwrap();
        
        storage.clear();
        access_order.clear();
        
        // 重置统计
        let mut stats = self.statistics.lock().unwrap();
        stats.current_size = 0;
        stats.current_entries = 0;
        
        info!("缓存已清空");
    }
    
    // 清理过期项
    pub fn cleanup(&self) -> Result<u64, GameError> {
        let now = std::time::Instant::now();
        let mut expired_keys = Vec::new();
        let mut freed_size = 0u64;
        
        // 查找过期项
        {
            let storage = self.storage.read().unwrap();
            for (key, entry) in storage.iter() {
                if let Some(ttl) = entry.ttl {
                    if now.duration_since(entry.created_at) > ttl {
                        expired_keys.push(key.clone());
                        freed_size += entry.size as u64;
                    }
                }
            }
        }
        
        // 删除过期项
        for key in &expired_keys {
            self.remove(key);
        }
        
        // 更新统计
        if !expired_keys.is_empty() {
            let mut stats = self.statistics.lock().unwrap();
            stats.evictions += expired_keys.len() as u64;
            
            info!("清理过期缓存: {} 项, 释放 {} 字节", expired_keys.len(), freed_size);
        }
        
        Ok(freed_size)
    }
    
    // 强制淘汰
    pub fn evict(&self, count: usize) -> Result<u64, GameError> {
        let mut evicted_size = 0u64;
        let mut evicted_count = 0;
        
        let keys_to_evict = {
            let access_order = self.access_order.lock().unwrap();
            let storage = self.storage.read().unwrap();
            
            let mut keys = Vec::new();
            
            match self.config.eviction_policy {
                EvictionPolicy::LRU => {
                    // 从最少使用的开始淘汰
                    for (i, key) in access_order.iter().enumerate() {
                        if i >= count {
                            break;
                        }
                        if let Some(entry) = storage.get(key) {
                            keys.push((key.clone(), entry.size));
                        }
                    }
                },
                EvictionPolicy::LFU => {
                    // 按访问次数排序，淘汰访问次数最少的
                    let mut entries: Vec<_> = storage.iter()
                        .map(|(k, v)| (k.clone(), v.access_count, v.size))
                        .collect();
                    
                    entries.sort_by_key(|&(_, access_count, _)| access_count);
                    
                    for (key, _, size) in entries.into_iter().take(count) {
                        keys.push((key, size));
                    }
                },
                EvictionPolicy::FIFO => {
                    // 按创建时间排序，淘汰最早创建的
                    let mut entries: Vec<_> = storage.iter()
                        .map(|(k, v)| (k.clone(), v.created_at, v.size))
                        .collect();
                    
                    entries.sort_by_key(|&(_, created_at, _)| created_at);
                    
                    for (key, _, size) in entries.into_iter().take(count) {
                        keys.push((key, size));
                    }
                },
                EvictionPolicy::TTL => {
                    // 优先淘汰即将过期的
                    let now = std::time::Instant::now();
                    let mut entries: Vec<_> = storage.iter()
                        .filter_map(|(k, v)| {
                            v.ttl.map(|ttl| {
                                let remaining = ttl.saturating_sub(now.duration_since(v.created_at));
                                (k.clone(), remaining, v.size)
                            })
                        })
                        .collect();
                    
                    entries.sort_by_key(|&(_, remaining, _)| remaining);
                    
                    for (key, _, size) in entries.into_iter().take(count) {
                        keys.push((key, size));
                    }
                },
            }
            
            keys
        };
        
        // 执行淘汰
        for (key, size) in keys_to_evict {
            if self.remove(&key) {
                evicted_size += size as u64;
                evicted_count += 1;
            }
        }
        
        if evicted_count > 0 {
            let mut stats = self.statistics.lock().unwrap();
            stats.evictions += evicted_count as u64;
            
            info!("强制淘汰缓存: {} 项, 释放 {} 字节", evicted_count, evicted_size);
        }
        
        Ok(evicted_size)
    }
    
    // 获取缓存大小
    pub fn get_size(&self) -> u64 {
        let stats = self.statistics.lock().unwrap();
        stats.current_size
    }
    
    // 获取缓存项数量
    pub fn get_entry_count(&self) -> usize {
        let stats = self.statistics.lock().unwrap();
        stats.current_entries
    }
    
    // 获取缓存统计信息
    pub fn get_statistics(&self) -> CacheStatistics {
        let stats = self.statistics.lock().unwrap();
        stats.clone()
    }
    
    // 获取命中率
    pub fn get_hit_rate(&self) -> f32 {
        let stats = self.statistics.lock().unwrap();
        if stats.total_gets > 0 {
            stats.hits as f32 / stats.total_gets as f32
        } else {
            0.0
        }
    }
    
    // 获取所有键
    pub fn get_keys(&self) -> Vec<String> {
        let storage = self.storage.read().unwrap();
        storage.keys().cloned().collect()
    }
    
    // 获取缓存信息摘要
    pub fn get_info(&self) -> CacheInfo {
        let stats = self.statistics.lock().unwrap();
        
        CacheInfo {
            current_size: stats.current_size,
            max_size: self.config.max_size_bytes,
            current_entries: stats.current_entries,
            max_entries: self.config.max_entries,
            hit_rate: if stats.total_gets > 0 {
                stats.hits as f32 / stats.total_gets as f32
            } else {
                0.0
            },
            eviction_policy: self.config.eviction_policy,
            total_hits: stats.hits,
            total_misses: stats.misses,
            total_evictions: stats.evictions,
        }
    }
    
    // 私有方法
    fn ensure_space_available(&self, needed_size: usize) -> Result<(), GameError> {
        let current_size = self.get_size();
        let current_count = self.get_entry_count();
        
        // 检查大小限制
        if current_size + needed_size as u64 > self.config.max_size_bytes {
            let bytes_to_free = (current_size + needed_size as u64) - self.config.max_size_bytes;
            let items_to_evict = std::cmp::max(1, (bytes_to_free / 1024) as usize); // 估算需要淘汰的项数
            
            self.evict(items_to_evict)?;
        }
        
        // 检查项数限制
        if current_count >= self.config.max_entries {
            self.evict(1)?;
        }
        
        Ok(())
    }
}

// 缓存信息
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub current_size: u64,
    pub max_size: u64,
    pub current_entries: usize,
    pub max_entries: usize,
    pub hit_rate: f32,
    pub eviction_policy: EvictionPolicy,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
}

// 缓存构建器
pub struct CacheBuilder {
    max_size_bytes: u64,
    max_entries: usize,
    default_ttl: Option<std::time::Duration>,
    eviction_policy: EvictionPolicy,
}

impl CacheBuilder {
    pub fn new() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            max_entries: 10000,
            default_ttl: Some(std::time::Duration::from_secs(3600)),
            eviction_policy: EvictionPolicy::LRU,
        }
    }
    
    pub fn max_size(mut self, size: u64) -> Self {
        self.max_size_bytes = size;
        self
    }
    
    pub fn max_entries(mut self, count: usize) -> Self {
        self.max_entries = count;
        self
    }
    
    pub fn default_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }
    
    pub fn eviction_policy(mut self, policy: EvictionPolicy) -> Self {
        self.eviction_policy = policy;
        self
    }
    
    pub fn build(self) -> Result<DataCache, GameError> {
        let config = CacheConfig {
            max_size_bytes: self.max_size_bytes,
            max_entries: self.max_entries,
            default_ttl: self.default_ttl,
            cleanup_interval: std::time::Duration::from_secs(60),
            eviction_policy: self.eviction_policy,
        };
        
        Ok(DataCache {
            storage: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(Mutex::new(LinkedList::new())),
            config,
            statistics: Arc::new(Mutex::new(CacheStatistics::default())),
        })
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_creation() {
        let cache = DataCache::new(1024 * 1024).unwrap();
        assert_eq!(cache.get_entry_count(), 0);
        assert_eq!(cache.get_size(), 0);
    }
    
    #[test]
    fn test_cache_set_get() {
        let cache = DataCache::new(1024 * 1024).unwrap();
        
        cache.set("test_key".to_string(), "test_value".to_string()).unwrap();
        
        let value: Option<String> = cache.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
        
        assert_eq!(cache.get_entry_count(), 1);
        assert!(cache.get_size() > 0);
    }
    
    #[test]
    fn test_cache_remove() {
        let cache = DataCache::new(1024 * 1024).unwrap();
        
        cache.set("test_key".to_string(), "test_value".to_string()).unwrap();
        assert!(cache.contains_key("test_key"));
        
        let removed = cache.remove("test_key");
        assert!(removed);
        assert!(!cache.contains_key("test_key"));
        
        let value: Option<String> = cache.get("test_key");
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_cache_clear() {
        let cache = DataCache::new(1024 * 1024).unwrap();
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        cache.set("key2".to_string(), "value2".to_string()).unwrap();
        
        assert_eq!(cache.get_entry_count(), 2);
        
        cache.clear();
        
        assert_eq!(cache.get_entry_count(), 0);
        assert_eq!(cache.get_size(), 0);
    }
    
    #[test]
    fn test_cache_statistics() {
        let cache = DataCache::new(1024 * 1024).unwrap();
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        let _value: Option<String> = cache.get("key1");
        let _none: Option<String> = cache.get("nonexistent");
        
        let stats = cache.get_statistics();
        assert_eq!(stats.total_sets, 1);
        assert_eq!(stats.total_gets, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        
        let hit_rate = cache.get_hit_rate();
        assert_eq!(hit_rate, 0.5);
    }
    
    #[test]
    fn test_cache_builder() {
        let cache = CacheBuilder::new()
            .max_size(2 * 1024 * 1024)
            .max_entries(5000)
            .eviction_policy(EvictionPolicy::LFU)
            .build()
            .unwrap();
        
        let info = cache.get_info();
        assert_eq!(info.max_size, 2 * 1024 * 1024);
        assert_eq!(info.max_entries, 5000);
        assert_eq!(info.eviction_policy, EvictionPolicy::LFU);
    }
}