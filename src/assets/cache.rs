// 资源缓存系统 - 智能的LRU缓存和内存管理
// 开发心理：实现高效的缓存机制，平衡内存使用和加载性能
// 设计原则：LRU策略、内存压力感知、统计追踪、线程安全

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime};
use log::{info, debug, warn, error};

// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub size: usize,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub priority: CachePriority,
    pub tags: Vec<String>,
}

impl CacheEntry {
    pub fn new(key: String, data: Vec<u8>, priority: CachePriority) -> Self {
        let now = Instant::now();
        Self {
            size: data.len(),
            key,
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            priority,
            tags: Vec::new(),
        }
    }
    
    pub fn access(&mut self) -> &[u8] {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }
    
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
    
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
    
    pub fn access_frequency(&self) -> f64 {
        let age_secs = self.age().as_secs_f64().max(1.0);
        self.access_count as f64 / age_secs
    }
}

// 缓存优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CachePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl CachePriority {
    pub fn retention_multiplier(&self) -> f64 {
        match self {
            CachePriority::Low => 0.5,
            CachePriority::Normal => 1.0,
            CachePriority::High => 2.0,
            CachePriority::Critical => 5.0,
        }
    }
}

// 缓存统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub insertions: u64,
    pub evictions: u64,
    pub total_requests: u64,
    pub current_size: usize,
    pub max_size: usize,
    pub entry_count: usize,
    pub average_entry_size: usize,
    pub oldest_entry_age: Duration,
    pub memory_pressure: f64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }
    
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
    
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            self.current_size as f64 / self.max_size as f64
        }
    }
}

// LRU访问顺序节点
#[derive(Debug)]
struct LRUNode {
    key: String,
    prev: Option<String>,
    next: Option<String>,
}

// 资源缓存
#[derive(Debug)]
pub struct AssetCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    lru_order: Mutex<HashMap<String, LRUNode>>,
    lru_head: Mutex<Option<String>>,
    lru_tail: Mutex<Option<String>>,
    
    max_size: usize,
    current_size: RwLock<usize>,
    max_entries: usize,
    
    // 统计信息
    stats: RwLock<CacheStats>,
    
    // 清理策略
    cleanup_threshold: f64,  // 触发清理的内存使用率
    cleanup_target: f64,     // 清理后的目标使用率
    min_idle_time: Duration, // 最小空闲时间
    max_age: Duration,       // 最大生存时间
    
    // 内存压力监控
    memory_pressure: RwLock<f64>,
    last_cleanup: RwLock<Instant>,
    cleanup_interval: Duration,
}

impl AssetCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            lru_order: Mutex::new(HashMap::new()),
            lru_head: Mutex::new(None),
            lru_tail: Mutex::new(None),
            
            max_size,
            current_size: RwLock::new(0),
            max_entries: max_size / 1024, // 假设平均1KB per entry
            
            stats: RwLock::new(CacheStats {
                max_size,
                ..Default::default()
            }),
            
            cleanup_threshold: 0.8,
            cleanup_target: 0.6,
            min_idle_time: Duration::from_secs(300), // 5分钟
            max_age: Duration::from_secs(3600),      // 1小时
            
            memory_pressure: RwLock::new(0.0),
            last_cleanup: RwLock::new(Instant::now()),
            cleanup_interval: Duration::from_secs(60), // 1分钟检查间隔
        }
    }
    
    // 插入缓存条目
    pub fn insert(&self, key: String, data: Vec<u8>) {
        self.insert_with_priority(key, data, CachePriority::Normal);
    }
    
    pub fn insert_with_priority(&self, key: String, data: Vec<u8>, priority: CachePriority) {
        let entry_size = data.len();
        
        // 检查是否需要为新条目腾出空间
        if self.should_make_space(entry_size) {
            self.make_space(entry_size);
        }
        
        let entry = CacheEntry::new(key.clone(), data, priority);
        
        // 插入或更新条目
        {
            let mut entries = self.entries.write().unwrap();
            let mut current_size = self.current_size.write().unwrap();
            
            if let Some(old_entry) = entries.insert(key.clone(), entry) {
                *current_size -= old_entry.size;
            }
            *current_size += entry_size;
        }
        
        // 更新LRU顺序
        self.move_to_front(&key);
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().unwrap();
            stats.insertions += 1;
            stats.current_size = *self.current_size.read().unwrap();
            stats.entry_count = self.entries.read().unwrap().len();
            
            if stats.entry_count > 0 {
                stats.average_entry_size = stats.current_size / stats.entry_count;
            }
        }
        
        debug!("缓存插入: {} (大小: {} bytes)", key, entry_size);
    }
    
    // 获取缓存条目
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_requests += 1;
        }
        
        let result = {
            let mut entries = self.entries.write().unwrap();
            if let Some(entry) = entries.get_mut(key) {
                let data = entry.access().to_vec();
                Some(data)
            } else {
                None
            }
        };
        
        match result {
            Some(data) => {
                // 更新LRU顺序
                self.move_to_front(key);
                
                // 更新统计信息
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.hits += 1;
                }
                
                debug!("缓存命中: {}", key);
                Some(data)
            }
            None => {
                // 更新统计信息
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.misses += 1;
                }
                
                debug!("缓存未命中: {}", key);
                None
            }
        }
    }
    
    // 移除缓存条目
    pub fn remove(&self, key: &str) -> Option<Vec<u8>> {
        let result = {
            let mut entries = self.entries.write().unwrap();
            entries.remove(key)
        };
        
        if let Some(entry) = result {
            // 更新当前大小
            {
                let mut current_size = self.current_size.write().unwrap();
                *current_size -= entry.size;
            }
            
            // 从LRU链表中移除
            self.remove_from_lru(key);
            
            // 更新统计信息
            {
                let mut stats = self.stats.write().unwrap();
                stats.evictions += 1;
                stats.current_size = *self.current_size.read().unwrap();
                stats.entry_count = self.entries.read().unwrap().len();
            }
            
            debug!("缓存移除: {}", key);
            Some(entry.data)
        } else {
            None
        }
    }
    
    // 检查是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.entries.read().unwrap().contains_key(key)
    }
    
    // 清空缓存
    pub fn clear(&self) {
        {
            let mut entries = self.entries.write().unwrap();
            let mut current_size = self.current_size.write().unwrap();
            let mut lru_order = self.lru_order.lock().unwrap();
            let mut head = self.lru_head.lock().unwrap();
            let mut tail = self.lru_tail.lock().unwrap();
            
            entries.clear();
            *current_size = 0;
            lru_order.clear();
            *head = None;
            *tail = None;
        }
        
        // 重置统计信息
        {
            let mut stats = self.stats.write().unwrap();
            stats.current_size = 0;
            stats.entry_count = 0;
            stats.average_entry_size = 0;
        }
        
        info!("缓存已清空");
    }
    
    // 检查是否需要为新条目腾出空间
    fn should_make_space(&self, new_entry_size: usize) -> bool {
        let current_size = *self.current_size.read().unwrap();
        let entries_count = self.entries.read().unwrap().len();
        
        current_size + new_entry_size > self.max_size ||
        entries_count >= self.max_entries ||
        self.utilization() > self.cleanup_threshold
    }
    
    // 为新条目腾出空间
    fn make_space(&self, needed_size: usize) {
        let target_size = (self.max_size as f64 * self.cleanup_target) as usize;
        let current_size = *self.current_size.read().unwrap();
        
        if current_size <= target_size {
            return;
        }
        
        let space_to_free = current_size - target_size + needed_size;
        let mut freed_space = 0;
        let mut removed_keys = Vec::new();
        
        // 从尾部开始移除条目（最少使用的）
        let mut current_key = self.lru_tail.lock().unwrap().clone();
        
        while freed_space < space_to_free && current_key.is_some() {
            let key = current_key.unwrap();
            
            // 检查是否可以移除（考虑优先级和最小空闲时间）
            let can_remove = {
                let entries = self.entries.read().unwrap();
                if let Some(entry) = entries.get(&key) {
                    let idle_time = entry.idle_time();
                    let min_idle = Duration::from_secs_f64(
                        self.min_idle_time.as_secs_f64() / entry.priority.retention_multiplier()
                    );
                    
                    idle_time >= min_idle || entry.priority == CachePriority::Low
                } else {
                    false
                }
            };
            
            if can_remove {
                let entry_size = {
                    let entries = self.entries.read().unwrap();
                    entries.get(&key).map(|e| e.size).unwrap_or(0)
                };
                
                removed_keys.push(key.clone());
                freed_space += entry_size;
            }
            
            // 移动到前一个条目
            current_key = {
                let lru_order = self.lru_order.lock().unwrap();
                lru_order.get(&key).and_then(|node| node.prev.clone())
            };
        }
        
        // 移除选中的条目
        for key in removed_keys {
            self.remove(&key);
        }
        
        if freed_space > 0 {
            debug!("为新条目腾出空间: {} bytes", freed_space);
        }
    }
    
    // 将条目移动到LRU链表头部
    fn move_to_front(&self, key: &str) {
        let mut lru_order = self.lru_order.lock().unwrap();
        let mut head = self.lru_head.lock().unwrap();
        let mut tail = self.lru_tail.lock().unwrap();
        
        // 如果条目不在LRU链表中，添加它
        if !lru_order.contains_key(key) {
            let new_node = LRUNode {
                key: key.to_string(),
                prev: None,
                next: head.clone(),
            };
            
            if let Some(ref old_head) = *head {
                if let Some(old_head_node) = lru_order.get_mut(old_head) {
                    old_head_node.prev = Some(key.to_string());
                }
            }
            
            if head.is_none() {
                *tail = Some(key.to_string());
            }
            
            *head = Some(key.to_string());
            lru_order.insert(key.to_string(), new_node);
            
            return;
        }
        
        // 如果已经是头部，无需操作
        if head.as_ref() == Some(&key.to_string()) {
            return;
        }
        
        // 从当前位置移除
        if let Some(node) = lru_order.get(key) {
            let prev = node.prev.clone();
            let next = node.next.clone();
            
            // 更新前一个节点
            if let Some(ref prev_key) = prev {
                if let Some(prev_node) = lru_order.get_mut(prev_key) {
                    prev_node.next = next.clone();
                }
            }
            
            // 更新后一个节点
            if let Some(ref next_key) = next {
                if let Some(next_node) = lru_order.get_mut(next_key) {
                    next_node.prev = prev.clone();
                }
            } else {
                // 这是尾部节点
                *tail = prev;
            }
        }
        
        // 移动到头部
        if let Some(node) = lru_order.get_mut(key) {
            node.prev = None;
            node.next = head.clone();
        }
        
        if let Some(ref old_head) = *head {
            if let Some(old_head_node) = lru_order.get_mut(old_head) {
                old_head_node.prev = Some(key.to_string());
            }
        }
        
        *head = Some(key.to_string());
    }
    
    // 从LRU链表中移除条目
    fn remove_from_lru(&self, key: &str) {
        let mut lru_order = self.lru_order.lock().unwrap();
        let mut head = self.lru_head.lock().unwrap();
        let mut tail = self.lru_tail.lock().unwrap();
        
        if let Some(node) = lru_order.remove(key) {
            // 更新前一个节点
            if let Some(ref prev_key) = node.prev {
                if let Some(prev_node) = lru_order.get_mut(prev_key) {
                    prev_node.next = node.next.clone();
                }
            } else {
                // 这是头部节点
                *head = node.next.clone();
            }
            
            // 更新后一个节点
            if let Some(ref next_key) = node.next {
                if let Some(next_node) = lru_order.get_mut(next_key) {
                    next_node.prev = node.prev.clone();
                }
            } else {
                // 这是尾部节点
                *tail = node.prev.clone();
            }
        }
    }
    
    // 定期清理过期条目
    pub fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let last_cleanup = *self.last_cleanup.read().unwrap();
        
        // 检查是否需要清理
        if now.duration_since(last_cleanup) < self.cleanup_interval {
            return 0;
        }
        
        let mut expired_keys = Vec::new();
        
        // 查找过期条目
        {
            let entries = self.entries.read().unwrap();
            for (key, entry) in entries.iter() {
                if entry.age() > self.max_age ||
                   (entry.priority == CachePriority::Low && entry.idle_time() > self.min_idle_time * 2) {
                    expired_keys.push(key.clone());
                }
            }
        }
        
        // 移除过期条目
        for key in &expired_keys {
            self.remove(key);
        }
        
        // 更新最后清理时间
        {
            let mut last_cleanup = self.last_cleanup.write().unwrap();
            *last_cleanup = now;
        }
        
        if !expired_keys.is_empty() {
            debug!("清理了 {} 个过期缓存条目", expired_keys.len());
        }
        
        expired_keys.len()
    }
    
    // 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        *self.current_size.read().unwrap()
    }
    
    // 获取使用率
    pub fn utilization(&self) -> f64 {
        let current = *self.current_size.read().unwrap();
        current as f64 / self.max_size as f64
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.write().unwrap();
        
        // 更新当前状态
        stats.current_size = *self.current_size.read().unwrap();
        stats.entry_count = self.entries.read().unwrap().len();
        stats.memory_pressure = *self.memory_pressure.read().unwrap();
        
        // 计算平均条目大小
        if stats.entry_count > 0 {
            stats.average_entry_size = stats.current_size / stats.entry_count;
        }
        
        // 计算最老条目的年龄
        stats.oldest_entry_age = {
            let entries = self.entries.read().unwrap();
            entries.values()
                .map(|entry| entry.age())
                .max()
                .unwrap_or(Duration::ZERO)
        };
        
        stats.clone()
    }
    
    // 预热缓存（预加载常用资源）
    pub fn warmup(&self, assets: &[(String, Vec<u8>, CachePriority)]) {
        info!("开始缓存预热，预加载 {} 个资源", assets.len());
        
        for (key, data, priority) in assets {
            self.insert_with_priority(key.clone(), data.clone(), *priority);
        }
        
        info!("缓存预热完成");
    }
    
    // 优化缓存（重新组织LRU顺序）
    pub fn optimize(&self) {
        let start_time = Instant::now();
        
        // 按访问频率重新排序
        let mut entries_by_frequency = Vec::new();
        
        {
            let entries = self.entries.read().unwrap();
            for (key, entry) in entries.iter() {
                entries_by_frequency.push((key.clone(), entry.access_frequency()));
            }
        }
        
        // 按频率排序
        entries_by_frequency.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 重建LRU链表
        {
            let mut lru_order = self.lru_order.lock().unwrap();
            let mut head = self.lru_head.lock().unwrap();
            let mut tail = self.lru_tail.lock().unwrap();
            
            lru_order.clear();
            *head = None;
            *tail = None;
            
            for (key, _) in entries_by_frequency.iter().rev() {
                self.move_to_front(key);
            }
        }
        
        let optimization_time = start_time.elapsed();
        info!("缓存优化完成，耗时: {:?}", optimization_time);
    }
    
    // 设置内存压力
    pub fn set_memory_pressure(&self, pressure: f64) {
        let pressure = pressure.clamp(0.0, 1.0);
        *self.memory_pressure.write().unwrap() = pressure;
        
        // 根据内存压力调整清理阈值
        if pressure > 0.8 {
            // 高内存压力，激进清理
            self.make_space(0);
        } else if pressure > 0.6 {
            // 中等内存压力，预防性清理
            self.cleanup_expired();
        }
    }
    
    // 导出缓存报告
    pub fn export_report(&self) -> Result<String> {
        let stats = self.get_stats();
        let entries = self.entries.read().unwrap();
        
        let mut report = format!(
            "=== 缓存报告 ===\n\
             总请求数: {}\n\
             缓存命中: {} ({:.2}%)\n\
             缓存未命中: {} ({:.2}%)\n\
             当前条目数: {}\n\
             内存使用: {} / {} bytes ({:.1}%)\n\
             平均条目大小: {} bytes\n\
             最老条目年龄: {:?}\n\
             内存压力: {:.1}%\n\n",
            stats.total_requests,
            stats.hits, stats.hit_rate() * 100.0,
            stats.misses, stats.miss_rate() * 100.0,
            stats.entry_count,
            stats.current_size, stats.max_size, stats.utilization() * 100.0,
            stats.average_entry_size,
            stats.oldest_entry_age,
            stats.memory_pressure * 100.0
        );
        
        // 添加热门资源信息
        let mut hot_entries: Vec<_> = entries.iter().collect();
        hot_entries.sort_by(|a, b| b.1.access_count.cmp(&a.1.access_count));
        
        report.push_str("=== 热门资源 (前10) ===\n");
        for (key, entry) in hot_entries.iter().take(10) {
            report.push_str(&format!(
                "{}: {} 次访问, {:.2} Hz, {} bytes, 空闲 {:?}\n",
                key,
                entry.access_count,
                entry.access_frequency(),
                entry.size,
                entry.idle_time()
            ));
        }
        
        Ok(report)
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new(128 * 1024 * 1024) // 默认128MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = AssetCache::new(1024);
        
        // 测试插入和获取
        cache.insert("test1".to_string(), vec![1, 2, 3, 4]);
        assert!(cache.contains("test1"));
        assert_eq!(cache.get("test1"), Some(vec![1, 2, 3, 4]));
        
        // 测试未命中
        assert_eq!(cache.get("nonexistent"), None);
        
        // 测试移除
        cache.remove("test1");
        assert!(!cache.contains("test1"));
    }
    
    #[test]
    fn test_cache_lru_eviction() {
        let cache = AssetCache::new(100); // 很小的缓存
        
        // 插入几个条目
        cache.insert("a".to_string(), vec![0; 30]);
        cache.insert("b".to_string(), vec![0; 30]);
        cache.insert("c".to_string(), vec![0; 30]);
        
        // 访问第一个条目
        cache.get("a");
        
        // 插入新条目，应该触发LRU驱逐
        cache.insert("d".to_string(), vec![0; 30]);
        
        // 检查LRU行为
        assert!(cache.contains("a")); // 最近访问过
        assert!(cache.contains("d")); // 新插入
    }
    
    #[test]
    fn test_cache_priority() {
        let cache = AssetCache::new(100);
        
        // 插入不同优先级的条目
        cache.insert_with_priority("low".to_string(), vec![0; 30], CachePriority::Low);
        cache.insert_with_priority("high".to_string(), vec![0; 30], CachePriority::High);
        cache.insert_with_priority("critical".to_string(), vec![0; 30], CachePriority::Critical);
        
        // 触发空间不足
        cache.insert("new".to_string(), vec![0; 40]);
        
        // 高优先级条目应该保留
        assert!(cache.contains("critical"));
        assert!(cache.contains("high"));
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = AssetCache::new(1024);
        
        cache.insert("test".to_string(), vec![1, 2, 3]);
        cache.get("test"); // hit
        cache.get("nonexistent"); // miss
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hit_rate(), 0.5);
    }
}