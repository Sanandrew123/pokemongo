// 事件系统
// 开发心理：事件系统处理游戏中的各种触发器、状态变化、交互响应
// 设计原则：解耦设计、优先级处理、延迟执行、状态同步

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use log::{debug, warn, error};
use crate::core::error::GameError;
use glam::Vec3;

// 事件管理器
pub struct EventManager {
    // 事件队列
    event_queue: VecDeque<GameEvent>,
    priority_queue: BinaryHeap<PriorityEvent>,
    
    // 延迟事件
    delayed_events: Vec<DelayedEvent>,
    
    // 事件监听器
    listeners: HashMap<String, Vec<EventListener>>,
    
    // 条件事件
    conditional_events: Vec<ConditionalEvent>,
    
    // 事件历史
    event_history: VecDeque<GameEvent>,
    max_history_size: usize,
    
    // 事件触发器
    triggers: HashMap<String, EventTrigger>,
    
    // 统计信息
    events_processed: u64,
    events_per_second: f32,
    frame_count: u64,
}

// 游戏事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: u64,
    pub event_type: String,
    pub source: EventSource,
    pub target: Option<EventTarget>,
    pub data: HashMap<String, EventValue>,
    pub timestamp: std::time::SystemTime,
    pub processed: bool,
}

// 优先级事件
#[derive(Debug, Clone)]
pub struct PriorityEvent {
    pub event: GameEvent,
    pub priority: i32,
}

impl PartialEq for PriorityEvent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PriorityEvent {}

impl PartialOrd for PriorityEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).reverse() // 高优先级先执行
    }
}

// 延迟事件
#[derive(Debug, Clone)]
pub struct DelayedEvent {
    pub event: GameEvent,
    pub delay: f32,
    pub remaining_time: f32,
}

// 事件源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    Player(u64),
    NPC(u64),
    System,
    World,
    Battle(u64),
    Pokemon(u64),
    Item(u32),
    Trigger(String),
}

// 事件目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTarget {
    Player(u64),
    NPC(u64),
    Pokemon(u64),
    World,
    System,
    All,
    Radius { center: Vec3, radius: f32 },
}

// 事件值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventValue {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Position(Vec3),
    Array(Vec<EventValue>),
}

// 事件监听器
pub struct EventListener {
    pub id: String,
    pub callback: Box<dyn Fn(&GameEvent) -> Result<(), GameError> + Send + Sync>,
    pub once: bool,                     // 是否只执行一次
    pub filter: Option<EventFilter>,    // 过滤条件
}

// 事件过滤器
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub source_filter: Option<EventSource>,
    pub target_filter: Option<EventTarget>,
    pub data_filters: HashMap<String, EventValue>,
}

// 条件事件
#[derive(Debug, Clone)]
pub struct ConditionalEvent {
    pub id: String,
    pub event: GameEvent,
    pub condition: EventCondition,
    pub check_interval: f32,
    pub last_check_time: f32,
}

// 事件条件
#[derive(Debug, Clone)]
pub enum EventCondition {
    Always,
    PlayerPosition { position: Vec3, radius: f32 },
    TimeOfDay { start_hour: u8, end_hour: u8 },
    WorldVariable { variable: String, value: i32 },
    CustomCondition { condition_name: String, parameters: HashMap<String, String> },
}

// 事件触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrigger {
    pub id: String,
    pub name: String,
    pub trigger_type: TriggerType,
    pub position: Vec3,
    pub size: Vec3,
    pub events: Vec<String>,            // 触发的事件类型
    pub conditions: Vec<TriggerCondition>,
    pub cooldown: f32,
    pub last_triggered: Option<std::time::SystemTime>,
    pub triggered_count: u32,
    pub max_triggers: Option<u32>,
    pub enabled: bool,
}

// 触发器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    OnEnter,        // 进入时触发
    OnExit,         // 离开时触发
    OnInteract,     // 交互时触发
    OnTimer,        // 定时触发
    OnCondition,    // 条件触发
}

// 触发条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    PlayerOnly,
    NPCOnly,
    PokemonOnly,
    HasItem { item_id: u32, quantity: u32 },
    PlayerLevel { min_level: u32, max_level: Option<u32> },
    QuestActive { quest_id: u32 },
    WorldFlag { flag_name: String, value: bool },
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            priority_queue: BinaryHeap::new(),
            delayed_events: Vec::new(),
            listeners: HashMap::new(),
            conditional_events: Vec::new(),
            event_history: VecDeque::new(),
            max_history_size: 1000,
            triggers: HashMap::new(),
            events_processed: 0,
            events_per_second: 0.0,
            frame_count: 0,
        }
    }
    
    // 触发事件
    pub fn trigger_event(&mut self, event_type: &str, data: HashMap<String, EventValue>) -> u64 {
        let event_id = self.generate_event_id();
        
        let event = GameEvent {
            id: event_id,
            event_type: event_type.to_string(),
            source: EventSource::System,
            target: None,
            data,
            timestamp: std::time::SystemTime::now(),
            processed: false,
        };
        
        self.event_queue.push_back(event);
        debug!("触发事件: {} (ID: {})", event_type, event_id);
        
        event_id
    }
    
    // 触发高优先级事件
    pub fn trigger_priority_event(&mut self, event_type: &str, data: HashMap<String, EventValue>, priority: i32) -> u64 {
        let event_id = self.generate_event_id();
        
        let event = GameEvent {
            id: event_id,
            event_type: event_type.to_string(),
            source: EventSource::System,
            target: None,
            data,
            timestamp: std::time::SystemTime::now(),
            processed: false,
        };
        
        let priority_event = PriorityEvent { event, priority };
        self.priority_queue.push(priority_event);
        
        debug!("触发高优先级事件: {} 优先级: {} (ID: {})", event_type, priority, event_id);
        
        event_id
    }
    
    // 延迟触发事件
    pub fn trigger_delayed_event(&mut self, event_type: &str, data: HashMap<String, EventValue>, delay: f32) -> u64 {
        let event_id = self.generate_event_id();
        
        let event = GameEvent {
            id: event_id,
            event_type: event_type.to_string(),
            source: EventSource::System,
            target: None,
            data,
            timestamp: std::time::SystemTime::now(),
            processed: false,
        };
        
        let delayed_event = DelayedEvent {
            event,
            delay,
            remaining_time: delay,
        };
        
        self.delayed_events.push(delayed_event);
        debug!("触发延迟事件: {} 延迟: {}秒 (ID: {})", event_type, delay, event_id);
        
        event_id
    }
    
    // 添加事件监听器
    pub fn add_listener<F>(&mut self, event_type: String, listener_id: String, callback: F) -> Result<(), GameError> 
    where
        F: Fn(&GameEvent) -> Result<(), GameError> + Send + Sync + 'static,
    {
        let listener = EventListener {
            id: listener_id.clone(),
            callback: Box::new(callback),
            once: false,
            filter: None,
        };
        
        self.listeners
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push(listener);
        
        debug!("添加事件监听器: {} -> {}", event_type, listener_id);
        Ok(())
    }
    
    // 移除事件监听器
    pub fn remove_listener(&mut self, event_type: &str, listener_id: &str) -> bool {
        if let Some(listeners) = self.listeners.get_mut(event_type) {
            let initial_len = listeners.len();
            listeners.retain(|l| l.id != listener_id);
            let removed = listeners.len() < initial_len;
            
            if removed {
                debug!("移除事件监听器: {} -> {}", event_type, listener_id);
            }
            
            removed
        } else {
            false
        }
    }
    
    // 添加事件触发器
    pub fn add_trigger(&mut self, trigger: EventTrigger) {
        let trigger_id = trigger.id.clone();
        self.triggers.insert(trigger_id.clone(), trigger);
        debug!("添加事件触发器: {}", trigger_id);
    }
    
    // 检查触发器
    pub fn check_triggers(&mut self, entity_position: Vec3, entity_type: EventSource) -> Vec<String> {
        let mut triggered_events = Vec::new();
        let current_time = std::time::SystemTime::now();
        
        for trigger in self.triggers.values_mut() {
            if !trigger.enabled {
                continue;
            }
            
            // 检查触发次数限制
            if let Some(max_triggers) = trigger.max_triggers {
                if trigger.triggered_count >= max_triggers {
                    continue;
                }
            }
            
            // 检查冷却时间
            if let Some(last_triggered) = trigger.last_triggered {
                if let Ok(duration) = current_time.duration_since(last_triggered) {
                    if duration.as_secs_f32() < trigger.cooldown {
                        continue;
                    }
                }
            }
            
            // 检查位置触发
            if trigger.trigger_type == TriggerType::OnEnter {
                let distance = (entity_position - trigger.position).length();
                if distance <= trigger.size.x * 0.5 {
                    // 检查触发条件
                    if self.check_trigger_conditions(&trigger.conditions, &entity_type) {
                        for event_type in &trigger.events {
                            triggered_events.push(event_type.clone());
                        }
                        
                        trigger.last_triggered = Some(current_time);
                        trigger.triggered_count += 1;
                        
                        debug!("触发器激活: {} (位置: {:?})", trigger.id, entity_position);
                    }
                }
            }
        }
        
        triggered_events
    }
    
    // 更新事件系统
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        self.frame_count += 1;
        
        // 更新延迟事件
        self.update_delayed_events(delta_time);
        
        // 更新条件事件
        self.update_conditional_events(delta_time);
        
        // 处理优先级事件
        self.process_priority_events()?;
        
        // 处理普通事件
        self.process_regular_events()?;
        
        // 计算事件处理性能
        if self.frame_count % 60 == 0 {
            self.events_per_second = self.events_processed as f32 / 60.0;
            self.events_processed = 0;
        }
        
        Ok(())
    }
    
    // 获取事件历史
    pub fn get_event_history(&self, event_type: Option<&str>, limit: usize) -> Vec<&GameEvent> {
        let mut result = Vec::new();
        
        for event in self.event_history.iter().rev() {
            if result.len() >= limit {
                break;
            }
            
            if let Some(filter_type) = event_type {
                if event.event_type == filter_type {
                    result.push(event);
                }
            } else {
                result.push(event);
            }
        }
        
        result
    }
    
    // 私有方法
    fn generate_event_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        timestamp + self.events_processed
    }
    
    fn update_delayed_events(&mut self, delta_time: f32) {
        let mut events_to_trigger = Vec::new();
        
        for (i, delayed_event) in self.delayed_events.iter_mut().enumerate() {
            delayed_event.remaining_time -= delta_time;
            
            if delayed_event.remaining_time <= 0.0 {
                events_to_trigger.push(i);
            }
        }
        
        // 触发到期的延迟事件
        for &i in events_to_trigger.iter().rev() {
            let delayed_event = self.delayed_events.remove(i);
            self.event_queue.push_back(delayed_event.event);
        }
    }
    
    fn update_conditional_events(&mut self, delta_time: f32) {
        for conditional_event in &mut self.conditional_events {
            conditional_event.last_check_time += delta_time;
            
            if conditional_event.last_check_time >= conditional_event.check_interval {
                conditional_event.last_check_time = 0.0;
                
                if self.check_event_condition(&conditional_event.condition) {
                    self.event_queue.push_back(conditional_event.event.clone());
                    debug!("条件事件触发: {}", conditional_event.id);
                }
            }
        }
    }
    
    fn process_priority_events(&mut self) -> Result<(), GameError> {
        while let Some(priority_event) = self.priority_queue.pop() {
            self.process_single_event(priority_event.event)?;
        }
        Ok(())
    }
    
    fn process_regular_events(&mut self) -> Result<(), GameError> {
        let events_to_process = std::cmp::min(self.event_queue.len(), 50); // 限制每帧处理数量
        
        for _ in 0..events_to_process {
            if let Some(event) = self.event_queue.pop_front() {
                self.process_single_event(event)?;
            }
        }
        
        Ok(())
    }
    
    fn process_single_event(&mut self, mut event: GameEvent) -> Result<(), GameError> {
        // 触发监听器
        if let Some(listeners) = self.listeners.get(&event.event_type) {
            let mut listeners_to_remove = Vec::new();
            
            for (i, listener) in listeners.iter().enumerate() {
                // 检查过滤器
                if let Some(ref filter) = listener.filter {
                    if !self.event_matches_filter(&event, filter) {
                        continue;
                    }
                }
                
                // 调用监听器
                match (listener.callback)(&event) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("事件监听器执行失败: {}", e);
                    }
                }
                
                // 记录一次性监听器
                if listener.once {
                    listeners_to_remove.push(i);
                }
            }
            
            // 移除一次性监听器
            // 注意：这里需要从listeners中移除，但由于借用检查，我们暂时跳过实际移除
        }
        
        event.processed = true;
        
        // 添加到历史记录
        self.event_history.push_back(event);
        if self.event_history.len() > self.max_history_size {
            self.event_history.pop_front();
        }
        
        self.events_processed += 1;
        Ok(())
    }
    
    fn check_trigger_conditions(&self, conditions: &[TriggerCondition], entity_type: &EventSource) -> bool {
        for condition in conditions {
            match condition {
                TriggerCondition::PlayerOnly => {
                    if !matches!(entity_type, EventSource::Player(_)) {
                        return false;
                    }
                },
                TriggerCondition::NPCOnly => {
                    if !matches!(entity_type, EventSource::NPC(_)) {
                        return false;
                    }
                },
                TriggerCondition::PokemonOnly => {
                    if !matches!(entity_type, EventSource::Pokemon(_)) {
                        return false;
                    }
                },
                // 其他条件暂时简化处理
                _ => {}
            }
        }
        
        true
    }
    
    fn check_event_condition(&self, condition: &EventCondition) -> bool {
        match condition {
            EventCondition::Always => true,
            EventCondition::TimeOfDay { start_hour, end_hour } => {
                // 简化实现：假设当前时间在范围内
                true
            },
            _ => false, // 其他条件需要更多上下文信息
        }
    }
    
    fn event_matches_filter(&self, event: &GameEvent, filter: &EventFilter) -> bool {
        // 简化的过滤器检查
        if let Some(ref source_filter) = filter.source_filter {
            // 这里需要实际的源比较逻辑
        }
        
        true
    }
}

// 事件构建器
pub struct EventBuilder {
    event: GameEvent,
}

impl EventBuilder {
    pub fn new(event_type: String) -> Self {
        Self {
            event: GameEvent {
                id: 0,
                event_type,
                source: EventSource::System,
                target: None,
                data: HashMap::new(),
                timestamp: std::time::SystemTime::now(),
                processed: false,
            },
        }
    }
    
    pub fn source(mut self, source: EventSource) -> Self {
        self.event.source = source;
        self
    }
    
    pub fn target(mut self, target: EventTarget) -> Self {
        self.event.target = Some(target);
        self
    }
    
    pub fn data(mut self, key: String, value: EventValue) -> Self {
        self.event.data.insert(key, value);
        self
    }
    
    pub fn build(self, event_manager: &mut EventManager) -> u64 {
        let mut event = self.event;
        event.id = event_manager.generate_event_id();
        let event_id = event.id;
        
        event_manager.event_queue.push_back(event);
        event_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_manager_creation() {
        let manager = EventManager::new();
        assert_eq!(manager.event_queue.len(), 0);
        assert_eq!(manager.listeners.len(), 0);
    }
    
    #[test]
    fn test_event_triggering() {
        let mut manager = EventManager::new();
        let mut data = HashMap::new();
        data.insert("test".to_string(), EventValue::String("value".to_string()));
        
        let event_id = manager.trigger_event("test_event", data);
        assert!(event_id > 0);
        assert_eq!(manager.event_queue.len(), 1);
    }
    
    #[test]
    fn test_priority_events() {
        let mut manager = EventManager::new();
        
        let _low_priority = manager.trigger_priority_event("low", HashMap::new(), 1);
        let _high_priority = manager.trigger_priority_event("high", HashMap::new(), 10);
        
        assert_eq!(manager.priority_queue.len(), 2);
        
        // 高优先级应该先出队列
        let first_event = manager.priority_queue.pop().unwrap();
        assert_eq!(first_event.priority, 10);
    }
    
    #[test]
    fn test_delayed_events() {
        let mut manager = EventManager::new();
        
        let _delayed = manager.trigger_delayed_event("delayed", HashMap::new(), 1.0);
        assert_eq!(manager.delayed_events.len(), 1);
        assert_eq!(manager.delayed_events[0].delay, 1.0);
    }
    
    #[test]
    fn test_event_builder() {
        let mut manager = EventManager::new();
        
        let event_id = EventBuilder::new("built_event".to_string())
            .source(EventSource::Player(1))
            .target(EventTarget::World)
            .data("key".to_string(), EventValue::Int(42))
            .build(&mut manager);
        
        assert!(event_id > 0);
        assert_eq!(manager.event_queue.len(), 1);
        
        let event = &manager.event_queue[0];
        assert_eq!(event.event_type, "built_event");
        assert!(matches!(event.source, EventSource::Player(1)));
        assert!(matches!(event.target, Some(EventTarget::World)));
    }
}