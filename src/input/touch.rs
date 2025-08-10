// 触摸输入管理
// 开发心理：触摸是移动设备的主要交互方式，需要支持多点触控、手势识别、压力感应
// 设计原则：事件驱动、多点跟踪、手势识别、性能优化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::debug;

// 触摸点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchState {
    Started,    // 触摸开始
    Moving,     // 移动中
    Ended,      // 触摸结束
    Cancelled,  // 触摸取消
}

// 触摸点信息
#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,                    // 触摸点唯一ID
    pub position: glam::Vec2,       // 当前位置
    pub start_position: glam::Vec2, // 起始位置
    pub previous_position: glam::Vec2, // 上一帧位置
    pub delta: glam::Vec2,          // 位置变化
    pub state: TouchState,          // 状态
    pub pressure: f32,              // 压力 (0.0-1.0)
    pub major_axis: f32,            // 椭圆长轴
    pub minor_axis: f32,            // 椭圆短轴
    pub angle: f32,                 // 角度(弧度)
    pub start_time: std::time::Instant, // 开始时间
    pub last_update: std::time::Instant, // 最后更新时间
}

// 手势类型
#[derive(Debug, Clone, PartialEq)]
pub enum GestureType {
    Tap,                // 单击
    DoubleTap,          // 双击
    LongPress,          // 长按
    Pan,                // 拖拽
    Pinch,              // 缩放
    Rotate,             // 旋转
    Swipe(SwipeDirection), // 滑动
    Custom(String),     // 自定义手势
}

// 滑动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

// 手势事件
#[derive(Debug, Clone)]
pub struct GestureEvent {
    pub gesture_type: GestureType,
    pub position: glam::Vec2,       // 手势中心点
    pub delta: glam::Vec2,          // 变化量
    pub scale: f32,                 // 缩放比例
    pub rotation: f32,              // 旋转角度
    pub velocity: glam::Vec2,       // 速度
    pub duration: f32,              // 持续时间
    pub touch_points: Vec<u64>,     // 参与的触摸点ID
    pub timestamp: std::time::Instant,
}

// 触摸事件
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub touch_point: TouchPoint,
    pub timestamp: std::time::Instant,
}

// 触摸配置
#[derive(Debug, Clone)]
pub struct TouchConfig {
    // 手势识别参数
    pub tap_max_duration: f32,      // 最大点击时长
    pub tap_max_distance: f32,      // 最大点击距离
    pub double_tap_interval: f32,   // 双击间隔
    pub long_press_duration: f32,   // 长按时长
    pub swipe_min_distance: f32,    // 最小滑动距离
    pub swipe_max_angle: f32,       // 滑动角度容差
    
    // 多点触控参数
    pub max_touch_points: usize,    // 最大触摸点数
    pub touch_merge_distance: f32,  // 触摸点合并距离
    
    // 性能参数
    pub update_frequency: f32,      // 更新频率(Hz)
    pub event_history_size: usize,  // 事件历史大小
    
    // 过滤参数
    pub position_smoothing: f32,    // 位置平滑因子
    pub velocity_smoothing: f32,    // 速度平滑因子
    pub jitter_threshold: f32,      // 抖动阈值
}

// 触摸管理器
pub struct TouchManager {
    // 活跃触摸点
    active_touches: HashMap<u64, TouchPoint>,
    
    // 配置
    config: TouchConfig,
    
    // 手势识别状态
    gesture_recognizers: Vec<Box<dyn GestureRecognizer>>,
    current_gestures: Vec<GestureEvent>,
    
    // 事件历史
    touch_events: Vec<TouchEvent>,
    gesture_events: Vec<GestureEvent>,
    
    // 统计信息
    total_touches: u64,
    max_simultaneous_touches: usize,
    
    // 性能监控
    last_update: std::time::Instant,
    frame_times: Vec<f32>,
}

impl TouchManager {
    pub fn new() -> Self {
        let config = TouchConfig::default();
        let mut manager = Self {
            active_touches: HashMap::new(),
            config,
            gesture_recognizers: Vec::new(),
            current_gestures: Vec::new(),
            touch_events: Vec::new(),
            gesture_events: Vec::new(),
            total_touches: 0,
            max_simultaneous_touches: 0,
            last_update: std::time::Instant::now(),
            frame_times: Vec::new(),
        };
        
        manager.setup_default_recognizers();
        manager
    }
    
    // 更新触摸状态
    pub fn update(&mut self, delta_time: f32) {
        let current_time = std::time::Instant::now();
        
        // 性能监控
        let frame_time = current_time.duration_since(self.last_update).as_secs_f32();
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        self.last_update = current_time;
        
        // 更新触摸点
        self.update_touch_points(delta_time);
        
        // 运行手势识别
        self.run_gesture_recognition();
        
        // 清理过期事件
        self.cleanup_events();
        
        // 应用平滑滤波
        self.apply_smoothing();
    }
    
    // 处理触摸开始
    pub fn handle_touch_started(&mut self, id: u64, position: glam::Vec2, pressure: f32) {
        let current_time = std::time::Instant::now();
        
        // 检查是否需要合并相近的触摸点
        if let Some(existing_id) = self.find_nearby_touch(position, self.config.touch_merge_distance) {
            debug!("合并触摸点: {} -> {}", id, existing_id);
            return;
        }
        
        let touch_point = TouchPoint {
            id,
            position,
            start_position: position,
            previous_position: position,
            delta: glam::Vec2::ZERO,
            state: TouchState::Started,
            pressure,
            major_axis: 10.0, // 默认值
            minor_axis: 10.0,
            angle: 0.0,
            start_time: current_time,
            last_update: current_time,
        };
        
        self.active_touches.insert(id, touch_point.clone());
        self.total_touches += 1;
        self.max_simultaneous_touches = self.max_simultaneous_touches.max(self.active_touches.len());
        
        // 创建触摸事件
        let event = TouchEvent {
            touch_point,
            timestamp: current_time,
        };
        self.touch_events.push(event);
        
        debug!("触摸开始: ID={} 位置=({:.1}, {:.1}) 压力={:.2}", 
               id, position.x, position.y, pressure);
    }
    
    // 处理触摸移动
    pub fn handle_touch_moved(&mut self, id: u64, position: glam::Vec2, pressure: f32) {
        if let Some(touch_point) = self.active_touches.get_mut(&id) {
            let current_time = std::time::Instant::now();
            
            // 计算变化量
            let delta = position - touch_point.position;
            
            // 抖动过滤
            if delta.length() < self.config.jitter_threshold {
                return;
            }
            
            // 更新触摸点
            touch_point.previous_position = touch_point.position;
            touch_point.position = position;
            touch_point.delta = delta;
            touch_point.state = TouchState::Moving;
            touch_point.pressure = pressure;
            touch_point.last_update = current_time;
            
            // 创建事件
            let event = TouchEvent {
                touch_point: touch_point.clone(),
                timestamp: current_time,
            };
            self.touch_events.push(event);
        }
    }
    
    // 处理触摸结束
    pub fn handle_touch_ended(&mut self, id: u64, position: glam::Vec2) {
        if let Some(mut touch_point) = self.active_touches.remove(&id) {
            let current_time = std::time::Instant::now();
            
            touch_point.position = position;
            touch_point.state = TouchState::Ended;
            touch_point.last_update = current_time;
            
            let event = TouchEvent {
                touch_point,
                timestamp: current_time,
            };
            self.touch_events.push(event);
            
            debug!("触摸结束: ID={} 位置=({:.1}, {:.1})", id, position.x, position.y);
        }
    }
    
    // 处理触摸取消
    pub fn handle_touch_cancelled(&mut self, id: u64) {
        if let Some(mut touch_point) = self.active_touches.remove(&id) {
            touch_point.state = TouchState::Cancelled;
            touch_point.last_update = std::time::Instant::now();
            
            let event = TouchEvent {
                touch_point,
                timestamp: std::time::Instant::now(),
            };
            self.touch_events.push(event);
            
            debug!("触摸取消: ID={}", id);
        }
    }
    
    // 获取活跃触摸点
    pub fn get_active_touches(&self) -> Vec<&TouchPoint> {
        self.active_touches.values().collect()
    }
    
    // 获取触摸点
    pub fn get_touch(&self, id: u64) -> Option<&TouchPoint> {
        self.active_touches.get(&id)
    }
    
    // 获取触摸点数量
    pub fn get_touch_count(&self) -> usize {
        self.active_touches.len()
    }
    
    // 获取最近的手势事件
    pub fn get_recent_gesture_events(&self) -> &[GestureEvent] {
        &self.gesture_events
    }
    
    // 获取当前手势
    pub fn get_current_gestures(&self) -> &[GestureEvent] {
        &self.current_gestures
    }
    
    // 检查特定手势是否活跃
    pub fn is_gesture_active(&self, gesture_type: &GestureType) -> bool {
        self.current_gestures.iter().any(|g| &g.gesture_type == gesture_type)
    }
    
    // 获取触摸中心点
    pub fn get_touch_center(&self) -> Option<glam::Vec2> {
        if self.active_touches.is_empty() {
            return None;
        }
        
        let sum = self.active_touches.values()
            .fold(glam::Vec2::ZERO, |acc, touch| acc + touch.position);
        Some(sum / self.active_touches.len() as f32)
    }
    
    // 获取触摸范围
    pub fn get_touch_bounds(&self) -> Option<(glam::Vec2, glam::Vec2)> {
        if self.active_touches.is_empty() {
            return None;
        }
        
        let positions: Vec<glam::Vec2> = self.active_touches.values()
            .map(|t| t.position)
            .collect();
        
        let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = positions.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = positions.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        
        Some((glam::Vec2::new(min_x, min_y), glam::Vec2::new(max_x, max_y)))
    }
    
    // 配置管理
    pub fn set_config(&mut self, config: TouchConfig) {
        self.config = config;
    }
    
    pub fn get_config(&self) -> &TouchConfig {
        &self.config
    }
    
    // 添加手势识别器
    pub fn add_gesture_recognizer(&mut self, recognizer: Box<dyn GestureRecognizer>) {
        self.gesture_recognizers.push(recognizer);
    }
    
    // 获取性能统计
    pub fn get_performance_stats(&self) -> TouchPerformanceStats {
        let avg_frame_time = if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        };
        
        TouchPerformanceStats {
            total_touches: self.total_touches,
            active_touches: self.active_touches.len(),
            max_simultaneous_touches: self.max_simultaneous_touches,
            average_frame_time: avg_frame_time,
            events_per_second: self.touch_events.len() as f32 / avg_frame_time.max(0.001),
        }
    }
    
    // 清除所有状态
    pub fn clear(&mut self) {
        self.active_touches.clear();
        self.current_gestures.clear();
        self.touch_events.clear();
        self.gesture_events.clear();
    }
    
    // 私有方法
    fn setup_default_recognizers(&mut self) {
        // 基础手势识别器
        self.add_gesture_recognizer(Box::new(TapRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(PanRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(PinchRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(SwipeRecognizer::new(self.config.clone())));
    }
    
    fn update_touch_points(&mut self, _delta_time: f32) {
        let current_time = std::time::Instant::now();
        
        for touch in self.active_touches.values_mut() {
            // 更新速度计算等
            let time_diff = current_time.duration_since(touch.last_update).as_secs_f32();
            if time_diff > 0.0 {
                // 计算速度等其他属性
            }
        }
    }
    
    fn run_gesture_recognition(&mut self) {
        self.current_gestures.clear();
        
        for recognizer in &mut self.gesture_recognizers {
            if let Some(gesture) = recognizer.recognize(&self.active_touches, &self.touch_events) {
                self.current_gestures.push(gesture.clone());
                self.gesture_events.push(gesture);
            }
        }
    }
    
    fn cleanup_events(&mut self) {
        let max_age = std::time::Duration::from_secs(5);
        let current_time = std::time::Instant::now();
        
        self.touch_events.retain(|event| {
            current_time.duration_since(event.timestamp) < max_age
        });
        
        self.gesture_events.retain(|event| {
            current_time.duration_since(event.timestamp) < max_age
        });
        
        if self.touch_events.len() > self.config.event_history_size {
            let excess = self.touch_events.len() - self.config.event_history_size;
            self.touch_events.drain(0..excess);
        }
    }
    
    fn apply_smoothing(&mut self) {
        // 应用位置和速度平滑
        for touch in self.active_touches.values_mut() {
            // 简单的平滑滤波实现
            // 实际项目中可能需要更复杂的滤波算法
        }
    }
    
    fn find_nearby_touch(&self, position: glam::Vec2, max_distance: f32) -> Option<u64> {
        self.active_touches.iter()
            .find(|(_, touch)| (touch.position - position).length() <= max_distance)
            .map(|(&id, _)| id)
    }
}

// 手势识别器接口
pub trait GestureRecognizer {
    fn recognize(
        &mut self,
        active_touches: &HashMap<u64, TouchPoint>,
        touch_events: &[TouchEvent],
    ) -> Option<GestureEvent>;
    
    fn reset(&mut self);
}

// 点击手势识别器
struct TapRecognizer {
    config: TouchConfig,
    last_tap_time: Option<std::time::Instant>,
    last_tap_position: Option<glam::Vec2>,
}

impl TapRecognizer {
    fn new(config: TouchConfig) -> Self {
        Self {
            config,
            last_tap_time: None,
            last_tap_position: None,
        }
    }
}

impl GestureRecognizer for TapRecognizer {
    fn recognize(
        &mut self,
        _active_touches: &HashMap<u64, TouchPoint>,
        touch_events: &[TouchEvent],
    ) -> Option<GestureEvent> {
        // 查找刚结束的触摸点
        for event in touch_events.iter().rev() {
            if event.touch_point.state == TouchState::Ended {
                let duration = event.timestamp.duration_since(event.touch_point.start_time).as_secs_f32();
                let distance = (event.touch_point.position - event.touch_point.start_position).length();
                
                if duration <= self.config.tap_max_duration && distance <= self.config.tap_max_distance {
                    let gesture_type = if let (Some(last_time), Some(last_pos)) = (self.last_tap_time, self.last_tap_position) {
                        let time_diff = event.timestamp.duration_since(last_time).as_secs_f32();
                        let pos_diff = (event.touch_point.position - last_pos).length();
                        
                        if time_diff <= self.config.double_tap_interval && pos_diff <= self.config.tap_max_distance {
                            GestureType::DoubleTap
                        } else {
                            GestureType::Tap
                        }
                    } else {
                        GestureType::Tap
                    };
                    
                    self.last_tap_time = Some(event.timestamp);
                    self.last_tap_position = Some(event.touch_point.position);
                    
                    return Some(GestureEvent {
                        gesture_type,
                        position: event.touch_point.position,
                        delta: glam::Vec2::ZERO,
                        scale: 1.0,
                        rotation: 0.0,
                        velocity: glam::Vec2::ZERO,
                        duration,
                        touch_points: vec![event.touch_point.id],
                        timestamp: event.timestamp,
                    });
                }
            }
        }
        
        None
    }
    
    fn reset(&mut self) {
        self.last_tap_time = None;
        self.last_tap_position = None;
    }
}

// 其他手势识别器的简化实现
struct PanRecognizer { config: TouchConfig }
struct PinchRecognizer { config: TouchConfig }
struct SwipeRecognizer { config: TouchConfig }

impl PanRecognizer {
    fn new(config: TouchConfig) -> Self { Self { config } }
}
impl PinchRecognizer {
    fn new(config: TouchConfig) -> Self { Self { config } }
}
impl SwipeRecognizer {
    fn new(config: TouchConfig) -> Self { Self { config } }
}

impl GestureRecognizer for PanRecognizer {
    fn recognize(&mut self, _active_touches: &HashMap<u64, TouchPoint>, _touch_events: &[TouchEvent]) -> Option<GestureEvent> { None }
    fn reset(&mut self) {}
}

impl GestureRecognizer for PinchRecognizer {
    fn recognize(&mut self, _active_touches: &HashMap<u64, TouchPoint>, _touch_events: &[TouchEvent]) -> Option<GestureEvent> { None }
    fn reset(&mut self) {}
}

impl GestureRecognizer for SwipeRecognizer {
    fn recognize(&mut self, _active_touches: &HashMap<u64, TouchPoint>, _touch_events: &[TouchEvent]) -> Option<GestureEvent> { None }
    fn reset(&mut self) {}
}

// 性能统计
#[derive(Debug, Clone)]
pub struct TouchPerformanceStats {
    pub total_touches: u64,
    pub active_touches: usize,
    pub max_simultaneous_touches: usize,
    pub average_frame_time: f32,
    pub events_per_second: f32,
}

// 默认配置
impl Default for TouchConfig {
    fn default() -> Self {
        Self {
            tap_max_duration: 0.3,
            tap_max_distance: 20.0,
            double_tap_interval: 0.4,
            long_press_duration: 1.0,
            swipe_min_distance: 50.0,
            swipe_max_angle: 0.5, // 约30度
            max_touch_points: 10,
            touch_merge_distance: 30.0,
            update_frequency: 60.0,
            event_history_size: 1000,
            position_smoothing: 0.1,
            velocity_smoothing: 0.2,
            jitter_threshold: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_touch_manager_creation() {
        let manager = TouchManager::new();
        assert_eq!(manager.get_touch_count(), 0);
    }
    
    #[test]
    fn test_touch_start_end() {
        let mut manager = TouchManager::new();
        let position = glam::Vec2::new(100.0, 200.0);
        
        manager.handle_touch_started(1, position, 1.0);
        assert_eq!(manager.get_touch_count(), 1);
        
        let touch = manager.get_touch(1).unwrap();
        assert_eq!(touch.position, position);
        assert_eq!(touch.state, TouchState::Started);
        
        manager.handle_touch_ended(1, position);
        assert_eq!(manager.get_touch_count(), 0);
    }
    
    #[test]
    fn test_touch_movement() {
        let mut manager = TouchManager::new();
        let start_pos = glam::Vec2::new(10.0, 10.0);
        let end_pos = glam::Vec2::new(50.0, 50.0);
        
        manager.handle_touch_started(1, start_pos, 1.0);
        manager.handle_touch_moved(1, end_pos, 1.0);
        
        let touch = manager.get_touch(1).unwrap();
        assert_eq!(touch.position, end_pos);
        assert_eq!(touch.state, TouchState::Moving);
        assert_eq!(touch.previous_position, start_pos);
    }
    
    #[test]
    fn test_multi_touch() {
        let mut manager = TouchManager::new();
        
        manager.handle_touch_started(1, glam::Vec2::new(0.0, 0.0), 1.0);
        manager.handle_touch_started(2, glam::Vec2::new(100.0, 100.0), 0.8);
        
        assert_eq!(manager.get_touch_count(), 2);
        
        let center = manager.get_touch_center().unwrap();
        assert_eq!(center, glam::Vec2::new(50.0, 50.0));
    }
}