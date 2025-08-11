/*
* 开发心理过程：
* 1. 设计高质量的随机数生成器，支持多种分布和用途
* 2. 实现可重现的随机序列，便于测试和调试
* 3. 支持噪声函数生成，用于地形和程序化内容
* 4. 提供权重选择算法，用于Pokemon生成和物品掉落
* 5. 集成性能优化，支持大量随机数快速生成
* 6. 实现线程安全的随机数池，支持多线程场景
* 7. 提供统计功能，帮助平衡游戏内容
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

/// 高性能随机数生成器
#[derive(Debug, Clone)]
pub struct RandomGenerator {
    /// 主随机数生成器
    rng: StdRng,
    /// 种子值
    seed: u64,
    /// 生成统计
    stats: RandomStats,
    /// 噪声生成器配置
    noise_config: NoiseConfig,
}

#[derive(Debug, Clone, Default)]
pub struct RandomStats {
    /// 总生成次数
    pub total_generations: u64,
    /// 各类型生成次数
    pub generation_counts: HashMap<String, u64>,
    /// 平均值统计
    pub average_values: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct NoiseConfig {
    /// 八度数
    pub octaves: u8,
    /// 频率
    pub frequency: f64,
    /// 持久性
    pub persistence: f64,
    /// 种子偏移
    pub seed_offset: u32,
}

#[derive(Debug, Clone)]
pub struct WeightedItem<T> {
    pub item: T,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct RandomRange<T> {
    pub min: T,
    pub max: T,
}

impl RandomGenerator {
    /// 创建新的随机数生成器
    pub fn new() -> Self {
        let seed = rand::random::<u64>();
        Self::with_seed(seed)
    }

    /// 使用指定种子创建
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
            stats: RandomStats::default(),
            noise_config: NoiseConfig::default(),
        }
    }

    /// 获取种子
    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    /// 重置种子
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = StdRng::seed_from_u64(seed);
        self.stats = RandomStats::default();
    }

    /// 生成随机布尔值
    pub fn bool(&mut self) -> bool {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("bool".to_string()).or_insert(0) += 1;
        self.rng.gen()
    }

    /// 基于概率生成布尔值
    pub fn probability(&mut self) -> f32 {
        self.range_f32(0.0, 1.0)
    }

    /// 基于概率检查
    pub fn chance(&mut self, probability: f32) -> bool {
        self.probability() < probability.clamp(0.0, 1.0)
    }

    /// 生成指定范围的整数 [min, max)
    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("range_i32".to_string()).or_insert(0) += 1;
        self.rng.gen_range(min..max)
    }

    /// 生成指定范围的整数 [min, max] (包含max)
    pub fn range_inclusive(&mut self, min: i32, max: i32) -> i32 {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("range_i32_inclusive".to_string()).or_insert(0) += 1;
        self.rng.gen_range(min..=max)
    }

    /// 生成指定范围的浮点数
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("range_f32".to_string()).or_insert(0) += 1;
        self.rng.gen_range(min..max)
    }

    /// 生成指定范围的浮点数
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("range_f64".to_string()).or_insert(0) += 1;
        self.rng.gen_range(min..max)
    }

    /// 生成0-1之间的浮点数
    pub fn unit_f32(&mut self) -> f32 {
        self.range_f32(0.0, 1.0)
    }

    /// 生成0-1之间的浮点数
    pub fn unit_f64(&mut self) -> f64 {
        self.range_f64(0.0, 1.0)
    }

    /// 生成正态分布随机数
    pub fn normal(&mut self, mean: f32, std_dev: f32) -> f32 {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("normal".to_string()).or_insert(0) += 1;
        
        // Box-Muller变换
        static mut NEXT_GAUSSIAN: Option<f64> = None;
        static mut HAS_NEXT: bool = false;
        
        unsafe {
            if HAS_NEXT {
                HAS_NEXT = false;
                return (NEXT_GAUSSIAN.unwrap() * std_dev as f64 + mean as f64) as f32;
            } else {
                let u1 = self.unit_f64();
                let u2 = self.unit_f64();
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
                
                NEXT_GAUSSIAN = Some(z1);
                HAS_NEXT = true;
                
                return (z0 * std_dev as f64 + mean as f64) as f32;
            }
        }
    }

    /// 从切片中随机选择一个元素
    pub fn choose<T>(&mut self, items: &[T]) -> Option<&T> {
        if items.is_empty() {
            return None;
        }
        
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("choose".to_string()).or_insert(0) += 1;
        
        items.choose(&mut self.rng)
    }

    /// 从切片中随机选择多个元素（不重复）
    pub fn choose_multiple<T>(&mut self, items: &[T], amount: usize) -> Vec<&T> {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("choose_multiple".to_string()).or_insert(0) += 1;
        
        items.choose_multiple(&mut self.rng, amount).collect()
    }

    /// 基于权重选择元素
    pub fn weighted_choose<T>(&mut self, items: &[WeightedItem<T>]) -> Option<&T> {
        if items.is_empty() {
            return None;
        }

        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("weighted_choose".to_string()).or_insert(0) += 1;

        let total_weight: f32 = items.iter().map(|item| item.weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut random_weight = self.range_f32(0.0, total_weight);
        
        for item in items {
            random_weight -= item.weight;
            if random_weight <= 0.0 {
                return Some(&item.item);
            }
        }

        // fallback: 返回最后一个元素
        items.last().map(|item| &item.item)
    }

    /// 基于权重选择多个元素
    pub fn weighted_choose_multiple<T>(&mut self, items: &[WeightedItem<T>], amount: usize) -> Vec<&T> {
        let mut selected = Vec::new();
        let mut remaining_items = items.to_vec();

        for _ in 0..amount.min(items.len()) {
            if let Some(chosen) = self.weighted_choose(&remaining_items) {
                // 找到选中的元素并添加到结果
                if let Some(pos) = remaining_items.iter().position(|item| std::ptr::eq(&item.item, chosen)) {
                    selected.push(chosen);
                    remaining_items.remove(pos);
                }
            }
        }

        selected
    }

    /// 随机打乱切片
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("shuffle".to_string()).or_insert(0) += 1;
        items.shuffle(&mut self.rng);
    }

    /// 生成随机字符串
    pub fn string(&mut self, length: usize, charset: &str) -> String {
        self.stats.total_generations += 1;
        *self.stats.generation_counts.entry("string".to_string()).or_insert(0) += 1;
        
        let chars: Vec<char> = charset.chars().collect();
        (0..length)
            .map(|_| *self.choose(&chars).unwrap_or(&'?'))
            .collect()
    }

    /// 生成随机UUID字符串
    pub fn uuid_string(&mut self) -> String {
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.rng.gen::<u32>(),
            self.rng.gen::<u16>(),
            (self.rng.gen::<u16>() & 0x0FFF) | 0x4000, // Version 4
            (self.rng.gen::<u16>() & 0x3FFF) | 0x8000, // Variant
            self.rng.gen::<u64>() & 0xFFFFFFFFFFFF
        )
    }

    /// 2D Perlin噪声
    pub fn noise_2d(&mut self, x: f32, y: f32) -> f32 {
        self.perlin_noise_2d(x, y, &self.noise_config.clone())
    }

    /// 带配置的2D Perlin噪声
    pub fn noise_2d_with_config(&mut self, x: f32, y: f32, config: &NoiseConfig) -> f32 {
        self.perlin_noise_2d(x, y, config)
    }

    /// 分层噪声（多八度）
    pub fn fractal_noise_2d(&mut self, x: f32, y: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.noise_config.frequency;
        
        for _ in 0..self.noise_config.octaves {
            value += amplitude * self.perlin_noise_2d(x * frequency as f32, y * frequency as f32, &self.noise_config);
            amplitude *= self.noise_config.persistence as f32;
            frequency *= 2.0;
        }
        
        value
    }

    /// 简化的Perlin噪声实现
    fn perlin_noise_2d(&mut self, x: f32, y: f32, config: &NoiseConfig) -> f32 {
        // 简化的噪声实现，实际可以使用更复杂的算法
        let xi = (x.floor() as i32) & 255;
        let yi = (y.floor() as i32) & 255;
        
        let xf = x - x.floor();
        let yf = y - y.floor();
        
        // 简化的梯度查找
        let aa = self.noise_hash(xi, yi, config.seed_offset);
        let ab = self.noise_hash(xi, yi + 1, config.seed_offset);
        let ba = self.noise_hash(xi + 1, yi, config.seed_offset);
        let bb = self.noise_hash(xi + 1, yi + 1, config.seed_offset);
        
        // 双线性插值
        let u = self.fade(xf);
        let v = self.fade(yf);
        
        let x1 = self.lerp(aa as f32, ba as f32, u);
        let x2 = self.lerp(ab as f32, bb as f32, u);
        self.lerp(x1, x2, v) * 2.0 - 1.0 // 转换到 [-1, 1] 范围
    }

    fn noise_hash(&self, x: i32, y: i32, seed: u32) -> u32 {
        let mut hash = seed;
        hash ^= (x as u32).wrapping_mul(1619);
        hash ^= (y as u32).wrapping_mul(31337);
        hash ^= hash >> 16;
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0xc2b2ae35);
        hash ^= hash >> 16;
        hash & 0xFF
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// 生成泊松分布随机数
    pub fn poisson(&mut self, lambda: f32) -> u32 {
        // Knuth算法
        let l = (-lambda).exp();
        let mut k = 0;
        let mut p = 1.0;
        
        loop {
            k += 1;
            p *= self.unit_f32();
            if p <= l {
                break;
            }
        }
        
        k - 1
    }

    /// 生成指数分布随机数
    pub fn exponential(&mut self, lambda: f32) -> f32 {
        -self.unit_f32().ln() / lambda
    }

    /// 生成伽马分布随机数（简化版）
    pub fn gamma(&mut self, shape: f32, scale: f32) -> f32 {
        // 简化实现，仅适用于shape >= 1
        if shape < 1.0 {
            return self.gamma(shape + 1.0, scale) * self.unit_f32().powf(1.0 / shape);
        }
        
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        
        loop {
            let x = self.normal(0.0, 1.0);
            let v = 1.0 + c * x;
            if v > 0.0 {
                let v = v * v * v;
                let u = self.unit_f32();
                let x2 = x * x;
                if u < 1.0 - 0.331 * x2 * x2 || u.ln() < 0.5 * x2 + d * (1.0 - v + v.ln()) {
                    return d * v * scale;
                }
            }
        }
    }

    /// 设置噪声配置
    pub fn set_noise_config(&mut self, config: NoiseConfig) {
        self.noise_config = config;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &RandomStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = RandomStats::default();
    }

    /// 生成随机种子集合（用于多线程）
    pub fn generate_seed_pool(&mut self, count: usize) -> Vec<u64> {
        (0..count).map(|_| self.rng.gen()).collect()
    }
}

impl Default for RandomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            octaves: 4,
            frequency: 0.01,
            persistence: 0.5,
            seed_offset: 0,
        }
    }
}

impl<T> WeightedItem<T> {
    pub fn new(item: T, weight: f32) -> Self {
        Self { item, weight }
    }
}

impl<T> RandomRange<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

/// 线程安全的随机数管理器
#[derive(Debug)]
pub struct RandomManager {
    seed_pool: Vec<u64>,
    next_seed_index: std::sync::atomic::AtomicUsize,
}

impl RandomManager {
    pub fn new(pool_size: usize) -> Self {
        let mut rng = RandomGenerator::new();
        let seed_pool = rng.generate_seed_pool(pool_size);
        
        Self {
            seed_pool,
            next_seed_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// 获取一个新的随机数生成器
    pub fn get_generator(&self) -> RandomGenerator {
        let index = self.next_seed_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let seed = self.seed_pool[index % self.seed_pool.len()];
        RandomGenerator::with_seed(seed)
    }

    /// 获取指定索引的种子
    pub fn get_seed(&self, index: usize) -> u64 {
        self.seed_pool[index % self.seed_pool.len()]
    }
}

/// 随机数工具函数
pub mod utils {
    use super::*;

    /// 生成随机颜色
    pub fn random_color(rng: &mut RandomGenerator) -> bevy::prelude::Color {
        bevy::prelude::Color::rgba(
            rng.unit_f32(),
            rng.unit_f32(),
            rng.unit_f32(),
            1.0,
        )
    }

    /// 生成随机2D向量
    pub fn random_vec2(rng: &mut RandomGenerator) -> bevy::prelude::Vec2 {
        bevy::prelude::Vec2::new(
            rng.range_f32(-1.0, 1.0),
            rng.range_f32(-1.0, 1.0),
        )
    }

    /// 生成单位圆内的随机点
    pub fn random_point_in_circle(rng: &mut RandomGenerator, radius: f32) -> bevy::prelude::Vec2 {
        let angle = rng.range_f32(0.0, 2.0 * std::f32::consts::PI);
        let distance = rng.unit_f32().sqrt() * radius;
        bevy::prelude::Vec2::new(
            angle.cos() * distance,
            angle.sin() * distance,
        )
    }

    /// 生成矩形内的随机点
    pub fn random_point_in_rect(
        rng: &mut RandomGenerator,
        min: bevy::prelude::Vec2,
        max: bevy::prelude::Vec2,
    ) -> bevy::prelude::Vec2 {
        bevy::prelude::Vec2::new(
            rng.range_f32(min.x, max.x),
            rng.range_f32(min.y, max.y),
        )
    }

    /// 基于权重创建加权项目列表
    pub fn create_weighted_list<T>(items: Vec<(T, f32)>) -> Vec<WeightedItem<T>> {
        items
            .into_iter()
            .map(|(item, weight)| WeightedItem::new(item, weight))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_generator() {
        let mut rng = RandomGenerator::with_seed(12345);
        
        // 测试基本随机数生成
        let val1 = rng.range(1, 10);
        assert!(val1 >= 1 && val1 < 10);
        
        let val2 = rng.range_f32(0.0, 1.0);
        assert!(val2 >= 0.0 && val2 < 1.0);
        
        // 测试可重现性
        let mut rng2 = RandomGenerator::with_seed(12345);
        assert_eq!(rng2.range(1, 10), val1);
    }

    #[test]
    fn test_weighted_choice() {
        let mut rng = RandomGenerator::with_seed(54321);
        let items = vec![
            WeightedItem::new("rare", 1.0),
            WeightedItem::new("common", 10.0),
        ];
        
        let mut common_count = 0;
        let mut rare_count = 0;
        
        for _ in 0..1000 {
            match rng.weighted_choose(&items) {
                Some(&"common") => common_count += 1,
                Some(&"rare") => rare_count += 1,
                _ => {},
            }
        }
        
        // common应该比rare出现更频繁
        assert!(common_count > rare_count);
        assert!(common_count > 800); // 大约90%概率
    }

    #[test]
    fn test_normal_distribution() {
        let mut rng = RandomGenerator::new();
        let values: Vec<f32> = (0..1000).map(|_| rng.normal(0.0, 1.0)).collect();
        
        let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
        let variance: f32 = values.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / values.len() as f32;
        
        // 检查均值接近0，方差接近1
        assert!((mean - 0.0).abs() < 0.1);
        assert!((variance - 1.0).abs() < 0.2);
    }

    #[test]
    fn test_noise_generation() {
        let mut rng = RandomGenerator::new();
        
        let noise1 = rng.noise_2d(0.0, 0.0);
        let noise2 = rng.noise_2d(1.0, 0.0);
        let noise3 = rng.noise_2d(0.0, 1.0);
        
        // 噪声值应该在[-1, 1]范围内
        assert!(noise1 >= -1.0 && noise1 <= 1.0);
        assert!(noise2 >= -1.0 && noise2 <= 1.0);
        assert!(noise3 >= -1.0 && noise3 <= 1.0);
        
        // 不同位置的噪声值应该不同（大概率）
        assert_ne!(noise1, noise2);
        assert_ne!(noise1, noise3);
    }

    #[test]
    fn test_shuffle() {
        let mut rng = RandomGenerator::with_seed(98765);
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();
        
        rng.shuffle(&mut items);
        
        // 应该包含相同的元素但顺序不同（大概率）
        assert_eq!(items.len(), original.len());
        for item in &original {
            assert!(items.contains(item));
        }
        // 大概率不会完全相同
        assert_ne!(items, original);
    }

    #[test]
    fn test_probability() {
        let mut rng = RandomGenerator::new();
        let mut success_count = 0;
        
        for _ in 0..1000 {
            if rng.chance(0.3) {
                success_count += 1;
            }
        }
        
        // 30%概率，期望约300次成功
        assert!(success_count > 200 && success_count < 400);
    }

    #[test]
    fn test_string_generation() {
        let mut rng = RandomGenerator::new();
        let charset = "abcdefghijklmnopqrstuvwxyz";
        
        let string1 = rng.string(10, charset);
        let string2 = rng.string(10, charset);
        
        assert_eq!(string1.len(), 10);
        assert_eq!(string2.len(), 10);
        assert_ne!(string1, string2); // 大概率不同
        
        // 检查字符都在字符集内
        for ch in string1.chars() {
            assert!(charset.contains(ch));
        }
    }

    #[test]
    fn test_random_manager() {
        let manager = RandomManager::new(10);
        let mut gen1 = manager.get_generator();
        let mut gen2 = manager.get_generator();
        
        // 不同生成器应该产生不同的随机数序列
        let val1 = gen1.range(1, 1000);
        let val2 = gen2.range(1, 1000);
        
        // 大概率不同
        assert_ne!(val1, val2);
    }
}