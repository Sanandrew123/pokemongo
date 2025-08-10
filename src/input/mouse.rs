// 鼠标输入管理
// 开发心理：鼠标是PC游戏的精确定位设备，需要处理点击、移动、滚轮等事件
// 设计原则：事件驱动、坐标转换、双击检测、拖拽支持

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::debug;

// 鼠标按键定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,        // 后退键
    Forward,     // 前进键
    Other(u8),   // 其他按键
}

// 鼠标按键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    Released,
    Pressed,
    Held,
}

// 鼠标事件
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub button: Option<MouseButton>,
    pub state: MouseState,
    pub position: glam::Vec2,
    pub delta: glam::Vec2,
    pub scroll_delta: glam::Vec2,
    pub timestamp: std::time::Instant,
}

// 鼠标管理器
pub struct MouseManager {
    // 按键状态
    button_states: HashMap<MouseButton, MouseState>,
    button_press_times: HashMap<MouseButton, std::time::Instant>,
    
    // 位置和移动
    current_position: glam::Vec2,
    previous_position: glam::Vec2,
    position_delta: glam::Vec2,
    
    // 滚轮状态
    scroll_delta: glam::Vec2,
    accumulated_scroll: glam::Vec2,
    
    // 配置参数
    sensitivity: f32,
    scroll_sensitivity: f32,
    enable_acceleration: bool,
    acceleration_factor: f32,
    
    // 双击检测
    double_click_time: f32,
    double_click_distance: f32,
    last_click_time: HashMap<MouseButton, std::time::Instant>,
    last_click_position: HashMap<MouseButton, glam::Vec2>,
    
    // 拖拽检测
    drag_threshold: f32,
    dragging_buttons: HashMap<MouseButton, glam::Vec2>, // 按键 -> 拖拽起始位置
    
    // 事件历史
    recent_events: Vec<MouseEvent>,
    max_event_history: usize,
    
    // 约束区域
    constraint_area: Option<(glam::Vec2, glam::Vec2)>, // (min, max)
    cursor_locked: bool,
    cursor_visible: bool,
}

impl MouseManager {
    pub fn new() -> Self {
        Self {
            button_states: HashMap::new(),
            button_press_times: HashMap::new(),
            current_position: glam::Vec2::ZERO,
            previous_position: glam::Vec2::ZERO,
            position_delta: glam::Vec2::ZERO,
            scroll_delta: glam::Vec2::ZERO,
            accumulated_scroll: glam::Vec2::ZERO,
            sensitivity: 1.0,
            scroll_sensitivity: 1.0,
            enable_acceleration: false,
            acceleration_factor: 1.5,
            double_click_time: 0.3, // 300ms
            double_click_distance: 5.0, // 5像素
            last_click_time: HashMap::new(),
            last_click_position: HashMap::new(),
            drag_threshold: 3.0, // 3像素
            dragging_buttons: HashMap::new(),
            recent_events: Vec::new(),
            max_event_history: 100,
            constraint_area: None,
            cursor_locked: false,
            cursor_visible: true,
        }
    }
    
    // 更新鼠标状态（每帧调用）
    pub fn update(&mut self, delta_time: f32) {
        // 保存上一帧位置
        self.previous_position = self.current_position;
        
        // 重置帧间变化
        self.scroll_delta = glam::Vec2::ZERO;
        
        // 更新按键状态（将Pressed转为Held）
        for (_, state) in self.button_states.iter_mut() {
            if *state == MouseState::Pressed {
                *state = MouseState::Held;
            }
        }
        
        // 应用鼠标加速
        if self.enable_acceleration {
            let speed = self.position_delta.length();
            let accel_factor = 1.0 + (speed * self.acceleration_factor * 0.001);
            self.position_delta *= accel_factor;
        }
        
        // 应用灵敏度
        self.position_delta *= self.sensitivity;
        
        // 清理过期的事件历史
        if self.recent_events.len() > self.max_event_history {
            let excess = self.recent_events.len() - self.max_event_history;
            self.recent_events.drain(0..excess);
        }
    }
    
    // 处理鼠标按键按下
    pub fn handle_button_pressed(&mut self, button: MouseButton, position: glam::Vec2) {
        let current_time = std::time::Instant::now();
        
        // 更新按键状态
        self.button_states.insert(button, MouseState::Pressed);
        self.button_press_times.insert(button, current_time);
        
        // 检测双击
        let is_double_click = self.check_double_click(button, position, current_time);
        
        // 开始拖拽检测
        if !is_double_click {
            self.dragging_buttons.insert(button, position);
        }
        
        // 记录事件
        let event = MouseEvent {
            button: Some(button),
            state: MouseState::Pressed,
            position,
            delta: glam::Vec2::ZERO,
            scroll_delta: glam::Vec2::ZERO,
            timestamp: current_time,
        };
        self.recent_events.push(event);
        
        debug!("鼠标按键按下: {:?} 位置: ({:.1}, {:.1})", button, position.x, position.y);
    }
    
    // 处理鼠标按键释放
    pub fn handle_button_released(&mut self, button: MouseButton, position: glam::Vec2) {
        let current_time = std::time::Instant::now();
        
        // 更新按键状态
        self.button_states.insert(button, MouseState::Released);
        self.button_press_times.remove(&button);
        
        // 停止拖拽
        self.dragging_buttons.remove(&button);
        
        // 记录事件
        let event = MouseEvent {
            button: Some(button),
            state: MouseState::Released,
            position,
            delta: glam::Vec2::ZERO,
            scroll_delta: glam::Vec2::ZERO,
            timestamp: current_time,
        };
        self.recent_events.push(event);
        
        debug!("鼠标按键释放: {:?} 位置: ({:.1}, {:.1})", button, position.x, position.y);
    }
    
    // 处理鼠标移动
    pub fn handle_mouse_moved(&mut self, position: glam::Vec2, delta: glam::Vec2) {
        // 应用位置约束
        let constrained_position = self.apply_position_constraints(position);
        
        // 更新位置
        self.current_position = constrained_position;
        self.position_delta = delta;
        
        // 更新拖拽状态
        self.update_drag_state();
        
        // 记录事件
        let event = MouseEvent {
            button: None,
            state: MouseState::Released, // 移动事件没有按键状态
            position: constrained_position,
            delta,
            scroll_delta: glam::Vec2::ZERO,
            timestamp: std::time::Instant::now(),
        };
        self.recent_events.push(event);
    }
    
    // 处理鼠标滚轮
    pub fn handle_scroll(&mut self, delta: glam::Vec2) {
        self.scroll_delta = delta * self.scroll_sensitivity;
        self.accumulated_scroll += self.scroll_delta;
        
        // 记录事件
        let event = MouseEvent {
            button: None,
            state: MouseState::Released,
            position: self.current_position,
            delta: glam::Vec2::ZERO,
            scroll_delta: self.scroll_delta,
            timestamp: std::time::Instant::now(),
        };
        self.recent_events.push(event);
        
        debug!("鼠标滚轮: ({:.1}, {:.1})", delta.x, delta.y);
    }
    
    // 检查按键是否被按下
    pub fn is_button_pressed(&self, button: &MouseButton) -> bool {
        matches!(
            self.button_states.get(button),
            Some(MouseState::Pressed) | Some(MouseState::Held)
        )
    }
    
    // 检查按键是否刚被按下
    pub fn is_button_just_pressed(&self, button: &MouseButton) -> bool {
        matches!(self.button_states.get(button), Some(MouseState::Pressed))
    }
    
    // 检查按键是否刚被释放
    pub fn is_button_just_released(&self, button: &MouseButton) -> bool {
        matches!(self.button_states.get(button), Some(MouseState::Released))
    }
    
    // 获取当前鼠标位置
    pub fn get_position(&self) -> glam::Vec2 {
        self.current_position
    }
    
    // 获取上一帧位置
    pub fn get_previous_position(&self) -> glam::Vec2 {
        self.previous_position
    }
    
    // 获取位置变化
    pub fn get_delta(&self) -> glam::Vec2 {
        self.position_delta
    }
    
    // 获取当前滚轮增量
    pub fn get_scroll_delta(&self) -> glam::Vec2 {
        self.scroll_delta
    }
    
    // 获取累积滚轮值
    pub fn get_accumulated_scroll(&self) -> glam::Vec2 {
        self.accumulated_scroll
    }
    
    // 重置累积滚轮值
    pub fn reset_accumulated_scroll(&mut self) {
        self.accumulated_scroll = glam::Vec2::ZERO;
    }
    
    // 检查是否正在拖拽
    pub fn is_dragging(&self, button: &MouseButton) -> bool {
        self.dragging_buttons.contains_key(button)
    }
    
    // 获取拖拽距离
    pub fn get_drag_distance(&self, button: &MouseButton) -> Option<f32> {
        self.dragging_buttons.get(button).map(|start_pos| {
            (self.current_position - *start_pos).length()
        })
    }
    
    // 获取拖拽向量
    pub fn get_drag_vector(&self, button: &MouseButton) -> Option<glam::Vec2> {
        self.dragging_buttons.get(button).map(|start_pos| {
            self.current_position - *start_pos
        })
    }
    
    // 获取按键按下时长
    pub fn get_button_press_duration(&self, button: &MouseButton) -> Option<f32> {
        self.button_press_times.get(button).map(|&press_time| {
            std::time::Instant::now().duration_since(press_time).as_secs_f32()
        })
    }
    
    // 设置鼠标灵敏度
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.max(0.1);
    }
    
    // 设置滚轮灵敏度
    pub fn set_scroll_sensitivity(&mut self, sensitivity: f32) {
        self.scroll_sensitivity = sensitivity.max(0.1);
    }
    
    // 设置鼠标加速
    pub fn set_acceleration(&mut self, enable: bool, factor: f32) {
        self.enable_acceleration = enable;
        self.acceleration_factor = factor.max(1.0);
    }
    
    // 设置双击参数
    pub fn set_double_click_params(&mut self, time: f32, distance: f32) {
        self.double_click_time = time.max(0.1);
        self.double_click_distance = distance.max(1.0);
    }
    
    // 设置拖拽阈值
    pub fn set_drag_threshold(&mut self, threshold: f32) {
        self.drag_threshold = threshold.max(1.0);
    }
    
    // 设置约束区域
    pub fn set_constraint_area(&mut self, min: glam::Vec2, max: glam::Vec2) {
        self.constraint_area = Some((min, max));
    }
    
    // 移除约束区域
    pub fn remove_constraint_area(&mut self) {
        self.constraint_area = None;
    }
    
    // 锁定鼠标
    pub fn lock_cursor(&mut self, locked: bool) {
        self.cursor_locked = locked;
        // TODO: 实际的系统光标锁定调用
    }
    
    // 设置光标可见性
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        // TODO: 实际的系统光标可见性调用
    }
    
    // 获取最近的事件
    pub fn get_recent_events(&self) -> &[MouseEvent] {
        &self.recent_events
    }
    
    // 获取所有按下的按键
    pub fn get_pressed_buttons(&self) -> Vec<MouseButton> {
        self.button_states
            .iter()
            .filter_map(|(&button, &state)| {
                if matches!(state, MouseState::Pressed | MouseState::Held) {
                    Some(button)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 检查是否有任何按键被按下
    pub fn any_button_pressed(&self) -> bool {
        self.button_states.values().any(|&state| {
            matches!(state, MouseState::Pressed | MouseState::Held)
        })
    }
    
    // 清除所有状态
    pub fn clear_all_states(&mut self) {
        self.button_states.clear();
        self.button_press_times.clear();
        self.last_click_time.clear();
        self.last_click_position.clear();
        self.dragging_buttons.clear();
        self.recent_events.clear();
        self.position_delta = glam::Vec2::ZERO;
        self.scroll_delta = glam::Vec2::ZERO;
    }
    
    // 私有方法
    fn check_double_click(&mut self, button: MouseButton, position: glam::Vec2, current_time: std::time::Instant) -> bool {
        if let (Some(&last_time), Some(&last_pos)) = (
            self.last_click_time.get(&button),
            self.last_click_position.get(&button)
        ) {
            let time_diff = current_time.duration_since(last_time).as_secs_f32();
            let distance = (position - last_pos).length();
            
            if time_diff <= self.double_click_time && distance <= self.double_click_distance {
                debug!("检测到双击: {:?}", button);
                return true;
            }
        }
        
        // 更新最后点击信息
        self.last_click_time.insert(button, current_time);
        self.last_click_position.insert(button, position);
        
        false
    }
    
    fn update_drag_state(&mut self) {
        let mut to_remove = Vec::new();
        
        for (&button, &start_pos) in &self.dragging_buttons {
            let distance = (self.current_position - start_pos).length();
            
            // 如果移动距离超过阈值，开始拖拽
            if distance > self.drag_threshold {
                debug!("开始拖拽: {:?} 距离: {:.1}", button, distance);
            }
            
            // 如果按键已释放，停止拖拽
            if !self.is_button_pressed(&button) {
                to_remove.push(button);
            }
        }
        
        for button in to_remove {
            self.dragging_buttons.remove(&button);
        }
    }
    
    fn apply_position_constraints(&self, position: glam::Vec2) -> glam::Vec2 {
        if let Some((min, max)) = self.constraint_area {
            glam::Vec2::new(
                position.x.clamp(min.x, max.x),
                position.y.clamp(min.y, max.y),
            )
        } else {
            position
        }
    }
}

// 便利函数：鼠标按键转换
impl MouseButton {
    pub fn from_u8(button_id: u8) -> Self {
        match button_id {
            0 => MouseButton::Left,
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            3 => MouseButton::Back,
            4 => MouseButton::Forward,
            other => MouseButton::Other(other),
        }
    }
    
    pub fn to_u8(&self) -> u8 {
        match self {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(id) => *id,
        }
    }
    
    pub fn is_primary(&self) -> bool {
        matches!(self, MouseButton::Left | MouseButton::Right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mouse_manager_creation() {
        let manager = MouseManager::new();
        assert!(!manager.any_button_pressed());
        assert_eq!(manager.get_position(), glam::Vec2::ZERO);
    }
    
    #[test]
    fn test_button_press_release() {
        let mut manager = MouseManager::new();
        let position = glam::Vec2::new(100.0, 200.0);
        
        assert!(!manager.is_button_pressed(&MouseButton::Left));
        
        manager.handle_button_pressed(MouseButton::Left, position);
        assert!(manager.is_button_pressed(&MouseButton::Left));
        assert!(manager.is_button_just_pressed(&MouseButton::Left));
        
        manager.update(0.016); // 模拟一帧
        assert!(manager.is_button_pressed(&MouseButton::Left));
        assert!(!manager.is_button_just_pressed(&MouseButton::Left)); // 不再是刚按下
        
        manager.handle_button_released(MouseButton::Left, position);
        assert!(!manager.is_button_pressed(&MouseButton::Left));
    }
    
    #[test]
    fn test_mouse_movement() {
        let mut manager = MouseManager::new();
        let position1 = glam::Vec2::new(10.0, 20.0);
        let position2 = glam::Vec2::new(15.0, 25.0);
        let delta = position2 - position1;
        
        manager.handle_mouse_moved(position1, glam::Vec2::ZERO);
        assert_eq!(manager.get_position(), position1);
        
        manager.handle_mouse_moved(position2, delta);
        assert_eq!(manager.get_position(), position2);
        assert_eq!(manager.get_delta(), delta);
    }
    
    #[test]
    fn test_scroll() {
        let mut manager = MouseManager::new();
        let scroll = glam::Vec2::new(0.0, 1.0);
        
        manager.handle_scroll(scroll);
        assert_eq!(manager.get_scroll_delta(), scroll);
        assert_eq!(manager.get_accumulated_scroll(), scroll);
        
        manager.handle_scroll(scroll);
        assert_eq!(manager.get_accumulated_scroll(), scroll * 2.0);
        
        manager.reset_accumulated_scroll();
        assert_eq!(manager.get_accumulated_scroll(), glam::Vec2::ZERO);
    }
    
    #[test]
    fn test_button_conversion() {
        assert_eq!(MouseButton::from_u8(0), MouseButton::Left);
        assert_eq!(MouseButton::from_u8(1), MouseButton::Right);
        assert_eq!(MouseButton::from_u8(2), MouseButton::Middle);
        
        assert_eq!(MouseButton::Left.to_u8(), 0);
        assert_eq!(MouseButton::Right.to_u8(), 1);
        
        assert!(MouseButton::Left.is_primary());
        assert!(MouseButton::Right.is_primary());
        assert!(!MouseButton::Middle.is_primary());
    }
    
    #[test]
    fn test_drag_detection() {
        let mut manager = MouseManager::new();
        let start_pos = glam::Vec2::new(10.0, 10.0);
        let end_pos = glam::Vec2::new(20.0, 20.0);
        
        manager.handle_button_pressed(MouseButton::Left, start_pos);
        assert!(manager.is_dragging(&MouseButton::Left));
        
        manager.handle_mouse_moved(end_pos, end_pos - start_pos);
        
        let drag_distance = manager.get_drag_distance(&MouseButton::Left);
        assert!(drag_distance.is_some());
        assert!(drag_distance.unwrap() > manager.drag_threshold);
    }
}