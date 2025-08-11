// 主菜单状态
// 开发心理：主菜单是游戏入口，需要美观界面、清晰导航、快速响应
// 设计原则：用户友好、视觉吸引、功能完整、性能优化

use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::Renderer2D;
use crate::graphics::ui::{UIManager, ElementType, UIEvent};
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
use super::{GameState, GameStateType, StateTransition};
use glam::{Vec2, Vec4};

// 菜单项
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    NewGame,
    Continue,
    Settings,
    Credits,
    Quit,
}

// 主菜单状态
pub struct MainMenuState {
    name: String,
    ui_manager: UIManager,
    
    // UI元素ID
    title_label_id: Option<u32>,
    menu_buttons: Vec<(MenuItem, u32)>,
    background_image_id: Option<u32>,
    
    // 状态
    selected_item: usize,
    
    // 动画
    title_pulse: f32,
    button_hover_scale: f32,
    background_parallax: Vec2,
    
    // 配置
    background_color: Vec4,
    title_color: Vec4,
    button_colors: (Vec4, Vec4, Vec4), // normal, hover, pressed
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            name: "MainMenuState".to_string(),
            ui_manager: UIManager::new(Vec2::new(800.0, 600.0)),
            title_label_id: None,
            menu_buttons: Vec::new(),
            background_image_id: None,
            selected_item: 0,
            title_pulse: 0.0,
            button_hover_scale: 1.0,
            background_parallax: Vec2::ZERO,
            background_color: Vec4::new(0.05, 0.1, 0.2, 1.0),
            title_color: Vec4::new(1.0, 0.9, 0.3, 1.0),
            button_colors: (
                Vec4::new(0.2, 0.4, 0.8, 1.0),
                Vec4::new(0.3, 0.5, 0.9, 1.0),
                Vec4::new(0.1, 0.3, 0.7, 1.0),
            ),
        }
    }
    
    // 初始化UI
    fn setup_ui(&mut self) -> Result<(), GameError> {
        // 游戏标题
        self.title_label_id = Some(self.ui_manager.create_element(
            "title".to_string(),
            ElementType::Label,
            None,
        )?);
        
        if let Some(title_id) = self.title_label_id {
            self.ui_manager.set_element_text(title_id, "Pokemon GO".to_string())?;
            self.ui_manager.set_element_position(title_id, Vec2::new(400.0, 150.0))?;
            self.ui_manager.set_element_size(title_id, Vec2::new(400.0, 80.0))?;
        }
        
        // 创建菜单按钮
        let menu_items = vec![
            (MenuItem::NewGame, "新游戏"),
            (MenuItem::Continue, "继续游戏"),
            (MenuItem::Settings, "设置"),
            (MenuItem::Credits, "制作人员"),
            (MenuItem::Quit, "退出游戏"),
        ];
        
        let button_start_y = 280.0;
        let button_spacing = 60.0;
        
        for (i, (item, text)) in menu_items.iter().enumerate() {
            let button_id = self.ui_manager.create_element(
                format!("button_{:?}", item),
                ElementType::Button,
                None,
            )?;
            
            self.ui_manager.set_element_text(button_id, text.to_string())?;
            self.ui_manager.set_element_position(
                button_id,
                Vec2::new(400.0, button_start_y + i as f32 * button_spacing),
            )?;
            self.ui_manager.set_element_size(button_id, Vec2::new(200.0, 45.0))?;
            
            self.menu_buttons.push((item.clone(), button_id));
            
            // 添加点击事件处理器
            let item_clone = item.clone();
            self.ui_manager.add_event_handler(
                button_id,
                "click".to_string(),
                move |_event| {
                    debug!("菜单项被点击: {:?}", item_clone);
                },
            )?;
        }
        
        debug!("主菜单UI初始化完成");
        Ok(())
    }
    
    // 处理菜单选择
    fn handle_menu_selection(&mut self, item: MenuItem) -> StateTransition {
        debug!("处理菜单选择: {:?}", item);
        
        match item {
            MenuItem::NewGame => {
                // 开始新游戏
                StateTransition::Push(GameStateType::Loading)
            },
            MenuItem::Continue => {
                // 继续游戏 (检查存档)
                if self.has_save_file() {
                    StateTransition::Push(GameStateType::Loading)
                } else {
                    warn!("没有找到存档文件");
                    StateTransition::None
                }
            },
            MenuItem::Settings => {
                StateTransition::Push(GameStateType::Settings)
            },
            MenuItem::Credits => {
                StateTransition::Push(GameStateType::Credits)
            },
            MenuItem::Quit => {
                StateTransition::Quit
            },
        }
    }
    
    // 检查是否有存档文件
    fn has_save_file(&self) -> bool {
        // 简化实现，实际应该检查文件系统
        std::path::Path::new("save.dat").exists()
    }
    
    // 更新动画
    fn update_animations(&mut self, delta_time: f32) {
        // 标题脉动动画
        self.title_pulse += delta_time * 2.0;
        if self.title_pulse >= std::f32::consts::PI * 2.0 {
            self.title_pulse -= std::f32::consts::PI * 2.0;
        }
        
        // 背景视差效果
        self.background_parallax.x += delta_time * 10.0;
        self.background_parallax.y += delta_time * 5.0;
        
        if self.background_parallax.x > 100.0 {
            self.background_parallax.x -= 100.0;
        }
        if self.background_parallax.y > 100.0 {
            self.background_parallax.y -= 100.0;
        }
    }
    
    // 键盘导航
    fn navigate_menu(&mut self, direction: i32) {
        let old_selected = self.selected_item;
        
        if direction > 0 {
            self.selected_item = (self.selected_item + 1) % self.menu_buttons.len();
        } else if direction < 0 {
            self.selected_item = if self.selected_item == 0 {
                self.menu_buttons.len() - 1
            } else {
                self.selected_item - 1
            };
        }
        
        if old_selected != self.selected_item {
            // 更新按钮状态
            if let Some((_, old_button_id)) = self.menu_buttons.get(old_selected) {
                // 重置旧按钮状态
            }
            
            if let Some((_, new_button_id)) = self.menu_buttons.get(self.selected_item) {
                // 设置新按钮为聚焦状态
                self.ui_manager.set_focus(Some(*new_button_id)).ok();
            }
            
            debug!("菜单导航: {} -> {}", old_selected, self.selected_item);
        }
    }
    
    // 确认选择
    fn confirm_selection(&mut self) -> StateTransition {
        if let Some((item, _)) = self.menu_buttons.get(self.selected_item) {
            self.handle_menu_selection(item.clone())
        } else {
            StateTransition::None
        }
    }
}

impl GameState for MainMenuState {
    fn get_type(&self) -> GameStateType {
        GameStateType::MainMenu
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("进入主菜单状态");
        
        // 初始化UI
        self.setup_ui()?;
        
        // 重置动画状态
        self.title_pulse = 0.0;
        self.background_parallax = Vec2::ZERO;
        self.selected_item = 0;
        
        Ok(())
    }
    
    fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("退出主菜单状态");
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), GameError> {
        debug!("暂停主菜单状态");
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), GameError> {
        debug!("恢复主菜单状态");
        Ok(())
    }
    
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError> {
        // 更新动画
        self.update_animations(delta_time);
        
        // 更新UI
        self.ui_manager.update(delta_time)?;
        
        Ok(StateTransition::None)
    }
    
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 清屏
        renderer.clear(self.background_color)?;
        
        // 渲染背景 (如果有)
        if let Some(bg_id) = self.background_image_id {
            renderer.draw_sprite(
                self.background_parallax,
                Vec2::new(800.0, 600.0),
                bg_id,
                None,
                Vec4::new(1.0, 1.0, 1.0, 0.3),
                0.0,
                false,
                false,
            )?;
        }
        
        // 渲染UI
        self.ui_manager.render(renderer)?;
        
        // 渲染标题特效
        if let Some(title_id) = self.title_label_id {
            let pulse_scale = 1.0 + self.title_pulse.sin() * 0.05;
            let title_color = self.title_color * (0.9 + self.title_pulse.sin() * 0.1);
            
            // 标题阴影效果
            renderer.draw_text(
                "Pokemon GO",
                Vec2::new(402.0, 152.0),
                48.0 * pulse_scale,
                Vec4::new(0.0, 0.0, 0.0, 0.5),
                1,
            )?;
            
            // 主标题
            renderer.draw_text(
                "Pokemon GO",
                Vec2::new(400.0, 150.0),
                48.0 * pulse_scale,
                title_color,
                1,
            )?;
        }
        
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError> {
        // 委托给UI管理器处理
        let is_pressed = event.state == crate::input::mouse::MouseState::Pressed;
        let handled = if let Some(button) = event.button {
            self.ui_manager.handle_mouse_event(
                event.position,
                Some(button),
                is_pressed,
            )?
        } else {
            false
        };
        
        if handled {
            // 检查是否点击了菜单按钮
            for (item, button_id) in &self.menu_buttons {
                // 这里应该检查具体哪个按钮被点击
                // 简化实现
            }
        }
        
        Ok(handled)
    }
    
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        if !pressed {
            return Ok(false);
        }
        
        match key {
            "ArrowUp" | "w" | "W" => {
                self.navigate_menu(-1);
                Ok(true)
            },
            "ArrowDown" | "s" | "S" => {
                self.navigate_menu(1);
                Ok(true)
            },
            "Return" | "Space" => {
                let transition = self.confirm_selection();
                // 这里应该将转换传递给状态管理器
                Ok(true)
            },
            "Escape" => {
                // ESC键退出游戏
                Ok(false) // 让上层处理
            },
            _ => Ok(false),
        }
    }
    
    fn handle_gamepad_event(&mut self, event: &GamepadEvent) -> Result<bool, GameError> {
        match event {
            GamepadEvent::ButtonPressed { button, .. } => {
                match button.as_str() {
                    "DPadUp" | "LeftStickUp" => {
                        self.navigate_menu(-1);
                        Ok(true)
                    },
                    "DPadDown" | "LeftStickDown" => {
                        self.navigate_menu(1);
                        Ok(true)
                    },
                    "A" | "Cross" => {
                        let transition = self.confirm_selection();
                        Ok(true)
                    },
                    "B" | "Circle" => {
                        // 返回或退出
                        Ok(false)
                    },
                    _ => Ok(false),
                }
            },
            _ => Ok(false),
        }
    }
    
    fn get_ui_manager(&mut self) -> Option<&mut UIManager> {
        Some(&mut self.ui_manager)
    }
    
    fn is_transparent(&self) -> bool {
        false
    }
    
    fn blocks_input(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_main_menu_creation() {
        let menu = MainMenuState::new();
        assert_eq!(menu.get_type(), GameStateType::MainMenu);
        assert_eq!(menu.get_name(), "MainMenuState");
        assert_eq!(menu.selected_item, 0);
    }
    
    #[test]
    fn test_menu_navigation() {
        let mut menu = MainMenuState::new();
        
        // 模拟菜单按钮
        menu.menu_buttons = vec![
            (MenuItem::NewGame, 1),
            (MenuItem::Continue, 2),
            (MenuItem::Settings, 3),
            (MenuItem::Quit, 4),
        ];
        
        // 测试向下导航
        menu.navigate_menu(1);
        assert_eq!(menu.selected_item, 1);
        
        // 测试向上导航
        menu.navigate_menu(-1);
        assert_eq!(menu.selected_item, 0);
        
        // 测试边界处理
        menu.navigate_menu(-1);
        assert_eq!(menu.selected_item, 3); // 应该回到最后一个
    }
    
    #[test]
    fn test_menu_selection() {
        let mut menu = MainMenuState::new();
        
        // 测试新游戏选择
        let transition = menu.handle_menu_selection(MenuItem::NewGame);
        assert_eq!(transition, StateTransition::Push(GameStateType::Loading));
        
        // 测试退出选择
        let transition = menu.handle_menu_selection(MenuItem::Quit);
        assert_eq!(transition, StateTransition::Quit);
    }
}