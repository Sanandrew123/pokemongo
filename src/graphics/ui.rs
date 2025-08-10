// UI系统
// 开发心理：UI是玩家交互的主要界面，需要响应式布局、事件处理、样式系统
// 设计原则：组件化设计、灵活布局、事件冒泡、样式继承

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::renderer::Renderer2D;
use crate::input::mouse::MouseButton;
use glam::{Vec2, Vec4};

// UI元素ID类型
pub type ElementId = u32;

// UI事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum UIEvent {
    Click { position: Vec2, button: MouseButton },
    Hover { position: Vec2, entered: bool },
    KeyPress { key: String, modifiers: KeyModifiers },
    Focus { gained: bool },
    ValueChanged { old_value: String, new_value: String },
    Scroll { delta: Vec2 },
    Resize { old_size: Vec2, new_size: Vec2 },
    Custom { event_type: String, data: HashMap<String, String> },
}

// 键盘修饰键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

// UI元素类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Panel,          // 面板容器
    Button,         // 按钮
    Label,          // 文本标签
    TextInput,      // 文本输入框
    Image,          // 图片
    ScrollView,     // 滚动视图
    ListView,       // 列表视图
    ProgressBar,    // 进度条
    Slider,         // 滑动条
    Toggle,         // 开关
    Dropdown,       // 下拉菜单
    Custom,         // 自定义元素
}

// 布局类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    None,           // 无布局
    Horizontal,     // 水平布局
    Vertical,       // 垂直布局
    Grid,           // 网格布局
    Absolute,       // 绝对定位
    Relative,       // 相对定位
    Flex,           // 弹性布局
}

// 对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Start,          // 开始位置
    Center,         // 居中
    End,            // 结束位置
    Stretch,        // 拉伸
    SpaceBetween,   // 空间均分
    SpaceAround,    // 周围留空
}

// 尺寸单位
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SizeUnit {
    Pixels(f32),        // 像素
    Percent(f32),       // 百分比
    Auto,               // 自动
    Fill,               // 填充
}

// 边距/内边距
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

// UI样式
#[derive(Debug, Clone)]
pub struct UIStyle {
    // 尺寸
    pub width: SizeUnit,
    pub height: SizeUnit,
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
    
    // 边距
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    
    // 背景
    pub background_color: Vec4,
    pub background_image: Option<u32>,      // 纹理ID
    
    // 边框
    pub border_width: f32,
    pub border_color: Vec4,
    pub border_radius: f32,
    
    // 文本样式
    pub font_size: f32,
    pub font_color: Vec4,
    pub text_align: Alignment,
    pub line_height: f32,
    
    // 阴影
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
    pub shadow_color: Vec4,
    
    // 透明度
    pub opacity: f32,
    
    // 可见性
    pub visible: bool,
    
    // Z索引
    pub z_index: i32,
}

// UI元素状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    Normal,         // 正常
    Hovered,        // 悬停
    Pressed,        // 按下
    Focused,        // 聚焦
    Disabled,       // 禁用
    Selected,       // 选中
}

// UI元素
#[derive(Debug, Clone)]
pub struct UIElement {
    pub id: ElementId,
    pub name: String,
    pub element_type: ElementType,
    
    // 层次结构
    pub parent: Option<ElementId>,
    pub children: Vec<ElementId>,
    
    // 位置和尺寸
    pub position: Vec2,
    pub size: Vec2,
    pub calculated_size: Vec2,      // 布局计算后的实际尺寸
    pub content_size: Vec2,         // 内容尺寸
    
    // 样式
    pub style: UIStyle,
    pub state_styles: HashMap<ElementState, UIStyle>, // 不同状态的样式
    
    // 状态
    pub state: ElementState,
    pub enabled: bool,
    pub focusable: bool,
    
    // 布局
    pub layout_type: LayoutType,
    
    // 内容
    pub text: String,
    pub value: String,              // 通用值字段
    
    // 事件处理
    pub event_handlers: HashMap<String, Box<dyn Fn(&UIEvent) + Send>>,
    
    // 自定义数据
    pub user_data: HashMap<String, String>,
    
    // 动画
    pub animation_time: f32,
    pub target_position: Option<Vec2>,
    pub target_size: Option<Vec2>,
    pub animation_duration: f32,
}

// UI布局信息
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub element_id: ElementId,
    pub position: Vec2,
    pub size: Vec2,
    pub content_rect: (Vec2, Vec2),     // (position, size) 内容区域
    pub margin_rect: (Vec2, Vec2),      // (position, size) 外边距区域
    pub padding_rect: (Vec2, Vec2),     // (position, size) 内边距区域
}

// UI管理器
pub struct UIManager {
    // 元素管理
    elements: HashMap<ElementId, UIElement>,
    next_element_id: ElementId,
    root_elements: Vec<ElementId>,      // 根元素列表
    
    // 焦点管理
    focused_element: Option<ElementId>,
    focus_chain: Vec<ElementId>,        // 焦点链
    
    // 事件管理
    hovered_element: Option<ElementId>,
    pressed_elements: HashMap<MouseButton, ElementId>,
    
    // 布局系统
    layout_dirty: bool,
    layout_cache: HashMap<ElementId, LayoutInfo>,
    
    // 渲染系统
    render_queue: Vec<ElementId>,       // 按Z索引排序的渲染队列
    
    // 主题系统
    current_theme: String,
    themes: HashMap<String, UITheme>,
    
    // 配置
    screen_size: Vec2,
    ui_scale: f32,
    
    // 统计
    total_elements: u64,
    rendered_elements: u32,
    layout_calculations: u64,
    
    // 调试
    debug_mode: bool,
    show_bounds: bool,
}

// UI主题
#[derive(Debug, Clone)]
pub struct UITheme {
    pub name: String,
    pub default_styles: HashMap<ElementType, UIStyle>,
    pub colors: HashMap<String, Vec4>,
    pub fonts: HashMap<String, u32>,
    pub textures: HashMap<String, u32>,
}

impl UIManager {
    pub fn new(screen_size: Vec2) -> Self {
        let mut manager = Self {
            elements: HashMap::new(),
            next_element_id: 1,
            root_elements: Vec::new(),
            focused_element: None,
            focus_chain: Vec::new(),
            hovered_element: None,
            pressed_elements: HashMap::new(),
            layout_dirty: false,
            layout_cache: HashMap::new(),
            render_queue: Vec::new(),
            current_theme: "default".to_string(),
            themes: HashMap::new(),
            screen_size,
            ui_scale: 1.0,
            total_elements: 0,
            rendered_elements: 0,
            layout_calculations: 0,
            debug_mode: false,
            show_bounds: false,
        };
        
        manager.create_default_theme();
        manager
    }
    
    // 创建UI元素
    pub fn create_element(
        &mut self,
        name: String,
        element_type: ElementType,
        parent: Option<ElementId>,
    ) -> Result<ElementId, GameError> {
        let element_id = self.next_element_id;
        self.next_element_id += 1;
        
        let mut element = UIElement {
            id: element_id,
            name: name.clone(),
            element_type,
            parent,
            children: Vec::new(),
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 30.0),
            calculated_size: Vec2::ZERO,
            content_size: Vec2::ZERO,
            style: self.get_default_style(element_type),
            state_styles: HashMap::new(),
            state: ElementState::Normal,
            enabled: true,
            focusable: matches!(element_type, 
                ElementType::Button | ElementType::TextInput | ElementType::Toggle),
            layout_type: LayoutType::None,
            text: String::new(),
            value: String::new(),
            event_handlers: HashMap::new(),
            user_data: HashMap::new(),
            animation_time: 0.0,
            target_position: None,
            target_size: None,
            animation_duration: 0.0,
        };
        
        // 设置父子关系
        if let Some(parent_id) = parent {
            if let Some(parent_element) = self.elements.get_mut(&parent_id) {
                parent_element.children.push(element_id);
            } else {
                return Err(GameError::UI(format!("父元素不存在: {}", parent_id)));
            }
        } else {
            self.root_elements.push(element_id);
        }
        
        self.elements.insert(element_id, element);
        self.layout_dirty = true;
        self.total_elements += 1;
        
        debug!("创建UI元素: '{}' ID={} 类型={:?}", name, element_id, element_type);
        Ok(element_id)
    }
    
    // 销毁UI元素
    pub fn destroy_element(&mut self, element_id: ElementId) -> Result<(), GameError> {
        if let Some(element) = self.elements.get(&element_id) {
            // 递归销毁子元素
            let children = element.children.clone();
            for child_id in children {
                self.destroy_element(child_id)?;
            }
            
            // 从父元素中移除
            if let Some(parent_id) = element.parent {
                if let Some(parent) = self.elements.get_mut(&parent_id) {
                    parent.children.retain(|&id| id != element_id);
                }
            } else {
                self.root_elements.retain(|&id| id != element_id);
            }
            
            // 清理焦点
            if self.focused_element == Some(element_id) {
                self.focused_element = None;
            }
            self.focus_chain.retain(|&id| id != element_id);
            
            // 清理悬停状态
            if self.hovered_element == Some(element_id) {
                self.hovered_element = None;
            }
            
            // 清理按压状态
            self.pressed_elements.retain(|_, &mut id| id != element_id);
            
            // 移除元素
            self.elements.remove(&element_id);
            self.layout_dirty = true;
            
            debug!("销毁UI元素: ID={}", element_id);
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 获取元素
    pub fn get_element(&self, element_id: ElementId) -> Option<&UIElement> {
        self.elements.get(&element_id)
    }
    
    // 获取可变元素
    pub fn get_element_mut(&mut self, element_id: ElementId) -> Option<&mut UIElement> {
        self.elements.get_mut(&element_id)
    }
    
    // 设置元素位置
    pub fn set_element_position(&mut self, element_id: ElementId, position: Vec2) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.position = position;
            self.layout_dirty = true;
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置元素尺寸
    pub fn set_element_size(&mut self, element_id: ElementId, size: Vec2) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.size = size;
            self.layout_dirty = true;
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置元素文本
    pub fn set_element_text(&mut self, element_id: ElementId, text: String) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.text = text;
            self.layout_dirty = true;
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置元素值
    pub fn set_element_value(&mut self, element_id: ElementId, value: String) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            let old_value = element.value.clone();
            element.value = value.clone();
            
            // 触发值变化事件
            let event = UIEvent::ValueChanged {
                old_value,
                new_value: value,
            };
            self.dispatch_event(element_id, &event);
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置元素可见性
    pub fn set_element_visible(&mut self, element_id: ElementId, visible: bool) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.style.visible = visible;
            self.layout_dirty = true;
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置元素启用状态
    pub fn set_element_enabled(&mut self, element_id: ElementId, enabled: bool) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.enabled = enabled;
            if !enabled && element.state != ElementState::Disabled {
                element.state = ElementState::Disabled;
            } else if enabled && element.state == ElementState::Disabled {
                element.state = ElementState::Normal;
            }
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 设置焦点
    pub fn set_focus(&mut self, element_id: Option<ElementId>) -> Result<(), GameError> {
        if let Some(old_focused) = self.focused_element {
            if let Some(element) = self.elements.get_mut(&old_focused) {
                element.state = ElementState::Normal;
                self.dispatch_event(old_focused, &UIEvent::Focus { gained: false });
            }
        }
        
        if let Some(new_focused) = element_id {
            if let Some(element) = self.elements.get_mut(&new_focused) {
                if element.focusable && element.enabled {
                    element.state = ElementState::Focused;
                    self.dispatch_event(new_focused, &UIEvent::Focus { gained: true });
                } else {
                    return Err(GameError::UI("元素不可聚焦".to_string()));
                }
            } else {
                return Err(GameError::UI(format!("UI元素不存在: {}", new_focused)));
            }
        }
        
        self.focused_element = element_id;
        Ok(())
    }
    
    // 处理鼠标事件
    pub fn handle_mouse_event(
        &mut self,
        position: Vec2,
        button: Option<MouseButton>,
        pressed: bool,
    ) -> Result<(), GameError> {
        // 查找位置下的元素
        let hit_element = self.find_element_at_position(position);
        
        // 处理悬停事件
        if self.hovered_element != hit_element {
            if let Some(old_hovered) = self.hovered_element {
                if let Some(element) = self.elements.get_mut(&old_hovered) {
                    element.state = ElementState::Normal;
                }
                self.dispatch_event(old_hovered, &UIEvent::Hover { 
                    position, 
                    entered: false 
                });
            }
            
            if let Some(new_hovered) = hit_element {
                if let Some(element) = self.elements.get_mut(&new_hovered) {
                    if element.enabled && element.state == ElementState::Normal {
                        element.state = ElementState::Hovered;
                    }
                }
                self.dispatch_event(new_hovered, &UIEvent::Hover { 
                    position, 
                    entered: true 
                });
            }
            
            self.hovered_element = hit_element;
        }
        
        // 处理点击事件
        if let Some(button) = button {
            if pressed {
                if let Some(element_id) = hit_element {
                    if let Some(element) = self.elements.get_mut(&element_id) {
                        if element.enabled {
                            element.state = ElementState::Pressed;
                            self.pressed_elements.insert(button, element_id);
                            
                            // 设置焦点
                            if element.focusable {
                                self.set_focus(Some(element_id))?;
                            }
                        }
                    }
                }
            } else {
                // 释放按钮
                if let Some(pressed_element) = self.pressed_elements.remove(&button) {
                    if let Some(element) = self.elements.get_mut(&pressed_element) {
                        element.state = if self.hovered_element == Some(pressed_element) {
                            ElementState::Hovered
                        } else {
                            ElementState::Normal
                        };
                        
                        // 如果在同一个元素上释放，触发点击事件
                        if hit_element == Some(pressed_element) {
                            self.dispatch_event(pressed_element, &UIEvent::Click { 
                                position, 
                                button 
                            });
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    // 处理键盘事件
    pub fn handle_keyboard_event(
        &mut self,
        key: String,
        modifiers: KeyModifiers,
        pressed: bool,
    ) -> Result<(), GameError> {
        if pressed {
            if let Some(focused_element) = self.focused_element {
                self.dispatch_event(focused_element, &UIEvent::KeyPress { key, modifiers });
            }
            
            // 处理Tab键焦点切换
            if key == "Tab" && !modifiers.ctrl && !modifiers.alt {
                if modifiers.shift {
                    self.focus_previous()?;
                } else {
                    self.focus_next()?;
                }
            }
        }
        
        Ok(())
    }
    
    // 更新UI系统
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        // 更新动画
        self.update_animations(delta_time);
        
        // 计算布局
        if self.layout_dirty {
            self.calculate_layout()?;
            self.layout_dirty = false;
        }
        
        // 更新渲染队列
        self.update_render_queue();
        
        Ok(())
    }
    
    // 渲染UI
    pub fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        self.rendered_elements = 0;
        
        for &element_id in &self.render_queue {
            if let Some(element) = self.elements.get(&element_id) {
                if element.style.visible {
                    self.render_element(renderer, element)?;
                    self.rendered_elements += 1;
                }
            }
        }
        
        // 渲染调试信息
        if self.debug_mode {
            self.render_debug_info(renderer)?;
        }
        
        Ok(())
    }
    
    // 添加事件处理器
    pub fn add_event_handler<F>(
        &mut self,
        element_id: ElementId,
        event_type: String,
        handler: F,
    ) -> Result<(), GameError>
    where
        F: Fn(&UIEvent) + Send + 'static,
    {
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.event_handlers.insert(event_type, Box::new(handler));
            Ok(())
        } else {
            Err(GameError::UI(format!("UI元素不存在: {}", element_id)))
        }
    }
    
    // 查找元素
    pub fn find_element_by_name(&self, name: &str) -> Option<ElementId> {
        self.elements
            .iter()
            .find(|(_, element)| element.name == name)
            .map(|(&id, _)| id)
    }
    
    // 获取UI统计信息
    pub fn get_stats(&self) -> UIStats {
        UIStats {
            total_elements: self.elements.len(),
            root_elements: self.root_elements.len(),
            rendered_elements: self.rendered_elements,
            layout_calculations: self.layout_calculations,
            focused_element: self.focused_element,
            hovered_element: self.hovered_element,
            memory_usage: self.calculate_memory_usage(),
        }
    }
    
    // 设置屏幕尺寸
    pub fn set_screen_size(&mut self, size: Vec2) {
        self.screen_size = size;
        self.layout_dirty = true;
        debug!("设置屏幕尺寸: {:?}", size);
    }
    
    // 设置UI缩放
    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale = scale;
        self.layout_dirty = true;
        debug!("设置UI缩放: {}", scale);
    }
    
    // 私有方法
    fn create_default_theme(&mut self) {
        let theme = UITheme {
            name: "default".to_string(),
            default_styles: self.create_default_styles(),
            colors: self.create_default_colors(),
            fonts: HashMap::new(),
            textures: HashMap::new(),
        };
        
        self.themes.insert("default".to_string(), theme);
    }
    
    fn create_default_styles(&self) -> HashMap<ElementType, UIStyle> {
        let mut styles = HashMap::new();
        
        // 默认样式
        let default_style = UIStyle {
            width: SizeUnit::Auto,
            height: SizeUnit::Auto,
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            margin: EdgeInsets { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 },
            padding: EdgeInsets { left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
            background_color: Vec4::new(0.2, 0.2, 0.2, 1.0),
            background_image: None,
            border_width: 1.0,
            border_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            border_radius: 3.0,
            font_size: 14.0,
            font_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            text_align: Alignment::Start,
            line_height: 1.2,
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_color: Vec4::new(0.0, 0.0, 0.0, 0.5),
            opacity: 1.0,
            visible: true,
            z_index: 0,
        };
        
        // 按钮样式
        let mut button_style = default_style.clone();
        button_style.background_color = Vec4::new(0.3, 0.5, 0.8, 1.0);
        button_style.padding = EdgeInsets { left: 10.0, top: 8.0, right: 10.0, bottom: 8.0 };
        button_style.text_align = Alignment::Center;
        
        // 文本输入框样式
        let mut input_style = default_style.clone();
        input_style.background_color = Vec4::new(1.0, 1.0, 1.0, 1.0);
        input_style.font_color = Vec4::new(0.0, 0.0, 0.0, 1.0);
        input_style.border_color = Vec4::new(0.6, 0.6, 0.6, 1.0);
        
        styles.insert(ElementType::Panel, default_style.clone());
        styles.insert(ElementType::Button, button_style);
        styles.insert(ElementType::Label, default_style.clone());
        styles.insert(ElementType::TextInput, input_style);
        styles.insert(ElementType::Image, default_style.clone());
        
        styles
    }
    
    fn create_default_colors(&self) -> HashMap<String, Vec4> {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), Vec4::new(0.3, 0.5, 0.8, 1.0));
        colors.insert("secondary".to_string(), Vec4::new(0.6, 0.6, 0.6, 1.0));
        colors.insert("success".to_string(), Vec4::new(0.2, 0.7, 0.3, 1.0));
        colors.insert("warning".to_string(), Vec4::new(0.9, 0.6, 0.1, 1.0));
        colors.insert("error".to_string(), Vec4::new(0.8, 0.2, 0.2, 1.0));
        colors.insert("text".to_string(), Vec4::new(0.9, 0.9, 0.9, 1.0));
        colors.insert("background".to_string(), Vec4::new(0.1, 0.1, 0.1, 1.0));
        colors
    }
    
    fn get_default_style(&self, element_type: ElementType) -> UIStyle {
        if let Some(theme) = self.themes.get(&self.current_theme) {
            if let Some(style) = theme.default_styles.get(&element_type) {
                return style.clone();
            }
        }
        
        // 如果没有找到主题或样式，返回基础默认样式
        UIStyle {
            width: SizeUnit::Auto,
            height: SizeUnit::Auto,
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            margin: EdgeInsets { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 },
            padding: EdgeInsets { left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
            background_color: Vec4::new(0.2, 0.2, 0.2, 1.0),
            background_image: None,
            border_width: 1.0,
            border_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            border_radius: 3.0,
            font_size: 14.0,
            font_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            text_align: Alignment::Start,
            line_height: 1.2,
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_color: Vec4::new(0.0, 0.0, 0.0, 0.5),
            opacity: 1.0,
            visible: true,
            z_index: 0,
        }
    }
    
    fn find_element_at_position(&self, position: Vec2) -> Option<ElementId> {
        // 按Z索引从高到低遍历
        for &element_id in self.render_queue.iter().rev() {
            if let Some(element) = self.elements.get(&element_id) {
                if !element.style.visible || !element.enabled {
                    continue;
                }
                
                if self.point_in_element(position, element) {
                    return Some(element_id);
                }
            }
        }
        None
    }
    
    fn point_in_element(&self, point: Vec2, element: &UIElement) -> bool {
        let pos = element.position;
        let size = element.calculated_size;
        
        point.x >= pos.x && point.x <= pos.x + size.x &&
        point.y >= pos.y && point.y <= pos.y + size.y
    }
    
    fn dispatch_event(&self, element_id: ElementId, event: &UIEvent) {
        if let Some(element) = self.elements.get(&element_id) {
            // 查找对应的事件处理器
            let event_type = match event {
                UIEvent::Click { .. } => "click",
                UIEvent::Hover { .. } => "hover",
                UIEvent::KeyPress { .. } => "keypress",
                UIEvent::Focus { .. } => "focus",
                UIEvent::ValueChanged { .. } => "valuechanged",
                UIEvent::Scroll { .. } => "scroll",
                UIEvent::Resize { .. } => "resize",
                UIEvent::Custom { event_type, .. } => event_type,
            };
            
            if let Some(handler) = element.event_handlers.get(event_type) {
                handler(event);
            }
            
            debug!("分派事件: 元素={} 事件={}", element_id, event_type);
        }
    }
    
    fn focus_next(&mut self) -> Result<(), GameError> {
        // 简化的焦点切换实现
        let focusable_elements: Vec<ElementId> = self.elements
            .iter()
            .filter(|(_, element)| element.focusable && element.enabled && element.style.visible)
            .map(|(&id, _)| id)
            .collect();
        
        if focusable_elements.is_empty() {
            return Ok(());
        }
        
        let current_index = self.focused_element
            .and_then(|id| focusable_elements.iter().position(|&x| x == id))
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % focusable_elements.len();
        self.set_focus(Some(focusable_elements[next_index]))?;
        
        Ok(())
    }
    
    fn focus_previous(&mut self) -> Result<(), GameError> {
        // 简化的焦点切换实现
        let focusable_elements: Vec<ElementId> = self.elements
            .iter()
            .filter(|(_, element)| element.focusable && element.enabled && element.style.visible)
            .map(|(&id, _)| id)
            .collect();
        
        if focusable_elements.is_empty() {
            return Ok(());
        }
        
        let current_index = self.focused_element
            .and_then(|id| focusable_elements.iter().position(|&x| x == id))
            .unwrap_or(0);
        
        let prev_index = if current_index == 0 {
            focusable_elements.len() - 1
        } else {
            current_index - 1
        };
        
        self.set_focus(Some(focusable_elements[prev_index]))?;
        
        Ok(())
    }
    
    fn update_animations(&mut self, delta_time: f32) {
        for element in self.elements.values_mut() {
            if element.animation_duration > 0.0 {
                element.animation_time += delta_time;
                
                let progress = (element.animation_time / element.animation_duration).min(1.0);
                
                // 更新位置动画
                if let Some(target_pos) = element.target_position {
                    let start_pos = element.position;
                    element.position = start_pos.lerp(target_pos, progress);
                    
                    if progress >= 1.0 {
                        element.target_position = None;
                    }
                }
                
                // 更新尺寸动画
                if let Some(target_size) = element.target_size {
                    let start_size = element.size;
                    element.size = start_size.lerp(target_size, progress);
                    
                    if progress >= 1.0 {
                        element.target_size = None;
                    }
                }
                
                // 重置动画
                if progress >= 1.0 {
                    element.animation_time = 0.0;
                    element.animation_duration = 0.0;
                    self.layout_dirty = true;
                }
            }
        }
    }
    
    fn calculate_layout(&mut self) -> Result<(), GameError> {
        self.layout_cache.clear();
        self.layout_calculations += 1;
        
        // 计算根元素布局
        for &root_id in &self.root_elements.clone() {
            self.calculate_element_layout(root_id, Vec2::ZERO, self.screen_size)?;
        }
        
        debug!("计算UI布局: {} 个元素", self.elements.len());
        Ok(())
    }
    
    fn calculate_element_layout(
        &mut self,
        element_id: ElementId,
        parent_position: Vec2,
        parent_size: Vec2,
    ) -> Result<Vec2, GameError> {
        let element = self.elements.get(&element_id)
            .ok_or_else(|| GameError::UI(format!("元素不存在: {}", element_id)))?;
        
        if !element.style.visible {
            return Ok(Vec2::ZERO);
        }
        
        // 计算实际尺寸
        let calculated_width = match element.style.width {
            SizeUnit::Pixels(w) => w,
            SizeUnit::Percent(p) => parent_size.x * p / 100.0,
            SizeUnit::Auto => element.size.x, // 使用设置的尺寸
            SizeUnit::Fill => parent_size.x,
        };
        
        let calculated_height = match element.style.height {
            SizeUnit::Pixels(h) => h,
            SizeUnit::Percent(p) => parent_size.y * p / 100.0,
            SizeUnit::Auto => element.size.y, // 使用设置的尺寸
            SizeUnit::Fill => parent_size.y,
        };
        
        let calculated_size = Vec2::new(
            calculated_width.clamp(element.style.min_width, element.style.max_width),
            calculated_height.clamp(element.style.min_height, element.style.max_height),
        );
        
        // 计算位置 (简化实现)
        let calculated_position = parent_position + element.position;
        
        // 更新元素的计算尺寸
        if let Some(element) = self.elements.get_mut(&element_id) {
            element.calculated_size = calculated_size;
        }
        
        // 缓存布局信息
        let layout_info = LayoutInfo {
            element_id,
            position: calculated_position,
            size: calculated_size,
            content_rect: (calculated_position, calculated_size),
            margin_rect: (calculated_position, calculated_size),
            padding_rect: (calculated_position, calculated_size),
        };
        
        self.layout_cache.insert(element_id, layout_info);
        
        // 递归计算子元素布局
        let children = self.elements.get(&element_id).unwrap().children.clone();
        for child_id in children {
            self.calculate_element_layout(child_id, calculated_position, calculated_size)?;
        }
        
        Ok(calculated_size)
    }
    
    fn update_render_queue(&mut self) {
        self.render_queue.clear();
        
        // 收集所有可见元素
        for (&element_id, element) in &self.elements {
            if element.style.visible {
                self.render_queue.push(element_id);
            }
        }
        
        // 按Z索引排序
        self.render_queue.sort_by(|&a, &b| {
            let element_a = &self.elements[&a];
            let element_b = &self.elements[&b];
            element_a.style.z_index.cmp(&element_b.style.z_index)
        });
    }
    
    fn render_element(&self, renderer: &mut Renderer2D, element: &UIElement) -> Result<(), GameError> {
        let position = element.position;
        let size = element.calculated_size;
        
        // 渲染背景
        if element.style.background_color.w > 0.0 {
            let background_color = element.style.background_color * element.style.opacity;
            
            if let Some(texture_id) = element.style.background_image {
                renderer.draw_sprite(
                    position,
                    size,
                    texture_id,
                    None,
                    background_color,
                    0.0,
                    false,
                    false,
                )?;
            } else {
                // 使用白色纹理绘制纯色背景
                renderer.draw_quad(position, size, 1, background_color, 0.0)?;
            }
        }
        
        // 渲染边框
        if element.style.border_width > 0.0 && element.style.border_color.w > 0.0 {
            let border_color = element.style.border_color * element.style.opacity;
            let border_width = element.style.border_width;
            
            // 简化的边框渲染 (四条线)
            renderer.draw_line(
                position,
                position + Vec2::new(size.x, 0.0),
                border_width,
                border_color,
            )?;
            renderer.draw_line(
                position + Vec2::new(size.x, 0.0),
                position + size,
                border_width,
                border_color,
            )?;
            renderer.draw_line(
                position + size,
                position + Vec2::new(0.0, size.y),
                border_width,
                border_color,
            )?;
            renderer.draw_line(
                position + Vec2::new(0.0, size.y),
                position,
                border_width,
                border_color,
            )?;
        }
        
        // 渲染文本
        if !element.text.is_empty() {
            let text_color = element.style.font_color * element.style.opacity;
            let text_position = position + Vec2::new(
                element.style.padding.left,
                element.style.padding.top,
            );
            
            renderer.draw_text(
                &element.text,
                text_position,
                element.style.font_size * self.ui_scale,
                text_color,
                1, // 假设使用字体纹理ID=1
            )?;
        }
        
        Ok(())
    }
    
    fn render_debug_info(&self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        if self.show_bounds {
            // 渲染元素边界
            for element in self.elements.values() {
                if element.style.visible {
                    let debug_color = Vec4::new(1.0, 0.0, 0.0, 0.5);
                    
                    // 绘制边界框
                    renderer.draw_line(
                        element.position,
                        element.position + Vec2::new(element.calculated_size.x, 0.0),
                        1.0,
                        debug_color,
                    )?;
                    renderer.draw_line(
                        element.position + Vec2::new(element.calculated_size.x, 0.0),
                        element.position + element.calculated_size,
                        1.0,
                        debug_color,
                    )?;
                    renderer.draw_line(
                        element.position + element.calculated_size,
                        element.position + Vec2::new(0.0, element.calculated_size.y),
                        1.0,
                        debug_color,
                    )?;
                    renderer.draw_line(
                        element.position + Vec2::new(0.0, element.calculated_size.y),
                        element.position,
                        1.0,
                        debug_color,
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    fn calculate_memory_usage(&self) -> usize {
        let mut total = 0;
        
        // UI元素内存
        total += self.elements.len() * std::mem::size_of::<UIElement>();
        
        // 布局缓存内存
        total += self.layout_cache.len() * std::mem::size_of::<LayoutInfo>();
        
        // 主题内存
        for theme in self.themes.values() {
            total += std::mem::size_of::<UITheme>();
            total += theme.default_styles.len() * std::mem::size_of::<UIStyle>();
        }
        
        total
    }
}

// UI统计信息
#[derive(Debug, Clone)]
pub struct UIStats {
    pub total_elements: usize,
    pub root_elements: usize,
    pub rendered_elements: u32,
    pub layout_calculations: u64,
    pub focused_element: Option<ElementId>,
    pub hovered_element: Option<ElementId>,
    pub memory_usage: usize,
}

// 默认实现
impl Default for KeyModifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
        }
    }
}

impl Default for EdgeInsets {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

impl EdgeInsets {
    pub fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }
    
    pub fn horizontal_vertical(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_manager_creation() {
        let manager = UIManager::new(Vec2::new(800.0, 600.0));
        assert_eq!(manager.screen_size, Vec2::new(800.0, 600.0));
        assert_eq!(manager.elements.len(), 0);
        assert_eq!(manager.root_elements.len(), 0);
    }
    
    #[test]
    fn test_element_creation() {
        let mut manager = UIManager::new(Vec2::new(800.0, 600.0));
        
        let button_id = manager.create_element(
            "test_button".to_string(),
            ElementType::Button,
            None,
        ).unwrap();
        
        assert_eq!(button_id, 1);
        assert_eq!(manager.elements.len(), 1);
        assert_eq!(manager.root_elements.len(), 1);
        
        let element = manager.get_element(button_id).unwrap();
        assert_eq!(element.name, "test_button");
        assert_eq!(element.element_type, ElementType::Button);
        assert!(element.focusable);
    }
    
    #[test]
    fn test_element_hierarchy() {
        let mut manager = UIManager::new(Vec2::new(800.0, 600.0));
        
        let parent_id = manager.create_element(
            "parent".to_string(),
            ElementType::Panel,
            None,
        ).unwrap();
        
        let child_id = manager.create_element(
            "child".to_string(),
            ElementType::Button,
            Some(parent_id),
        ).unwrap();
        
        let parent = manager.get_element(parent_id).unwrap();
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0], child_id);
        
        let child = manager.get_element(child_id).unwrap();
        assert_eq!(child.parent, Some(parent_id));
    }
    
    #[test]
    fn test_element_properties() {
        let mut manager = UIManager::new(Vec2::new(800.0, 600.0));
        let element_id = manager.create_element("test".to_string(), ElementType::Label, None).unwrap();
        
        // 测试位置
        manager.set_element_position(element_id, Vec2::new(100.0, 200.0)).unwrap();
        let element = manager.get_element(element_id).unwrap();
        assert_eq!(element.position, Vec2::new(100.0, 200.0));
        
        // 测试文本
        manager.set_element_text(element_id, "Hello World".to_string()).unwrap();
        let element = manager.get_element(element_id).unwrap();
        assert_eq!(element.text, "Hello World");
        
        // 测试可见性
        manager.set_element_visible(element_id, false).unwrap();
        let element = manager.get_element(element_id).unwrap();
        assert!(!element.style.visible);
    }
    
    #[test]
    fn test_focus_management() {
        let mut manager = UIManager::new(Vec2::new(800.0, 600.0));
        
        let button1 = manager.create_element("button1".to_string(), ElementType::Button, None).unwrap();
        let button2 = manager.create_element("button2".to_string(), ElementType::Button, None).unwrap();
        
        // 设置焦点
        manager.set_focus(Some(button1)).unwrap();
        assert_eq!(manager.focused_element, Some(button1));
        
        let element = manager.get_element(button1).unwrap();
        assert_eq!(element.state, ElementState::Focused);
        
        // 切换焦点
        manager.set_focus(Some(button2)).unwrap();
        assert_eq!(manager.focused_element, Some(button2));
        
        let element1 = manager.get_element(button1).unwrap();
        assert_eq!(element1.state, ElementState::Normal);
        
        let element2 = manager.get_element(button2).unwrap();
        assert_eq!(element2.state, ElementState::Focused);
    }
}