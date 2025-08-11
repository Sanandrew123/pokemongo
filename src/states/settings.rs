// 设置状态
// 开发心理：设置界面需要清晰分类、实时反馈、数据持久化
// 设计原则：用户友好、分类明确、即时生效、数据保存

use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::Renderer2D;
use crate::graphics::ui::{UIManager, ElementType};
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
use super::{GameState, GameStateType, StateTransition};
use glam::{Vec2, Vec4};
use std::collections::HashMap;

// 设置类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Audio,
    Video,
    Controls,
    Gameplay,
}

// 设置状态
pub struct SettingsState {
    name: String,
    ui_manager: UIManager,
    
    // 当前设置
    current_category: SettingsCategory,
    settings_data: HashMap<String, SettingValue>,
    
    // UI元素
    category_buttons: Vec<(SettingsCategory, u32)>,
    setting_elements: Vec<u32>,
    
    // 临时设置 (未保存)
    temp_settings: HashMap<String, SettingValue>,
    has_unsaved_changes: bool,
}

// 设置值类型
#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Range(f32, f32, f32), // value, min, max
}

impl SettingsState {
    pub fn new() -> Self {
        let mut state = Self {
            name: "SettingsState".to_string(),
            ui_manager: UIManager::new(Vec2::new(800.0, 600.0)),
            current_category: SettingsCategory::Audio,
            settings_data: HashMap::new(),
            category_buttons: Vec::new(),
            setting_elements: Vec::new(),
            temp_settings: HashMap::new(),
            has_unsaved_changes: false,
        };
        
        state.initialize_default_settings();
        state
    }
    
    // 初始化默认设置
    fn initialize_default_settings(&mut self) {
        // 音频设置
        self.settings_data.insert("master_volume".to_string(), SettingValue::Range(1.0, 0.0, 1.0));
        self.settings_data.insert("music_volume".to_string(), SettingValue::Range(0.8, 0.0, 1.0));
        self.settings_data.insert("sfx_volume".to_string(), SettingValue::Range(0.9, 0.0, 1.0));
        self.settings_data.insert("voice_volume".to_string(), SettingValue::Range(1.0, 0.0, 1.0));
        self.settings_data.insert("mute_audio".to_string(), SettingValue::Bool(false));
        
        // 视频设置
        self.settings_data.insert("resolution".to_string(), SettingValue::String("1920x1080".to_string()));
        self.settings_data.insert("fullscreen".to_string(), SettingValue::Bool(false));
        self.settings_data.insert("vsync".to_string(), SettingValue::Bool(true));
        self.settings_data.insert("fps_limit".to_string(), SettingValue::Int(60));
        self.settings_data.insert("brightness".to_string(), SettingValue::Range(1.0, 0.5, 1.5));
        
        // 控制设置
        self.settings_data.insert("mouse_sensitivity".to_string(), SettingValue::Range(1.0, 0.1, 3.0));
        self.settings_data.insert("invert_mouse".to_string(), SettingValue::Bool(false));
        self.settings_data.insert("gamepad_enabled".to_string(), SettingValue::Bool(true));
        
        // 游戏设置
        self.settings_data.insert("auto_save".to_string(), SettingValue::Bool(true));
        self.settings_data.insert("battle_animations".to_string(), SettingValue::Bool(true));
        self.settings_data.insert("text_speed".to_string(), SettingValue::Range(1.0, 0.5, 3.0));
        self.settings_data.insert("difficulty".to_string(), SettingValue::String("Normal".to_string()));
    }
    
    // 设置UI
    fn setup_ui(&mut self) -> Result<(), GameError> {
        // 创建分类按钮
        let categories = [
            (SettingsCategory::Audio, "音频"),
            (SettingsCategory::Video, "视频"),
            (SettingsCategory::Controls, "控制"),
            (SettingsCategory::Gameplay, "游戏"),
        ];
        
        for (i, (category, name)) in categories.iter().enumerate() {
            let button_id = self.ui_manager.create_element(
                format!("category_{:?}", category),
                ElementType::Button,
                None,
            )?;
            
            self.ui_manager.set_element_text(button_id, name.to_string())?;
            self.ui_manager.set_element_position(
                button_id,
                Vec2::new(50.0 + i as f32 * 120.0, 50.0),
            )?;
            self.ui_manager.set_element_size(button_id, Vec2::new(100.0, 30.0))?;
            
            self.category_buttons.push((*category, button_id));
        }
        
        // 创建设置面板
        self.create_settings_panel()?;
        
        // 创建保存/取消按钮
        let save_button = self.ui_manager.create_element(
            "save_button".to_string(),
            ElementType::Button,
            None,
        )?;
        self.ui_manager.set_element_text(save_button, "保存".to_string())?;
        self.ui_manager.set_element_position(save_button, Vec2::new(600.0, 550.0))?;
        self.ui_manager.set_element_size(save_button, Vec2::new(80.0, 30.0))?;
        
        let cancel_button = self.ui_manager.create_element(
            "cancel_button".to_string(),
            ElementType::Button,
            None,
        )?;
        self.ui_manager.set_element_text(cancel_button, "取消".to_string())?;
        self.ui_manager.set_element_position(cancel_button, Vec2::new(700.0, 550.0))?;
        self.ui_manager.set_element_size(cancel_button, Vec2::new(80.0, 30.0))?;
        
        Ok(())
    }
    
    // 创建设置面板
    fn create_settings_panel(&mut self) -> Result<(), GameError> {
        // 清除现有设置元素
        for &element_id in &self.setting_elements {
            self.ui_manager.destroy_element(element_id)?;
        }
        self.setting_elements.clear();
        
        let settings = match self.current_category {
            SettingsCategory::Audio => vec![
                ("master_volume", "主音量"),
                ("music_volume", "音乐音量"),
                ("sfx_volume", "音效音量"),
                ("voice_volume", "语音音量"),
                ("mute_audio", "静音"),
            ],
            SettingsCategory::Video => vec![
                ("resolution", "分辨率"),
                ("fullscreen", "全屏"),
                ("vsync", "垂直同步"),
                ("fps_limit", "帧率限制"),
                ("brightness", "亮度"),
            ],
            SettingsCategory::Controls => vec![
                ("mouse_sensitivity", "鼠标灵敏度"),
                ("invert_mouse", "反转鼠标"),
                ("gamepad_enabled", "启用手柄"),
            ],
            SettingsCategory::Gameplay => vec![
                ("auto_save", "自动保存"),
                ("battle_animations", "战斗动画"),
                ("text_speed", "文字速度"),
                ("difficulty", "游戏难度"),
            ],
        };
        
        for (i, (key, display_name)) in settings.iter().enumerate() {
            // 标签
            let label_id = self.ui_manager.create_element(
                format!("label_{}", key),
                ElementType::Label,
                None,
            )?;
            self.ui_manager.set_element_text(label_id, display_name.to_string())?;
            self.ui_manager.set_element_position(
                label_id,
                Vec2::new(100.0, 120.0 + i as f32 * 40.0),
            )?;
            self.ui_manager.set_element_size(label_id, Vec2::new(150.0, 25.0))?;
            self.setting_elements.push(label_id);
            
            // 根据设置类型创建对应的控件
            if let Some(setting_value) = self.settings_data.get(*key) {
                let control_id = match setting_value {
                    SettingValue::Bool(_) => {
                        let toggle_id = self.ui_manager.create_element(
                            format!("toggle_{}", key),
                            ElementType::Toggle,
                            None,
                        )?;
                        self.ui_manager.set_element_position(
                            toggle_id,
                            Vec2::new(300.0, 120.0 + i as f32 * 40.0),
                        )?;
                        self.ui_manager.set_element_size(toggle_id, Vec2::new(50.0, 25.0))?;
                        toggle_id
                    },
                    SettingValue::Range(value, min, max) => {
                        let slider_id = self.ui_manager.create_element(
                            format!("slider_{}", key),
                            ElementType::Slider,
                            None,
                        )?;
                        self.ui_manager.set_element_position(
                            slider_id,
                            Vec2::new(300.0, 120.0 + i as f32 * 40.0),
                        )?;
                        self.ui_manager.set_element_size(slider_id, Vec2::new(200.0, 25.0))?;
                        self.ui_manager.set_element_value(
                            slider_id, 
                            format!("{:.2}", value)
                        )?;
                        slider_id
                    },
                    SettingValue::String(_) | SettingValue::Int(_) => {
                        let dropdown_id = self.ui_manager.create_element(
                            format!("dropdown_{}", key),
                            ElementType::Dropdown,
                            None,
                        )?;
                        self.ui_manager.set_element_position(
                            dropdown_id,
                            Vec2::new(300.0, 120.0 + i as f32 * 40.0),
                        )?;
                        self.ui_manager.set_element_size(dropdown_id, Vec2::new(150.0, 25.0))?;
                        dropdown_id
                    },
                };
                
                self.setting_elements.push(control_id);
            }
        }
        
        Ok(())
    }
    
    // 切换分类
    fn switch_category(&mut self, category: SettingsCategory) -> Result<(), GameError> {
        if self.current_category != category {
            self.current_category = category;
            self.create_settings_panel()?;
            debug!("切换设置分类: {:?}", category);
        }
        Ok(())
    }
    
    // 保存设置
    fn save_settings(&mut self) -> Result<(), GameError> {
        // 应用临时设置
        for (key, value) in &self.temp_settings {
            self.settings_data.insert(key.clone(), value.clone());
        }
        
        // 清除临时设置
        self.temp_settings.clear();
        self.has_unsaved_changes = false;
        
        // 这里应该保存到文件
        debug!("设置已保存");
        Ok(())
    }
    
    // 取消更改
    fn cancel_changes(&mut self) -> Result<(), GameError> {
        self.temp_settings.clear();
        self.has_unsaved_changes = false;
        self.create_settings_panel()?; // 重新创建面板以显示原始值
        debug!("取消设置更改");
        Ok(())
    }
    
    // 更新设置值
    fn update_setting(&mut self, key: &str, value: SettingValue) {
        self.temp_settings.insert(key.to_string(), value);
        self.has_unsaved_changes = true;
        debug!("更新设置: {} = {:?}", key, self.temp_settings.get(key));
    }
    
    // 获取设置值
    fn get_setting(&self, key: &str) -> Option<&SettingValue> {
        self.temp_settings.get(key)
            .or_else(|| self.settings_data.get(key))
    }
}

impl GameState for SettingsState {
    fn get_type(&self) -> GameStateType {
        GameStateType::Settings
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("进入设置状态");
        self.setup_ui()?;
        Ok(())
    }
    
    fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("退出设置状态");
        
        // 如果有未保存的更改，询问是否保存
        if self.has_unsaved_changes {
            warn!("有未保存的设置更改");
            // 在实际实现中，这里应该弹出确认对话框
        }
        
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), GameError> {
        debug!("暂停设置状态");
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), GameError> {
        debug!("恢复设置状态");
        Ok(())
    }
    
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError> {
        self.ui_manager.update(delta_time)?;
        Ok(StateTransition::None)
    }
    
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 背景
        renderer.clear(Vec4::new(0.1, 0.1, 0.15, 1.0))?;
        
        // 标题
        renderer.draw_text(
            "游戏设置",
            Vec2::new(350.0, 20.0),
            24.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            1,
        )?;
        
        // 渲染UI
        self.ui_manager.render(renderer)?;
        
        // 未保存更改指示器
        if self.has_unsaved_changes {
            renderer.draw_text(
                "* 有未保存的更改",
                Vec2::new(50.0, 550.0),
                14.0,
                Vec4::new(1.0, 0.7, 0.0, 1.0),
                1,
            )?;
        }
        
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError> {
        // 检查分类按钮点击
        for (category, button_id) in &self.category_buttons {
            // 简化的点击检测实现
            if event.pressed {
                self.switch_category(*category)?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        if !pressed {
            return Ok(false);
        }
        
        match key {
            "Escape" => {
                if self.has_unsaved_changes {
                    self.cancel_changes()?;
                } else {
                    return Ok(false); // 让上层处理返回
                }
                Ok(true)
            },
            "Return" => {
                if self.has_unsaved_changes {
                    self.save_settings()?;
                }
                Ok(true)
            },
            "1" => {
                self.switch_category(SettingsCategory::Audio)?;
                Ok(true)
            },
            "2" => {
                self.switch_category(SettingsCategory::Video)?;
                Ok(true)
            },
            "3" => {
                self.switch_category(SettingsCategory::Controls)?;
                Ok(true)
            },
            "4" => {
                self.switch_category(SettingsCategory::Gameplay)?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }
    
    fn handle_gamepad_event(&mut self, _event: &GamepadEvent) -> Result<bool, GameError> {
        Ok(false)
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
    fn test_settings_state_creation() {
        let settings = SettingsState::new();
        assert_eq!(settings.get_type(), GameStateType::Settings);
        assert_eq!(settings.current_category, SettingsCategory::Audio);
        assert!(!settings.has_unsaved_changes);
    }
    
    #[test]
    fn test_default_settings() {
        let settings = SettingsState::new();
        
        // 检查一些默认设置
        assert!(matches!(
            settings.get_setting("master_volume"),
            Some(SettingValue::Range(1.0, 0.0, 1.0))
        ));
        
        assert!(matches!(
            settings.get_setting("fullscreen"),
            Some(SettingValue::Bool(false))
        ));
    }
    
    #[test]
    fn test_setting_update() {
        let mut settings = SettingsState::new();
        
        settings.update_setting("master_volume", SettingValue::Range(0.5, 0.0, 1.0));
        assert!(settings.has_unsaved_changes);
        
        assert!(matches!(
            settings.get_setting("master_volume"),
            Some(SettingValue::Range(0.5, 0.0, 1.0))
        ));
    }
}