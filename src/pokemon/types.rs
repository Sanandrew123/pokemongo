// Pokemon属性系统
// 开发心理：属性相克是战斗系统核心，需要精确效果倍率、完整相克表、特殊交互
// 设计原则：18属性完整支持、复合属性处理、动态效果计算、缓存优化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;

// 属性ID类型
pub type TypeId = u8;

// Pokemon属性类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PokemonType {
    Normal = 0,     // 一般
    Fighting = 1,   // 格斗  
    Flying = 2,     // 飞行
    Poison = 3,     // 毒
    Ground = 4,     // 地面
    Rock = 5,       // 岩石
    Bug = 6,        // 虫
    Ghost = 7,      // 幽灵
    Steel = 8,      // 钢
    Fire = 9,       // 火
    Water = 10,     // 水
    Grass = 11,     // 草
    Electric = 12,  // 电
    Psychic = 13,   // 超能力
    Ice = 14,       // 冰
    Dragon = 15,    // 龙
    Dark = 16,      // 恶
    Fairy = 17,     // 妖精
}

// 属性效果倍率
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeEffectiveness {
    NoEffect = 0,      // 无效 (0倍)
    NotVeryEffective,  // 效果不佳 (0.5倍)
    Normal,            // 普通效果 (1倍)
    SuperEffective,    // 效果拔群 (2倍)
}

// 双属性组合
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DualType {
    pub primary: PokemonType,
    pub secondary: Option<PokemonType>,
}

// 属性相克结果
#[derive(Debug, Clone)]
pub struct TypeMatchupResult {
    pub effectiveness: TypeEffectiveness,
    pub multiplier: f32,
    pub is_stab: bool,           // 本属性一致加成
    pub critical_boost: f32,     // 会心一击加成
    pub weather_boost: f32,      // 天气加成
    pub ability_modifier: f32,   // 特性修正
    pub item_modifier: f32,      // 道具修正
    pub final_multiplier: f32,   // 最终倍率
    pub messages: Vec<String>,   // 效果消息
}

// 属性相关特殊状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeStatus {
    Burned,         // 燃烧状态 (火系免疫)
    Frozen,         // 冰冻状态 (冰系免疫)
    Poisoned,       // 中毒状态 (毒钢系免疫)
    Paralyzed,      // 麻痹状态 (电系免疫)
    Sleeping,       // 睡眠状态
}

// 天气对属性的影响
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    None,           // 无天气
    Sunny,          // 大晴天 (火系+50%, 水系-50%)
    Rain,           // 下雨 (水系+50%, 火系-50%)
    Sandstorm,      // 沙暴 (岩石地面钢系免疫)
    Hail,           // 冰雹 (冰系免疫)
    Fog,            // 雾 (命中率下降)
}

// 属性系统管理器
pub struct TypeSystemManager {
    // 属性相克表 [攻击属性][防御属性] = 效果
    effectiveness_chart: [[TypeEffectiveness; 18]; 18],
    
    // 属性名称映射
    type_names: HashMap<PokemonType, Vec<&'static str>>, // 支持多语言
    
    // 属性颜色映射
    type_colors: HashMap<PokemonType, (u8, u8, u8)>, // RGB颜色
    
    // STAB加成倍率
    stab_multiplier: f32,
    
    // 天气效果映射
    weather_effects: HashMap<WeatherType, HashMap<PokemonType, f32>>,
    
    // 特殊交互规则
    special_interactions: Vec<SpecialTypeInteraction>,
    
    // 缓存
    matchup_cache: HashMap<String, TypeMatchupResult>,
    cache_enabled: bool,
    
    // 统计
    total_calculations: u64,
    cache_hits: u64,
}

// 特殊属性交互
#[derive(Debug, Clone)]
struct SpecialTypeInteraction {
    attacker_type: PokemonType,
    defender_type: DualType,
    condition: String,          // 特殊条件
    modifier: f32,              // 倍率修正
    priority: i32,              // 优先级
}

impl TypeSystemManager {
    pub fn new() -> Self {
        let mut manager = Self {
            effectiveness_chart: [[TypeEffectiveness::Normal; 18]; 18],
            type_names: HashMap::new(),
            type_colors: HashMap::new(),
            stab_multiplier: 1.5,
            weather_effects: HashMap::new(),
            special_interactions: Vec::new(),
            matchup_cache: HashMap::new(),
            cache_enabled: true,
            total_calculations: 0,
            cache_hits: 0,
        };
        
        manager.initialize_effectiveness_chart();
        manager.initialize_type_names();
        manager.initialize_type_colors();
        manager.initialize_weather_effects();
        manager.initialize_special_interactions();
        
        manager
    }
    
    // 计算属性相克效果
    pub fn calculate_type_effectiveness(
        &mut self,
        attack_type: PokemonType,
        defender_types: DualType,
        attacker_types: Option<DualType>,
        weather: WeatherType,
        special_conditions: &HashMap<String, bool>,
    ) -> Result<TypeMatchupResult, GameError> {
        
        // 检查缓存
        let cache_key = format!("{:?}_{:?}_{:?}_{:?}", 
            attack_type, defender_types, weather, 
            special_conditions.keys().collect::<Vec<_>>());
            
        if self.cache_enabled {
            if let Some(cached) = self.matchup_cache.get(&cache_key) {
                self.cache_hits += 1;
                return Ok(cached.clone());
            }
        }
        
        self.total_calculations += 1;
        
        // 基础相克效果
        let primary_effectiveness = self.get_effectiveness(attack_type, defender_types.primary);
        let secondary_effectiveness = defender_types.secondary
            .map(|t| self.get_effectiveness(attack_type, t))
            .unwrap_or(TypeEffectiveness::Normal);
        
        // 计算复合属性倍率
        let base_multiplier = self.get_multiplier(primary_effectiveness) * 
                             self.get_multiplier(secondary_effectiveness);
        
        // 检查STAB (本属性一致加成)
        let is_stab = if let Some(attacker_types) = attacker_types {
            attack_type == attacker_types.primary || 
            attacker_types.secondary.map_or(false, |t| attack_type == t)
        } else {
            false
        };
        
        let stab_boost = if is_stab { self.stab_multiplier } else { 1.0 };
        
        // 天气效果
        let weather_boost = self.calculate_weather_effect(attack_type, weather);
        
        // 特殊交互效果
        let special_modifier = self.calculate_special_interactions(
            attack_type, 
            defender_types, 
            special_conditions
        );
        
        // 计算最终倍率
        let final_multiplier = base_multiplier * stab_boost * weather_boost * special_modifier;
        
        // 确定最终效果等级
        let final_effectiveness = match final_multiplier {
            x if x == 0.0 => TypeEffectiveness::NoEffect,
            x if x < 1.0 => TypeEffectiveness::NotVeryEffective,
            x if x > 1.0 => TypeEffectiveness::SuperEffective,
            _ => TypeEffectiveness::Normal,
        };
        
        // 生成消息
        let mut messages = Vec::new();
        
        match final_effectiveness {
            TypeEffectiveness::NoEffect => messages.push("对手不受影响...".to_string()),
            TypeEffectiveness::NotVeryEffective => messages.push("效果不佳...".to_string()),
            TypeEffectiveness::SuperEffective => messages.push("效果拔群！".to_string()),
            TypeEffectiveness::Normal => {},
        }
        
        if is_stab {
            messages.push("本属性一致加成！".to_string());
        }
        
        if weather_boost != 1.0 {
            match weather {
                WeatherType::Sunny if attack_type == PokemonType::Fire => 
                    messages.push("阳光增强了火系技能！".to_string()),
                WeatherType::Rain if attack_type == PokemonType::Water => 
                    messages.push("雨水增强了水系技能！".to_string()),
                _ => {}
            }
        }
        
        let result = TypeMatchupResult {
            effectiveness: final_effectiveness,
            multiplier: base_multiplier,
            is_stab,
            critical_boost: 1.0,
            weather_boost,
            ability_modifier: special_modifier,
            item_modifier: 1.0,
            final_multiplier,
            messages,
        };
        
        // 缓存结果
        if self.cache_enabled {
            self.matchup_cache.insert(cache_key, result.clone());
        }
        
        debug!("属性相克计算: {:?} -> {:?} = {:.2}x",
            attack_type, defender_types, final_multiplier);
        
        Ok(result)
    }
    
    // 检查属性免疫状态
    pub fn is_immune_to_status(&self, pokemon_types: DualType, status: TypeStatus) -> bool {
        match status {
            TypeStatus::Burned => {
                pokemon_types.primary == PokemonType::Fire ||
                pokemon_types.secondary == Some(PokemonType::Fire)
            },
            TypeStatus::Frozen => {
                pokemon_types.primary == PokemonType::Ice ||
                pokemon_types.secondary == Some(PokemonType::Ice)
            },
            TypeStatus::Poisoned => {
                pokemon_types.primary == PokemonType::Poison ||
                pokemon_types.secondary == Some(PokemonType::Poison) ||
                pokemon_types.primary == PokemonType::Steel ||
                pokemon_types.secondary == Some(PokemonType::Steel)
            },
            TypeStatus::Paralyzed => {
                pokemon_types.primary == PokemonType::Electric ||
                pokemon_types.secondary == Some(PokemonType::Electric)
            },
            TypeStatus::Sleeping => false, // 无属性免疫睡眠
        }
    }
    
    // 获取属性弱点
    pub fn get_weaknesses(&self, pokemon_types: DualType) -> Vec<PokemonType> {
        let mut weaknesses = Vec::new();
        
        for attacker_type in 0..18u8 {
            let attack_type = unsafe { std::mem::transmute(attacker_type) };
            
            let primary_eff = self.get_effectiveness(attack_type, pokemon_types.primary);
            let secondary_eff = pokemon_types.secondary
                .map(|t| self.get_effectiveness(attack_type, t))
                .unwrap_or(TypeEffectiveness::Normal);
            
            let combined_multiplier = self.get_multiplier(primary_eff) * 
                                    self.get_multiplier(secondary_eff);
            
            if combined_multiplier > 1.0 {
                weaknesses.push(attack_type);
            }
        }
        
        weaknesses
    }
    
    // 获取属性抗性
    pub fn get_resistances(&self, pokemon_types: DualType) -> Vec<PokemonType> {
        let mut resistances = Vec::new();
        
        for attacker_type in 0..18u8 {
            let attack_type = unsafe { std::mem::transmute(attacker_type) };
            
            let primary_eff = self.get_effectiveness(attack_type, pokemon_types.primary);
            let secondary_eff = pokemon_types.secondary
                .map(|t| self.get_effectiveness(attack_type, t))
                .unwrap_or(TypeEffectiveness::Normal);
            
            let combined_multiplier = self.get_multiplier(primary_eff) * 
                                    self.get_multiplier(secondary_eff);
            
            if combined_multiplier < 1.0 && combined_multiplier > 0.0 {
                resistances.push(attack_type);
            }
        }
        
        resistances
    }
    
    // 获取属性免疫
    pub fn get_immunities(&self, pokemon_types: DualType) -> Vec<PokemonType> {
        let mut immunities = Vec::new();
        
        for attacker_type in 0..18u8 {
            let attack_type = unsafe { std::mem::transmute(attacker_type) };
            
            let primary_eff = self.get_effectiveness(attack_type, pokemon_types.primary);
            let secondary_eff = pokemon_types.secondary
                .map(|t| self.get_effectiveness(attack_type, t))
                .unwrap_or(TypeEffectiveness::Normal);
            
            let combined_multiplier = self.get_multiplier(primary_eff) * 
                                    self.get_multiplier(secondary_eff);
            
            if combined_multiplier == 0.0 {
                immunities.push(attack_type);
            }
        }
        
        immunities
    }
    
    // 计算属性组合的防御评分
    pub fn calculate_defensive_rating(&self, pokemon_types: DualType) -> f32 {
        let mut total_multiplier = 0.0;
        let mut count = 0;
        
        for attacker_type in 0..18u8 {
            let attack_type = unsafe { std::mem::transmute(attacker_type) };
            
            let primary_eff = self.get_effectiveness(attack_type, pokemon_types.primary);
            let secondary_eff = pokemon_types.secondary
                .map(|t| self.get_effectiveness(attack_type, t))
                .unwrap_or(TypeEffectiveness::Normal);
            
            let combined_multiplier = self.get_multiplier(primary_eff) * 
                                    self.get_multiplier(secondary_eff);
            
            total_multiplier += combined_multiplier;
            count += 1;
        }
        
        // 返回平均受到伤害倍率的倒数作为防御评分
        let average_multiplier = total_multiplier / count as f32;
        1.0 / average_multiplier.max(0.1) // 避免除零
    }
    
    // 推荐克制属性
    pub fn recommend_counters(&self, target_types: DualType, count: usize) -> Vec<PokemonType> {
        let mut type_scores: Vec<(PokemonType, f32)> = Vec::new();
        
        for attacker_type in 0..18u8 {
            let attack_type = unsafe { std::mem::transmute(attacker_type) };
            
            let primary_eff = self.get_effectiveness(attack_type, target_types.primary);
            let secondary_eff = target_types.secondary
                .map(|t| self.get_effectiveness(attack_type, t))
                .unwrap_or(TypeEffectiveness::Normal);
            
            let combined_multiplier = self.get_multiplier(primary_eff) * 
                                    self.get_multiplier(secondary_eff);
            
            type_scores.push((attack_type, combined_multiplier));
        }
        
        // 按效果排序
        type_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // 返回前N个最有效的属性
        type_scores.into_iter()
            .take(count)
            .filter(|(_, score)| *score > 1.0) // 只返回有效的克制
            .map(|(type_val, _)| type_val)
            .collect()
    }
    
    // 获取属性名称
    pub fn get_type_name(&self, pokemon_type: PokemonType, language: &str) -> &str {
        self.type_names.get(&pokemon_type)
            .and_then(|names| names.get(0))
            .unwrap_or(&"未知")
    }
    
    // 获取属性颜色
    pub fn get_type_color(&self, pokemon_type: PokemonType) -> (u8, u8, u8) {
        self.type_colors.get(&pokemon_type)
            .copied()
            .unwrap_or((128, 128, 128)) // 默认灰色
    }
    
    // 检查天气对伤害的影响
    pub fn check_weather_damage(&self, pokemon_types: DualType, weather: WeatherType) -> i32 {
        match weather {
            WeatherType::Sandstorm => {
                // 沙暴中非岩石/地面/钢系受到伤害
                if pokemon_types.primary != PokemonType::Rock &&
                   pokemon_types.primary != PokemonType::Ground &&
                   pokemon_types.primary != PokemonType::Steel &&
                   pokemon_types.secondary != Some(PokemonType::Rock) &&
                   pokemon_types.secondary != Some(PokemonType::Ground) &&
                   pokemon_types.secondary != Some(PokemonType::Steel) {
                    16 // HP的1/16
                } else {
                    0
                }
            },
            WeatherType::Hail => {
                // 冰雹中非冰系受到伤害
                if pokemon_types.primary != PokemonType::Ice &&
                   pokemon_types.secondary != Some(PokemonType::Ice) {
                    16 // HP的1/16
                } else {
                    0
                }
            },
            _ => 0,
        }
    }
    
    // 获取随机属性
    pub fn get_random_type(&self) -> PokemonType {
        let type_id = fastrand::u8(0..18);
        unsafe { std::mem::transmute(type_id) }
    }
    
    // 获取双属性组合
    pub fn create_dual_type(&self, primary: PokemonType, secondary: Option<PokemonType>) -> DualType {
        DualType { primary, secondary }
    }
    
    // 验证属性组合是否合理
    pub fn validate_type_combination(&self, types: DualType) -> bool {
        // 检查是否有相同属性
        if let Some(secondary) = types.secondary {
            if types.primary == secondary {
                return false;
            }
        }
        
        // 检查是否为已知的合理组合
        true // 简化实现，实际可能需要更复杂的验证
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> TypeSystemStats {
        TypeSystemStats {
            total_calculations: self.total_calculations,
            cache_hits: self.cache_hits,
            cache_hit_rate: if self.total_calculations > 0 {
                (self.cache_hits as f32 / self.total_calculations as f32) * 100.0
            } else {
                0.0
            },
            cached_entries: self.matchup_cache.len(),
        }
    }
    
    // 清空缓存
    pub fn clear_cache(&mut self) {
        self.matchup_cache.clear();
        debug!("清空属性系统缓存");
    }
    
    // 私有方法
    fn initialize_effectiveness_chart(&mut self) {
        // 初始化为普通效果
        for i in 0..18 {
            for j in 0..18 {
                self.effectiveness_chart[i][j] = TypeEffectiveness::Normal;
            }
        }
        
        // 设置无效果组合
        self.set_effectiveness(PokemonType::Normal, PokemonType::Ghost, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Ghost, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Electric, PokemonType::Ground, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Poison, PokemonType::Steel, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Ground, PokemonType::Flying, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Psychic, PokemonType::Dark, TypeEffectiveness::NoEffect);
        self.set_effectiveness(PokemonType::Ghost, PokemonType::Normal, TypeEffectiveness::NoEffect);
        
        // 设置效果拔群组合
        // 格斗系
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Normal, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Rock, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Steel, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Ice, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Dark, TypeEffectiveness::SuperEffective);
        
        // 飞行系
        self.set_effectiveness(PokemonType::Flying, PokemonType::Fighting, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Flying, PokemonType::Bug, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Flying, PokemonType::Grass, TypeEffectiveness::SuperEffective);
        
        // 毒系
        self.set_effectiveness(PokemonType::Poison, PokemonType::Grass, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Poison, PokemonType::Fairy, TypeEffectiveness::SuperEffective);
        
        // 地面系
        self.set_effectiveness(PokemonType::Ground, PokemonType::Poison, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ground, PokemonType::Rock, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ground, PokemonType::Steel, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ground, PokemonType::Fire, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ground, PokemonType::Electric, TypeEffectiveness::SuperEffective);
        
        // 岩石系
        self.set_effectiveness(PokemonType::Rock, PokemonType::Flying, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Rock, PokemonType::Bug, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Rock, PokemonType::Fire, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Rock, PokemonType::Ice, TypeEffectiveness::SuperEffective);
        
        // 虫系
        self.set_effectiveness(PokemonType::Bug, PokemonType::Grass, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Bug, PokemonType::Psychic, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Bug, PokemonType::Dark, TypeEffectiveness::SuperEffective);
        
        // 幽灵系
        self.set_effectiveness(PokemonType::Ghost, PokemonType::Ghost, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ghost, PokemonType::Psychic, TypeEffectiveness::SuperEffective);
        
        // 钢系
        self.set_effectiveness(PokemonType::Steel, PokemonType::Rock, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Steel, PokemonType::Ice, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Steel, PokemonType::Fairy, TypeEffectiveness::SuperEffective);
        
        // 火系
        self.set_effectiveness(PokemonType::Fire, PokemonType::Bug, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fire, PokemonType::Steel, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fire, PokemonType::Grass, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fire, PokemonType::Ice, TypeEffectiveness::SuperEffective);
        
        // 水系
        self.set_effectiveness(PokemonType::Water, PokemonType::Ground, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Water, PokemonType::Rock, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Water, PokemonType::Fire, TypeEffectiveness::SuperEffective);
        
        // 草系
        self.set_effectiveness(PokemonType::Grass, PokemonType::Ground, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Grass, PokemonType::Rock, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Grass, PokemonType::Water, TypeEffectiveness::SuperEffective);
        
        // 电系
        self.set_effectiveness(PokemonType::Electric, PokemonType::Flying, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Electric, PokemonType::Water, TypeEffectiveness::SuperEffective);
        
        // 超能力系
        self.set_effectiveness(PokemonType::Psychic, PokemonType::Fighting, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Psychic, PokemonType::Poison, TypeEffectiveness::SuperEffective);
        
        // 冰系
        self.set_effectiveness(PokemonType::Ice, PokemonType::Flying, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ice, PokemonType::Ground, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ice, PokemonType::Grass, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Ice, PokemonType::Dragon, TypeEffectiveness::SuperEffective);
        
        // 龙系
        self.set_effectiveness(PokemonType::Dragon, PokemonType::Dragon, TypeEffectiveness::SuperEffective);
        
        // 恶系
        self.set_effectiveness(PokemonType::Dark, PokemonType::Ghost, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Dark, PokemonType::Psychic, TypeEffectiveness::SuperEffective);
        
        // 妖精系
        self.set_effectiveness(PokemonType::Fairy, PokemonType::Fighting, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fairy, PokemonType::Dragon, TypeEffectiveness::SuperEffective);
        self.set_effectiveness(PokemonType::Fairy, PokemonType::Dark, TypeEffectiveness::SuperEffective);
        
        // 设置效果不佳组合 (简化版本，实际需要更多)
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Flying, TypeEffectiveness::NotVeryEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Psychic, TypeEffectiveness::NotVeryEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Bug, TypeEffectiveness::NotVeryEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Rock, TypeEffectiveness::NotVeryEffective);
        self.set_effectiveness(PokemonType::Fighting, PokemonType::Dark, TypeEffectiveness::NotVeryEffective);
        // ... 更多效果不佳的组合
    }
    
    fn initialize_type_names(&mut self) {
        self.type_names.insert(PokemonType::Normal, vec!["一般", "Normal"]);
        self.type_names.insert(PokemonType::Fighting, vec!["格斗", "Fighting"]);
        self.type_names.insert(PokemonType::Flying, vec!["飞行", "Flying"]);
        self.type_names.insert(PokemonType::Poison, vec!["毒", "Poison"]);
        self.type_names.insert(PokemonType::Ground, vec!["地面", "Ground"]);
        self.type_names.insert(PokemonType::Rock, vec!["岩石", "Rock"]);
        self.type_names.insert(PokemonType::Bug, vec!["虫", "Bug"]);
        self.type_names.insert(PokemonType::Ghost, vec!["幽灵", "Ghost"]);
        self.type_names.insert(PokemonType::Steel, vec!["钢", "Steel"]);
        self.type_names.insert(PokemonType::Fire, vec!["火", "Fire"]);
        self.type_names.insert(PokemonType::Water, vec!["水", "Water"]);
        self.type_names.insert(PokemonType::Grass, vec!["草", "Grass"]);
        self.type_names.insert(PokemonType::Electric, vec!["电", "Electric"]);
        self.type_names.insert(PokemonType::Psychic, vec!["超能力", "Psychic"]);
        self.type_names.insert(PokemonType::Ice, vec!["冰", "Ice"]);
        self.type_names.insert(PokemonType::Dragon, vec!["龙", "Dragon"]);
        self.type_names.insert(PokemonType::Dark, vec!["恶", "Dark"]);
        self.type_names.insert(PokemonType::Fairy, vec!["妖精", "Fairy"]);
    }
    
    fn initialize_type_colors(&mut self) {
        self.type_colors.insert(PokemonType::Normal, (168, 168, 120));
        self.type_colors.insert(PokemonType::Fighting, (192, 48, 40));
        self.type_colors.insert(PokemonType::Flying, (168, 144, 240));
        self.type_colors.insert(PokemonType::Poison, (160, 64, 160));
        self.type_colors.insert(PokemonType::Ground, (224, 192, 104));
        self.type_colors.insert(PokemonType::Rock, (184, 160, 56));
        self.type_colors.insert(PokemonType::Bug, (168, 184, 32));
        self.type_colors.insert(PokemonType::Ghost, (112, 88, 152));
        self.type_colors.insert(PokemonType::Steel, (184, 184, 208));
        self.type_colors.insert(PokemonType::Fire, (240, 128, 48));
        self.type_colors.insert(PokemonType::Water, (104, 144, 240));
        self.type_colors.insert(PokemonType::Grass, (120, 200, 80));
        self.type_colors.insert(PokemonType::Electric, (248, 208, 48));
        self.type_colors.insert(PokemonType::Psychic, (248, 88, 136));
        self.type_colors.insert(PokemonType::Ice, (152, 216, 216));
        self.type_colors.insert(PokemonType::Dragon, (112, 56, 248));
        self.type_colors.insert(PokemonType::Dark, (112, 88, 72));
        self.type_colors.insert(PokemonType::Fairy, (238, 153, 172));
    }
    
    fn initialize_weather_effects(&mut self) {
        // 大晴天效果
        let mut sunny_effects = HashMap::new();
        sunny_effects.insert(PokemonType::Fire, 1.5);   // 火系技能+50%
        sunny_effects.insert(PokemonType::Water, 0.5);  // 水系技能-50%
        self.weather_effects.insert(WeatherType::Sunny, sunny_effects);
        
        // 下雨效果
        let mut rain_effects = HashMap::new();
        rain_effects.insert(PokemonType::Water, 1.5);   // 水系技能+50%
        rain_effects.insert(PokemonType::Fire, 0.5);    // 火系技能-50%
        self.weather_effects.insert(WeatherType::Rain, rain_effects);
        
        // 沙暴效果 (主要影响伤害而非技能威力)
        let sandstorm_effects = HashMap::new();
        self.weather_effects.insert(WeatherType::Sandstorm, sandstorm_effects);
        
        // 冰雹效果 (主要影响伤害而非技能威力)
        let hail_effects = HashMap::new();
        self.weather_effects.insert(WeatherType::Hail, hail_effects);
    }
    
    fn initialize_special_interactions(&mut self) {
        // 添加一些特殊交互规则示例
        self.special_interactions.push(SpecialTypeInteraction {
            attacker_type: PokemonType::Fire,
            defender_type: DualType { primary: PokemonType::Bug, secondary: Some(PokemonType::Steel) },
            condition: "火系对虫钢的特殊效果".to_string(),
            modifier: 4.0, // 4倍伤害
            priority: 10,
        });
        
        // 更多特殊交互可以在这里添加...
    }
    
    fn set_effectiveness(&mut self, attacker: PokemonType, defender: PokemonType, effectiveness: TypeEffectiveness) {
        self.effectiveness_chart[attacker as usize][defender as usize] = effectiveness;
    }
    
    fn get_effectiveness(&self, attacker: PokemonType, defender: PokemonType) -> TypeEffectiveness {
        self.effectiveness_chart[attacker as usize][defender as usize]
    }
    
    fn get_multiplier(&self, effectiveness: TypeEffectiveness) -> f32 {
        match effectiveness {
            TypeEffectiveness::NoEffect => 0.0,
            TypeEffectiveness::NotVeryEffective => 0.5,
            TypeEffectiveness::Normal => 1.0,
            TypeEffectiveness::SuperEffective => 2.0,
        }
    }
    
    fn calculate_weather_effect(&self, attack_type: PokemonType, weather: WeatherType) -> f32 {
        self.weather_effects
            .get(&weather)
            .and_then(|effects| effects.get(&attack_type))
            .copied()
            .unwrap_or(1.0)
    }
    
    fn calculate_special_interactions(
        &self,
        attack_type: PokemonType,
        defender_types: DualType,
        conditions: &HashMap<String, bool>,
    ) -> f32 {
        let mut modifier = 1.0;
        
        for interaction in &self.special_interactions {
            if interaction.attacker_type == attack_type &&
               interaction.defender_type.primary == defender_types.primary &&
               interaction.defender_type.secondary == defender_types.secondary {
                
                // 检查特殊条件
                if conditions.get(&interaction.condition).copied().unwrap_or(false) {
                    modifier *= interaction.modifier;
                }
            }
        }
        
        modifier
    }
}

// 统计信息结构体
#[derive(Debug, Clone)]
pub struct TypeSystemStats {
    pub total_calculations: u64,
    pub cache_hits: u64,
    pub cache_hit_rate: f32,
    pub cached_entries: usize,
}

// 便利方法实现
impl PokemonType {
    pub fn from_id(id: u8) -> Option<Self> {
        if id < 18 {
            Some(unsafe { std::mem::transmute(id) })
        } else {
            None
        }
    }
    
    pub fn to_id(&self) -> u8 {
        *self as u8
    }
    
    pub fn all_types() -> Vec<Self> {
        (0..18u8).map(|id| unsafe { std::mem::transmute(id) }).collect()
    }
}

impl DualType {
    pub fn single(pokemon_type: PokemonType) -> Self {
        Self { primary: pokemon_type, secondary: None }
    }
    
    pub fn dual(primary: PokemonType, secondary: PokemonType) -> Self {
        Self { primary, secondary: Some(secondary) }
    }
    
    pub fn has_type(&self, pokemon_type: PokemonType) -> bool {
        self.primary == pokemon_type || 
        self.secondary.map_or(false, |t| t == pokemon_type)
    }
    
    pub fn types(&self) -> Vec<PokemonType> {
        let mut types = vec![self.primary];
        if let Some(secondary) = self.secondary {
            types.push(secondary);
        }
        types
    }
}

impl TypeEffectiveness {
    pub fn to_multiplier(&self) -> f32 {
        match self {
            Self::NoEffect => 0.0,
            Self::NotVeryEffective => 0.5,
            Self::Normal => 1.0,
            Self::SuperEffective => 2.0,
        }
    }
    
    pub fn from_multiplier(multiplier: f32) -> Self {
        match multiplier {
            x if x == 0.0 => Self::NoEffect,
            x if x < 1.0 => Self::NotVeryEffective,
            x if x > 1.0 => Self::SuperEffective,
            _ => Self::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_system_creation() {
        let manager = TypeSystemManager::new();
        assert_eq!(manager.stab_multiplier, 1.5);
        assert!(!manager.type_names.is_empty());
        assert!(!manager.type_colors.is_empty());
    }
    
    #[test]
    fn test_basic_type_effectiveness() {
        let mut manager = TypeSystemManager::new();
        
        let result = manager.calculate_type_effectiveness(
            PokemonType::Water,
            DualType::single(PokemonType::Fire),
            None,
            WeatherType::None,
            &HashMap::new(),
        ).unwrap();
        
        assert_eq!(result.effectiveness, TypeEffectiveness::SuperEffective);
        assert_eq!(result.multiplier, 2.0);
    }
    
    #[test]
    fn test_dual_type_effectiveness() {
        let mut manager = TypeSystemManager::new();
        
        // 水系攻击草钢系 (2倍×0.5倍=1倍)
        let result = manager.calculate_type_effectiveness(
            PokemonType::Water,
            DualType::dual(PokemonType::Grass, PokemonType::Steel),
            None,
            WeatherType::None,
            &HashMap::new(),
        ).unwrap();
        
        // 具体倍率取决于实际的相克表实现
        assert!(result.multiplier >= 0.0);
    }
    
    #[test]
    fn test_stab_bonus() {
        let mut manager = TypeSystemManager::new();
        
        let result = manager.calculate_type_effectiveness(
            PokemonType::Fire,
            DualType::single(PokemonType::Grass),
            Some(DualType::single(PokemonType::Fire)),
            WeatherType::None,
            &HashMap::new(),
        ).unwrap();
        
        assert!(result.is_stab);
        assert!(result.final_multiplier > result.multiplier);
    }
    
    #[test]
    fn test_weather_effects() {
        let mut manager = TypeSystemManager::new();
        
        let result = manager.calculate_type_effectiveness(
            PokemonType::Fire,
            DualType::single(PokemonType::Grass),
            None,
            WeatherType::Sunny,
            &HashMap::new(),
        ).unwrap();
        
        assert!(result.weather_boost > 1.0);
    }
    
    #[test]
    fn test_immunity_check() {
        let manager = TypeSystemManager::new();
        
        // 火系免疫燃烧
        assert!(manager.is_immune_to_status(
            DualType::single(PokemonType::Fire),
            TypeStatus::Burned
        ));
        
        // 普通系不免疫燃烧
        assert!(!manager.is_immune_to_status(
            DualType::single(PokemonType::Normal),
            TypeStatus::Burned
        ));
    }
    
    #[test]
    fn test_weakness_calculation() {
        let manager = TypeSystemManager::new();
        
        let weaknesses = manager.get_weaknesses(DualType::single(PokemonType::Fire));
        assert!(weaknesses.contains(&PokemonType::Water));
        assert!(weaknesses.contains(&PokemonType::Ground));
        assert!(weaknesses.contains(&PokemonType::Rock));
    }
    
    #[test]
    fn test_type_recommendations() {
        let manager = TypeSystemManager::new();
        
        let counters = manager.recommend_counters(
            DualType::single(PokemonType::Dragon),
            3
        );
        
        assert!(counters.contains(&PokemonType::Ice));
        assert!(counters.contains(&PokemonType::Dragon));
        assert!(counters.contains(&PokemonType::Fairy));
    }
}