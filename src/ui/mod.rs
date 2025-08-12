// UI模块
// 开发心理：提供基础UI组件和管理系统
// 设计原则：模块化、可复用、响应式布局

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::error::GameError;

// 基础UI组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub id: String,
    pub element_type: ElementType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub visible: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Text,
    Image,
    Panel,
    Slider,
    Input,
}

// UI事件
#[derive(Debug, Clone)]
pub enum UIEvent {
    Click(String),
    Hover(String),
    Input(String, String),
    ValueChanged(String, f32),
}

// UI管理器
#[derive(Debug)]
pub struct UIManager {
    elements: HashMap<String, UIElement>,
    event_queue: Vec<UIEvent>,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            event_queue: Vec::new(),
        }
    }
    
    pub fn add_element(&mut self, element: UIElement) {
        self.elements.insert(element.id.clone(), element);
    }
    
    pub fn remove_element(&mut self, id: &str) {
        self.elements.remove(id);
    }
    
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        // 基础更新逻辑
        Ok(())
    }
    
    pub fn render(&self) -> Result<(), GameError> {
        // 基础渲染逻辑
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: UIEvent) {
        self.event_queue.push(event);
    }
    
    pub fn get_events(&mut self) -> Vec<UIEvent> {
        std::mem::take(&mut self.event_queue)
    }
    
    // 创建UI元素
    pub fn create_element(&mut self, id: String, element_type: ElementType) -> Result<(), GameError> {
        let element = UIElement {
            id: id.clone(),
            element_type,
            position: (0.0, 0.0),
            size: (100.0, 30.0),
            visible: true,
            enabled: true,
        };
        self.elements.insert(id, element);
        Ok(())
    }
    
    // 设置元素文本
    pub fn set_element_text(&mut self, id: &str, text: String) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(id) {
            // 在实际实现中，这里应该设置元素的文本属性
            // 目前只是占位符实现
            Ok(())
        } else {
            Err(GameError::UIError(format!("元素不存在: {}", id)))
        }
    }
    
    // 设置元素位置
    pub fn set_element_position(&mut self, id: &str, position: (f32, f32)) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.position = position;
            Ok(())
        } else {
            Err(GameError::UIError(format!("元素不存在: {}", id)))
        }
    }
    
    // 设置元素大小
    pub fn set_element_size(&mut self, id: &str, size: (f32, f32)) -> Result<(), GameError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.size = size;
            Ok(())
        } else {
            Err(GameError::UIError(format!("元素不存在: {}", id)))
        }
    }
    
    // 添加事件处理器
    pub fn add_event_handler(&mut self, id: &str, handler: fn(&UIEvent)) -> Result<(), GameError> {
        // 在实际实现中，这里应该存储事件处理器
        // 目前只是占位符实现
        if self.elements.contains_key(id) {
            Ok(())
        } else {
            Err(GameError::UIError(format!("元素不存在: {}", id)))
        }
    }
    
    // 设置焦点
    pub fn set_focus(&mut self, id: &str) -> Result<(), GameError> {
        if self.elements.contains_key(id) {
            // 在实际实现中，这里应该管理焦点状态
            Ok(())
        } else {
            Err(GameError::UIError(format!("元素不存在: {}", id)))
        }
    }
    
    // 处理鼠标事件
    pub fn handle_mouse_event(&mut self, x: f32, y: f32, button: u32) -> Result<(), GameError> {
        // 检查哪个元素被点击
        for (id, element) in &self.elements {
            if x >= element.position.0 && x <= element.position.0 + element.size.0 &&
               y >= element.position.1 && y <= element.position.1 + element.size.1 {
                self.handle_event(UIEvent::Click(id.clone()));
                break;
            }
        }
        Ok(())
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}