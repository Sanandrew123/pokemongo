// 键盘输入管理
// 开发心理：键盘是PC游戏的主要输入方式，需要精确的按键状态跟踪
// 设计原则：状态机管理、防重复触发、支持组合键、可配置映射

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::debug;

// 键码定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // 字母键
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // 数字键
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // 功能键
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    
    // 方向键
    Up, Down, Left, Right,
    
    // 特殊键
    Escape, Enter, Return, Space, Tab, Backspace, Delete,
    Insert, Home, End, PageUp, PageDown,
    
    // 修饰键
    LeftShift, RightShift, LeftControl, RightControl,
    LeftAlt, RightAlt, LeftSuper, RightSuper,
    
    // 锁定键
    CapsLock, NumLock, ScrollLock,
    
    // 小键盘
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadDecimal, NumpadDivide, NumpadMultiply,
    NumpadSubtract, NumpadAdd, NumpadEnter, NumpadEqual,
    
    // 标点符号
    Semicolon, Equal, Comma, Minus, Period, Slash, Grave,
    LeftBracket, RightBracket, Backslash, Quote,
    
    // 媒体键
    VolumeUp, VolumeDown, VolumeMute,
    MediaPlay, MediaPause, MediaStop, MediaNext, MediaPrevious,
    
    // 系统键
    PrintScreen, Pause, Menu,
    
    // 未知键
    Unknown(u32),
}

// 键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Released,
    Pressed,
    Repeat,
}

// 键盘事件
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: KeyCode,
    pub state: KeyState,
    pub modifiers: KeyModifiers,
    pub timestamp: std::time::Instant,
}

// 修饰键状态
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool, // Windows键/Cmd键
}

impl KeyModifiers {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }
    
    pub fn with_ctrl(mut self, ctrl: bool) -> Self {
        self.ctrl = ctrl;
        self
    }
    
    pub fn with_alt(mut self, alt: bool) -> Self {
        self.alt = alt;
        self
    }
    
    pub fn with_super(mut self, super_key: bool) -> Self {
        self.super_key = super_key;
        self
    }
    
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.super_key
    }
    
    pub fn matches(&self, other: &KeyModifiers) -> bool {
        self.shift == other.shift &&
        self.ctrl == other.ctrl &&
        self.alt == other.alt &&
        self.super_key == other.super_key
    }
}

// 键组合
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombination {
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::default(),
        }
    }
    
    pub fn with_shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }
    
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers.ctrl = true;
        self
    }
    
    pub fn with_alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }
    
    pub fn with_super(mut self) -> Self {
        self.modifiers.super_key = true;
        self
    }
}

// 键盘管理器
pub struct KeyboardManager {
    key_states: HashMap<KeyCode, KeyState>,
    key_press_times: HashMap<KeyCode, std::time::Instant>,
    key_repeat_delays: HashMap<KeyCode, f32>,
    modifiers: KeyModifiers,
    
    // 配置
    repeat_delay: f32,      // 重复触发延迟
    repeat_rate: f32,       // 重复触发频率
    enable_repeat: bool,
    
    // 事件历史
    recent_events: Vec<KeyboardEvent>,
    max_event_history: usize,
    
    // 组合键检测
    combination_timeout: f32,
    pending_combinations: Vec<(KeyCode, std::time::Instant)>,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            key_press_times: HashMap::new(),
            key_repeat_delays: HashMap::new(),
            modifiers: KeyModifiers::default(),
            repeat_delay: 0.5,      // 500ms 初始延迟
            repeat_rate: 0.033,     // 30次/秒
            enable_repeat: true,
            recent_events: Vec::new(),
            max_event_history: 100,
            combination_timeout: 1.0, // 1秒组合键超时
            pending_combinations: Vec::new(),
        }
    }
    
    // 更新键盘状态（每帧调用）
    pub fn update(&mut self, delta_time: f32) {
        // 处理按键重复
        if self.enable_repeat {
            self.handle_key_repeat(delta_time);
        }
        
        // 清理过期的组合键
        let now = std::time::Instant::now();
        self.pending_combinations.retain(|(_, timestamp)| {
            now.duration_since(*timestamp).as_secs_f32() < self.combination_timeout
        });
        
        // 清理过期的事件历史
        if self.recent_events.len() > self.max_event_history {
            let excess = self.recent_events.len() - self.max_event_history;
            self.recent_events.drain(0..excess);
        }
        
        // 更新修饰键状态
        self.update_modifiers();
    }
    
    // 处理按键按下
    pub fn handle_key_pressed(&mut self, key: KeyCode, is_repeat: bool) {
        let current_time = std::time::Instant::now();
        
        let new_state = if is_repeat {
            KeyState::Repeat
        } else {
            KeyState::Pressed
        };
        
        // 更新键状态
        let previous_state = self.key_states.get(&key).unwrap_or(&KeyState::Released);
        self.key_states.insert(key, new_state);
        
        // 记录按下时间
        if !is_repeat {
            self.key_press_times.insert(key, current_time);
            self.key_repeat_delays.insert(key, 0.0);
            
            // 添加到组合键检测
            self.pending_combinations.push((key, current_time));
        }
        
        // 记录事件
        let event = KeyboardEvent {
            key,
            state: new_state,
            modifiers: self.modifiers,
            timestamp: current_time,
        };
        self.recent_events.push(event);
        
        debug!("键盘按下: {:?} (重复: {})", key, is_repeat);
    }
    
    // 处理按键释放
    pub fn handle_key_released(&mut self, key: KeyCode) {
        let current_time = std::time::Instant::now();
        
        // 更新键状态
        self.key_states.insert(key, KeyState::Released);
        
        // 清理相关数据
        self.key_press_times.remove(&key);
        self.key_repeat_delays.remove(&key);
        
        // 记录事件
        let event = KeyboardEvent {
            key,
            state: KeyState::Released,
            modifiers: self.modifiers,
            timestamp: current_time,
        };
        self.recent_events.push(event);
        
        debug!("键盘释放: {:?}", key);
    }
    
    // 检查键是否被按下
    pub fn is_key_pressed(&self, key: &KeyCode) -> bool {
        matches!(
            self.key_states.get(key),
            Some(KeyState::Pressed) | Some(KeyState::Repeat)
        )
    }
    
    // 检查键是否刚被按下（不包括重复）
    pub fn is_key_just_pressed(&self, key: &KeyCode) -> bool {
        matches!(self.key_states.get(key), Some(KeyState::Pressed))
    }
    
    // 检查键是否刚被释放
    pub fn is_key_just_released(&self, key: &KeyCode) -> bool {
        // 这需要与上一帧状态比较，这里简化处理
        // 在实际实现中应该维护上一帧的状态
        matches!(self.key_states.get(key), Some(KeyState::Released))
    }
    
    // 检查键是否处于重复状态
    pub fn is_key_repeating(&self, key: &KeyCode) -> bool {
        matches!(self.key_states.get(key), Some(KeyState::Repeat))
    }
    
    // 获取键的按下时长
    pub fn get_key_press_duration(&self, key: &KeyCode) -> Option<f32> {
        self.key_press_times.get(key).map(|&press_time| {
            std::time::Instant::now().duration_since(press_time).as_secs_f32()
        })
    }
    
    // 检查组合键
    pub fn is_combination_pressed(&self, combination: &KeyCombination) -> bool {
        self.is_key_pressed(&combination.key) && 
        self.modifiers.matches(&combination.modifiers)
    }
    
    pub fn is_combination_just_pressed(&self, combination: &KeyCombination) -> bool {
        self.is_key_just_pressed(&combination.key) && 
        self.modifiers.matches(&combination.modifiers)
    }
    
    // 获取当前修饰键状态
    pub fn get_modifiers(&self) -> KeyModifiers {
        self.modifiers
    }
    
    // 检查特定修饰键
    pub fn is_shift_pressed(&self) -> bool {
        self.modifiers.shift
    }
    
    pub fn is_ctrl_pressed(&self) -> bool {
        self.modifiers.ctrl
    }
    
    pub fn is_alt_pressed(&self) -> bool {
        self.modifiers.alt
    }
    
    pub fn is_super_pressed(&self) -> bool {
        self.modifiers.super_key
    }
    
    // 获取所有当前按下的键
    pub fn get_pressed_keys(&self) -> Vec<KeyCode> {
        self.key_states
            .iter()
            .filter_map(|(&key, &state)| {
                if matches!(state, KeyState::Pressed | KeyState::Repeat) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 检查是否有任何键被按下
    pub fn any_key_pressed(&self) -> bool {
        self.key_states.values().any(|&state| {
            matches!(state, KeyState::Pressed | KeyState::Repeat)
        })
    }
    
    // 获取最近的事件
    pub fn get_recent_events(&self) -> &[KeyboardEvent] {
        &self.recent_events
    }
    
    // 获取最近按下的组合键
    pub fn get_recent_combinations(&self, max_age: f32) -> Vec<KeyCode> {
        let now = std::time::Instant::now();
        self.pending_combinations
            .iter()
            .filter(|(_, timestamp)| {
                now.duration_since(*timestamp).as_secs_f32() <= max_age
            })
            .map(|(key, _)| *key)
            .collect()
    }
    
    // 清除所有状态
    pub fn clear_all_states(&mut self) {
        self.key_states.clear();
        self.key_press_times.clear();
        self.key_repeat_delays.clear();
        self.modifiers = KeyModifiers::default();
        self.recent_events.clear();
        self.pending_combinations.clear();
    }
    
    // 配置设置
    pub fn set_repeat_delay(&mut self, delay: f32) {
        self.repeat_delay = delay.max(0.0);
    }
    
    pub fn set_repeat_rate(&mut self, rate: f32) {
        self.repeat_rate = rate.max(0.001); // 最小1ms间隔
    }
    
    pub fn set_enable_repeat(&mut self, enable: bool) {
        self.enable_repeat = enable;
    }
    
    pub fn set_combination_timeout(&mut self, timeout: f32) {
        self.combination_timeout = timeout.max(0.1);
    }
    
    // 私有方法
    fn handle_key_repeat(&mut self, delta_time: f32) {
        let keys_to_repeat: Vec<KeyCode> = self.key_states
            .iter()
            .filter_map(|(&key, &state)| {
                if state == KeyState::Pressed {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();
        
        for key in keys_to_repeat {
            if let Some(delay) = self.key_repeat_delays.get_mut(&key) {
                *delay += delta_time;
                
                // 检查是否应该开始重复
                if *delay >= self.repeat_delay {
                    // 重置延迟为重复间隔
                    *delay = 0.0;
                    
                    // 触发重复事件
                    self.handle_key_pressed(key, true);
                }
            }
        }
    }
    
    fn update_modifiers(&mut self) {
        self.modifiers.shift = 
            self.is_key_pressed(&KeyCode::LeftShift) || 
            self.is_key_pressed(&KeyCode::RightShift);
            
        self.modifiers.ctrl = 
            self.is_key_pressed(&KeyCode::LeftControl) || 
            self.is_key_pressed(&KeyCode::RightControl);
            
        self.modifiers.alt = 
            self.is_key_pressed(&KeyCode::LeftAlt) || 
            self.is_key_pressed(&KeyCode::RightAlt);
            
        self.modifiers.super_key = 
            self.is_key_pressed(&KeyCode::LeftSuper) || 
            self.is_key_pressed(&KeyCode::RightSuper);
    }
}

// 便利函数：键码转换
impl KeyCode {
    // 将字符转换为键码
    pub fn from_char(c: char) -> Option<KeyCode> {
        match c.to_ascii_uppercase() {
            'A' => Some(KeyCode::A),
            'B' => Some(KeyCode::B),
            'C' => Some(KeyCode::C),
            'D' => Some(KeyCode::D),
            'E' => Some(KeyCode::E),
            'F' => Some(KeyCode::F),
            'G' => Some(KeyCode::G),
            'H' => Some(KeyCode::H),
            'I' => Some(KeyCode::I),
            'J' => Some(KeyCode::J),
            'K' => Some(KeyCode::K),
            'L' => Some(KeyCode::L),
            'M' => Some(KeyCode::M),
            'N' => Some(KeyCode::N),
            'O' => Some(KeyCode::O),
            'P' => Some(KeyCode::P),
            'Q' => Some(KeyCode::Q),
            'R' => Some(KeyCode::R),
            'S' => Some(KeyCode::S),
            'T' => Some(KeyCode::T),
            'U' => Some(KeyCode::U),
            'V' => Some(KeyCode::V),
            'W' => Some(KeyCode::W),
            'X' => Some(KeyCode::X),
            'Y' => Some(KeyCode::Y),
            'Z' => Some(KeyCode::Z),
            '0' => Some(KeyCode::Key0),
            '1' => Some(KeyCode::Key1),
            '2' => Some(KeyCode::Key2),
            '3' => Some(KeyCode::Key3),
            '4' => Some(KeyCode::Key4),
            '5' => Some(KeyCode::Key5),
            '6' => Some(KeyCode::Key6),
            '7' => Some(KeyCode::Key7),
            '8' => Some(KeyCode::Key8),
            '9' => Some(KeyCode::Key9),
            ' ' => Some(KeyCode::Space),
            _ => None,
        }
    }
    
    // 将键码转换为字符
    pub fn to_char(&self) -> Option<char> {
        match self {
            KeyCode::A => Some('A'),
            KeyCode::B => Some('B'),
            KeyCode::C => Some('C'),
            KeyCode::D => Some('D'),
            KeyCode::E => Some('E'),
            KeyCode::F => Some('F'),
            KeyCode::G => Some('G'),
            KeyCode::H => Some('H'),
            KeyCode::I => Some('I'),
            KeyCode::J => Some('J'),
            KeyCode::K => Some('K'),
            KeyCode::L => Some('L'),
            KeyCode::M => Some('M'),
            KeyCode::N => Some('N'),
            KeyCode::O => Some('O'),
            KeyCode::P => Some('P'),
            KeyCode::Q => Some('Q'),
            KeyCode::R => Some('R'),
            KeyCode::S => Some('S'),
            KeyCode::T => Some('T'),
            KeyCode::U => Some('U'),
            KeyCode::V => Some('V'),
            KeyCode::W => Some('W'),
            KeyCode::X => Some('X'),
            KeyCode::Y => Some('Y'),
            KeyCode::Z => Some('Z'),
            KeyCode::Key0 => Some('0'),
            KeyCode::Key1 => Some('1'),
            KeyCode::Key2 => Some('2'),
            KeyCode::Key3 => Some('3'),
            KeyCode::Key4 => Some('4'),
            KeyCode::Key5 => Some('5'),
            KeyCode::Key6 => Some('6'),
            KeyCode::Key7 => Some('7'),
            KeyCode::Key8 => Some('8'),
            KeyCode::Key9 => Some('9'),
            KeyCode::Space => Some(' '),
            _ => None,
        }
    }
    
    // 检查是否是修饰键
    pub fn is_modifier(&self) -> bool {
        matches!(self,
            KeyCode::LeftShift | KeyCode::RightShift |
            KeyCode::LeftControl | KeyCode::RightControl |
            KeyCode::LeftAlt | KeyCode::RightAlt |
            KeyCode::LeftSuper | KeyCode::RightSuper
        )
    }
    
    // 检查是否是功能键
    pub fn is_function_key(&self) -> bool {
        matches!(self,
            KeyCode::F1 | KeyCode::F2 | KeyCode::F3 | KeyCode::F4 |
            KeyCode::F5 | KeyCode::F6 | KeyCode::F7 | KeyCode::F8 |
            KeyCode::F9 | KeyCode::F10 | KeyCode::F11 | KeyCode::F12 |
            KeyCode::F13 | KeyCode::F14 | KeyCode::F15 | KeyCode::F16 |
            KeyCode::F17 | KeyCode::F18 | KeyCode::F19 | KeyCode::F20 |
            KeyCode::F21 | KeyCode::F22 | KeyCode::F23 | KeyCode::F24
        )
    }
    
    // 检查是否是数字键
    pub fn is_digit(&self) -> bool {
        matches!(self,
            KeyCode::Key0 | KeyCode::Key1 | KeyCode::Key2 | KeyCode::Key3 |
            KeyCode::Key4 | KeyCode::Key5 | KeyCode::Key6 | KeyCode::Key7 |
            KeyCode::Key8 | KeyCode::Key9
        )
    }
    
    // 检查是否是字母键
    pub fn is_letter(&self) -> bool {
        matches!(self,
            KeyCode::A | KeyCode::B | KeyCode::C | KeyCode::D | KeyCode::E |
            KeyCode::F | KeyCode::G | KeyCode::H | KeyCode::I | KeyCode::J |
            KeyCode::K | KeyCode::L | KeyCode::M | KeyCode::N | KeyCode::O |
            KeyCode::P | KeyCode::Q | KeyCode::R | KeyCode::S | KeyCode::T |
            KeyCode::U | KeyCode::V | KeyCode::W | KeyCode::X | KeyCode::Y |
            KeyCode::Z
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyboard_manager_creation() {
        let manager = KeyboardManager::new();
        assert!(!manager.any_key_pressed());
        assert!(!manager.is_shift_pressed());
    }
    
    #[test]
    fn test_key_press_release() {
        let mut manager = KeyboardManager::new();
        
        assert!(!manager.is_key_pressed(&KeyCode::A));
        
        manager.handle_key_pressed(KeyCode::A, false);
        assert!(manager.is_key_pressed(&KeyCode::A));
        assert!(manager.any_key_pressed());
        
        manager.handle_key_released(KeyCode::A);
        // 注意：实际实现中需要状态比较来确定just_released
    }
    
    #[test]
    fn test_key_modifiers() {
        let mut manager = KeyboardManager::new();
        
        manager.handle_key_pressed(KeyCode::LeftShift, false);
        manager.update_modifiers();
        assert!(manager.is_shift_pressed());
        
        manager.handle_key_pressed(KeyCode::LeftControl, false);
        manager.update_modifiers();
        assert!(manager.is_ctrl_pressed());
    }
    
    #[test]
    fn test_key_combination() {
        let combination = KeyCombination::new(KeyCode::A)
            .with_ctrl()
            .with_shift();
        
        assert_eq!(combination.key, KeyCode::A);
        assert!(combination.modifiers.ctrl);
        assert!(combination.modifiers.shift);
        assert!(!combination.modifiers.alt);
    }
    
    #[test]
    fn test_keycode_conversion() {
        assert_eq!(KeyCode::from_char('A'), Some(KeyCode::A));
        assert_eq!(KeyCode::from_char('a'), Some(KeyCode::A));
        assert_eq!(KeyCode::from_char('1'), Some(KeyCode::Key1));
        assert_eq!(KeyCode::from_char(' '), Some(KeyCode::Space));
        
        assert_eq!(KeyCode::A.to_char(), Some('A'));
        assert_eq!(KeyCode::Key1.to_char(), Some('1'));
        assert_eq!(KeyCode::Space.to_char(), Some(' '));
        
        assert!(KeyCode::A.is_letter());
        assert!(KeyCode::Key1.is_digit());
        assert!(KeyCode::F1.is_function_key());
        assert!(KeyCode::LeftShift.is_modifier());
    }
}