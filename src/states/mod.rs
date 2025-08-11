// 游戏状态系统
// 开发心理：状态机是游戏架构核心，需要清晰状态转换、资源管理、事件处理
// 设计原则：状态封装、转换控制、资源生命周期、栈式管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::Renderer2D;
use crate::graphics::ui::UIManager;
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
use glam::Vec2;

// Bevy States枚举 - 用于Bevy状态管理
use bevy::prelude::*;

pub mod menu;
pub mod battle;
pub mod overworld;
pub mod settings;
pub mod loading;

// Bevy States枚举 - 符合Bevy状态管理要求
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,        // 加载状态（默认）
    MainMenu,       // 主菜单
    GameMenu,       // 游戏内菜单
    Overworld,      // 大地图
    Battle,         // 战斗
    Inventory,      // 背包
    Pokemon,        // Pokemon管理
    Settings,       // 设置
    Pause,          // 暂停
    Credits,        // 制作人员
}

// 状态ID类型
pub type StateId = u32;

// 游戏状态类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameStateType {
    Loading,        // 加载状态
    MainMenu,       // 主菜单
    GameMenu,       // 游戏内菜单
    Overworld,      // 大地图
    Battle,         // 战斗
    Inventory,      // 背包
    Pokemon,        // Pokemon管理
    Settings,       // 设置
    Pause,          // 暂停
    Credits,        // 制作人员
    Custom(String), // 自定义状态
}

// 状态转换类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransition {
    None,           // 无变化
    Push(GameStateType),    // 推入新状态
    Pop,            // 弹出当前状态
    Replace(GameStateType), // 替换当前状态
    Clear,          // 清空状态栈
    Quit,           // 退出游戏
}

// 游戏状态接口
pub trait GameState: Send {
    fn get_type(&self) -> GameStateType;
    fn get_name(&self) -> &str;
    
    // 生命周期
    fn enter(&mut self, previous_state: Option<GameStateType>) -> Result<(), GameError>;
    fn exit(&mut self, next_state: Option<GameStateType>) -> Result<(), GameError>;
    fn pause(&mut self) -> Result<(), GameError>;
    fn resume(&mut self) -> Result<(), GameError>;
    
    // 更新循环
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError>;
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError>;
    
    // 事件处理
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError>;
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError>;
    fn handle_gamepad_event(&mut self, event: &GamepadEvent) -> Result<bool, GameError>;
    
    // 资源管理
    fn load_resources(&mut self) -> Result<(), GameError> { Ok(()) }
    fn unload_resources(&mut self) -> Result<(), GameError> { Ok(()) }
    
    // UI管理
    fn get_ui_manager(&mut self) -> Option<&mut UIManager> { None }
    
    // 是否透明(下层状态是否继续渲染)
    fn is_transparent(&self) -> bool { false }
    
    // 是否阻塞输入(下层状态是否接收输入)
    fn blocks_input(&self) -> bool { true }
}

// 状态数据
#[derive(Debug)]
pub struct StateInfo {
    pub id: StateId,
    pub state_type: GameStateType,
    pub state: Box<dyn GameState>,
    pub paused: bool,
    pub transparent: bool,
    pub blocks_input: bool,
    pub enter_time: std::time::Instant,
    pub update_count: u64,
}

// 状态管理器
pub struct StateManager {
    // 状态栈
    state_stack: Vec<StateInfo>,
    next_state_id: StateId,
    
    // 待处理的转换
    pending_transitions: Vec<StateTransition>,
    
    // 状态工厂
    state_factories: HashMap<GameStateType, Box<dyn Fn() -> Box<dyn GameState> + Send>>,
    
    // 配置
    max_stack_depth: usize,
    
    // 统计信息
    total_states_created: u64,
    state_transition_count: u64,
    current_frame: u64,
    
    // 调试
    debug_mode: bool,
    log_transitions: bool,
}

impl StateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            state_stack: Vec::new(),
            next_state_id: 1,
            pending_transitions: Vec::new(),
            state_factories: HashMap::new(),
            max_stack_depth: 10,
            total_states_created: 0,
            state_transition_count: 0,
            current_frame: 0,
            debug_mode: false,
            log_transitions: true,
        };
        
        manager.register_default_states();
        manager
    }
    
    // 注册状态工厂
    pub fn register_state_factory<F>(&mut self, state_type: GameStateType, factory: F)
    where
        F: Fn() -> Box<dyn GameState> + Send + 'static,
    {
        self.state_factories.insert(state_type, Box::new(factory));
        debug!("注册状态工厂: {:?}", state_type);
    }
    
    // 推入状态
    pub fn push_state(&mut self, state_type: GameStateType) -> Result<StateId, GameError> {
        if self.state_stack.len() >= self.max_stack_depth {
            return Err(GameError::State(format!("状态栈深度超过限制: {}", self.max_stack_depth)));
        }
        
        // 暂停当前状态
        if let Some(current_state) = self.state_stack.last_mut() {
            current_state.state.pause()?;
            current_state.paused = true;
        }
        
        // 创建新状态
        let state_id = self.create_state(state_type)?;
        
        if self.log_transitions {
            debug!("推入状态: {:?} ID={} 栈深度={}", 
                state_type, state_id, self.state_stack.len());
        }
        
        Ok(state_id)
    }
    
    // 弹出状态
    pub fn pop_state(&mut self) -> Result<Option<GameStateType>, GameError> {
        if let Some(mut state_info) = self.state_stack.pop() {
            let state_type = state_info.state_type;
            let next_state_type = self.get_current_state_type();
            
            // 退出状态
            state_info.state.exit(next_state_type)?;
            state_info.state.unload_resources()?;
            
            // 恢复前一个状态
            if let Some(current_state) = self.state_stack.last_mut() {
                current_state.state.resume()?;
                current_state.paused = false;
            }
            
            self.state_transition_count += 1;
            
            if self.log_transitions {
                debug!("弹出状态: {:?} 栈深度={}", state_type, self.state_stack.len());
            }
            
            Ok(Some(state_type))
        } else {
            Ok(None)
        }
    }
    
    // 替换状态
    pub fn replace_state(&mut self, state_type: GameStateType) -> Result<StateId, GameError> {
        // 先弹出当前状态
        self.pop_state()?;
        
        // 然后推入新状态
        self.push_state(state_type)
    }
    
    // 清空状态栈
    pub fn clear_states(&mut self) -> Result<(), GameError> {
        while !self.state_stack.is_empty() {
            self.pop_state()?;
        }
        
        if self.log_transitions {
            debug!("清空状态栈");
        }
        
        Ok(())
    }
    
    // 获取当前状态
    pub fn get_current_state(&self) -> Option<&StateInfo> {
        self.state_stack.last()
    }
    
    // 获取当前状态类型
    pub fn get_current_state_type(&self) -> Option<GameStateType> {
        self.state_stack.last().map(|s| s.state_type)
    }
    
    // 检查状态是否存在
    pub fn has_state(&self, state_type: GameStateType) -> bool {
        self.state_stack.iter().any(|s| s.state_type == state_type)
    }
    
    // 获取状态栈深度
    pub fn get_stack_depth(&self) -> usize {
        self.state_stack.len()
    }
    
    // 处理状态转换
    pub fn handle_transition(&mut self, transition: StateTransition) -> Result<(), GameError> {
        match transition {
            StateTransition::None => {},
            StateTransition::Push(state_type) => {
                self.push_state(state_type)?;
            },
            StateTransition::Pop => {
                self.pop_state()?;
            },
            StateTransition::Replace(state_type) => {
                self.replace_state(state_type)?;
            },
            StateTransition::Clear => {
                self.clear_states()?;
            },
            StateTransition::Quit => {
                self.clear_states()?;
                // 在实际实现中，这里应该设置退出标志
                debug!("请求退出游戏");
            },
        }
        
        Ok(())
    }
    
    // 更新状态管理器
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        self.current_frame += 1;
        
        // 处理待处理的转换
        let transitions = std::mem::take(&mut self.pending_transitions);
        for transition in transitions {
            self.handle_transition(transition)?;
        }
        
        // 更新活跃状态
        if let Some(current_state) = self.state_stack.last_mut() {
            if !current_state.paused {
                let transition = current_state.state.update(delta_time)?;
                current_state.update_count += 1;
                
                // 处理状态转换
                if transition != StateTransition::None {
                    self.handle_transition(transition)?;
                }
            }
        }
        
        Ok(())
    }
    
    // 渲染状态
    pub fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 找到第一个不透明状态的索引
        let mut start_index = 0;
        for (i, state_info) in self.state_stack.iter().enumerate().rev() {
            if !state_info.transparent {
                start_index = i;
                break;
            }
        }
        
        // 从该索引开始渲染所有状态
        for state_info in &mut self.state_stack[start_index..] {
            state_info.state.render(renderer)?;
        }
        
        Ok(())
    }
    
    // 处理鼠标事件
    pub fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError> {
        // 从顶层状态开始处理
        for state_info in self.state_stack.iter_mut().rev() {
            if !state_info.paused {
                let handled = state_info.state.handle_mouse_event(event)?;
                if handled && state_info.blocks_input {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    // 处理键盘事件
    pub fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        // 从顶层状态开始处理
        for state_info in self.state_stack.iter_mut().rev() {
            if !state_info.paused {
                let handled = state_info.state.handle_keyboard_event(key, pressed)?;
                if handled && state_info.blocks_input {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    // 处理手柄事件
    pub fn handle_gamepad_event(&mut self, event: &GamepadEvent) -> Result<bool, GameError> {
        // 从顶层状态开始处理
        for state_info in self.state_stack.iter_mut().rev() {
            if !state_info.paused {
                let handled = state_info.state.handle_gamepad_event(event)?;
                if handled && state_info.blocks_input {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    // 获取统计信息
    pub fn get_stats(&self) -> StateManagerStats {
        StateManagerStats {
            active_states: self.state_stack.len(),
            stack_depth: self.state_stack.len(),
            total_states_created: self.total_states_created,
            state_transitions: self.state_transition_count,
            current_frame: self.current_frame,
            current_state: self.get_current_state_type(),
        }
    }
    
    // 设置调试模式
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
        debug!("状态管理器调试模式: {}", enabled);
    }
    
    // 私有方法
    fn create_state(&mut self, state_type: GameStateType) -> Result<StateId, GameError> {
        let factory = self.state_factories.get(&state_type)
            .ok_or_else(|| GameError::State(format!("未注册的状态类型: {:?}", state_type)))?;
        
        let mut state = factory();
        let previous_state_type = self.get_current_state_type();
        
        // 加载资源并进入状态
        state.load_resources()?;
        state.enter(previous_state_type)?;
        
        let state_id = self.next_state_id;
        self.next_state_id += 1;
        
        let state_info = StateInfo {
            id: state_id,
            state_type,
            state,
            paused: false,
            transparent: false, // 这里应该查询状态的透明性
            blocks_input: true, // 这里应该查询状态的输入阻塞性
            enter_time: std::time::Instant::now(),
            update_count: 0,
        };
        
        self.state_stack.push(state_info);
        self.total_states_created += 1;
        self.state_transition_count += 1;
        
        Ok(state_id)
    }
    
    fn register_default_states(&mut self) {
        // 注册默认状态工厂
        self.register_state_factory(GameStateType::Loading, || {
            Box::new(loading::LoadingState::new())
        });
        
        self.register_state_factory(GameStateType::MainMenu, || {
            Box::new(menu::MainMenuState::new())
        });
        
        self.register_state_factory(GameStateType::Settings, || {
            Box::new(settings::SettingsState::new())
        });
        
        self.register_state_factory(GameStateType::Battle, || {
            Box::new(battle::BattleState::new())
        });
        
        self.register_state_factory(GameStateType::Overworld, || {
            Box::new(overworld::OverworldState::new())
        });
    }
}

// 统计信息
#[derive(Debug, Clone)]
pub struct StateManagerStats {
    pub active_states: usize,
    pub stack_depth: usize,
    pub total_states_created: u64,
    pub state_transitions: u64,
    pub current_frame: u64,
    pub current_state: Option<GameStateType>,
}

// 便利宏
#[macro_export]
macro_rules! define_state {
    ($name:ident, $state_type:expr) => {
        pub struct $name {
            name: String,
        }
        
        impl $name {
            pub fn new() -> Self {
                Self {
                    name: stringify!($name).to_string(),
                }
            }
        }
        
        impl GameState for $name {
            fn get_type(&self) -> GameStateType {
                $state_type
            }
            
            fn get_name(&self) -> &str {
                &self.name
            }
            
            fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
                debug!("进入状态: {}", self.name);
                Ok(())
            }
            
            fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
                debug!("退出状态: {}", self.name);
                Ok(())
            }
            
            fn pause(&mut self) -> Result<(), GameError> {
                debug!("暂停状态: {}", self.name);
                Ok(())
            }
            
            fn resume(&mut self) -> Result<(), GameError> {
                debug!("恢复状态: {}", self.name);
                Ok(())
            }
            
            fn update(&mut self, _delta_time: f32) -> Result<StateTransition, GameError> {
                Ok(StateTransition::None)
            }
            
            fn render(&mut self, _renderer: &mut Renderer2D) -> Result<(), GameError> {
                Ok(())
            }
            
            fn handle_mouse_event(&mut self, _event: &MouseEvent) -> Result<bool, GameError> {
                Ok(false)
            }
            
            fn handle_keyboard_event(&mut self, _key: &str, _pressed: bool) -> Result<bool, GameError> {
                Ok(false)
            }
            
            fn handle_gamepad_event(&mut self, _event: &GamepadEvent) -> Result<bool, GameError> {
                Ok(false)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 测试状态
    define_state!(TestState, GameStateType::Custom("test".to_string()));
    
    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        assert_eq!(manager.get_stack_depth(), 0);
        assert!(manager.get_current_state().is_none());
    }
    
    #[test]
    fn test_state_registration() {
        let mut manager = StateManager::new();
        
        manager.register_state_factory(GameStateType::Custom("test".to_string()), || {
            Box::new(TestState::new())
        });
        
        assert!(manager.state_factories.contains_key(&GameStateType::Custom("test".to_string())));
    }
    
    #[test]
    fn test_state_push_pop() {
        let mut manager = StateManager::new();
        
        manager.register_state_factory(GameStateType::Custom("test".to_string()), || {
            Box::new(TestState::new())
        });
        
        // 推入状态
        let state_id = manager.push_state(GameStateType::Custom("test".to_string())).unwrap();
        assert_eq!(manager.get_stack_depth(), 1);
        assert!(manager.get_current_state().is_some());
        
        // 弹出状态
        let popped_type = manager.pop_state().unwrap();
        assert_eq!(popped_type, Some(GameStateType::Custom("test".to_string())));
        assert_eq!(manager.get_stack_depth(), 0);
    }
    
    #[test]
    fn test_state_transitions() {
        let mut manager = StateManager::new();
        
        manager.register_state_factory(GameStateType::Custom("test1".to_string()), || {
            Box::new(TestState::new())
        });
        manager.register_state_factory(GameStateType::Custom("test2".to_string()), || {
            Box::new(TestState::new())
        });
        
        // 测试推入转换
        manager.handle_transition(StateTransition::Push(GameStateType::Custom("test1".to_string()))).unwrap();
        assert_eq!(manager.get_stack_depth(), 1);
        
        // 测试替换转换
        manager.handle_transition(StateTransition::Replace(GameStateType::Custom("test2".to_string()))).unwrap();
        assert_eq!(manager.get_stack_depth(), 1);
        assert_eq!(manager.get_current_state_type(), Some(GameStateType::Custom("test2".to_string())));
        
        // 测试清空转换
        manager.handle_transition(StateTransition::Clear).unwrap();
        assert_eq!(manager.get_stack_depth(), 0);
    }
}