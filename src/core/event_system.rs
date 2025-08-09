// 事件系统 - 解耦组件间通信
// 开发心理：事件驱动架构是现代游戏引擎的核心，实现松耦合的组件通信
// 高性能事件分发，支持优先级、过滤器和异步处理

use crate::core::{GameError, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use log::{debug, warn};

// 事件特征
pub trait Event: Any + Send + Sync + Debug {
    fn event_type(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

// 事件监听器特征
pub trait EventListener<T: Event>: Send + Sync {
    fn handle_event(&mut self, event: &T) -> Result<()>;
}

// 事件优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Highest = 4,
}

// 事件处理器包装
pub struct EventHandler {
    pub priority: EventPriority,
    pub handler: Box<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>,
}

// 事件分发器
pub struct EventDispatcher {
    handlers: RwLock<HashMap<TypeId, Vec<EventHandler>>>,
    event_queue: Mutex<VecDeque<Box<dyn Event>>>,
    enabled: RwLock<bool>,
    stats: RwLock<EventStats>,
}

#[derive(Debug, Default)]
struct EventStats {
    events_dispatched: u64,
    events_queued: u64,
    handlers_registered: u32,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            event_queue: Mutex::new(VecDeque::new()),
            enabled: RwLock::new(true),
            stats: RwLock::new(EventStats::default()),
        }
    }

    // 注册事件监听器
    pub fn register_handler<T: Event + 'static, F>(&self, handler: F, priority: EventPriority) -> Result<()>
    where
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        
        let wrapped_handler = Box::new(move |event: &dyn Event| -> Result<()> {
            if let Some(typed_event) = event.as_any().downcast_ref::<T>() {
                handler(typed_event)
            } else {
                Err(GameError::InvalidInput("事件类型不匹配".to_string()))
            }
        });

        let event_handler = EventHandler {
            priority,
            handler: wrapped_handler,
        };

        let mut handlers = self.handlers.write().unwrap();
        let handler_list = handlers.entry(type_id).or_insert_with(Vec::new);
        handler_list.push(event_handler);
        
        // 按优先级排序（高优先级在前）
        handler_list.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 更新统计
        let mut stats = self.stats.write().unwrap();
        stats.handlers_registered += 1;

        debug!("注册事件处理器: {}", std::any::type_name::<T>());
        Ok(())
    }

    // 立即分发事件
    pub fn dispatch<T: Event + 'static>(&self, event: T) -> Result<()> {
        if !*self.enabled.read().unwrap() {
            return Ok(());
        }

        let type_id = TypeId::of::<T>();
        let handlers = self.handlers.read().unwrap();
        
        if let Some(handler_list) = handlers.get(&type_id) {
            debug!("分发事件: {} 到 {} 个处理器", event.event_type(), handler_list.len());
            
            for handler in handler_list {
                if let Err(e) = (handler.handler)(&event) {
                    warn!("事件处理器执行失败: {}", e);
                }
            }
        }

        // 更新统计
        let mut stats = self.stats.write().unwrap();
        stats.events_dispatched += 1;

        Ok(())
    }

    // 将事件加入队列，延迟处理
    pub fn queue_event<T: Event + 'static>(&self, event: T) -> Result<()> {
        if !*self.enabled.read().unwrap() {
            return Ok(());
        }

        let mut queue = self.event_queue.lock().unwrap();
        queue.push_back(Box::new(event));

        // 更新统计
        let mut stats = self.stats.write().unwrap();
        stats.events_queued += 1;

        Ok(())
    }

    // 处理队列中的所有事件
    pub fn process_queued_events(&self) -> Result<()> {
        let mut queue = self.event_queue.lock().unwrap();
        let events_to_process: Vec<_> = queue.drain(..).collect();
        drop(queue);

        for event in events_to_process {
            self.dispatch_boxed_event(event)?;
        }

        Ok(())
    }

    // 处理装箱的事件
    fn dispatch_boxed_event(&self, event: Box<dyn Event>) -> Result<()> {
        if !*self.enabled.read().unwrap() {
            return Ok(());
        }

        let handlers = self.handlers.read().unwrap();
        let type_id = event.as_any().type_id();
        
        if let Some(handler_list) = handlers.get(&type_id) {
            debug!("分发装箱事件: {} 到 {} 个处理器", event.event_type(), handler_list.len());
            
            for handler in handler_list {
                if let Err(e) = (handler.handler)(event.as_ref()) {
                    warn!("事件处理器执行失败: {}", e);
                }
            }
        }

        Ok(())
    }

    // 清空事件队列
    pub fn clear_queue(&self) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.clear();
        debug!("事件队列已清空");
    }

    // 启用/禁用事件系统
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().unwrap() = enabled;
        debug!("事件系统 {}", if enabled { "已启用" } else { "已禁用" });
    }

    // 获取统计信息
    pub fn get_stats(&self) -> EventStats {
        self.stats.read().unwrap().clone()
    }

    // 清除所有处理器
    pub fn clear_handlers(&self) {
        let mut handlers = self.handlers.write().unwrap();
        handlers.clear();
        
        let mut stats = self.stats.write().unwrap();
        stats.handlers_registered = 0;
        
        debug!("所有事件处理器已清除");
    }
}

// 全局事件系统
static mut EVENT_SYSTEM: Option<EventDispatcher> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub struct EventSystem;

impl EventSystem {
    pub fn init() -> Result<()> {
        unsafe {
            INIT.call_once(|| {
                EVENT_SYSTEM = Some(EventDispatcher::new());
            });
        }
        Ok(())
    }

    pub fn instance() -> &'static EventDispatcher {
        unsafe {
            EVENT_SYSTEM.as_ref().expect("事件系统未初始化")
        }
    }

    pub fn cleanup() {
        unsafe {
            if let Some(ref system) = EVENT_SYSTEM {
                system.clear_handlers();
                system.clear_queue();
            }
        }
    }

    // 便捷方法
    pub fn dispatch<T: Event + 'static>(event: T) -> Result<()> {
        Self::instance().dispatch(event)
    }

    pub fn queue<T: Event + 'static>(event: T) -> Result<()> {
        Self::instance().queue_event(event)
    }

    pub fn register<T: Event + 'static, F>(handler: F, priority: EventPriority) -> Result<()>
    where
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        Self::instance().register_handler(handler, priority)
    }

    pub fn process_queue() -> Result<()> {
        Self::instance().process_queued_events()
    }
}

// 常用游戏事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameExitEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleStartEvent {
    pub player_pokemon: Vec<String>,
    pub opponent_pokemon: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonCaughtEvent {
    pub pokemon_name: String,
    pub level: u8,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeEvent {
    pub from_state: String,
    pub to_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub input_type: String,
    pub key_code: Option<u32>,
    pub mouse_pos: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEvent {
    pub event_type: AudioEventType,
    pub sound_name: String,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEventType {
    Play,
    Stop,
    Pause,
    Resume,
    VolumeChange,
}

// 实现Event特征
impl Event for GameStartEvent {
    fn event_type(&self) -> &'static str { "GameStart" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for GameExitEvent {
    fn event_type(&self) -> &'static str { "GameExit" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for BattleStartEvent {
    fn event_type(&self) -> &'static str { "BattleStart" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for PokemonCaughtEvent {
    fn event_type(&self) -> &'static str { "PokemonCaught" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for StateChangeEvent {
    fn event_type(&self) -> &'static str { "StateChange" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for InputEvent {
    fn event_type(&self) -> &'static str { "Input" }
    fn as_any(&self) -> &dyn Any { self }
}

impl Event for AudioEvent {
    fn event_type(&self) -> &'static str { "Audio" }
    fn as_any(&self) -> &dyn Any { self }
}

// 事件过滤器
pub trait EventFilter<T: Event>: Send + Sync {
    fn should_handle(&self, event: &T) -> bool;
}

pub struct AlwaysFilter;
impl<T: Event> EventFilter<T> for AlwaysFilter {
    fn should_handle(&self, _event: &T) -> bool { true }
}

// 克隆统计信息
impl Clone for EventStats {
    fn clone(&self) -> Self {
        Self {
            events_dispatched: self.events_dispatched,
            events_queued: self.events_queued,
            handlers_registered: self.handlers_registered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestEvent {
        message: String,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str { "Test" }
        fn as_any(&self) -> &dyn Any { self }
    }

    #[test]
    fn test_event_dispatch() {
        let dispatcher = EventDispatcher::new();
        let mut received = false;
        
        dispatcher.register_handler(
            |event: &TestEvent| {
                assert_eq!(event.message, "test");
                Ok(())
            },
            EventPriority::Normal
        ).unwrap();

        let test_event = TestEvent {
            message: "test".to_string(),
        };

        dispatcher.dispatch(test_event).unwrap();
    }

    #[test]
    fn test_event_queue() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        dispatcher.register_handler(
            move |_: &TestEvent| {
                *counter_clone.lock().unwrap() += 1;
                Ok(())
            },
            EventPriority::Normal
        ).unwrap();

        // 队列多个事件
        for i in 0..5 {
            dispatcher.queue_event(TestEvent {
                message: format!("test_{}", i),
            }).unwrap();
        }

        // 处理队列
        dispatcher.process_queued_events().unwrap();

        assert_eq!(*counter.lock().unwrap(), 5);
    }

    #[test]
    fn test_priority_order() {
        let dispatcher = EventDispatcher::new();
        let order = Arc::new(Mutex::new(Vec::new()));
        
        // 注册不同优先级的处理器
        let order1 = order.clone();
        dispatcher.register_handler(
            move |_: &TestEvent| {
                order1.lock().unwrap().push(1);
                Ok(())
            },
            EventPriority::Low
        ).unwrap();

        let order2 = order.clone();
        dispatcher.register_handler(
            move |_: &TestEvent| {
                order2.lock().unwrap().push(2);
                Ok(())
            },
            EventPriority::High
        ).unwrap();

        let order3 = order.clone();
        dispatcher.register_handler(
            move |_: &TestEvent| {
                order3.lock().unwrap().push(3);
                Ok(())
            },
            EventPriority::Normal
        ).unwrap();

        dispatcher.dispatch(TestEvent {
            message: "priority_test".to_string(),
        }).unwrap();

        let result = order.lock().unwrap();
        assert_eq!(*result, vec![2, 3, 1]); // High, Normal, Low
    }
}