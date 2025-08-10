// 游戏手柄输入管理
// 开发心理：手柄是游戏的核心输入设备，需要支持多种手柄类型、振动反馈、模拟摇杆
// 设计原则：跨平台兼容、死区处理、按键映射、状态管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};

// 手柄按键定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    // 面部按键
    South,      // A / X
    East,       // B / Circle  
    West,       // X / Square
    North,      // Y / Triangle
    
    // 肩键
    LeftBumper,     // LB / L1
    RightBumper,    // RB / R1
    LeftTrigger2,   // LT / L2 (数字)
    RightTrigger2,  // RT / R2 (数字)
    
    // 中央按键
    Select,     // Back / Share
    Start,      // Start / Options
    Mode,       // Xbox / PS按键
    
    // 摇杆按键
    LeftThumb,  // 左摇杆按下
    RightThumb, // 右摇杆按下
    
    // 方向键
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // 扩展按键
    Paddle1,    // 背键1
    Paddle2,    // 背键2
    Paddle3,    // 背键3
    Paddle4,    // 背键4
    Touchpad,   // 触控板按下 (PS4/PS5)
    
    Unknown(u8), // 未知按键
}

// 手柄轴定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,     // 左摇杆X轴
    LeftStickY,     // 左摇杆Y轴
    RightStickX,    // 右摇杆X轴
    RightStickY,    // 右摇杆Y轴
    LeftTrigger,    // 左扳机 (模拟)
    RightTrigger,   // 右扳机 (模拟)
    
    // 扩展轴
    MotionX,        // 陀螺仪X
    MotionY,        // 陀螺仪Y
    MotionZ,        // 陀螺仪Z
    TouchpadX,      // 触控板X
    TouchpadY,      // 触控板Y
    
    Unknown(u8),
}

// 手柄类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamepadType {
    Xbox360,
    XboxOne,
    XboxSeriesX,
    PlayStation3,
    PlayStation4,
    PlayStation5,
    NintendoSwitch,
    Generic,
    Unknown,
}

// 按键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Released,
    Pressed,
    Held,
}

// 振动类型
#[derive(Debug, Clone, Copy)]
pub struct VibrationEffect {
    pub left_motor: f32,    // 低频马达 (0.0-1.0)
    pub right_motor: f32,   // 高频马达 (0.0-1.0)
    pub duration_ms: u32,   // 持续时间
}

// 手柄状态
#[derive(Debug, Clone)]
pub struct GamepadState {
    pub id: u32,
    pub name: String,
    pub gamepad_type: GamepadType,
    pub connected: bool,
    pub battery_level: Option<f32>, // 0.0-1.0
    
    // 按键状态
    pub buttons: HashMap<GamepadButton, ButtonState>,
    pub button_values: HashMap<GamepadButton, f32>, // 压力敏感按键
    
    // 轴状态
    pub axes: HashMap<GamepadAxis, f32>,
    pub axes_raw: HashMap<GamepadAxis, f32>, // 原始值(未处理死区)
    
    // 振动状态
    pub vibration: Option<VibrationEffect>,
    
    // 配置
    pub dead_zone: f32,
    pub trigger_threshold: f32,
}

// 手柄事件
#[derive(Debug, Clone)]
pub struct GamepadEvent {
    pub gamepad_id: u32,
    pub event_type: GamepadEventType,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum GamepadEventType {
    Connected(GamepadState),
    Disconnected,
    ButtonPressed(GamepadButton),
    ButtonReleased(GamepadButton),
    AxisChanged(GamepadAxis, f32, f32), // (axis, old_value, new_value)
    BatteryChanged(f32),
}

// 手柄管理器
pub struct GamepadManager {
    gamepads: HashMap<u32, GamepadState>,
    
    // 全局配置
    default_dead_zone: f32,
    default_trigger_threshold: f32,
    enable_vibration: bool,
    
    // 按键映射
    button_mappings: HashMap<GamepadType, HashMap<u8, GamepadButton>>,
    axis_mappings: HashMap<GamepadType, HashMap<u8, GamepadAxis>>,
    
    // 事件历史
    recent_events: Vec<GamepadEvent>,
    max_event_history: usize,
    
    // 输入过滤
    axis_filters: HashMap<GamepadAxis, AxisFilter>,
    button_debounce_time: f32,
    last_button_times: HashMap<(u32, GamepadButton), std::time::Instant>,
}

// 轴过滤器
#[derive(Debug, Clone)]
struct AxisFilter {
    smoothing_factor: f32,
    last_value: f32,
    change_threshold: f32,
}

impl GamepadManager {
    pub fn new() -> Self {
        let mut manager = Self {
            gamepads: HashMap::new(),
            default_dead_zone: 0.15,
            default_trigger_threshold: 0.1,
            enable_vibration: true,
            button_mappings: HashMap::new(),
            axis_mappings: HashMap::new(),
            recent_events: Vec::new(),
            max_event_history: 200,
            axis_filters: HashMap::new(),
            button_debounce_time: 0.01, // 10ms
            last_button_times: HashMap::new(),
        };
        
        manager.setup_default_mappings();
        manager.setup_axis_filters();
        manager
    }
    
    // 更新所有手柄状态
    pub fn update(&mut self, delta_time: f32) {
        // 更新按键状态 (Pressed -> Held)
        for gamepad in self.gamepads.values_mut() {
            for state in gamepad.buttons.values_mut() {
                if *state == ButtonState::Pressed {
                    *state = ButtonState::Held;
                }
            }
            
            // 更新振动
            if let Some(vibration) = &mut gamepad.vibration {
                if vibration.duration_ms > 0 {
                    let decrease = (delta_time * 1000.0) as u32;
                    vibration.duration_ms = vibration.duration_ms.saturating_sub(decrease);
                    
                    if vibration.duration_ms == 0 {
                        gamepad.vibration = None;
                        // TODO: 停止实际的振动
                    }
                }
            }
        }
        
        // 应用轴过滤
        self.apply_axis_filtering(delta_time);
        
        // 清理事件历史
        if self.recent_events.len() > self.max_event_history {
            let excess = self.recent_events.len() - self.max_event_history;
            self.recent_events.drain(0..excess);
        }
    }
    
    // 添加手柄
    pub fn add_gamepad(&mut self, id: u32, name: String, gamepad_type: GamepadType) {
        let gamepad = GamepadState {
            id,
            name: name.clone(),
            gamepad_type: gamepad_type.clone(),
            connected: true,
            battery_level: None,
            buttons: HashMap::new(),
            button_values: HashMap::new(),
            axes: HashMap::new(),
            axes_raw: HashMap::new(),
            vibration: None,
            dead_zone: self.default_dead_zone,
            trigger_threshold: self.default_trigger_threshold,
        };
        
        self.gamepads.insert(id, gamepad.clone());
        
        let event = GamepadEvent {
            gamepad_id: id,
            event_type: GamepadEventType::Connected(gamepad),
            timestamp: std::time::Instant::now(),
        };
        self.recent_events.push(event);
        
        debug!("手柄连接: ID={} 名称='{}' 类型={:?}", id, name, gamepad_type);
    }
    
    // 移除手柄
    pub fn remove_gamepad(&mut self, id: u32) {
        if self.gamepads.remove(&id).is_some() {
            let event = GamepadEvent {
                gamepad_id: id,
                event_type: GamepadEventType::Disconnected,
                timestamp: std::time::Instant::now(),
            };
            self.recent_events.push(event);
            
            debug!("手柄断开: ID={}", id);
        }
    }
    
    // 处理按键按下
    pub fn handle_button_pressed(&mut self, gamepad_id: u32, button: GamepadButton, value: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            let current_time = std::time::Instant::now();
            
            // 防抖处理
            if let Some(&last_time) = self.last_button_times.get(&(gamepad_id, button)) {
                if current_time.duration_since(last_time).as_secs_f32() < self.button_debounce_time {
                    return;
                }
            }
            
            gamepad.buttons.insert(button, ButtonState::Pressed);
            gamepad.button_values.insert(button, value);
            self.last_button_times.insert((gamepad_id, button), current_time);
            
            let event = GamepadEvent {
                gamepad_id,
                event_type: GamepadEventType::ButtonPressed(button),
                timestamp: current_time,
            };
            self.recent_events.push(event);
            
            debug!("手柄按键按下: ID={} 按键={:?} 值={:.2}", gamepad_id, button, value);
        }
    }
    
    // 处理按键释放
    pub fn handle_button_released(&mut self, gamepad_id: u32, button: GamepadButton) {
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            gamepad.buttons.insert(button, ButtonState::Released);
            gamepad.button_values.insert(button, 0.0);
            
            let event = GamepadEvent {
                gamepad_id,
                event_type: GamepadEventType::ButtonReleased(button),
                timestamp: std::time::Instant::now(),
            };
            self.recent_events.push(event);
            
            debug!("手柄按键释放: ID={} 按键={:?}", gamepad_id, button);
        }
    }
    
    // 处理轴变化
    pub fn handle_axis_changed(&mut self, gamepad_id: u32, axis: GamepadAxis, value: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            let old_value = gamepad.axes.get(&axis).copied().unwrap_or(0.0);
            gamepad.axes_raw.insert(axis, value);
            
            // 应用死区
            let processed_value = self.apply_dead_zone(axis, value, gamepad.dead_zone);
            gamepad.axes.insert(axis, processed_value);
            
            // 只有变化足够大时才发送事件
            let change_threshold = self.axis_filters.get(&axis)
                .map(|f| f.change_threshold)
                .unwrap_or(0.01);
            
            if (processed_value - old_value).abs() > change_threshold {
                let event = GamepadEvent {
                    gamepad_id,
                    event_type: GamepadEventType::AxisChanged(axis, old_value, processed_value),
                    timestamp: std::time::Instant::now(),
                };
                self.recent_events.push(event);
            }
        }
    }
    
    // 设置振动
    pub fn set_vibration(&mut self, gamepad_id: u32, effect: VibrationEffect) -> Result<(), String> {
        if !self.enable_vibration {
            return Err("振动已禁用".to_string());
        }
        
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            gamepad.vibration = Some(effect);
            // TODO: 调用实际的振动API
            debug!("设置振动: ID={} 左={:.2} 右={:.2} 时长={}ms", 
                   gamepad_id, effect.left_motor, effect.right_motor, effect.duration_ms);
            Ok(())
        } else {
            Err(format!("手柄不存在: {}", gamepad_id))
        }
    }
    
    // 停止振动
    pub fn stop_vibration(&mut self, gamepad_id: u32) -> Result<(), String> {
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            gamepad.vibration = None;
            // TODO: 停止实际的振动
            debug!("停止振动: ID={}", gamepad_id);
            Ok(())
        } else {
            Err(format!("手柄不存在: {}", gamepad_id))
        }
    }
    
    // 获取手柄状态
    pub fn get_gamepad(&self, id: u32) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }
    
    // 获取所有连接的手柄
    pub fn get_connected_gamepads(&self) -> Vec<&GamepadState> {
        self.gamepads.values().filter(|g| g.connected).collect()
    }
    
    // 检查按键状态
    pub fn is_button_pressed(&self, gamepad_id: u32, button: &GamepadButton) -> bool {
        self.gamepads.get(&gamepad_id)
            .and_then(|g| g.buttons.get(button))
            .map(|&state| matches!(state, ButtonState::Pressed | ButtonState::Held))
            .unwrap_or(false)
    }
    
    pub fn is_button_just_pressed(&self, gamepad_id: u32, button: &GamepadButton) -> bool {
        self.gamepads.get(&gamepad_id)
            .and_then(|g| g.buttons.get(button))
            .map(|&state| state == ButtonState::Pressed)
            .unwrap_or(false)
    }
    
    // 获取轴值
    pub fn get_axis_value(&self, gamepad_id: u32, axis: &GamepadAxis) -> f32 {
        self.gamepads.get(&gamepad_id)
            .and_then(|g| g.axes.get(axis))
            .copied()
            .unwrap_or(0.0)
    }
    
    // 获取摇杆向量
    pub fn get_stick_vector(&self, gamepad_id: u32, stick: StickType) -> glam::Vec2 {
        let (x_axis, y_axis) = match stick {
            StickType::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
            StickType::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
        };
        
        glam::Vec2::new(
            self.get_axis_value(gamepad_id, &x_axis),
            self.get_axis_value(gamepad_id, &y_axis),
        )
    }
    
    // 配置设置
    pub fn set_dead_zone(&mut self, gamepad_id: u32, dead_zone: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
            gamepad.dead_zone = dead_zone.clamp(0.0, 0.9);
        }
    }
    
    pub fn set_global_vibration(&mut self, enabled: bool) {
        self.enable_vibration = enabled;
        if !enabled {
            // 停止所有振动
            for gamepad in self.gamepads.values_mut() {
                gamepad.vibration = None;
            }
        }
    }
    
    // 获取最近事件
    pub fn get_recent_events(&self) -> &[GamepadEvent] {
        &self.recent_events
    }
    
    // 私有方法
    fn setup_default_mappings(&mut self) {
        // Xbox控制器映射
        let mut xbox_buttons = HashMap::new();
        xbox_buttons.insert(0, GamepadButton::South);
        xbox_buttons.insert(1, GamepadButton::East);
        xbox_buttons.insert(2, GamepadButton::West);
        xbox_buttons.insert(3, GamepadButton::North);
        // ... 更多映射
        
        self.button_mappings.insert(GamepadType::XboxOne, xbox_buttons);
        
        // PlayStation控制器映射
        let mut ps_buttons = HashMap::new();
        ps_buttons.insert(0, GamepadButton::South); // X
        ps_buttons.insert(1, GamepadButton::East);  // Circle
        ps_buttons.insert(2, GamepadButton::West);  // Square
        ps_buttons.insert(3, GamepadButton::North); // Triangle
        
        self.button_mappings.insert(GamepadType::PlayStation4, ps_buttons);
    }
    
    fn setup_axis_filters(&mut self) {
        // 摇杆轴的过滤设置
        for axis in [GamepadAxis::LeftStickX, GamepadAxis::LeftStickY, 
                     GamepadAxis::RightStickX, GamepadAxis::RightStickY] {
            self.axis_filters.insert(axis, AxisFilter {
                smoothing_factor: 0.1,
                last_value: 0.0,
                change_threshold: 0.02,
            });
        }
        
        // 扳机轴的过滤设置
        for axis in [GamepadAxis::LeftTrigger, GamepadAxis::RightTrigger] {
            self.axis_filters.insert(axis, AxisFilter {
                smoothing_factor: 0.05,
                last_value: 0.0,
                change_threshold: 0.05,
            });
        }
    }
    
    fn apply_dead_zone(&self, axis: GamepadAxis, value: f32, dead_zone: f32) -> f32 {
        match axis {
            GamepadAxis::LeftStickX | GamepadAxis::LeftStickY |
            GamepadAxis::RightStickX | GamepadAxis::RightStickY => {
                if value.abs() < dead_zone {
                    0.0
                } else {
                    // 重新映射到[0,1]范围
                    let sign = value.signum();
                    let normalized = (value.abs() - dead_zone) / (1.0 - dead_zone);
                    sign * normalized.clamp(0.0, 1.0)
                }
            }
            GamepadAxis::LeftTrigger | GamepadAxis::RightTrigger => {
                if value < self.default_trigger_threshold {
                    0.0
                } else {
                    value
                }
            }
            _ => value,
        }
    }
    
    fn apply_axis_filtering(&mut self, _delta_time: f32) {
        for gamepad in self.gamepads.values_mut() {
            for (&axis, &raw_value) in &gamepad.axes_raw.clone() {
                if let Some(filter) = self.axis_filters.get_mut(&axis) {
                    // 简单的低通滤波
                    filter.last_value = filter.last_value * (1.0 - filter.smoothing_factor) 
                                      + raw_value * filter.smoothing_factor;
                    
                    gamepad.axes.insert(axis, filter.last_value);
                }
            }
        }
    }
}

// 摇杆类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickType {
    Left,
    Right,
}

// 便利方法
impl GamepadType {
    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower.contains("xbox") {
            if name_lower.contains("360") {
                GamepadType::Xbox360
            } else if name_lower.contains("one") {
                GamepadType::XboxOne
            } else if name_lower.contains("series") {
                GamepadType::XboxSeriesX
            } else {
                GamepadType::XboxOne // 默认
            }
        } else if name_lower.contains("playstation") || name_lower.contains("ps") {
            if name_lower.contains("3") {
                GamepadType::PlayStation3
            } else if name_lower.contains("4") {
                GamepadType::PlayStation4
            } else if name_lower.contains("5") {
                GamepadType::PlayStation5
            } else {
                GamepadType::PlayStation4 // 默认
            }
        } else if name_lower.contains("nintendo") || name_lower.contains("switch") {
            GamepadType::NintendoSwitch
        } else {
            GamepadType::Generic
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gamepad_manager_creation() {
        let manager = GamepadManager::new();
        assert_eq!(manager.get_connected_gamepads().len(), 0);
    }
    
    #[test]
    fn test_add_remove_gamepad() {
        let mut manager = GamepadManager::new();
        
        manager.add_gamepad(0, "Test Controller".to_string(), GamepadType::Xbox360);
        assert_eq!(manager.get_connected_gamepads().len(), 1);
        
        manager.remove_gamepad(0);
        assert_eq!(manager.gamepads.len(), 0);
    }
    
    #[test]
    fn test_button_press_release() {
        let mut manager = GamepadManager::new();
        manager.add_gamepad(0, "Test".to_string(), GamepadType::Generic);
        
        assert!(!manager.is_button_pressed(0, &GamepadButton::South));
        
        manager.handle_button_pressed(0, GamepadButton::South, 1.0);
        assert!(manager.is_button_pressed(0, &GamepadButton::South));
        assert!(manager.is_button_just_pressed(0, &GamepadButton::South));
        
        manager.handle_button_released(0, GamepadButton::South);
        assert!(!manager.is_button_pressed(0, &GamepadButton::South));
    }
    
    #[test]
    fn test_axis_dead_zone() {
        let mut manager = GamepadManager::new();
        manager.add_gamepad(0, "Test".to_string(), GamepadType::Generic);
        
        // 小于死区的值应该被过滤为0
        manager.handle_axis_changed(0, GamepadAxis::LeftStickX, 0.1);
        assert_eq!(manager.get_axis_value(0, &GamepadAxis::LeftStickX), 0.0);
        
        // 大于死区的值应该被重新映射
        manager.handle_axis_changed(0, GamepadAxis::LeftStickX, 0.5);
        let value = manager.get_axis_value(0, &GamepadAxis::LeftStickX);
        assert!(value > 0.0 && value < 0.5);
    }
    
    #[test]
    fn test_stick_vector() {
        let mut manager = GamepadManager::new();
        manager.add_gamepad(0, "Test".to_string(), GamepadType::Generic);
        
        manager.handle_axis_changed(0, GamepadAxis::LeftStickX, 0.8);
        manager.handle_axis_changed(0, GamepadAxis::LeftStickY, 0.6);
        
        let vector = manager.get_stick_vector(0, StickType::Left);
        assert!(vector.x > 0.0);
        assert!(vector.y > 0.0);
    }
}