// 宝可梦进化系统
// 开发心理：进化是宝可梦的核心机制，需要多样条件、动画表现、数据保持
// 设计原则：条件验证、状态管理、动画集成、数据完整性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;
use crate::pokemon::species::SpeciesId;
use crate::pokemon::moves::MoveId;
use crate::pokemon::types::PokemonType;
use crate::battle::status_effects::StatusEffectType;

// 进化条件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionConditionType {
    // 基础条件
    Level(u8),                      // 等级
    Friendship(u8),                 // 亲密度
    Trade,                          // 交换
    TradeWithItem(u32),            // 携带道具交换
    UseItem(u32),                  // 使用道具
    
    // 时间条件
    TimeOfDay(TimeOfDay),          // 时间段
    DayOfWeek(u8),                 // 星期几
    
    // 属性条件
    HighAttack,                    // 攻击力高于防御力
    HighDefense,                   // 防御力高于攻击力
    EqualAttackDefense,            // 攻防相等
    
    // 环境条件
    Location(String),              // 特定地点
    Weather(StatusEffectType),     // 天气条件
    MapType(MapType),              // 地图类型
    
    // 技能条件
    KnowsMove(MoveId),            // 学会特定技能
    MoveType(PokemonType),        // 学会特定属性技能
    
    // 队伍条件
    PartyMember(SpeciesId),       // 队伍中有特定宝可梦
    PartyFull,                    // 队伍满员
    PartyEmpty(u8),               // 队伍中有n个空位
    
    // 统计条件
    BattlesWon(u32),              // 胜利场次
    StepsWalked(u32),             // 行走步数
    DamageDealt(u64),             // 累计造成伤害
    DamageTaken(u64),             // 累计承受伤害
    
    // 特殊条件
    Gender(Gender),               // 性别
    Nature(String),               // 性格
    HeldItem(u32),                // 携带道具
    StatusEffect(StatusEffectType), // 特定状态效果
    Random(f32),                  // 随机概率
    
    // 组合条件
    And(Vec<EvolutionConditionType>), // 与条件
    Or(Vec<EvolutionConditionType>),  // 或条件
    Not(Box<EvolutionConditionType>), // 非条件
    
    // 自定义条件
    Custom(String),
}

// 时间段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,    // 早晨 (6:00-12:00)
    Afternoon,  // 下午 (12:00-18:00)
    Evening,    // 傍晚 (18:00-20:00)
    Night,      // 夜晚 (20:00-6:00)
}

// 地图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapType {
    Grassland,  // 草地
    Forest,     // 森林
    Mountain,   // 山区
    Cave,       // 洞穴
    Beach,      // 海滩
    City,       // 城市
    Route,      // 道路
    Building,   // 建筑物
}

// 性别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,       // 雄性
    Female,     // 雌性
    Genderless, // 无性别
}

// 进化方式
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionMethod {
    Standard,               // 标准进化
    Branched(String),       // 分支进化
    Mega(u32),              // 超级进化(需要道具)
    Gigantamax,            // 超极巨化
    Regional(String),       // 地区形态
    Trade,                 // 交换进化
    Special(String),       // 特殊进化
}

// 进化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evolution {
    pub id: u32,
    pub pre_evolution: SpeciesId,      // 进化前物种
    pub post_evolution: SpeciesId,     // 进化后物种
    pub method: EvolutionMethod,        // 进化方式
    pub conditions: Vec<EvolutionConditionType>, // 进化条件
    pub trigger_event: EvolutionTrigger, // 触发事件
    pub can_be_cancelled: bool,        // 是否可取消
    pub animation_id: Option<String>,   // 动画ID
    pub required_items: Vec<u32>,      // 所需道具
    pub consumed_items: Vec<u32>,      // 消耗道具
    pub level_requirement: Option<u8>, // 等级要求
    pub friendship_requirement: Option<u8>, // 亲密度要求
    pub metadata: HashMap<String, String>, // 额外数据
}

// 进化触发事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionTrigger {
    LevelUp,        // 升级时
    Trade,          // 交换时
    UseItem,        // 使用道具时
    TimeChange,     // 时间变化时
    LocationChange, // 地点变化时
    BattleEnd,      // 战斗结束时
    Manual,         // 手动触发
    Automatic,      // 自动触发
}

// 进化结果
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub success: bool,
    pub pre_evolution_id: SpeciesId,
    pub post_evolution_id: SpeciesId,
    pub evolution_id: u32,
    pub animation_triggered: bool,
    pub items_consumed: Vec<u32>,
    pub stats_changed: bool,
    pub moves_learned: Vec<MoveId>,
    pub moves_forgotten: Vec<MoveId>,
    pub abilities_changed: Vec<u16>,
    pub message: String,
    pub can_be_undone: bool,
}

// 进化链
#[derive(Debug, Clone)]
pub struct EvolutionChain {
    pub chain_id: u32,
    pub base_species: SpeciesId,
    pub evolutions: Vec<Evolution>,
    pub branch_count: u8,
    pub max_stage: u8,
    pub special_conditions: Vec<String>,
}

// 进化状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionState {
    NotEvolving,        // 未进化
    ConditionsMet,      // 满足条件
    Evolving,           // 进化中
    EvolutionComplete,  // 进化完成
    EvolutionCancelled, // 进化取消
    EvolutionFailed,    // 进化失败
}

// 进化上下文
#[derive(Debug, Clone)]
pub struct EvolutionContext {
    pub pokemon_id: u32,
    pub current_species: SpeciesId,
    pub level: u8,
    pub experience: u64,
    pub friendship: u8,
    pub nature: String,
    pub gender: Gender,
    pub held_item: Option<u32>,
    pub location: String,
    pub time_of_day: TimeOfDay,
    pub weather: Option<StatusEffectType>,
    pub map_type: MapType,
    pub party_members: Vec<SpeciesId>,
    pub known_moves: Vec<MoveId>,
    pub battle_stats: BattleStats,
    pub status_effects: Vec<StatusEffectType>,
    pub trainer_id: u32,
}

// 战斗统计
#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    pub battles_won: u32,
    pub battles_lost: u32,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub steps_walked: u32,
    pub items_used: u32,
}

// 进化管理器
pub struct EvolutionManager {
    // 进化数据
    evolutions: HashMap<u32, Evolution>,
    evolution_chains: HashMap<u32, EvolutionChain>,
    species_evolutions: HashMap<SpeciesId, Vec<u32>>, // 物种对应的进化ID列表
    
    // 当前进化状态
    active_evolutions: HashMap<u32, EvolutionState>, // pokemon_id -> state
    pending_evolutions: Vec<PendingEvolution>,
    
    // 配置
    allow_evolution_cancellation: bool,
    enable_evolution_animations: bool,
    auto_evolve: bool,
    skip_evolution_cutscenes: bool,
    
    // 统计
    total_evolutions: u64,
    successful_evolutions: u64,
    cancelled_evolutions: u64,
    evolution_history: Vec<EvolutionRecord>,
}

// 待处理进化
#[derive(Debug, Clone)]
struct PendingEvolution {
    pokemon_id: u32,
    evolution_id: u32,
    trigger_time: std::time::Instant,
    context: EvolutionContext,
    can_be_delayed: bool,
}

// 进化记录
#[derive(Debug, Clone)]
struct EvolutionRecord {
    pokemon_id: u32,
    evolution_id: u32,
    timestamp: std::time::Instant,
    success: bool,
    method: EvolutionMethod,
    trigger: EvolutionTrigger,
}

impl EvolutionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            evolutions: HashMap::new(),
            evolution_chains: HashMap::new(),
            species_evolutions: HashMap::new(),
            active_evolutions: HashMap::new(),
            pending_evolutions: Vec::new(),
            allow_evolution_cancellation: true,
            enable_evolution_animations: true,
            auto_evolve: false,
            skip_evolution_cutscenes: false,
            total_evolutions: 0,
            successful_evolutions: 0,
            cancelled_evolutions: 0,
            evolution_history: Vec::new(),
        };
        
        manager.initialize_evolution_data();
        manager
    }
    
    // 检查进化条件
    pub fn check_evolution_conditions(
        &self,
        pokemon_id: u32,
        context: &EvolutionContext,
    ) -> Vec<u32> {
        let mut available_evolutions = Vec::new();
        
        if let Some(evolution_ids) = self.species_evolutions.get(&context.current_species) {
            for &evolution_id in evolution_ids {
                if let Some(evolution) = self.evolutions.get(&evolution_id) {
                    if self.evaluate_conditions(&evolution.conditions, context) {
                        available_evolutions.push(evolution_id);
                    }
                }
            }
        }
        
        debug!("宝可梦 {} 可进化选项: {:?}", pokemon_id, available_evolutions);
        available_evolutions
    }
    
    // 触发进化
    pub fn trigger_evolution(
        &mut self,
        pokemon_id: u32,
        evolution_id: u32,
        context: EvolutionContext,
        force: bool,
    ) -> Result<EvolutionResult, GameError> {
        let evolution = self.evolutions.get(&evolution_id)
            .ok_or_else(|| GameError::Evolution(format!("进化数据不存在: {}", evolution_id)))?
            .clone();
        
        // 验证条件（除非强制进化）
        if !force && !self.evaluate_conditions(&evolution.conditions, &context) {
            return Err(GameError::Evolution("进化条件不满足".to_string()));
        }
        
        // 检查是否可以进化
        if let Some(&current_state) = self.active_evolutions.get(&pokemon_id) {
            if current_state == EvolutionState::Evolving {
                return Err(GameError::Evolution("宝可梦正在进化中".to_string()));
            }
        }
        
        // 开始进化过程
        self.active_evolutions.insert(pokemon_id, EvolutionState::Evolving);
        
        // 消耗道具
        let mut consumed_items = Vec::new();
        for &item_id in &evolution.consumed_items {
            // 这里应该调用物品系统来消耗道具
            consumed_items.push(item_id);
            debug!("消耗道具: {}", item_id);
        }
        
        // 执行进化
        let result = self.execute_evolution(pokemon_id, &evolution, &context)?;
        
        // 更新状态
        if result.success {
            self.active_evolutions.insert(pokemon_id, EvolutionState::EvolutionComplete);
            self.successful_evolutions += 1;
            
            // 记录历史
            self.record_evolution(pokemon_id, evolution_id, &evolution, EvolutionTrigger::Manual, true);
        } else {
            self.active_evolutions.insert(pokemon_id, EvolutionState::EvolutionFailed);
        }
        
        self.total_evolutions += 1;
        
        Ok(result)
    }
    
    // 取消进化
    pub fn cancel_evolution(&mut self, pokemon_id: u32) -> Result<(), GameError> {
        if !self.allow_evolution_cancellation {
            return Err(GameError::Evolution("进化取消功能已禁用".to_string()));
        }
        
        let current_state = self.active_evolutions.get(&pokemon_id).copied();
        
        match current_state {
            Some(EvolutionState::ConditionsMet) | Some(EvolutionState::Evolving) => {
                self.active_evolutions.insert(pokemon_id, EvolutionState::EvolutionCancelled);
                self.cancelled_evolutions += 1;
                debug!("取消宝可梦 {} 的进化", pokemon_id);
                Ok(())
            }
            Some(EvolutionState::EvolutionComplete) => {
                Err(GameError::Evolution("进化已完成，无法取消".to_string()))
            }
            _ => {
                Err(GameError::Evolution("没有正在进行的进化".to_string()))
            }
        }
    }
    
    // 添加待处理进化
    pub fn add_pending_evolution(
        &mut self,
        pokemon_id: u32,
        evolution_id: u32,
        context: EvolutionContext,
    ) {
        let pending = PendingEvolution {
            pokemon_id,
            evolution_id,
            trigger_time: std::time::Instant::now(),
            context,
            can_be_delayed: true,
        };
        
        self.pending_evolutions.push(pending);
        self.active_evolutions.insert(pokemon_id, EvolutionState::ConditionsMet);
        
        debug!("添加待处理进化: 宝可梦 {} -> 进化 {}", pokemon_id, evolution_id);
    }
    
    // 处理待处理的进化
    pub fn process_pending_evolutions(&mut self) -> Vec<EvolutionResult> {
        let mut results = Vec::new();
        let mut completed_indices = Vec::new();
        
        for (index, pending) in self.pending_evolutions.iter().enumerate() {
            if self.auto_evolve || self.should_auto_trigger(pending) {
                match self.trigger_evolution(
                    pending.pokemon_id,
                    pending.evolution_id,
                    pending.context.clone(),
                    false,
                ) {
                    Ok(result) => {
                        results.push(result);
                        completed_indices.push(index);
                    }
                    Err(e) => {
                        warn!("自动进化失败: {}", e);
                    }
                }
            }
        }
        
        // 移除已处理的进化
        for &index in completed_indices.iter().rev() {
            self.pending_evolutions.remove(index);
        }
        
        results
    }
    
    // 获取进化链信息
    pub fn get_evolution_chain(&self, species_id: SpeciesId) -> Option<&EvolutionChain> {
        // 查找包含该物种的进化链
        self.evolution_chains.values().find(|chain| {
            chain.base_species == species_id || 
            chain.evolutions.iter().any(|evo| 
                evo.pre_evolution == species_id || evo.post_evolution == species_id
            )
        })
    }
    
    // 获取可能的进化形态
    pub fn get_possible_evolutions(&self, species_id: SpeciesId) -> Vec<&Evolution> {
        self.species_evolutions.get(&species_id)
            .map(|evolution_ids| {
                evolution_ids.iter()
                    .filter_map(|&id| self.evolutions.get(&id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    // 获取进化状态
    pub fn get_evolution_state(&self, pokemon_id: u32) -> EvolutionState {
        self.active_evolutions.get(&pokemon_id).copied().unwrap_or(EvolutionState::NotEvolving)
    }
    
    // 设置配置
    pub fn set_auto_evolve(&mut self, enabled: bool) {
        self.auto_evolve = enabled;
        debug!("自动进化设置: {}", enabled);
    }
    
    pub fn set_animation_enabled(&mut self, enabled: bool) {
        self.enable_evolution_animations = enabled;
        debug!("进化动画设置: {}", enabled);
    }
    
    pub fn set_cancellation_allowed(&mut self, allowed: bool) {
        self.allow_evolution_cancellation = allowed;
        debug!("进化取消设置: {}", allowed);
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> EvolutionStats {
        EvolutionStats {
            total_evolutions: self.total_evolutions,
            successful_evolutions: self.successful_evolutions,
            cancelled_evolutions: self.cancelled_evolutions,
            success_rate: if self.total_evolutions > 0 {
                self.successful_evolutions as f32 / self.total_evolutions as f32
            } else {
                0.0
            },
            pending_evolutions: self.pending_evolutions.len(),
            active_evolutions: self.active_evolutions.len(),
        }
    }
    
    // 私有方法
    fn initialize_evolution_data(&mut self) {
        self.load_standard_evolutions();
        self.load_special_evolutions();
        self.build_evolution_chains();
        self.index_species_evolutions();
    }
    
    fn load_standard_evolutions(&mut self) {
        // 小火龙进化链
        let charmander_evolution = Evolution {
            id: 1,
            pre_evolution: 4,  // 小火龙
            post_evolution: 5, // 火恐龙
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(16)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("charmander_to_charmeleon".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(16),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(1, charmander_evolution);
        
        let charmeleon_evolution = Evolution {
            id: 2,
            pre_evolution: 5,  // 火恐龙
            post_evolution: 6, // 喷火龙
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(36)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("charmeleon_to_charizard".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(36),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(2, charmeleon_evolution);
        
        // 杰尼龟进化链
        let squirtle_evolution = Evolution {
            id: 3,
            pre_evolution: 7,  // 杰尼龟
            post_evolution: 8, // 卡咪龟
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(16)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("squirtle_to_wartortle".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(16),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(3, squirtle_evolution);
        
        let wartortle_evolution = Evolution {
            id: 4,
            pre_evolution: 8,  // 卡咪龟
            post_evolution: 9, // 水箭龟
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(36)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("wartortle_to_blastoise".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(36),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(4, wartortle_evolution);
        
        // 妙蛙种子进化链
        let bulbasaur_evolution = Evolution {
            id: 5,
            pre_evolution: 1,  // 妙蛙种子
            post_evolution: 2, // 妙蛙草
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(16)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("bulbasaur_to_ivysaur".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(16),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(5, bulbasaur_evolution);
        
        let ivysaur_evolution = Evolution {
            id: 6,
            pre_evolution: 2,  // 妙蛙草
            post_evolution: 3, // 妙蛙花
            method: EvolutionMethod::Standard,
            conditions: vec![EvolutionConditionType::Level(32)],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("ivysaur_to_venusaur".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: Some(32),
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(6, ivysaur_evolution);
    }
    
    fn load_special_evolutions(&mut self) {
        // 伊布的多分支进化
        let eevee_vaporeon = Evolution {
            id: 10,
            pre_evolution: 133, // 伊布
            post_evolution: 134, // 水伊布
            method: EvolutionMethod::Branched("water".to_string()),
            conditions: vec![EvolutionConditionType::UseItem(1)], // 水之石
            trigger_event: EvolutionTrigger::UseItem,
            can_be_cancelled: false,
            animation_id: Some("eevee_to_vaporeon".to_string()),
            required_items: vec![1], // 水之石
            consumed_items: vec![1],
            level_requirement: None,
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(10, eevee_vaporeon);
        
        let eevee_jolteon = Evolution {
            id: 11,
            pre_evolution: 133, // 伊布
            post_evolution: 135, // 雷伊布
            method: EvolutionMethod::Branched("electric".to_string()),
            conditions: vec![EvolutionConditionType::UseItem(2)], // 雷之石
            trigger_event: EvolutionTrigger::UseItem,
            can_be_cancelled: false,
            animation_id: Some("eevee_to_jolteon".to_string()),
            required_items: vec![2], // 雷之石
            consumed_items: vec![2],
            level_requirement: None,
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(11, eevee_jolteon);
        
        let eevee_flareon = Evolution {
            id: 12,
            pre_evolution: 133, // 伊布
            post_evolution: 136, // 火伊布
            method: EvolutionMethod::Branched("fire".to_string()),
            conditions: vec![EvolutionConditionType::UseItem(3)], // 火之石
            trigger_event: EvolutionTrigger::UseItem,
            can_be_cancelled: false,
            animation_id: Some("eevee_to_flareon".to_string()),
            required_items: vec![3], // 火之石
            consumed_items: vec![3],
            level_requirement: None,
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(12, eevee_flareon);
        
        // 友好度进化示例
        let pichu_evolution = Evolution {
            id: 20,
            pre_evolution: 172, // 皮丘
            post_evolution: 25,  // 皮卡丘
            method: EvolutionMethod::Standard,
            conditions: vec![
                EvolutionConditionType::Friendship(220),
                EvolutionConditionType::TimeOfDay(TimeOfDay::Morning),
            ],
            trigger_event: EvolutionTrigger::LevelUp,
            can_be_cancelled: true,
            animation_id: Some("pichu_to_pikachu".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: None,
            friendship_requirement: Some(220),
            metadata: HashMap::new(),
        };
        self.evolutions.insert(20, pichu_evolution);
        
        // 交换进化示例
        let machoke_evolution = Evolution {
            id: 30,
            pre_evolution: 67, // 豪力
            post_evolution: 68, // 怪力
            method: EvolutionMethod::Trade,
            conditions: vec![EvolutionConditionType::Trade],
            trigger_event: EvolutionTrigger::Trade,
            can_be_cancelled: false,
            animation_id: Some("machoke_to_machamp".to_string()),
            required_items: Vec::new(),
            consumed_items: Vec::new(),
            level_requirement: None,
            friendship_requirement: None,
            metadata: HashMap::new(),
        };
        self.evolutions.insert(30, machoke_evolution);
    }
    
    fn build_evolution_chains(&mut self) {
        // 御三家进化链
        let starter_chain_fire = EvolutionChain {
            chain_id: 1,
            base_species: 4, // 小火龙
            evolutions: vec![
                self.evolutions[&1].clone(), // 小火龙 -> 火恐龙
                self.evolutions[&2].clone(), // 火恐龙 -> 喷火龙
            ],
            branch_count: 0,
            max_stage: 2,
            special_conditions: Vec::new(),
        };
        self.evolution_chains.insert(1, starter_chain_fire);
        
        let starter_chain_water = EvolutionChain {
            chain_id: 2,
            base_species: 7, // 杰尼龟
            evolutions: vec![
                self.evolutions[&3].clone(), // 杰尼龟 -> 卡咪龟
                self.evolutions[&4].clone(), // 卡咪龟 -> 水箭龟
            ],
            branch_count: 0,
            max_stage: 2,
            special_conditions: Vec::new(),
        };
        self.evolution_chains.insert(2, starter_chain_water);
        
        let starter_chain_grass = EvolutionChain {
            chain_id: 3,
            base_species: 1, // 妙蛙种子
            evolutions: vec![
                self.evolutions[&5].clone(), // 妙蛙种子 -> 妙蛙草
                self.evolutions[&6].clone(), // 妙蛙草 -> 妙蛙花
            ],
            branch_count: 0,
            max_stage: 2,
            special_conditions: Vec::new(),
        };
        self.evolution_chains.insert(3, starter_chain_grass);
        
        // 伊布分支进化链
        let eevee_chain = EvolutionChain {
            chain_id: 10,
            base_species: 133, // 伊布
            evolutions: vec![
                self.evolutions[&10].clone(), // 伊布 -> 水伊布
                self.evolutions[&11].clone(), // 伊布 -> 雷伊布
                self.evolutions[&12].clone(), // 伊布 -> 火伊布
            ],
            branch_count: 3,
            max_stage: 1,
            special_conditions: vec!["需要进化石".to_string()],
        };
        self.evolution_chains.insert(10, eevee_chain);
    }
    
    fn index_species_evolutions(&mut self) {
        for (&evolution_id, evolution) in &self.evolutions {
            self.species_evolutions
                .entry(evolution.pre_evolution)
                .or_insert_with(Vec::new)
                .push(evolution_id);
        }
    }
    
    fn evaluate_conditions(&self, conditions: &[EvolutionConditionType], context: &EvolutionContext) -> bool {
        for condition in conditions {
            if !self.evaluate_single_condition(condition, context) {
                return false;
            }
        }
        true
    }
    
    fn evaluate_single_condition(&self, condition: &EvolutionConditionType, context: &EvolutionContext) -> bool {
        match condition {
            EvolutionConditionType::Level(required_level) => {
                context.level >= *required_level
            }
            EvolutionConditionType::Friendship(required_friendship) => {
                context.friendship >= *required_friendship
            }
            EvolutionConditionType::Trade => {
                // 这里需要检查是否在交换过程中
                false // 简化实现
            }
            EvolutionConditionType::UseItem(item_id) => {
                context.held_item == Some(*item_id)
            }
            EvolutionConditionType::TimeOfDay(time) => {
                context.time_of_day == *time
            }
            EvolutionConditionType::Gender(gender) => {
                context.gender == *gender
            }
            EvolutionConditionType::Location(location) => {
                context.location == *location
            }
            EvolutionConditionType::Weather(weather) => {
                context.weather == Some(*weather)
            }
            EvolutionConditionType::KnowsMove(move_id) => {
                context.known_moves.contains(move_id)
            }
            EvolutionConditionType::PartyMember(species_id) => {
                context.party_members.contains(species_id)
            }
            EvolutionConditionType::And(conditions) => {
                conditions.iter().all(|cond| self.evaluate_single_condition(cond, context))
            }
            EvolutionConditionType::Or(conditions) => {
                conditions.iter().any(|cond| self.evaluate_single_condition(cond, context))
            }
            EvolutionConditionType::Not(condition) => {
                !self.evaluate_single_condition(condition, context)
            }
            EvolutionConditionType::Random(probability) => {
                fastrand::f32() < *probability
            }
            // 其他条件的实现...
            _ => true, // 未实现的条件默认为true
        }
    }
    
    fn execute_evolution(
        &self,
        pokemon_id: u32,
        evolution: &Evolution,
        context: &EvolutionContext,
    ) -> Result<EvolutionResult, GameError> {
        debug!("执行进化: {} -> {}", evolution.pre_evolution, evolution.post_evolution);
        
        // 创建进化结果
        let mut result = EvolutionResult {
            success: true,
            pre_evolution_id: evolution.pre_evolution,
            post_evolution_id: evolution.post_evolution,
            evolution_id: evolution.id,
            animation_triggered: self.enable_evolution_animations && evolution.animation_id.is_some(),
            items_consumed: evolution.consumed_items.clone(),
            stats_changed: true,
            moves_learned: Vec::new(), // 这里需要从物种数据中获取
            moves_forgotten: Vec::new(),
            abilities_changed: Vec::new(),
            message: format!("恭喜！{}进化成{}了！", evolution.pre_evolution, evolution.post_evolution),
            can_be_undone: false,
        };
        
        // 触发动画
        if result.animation_triggered {
            if let Some(animation_id) = &evolution.animation_id {
                debug!("触发进化动画: {}", animation_id);
                // 这里应该调用动画系统
            }
        }
        
        // 学习新技能
        result.moves_learned = self.get_evolution_moves(evolution.post_evolution);
        
        // 更新能力
        result.abilities_changed = self.get_evolution_abilities(evolution.post_evolution);
        
        debug!("进化完成: {}", result.message);
        Ok(result)
    }
    
    fn get_evolution_moves(&self, species_id: SpeciesId) -> Vec<MoveId> {
        // 这里应该查询物种数据库获取进化时学会的技能
        match species_id {
            2 => vec![1, 2], // 妙蛙草学会的技能
            3 => vec![3, 4], // 妙蛙花学会的技能
            5 => vec![5, 6], // 火恐龙学会的技能
            6 => vec![7, 8], // 喷火龙学会的技能
            8 => vec![9, 10], // 卡咪龟学会的技能
            9 => vec![11, 12], // 水箭龟学会的技能
            _ => Vec::new(),
        }
    }
    
    fn get_evolution_abilities(&self, species_id: SpeciesId) -> Vec<u16> {
        // 这里应该查询物种数据库获取进化后的特性
        match species_id {
            3 => vec![1], // 妙蛙花的特性
            6 => vec![2], // 喷火龙的特性
            9 => vec![3], // 水箭龟的特性
            _ => Vec::new(),
        }
    }
    
    fn should_auto_trigger(&self, pending: &PendingEvolution) -> bool {
        // 检查是否应该自动触发进化
        let elapsed = pending.trigger_time.elapsed().as_secs();
        
        // 如果等待超过5秒，自动触发
        elapsed > 5
    }
    
    fn record_evolution(
        &mut self,
        pokemon_id: u32,
        evolution_id: u32,
        evolution: &Evolution,
        trigger: EvolutionTrigger,
        success: bool,
    ) {
        let record = EvolutionRecord {
            pokemon_id,
            evolution_id,
            timestamp: std::time::Instant::now(),
            success,
            method: evolution.method.clone(),
            trigger,
        };
        
        self.evolution_history.push(record);
        
        // 限制历史记录大小
        if self.evolution_history.len() > 1000 {
            self.evolution_history.remove(0);
        }
    }
}

// 统计信息
#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub total_evolutions: u64,
    pub successful_evolutions: u64,
    pub cancelled_evolutions: u64,
    pub success_rate: f32,
    pub pending_evolutions: usize,
    pub active_evolutions: usize,
}

// 工具函数
impl TimeOfDay {
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=19 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }
    
    pub fn current() -> Self {
        use std::time::SystemTime;
        
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let hours = ((now / 3600) % 24) as u8;
        Self::from_hour(hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evolution_manager_creation() {
        let manager = EvolutionManager::new();
        assert!(!manager.evolutions.is_empty());
        assert!(!manager.evolution_chains.is_empty());
    }
    
    #[test]
    fn test_level_condition() {
        let manager = EvolutionManager::new();
        
        let context = EvolutionContext {
            pokemon_id: 1,
            current_species: 4, // 小火龙
            level: 16,
            experience: 2000,
            friendship: 70,
            nature: "温和".to_string(),
            gender: Gender::Male,
            held_item: None,
            location: "真新镇".to_string(),
            time_of_day: TimeOfDay::Morning,
            weather: None,
            map_type: MapType::City,
            party_members: Vec::new(),
            known_moves: vec![1, 2, 3],
            battle_stats: BattleStats::default(),
            status_effects: Vec::new(),
            trainer_id: 1,
        };
        
        let available = manager.check_evolution_conditions(1, &context);
        assert!(!available.is_empty()); // 小火龙16级应该能进化
    }
    
    #[test]
    fn test_evolution_trigger() {
        let mut manager = EvolutionManager::new();
        
        let context = EvolutionContext {
            pokemon_id: 1,
            current_species: 4, // 小火龙
            level: 16,
            experience: 2000,
            friendship: 70,
            nature: "温和".to_string(),
            gender: Gender::Male,
            held_item: None,
            location: "真新镇".to_string(),
            time_of_day: TimeOfDay::Morning,
            weather: None,
            map_type: MapType::City,
            party_members: Vec::new(),
            known_moves: vec![1, 2, 3],
            battle_stats: BattleStats::default(),
            status_effects: Vec::new(),
            trainer_id: 1,
        };
        
        let result = manager.trigger_evolution(1, 1, context, false).unwrap();
        assert!(result.success);
        assert_eq!(result.pre_evolution_id, 4);
        assert_eq!(result.post_evolution_id, 5);
    }
    
    #[test]
    fn test_time_of_day() {
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(19), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(22), TimeOfDay::Night);
    }
}