// NPC系统
// 开发心理：NPC提供游戏交互、剧情推进、信息传递等功能，需要智能行为和对话系统
// 设计原则：行为树AI、对话系统、状态管理、任务分发

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::pokemon::stats::PokemonStats;
use glam::{Vec2, Vec3};

// NPC ID类型
pub type NPCId = u64;
pub type DialogueId = u32;

// NPC数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: NPCId,
    pub name: String,
    pub description: String,
    pub npc_type: NPCType,
    
    // 外观和位置
    pub sprite_id: u32,
    pub position: Vec3,
    pub facing_direction: Vec2,
    pub scale: Vec2,
    
    // AI和行为
    pub ai_behavior: AIBehavior,
    pub movement_pattern: MovementPattern,
    pub interaction_radius: f32,
    pub sight_range: f32,
    pub hearing_range: f32,
    
    // 对话系统
    pub dialogue_tree: Option<DialogueTree>,
    pub current_dialogue: Option<DialogueId>,
    pub dialogue_history: Vec<DialogueId>,
    
    // 状态和情绪
    pub mood: NPCMood,
    pub relationship_with_player: i32,  // -100到100
    pub memory: NPCMemory,
    
    // 交易系统
    pub shop: Option<Shop>,
    pub trade_offers: Vec<TradeOffer>,
    
    // 战斗系统
    pub trainer_data: Option<TrainerData>,
    pub battle_music: Option<String>,
    
    // 任务系统
    pub quests: Vec<u32>,               // 可提供的任务ID
    pub completed_player_quests: Vec<u32>,
    
    // 时间和调度
    pub daily_schedule: Schedule,
    pub active_times: Vec<(u8, u8)>,    // (开始时间, 结束时间)
    pub seasonal_behavior: HashMap<String, String>,
    
    // 自定义属性
    pub properties: HashMap<String, String>,
    pub flags: HashMap<String, bool>,
}

// NPC类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NPCType {
    Villager,       // 村民
    Shopkeeper,     // 商店老板
    Trainer,        // 训练师
    GymLeader,      // 道馆馆主
    Professor,      // 博士
    Nurse,          // 护士
    Officer,        // 警察
    Guide,          // 向导
    QuestGiver,     // 任务发布者
    Guard,          // 守卫
    Elder,          // 长老
    Child,          // 孩子
    Fisherman,      // 钓鱼者
    Hiker,          // 登山者
}

// AI行为类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehavior {
    Idle,                                           // idle
    Wander { radius: f32, speed: f32 },             // 随机游荡
    Patrol { points: Vec<Vec3>, speed: f32 },       // 巡逻
    Follow { target_id: u64, distance: f32 },       // 跟随
    Guard { area: (Vec3, f32) },                    // 守卫区域
    Hunt { target_type: String, aggro_range: f32 }, // 狩猎
    Custom { behavior_name: String, parameters: HashMap<String, f32> }, // 自定义
}

// 移动模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementPattern {
    pub pattern_type: MovementType,
    pub speed: f32,
    pub can_fly: bool,
    pub can_swim: bool,
    pub avoids_water: bool,
    pub prefers_roads: bool,
}

// 移动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    Stationary,     // 静止
    Random,         // 随机移动
    Scripted,       // 脚本移动
    PlayerFollowing,// 跟随玩家
    Aggressive,     // 主动攻击
    Defensive,      // 防御性
}

// NPC情绪
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NPCMood {
    Happy,          // 高兴
    Sad,            // 伤心
    Angry,          // 愤怒
    Neutral,        // 中性
    Excited,        // 兴奋
    Worried,        // 担心
    Confused,       // 困惑
    Friendly,       // 友好
    Suspicious,     // 怀疑
}

// NPC记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCMemory {
    pub last_interaction_time: Option<std::time::SystemTime>,
    pub interaction_count: u32,
    pub remembered_events: Vec<MemoryEvent>,
    pub known_facts: HashMap<String, String>,
    pub relationships: HashMap<u64, i32>,     // entity_id -> relationship
}

// 记忆事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub event_type: String,
    pub description: String,
    pub timestamp: std::time::SystemTime,
    pub importance: u8,                       // 0-10
    pub participants: Vec<u64>,               // 参与者ID
}

// 对话树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    pub root_node: DialogueId,
    pub nodes: HashMap<DialogueId, DialogueNode>,
    pub variables: HashMap<String, DialogueVariable>,
}

// 对话节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: DialogueId,
    pub speaker: String,
    pub text: String,
    pub conditions: Vec<DialogueCondition>,
    pub actions: Vec<DialogueAction>,
    pub choices: Vec<DialogueChoice>,
    pub next_node: Option<DialogueId>,
    pub auto_continue: bool,
    pub delay: f32,                           // 自动继续延迟
}

// 对话选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub next_node: DialogueId,
    pub conditions: Vec<DialogueCondition>,
    pub actions: Vec<DialogueAction>,
    pub cost: Option<DialogueCost>,
}

// 对话条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCondition {
    HasItem { item_id: u32, quantity: u32 },
    PlayerLevel { min_level: u32, max_level: Option<u32> },
    QuestCompleted { quest_id: u32 },
    Relationship { min_value: i32 },
    Flag { flag_name: String, value: bool },
    Variable { var_name: String, value: DialogueVariable },
    TimeOfDay { start_hour: u8, end_hour: u8 },
    Custom { condition_name: String, parameters: HashMap<String, String> },
}

// 对话动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueAction {
    GiveItem { item_id: u32, quantity: u32 },
    TakeItem { item_id: u32, quantity: u32 },
    GiveExperience { amount: u64 },
    ChangeRelationship { amount: i32 },
    SetFlag { flag_name: String, value: bool },
    SetVariable { var_name: String, value: DialogueVariable },
    StartQuest { quest_id: u32 },
    CompleteQuest { quest_id: u32 },
    PlaySound { sound_id: String },
    TriggerEvent { event_name: String, parameters: HashMap<String, String> },
    StartBattle { trainer_id: Option<u64> },
    OpenShop,
    Heal,
    Custom { action_name: String, parameters: HashMap<String, String> },
}

// 对话变量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogueVariable {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

// 对话花费
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCost {
    Money(u32),
    Items(Vec<(u32, u32)>),              // (item_id, quantity)
    Experience(u64),
}

// 商店系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub name: String,
    pub shop_type: ShopType,
    pub items: HashMap<u32, ShopItem>,
    pub buy_rate: f32,                    // 购买价格倍率
    pub sell_rate: f32,                   // 出售价格倍率
    pub special_deals: Vec<SpecialDeal>,
    pub restock_time: f32,                // 补货时间
    pub last_restock: std::time::SystemTime,
}

// 商店类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopType {
    General,        // 综合商店
    PokeMart,       // Pokemon中心商店
    Medicine,       // 药品店
    Pokeballs,      // 精灵球专店
    TechMachine,    // TM商店
    Berries,        // 树果店
    Equipment,      // 装备店
    Rare,           // 稀有物品店
}

// 商店物品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub item_id: u32,
    pub stock: i32,                       // -1表示无限库存
    pub price: u32,
    pub availability_conditions: Vec<DialogueCondition>,
}

// 特殊优惠
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialDeal {
    pub name: String,
    pub description: String,
    pub items: Vec<(u32, u32)>,           // (item_id, quantity)
    pub original_price: u32,
    pub sale_price: u32,
    pub conditions: Vec<DialogueCondition>,
    pub expiry: Option<std::time::SystemTime>,
}

// 交易提议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub requested_pokemon: PokemonRequest,
    pub offered_pokemon: PokemonOffer,
    pub conditions: Vec<DialogueCondition>,
    pub one_time_only: bool,
    pub completed: bool,
}

// Pokemon请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonRequest {
    pub species_id: u32,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    pub gender: Option<String>,
    pub nature: Option<String>,
    pub specific_moves: Vec<u32>,
}

// Pokemon提供
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonOffer {
    pub species_id: u32,
    pub level: u8,
    pub nickname: Option<String>,
    pub moves: Vec<u32>,
    pub held_item: Option<u32>,
    pub original_trainer: String,
}

// 训练师数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerData {
    pub trainer_class: String,
    pub title: Option<String>,
    pub pokemon_team: Vec<TrainerPokemon>,
    pub ai_difficulty: AIDifficulty,
    pub battle_intro: String,
    pub victory_text: String,
    pub defeat_text: String,
    pub rematch_available: bool,
    pub rematch_level_boost: u8,
    pub prize_money: u32,
}

// 训练师Pokemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerPokemon {
    pub species_id: u32,
    pub level: u8,
    pub nickname: Option<String>,
    pub moves: Vec<u32>,
    pub held_item: Option<u32>,
    pub ability: Option<u32>,
    pub stats: Option<PokemonStats>,
}

// AI难度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIDifficulty {
    Beginner,       // 新手
    Normal,         // 普通
    Hard,           // 困难
    Expert,         // 专家
    Champion,       // 冠军级
}

// 日程安排
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub daily_activities: HashMap<u8, ScheduledActivity>, // hour -> activity
    pub weekly_schedule: HashMap<u8, HashMap<u8, ScheduledActivity>>, // weekday -> hour -> activity
    pub special_events: HashMap<String, ScheduledActivity>,
}

// 计划活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledActivity {
    pub activity_type: String,
    pub location: Option<Vec3>,
    pub duration: f32,                    // 持续时间(小时)
    pub description: String,
    pub affects_availability: bool,
}

// NPC管理器
pub struct NPCManager {
    // NPC数据
    npcs: HashMap<NPCId, NPC>,
    next_npc_id: NPCId,
    
    // 活跃NPC
    active_npcs: Vec<NPCId>,
    npc_update_index: usize,              // 轮询更新索引
    
    // 对话系统
    active_dialogues: HashMap<NPCId, DialogueState>,
    dialogue_templates: HashMap<String, DialogueTree>,
    
    // AI更新
    ai_update_interval: f32,
    ai_update_timer: f32,
    npcs_per_ai_update: usize,
    
    // 统计
    total_npcs_created: u64,
    dialogues_started: u64,
    battles_initiated: u64,
    frame_count: u64,
}

// 对话状态
#[derive(Debug, Clone)]
pub struct DialogueState {
    pub current_node: DialogueId,
    pub variables: HashMap<String, DialogueVariable>,
    pub started_time: std::time::Instant,
    pub auto_continue_timer: f32,
}

impl NPCManager {
    pub fn new() -> Self {
        Self {
            npcs: HashMap::new(),
            next_npc_id: 1,
            active_npcs: Vec::new(),
            npc_update_index: 0,
            active_dialogues: HashMap::new(),
            dialogue_templates: HashMap::new(),
            ai_update_interval: 0.1,          // 每100ms更新一批NPC
            ai_update_timer: 0.0,
            npcs_per_ai_update: 5,            // 每次更新5个NPC
            total_npcs_created: 0,
            dialogues_started: 0,
            battles_initiated: 0,
            frame_count: 0,
        }
    }
    
    // 创建NPC
    pub fn create_npc(
        &mut self,
        name: String,
        npc_type: NPCType,
        position: Vec3,
        sprite_id: u32,
    ) -> Result<NPCId, GameError> {
        let npc_id = self.next_npc_id;
        self.next_npc_id += 1;
        
        let npc = NPC {
            id: npc_id,
            name: name.clone(),
            description: String::new(),
            npc_type,
            sprite_id,
            position,
            facing_direction: Vec2::new(0.0, -1.0),
            scale: Vec2::ONE,
            ai_behavior: AIBehavior::Idle,
            movement_pattern: MovementPattern {
                pattern_type: MovementType::Stationary,
                speed: 50.0,
                can_fly: false,
                can_swim: false,
                avoids_water: true,
                prefers_roads: false,
            },
            interaction_radius: 64.0,
            sight_range: 128.0,
            hearing_range: 96.0,
            dialogue_tree: None,
            current_dialogue: None,
            dialogue_history: Vec::new(),
            mood: NPCMood::Neutral,
            relationship_with_player: 0,
            memory: NPCMemory {
                last_interaction_time: None,
                interaction_count: 0,
                remembered_events: Vec::new(),
                known_facts: HashMap::new(),
                relationships: HashMap::new(),
            },
            shop: None,
            trade_offers: Vec::new(),
            trainer_data: None,
            battle_music: None,
            quests: Vec::new(),
            completed_player_quests: Vec::new(),
            daily_schedule: Schedule {
                daily_activities: HashMap::new(),
                weekly_schedule: HashMap::new(),
                special_events: HashMap::new(),
            },
            active_times: vec![(6, 22)], // 默认6点到22点活跃
            seasonal_behavior: HashMap::new(),
            properties: HashMap::new(),
            flags: HashMap::new(),
        };
        
        self.npcs.insert(npc_id, npc);
        self.active_npcs.push(npc_id);
        self.total_npcs_created += 1;
        
        debug!("创建NPC: '{}' ID={} 类型={:?}", name, npc_id, npc_type);
        Ok(npc_id)
    }
    
    // 获取NPC
    pub fn get_npc(&self, npc_id: NPCId) -> Option<&NPC> {
        self.npcs.get(&npc_id)
    }
    
    // 获取NPC(可变)
    pub fn get_npc_mut(&mut self, npc_id: NPCId) -> Option<&mut NPC> {
        self.npcs.get_mut(&npc_id)
    }
    
    // 开始对话
    pub fn start_dialogue(&mut self, npc_id: NPCId, player_id: u64) -> Result<Option<DialogueNode>, GameError> {
        if let Some(npc) = self.npcs.get_mut(&npc_id) {
            if let Some(ref dialogue_tree) = npc.dialogue_tree {
                let root_node = dialogue_tree.nodes.get(&dialogue_tree.root_node)
                    .ok_or_else(|| GameError::NPC("对话树根节点不存在".to_string()))?;
                
                // 检查对话条件
                if self.check_dialogue_conditions(&root_node.conditions, player_id, npc)? {
                    let dialogue_state = DialogueState {
                        current_node: dialogue_tree.root_node,
                        variables: dialogue_tree.variables.clone(),
                        started_time: std::time::Instant::now(),
                        auto_continue_timer: root_node.delay,
                    };
                    
                    self.active_dialogues.insert(npc_id, dialogue_state);
                    npc.current_dialogue = Some(dialogue_tree.root_node);
                    
                    // 更新NPC记忆
                    npc.memory.last_interaction_time = Some(std::time::SystemTime::now());
                    npc.memory.interaction_count += 1;
                    
                    self.dialogues_started += 1;
                    debug!("开始对话: NPC={} 节点={}", npc_id, dialogue_tree.root_node);
                    
                    return Ok(Some(root_node.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    // 选择对话选项
    pub fn choose_dialogue_option(
        &mut self,
        npc_id: NPCId,
        choice_index: usize,
        player_id: u64,
    ) -> Result<Option<DialogueNode>, GameError> {
        if let Some(dialogue_state) = self.active_dialogues.get_mut(&npc_id) {
            if let Some(npc) = self.npcs.get_mut(&npc_id) {
                if let Some(ref dialogue_tree) = npc.dialogue_tree {
                    if let Some(current_node) = dialogue_tree.nodes.get(&dialogue_state.current_node) {
                        if choice_index < current_node.choices.len() {
                            let choice = &current_node.choices[choice_index];
                            
                            // 检查选择条件
                            if self.check_dialogue_conditions(&choice.conditions, player_id, npc)? {
                                // 执行选择动作
                                self.execute_dialogue_actions(&choice.actions, player_id, npc)?;
                                
                                // 移动到下一个节点
                                dialogue_state.current_node = choice.next_node;
                                npc.current_dialogue = Some(choice.next_node);
                                
                                if let Some(next_node) = dialogue_tree.nodes.get(&choice.next_node) {
                                    dialogue_state.auto_continue_timer = next_node.delay;
                                    return Ok(Some(next_node.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    // 结束对话
    pub fn end_dialogue(&mut self, npc_id: NPCId) {
        self.active_dialogues.remove(&npc_id);
        if let Some(npc) = self.npcs.get_mut(&npc_id) {
            if let Some(dialogue_id) = npc.current_dialogue {
                npc.dialogue_history.push(dialogue_id);
            }
            npc.current_dialogue = None;
        }
        debug!("结束对话: NPC={}", npc_id);
    }
    
    // 更新NPC系统
    pub fn update(&mut self, delta_time: f32, player_position: Vec3) -> Result<(), GameError> {
        self.frame_count += 1;
        self.ai_update_timer += delta_time;
        
        // 更新活跃对话
        self.update_active_dialogues(delta_time)?;
        
        // 分批更新NPC AI
        if self.ai_update_timer >= self.ai_update_interval {
            self.update_npc_ai_batch(delta_time, player_position)?;
            self.ai_update_timer = 0.0;
        }
        
        // 更新NPC调度
        self.update_npc_schedules()?;
        
        Ok(())
    }
    
    // 查找附近的NPC
    pub fn find_npcs_near(&self, position: Vec3, radius: f32) -> Vec<NPCId> {
        self.active_npcs
            .iter()
            .filter_map(|&npc_id| {
                self.npcs.get(&npc_id).and_then(|npc| {
                    let distance = (npc.position - position).length();
                    if distance <= radius {
                        Some(npc_id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
    
    // 按类型查找NPC
    pub fn find_npcs_by_type(&self, npc_type: NPCType) -> Vec<NPCId> {
        self.npcs
            .iter()
            .filter_map(|(&npc_id, npc)| {
                if npc.npc_type == npc_type {
                    Some(npc_id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 设置NPC对话树
    pub fn set_dialogue_tree(&mut self, npc_id: NPCId, dialogue_tree: DialogueTree) -> Result<(), GameError> {
        if let Some(npc) = self.npcs.get_mut(&npc_id) {
            npc.dialogue_tree = Some(dialogue_tree);
            Ok(())
        } else {
            Err(GameError::NPC(format!("NPC不存在: {}", npc_id)))
        }
    }
    
    // 私有方法
    fn check_dialogue_conditions(
        &self,
        conditions: &[DialogueCondition],
        player_id: u64,
        npc: &NPC,
    ) -> Result<bool, GameError> {
        for condition in conditions {
            match condition {
                DialogueCondition::Relationship { min_value } => {
                    if npc.relationship_with_player < *min_value {
                        return Ok(false);
                    }
                },
                DialogueCondition::Flag { flag_name, value } => {
                    if npc.flags.get(flag_name) != Some(value) {
                        return Ok(false);
                    }
                },
                // 其他条件检查...
                _ => {
                    // 简化实现，默认通过
                }
            }
        }
        
        Ok(true)
    }
    
    fn execute_dialogue_actions(
        &self,
        actions: &[DialogueAction],
        player_id: u64,
        npc: &mut NPC,
    ) -> Result<(), GameError> {
        for action in actions {
            match action {
                DialogueAction::ChangeRelationship { amount } => {
                    npc.relationship_with_player = (npc.relationship_with_player + amount).clamp(-100, 100);
                    debug!("改变关系: NPC={} 变化={} 新值={}", npc.id, amount, npc.relationship_with_player);
                },
                DialogueAction::SetFlag { flag_name, value } => {
                    npc.flags.insert(flag_name.clone(), *value);
                    debug!("设置标志: NPC={} {}={}", npc.id, flag_name, value);
                },
                DialogueAction::StartBattle { trainer_id } => {
                    if npc.trainer_data.is_some() {
                        self.battles_initiated += 1;
                        debug!("开始战斗: NPC={}", npc.id);
                    }
                },
                // 其他动作执行...
                _ => {
                    // 简化实现
                }
            }
        }
        
        Ok(())
    }
    
    fn update_active_dialogues(&mut self, delta_time: f32) -> Result<(), GameError> {
        let mut dialogues_to_advance = Vec::new();
        
        for (&npc_id, dialogue_state) in &mut self.active_dialogues {
            if dialogue_state.auto_continue_timer > 0.0 {
                dialogue_state.auto_continue_timer -= delta_time;
                
                if dialogue_state.auto_continue_timer <= 0.0 {
                    dialogues_to_advance.push(npc_id);
                }
            }
        }
        
        // 处理自动继续的对话
        for npc_id in dialogues_to_advance {
            // 自动继续对话逻辑
        }
        
        Ok(())
    }
    
    fn update_npc_ai_batch(&mut self, delta_time: f32, player_position: Vec3) -> Result<(), GameError> {
        let start_index = self.npc_update_index;
        let end_index = (start_index + self.npcs_per_ai_update).min(self.active_npcs.len());
        
        for i in start_index..end_index {
            if let Some(&npc_id) = self.active_npcs.get(i) {
                self.update_single_npc_ai(npc_id, delta_time, player_position)?;
            }
        }
        
        self.npc_update_index = if end_index >= self.active_npcs.len() {
            0
        } else {
            end_index
        };
        
        Ok(())
    }
    
    fn update_single_npc_ai(&mut self, npc_id: NPCId, delta_time: f32, player_position: Vec3) -> Result<(), GameError> {
        if let Some(npc) = self.npcs.get_mut(&npc_id) {
            // 检查玩家距离
            let distance_to_player = (npc.position - player_position).length();
            
            // 根据AI行为更新NPC
            match &npc.ai_behavior {
                AIBehavior::Wander { radius, speed } => {
                    // 简单的随机游荡
                    if fastrand::f32() < 0.1 { // 10%概率改变方向
                        npc.facing_direction = Vec2::new(
                            (fastrand::f32() - 0.5) * 2.0,
                            (fastrand::f32() - 0.5) * 2.0,
                        ).normalize();
                    }
                    
                    let movement = Vec3::new(
                        npc.facing_direction.x * speed * delta_time,
                        0.0,
                        npc.facing_direction.y * speed * delta_time,
                    );
                    
                    npc.position += movement;
                },
                AIBehavior::Guard { area } => {
                    // 检查是否有入侵者
                    let area_distance = (npc.position - area.0).length();
                    if area_distance > area.1 {
                        // 返回守卫位置
                        let direction = (area.0 - npc.position).normalize();
                        npc.position += direction * npc.movement_pattern.speed * delta_time;
                    }
                },
                _ => {
                    // 其他AI行为
                }
            }
            
            // 更新心情
            self.update_npc_mood(npc, distance_to_player);
        }
        
        Ok(())
    }
    
    fn update_npc_mood(&self, npc: &mut NPC, distance_to_player: f32) {
        // 根据与玩家的关系和距离更新心情
        if npc.relationship_with_player > 50 {
            npc.mood = NPCMood::Friendly;
        } else if npc.relationship_with_player < -25 {
            npc.mood = NPCMood::Suspicious;
        } else {
            npc.mood = NPCMood::Neutral;
        }
        
        // 如果玩家太靠近且关系不好
        if distance_to_player < npc.interaction_radius * 0.5 && npc.relationship_with_player < 0 {
            npc.mood = NPCMood::Worried;
        }
    }
    
    fn update_npc_schedules(&mut self) -> Result<(), GameError> {
        // 简化的调度更新
        // 实际实现应该根据游戏内时间更新NPC位置和行为
        Ok(())
    }
}

impl Default for NPCMemory {
    fn default() -> Self {
        Self {
            last_interaction_time: None,
            interaction_count: 0,
            remembered_events: Vec::new(),
            known_facts: HashMap::new(),
            relationships: HashMap::new(),
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            daily_activities: HashMap::new(),
            weekly_schedule: HashMap::new(),
            special_events: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_npc_manager_creation() {
        let manager = NPCManager::new();
        assert_eq!(manager.npcs.len(), 0);
        assert_eq!(manager.active_npcs.len(), 0);
    }
    
    #[test]
    fn test_npc_creation() {
        let mut manager = NPCManager::new();
        
        let npc_id = manager.create_npc(
            "测试村民".to_string(),
            NPCType::Villager,
            Vec3::new(100.0, 0.0, 200.0),
            1,
        ).unwrap();
        
        assert!(npc_id > 0);
        assert_eq!(manager.npcs.len(), 1);
        assert_eq!(manager.active_npcs.len(), 1);
        
        let npc = manager.get_npc(npc_id).unwrap();
        assert_eq!(npc.name, "测试村民");
        assert_eq!(npc.npc_type, NPCType::Villager);
        assert_eq!(npc.position, Vec3::new(100.0, 0.0, 200.0));
    }
    
    #[test]
    fn test_npc_search() {
        let mut manager = NPCManager::new();
        
        let npc1 = manager.create_npc("村民1".to_string(), NPCType::Villager, Vec3::ZERO, 1).unwrap();
        let npc2 = manager.create_npc("商人".to_string(), NPCType::Shopkeeper, Vec3::new(50.0, 0.0, 0.0), 2).unwrap();
        let npc3 = manager.create_npc("村民2".to_string(), NPCType::Villager, Vec3::new(200.0, 0.0, 0.0), 3).unwrap();
        
        // 按距离查找
        let nearby = manager.find_npcs_near(Vec3::ZERO, 100.0);
        assert_eq!(nearby.len(), 2); // npc1 和 npc2
        
        // 按类型查找
        let villagers = manager.find_npcs_by_type(NPCType::Villager);
        assert_eq!(villagers.len(), 2); // npc1 和 npc3
        
        let shopkeepers = manager.find_npcs_by_type(NPCType::Shopkeeper);
        assert_eq!(shopkeepers.len(), 1); // npc2
    }
}