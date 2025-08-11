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
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}