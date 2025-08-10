// Pokemon统计系统
// 开发心理：能力值是Pokemon核心数据，需要精确计算、合理平衡、性能优化
// 设计原则：个体值系统、努力值培养、性格修正、等级成长曲线

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;

// 基础能力值类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    HP,         // 体力
    Attack,     // 攻击
    Defense,    // 防御
    SpAttack,   // 特攻
    SpDefense,  // 特防
    Speed,      // 速度
}

// 性格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nature {
    // 中性性格
    Hardy,      // 勤奋
    Docile,     // 温顺
    Serious,    // 认真
    Bashful,    // 害羞
    Quirky,     // 浮躁
    
    // 攻击性格
    Lonely,     // 怕寂寞 (+攻击 -防御)
    Brave,      // 勇敢 (+攻击 -速度)
    Adamant,    // 固执 (+攻击 -特攻)
    Naughty,    // 顽皮 (+攻击 -特防)
    
    // 防御性格
    Bold,       // 大胆 (+防御 -攻击)
    Relaxed,    // 悠闲 (+防御 -速度)
    Impish,     // 淘气 (+防御 -特攻)
    Lax,        // 乐天 (+防御 -特防)
    
    // 特攻性格
    Modest,     // 内敛 (+特攻 -攻击)
    Mild,       // 慢吞吞 (+特攻 -防御)
    Quiet,      // 冷静 (+特攻 -速度)
    Rash,       // 马虎 (+特攻 -特防)
    
    // 特防性格
    Calm,       // 温和 (+特防 -攻击)
    Gentle,     // 温厚 (+特防 -防御)
    Sassy,      // 自大 (+特防 -速度)
    Careful,    // 慎重 (+特防 -特攻)
    
    // 速度性格
    Timid,      // 胆小 (+速度 -攻击)
    Hasty,      // 急躁 (+速度 -防御)
    Jolly,      // 爽朗 (+速度 -特攻)
    Naive,      // 天真 (+速度 -特防)
}

// 基础能力值
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BaseStats {
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
}

// 个体值 (IV)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndividualValues {
    pub hp: u8,         // 0-31
    pub attack: u8,     // 0-31
    pub defense: u8,    // 0-31
    pub sp_attack: u8,  // 0-31
    pub sp_defense: u8, // 0-31
    pub speed: u8,      // 0-31
}

// 努力值 (EV)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EffortValues {
    pub hp: u8,         // 0-255
    pub attack: u8,     // 0-255
    pub defense: u8,    // 0-255
    pub sp_attack: u8,  // 0-255
    pub sp_defense: u8, // 0-255
    pub speed: u8,      // 0-255
}

// 能力值修正阶段 (-6 到 +6)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatStages {
    pub attack: i8,     // -6 到 +6
    pub defense: i8,    // -6 到 +6
    pub sp_attack: i8,  // -6 到 +6
    pub sp_defense: i8, // -6 到 +6
    pub speed: i8,      // -6 到 +6
    pub accuracy: i8,   // -6 到 +6
    pub evasion: i8,    // -6 到 +6
}

// 实际能力值
#[derive(Debug, Clone, Copy)]
pub struct ActualStats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub sp_attack: u32,
    pub sp_defense: u32,
    pub speed: u32,
    pub accuracy: u32,   // 命中率 (基础100)
    pub evasion: u32,    // 回避率 (基础100)
}

// Pokemon统计数据
#[derive(Debug, Clone)]
pub struct PokemonStats {
    pub species_id: u32,
    pub level: u8,                  // 1-100
    pub nature: Nature,
    pub base_stats: BaseStats,
    pub individual_values: IndividualValues,
    pub effort_values: EffortValues,
    pub stat_stages: StatStages,
    pub actual_stats: ActualStats,
    
    // 隐藏能力
    pub hidden_power_type: Option<crate::pokemon::types::TypeId>,
    pub hidden_power_power: u8,     // 威力30-70
    
    // 能力值历史
    pub stat_history: Vec<StatChange>,
    
    // 特殊状态
    pub stat_modifiers: HashMap<String, f32>, // 临时修正值
    pub permanent_modifiers: HashMap<String, i32>, // 永久修正值
}

// 能力值变化记录
#[derive(Debug, Clone)]
pub struct StatChange {
    pub stat_type: StatType,
    pub old_value: u32,
    pub new_value: u32,
    pub change_type: StatChangeType,
    pub timestamp: std::time::Instant,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatChangeType {
    LevelUp,        // 升级
    EvTraining,     // 努力值训练
    Nature,         // 性格变化
    Temporary,      // 临时修正
    Permanent,      // 永久修正
    Equipment,      // 装备
}

// 统计系统管理器
pub struct StatsManager {
    // 性格修正表
    nature_modifiers: HashMap<Nature, (Option<StatType>, Option<StatType>)>,
    
    // 能力值修正倍率表
    stat_stage_multipliers: [f32; 13], // 索引6为基础值1.0
    
    // 配置
    max_ev_total: u16,      // 总努力值上限 (510)
    max_ev_per_stat: u8,    // 单项努力值上限 (255)
    min_level: u8,          // 最低等级 (1)
    max_level: u8,          // 最高等级 (100)
    
    // 统计信息
    total_calculations: u64,
    cache_hits: u64,
    
    // 计算缓存
    stat_cache: HashMap<String, ActualStats>,
    cache_enabled: bool,
}

impl StatsManager {
    pub fn new() -> Self {
        let mut manager = Self {
            nature_modifiers: HashMap::new(),
            stat_stage_multipliers: [
                0.25, 0.28, 0.33, 0.4, 0.5, 0.66, // -6 到 -1
                1.0,                                // 0 (基础)
                1.5, 2.0, 2.5, 3.0, 3.5, 4.0      // +1 到 +6
            ],
            max_ev_total: 510,
            max_ev_per_stat: 255,
            min_level: 1,
            max_level: 100,
            total_calculations: 0,
            cache_hits: 0,
            stat_cache: HashMap::new(),
            cache_enabled: true,
        };
        
        manager.initialize_nature_modifiers();
        manager
    }
    
    // 计算实际能力值
    pub fn calculate_stats(&mut self, pokemon_stats: &mut PokemonStats) -> Result<(), GameError> {
        // 检查缓存
        let cache_key = self.generate_cache_key(pokemon_stats);
        if self.cache_enabled {
            if let Some(cached_stats) = self.stat_cache.get(&cache_key) {
                pokemon_stats.actual_stats = *cached_stats;
                self.cache_hits += 1;
                return Ok(());
            }
        }
        
        self.total_calculations += 1;
        
        // 计算基础能力值
        let level = pokemon_stats.level as f32;
        
        // HP计算公式: ((基础值 + 个体值) * 2 + 努力值/4) * 等级/100 + 等级 + 10
        let hp = if pokemon_stats.base_stats.hp > 1 {
            let base_hp = pokemon_stats.base_stats.hp as f32;
            let iv_hp = pokemon_stats.individual_values.hp as f32;
            let ev_hp = pokemon_stats.effort_values.hp as f32;
            
            (((base_hp + iv_hp) * 2.0 + ev_hp / 4.0) * level / 100.0 + level + 10.0).floor() as u32
        } else {
            1 // 特殊情况，如化石盔的HP为1
        };
        
        // 其他能力值计算公式: (((基础值 + 个体值) * 2 + 努力值/4) * 等级/100 + 5) * 性格修正
        let nature_mod = self.get_nature_modifier(pokemon_stats.nature);
        
        let attack = self.calculate_non_hp_stat(
            pokemon_stats.base_stats.attack,
            pokemon_stats.individual_values.attack,
            pokemon_stats.effort_values.attack,
            level,
            nature_mod.0.map_or(1.0, |boost| if boost == StatType::Attack { 1.1 } else { 1.0 }) *
            nature_mod.1.map_or(1.0, |nerf| if nerf == StatType::Attack { 0.9 } else { 1.0 })
        );
        
        let defense = self.calculate_non_hp_stat(
            pokemon_stats.base_stats.defense,
            pokemon_stats.individual_values.defense,
            pokemon_stats.effort_values.defense,
            level,
            nature_mod.0.map_or(1.0, |boost| if boost == StatType::Defense { 1.1 } else { 1.0 }) *
            nature_mod.1.map_or(1.0, |nerf| if nerf == StatType::Defense { 0.9 } else { 1.0 })
        );
        
        let sp_attack = self.calculate_non_hp_stat(
            pokemon_stats.base_stats.sp_attack,
            pokemon_stats.individual_values.sp_attack,
            pokemon_stats.effort_values.sp_attack,
            level,
            nature_mod.0.map_or(1.0, |boost| if boost == StatType::SpAttack { 1.1 } else { 1.0 }) *
            nature_mod.1.map_or(1.0, |nerf| if nerf == StatType::SpAttack { 0.9 } else { 1.0 })
        );
        
        let sp_defense = self.calculate_non_hp_stat(
            pokemon_stats.base_stats.sp_defense,
            pokemon_stats.individual_values.sp_defense,
            pokemon_stats.effort_values.sp_defense,
            level,
            nature_mod.0.map_or(1.0, |boost| if boost == StatType::SpDefense { 1.1 } else { 1.0 }) *
            nature_mod.1.map_or(1.0, |nerf| if nerf == StatType::SpDefense { 0.9 } else { 1.0 })
        );
        
        let speed = self.calculate_non_hp_stat(
            pokemon_stats.base_stats.speed,
            pokemon_stats.individual_values.speed,
            pokemon_stats.effort_values.speed,
            level,
            nature_mod.0.map_or(1.0, |boost| if boost == StatType::Speed { 1.1 } else { 1.0 }) *
            nature_mod.1.map_or(1.0, |nerf| if nerf == StatType::Speed { 0.9 } else { 1.0 })
        );
        
        let actual_stats = ActualStats {
            hp,
            attack,
            defense,
            sp_attack,
            sp_defense,
            speed,
            accuracy: 100,  // 基础命中率
            evasion: 100,   // 基础回避率
        };
        
        // 应用能力值修正阶段
        pokemon_stats.actual_stats = self.apply_stat_stages(&actual_stats, &pokemon_stats.stat_stages);
        
        // 应用临时修正
        self.apply_modifiers(&mut pokemon_stats.actual_stats, &pokemon_stats.stat_modifiers);
        
        // 缓存结果
        if self.cache_enabled {
            self.stat_cache.insert(cache_key, pokemon_stats.actual_stats);
        }
        
        debug!("计算Pokemon能力值完成: HP={} ATK={} DEF={} SPA={} SPD={} SPE={}",
            pokemon_stats.actual_stats.hp,
            pokemon_stats.actual_stats.attack,
            pokemon_stats.actual_stats.defense,
            pokemon_stats.actual_stats.sp_attack,
            pokemon_stats.actual_stats.sp_defense,
            pokemon_stats.actual_stats.speed
        );
        
        Ok(())
    }
    
    // 应用能力值变化
    pub fn apply_stat_change(
        &mut self,
        pokemon_stats: &mut PokemonStats,
        stat_type: StatType,
        stage_change: i8,
    ) -> Result<bool, GameError> {
        let old_stage = match stat_type {
            StatType::Attack => pokemon_stats.stat_stages.attack,
            StatType::Defense => pokemon_stats.stat_stages.defense,
            StatType::SpAttack => pokemon_stats.stat_stages.sp_attack,
            StatType::SpDefense => pokemon_stats.stat_stages.sp_defense,
            StatType::Speed => pokemon_stats.stat_stages.speed,
            StatType::HP => return Err(GameError::Stats("HP能力值不能被修正".to_string())),
        };
        
        let new_stage = (old_stage + stage_change).clamp(-6, 6);
        let actual_change = new_stage - old_stage;
        
        if actual_change == 0 {
            return Ok(false); // 没有变化
        }
        
        // 应用变化
        match stat_type {
            StatType::Attack => pokemon_stats.stat_stages.attack = new_stage,
            StatType::Defense => pokemon_stats.stat_stages.defense = new_stage,
            StatType::SpAttack => pokemon_stats.stat_stages.sp_attack = new_stage,
            StatType::SpDefense => pokemon_stats.stat_stages.sp_defense = new_stage,
            StatType::Speed => pokemon_stats.stat_stages.speed = new_stage,
            StatType::HP => unreachable!(),
        }
        
        // 记录变化
        let old_value = self.get_stat_value(&pokemon_stats.actual_stats, stat_type);
        self.calculate_stats(pokemon_stats)?;
        let new_value = self.get_stat_value(&pokemon_stats.actual_stats, stat_type);
        
        pokemon_stats.stat_history.push(StatChange {
            stat_type,
            old_value,
            new_value,
            change_type: StatChangeType::Temporary,
            timestamp: std::time::Instant::now(),
            source: "battle_effect".to_string(),
        });
        
        debug!("应用能力值修正: {:?} {} -> {} (阶段: {})",
            stat_type, old_value, new_value, new_stage);
        
        Ok(true)
    }
    
    // 训练努力值
    pub fn train_effort_value(
        &mut self,
        pokemon_stats: &mut PokemonStats,
        stat_type: StatType,
        amount: u8,
    ) -> Result<u8, GameError> {
        // 检查总努力值限制
        let current_total = self.calculate_total_evs(&pokemon_stats.effort_values);
        let remaining_total = self.max_ev_total.saturating_sub(current_total);
        
        if remaining_total == 0 {
            return Ok(0); // 已达到上限
        }
        
        // 检查单项努力值限制
        let current_stat_ev = self.get_effort_value(&pokemon_stats.effort_values, stat_type);
        let remaining_stat = self.max_ev_per_stat.saturating_sub(current_stat_ev);
        
        if remaining_stat == 0 {
            return Ok(0); // 该项已满
        }
        
        // 计算实际可以增加的量
        let actual_amount = amount.min(remaining_total as u8).min(remaining_stat);
        
        if actual_amount == 0 {
            return Ok(0);
        }
        
        // 应用努力值增加
        let old_value = self.get_stat_value(&pokemon_stats.actual_stats, stat_type);
        
        match stat_type {
            StatType::HP => pokemon_stats.effort_values.hp += actual_amount,
            StatType::Attack => pokemon_stats.effort_values.attack += actual_amount,
            StatType::Defense => pokemon_stats.effort_values.defense += actual_amount,
            StatType::SpAttack => pokemon_stats.effort_values.sp_attack += actual_amount,
            StatType::SpDefense => pokemon_stats.effort_values.sp_defense += actual_amount,
            StatType::Speed => pokemon_stats.effort_values.speed += actual_amount,
        }
        
        // 重新计算能力值
        self.calculate_stats(pokemon_stats)?;
        let new_value = self.get_stat_value(&pokemon_stats.actual_stats, stat_type);
        
        // 记录训练历史
        pokemon_stats.stat_history.push(StatChange {
            stat_type,
            old_value,
            new_value,
            change_type: StatChangeType::EvTraining,
            timestamp: std::time::Instant::now(),
            source: "training".to_string(),
        });
        
        debug!("努力值训练: {:?} +{} EV (总计: {})",
            stat_type, actual_amount, 
            self.calculate_total_evs(&pokemon_stats.effort_values));
        
        Ok(actual_amount)
    }
    
    // 等级提升
    pub fn level_up(&mut self, pokemon_stats: &mut PokemonStats) -> Result<Vec<StatChange>, GameError> {
        if pokemon_stats.level >= self.max_level {
            return Err(GameError::Stats("已达到最大等级".to_string()));
        }
        
        // 记录升级前的能力值
        let old_stats = pokemon_stats.actual_stats;
        
        // 提升等级
        pokemon_stats.level += 1;
        
        // 重新计算能力值
        self.calculate_stats(pokemon_stats)?;
        
        // 记录所有能力值变化
        let mut changes = Vec::new();
        
        if old_stats.hp != pokemon_stats.actual_stats.hp {
            changes.push(StatChange {
                stat_type: StatType::HP,
                old_value: old_stats.hp,
                new_value: pokemon_stats.actual_stats.hp,
                change_type: StatChangeType::LevelUp,
                timestamp: std::time::Instant::now(),
                source: format!("level_up_{}", pokemon_stats.level),
            });
        }
        
        if old_stats.attack != pokemon_stats.actual_stats.attack {
            changes.push(StatChange {
                stat_type: StatType::Attack,
                old_value: old_stats.attack,
                new_value: pokemon_stats.actual_stats.attack,
                change_type: StatChangeType::LevelUp,
                timestamp: std::time::Instant::now(),
                source: format!("level_up_{}", pokemon_stats.level),
            });
        }
        
        // 对其他能力值做同样处理...
        
        // 将变化记录到历史中
        for change in &changes {
            pokemon_stats.stat_history.push(change.clone());
        }
        
        debug!("Pokemon升级到 {} 级，能力值发生 {} 项变化",
            pokemon_stats.level, changes.len());
        
        Ok(changes)
    }
    
    // 重置能力值修正阶段
    pub fn reset_stat_stages(&mut self, pokemon_stats: &mut PokemonStats) -> Result<(), GameError> {
        let had_changes = pokemon_stats.stat_stages.attack != 0 ||
            pokemon_stats.stat_stages.defense != 0 ||
            pokemon_stats.stat_stages.sp_attack != 0 ||
            pokemon_stats.stat_stages.sp_defense != 0 ||
            pokemon_stats.stat_stages.speed != 0 ||
            pokemon_stats.stat_stages.accuracy != 0 ||
            pokemon_stats.stat_stages.evasion != 0;
        
        if had_changes {
            pokemon_stats.stat_stages = StatStages {
                attack: 0,
                defense: 0,
                sp_attack: 0,
                sp_defense: 0,
                speed: 0,
                accuracy: 0,
                evasion: 0,
            };
            
            self.calculate_stats(pokemon_stats)?;
            debug!("重置所有能力值修正阶段");
        }
        
        Ok(())
    }
    
    // 计算隐藏力量
    pub fn calculate_hidden_power(&self, ivs: &IndividualValues) -> (Option<crate::pokemon::types::TypeId>, u8) {
        // 隐藏力量属性计算
        let type_value = (ivs.hp % 2) +
            (ivs.attack % 2) * 2 +
            (ivs.defense % 2) * 4 +
            (ivs.speed % 2) * 8 +
            (ivs.sp_attack % 2) * 16 +
            (ivs.sp_defense % 2) * 32;
        
        let type_id = (type_value * 15 / 63) as u8; // 简化的类型映射
        
        // 隐藏力量威力计算
        let power_value = ((ivs.hp % 4) / 2) +
            ((ivs.attack % 4) / 2) * 2 +
            ((ivs.defense % 4) / 2) * 4 +
            ((ivs.speed % 4) / 2) * 8 +
            ((ivs.sp_attack % 4) / 2) * 16 +
            ((ivs.sp_defense % 4) / 2) * 32;
        
        let power = ((power_value * 40) / 63) as u8 + 30;
        
        (Some(type_id), power)
    }
    
    // 生成随机个体值
    pub fn generate_random_ivs(&self) -> IndividualValues {
        IndividualValues {
            hp: fastrand::u8(0..32),
            attack: fastrand::u8(0..32),
            defense: fastrand::u8(0..32),
            sp_attack: fastrand::u8(0..32),
            sp_defense: fastrand::u8(0..32),
            speed: fastrand::u8(0..32),
        }
    }
    
    // 生成完美个体值
    pub fn generate_perfect_ivs(&self) -> IndividualValues {
        IndividualValues {
            hp: 31,
            attack: 31,
            defense: 31,
            sp_attack: 31,
            sp_defense: 31,
            speed: 31,
        }
    }
    
    // 计算个体值总和
    pub fn calculate_iv_total(&self, ivs: &IndividualValues) -> u16 {
        ivs.hp as u16 + ivs.attack as u16 + ivs.defense as u16 +
        ivs.sp_attack as u16 + ivs.sp_defense as u16 + ivs.speed as u16
    }
    
    // 评估个体值品质
    pub fn evaluate_iv_quality(&self, ivs: &IndividualValues) -> IvQuality {
        let total = self.calculate_iv_total(ivs);
        match total {
            0..=90 => IvQuality::Poor,
            91..=120 => IvQuality::Fair,
            121..=150 => IvQuality::Good,
            151..=170 => IvQuality::Excellent,
            171..=185 => IvQuality::Outstanding,
            _ => IvQuality::Perfect,
        }
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> StatsManagerStats {
        StatsManagerStats {
            total_calculations: self.total_calculations,
            cache_hits: self.cache_hits,
            cache_hit_rate: if self.total_calculations > 0 {
                (self.cache_hits as f32 / self.total_calculations as f32) * 100.0
            } else {
                0.0
            },
            cached_entries: self.stat_cache.len(),
        }
    }
    
    // 清空缓存
    pub fn clear_cache(&mut self) {
        self.stat_cache.clear();
        debug!("清空能力值计算缓存");
    }
    
    // 私有方法
    fn initialize_nature_modifiers(&mut self) {
        // 中性性格
        self.nature_modifiers.insert(Nature::Hardy, (None, None));
        self.nature_modifiers.insert(Nature::Docile, (None, None));
        self.nature_modifiers.insert(Nature::Serious, (None, None));
        self.nature_modifiers.insert(Nature::Bashful, (None, None));
        self.nature_modifiers.insert(Nature::Quirky, (None, None));
        
        // 攻击性格
        self.nature_modifiers.insert(Nature::Lonely, (Some(StatType::Attack), Some(StatType::Defense)));
        self.nature_modifiers.insert(Nature::Brave, (Some(StatType::Attack), Some(StatType::Speed)));
        self.nature_modifiers.insert(Nature::Adamant, (Some(StatType::Attack), Some(StatType::SpAttack)));
        self.nature_modifiers.insert(Nature::Naughty, (Some(StatType::Attack), Some(StatType::SpDefense)));
        
        // 防御性格
        self.nature_modifiers.insert(Nature::Bold, (Some(StatType::Defense), Some(StatType::Attack)));
        self.nature_modifiers.insert(Nature::Relaxed, (Some(StatType::Defense), Some(StatType::Speed)));
        self.nature_modifiers.insert(Nature::Impish, (Some(StatType::Defense), Some(StatType::SpAttack)));
        self.nature_modifiers.insert(Nature::Lax, (Some(StatType::Defense), Some(StatType::SpDefense)));
        
        // 特攻性格
        self.nature_modifiers.insert(Nature::Modest, (Some(StatType::SpAttack), Some(StatType::Attack)));
        self.nature_modifiers.insert(Nature::Mild, (Some(StatType::SpAttack), Some(StatType::Defense)));
        self.nature_modifiers.insert(Nature::Quiet, (Some(StatType::SpAttack), Some(StatType::Speed)));
        self.nature_modifiers.insert(Nature::Rash, (Some(StatType::SpAttack), Some(StatType::SpDefense)));
        
        // 特防性格
        self.nature_modifiers.insert(Nature::Calm, (Some(StatType::SpDefense), Some(StatType::Attack)));
        self.nature_modifiers.insert(Nature::Gentle, (Some(StatType::SpDefense), Some(StatType::Defense)));
        self.nature_modifiers.insert(Nature::Sassy, (Some(StatType::SpDefense), Some(StatType::Speed)));
        self.nature_modifiers.insert(Nature::Careful, (Some(StatType::SpDefense), Some(StatType::SpAttack)));
        
        // 速度性格
        self.nature_modifiers.insert(Nature::Timid, (Some(StatType::Speed), Some(StatType::Attack)));
        self.nature_modifiers.insert(Nature::Hasty, (Some(StatType::Speed), Some(StatType::Defense)));
        self.nature_modifiers.insert(Nature::Jolly, (Some(StatType::Speed), Some(StatType::SpAttack)));
        self.nature_modifiers.insert(Nature::Naive, (Some(StatType::Speed), Some(StatType::SpDefense)));
    }
    
    fn calculate_non_hp_stat(&self, base: u16, iv: u8, ev: u8, level: f32, nature_mod: f32) -> u32 {
        let base_stat = ((((base as f32 + iv as f32) * 2.0 + ev as f32 / 4.0) * level / 100.0 + 5.0) * nature_mod).floor() as u32;
        base_stat.max(1) // 最低为1
    }
    
    fn get_nature_modifier(&self, nature: Nature) -> (Option<StatType>, Option<StatType>) {
        self.nature_modifiers.get(&nature).copied().unwrap_or((None, None))
    }
    
    fn apply_stat_stages(&self, base_stats: &ActualStats, stages: &StatStages) -> ActualStats {
        ActualStats {
            hp: base_stats.hp, // HP不受修正影响
            attack: (base_stats.attack as f32 * self.get_stage_multiplier(stages.attack)) as u32,
            defense: (base_stats.defense as f32 * self.get_stage_multiplier(stages.defense)) as u32,
            sp_attack: (base_stats.sp_attack as f32 * self.get_stage_multiplier(stages.sp_attack)) as u32,
            sp_defense: (base_stats.sp_defense as f32 * self.get_stage_multiplier(stages.sp_defense)) as u32,
            speed: (base_stats.speed as f32 * self.get_stage_multiplier(stages.speed)) as u32,
            accuracy: (100.0 * self.get_stage_multiplier(stages.accuracy)) as u32,
            evasion: (100.0 * self.get_stage_multiplier(stages.evasion)) as u32,
        }
    }
    
    fn get_stage_multiplier(&self, stage: i8) -> f32 {
        let index = (stage + 6) as usize;
        self.stat_stage_multipliers.get(index).copied().unwrap_or(1.0)
    }
    
    fn apply_modifiers(&self, stats: &mut ActualStats, modifiers: &HashMap<String, f32>) {
        for (modifier_name, value) in modifiers {
            match modifier_name.as_str() {
                "attack_boost" => stats.attack = (stats.attack as f32 * value) as u32,
                "defense_boost" => stats.defense = (stats.defense as f32 * value) as u32,
                "speed_boost" => stats.speed = (stats.speed as f32 * value) as u32,
                "sp_attack_boost" => stats.sp_attack = (stats.sp_attack as f32 * value) as u32,
                "sp_defense_boost" => stats.sp_defense = (stats.sp_defense as f32 * value) as u32,
                _ => {
                    debug!("未知的能力值修正器: {}", modifier_name);
                }
            }
        }
    }
    
    fn generate_cache_key(&self, pokemon_stats: &PokemonStats) -> String {
        format!(
            "{}_{}_{:?}_{:?}_{:?}_{:?}",
            pokemon_stats.species_id,
            pokemon_stats.level,
            pokemon_stats.nature,
            pokemon_stats.individual_values.hp, // 简化的缓存键
            pokemon_stats.effort_values.hp,
            pokemon_stats.stat_stages.attack
        )
    }
    
    fn get_stat_value(&self, stats: &ActualStats, stat_type: StatType) -> u32 {
        match stat_type {
            StatType::HP => stats.hp,
            StatType::Attack => stats.attack,
            StatType::Defense => stats.defense,
            StatType::SpAttack => stats.sp_attack,
            StatType::SpDefense => stats.sp_defense,
            StatType::Speed => stats.speed,
        }
    }
    
    fn get_effort_value(&self, evs: &EffortValues, stat_type: StatType) -> u8 {
        match stat_type {
            StatType::HP => evs.hp,
            StatType::Attack => evs.attack,
            StatType::Defense => evs.defense,
            StatType::SpAttack => evs.sp_attack,
            StatType::SpDefense => evs.sp_defense,
            StatType::Speed => evs.speed,
        }
    }
    
    fn calculate_total_evs(&self, evs: &EffortValues) -> u16 {
        evs.hp as u16 + evs.attack as u16 + evs.defense as u16 +
        evs.sp_attack as u16 + evs.sp_defense as u16 + evs.speed as u16
    }
}

// 个体值品质评级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IvQuality {
    Poor,        // 差劣 (0-90)
    Fair,        // 一般 (91-120)
    Good,        // 良好 (121-150)
    Excellent,   // 优秀 (151-170)
    Outstanding, // 杰出 (171-185)
    Perfect,     // 完美 (186)
}

// 统计管理器统计信息
#[derive(Debug, Clone)]
pub struct StatsManagerStats {
    pub total_calculations: u64,
    pub cache_hits: u64,
    pub cache_hit_rate: f32,
    pub cached_entries: usize,
}

// 默认实现
impl Default for BaseStats {
    fn default() -> Self {
        Self {
            hp: 45,
            attack: 45,
            defense: 45,
            sp_attack: 45,
            sp_defense: 45,
            speed: 45,
        }
    }
}

impl Default for IndividualValues {
    fn default() -> Self {
        Self {
            hp: 0,
            attack: 0,
            defense: 0,
            sp_attack: 0,
            sp_defense: 0,
            speed: 0,
        }
    }
}

impl Default for EffortValues {
    fn default() -> Self {
        Self {
            hp: 0,
            attack: 0,
            defense: 0,
            sp_attack: 0,
            sp_defense: 0,
            speed: 0,
        }
    }
}

impl Default for StatStages {
    fn default() -> Self {
        Self {
            attack: 0,
            defense: 0,
            sp_attack: 0,
            sp_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        }
    }
}

impl PokemonStats {
    pub fn new(species_id: u32, level: u8, nature: Nature, base_stats: BaseStats) -> Self {
        Self {
            species_id,
            level,
            nature,
            base_stats,
            individual_values: IndividualValues::default(),
            effort_values: EffortValues::default(),
            stat_stages: StatStages::default(),
            actual_stats: ActualStats {
                hp: 1,
                attack: 1,
                defense: 1,
                sp_attack: 1,
                sp_defense: 1,
                speed: 1,
                accuracy: 100,
                evasion: 100,
            },
            hidden_power_type: None,
            hidden_power_power: 30,
            stat_history: Vec::new(),
            stat_modifiers: HashMap::new(),
            permanent_modifiers: HashMap::new(),
        }
    }
    
    // 获取能力值总评
    pub fn get_stat_total(&self) -> u32 {
        self.actual_stats.hp + self.actual_stats.attack + self.actual_stats.defense +
        self.actual_stats.sp_attack + self.actual_stats.sp_defense + self.actual_stats.speed
    }
    
    // 添加临时修正
    pub fn add_temporary_modifier(&mut self, name: String, value: f32) {
        self.stat_modifiers.insert(name, value);
    }
    
    // 移除临时修正
    pub fn remove_temporary_modifier(&mut self, name: &str) {
        self.stat_modifiers.remove(name);
    }
    
    // 清空所有临时修正
    pub fn clear_temporary_modifiers(&mut self) {
        self.stat_modifiers.clear();
    }
    
    // 获取能力值历史
    pub fn get_recent_changes(&self, count: usize) -> Vec<&StatChange> {
        self.stat_history.iter().rev().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stats_manager_creation() {
        let manager = StatsManager::new();
        assert_eq!(manager.max_ev_total, 510);
        assert_eq!(manager.max_level, 100);
        assert!(!manager.nature_modifiers.is_empty());
    }
    
    #[test]
    fn test_stat_calculation() {
        let mut manager = StatsManager::new();
        let mut pokemon = PokemonStats::new(
            1, 
            50, 
            Nature::Adamant, 
            BaseStats {
                hp: 45,
                attack: 49,
                defense: 49,
                sp_attack: 65,
                sp_defense: 65,
                speed: 45,
            }
        );
        
        pokemon.individual_values = IndividualValues {
            hp: 31,
            attack: 31,
            defense: 31,
            sp_attack: 31,
            sp_defense: 31,
            speed: 31,
        };
        
        manager.calculate_stats(&mut pokemon).unwrap();
        
        // 验证HP计算
        assert!(pokemon.actual_stats.hp > 100);
        // 验证性格对攻击的加成
        assert!(pokemon.actual_stats.attack > pokemon.actual_stats.sp_attack);
    }
    
    #[test]
    fn test_iv_generation() {
        let manager = StatsManager::new();
        
        let random_ivs = manager.generate_random_ivs();
        assert!(random_ivs.hp <= 31);
        assert!(random_ivs.attack <= 31);
        
        let perfect_ivs = manager.generate_perfect_ivs();
        assert_eq!(manager.calculate_iv_total(&perfect_ivs), 186);
        
        let quality = manager.evaluate_iv_quality(&perfect_ivs);
        assert_eq!(quality, IvQuality::Perfect);
    }
    
    #[test]
    fn test_effort_value_training() {
        let mut manager = StatsManager::new();
        let mut pokemon = PokemonStats::new(1, 50, Nature::Hardy, BaseStats::default());
        
        let trained = manager.train_effort_value(&mut pokemon, StatType::Attack, 4).unwrap();
        assert_eq!(trained, 4);
        assert_eq!(pokemon.effort_values.attack, 4);
    }
    
    #[test]
    fn test_level_up() {
        let mut manager = StatsManager::new();
        let mut pokemon = PokemonStats::new(1, 49, Nature::Hardy, BaseStats::default());
        
        manager.calculate_stats(&mut pokemon).unwrap();
        let old_hp = pokemon.actual_stats.hp;
        
        let changes = manager.level_up(&mut pokemon).unwrap();
        assert_eq!(pokemon.level, 50);
        assert!(pokemon.actual_stats.hp > old_hp);
        assert!(!changes.is_empty());
    }
    
    #[test]
    fn test_stat_stages() {
        let mut manager = StatsManager::new();
        let mut pokemon = PokemonStats::new(1, 50, Nature::Hardy, BaseStats::default());
        
        manager.calculate_stats(&mut pokemon).unwrap();
        let old_attack = pokemon.actual_stats.attack;
        
        manager.apply_stat_change(&mut pokemon, StatType::Attack, 2).unwrap();
        assert!(pokemon.actual_stats.attack > old_attack);
        assert_eq!(pokemon.stat_stages.attack, 2);
    }
    
    #[test]
    fn test_hidden_power() {
        let manager = StatsManager::new();
        let ivs = IndividualValues {
            hp: 30,
            attack: 31,
            defense: 30,
            sp_attack: 31,
            sp_defense: 30,
            speed: 31,
        };
        
        let (type_id, power) = manager.calculate_hidden_power(&ivs);
        assert!(type_id.is_some());
        assert!(power >= 30 && power <= 70);
    }
}