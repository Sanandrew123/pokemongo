/*
 * 输入管理系统 - Input Manager System
 * 
 * 开发心理过程：
 * 设计统一的输入处理系统，支持键盘、鼠标、手柄等多种输入设备
 * 需要考虑输入映射、手势识别、输入缓冲和响应优化
 * 重点关注用户体验和输入延迟的最小化
 */

use bevy::prelude::*;
use bevy::input::gamepad::*;
use bevy::input::touch::*;
use bevy::window::CursorMoved;
use std::collections::{HashMap, VecDeque};
use crate::core::error::{GameResult, GameError};

// 输入设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDevice {
    Keyboard,
    Mouse,
    Gamepad(usize),
    Touch,
}

// 输入状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputState {
    Pressed,
    JustPressed,
    JustReleased,
    Released,
}

// 虚拟按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualButton {
    // 方向控制
    Up,
    Down,
    Left,
    Right,
    
    // 动作按键
    Confirm,    // 确认/A键
    Cancel,     // 取消/B键
    Menu,       // 菜单/Start
    Option,     // 选项/Select
    
    // 功能按键
    Action1,    // 动作1/X键
    Action2,    // 动作2/Y键
    Shoulder1,  // 左肩键/L1
    Shoulder2,  // 右肩键/R1
    Trigger1,   // 左扳机/L2
    Trigger2,   // 右扳机/R2
    
    // 系统按键
    Pause,
    Screenshot,
    Fullscreen,
}

// 手势类型
#[derive(Debug, Clone)]
pub enum GestureType {
    Tap { position: Vec2 },
    DoubleTap { position: Vec2 },
    LongPress { position: Vec2, duration: f32 },
    Swipe { start: Vec2, end: Vec2, direction: SwipeDirection },
    Pinch { center: Vec2, scale: f32 },
    Pan { start: Vec2, current: Vec2 },
}

// 滑动方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

// 输入事件
#[derive(Debug, Clone)]
pub enum InputEvent {
    VirtualButtonPressed(VirtualButton),
    VirtualButtonReleased(VirtualButton),
    MouseMoved(Vec2),
    MouseScrolled(Vec2),
    TouchGesture(GestureType),
    GamepadConnected(usize),
    GamepadDisconnected(usize),
}

// 输入映射配置
#[derive(Debug, Clone)]
pub struct InputMapping {
    pub keyboard_mappings: HashMap<KeyCode, VirtualButton>,
    pub mouse_mappings: HashMap<MouseButton, VirtualButton>,
    pub gamepad_mappings: HashMap<GamepadButtonType, VirtualButton>,
    pub touch_gestures_enabled: bool,
}

impl Default for InputMapping {
    fn default() -> Self {
        let mut keyboard_mappings = HashMap::new();
        keyboard_mappings.insert(KeyCode::W, VirtualButton::Up);
        keyboard_mappings.insert(KeyCode::S, VirtualButton::Down);
        keyboard_mappings.insert(KeyCode::A, VirtualButton::Left);
        keyboard_mappings.insert(KeyCode::D, VirtualButton::Right);
        keyboard_mappings.insert(KeyCode::ArrowUp, VirtualButton::Up);
        keyboard_mappings.insert(KeyCode::ArrowDown, VirtualButton::Down);
        keyboard_mappings.insert(KeyCode::ArrowLeft, VirtualButton::Left);
        keyboard_mappings.insert(KeyCode::ArrowRight, VirtualButton::Right);
        keyboard_mappings.insert(KeyCode::Return, VirtualButton::Confirm);
        keyboard_mappings.insert(KeyCode::Space, VirtualButton::Confirm);
        keyboard_mappings.insert(KeyCode::Escape, VirtualButton::Cancel);
        keyboard_mappings.insert(KeyCode::Tab, VirtualButton::Menu);
        keyboard_mappings.insert(KeyCode::F11, VirtualButton::Fullscreen);
        keyboard_mappings.insert(KeyCode::F12, VirtualButton::Screenshot);
        keyboard_mappings.insert(KeyCode::P, VirtualButton::Pause);

        let mut mouse_mappings = HashMap::new();
        mouse_mappings.insert(MouseButton::Left, VirtualButton::Confirm);
        mouse_mappings.insert(MouseButton::Right, VirtualButton::Cancel);
        mouse_mappings.insert(MouseButton::Middle, VirtualButton::Menu);

        let mut gamepad_mappings = HashMap::new();
        gamepad_mappings.insert(GamepadButtonType::South, VirtualButton::Confirm);
        gamepad_mappings.insert(GamepadButtonType::East, VirtualButton::Cancel);
        gamepad_mappings.insert(GamepadButtonType::West, VirtualButton::Action1);
        gamepad_mappings.insert(GamepadButtonType::North, VirtualButton::Action2);
        gamepad_mappings.insert(GamepadButtonType::Start, VirtualButton::Menu);
        gamepad_mappings.insert(GamepadButtonType::Select, VirtualButton::Option);
        gamepad_mappings.insert(GamepadButtonType::LeftTrigger, VirtualButton::Shoulder1);
        gamepad_mappings.insert(GamepadButtonType::RightTrigger, VirtualButton::Shoulder2);
        gamepad_mappings.insert(GamepadButtonType::LeftTrigger2, VirtualButton::Trigger1);
        gamepad_mappings.insert(GamepadButtonType::RightTrigger2, VirtualButton::Trigger2);

        Self {
            keyboard_mappings,
            mouse_mappings,
            gamepad_mappings,
            touch_gestures_enabled: true,
        }
    }
}

// 触摸点信息
#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,
    pub position: Vec2,
    pub pressure: f32,
    pub timestamp: f64,
}

// 手势识别器
#[derive(Debug)]
pub struct GestureRecognizer {
    touch_points: HashMap<u64, TouchPoint>,
    gesture_history: VecDeque<GestureType>,
    tap_threshold: f32,
    swipe_threshold: f32,
    long_press_duration: f32,
    double_tap_interval: f32,
    last_tap_time: f64,
    last_tap_position: Option<Vec2>,
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self {
            touch_points: HashMap::new(),
            gesture_history: VecDeque::with_capacity(10),
            tap_threshold: 10.0,
            swipe_threshold: 50.0,
            long_press_duration: 0.5,
            double_tap_interval: 0.3,
            last_tap_time: 0.0,
            last_tap_position: None,
        }
    }
}

// 输入缓冲区
#[derive(Debug)]
pub struct InputBuffer {
    button_states: HashMap<VirtualButton, InputState>,
    button_events: VecDeque<InputEvent>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    scroll_delta: Vec2,
    connected_gamepads: Vec<usize>,
    max_buffer_size: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            button_states: HashMap::new(),
            button_events: VecDeque::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            connected_gamepads: Vec::new(),
            max_buffer_size: 100,
        }
    }
}

// 输入管理器主结构
pub struct InputManager {
    mapping: InputMapping,
    buffer: InputBuffer,
    gesture_recognizer: GestureRecognizer,
    input_sensitivity: f32,
    deadzone_threshold: f32,
    enable_input: bool,
    debug_input: bool,
}

impl InputManager {
    // 创建新的输入管理器
    pub fn new() -> GameResult<Self> {
        Ok(Self {
            mapping: InputMapping::default(),
            buffer: InputBuffer::default(),
            gesture_recognizer: GestureRecognizer::default(),
            input_sensitivity: 1.0,
            deadzone_threshold: 0.1,
            enable_input: true,
            debug_input: false,
        })
    }

    // 初始化输入管理器
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化输入管理器");
        
        // 初始化所有虚拟按键状态
        for &button in [
            VirtualButton::Up, VirtualButton::Down, VirtualButton::Left, VirtualButton::Right,
            VirtualButton::Confirm, VirtualButton::Cancel, VirtualButton::Menu, VirtualButton::Option,
            VirtualButton::Action1, VirtualButton::Action2,
            VirtualButton::Shoulder1, VirtualButton::Shoulder2,
            VirtualButton::Trigger1, VirtualButton::Trigger2,
            VirtualButton::Pause, VirtualButton::Screenshot, VirtualButton::Fullscreen,
        ].iter() {
            self.buffer.button_states.insert(button, InputState::Released);
        }

        Ok(())
    }

    // 关闭输入管理器
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("关闭输入管理器");
        
        self.buffer.button_states.clear();
        self.buffer.button_events.clear();
        self.buffer.connected_gamepads.clear();
        self.gesture_recognizer.touch_points.clear();
        
        Ok(())
    }

    // 更新输入状态
    pub fn update(&mut self, _delta_time: f32) -> GameResult<()> {
        if !self.enable_input {
            return Ok(());
        }

        // 更新按键状态（JustPressed -> Pressed, JustReleased -> Released）
        for state in self.buffer.button_states.values_mut() {
            match *state {
                InputState::JustPressed => *state = InputState::Pressed,
                InputState::JustReleased => *state = InputState::Released,
                _ => {}
            }
        }

        // 重置鼠标增量
        self.buffer.mouse_delta = Vec2::ZERO;
        self.buffer.scroll_delta = Vec2::ZERO;

        // 清理过期的手势历史
        if self.gesture_recognizer.gesture_history.len() > 5 {
            self.gesture_recognizer.gesture_history.pop_front();
        }

        Ok(())
    }

    // 处理键盘输入
    pub fn handle_keyboard_input(&mut self, input: &Input<KeyCode>) {
        if !self.enable_input {
            return;
        }

        for (&key_code, &virtual_button) in &self.mapping.keyboard_mappings {
            let new_state = if input.just_pressed(key_code) {
                self.send_event(InputEvent::VirtualButtonPressed(virtual_button));
                InputState::JustPressed
            } else if input.just_released(key_code) {
                self.send_event(InputEvent::VirtualButtonReleased(virtual_button));
                InputState::JustReleased
            } else if input.pressed(key_code) {
                InputState::Pressed
            } else {
                InputState::Released
            };

            if let Some(current_state) = self.buffer.button_states.get(&virtual_button) {
                if *current_state != new_state {
                    self.buffer.button_states.insert(virtual_button, new_state);
                }
            }
        }
    }

    // 处理鼠标输入
    pub fn handle_mouse_input(&mut self, 
        buttons: &Input<MouseButton>,
        cursor_moved: &mut EventReader<CursorMoved>
    ) {
        if !self.enable_input {
            return;
        }

        // 处理鼠标按键
        for (&mouse_button, &virtual_button) in &self.mapping.mouse_mappings {
            let new_state = if buttons.just_pressed(mouse_button) {
                self.send_event(InputEvent::VirtualButtonPressed(virtual_button));
                InputState::JustPressed
            } else if buttons.just_released(mouse_button) {
                self.send_event(InputEvent::VirtualButtonReleased(virtual_button));
                InputState::JustReleased
            } else if buttons.pressed(mouse_button) {
                InputState::Pressed
            } else {
                InputState::Released
            };

            self.buffer.button_states.insert(virtual_button, new_state);
        }

        // 处理鼠标移动
        for event in cursor_moved.iter() {
            let new_position = event.position;
            self.buffer.mouse_delta = new_position - self.buffer.mouse_position;
            self.buffer.mouse_position = new_position;
            self.send_event(InputEvent::MouseMoved(new_position));
        }
    }

    // 处理手柄输入
    pub fn handle_gamepad_input(&mut self, 
        gamepads: &Res<Gamepads>,
        button_inputs: &Res<Input<GamepadButton>>,
        axes: &Res<Axis<GamepadAxis>>,
        gamepad_events: &mut EventReader<GamepadEvent>
    ) {
        if !self.enable_input {
            return;
        }

        // 处理手柄连接/断开事件
        for event in gamepad_events.iter() {
            match event {
                GamepadEvent::Connection(gamepad_event) => {
                    match gamepad_event.connection {
                        GamepadConnection::Connected(info) => {
                            let gamepad_id = gamepad_event.gamepad.id;
                            self.buffer.connected_gamepads.push(gamepad_id);
                            self.send_event(InputEvent::GamepadConnected(gamepad_id));
                            info!("手柄已连接: {} ({})", gamepad_id, info.name);
                        }
                        GamepadConnection::Disconnected => {
                            let gamepad_id = gamepad_event.gamepad.id;
                            self.buffer.connected_gamepads.retain(|&id| id != gamepad_id);
                            self.send_event(InputEvent::GamepadDisconnected(gamepad_id));
                            info!("手柄已断开: {}", gamepad_id);
                        }
                    }
                }
                _ => {}
            }
        }

        // 处理手柄按键
        for gamepad in gamepads.iter() {
            for (&gamepad_button_type, &virtual_button) in &self.mapping.gamepad_mappings {
                let button = GamepadButton::new(gamepad, gamepad_button_type);
                
                let new_state = if button_inputs.just_pressed(button) {
                    self.send_event(InputEvent::VirtualButtonPressed(virtual_button));
                    InputState::JustPressed
                } else if button_inputs.just_released(button) {
                    self.send_event(InputEvent::VirtualButtonReleased(virtual_button));
                    InputState::JustReleased
                } else if button_inputs.pressed(button) {
                    InputState::Pressed
                } else {
                    InputState::Released
                };

                if let Some(current_state) = self.buffer.button_states.get(&virtual_button) {
                    if *current_state != new_state {
                        self.buffer.button_states.insert(virtual_button, new_state);
                    }
                }
            }

            // 处理模拟摇杆
            self.handle_gamepad_axes(gamepad, axes);
        }
    }

    // 处理手柄摇杆
    fn handle_gamepad_axes(&mut self, gamepad: Gamepad, axes: &Res<Axis<GamepadAxis>>) {
        // 左摇杆
        let left_stick_x = axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX)).unwrap_or(0.0);
        let left_stick_y = axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY)).unwrap_or(0.0);

        // 应用死区
        let left_magnitude = (left_stick_x * left_stick_x + left_stick_y * left_stick_y).sqrt();
        if left_magnitude > self.deadzone_threshold {
            // 处理方向输入
            if left_stick_x.abs() > left_stick_y.abs() {
                if left_stick_x > self.deadzone_threshold {
                    self.buffer.button_states.insert(VirtualButton::Right, InputState::Pressed);
                } else if left_stick_x < -self.deadzone_threshold {
                    self.buffer.button_states.insert(VirtualButton::Left, InputState::Pressed);
                }
            } else {
                if left_stick_y > self.deadzone_threshold {
                    self.buffer.button_states.insert(VirtualButton::Up, InputState::Pressed);
                } else if left_stick_y < -self.deadzone_threshold {
                    self.buffer.button_states.insert(VirtualButton::Down, InputState::Pressed);
                }
            }
        }

        // 扳机
        let left_trigger = axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftZ)).unwrap_or(0.0);
        let right_trigger = axes.get(GamepadAxis::new(gamepad, GamepadAxisType::RightZ)).unwrap_or(0.0);

        if left_trigger > self.deadzone_threshold {
            self.buffer.button_states.insert(VirtualButton::Trigger1, InputState::Pressed);
        }
        if right_trigger > self.deadzone_threshold {
            self.buffer.button_states.insert(VirtualButton::Trigger2, InputState::Pressed);
        }
    }

    // 处理触摸输入
    pub fn handle_touch_input(&mut self, touch_events: &mut EventReader<TouchInput>, time: f64) {
        if !self.enable_input || !self.mapping.touch_gestures_enabled {
            return;
        }

        for event in touch_events.iter() {
            match event.phase {
                TouchPhase::Started => {
                    let touch_point = TouchPoint {
                        id: event.id,
                        position: event.position,
                        pressure: event.force.unwrap_or(1.0),
                        timestamp: time,
                    };
                    self.gesture_recognizer.touch_points.insert(event.id, touch_point);
                }
                TouchPhase::Moved => {
                    if let Some(touch_point) = self.gesture_recognizer.touch_points.get_mut(&event.id) {
                        let old_position = touch_point.position;
                        touch_point.position = event.position;
                        touch_point.timestamp = time;

                        // 检测拖拽手势
                        let gesture = GestureType::Pan {
                            start: old_position,
                            current: event.position,
                        };
                        self.send_gesture_event(gesture);
                    }
                }
                TouchPhase::Ended => {
                    if let Some(touch_point) = self.gesture_recognizer.touch_points.remove(&event.id) {
                        let duration = (time - touch_point.timestamp) as f32;
                        let distance = (event.position - touch_point.position).length();

                        if distance < self.gesture_recognizer.tap_threshold {
                            if duration > self.gesture_recognizer.long_press_duration {
                                // 长按
                                let gesture = GestureType::LongPress {
                                    position: event.position,
                                    duration,
                                };
                                self.send_gesture_event(gesture);
                            } else {
                                // 检测双击
                                let is_double_tap = if let Some(last_pos) = self.gesture_recognizer.last_tap_position {
                                    time - self.gesture_recognizer.last_tap_time < self.gesture_recognizer.double_tap_interval as f64 &&
                                    (event.position - last_pos).length() < self.gesture_recognizer.tap_threshold
                                } else {
                                    false
                                };

                                if is_double_tap {
                                    let gesture = GestureType::DoubleTap {
                                        position: event.position,
                                    };
                                    self.send_gesture_event(gesture);
                                    self.gesture_recognizer.last_tap_position = None;
                                } else {
                                    let gesture = GestureType::Tap {
                                        position: event.position,
                                    };
                                    self.send_gesture_event(gesture);
                                    self.gesture_recognizer.last_tap_time = time;
                                    self.gesture_recognizer.last_tap_position = Some(event.position);
                                }
                            }
                        } else if distance > self.gesture_recognizer.swipe_threshold {
                            // 滑动手势
                            let direction = self.determine_swipe_direction(touch_point.position, event.position);
                            let gesture = GestureType::Swipe {
                                start: touch_point.position,
                                end: event.position,
                                direction,
                            };
                            self.send_gesture_event(gesture);
                        }
                    }
                }
                TouchPhase::Cancelled => {
                    self.gesture_recognizer.touch_points.remove(&event.id);
                }
            }
        }
    }

    // 确定滑动方向
    fn determine_swipe_direction(&self, start: Vec2, end: Vec2) -> SwipeDirection {
        let delta = end - start;
        if delta.x.abs() > delta.y.abs() {
            if delta.x > 0.0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            }
        } else {
            if delta.y > 0.0 {
                SwipeDirection::Up
            } else {
                SwipeDirection::Down
            }
        }
    }

    // 发送输入事件
    fn send_event(&mut self, event: InputEvent) {
        if self.buffer.button_events.len() >= self.buffer.max_buffer_size {
            self.buffer.button_events.pop_front();
        }
        self.buffer.button_events.push_back(event);

        if self.debug_input {
            debug!("输入事件: {:?}", event);
        }
    }

    // 发送手势事件
    fn send_gesture_event(&mut self, gesture: GestureType) {
        self.gesture_recognizer.gesture_history.push_back(gesture.clone());
        self.send_event(InputEvent::TouchGesture(gesture));
    }

    // 查询虚拟按键状态
    pub fn is_button_pressed(&self, button: VirtualButton) -> bool {
        matches!(
            self.buffer.button_states.get(&button),
            Some(InputState::Pressed) | Some(InputState::JustPressed)
        )
    }

    pub fn is_button_just_pressed(&self, button: VirtualButton) -> bool {
        matches!(
            self.buffer.button_states.get(&button),
            Some(InputState::JustPressed)
        )
    }

    pub fn is_button_just_released(&self, button: VirtualButton) -> bool {
        matches!(
            self.buffer.button_states.get(&button),
            Some(InputState::JustReleased)
        )
    }

    // 获取鼠标状态
    pub fn get_mouse_position(&self) -> Vec2 {
        self.buffer.mouse_position
    }

    pub fn get_mouse_delta(&self) -> Vec2 {
        self.buffer.mouse_delta
    }

    pub fn get_scroll_delta(&self) -> Vec2 {
        self.buffer.scroll_delta
    }

    // 获取输入事件
    pub fn get_events(&mut self) -> impl Iterator<Item = InputEvent> + '_ {
        self.buffer.button_events.drain(..)
    }

    // 设置输入映射
    pub fn set_input_mapping(&mut self, mapping: InputMapping) {
        self.mapping = mapping;
    }

    // 设置输入敏感度
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.input_sensitivity = sensitivity.max(0.1).min(5.0);
    }

    // 设置死区阈值
    pub fn set_deadzone(&mut self, threshold: f32) {
        self.deadzone_threshold = threshold.max(0.0).min(0.9);
    }

    // 启用/禁用输入
    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.enable_input = enabled;
        if !enabled {
            // 清空所有按键状态
            for state in self.buffer.button_states.values_mut() {
                *state = InputState::Released;
            }
        }
    }

    // 启用/禁用调试输出
    pub fn set_debug_input(&mut self, debug: bool) {
        self.debug_input = debug;
    }

    // 获取连接的手柄列表
    pub fn get_connected_gamepads(&self) -> &[usize] {
        &self.buffer.connected_gamepads
    }

    // 清空输入缓冲区
    pub fn clear_input_buffer(&mut self) {
        self.buffer.button_events.clear();
        for state in self.buffer.button_states.values_mut() {
            if matches!(*state, InputState::JustPressed | InputState::JustReleased) {
                *state = InputState::Released;
            }
        }
    }

    // 获取手势历史
    pub fn get_recent_gestures(&self) -> &VecDeque<GestureType> {
        &self.gesture_recognizer.gesture_history
    }

    // 检测组合键
    pub fn is_key_combo_pressed(&self, buttons: &[VirtualButton]) -> bool {
        buttons.iter().all(|&button| self.is_button_pressed(button))
    }

    // 获取方向输入向量
    pub fn get_movement_vector(&self) -> Vec2 {
        let mut movement = Vec2::ZERO;

        if self.is_button_pressed(VirtualButton::Up) {
            movement.y += 1.0;
        }
        if self.is_button_pressed(VirtualButton::Down) {
            movement.y -= 1.0;
        }
        if self.is_button_pressed(VirtualButton::Left) {
            movement.x -= 1.0;
        }
        if self.is_button_pressed(VirtualButton::Right) {
            movement.x += 1.0;
        }

        // 标准化对角线移动
        if movement.length() > 0.0 {
            movement = movement.normalize() * self.input_sensitivity;
        }

        movement
    }
}

// Bevy系统实现
pub fn input_system(
    mut input_manager: ResMut<InputManager>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut cursor_moved: EventReader<CursorMoved>,
    gamepads: Res<Gamepads>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    mut gamepad_events: EventReader<GamepadEvent>,
    mut touch_events: EventReader<TouchInput>,
    time: Res<Time>,
) {
    // 处理各种输入
    input_manager.handle_keyboard_input(&keyboard_input);
    input_manager.handle_mouse_input(&mouse_input, &mut cursor_moved);
    input_manager.handle_gamepad_input(&gamepads, &gamepad_buttons, &gamepad_axes, &mut gamepad_events);
    input_manager.handle_touch_input(&mut touch_events, time.elapsed_seconds_f64());
    
    // 更新输入管理器
    let _ = input_manager.update(time.delta_seconds());
}

// 便捷函数
impl InputManager {
    pub fn create_custom_mapping() -> InputMapping {
        let mut mapping = InputMapping::default();
        
        // 可以在这里自定义按键映射
        mapping.keyboard_mappings.insert(KeyCode::J, VirtualButton::Confirm);
        mapping.keyboard_mappings.insert(KeyCode::K, VirtualButton::Cancel);
        
        mapping
    }

    pub fn is_any_key_pressed(&self) -> bool {
        self.buffer.button_states.values().any(|&state| {
            matches!(state, InputState::Pressed | InputState::JustPressed)
        })
    }

    pub fn get_pressed_buttons(&self) -> Vec<VirtualButton> {
        self.buffer.button_states.iter()
            .filter_map(|(&button, &state)| {
                if matches!(state, InputState::Pressed | InputState::JustPressed) {
                    Some(button)
                } else {
                    None
                }
            })
            .collect()
    }
}