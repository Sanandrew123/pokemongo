// Menu UI System - 菜单界面系统
//
// 开发心理过程：
// 1. 这是游戏UI系统的核心组件，负责主菜单、暂停菜单、设置菜单等界面
// 2. 需要支持键盘、手柄和触摸输入，提供良好的用户体验
// 3. 实现可配置的菜单系统，支持动态菜单项和子菜单嵌套
// 4. 集成音效、动画和视觉效果，增强用户交互体验
// 5. 为不同游戏状态提供对应的菜单界面

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

use crate::input::{InputAction, InputState};
use crate::graphics::{UIRenderer, TextStyle, Color as GameColor, Texture};
use crate::audio::{SoundManager, SoundEffect};
use crate::core::GameState;

pub type MenuId = u32;
pub type MenuItemId = u32;

/// 菜单管理器
#[derive(Resource)]
pub struct MenuManager {
    pub active_menu_stack: Vec<MenuId>,
    pub menus: HashMap<MenuId, Menu>,
    pub menu_history: VecDeque<MenuId>,
    pub transition_state: MenuTransition,
    pub current_selection: HashMap<MenuId, MenuItemId>,
    pub global_menu_settings: MenuSettings,
    pub input_buffer: InputBuffer,
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuManager {
    pub fn new() -> Self {
        let mut manager = Self {
            active_menu_stack: Vec::new(),
            menus: HashMap::new(),
            menu_history: VecDeque::new(),
            transition_state: MenuTransition::None,
            current_selection: HashMap::new(),
            global_menu_settings: MenuSettings::default(),
            input_buffer: InputBuffer::new(),
        };
        
        manager.initialize_default_menus();
        manager
    }

    fn initialize_default_menus(&mut self) {
        // 主菜单
        let main_menu = Menu {
            id: 1,
            title: "Pokemon Adventure".to_string(),
            menu_type: MenuType::Main,
            items: vec![
                MenuItem::new(1, "New Game".to_string(), MenuItemType::Action(MenuAction::NewGame)),
                MenuItem::new(2, "Continue".to_string(), MenuItemType::Action(MenuAction::LoadGame)),
                MenuItem::new(3, "Settings".to_string(), MenuItemType::SubMenu(2)),
                MenuItem::new(4, "Exit".to_string(), MenuItemType::Action(MenuAction::Exit)),
            ],
            layout: MenuLayout::Vertical,
            style: MenuStyle::default(),
            background: Some("main_menu_bg.png".to_string()),
            music: Some("main_theme.ogg".to_string()),
            is_visible: false,
            animation: MenuAnimation::FadeIn,
        };
        
        // 设置菜单
        let settings_menu = Menu {
            id: 2,
            title: "Settings".to_string(),
            menu_type: MenuType::Settings,
            items: vec![
                MenuItem::new(1, "Audio".to_string(), MenuItemType::SubMenu(3)),
                MenuItem::new(2, "Controls".to_string(), MenuItemType::SubMenu(4)),
                MenuItem::new(3, "Graphics".to_string(), MenuItemType::SubMenu(5)),
                MenuItem::new(4, "Gameplay".to_string(), MenuItemType::SubMenu(6)),
                MenuItem::new(5, "Back".to_string(), MenuItemType::Action(MenuAction::Back)),
            ],
            layout: MenuLayout::Vertical,
            style: MenuStyle::settings(),
            background: Some("settings_bg.png".to_string()),
            music: None,
            is_visible: false,
            animation: MenuAnimation::SlideLeft,
        };

        // 游戏内菜单
        let pause_menu = Menu {
            id: 10,
            title: "Paused".to_string(),
            menu_type: MenuType::Pause,
            items: vec![
                MenuItem::new(1, "Resume".to_string(), MenuItemType::Action(MenuAction::Resume)),
                MenuItem::new(2, "Pokemon".to_string(), MenuItemType::Action(MenuAction::OpenPokemonMenu)),
                MenuItem::new(3, "Bag".to_string(), MenuItemType::Action(MenuAction::OpenInventory)),
                MenuItem::new(4, "Pokedex".to_string(), MenuItemType::Action(MenuAction::OpenPokedex)),
                MenuItem::new(5, "Settings".to_string(), MenuItemType::SubMenu(2)),
                MenuItem::new(6, "Save Game".to_string(), MenuItemType::Action(MenuAction::SaveGame)),
                MenuItem::new(7, "Main Menu".to_string(), MenuItemType::Action(MenuAction::ReturnToMainMenu)),
            ],
            layout: MenuLayout::Vertical,
            style: MenuStyle::pause(),
            background: Some("pause_bg.png".to_string()),
            music: None,
            is_visible: false,
            animation: MenuAnimation::Scale,
        };

        self.menus.insert(1, main_menu);
        self.menus.insert(2, settings_menu);
        self.menus.insert(10, pause_menu);
    }

    pub fn open_menu(&mut self, menu_id: MenuId) -> Result<(), MenuError> {
        if !self.menus.contains_key(&menu_id) {
            return Err(MenuError::MenuNotFound);
        }

        // 如果有当前活动菜单，添加到历史
        if let Some(&current_menu) = self.active_menu_stack.last() {
            self.menu_history.push_back(current_menu);
        }

        self.active_menu_stack.push(menu_id);
        self.current_selection.entry(menu_id).or_insert(1);

        if let Some(menu) = self.menus.get_mut(&menu_id) {
            menu.is_visible = true;
            self.transition_state = MenuTransition::Opening(menu_id);
        }

        Ok(())
    }

    pub fn close_current_menu(&mut self) -> Result<MenuId, MenuError> {
        if let Some(menu_id) = self.active_menu_stack.pop() {
            if let Some(menu) = self.menus.get_mut(&menu_id) {
                menu.is_visible = false;
                self.transition_state = MenuTransition::Closing(menu_id);
            }
            Ok(menu_id)
        } else {
            Err(MenuError::NoActiveMenu)
        }
    }

    pub fn get_active_menu(&self) -> Option<&Menu> {
        self.active_menu_stack.last()
            .and_then(|&menu_id| self.menus.get(&menu_id))
    }

    pub fn get_active_menu_mut(&mut self) -> Option<&mut Menu> {
        let menu_id = *self.active_menu_stack.last()?;
        self.menus.get_mut(&menu_id)
    }

    pub fn handle_input(&mut self, input_state: &InputState) -> Vec<MenuAction> {
        let mut actions = Vec::new();

        if let Some(active_menu) = self.get_active_menu() {
            let menu_id = active_menu.id;
            
            // 处理导航输入
            if input_state.just_pressed(InputAction::MenuUp) {
                self.navigate_up(menu_id);
            } else if input_state.just_pressed(InputAction::MenuDown) {
                self.navigate_down(menu_id);
            } else if input_state.just_pressed(InputAction::MenuLeft) {
                self.navigate_left(menu_id);
            } else if input_state.just_pressed(InputAction::MenuRight) {
                self.navigate_right(menu_id);
            }

            // 处理确认和取消
            if input_state.just_pressed(InputAction::MenuConfirm) {
                if let Some(action) = self.activate_current_item(menu_id) {
                    actions.push(action);
                }
            } else if input_state.just_pressed(InputAction::MenuCancel) {
                actions.push(MenuAction::Back);
            }

            // 处理快捷键
            for (i, item) in active_menu.items.iter().enumerate() {
                if let Some(hotkey) = &item.hotkey {
                    if input_state.just_pressed(*hotkey) {
                        if let Some(action) = self.activate_item(menu_id, item.id) {
                            actions.push(action);
                        }
                    }
                }
            }
        }

        actions
    }

    fn navigate_up(&mut self, menu_id: MenuId) {
        if let Some(menu) = self.menus.get(&menu_id) {
            let current_selection = self.current_selection.get(&menu_id).copied().unwrap_or(1);
            let current_index = menu.items.iter()
                .position(|item| item.id == current_selection)
                .unwrap_or(0);

            let enabled_items: Vec<_> = menu.items.iter()
                .filter(|item| item.is_enabled)
                .collect();

            if !enabled_items.is_empty() {
                let current_pos = enabled_items.iter()
                    .position(|item| item.id == current_selection)
                    .unwrap_or(0);

                let new_pos = if current_pos == 0 {
                    enabled_items.len() - 1
                } else {
                    current_pos - 1
                };

                self.current_selection.insert(menu_id, enabled_items[new_pos].id);
            }
        }
    }

    fn navigate_down(&mut self, menu_id: MenuId) {
        if let Some(menu) = self.menus.get(&menu_id) {
            let current_selection = self.current_selection.get(&menu_id).copied().unwrap_or(1);
            
            let enabled_items: Vec<_> = menu.items.iter()
                .filter(|item| item.is_enabled)
                .collect();

            if !enabled_items.is_empty() {
                let current_pos = enabled_items.iter()
                    .position(|item| item.id == current_selection)
                    .unwrap_or(0);

                let new_pos = (current_pos + 1) % enabled_items.len();
                self.current_selection.insert(menu_id, enabled_items[new_pos].id);
            }
        }
    }

    fn navigate_left(&mut self, menu_id: MenuId) {
        if let Some(menu) = self.menus.get(&menu_id) {
            if menu.layout == MenuLayout::Grid {
                // Grid布局的左右导航逻辑
                // TODO: 实现网格导航
            }
        }
    }

    fn navigate_right(&mut self, menu_id: MenuId) {
        if let Some(menu) = self.menus.get(&menu_id) {
            if menu.layout == MenuLayout::Grid {
                // Grid布局的左右导航逻辑
                // TODO: 实现网格导航
            }
        }
    }

    fn activate_current_item(&mut self, menu_id: MenuId) -> Option<MenuAction> {
        let current_selection = self.current_selection.get(&menu_id).copied()?;
        self.activate_item(menu_id, current_selection)
    }

    fn activate_item(&mut self, menu_id: MenuId, item_id: MenuItemId) -> Option<MenuAction> {
        let menu = self.menus.get(&menu_id)?;
        let item = menu.items.iter().find(|item| item.id == item_id)?;

        if !item.is_enabled {
            return None;
        }

        match &item.item_type {
            MenuItemType::Action(action) => Some(action.clone()),
            MenuItemType::SubMenu(sub_menu_id) => {
                let _ = self.open_menu(*sub_menu_id);
                None
            }
            MenuItemType::Toggle(ref toggle) => {
                // TODO: 处理切换项
                None
            }
            MenuItemType::Slider(ref slider) => {
                // TODO: 处理滑块项
                None
            }
            MenuItemType::Text(ref text) => {
                // TODO: 处理文本输入项
                None
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // 更新动画和过渡效果
        self.update_transitions(delta_time);
        
        // 更新菜单项动画
        for menu in self.menus.values_mut() {
            if menu.is_visible {
                menu.update(delta_time);
            }
        }

        // 清理输入缓冲区
        self.input_buffer.update(delta_time);
    }

    fn update_transitions(&mut self, delta_time: f32) {
        match &mut self.transition_state {
            MenuTransition::Opening(menu_id) => {
                // 处理开启动画
                if let Some(menu) = self.menus.get_mut(menu_id) {
                    if menu.animation_finished() {
                        self.transition_state = MenuTransition::None;
                    }
                }
            }
            MenuTransition::Closing(menu_id) => {
                // 处理关闭动画
                if let Some(menu) = self.menus.get_mut(menu_id) {
                    if menu.animation_finished() {
                        menu.is_visible = false;
                        self.transition_state = MenuTransition::None;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render(&self, ui_renderer: &mut UIRenderer) {
        // 从栈底开始渲染菜单（背景菜单先渲染）
        for &menu_id in &self.active_menu_stack {
            if let Some(menu) = self.menus.get(&menu_id) {
                if menu.is_visible {
                    self.render_menu(menu, ui_renderer);
                }
            }
        }

        // 渲染过渡效果
        self.render_transitions(ui_renderer);
    }

    fn render_menu(&self, menu: &Menu, ui_renderer: &mut UIRenderer) {
        let selected_item_id = self.current_selection.get(&menu.id).copied();

        // 渲染背景
        if let Some(ref background) = menu.background {
            ui_renderer.draw_texture(background, Vec2::ZERO, Vec2::new(1920.0, 1080.0));
        }

        // 渲染标题
        if !menu.title.is_empty() {
            ui_renderer.draw_text(
                &menu.title,
                Vec2::new(960.0, 100.0),
                &menu.style.title_style,
            );
        }

        // 渲染菜单项
        match menu.layout {
            MenuLayout::Vertical => self.render_vertical_menu(menu, selected_item_id, ui_renderer),
            MenuLayout::Horizontal => self.render_horizontal_menu(menu, selected_item_id, ui_renderer),
            MenuLayout::Grid => self.render_grid_menu(menu, selected_item_id, ui_renderer),
        }

        // 渲染装饰效果
        self.render_menu_decorations(menu, ui_renderer);
    }

    fn render_vertical_menu(&self, menu: &Menu, selected_item_id: Option<MenuItemId>, ui_renderer: &mut UIRenderer) {
        let start_y = 300.0;
        let item_height = 80.0;
        let menu_width = 600.0;
        let menu_x = (1920.0 - menu_width) / 2.0;

        for (index, item) in menu.items.iter().enumerate() {
            let y = start_y + index as f32 * item_height;
            let is_selected = selected_item_id == Some(item.id);

            self.render_menu_item(item, Vec2::new(menu_x, y), Vec2::new(menu_width, item_height - 10.0), 
                                 is_selected, &menu.style, ui_renderer);
        }
    }

    fn render_horizontal_menu(&self, menu: &Menu, selected_item_id: Option<MenuItemId>, ui_renderer: &mut UIRenderer) {
        let start_x = 200.0;
        let item_width = 200.0;
        let item_height = 60.0;
        let menu_y = 500.0;

        for (index, item) in menu.items.iter().enumerate() {
            let x = start_x + index as f32 * (item_width + 20.0);
            let is_selected = selected_item_id == Some(item.id);

            self.render_menu_item(item, Vec2::new(x, menu_y), Vec2::new(item_width, item_height), 
                                 is_selected, &menu.style, ui_renderer);
        }
    }

    fn render_grid_menu(&self, menu: &Menu, selected_item_id: Option<MenuItemId>, ui_renderer: &mut UIRenderer) {
        let grid_cols = 3;
        let item_width = 180.0;
        let item_height = 80.0;
        let start_x = (1920.0 - grid_cols as f32 * (item_width + 20.0)) / 2.0;
        let start_y = 300.0;

        for (index, item) in menu.items.iter().enumerate() {
            let col = index % grid_cols;
            let row = index / grid_cols;
            let x = start_x + col as f32 * (item_width + 20.0);
            let y = start_y + row as f32 * (item_height + 20.0);
            let is_selected = selected_item_id == Some(item.id);

            self.render_menu_item(item, Vec2::new(x, y), Vec2::new(item_width, item_height), 
                                 is_selected, &menu.style, ui_renderer);
        }
    }

    fn render_menu_item(&self, item: &MenuItem, position: Vec2, size: Vec2, is_selected: bool, 
                       style: &MenuStyle, ui_renderer: &mut UIRenderer) {
        let background_color = if is_selected {
            style.selected_color
        } else if !item.is_enabled {
            style.disabled_color
        } else {
            style.normal_color
        };

        // 渲染背景
        ui_renderer.draw_rect(position, size, background_color);

        // 渲染边框（如果被选中）
        if is_selected {
            ui_renderer.draw_rect_outline(position, size, style.border_color, 2.0);
        }

        // 渲染图标（如果有）
        if let Some(ref icon) = item.icon {
            let icon_size = Vec2::new(32.0, 32.0);
            let icon_pos = Vec2::new(position.x + 10.0, position.y + (size.y - icon_size.y) / 2.0);
            ui_renderer.draw_texture(icon, icon_pos, icon_size);
        }

        // 渲染文本
        let text_style = if is_selected {
            &style.selected_text_style
        } else if !item.is_enabled {
            &style.disabled_text_style
        } else {
            &style.normal_text_style
        };

        let text_x = position.x + if item.icon.is_some() { 50.0 } else { 20.0 };
        let text_y = position.y + size.y / 2.0;
        
        ui_renderer.draw_text(&item.text, Vec2::new(text_x, text_y), text_style);

        // 渲染快捷键提示（如果有）
        if let Some(hotkey) = &item.hotkey {
            let hotkey_text = format!("[{:?}]", hotkey);
            let hotkey_x = position.x + size.x - 100.0;
            ui_renderer.draw_text(&hotkey_text, Vec2::new(hotkey_x, text_y), &style.hotkey_text_style);
        }

        // 渲染特殊项目类型的额外UI
        match &item.item_type {
            MenuItemType::SubMenu(_) => {
                // 渲染子菜单箭头
                let arrow_pos = Vec2::new(position.x + size.x - 30.0, text_y);
                ui_renderer.draw_text(">", arrow_pos, text_style);
            }
            MenuItemType::Toggle(toggle) => {
                // 渲染开关状态
                let toggle_text = if toggle.value { "ON" } else { "OFF" };
                let toggle_pos = Vec2::new(position.x + size.x - 80.0, text_y);
                ui_renderer.draw_text(toggle_text, toggle_pos, text_style);
            }
            MenuItemType::Slider(slider) => {
                // 渲染滑块
                self.render_slider(slider, Vec2::new(position.x + size.x - 120.0, text_y), ui_renderer);
            }
            _ => {}
        }
    }

    fn render_slider(&self, slider: &SliderData, position: Vec2, ui_renderer: &mut UIRenderer) {
        let slider_width = 100.0;
        let slider_height = 6.0;
        let knob_size = Vec2::new(12.0, 16.0);
        
        // 滑块背景
        ui_renderer.draw_rect(position, Vec2::new(slider_width, slider_height), GameColor::GRAY);
        
        // 滑块进度
        let progress = (slider.value - slider.min) / (slider.max - slider.min);
        let progress_width = slider_width * progress;
        ui_renderer.draw_rect(position, Vec2::new(progress_width, slider_height), GameColor::BLUE);
        
        // 滑块旋钮
        let knob_x = position.x + progress_width - knob_size.x / 2.0;
        let knob_y = position.y - (knob_size.y - slider_height) / 2.0;
        ui_renderer.draw_rect(Vec2::new(knob_x, knob_y), knob_size, GameColor::WHITE);
    }

    fn render_menu_decorations(&self, menu: &Menu, ui_renderer: &mut UIRenderer) {
        // 渲染菜单装饰效果（边框、阴影等）
        match menu.menu_type {
            MenuType::Main => {
                // 主菜单特殊效果
                // TODO: 粒子效果、背景动画等
            }
            MenuType::Pause => {
                // 暂停菜单半透明背景
                ui_renderer.draw_rect(Vec2::ZERO, Vec2::new(1920.0, 1080.0), 
                                    GameColor::from_rgba(0.0, 0.0, 0.0, 0.5));
            }
            _ => {}
        }
    }

    fn render_transitions(&self, ui_renderer: &mut UIRenderer) {
        // 渲染过渡动画效果
        match &self.transition_state {
            MenuTransition::Opening(menu_id) => {
                // 渲染打开动画
            }
            MenuTransition::Closing(menu_id) => {
                // 渲染关闭动画
            }
            _ => {}
        }
    }
}

/// 菜单定义
#[derive(Debug, Clone)]
pub struct Menu {
    pub id: MenuId,
    pub title: String,
    pub menu_type: MenuType,
    pub items: Vec<MenuItem>,
    pub layout: MenuLayout,
    pub style: MenuStyle,
    pub background: Option<String>,
    pub music: Option<String>,
    pub is_visible: bool,
    pub animation: MenuAnimation,
}

impl Menu {
    pub fn update(&mut self, delta_time: f32) {
        // 更新菜单项动画
        for item in &mut self.items {
            item.update(delta_time);
        }
    }

    pub fn animation_finished(&self) -> bool {
        // TODO: 检查动画是否完成
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    Main,
    Pause,
    Settings,
    Inventory,
    Pokemon,
    Battle,
    Shop,
    Save,
    Load,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuLayout {
    Vertical,
    Horizontal,
    Grid,
}

#[derive(Debug, Clone)]
pub enum MenuAnimation {
    None,
    FadeIn,
    FadeOut,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Scale,
    Bounce,
}

/// 菜单项定义
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: MenuItemId,
    pub text: String,
    pub item_type: MenuItemType,
    pub is_enabled: bool,
    pub is_visible: bool,
    pub icon: Option<String>,
    pub hotkey: Option<InputAction>,
    pub tooltip: Option<String>,
    pub animation_timer: f32,
}

impl MenuItem {
    pub fn new(id: MenuItemId, text: String, item_type: MenuItemType) -> Self {
        Self {
            id,
            text,
            item_type,
            is_enabled: true,
            is_visible: true,
            icon: None,
            hotkey: None,
            tooltip: None,
            animation_timer: 0.0,
        }
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_hotkey(mut self, hotkey: InputAction) -> Self {
        self.hotkey = Some(hotkey);
        self
    }

    pub fn with_tooltip(mut self, tooltip: String) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        self.animation_timer += delta_time;
    }
}

#[derive(Debug, Clone)]
pub enum MenuItemType {
    Action(MenuAction),
    SubMenu(MenuId),
    Toggle(ToggleData),
    Slider(SliderData),
    Text(TextInputData),
}

#[derive(Debug, Clone)]
pub struct ToggleData {
    pub value: bool,
    pub on_text: String,
    pub off_text: String,
}

#[derive(Debug, Clone)]
pub struct SliderData {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub format: String, // 显示格式，如 "{:.1}" 或 "{:.0}%"
}

#[derive(Debug, Clone)]
pub struct TextInputData {
    pub value: String,
    pub max_length: usize,
    pub placeholder: String,
    pub is_password: bool,
}

/// 菜单动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    // 游戏流程
    NewGame,
    LoadGame,
    SaveGame,
    Exit,
    Resume,
    Pause,
    ReturnToMainMenu,
    
    // 界面导航
    Back,
    OpenPokemonMenu,
    OpenInventory,
    OpenPokedex,
    OpenSettings,
    OpenShop,
    
    // 设置相关
    ChangeVolume(f32),
    ChangeResolution(u32, u32),
    ToggleFullscreen,
    ChangeLanguage(String),
    ResetSettings,
    
    // 自定义动作
    Custom(String),
}

/// 菜单样式
#[derive(Debug, Clone)]
pub struct MenuStyle {
    pub normal_color: GameColor,
    pub selected_color: GameColor,
    pub disabled_color: GameColor,
    pub border_color: GameColor,
    
    pub normal_text_style: TextStyle,
    pub selected_text_style: TextStyle,
    pub disabled_text_style: TextStyle,
    pub title_style: TextStyle,
    pub hotkey_text_style: TextStyle,
    
    pub padding: f32,
    pub margin: f32,
    pub border_width: f32,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            normal_color: GameColor::from_rgba(0.2, 0.2, 0.2, 0.8),
            selected_color: GameColor::from_rgba(0.0, 0.4, 0.8, 0.9),
            disabled_color: GameColor::from_rgba(0.1, 0.1, 0.1, 0.5),
            border_color: GameColor::WHITE,
            
            normal_text_style: TextStyle {
                font: "default".to_string(),
                size: 24.0,
                color: GameColor::WHITE,
            },
            selected_text_style: TextStyle {
                font: "default".to_string(),
                size: 26.0,
                color: GameColor::YELLOW,
            },
            disabled_text_style: TextStyle {
                font: "default".to_string(),
                size: 24.0,
                color: GameColor::GRAY,
            },
            title_style: TextStyle {
                font: "title".to_string(),
                size: 48.0,
                color: GameColor::WHITE,
            },
            hotkey_text_style: TextStyle {
                font: "small".to_string(),
                size: 16.0,
                color: GameColor::LIGHT_GRAY,
            },
            
            padding: 10.0,
            margin: 5.0,
            border_width: 2.0,
        }
    }
}

impl MenuStyle {
    pub fn settings() -> Self {
        let mut style = Self::default();
        style.normal_color = GameColor::from_rgba(0.1, 0.1, 0.3, 0.8);
        style.selected_color = GameColor::from_rgba(0.2, 0.2, 0.6, 0.9);
        style
    }

    pub fn pause() -> Self {
        let mut style = Self::default();
        style.normal_color = GameColor::from_rgba(0.0, 0.0, 0.0, 0.7);
        style.selected_color = GameColor::from_rgba(0.3, 0.3, 0.3, 0.8);
        style
    }
}

/// 菜单过渡状态
#[derive(Debug, Clone)]
pub enum MenuTransition {
    None,
    Opening(MenuId),
    Closing(MenuId),
    Switching(MenuId, MenuId), // from, to
}

/// 菜单设置
#[derive(Debug, Clone)]
pub struct MenuSettings {
    pub navigation_sounds: bool,
    pub confirm_sounds: bool,
    pub animation_speed: f32,
    pub auto_repeat_delay: f32,
    pub auto_repeat_rate: f32,
}

impl Default for MenuSettings {
    fn default() -> Self {
        Self {
            navigation_sounds: true,
            confirm_sounds: true,
            animation_speed: 1.0,
            auto_repeat_delay: 0.5,
            auto_repeat_rate: 0.1,
        }
    }
}

/// 输入缓冲区用于处理快速输入
#[derive(Debug)]
pub struct InputBuffer {
    pub buffer: VecDeque<(InputAction, f32)>,
    pub max_buffer_time: f32,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            max_buffer_time: 0.2, // 200ms缓冲时间
        }
    }

    pub fn add_input(&mut self, action: InputAction, current_time: f32) {
        self.buffer.push_back((action, current_time));
    }

    pub fn update(&mut self, current_time: f32) {
        // 清理过期的输入
        while let Some(&(_, time)) = self.buffer.front() {
            if current_time - time > self.max_buffer_time {
                self.buffer.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_buffered_input(&mut self) -> Option<InputAction> {
        self.buffer.pop_front().map(|(action, _)| action)
    }
}

/// 菜单错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuError {
    MenuNotFound,
    NoActiveMenu,
    InvalidMenuItem,
    TransitionInProgress,
}

/// Bevy系统：处理菜单输入
pub fn menu_input_system(
    input_state: Res<InputState>,
    mut menu_manager: ResMut<MenuManager>,
    mut sound_manager: ResMut<SoundManager>,
    mut game_state: ResMut<State<GameState>>,
) {
    let actions = menu_manager.handle_input(&input_state);
    
    for action in actions {
        // 播放音效
        match action {
            MenuAction::Back | MenuAction::Resume => {
                if menu_manager.global_menu_settings.navigation_sounds {
                    sound_manager.play_sound_effect(SoundEffect::MenuCancel);
                }
            }
            _ => {
                if menu_manager.global_menu_settings.confirm_sounds {
                    sound_manager.play_sound_effect(SoundEffect::MenuConfirm);
                }
            }
        }

        // 处理菜单动作
        match action {
            MenuAction::NewGame => {
                game_state.set(GameState::InGame);
            }
            MenuAction::LoadGame => {
                // TODO: 打开存档选择菜单
            }
            MenuAction::SaveGame => {
                // TODO: 执行保存操作
            }
            MenuAction::Exit => {
                std::process::exit(0);
            }
            MenuAction::Resume => {
                let _ = menu_manager.close_current_menu();
                game_state.set(GameState::InGame);
            }
            MenuAction::Pause => {
                let _ = menu_manager.open_menu(10); // 暂停菜单ID
                game_state.set(GameState::Paused);
            }
            MenuAction::Back => {
                let _ = menu_manager.close_current_menu();
            }
            MenuAction::ReturnToMainMenu => {
                menu_manager.active_menu_stack.clear();
                let _ = menu_manager.open_menu(1); // 主菜单ID
                game_state.set(GameState::MainMenu);
            }
            _ => {
                // 其他动作的处理
                println!("Menu action: {:?}", action);
            }
        }
    }
}

/// Bevy系统：更新菜单状态
pub fn menu_update_system(
    time: Res<Time>,
    mut menu_manager: ResMut<MenuManager>,
) {
    menu_manager.update(time.delta_seconds());
}

/// Bevy系统：渲染菜单
pub fn menu_render_system(
    menu_manager: Res<MenuManager>,
    mut ui_renderer: ResMut<UIRenderer>,
) {
    menu_manager.render(&mut ui_renderer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_manager_creation() {
        let manager = MenuManager::new();
        assert!(!manager.menus.is_empty());
        assert!(manager.menus.contains_key(&1)); // 主菜单
        assert!(manager.menus.contains_key(&2)); // 设置菜单
        assert!(manager.menus.contains_key(&10)); // 暂停菜单
    }

    #[test]
    fn test_menu_navigation() {
        let mut manager = MenuManager::new();
        manager.open_menu(1).unwrap();
        
        assert_eq!(manager.active_menu_stack.last(), Some(&1));
        assert_eq!(manager.current_selection.get(&1), Some(&1));
        
        manager.navigate_down(1);
        assert_eq!(manager.current_selection.get(&1), Some(&2));
    }

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new(1, "Test Item".to_string(), 
                                MenuItemType::Action(MenuAction::NewGame))
            .with_icon("test_icon.png".to_string())
            .with_hotkey(InputAction::MenuConfirm)
            .enabled(true);
        
        assert_eq!(item.text, "Test Item");
        assert!(item.is_enabled);
        assert!(item.icon.is_some());
        assert!(item.hotkey.is_some());
    }

    #[test]
    fn test_input_buffer() {
        let mut buffer = InputBuffer::new();
        
        buffer.add_input(InputAction::MenuUp, 0.0);
        buffer.add_input(InputAction::MenuDown, 0.1);
        
        assert_eq!(buffer.buffer.len(), 2);
        
        buffer.update(0.3); // 超过缓冲时间
        assert_eq!(buffer.buffer.len(), 0);
    }
}