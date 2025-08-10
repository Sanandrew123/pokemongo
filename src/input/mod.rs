// 输入管理系统
// 开发心理：输入系统是用户与游戏交互的桥梁，需要响应快速、支持多平台
// 设计原则：事件驱动、可配置按键、支持多种输入设备、防抖动处理

pub mod keyboard;
pub mod mouse;
pub mod gamepad;
pub mod touch;

pub use keyboard::{KeyboardManager, KeyCode, KeyState};
pub use mouse::{MouseManager, MouseButton, MouseState};
pub use gamepad::{GamepadManager, GamepadButton, GamepadAxis, GamepadId};
pub use touch::{TouchManager, TouchEvent, TouchPhase, TouchId};

use crate::core::{GameError, Result};
use crate::core::event_system::{Event, EventSystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};

// 输入事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    KeyPressed { key: KeyCode, repeat: bool },
    KeyReleased { key: KeyCode },
    MousePressed { button: MouseButton, position: glam::Vec2 },
    MouseReleased { button: MouseButton, position: glam::Vec2 },
    MouseMoved { position: glam::Vec2, delta: glam::Vec2 },
    MouseScrolled { delta: glam::Vec2 },
    GamepadConnected { gamepad_id: GamepadId },
    GamepadDisconnected { gamepad_id: GamepadId },
    GamepadButtonPressed { gamepad_id: GamepadId, button: GamepadButton },
    GamepadButtonReleased { gamepad_id: GamepadId, button: GamepadButton },
    GamepadAxisChanged { gamepad_id: GamepadId, axis: GamepadAxis, value: f32 },
    TouchStarted { touch_id: TouchId, position: glam::Vec2 },
    TouchMoved { touch_id: TouchId, position: glam::Vec2, delta: glam::Vec2 },
    TouchEnded { touch_id: TouchId, position: glam::Vec2 },
    TouchCancelled { touch_id: TouchId },
}

impl Event for InputEvent {
    fn event_type(&self) -> &'static str {
        match self {
            InputEvent::KeyPressed { .. } => "KeyPressed",
            InputEvent::KeyReleased { .. } => "KeyReleased",
            InputEvent::MousePressed { .. } => "MousePressed",
            InputEvent::MouseReleased { .. } => "MouseReleased",
            InputEvent::MouseMoved { .. } => "MouseMoved",
            InputEvent::MouseScrolled { .. } => "MouseScrolled",
            InputEvent::GamepadConnected { .. } => "GamepadConnected",
            InputEvent::GamepadDisconnected { .. } => "GamepadDisconnected",
            InputEvent::GamepadButtonPressed { .. } => "GamepadButtonPressed",
            InputEvent::GamepadButtonReleased { .. } => "GamepadButtonReleased",
            InputEvent::GamepadAxisChanged { .. } => "GamepadAxisChanged",
            InputEvent::TouchStarted { .. } => "TouchStarted",
            InputEvent::TouchMoved { .. } => "TouchMoved",
            InputEvent::TouchEnded { .. } => "TouchEnded",
            InputEvent::TouchCancelled { .. } => "TouchCancelled",
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 输入动作系统
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    // 移动动作
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    
    // 相机控制
    CameraUp,
    CameraDown,
    CameraLeft,
    CameraRight,
    CameraZoomIn,
    CameraZoomOut,
    
    // 游戏动作
    Confirm,
    Cancel,
    Menu,
    Pause,
    Interact,
    
    // 战斗动作
    BattleAttack,
    BattleItem,
    BattleSwitch,
    BattleRun,
    
    // 调试动作
    DebugToggle,
    DebugReload,
    DebugCapture,
    
    // 自定义动作
    Custom(String),
}

// 输入绑定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    GamepadButton { gamepad_id: Option<GamepadId>, button: GamepadButton },
    GamepadAxis { gamepad_id: Option<GamepadId>, axis: GamepadAxis, threshold: f32 },
    Combination(Vec<InputBinding>), // 组合键
}

// 输入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub bindings: HashMap<InputAction, Vec<InputBinding>>,
    pub mouse_sensitivity: f32,
    pub gamepad_deadzone: f32,
    pub enable_mouse_acceleration: bool,
    pub enable_key_repeat: bool,
    pub double_click_time: f32,
    pub long_press_time: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        
        // 默认键位绑定
        bindings.insert(InputAction::MoveUp, vec![
            InputBinding::Key(KeyCode::W),
            InputBinding::Key(KeyCode::Up),
            InputBinding::GamepadAxis { 
                gamepad_id: None, 
                axis: GamepadAxis::LeftStickY, 
                threshold: 0.5 
            }
        ]);
        bindings.insert(InputAction::MoveDown, vec![
            InputBinding::Key(KeyCode::S),
            InputBinding::Key(KeyCode::Down),
            InputBinding::GamepadAxis { 
                gamepad_id: None, 
                axis: GamepadAxis::LeftStickY, 
                threshold: -0.5 
            }
        ]);
        bindings.insert(InputAction::MoveLeft, vec![
            InputBinding::Key(KeyCode::A),
            InputBinding::Key(KeyCode::Left),
            InputBinding::GamepadAxis { 
                gamepad_id: None, 
                axis: GamepadAxis::LeftStickX, 
                threshold: -0.5 
            }
        ]);
        bindings.insert(InputAction::MoveRight, vec![
            InputBinding::Key(KeyCode::D),
            InputBinding::Key(KeyCode::Right),
            InputBinding::GamepadAxis { 
                gamepad_id: None, 
                axis: GamepadAxis::LeftStickX, 
                threshold: 0.5 
            }
        ]);
        
        bindings.insert(InputAction::Confirm, vec![
            InputBinding::Key(KeyCode::Enter),
            InputBinding::Key(KeyCode::Space),
            InputBinding::MouseButton(MouseButton::Left),
            InputBinding::GamepadButton { gamepad_id: None, button: GamepadButton::A }
        ]);
        bindings.insert(InputAction::Cancel, vec![
            InputBinding::Key(KeyCode::Escape),
            InputBinding::MouseButton(MouseButton::Right),
            InputBinding::GamepadButton { gamepad_id: None, button: GamepadButton::B }
        ]);
        
        bindings.insert(InputAction::Menu, vec![
            InputBinding::Key(KeyCode::Tab),
            InputBinding::GamepadButton { gamepad_id: None, button: GamepadButton::Start }
        ]);
        bindings.insert(InputAction::Pause, vec![
            InputBinding::Key(KeyCode::P),
            InputBinding::Key(KeyCode::Escape),
            InputBinding::GamepadButton { gamepad_id: None, button: GamepadButton::Start }
        ]);
        
        Self {
            bindings,
            mouse_sensitivity: 1.0,
            gamepad_deadzone: 0.15,
            enable_mouse_acceleration: false,
            enable_key_repeat: true,
            double_click_time: 0.3,
            long_press_time: 0.8,
        }
    }
}

// 输入状态
#[derive(Debug, Clone)]
pub struct InputState {
    pub action_states: HashMap<InputAction, f32>, // 0.0 = not pressed, 1.0 = fully pressed
    pub action_just_pressed: HashMap<InputAction, bool>,
    pub action_just_released: HashMap<InputAction, bool>,
    pub action_press_duration: HashMap<InputAction, f32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            action_states: HashMap::new(),
            action_just_pressed: HashMap::new(),
            action_just_released: HashMap::new(),
            action_press_duration: HashMap::new(),
        }
    }
}

impl InputState {
    pub fn is_action_pressed(&self, action: &InputAction) -> bool {
        self.action_states.get(action).unwrap_or(&0.0) > 0.0
    }
    
    pub fn is_action_just_pressed(&self, action: &InputAction) -> bool {
        *self.action_just_pressed.get(action).unwrap_or(&false)
    }
    
    pub fn is_action_just_released(&self, action: &InputAction) -> bool {
        *self.action_just_released.get(action).unwrap_or(&false)
    }
    
    pub fn get_action_strength(&self, action: &InputAction) -> f32 {
        *self.action_states.get(action).unwrap_or(&0.0)
    }
    
    pub fn get_action_press_duration(&self, action: &InputAction) -> f32 {
        *self.action_press_duration.get(action).unwrap_or(&0.0)
    }
    
    pub fn is_action_long_pressed(&self, action: &InputAction, threshold: f32) -> bool {
        self.get_action_press_duration(action) >= threshold
    }
    
    pub fn clear_just_pressed_released(&mut self) {
        self.action_just_pressed.clear();
        self.action_just_released.clear();
    }
}

// 主要输入管理器
pub struct InputManager {
    keyboard: KeyboardManager,
    mouse: MouseManager,
    gamepad: GamepadManager,
    touch: TouchManager,
    
    config: InputConfig,
    current_state: InputState,
    previous_state: InputState,
    
    // 时间相关
    delta_time: f32,
    
    // 输入缓冲区（用于连击检测等）
    input_buffer: Vec<(InputAction, f32)>,
    buffer_duration: f32,
    
    // 输入锁定（用于UI等场景）
    input_locked: bool,
    locked_actions: std::collections::HashSet<InputAction>,
}

impl InputManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            keyboard: KeyboardManager::new(),
            mouse: MouseManager::new(),
            gamepad: GamepadManager::new()?,
            touch: TouchManager::new(),
            config: InputConfig::default(),
            current_state: InputState::default(),
            previous_state: InputState::default(),
            delta_time: 0.0,
            input_buffer: Vec::new(),
            buffer_duration: 1.0, // 1秒缓冲
            input_locked: false,
            locked_actions: std::collections::HashSet::new(),
        })
    }
    
    // 更新输入状态
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        self.delta_time = delta_time;
        
        // 更新各个输入设备
        self.keyboard.update(delta_time);
        self.mouse.update(delta_time);
        self.gamepad.update(delta_time)?;
        self.touch.update(delta_time);
        
        // 备份上一帧状态
        self.previous_state = self.current_state.clone();
        
        // 清除瞬时状态
        self.current_state.clear_just_pressed_released();
        
        // 更新动作状态
        self.update_action_states()?;
        
        // 更新输入缓冲区
        self.update_input_buffer(delta_time);
        
        Ok(())
    }
    
    // 处理输入事件
    pub fn handle_event(&mut self, event: &InputEvent) -> Result<()> {
        match event {
            InputEvent::KeyPressed { key, repeat } => {
                self.keyboard.handle_key_pressed(*key, *repeat);
            },
            InputEvent::KeyReleased { key } => {
                self.keyboard.handle_key_released(*key);
            },
            InputEvent::MousePressed { button, position } => {
                self.mouse.handle_button_pressed(*button, *position);
            },
            InputEvent::MouseReleased { button, position } => {
                self.mouse.handle_button_released(*button, *position);
            },
            InputEvent::MouseMoved { position, delta } => {
                self.mouse.handle_mouse_moved(*position, *delta);
            },
            InputEvent::MouseScrolled { delta } => {
                self.mouse.handle_scroll(*delta);
            },
            InputEvent::GamepadConnected { gamepad_id } => {
                self.gamepad.handle_gamepad_connected(*gamepad_id);
            },
            InputEvent::GamepadDisconnected { gamepad_id } => {
                self.gamepad.handle_gamepad_disconnected(*gamepad_id);
            },
            InputEvent::GamepadButtonPressed { gamepad_id, button } => {
                self.gamepad.handle_button_pressed(*gamepad_id, *button);
            },
            InputEvent::GamepadButtonReleased { gamepad_id, button } => {
                self.gamepad.handle_button_released(*gamepad_id, *button);
            },
            InputEvent::GamepadAxisChanged { gamepad_id, axis, value } => {
                self.gamepad.handle_axis_changed(*gamepad_id, *axis, *value);
            },
            InputEvent::TouchStarted { touch_id, position } => {
                self.touch.handle_touch_started(*touch_id, *position);
            },
            InputEvent::TouchMoved { touch_id, position, delta } => {
                self.touch.handle_touch_moved(*touch_id, *position, *delta);
            },
            InputEvent::TouchEnded { touch_id, position } => {
                self.touch.handle_touch_ended(*touch_id, *position);
            },
            InputEvent::TouchCancelled { touch_id } => {
                self.touch.handle_touch_cancelled(*touch_id);
            },
        }
        
        Ok(())
    }
    
    // 获取当前输入状态
    pub fn get_input_state(&self) -> &InputState {
        &self.current_state
    }
    
    // 检查动作是否被按下
    pub fn is_action_pressed(&self, action: &InputAction) -> bool {
        if self.input_locked || self.locked_actions.contains(action) {
            return false;
        }
        self.current_state.is_action_pressed(action)
    }
    
    // 检查动作是否刚被按下
    pub fn is_action_just_pressed(&self, action: &InputAction) -> bool {
        if self.input_locked || self.locked_actions.contains(action) {
            return false;
        }
        self.current_state.is_action_just_pressed(action)
    }
    
    // 检查动作是否刚被释放
    pub fn is_action_just_released(&self, action: &InputAction) -> bool {
        if self.input_locked || self.locked_actions.contains(action) {
            return false;
        }
        self.current_state.is_action_just_released(action)
    }
    
    // 获取动作强度（用于手柄摇杆等）
    pub fn get_action_strength(&self, action: &InputAction) -> f32 {
        if self.input_locked || self.locked_actions.contains(action) {
            return 0.0;
        }
        self.current_state.get_action_strength(action)
    }
    
    // 获取2D移动向量
    pub fn get_movement_vector(&self) -> glam::Vec2 {
        let x = self.get_action_strength(&InputAction::MoveRight) - 
               self.get_action_strength(&InputAction::MoveLeft);
        let y = self.get_action_strength(&InputAction::MoveUp) - 
               self.get_action_strength(&InputAction::MoveDown);
        
        let vector = glam::Vec2::new(x, y);
        
        // 归一化对角移动
        if vector.length() > 1.0 {
            vector.normalize()
        } else {
            vector
        }
    }
    
    // 配置管理
    pub fn set_config(&mut self, config: InputConfig) {
        self.config = config;
    }
    
    pub fn get_config(&self) -> &InputConfig {
        &self.config
    }
    
    pub fn get_config_mut(&mut self) -> &mut InputConfig {
        &mut self.config
    }
    
    // 输入锁定
    pub fn lock_input(&mut self) {
        self.input_locked = true;
    }
    
    pub fn unlock_input(&mut self) {
        self.input_locked = false;
    }
    
    pub fn is_input_locked(&self) -> bool {
        self.input_locked
    }
    
    pub fn lock_action(&mut self, action: InputAction) {
        self.locked_actions.insert(action);
    }
    
    pub fn unlock_action(&mut self, action: &InputAction) {
        self.locked_actions.remove(action);
    }
    
    // 输入缓冲区相关
    pub fn add_to_buffer(&mut self, action: InputAction) {
        self.input_buffer.push((action, 0.0));
    }
    
    pub fn get_buffered_actions(&self, max_age: f32) -> Vec<InputAction> {
        self.input_buffer.iter()
            .filter(|(_, age)| *age <= max_age)
            .map(|(action, _)| action.clone())
            .collect()
    }
    
    pub fn clear_buffer(&mut self) {
        self.input_buffer.clear();
    }
    
    // 获取原始设备状态
    pub fn get_keyboard(&self) -> &KeyboardManager {
        &self.keyboard
    }
    
    pub fn get_mouse(&self) -> &MouseManager {
        &self.mouse
    }
    
    pub fn get_gamepad(&self) -> &GamepadManager {
        &self.gamepad
    }
    
    pub fn get_touch(&self) -> &TouchManager {
        &self.touch
    }
    
    // 私有方法
    fn update_action_states(&mut self) -> Result<()> {
        for (action, bindings) in &self.config.bindings {
            let mut action_value = 0.0f32;
            let mut any_input = false;
            
            for binding in bindings {
                let binding_value = self.evaluate_binding(binding)?;
                if binding_value.abs() > action_value.abs() {
                    action_value = binding_value;
                    any_input = true;
                }
            }
            
            // 检查是否刚按下或释放
            let previous_value = self.previous_state.action_states.get(action).unwrap_or(&0.0);
            let just_pressed = *previous_value <= 0.0 && action_value > 0.0;
            let just_released = *previous_value > 0.0 && action_value <= 0.0;
            
            // 更新状态
            if action_value > 0.0 {
                self.current_state.action_states.insert(action.clone(), action_value);
                
                // 更新按压时间
                let duration = self.previous_state.action_press_duration.get(action).unwrap_or(&0.0) + self.delta_time;
                self.current_state.action_press_duration.insert(action.clone(), duration);
            } else {
                self.current_state.action_states.remove(action);
                self.current_state.action_press_duration.remove(action);
            }
            
            if just_pressed {
                self.current_state.action_just_pressed.insert(action.clone(), true);
                self.add_to_buffer(action.clone());
                debug!("动作按下: {:?}", action);
            }
            
            if just_released {
                self.current_state.action_just_released.insert(action.clone(), true);
                debug!("动作释放: {:?}", action);
            }
        }
        
        Ok(())
    }
    
    fn evaluate_binding(&self, binding: &InputBinding) -> Result<f32> {
        match binding {
            InputBinding::Key(key) => {
                Ok(if self.keyboard.is_key_pressed(key) { 1.0 } else { 0.0 })
            },
            InputBinding::MouseButton(button) => {
                Ok(if self.mouse.is_button_pressed(button) { 1.0 } else { 0.0 })
            },
            InputBinding::GamepadButton { gamepad_id, button } => {
                Ok(if self.gamepad.is_button_pressed(*gamepad_id, button) { 1.0 } else { 0.0 })
            },
            InputBinding::GamepadAxis { gamepad_id, axis, threshold } => {
                let axis_value = self.gamepad.get_axis_value(*gamepad_id, axis);
                
                // 应用死区
                let adjusted_value = if axis_value.abs() < self.config.gamepad_deadzone {
                    0.0
                } else {
                    axis_value
                };
                
                // 检查阈值
                if threshold.is_sign_positive() {
                    Ok(if adjusted_value >= *threshold { adjusted_value } else { 0.0 })
                } else {
                    Ok(if adjusted_value <= *threshold { -adjusted_value } else { 0.0 })
                }
            },
            InputBinding::Combination(bindings) => {
                // 组合键：所有绑定都必须激活
                let mut min_value = 1.0f32;
                for sub_binding in bindings {
                    let value = self.evaluate_binding(sub_binding)?;
                    if value <= 0.0 {
                        return Ok(0.0);
                    }
                    min_value = min_value.min(value);
                }
                Ok(min_value)
            }
        }
    }
    
    fn update_input_buffer(&mut self, delta_time: f32) {
        // 更新缓冲区中动作的年龄
        for (_, age) in &mut self.input_buffer {
            *age += delta_time;
        }
        
        // 移除过期的动作
        self.input_buffer.retain(|(_, age)| *age < self.buffer_duration);
    }
}

// 便利函数：创建全局输入管理器
static mut INPUT_MANAGER: Option<InputManager> = None;
static INPUT_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

pub struct Input;

impl Input {
    pub fn initialize() -> Result<()> {
        unsafe {
            INPUT_MANAGER_INIT.call_once(|| {
                match InputManager::new() {
                    Ok(manager) => {
                        INPUT_MANAGER = Some(manager);
                    },
                    Err(e) => {
                        log::error!("输入系统初始化失败: {}", e);
                    }
                }
            });
        }
        
        if unsafe { INPUT_MANAGER.is_none() } {
            return Err(GameError::InitializationFailed("输入系统初始化失败".to_string()));
        }
        
        Ok(())
    }
    
    pub fn instance() -> Result<&'static mut InputManager> {
        unsafe {
            INPUT_MANAGER.as_mut()
                .ok_or_else(|| GameError::SystemError("输入系统未初始化".to_string()))
        }
    }
    
    pub fn cleanup() {
        unsafe {
            INPUT_MANAGER = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_config_default() {
        let config = InputConfig::default();
        assert!(!config.bindings.is_empty());
        assert!(config.bindings.contains_key(&InputAction::MoveUp));
        assert!(config.bindings.contains_key(&InputAction::Confirm));
    }
    
    #[test]
    fn test_input_state() {
        let mut state = InputState::default();
        
        assert!(!state.is_action_pressed(&InputAction::Confirm));
        
        state.action_states.insert(InputAction::Confirm, 1.0);
        assert!(state.is_action_pressed(&InputAction::Confirm));
        
        state.action_just_pressed.insert(InputAction::Confirm, true);
        assert!(state.is_action_just_pressed(&InputAction::Confirm));
    }
    
    #[test]
    fn test_input_binding_evaluation() {
        // 这里需要实际的设备状态才能测试
        // 在实际游戏中会有更完整的测试
    }
}