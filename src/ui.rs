// UI系统 - 用户界面管理和交互
// 开发心理：UI是玩家与游戏交互的桥梁，需要直观易用的界面设计
// 设计原则：响应式设计、可扩展组件、主题支持、无障碍访问

use crate::core::{GameError, Result};
use crate::player::Player;
use crate::pokemon::Pokemon;
use crate::utils::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};

// UI管理器
pub struct UIManager {
    screens: HashMap<String, Box<dyn UIScreen>>,
    current_screen: Option<String>,
    screen_stack: Vec<String>,
    
    // 主题系统
    current_theme: UITheme,
    themes: HashMap<String, UITheme>,
    
    // 布局系统
    layout_manager: LayoutManager,
    
    // 动画系统
    animation_manager: AnimationManager,
    
    // 输入系统
    input_handler: InputHandler,
    
    // UI状态
    is_initialized: bool,
    scale_factor: f32,
    screen_size: (u32, u32),
}

// UI屏幕特性
pub trait UIScreen {
    fn initialize(&mut self) -> Result<()>;
    fn update(&mut self, delta_time: Duration) -> Result<()>;
    fn render(&self, renderer: &mut dyn UIRenderer) -> Result<()>;
    fn handle_input(&mut self, input: &InputEvent) -> Result<UIResponse>;
    fn on_enter(&mut self) -> Result<()>;
    fn on_exit(&mut self) -> Result<()>;
    fn get_name(&self) -> &str;
}

// UI渲染器特性
pub trait UIRenderer {
    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
    fn draw_text(&mut self, text: &str, position: Vec2, font: &UIFont, color: Color) -> Result<()>;
    fn draw_image(&mut self, texture_id: u32, rect: Rect) -> Result<()>;
    fn draw_panel(&mut self, panel: &UIPanel) -> Result<()>;
    fn set_clip_rect(&mut self, rect: Rect);
    fn clear_clip_rect(&mut self);
}

// UI组件基础
#[derive(Debug, Clone)]
pub struct UIComponent {
    pub id: String,
    pub position: Vec2,
    pub size: Vec2,
    pub visible: bool,
    pub enabled: bool,
    pub z_order: i32,
    pub style: UIStyle,
    pub children: Vec<UIComponent>,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct UIStyle {
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub padding: UIMargin,
    pub margin: UIMargin,
    pub font: Option<UIFont>,
    pub text_color: Color,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone, Copy)]
pub struct UIMargin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone)]
pub struct UIFont {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

// UI主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITheme {
    pub name: String,
    pub colors: UIColorPalette,
    pub fonts: UIFontSet,
    pub spacing: UISpacing,
    pub animations: UIAnimationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub disabled: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIFontSet {
    pub title: UIFont,
    pub body: UIFont,
    pub caption: UIFont,
    pub button: UIFont,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISpacing {
    pub tiny: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub huge: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAnimationSettings {
    pub duration_fast: Duration,
    pub duration_normal: Duration,
    pub duration_slow: Duration,
    pub easing: EasingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

// 布局管理器
pub struct LayoutManager {
    layouts: HashMap<String, Box<dyn UILayout>>,
    current_layout: Option<String>,
}

pub trait UILayout {
    fn calculate_positions(&self, components: &mut [UIComponent], container_size: Vec2);
    fn get_preferred_size(&self, components: &[UIComponent]) -> Vec2;
}

// 动画管理器
pub struct AnimationManager {
    animations: Vec<UIAnimation>,
    next_id: u32,
}

#[derive(Debug, Clone)]
pub struct UIAnimation {
    pub id: u32,
    pub target_id: String,
    pub property: AnimationProperty,
    pub start_value: f32,
    pub end_value: f32,
    pub duration: Duration,
    pub elapsed: Duration,
    pub easing: EasingType,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationProperty {
    PositionX,
    PositionY,
    Width,
    Height,
    Opacity,
    Scale,
    Rotation,
}

// 输入处理器
pub struct InputHandler {
    mouse_position: Vec2,
    mouse_buttons: HashMap<MouseButton, bool>,
    keyboard_keys: HashMap<KeyCode, bool>,
    focused_component: Option<String>,
    hovered_component: Option<String>,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseMove(Vec2),
    MouseButton { button: MouseButton, pressed: bool, position: Vec2 },
    KeyBoard { key: KeyCode, pressed: bool },
    Text(String),
    Scroll(Vec2),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    Enter, Escape, Space, Tab, Backspace,
    Up, Down, Left, Right,
}

#[derive(Debug, Clone)]
pub enum UIResponse {
    None,
    Handled,
    FocusComponent(String),
    ChangeScreen(String),
    ShowDialog(DialogInfo),
    CloseDialog,
    Custom(String, HashMap<String, String>),
}

// 对话框
#[derive(Debug, Clone)]
pub struct DialogInfo {
    pub title: String,
    pub message: String,
    pub dialog_type: DialogType,
    pub buttons: Vec<DialogButton>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Question,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub text: String,
    pub response: String,
    pub is_default: bool,
    pub is_cancel: bool,
}

// UI面板
#[derive(Debug, Clone)]
pub struct UIPanel {
    pub base: UIComponent,
    pub panel_type: PanelType,
    pub content: Vec<UIComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    Window,
    Dialog,
    Tooltip,
    Menu,
    Inventory,
    Battle,
    Pokedex,
    Settings,
}

// 预定义UI屏幕
pub struct MainMenuScreen {
    components: Vec<UIComponent>,
    background_texture: Option<u32>,
}

pub struct GameplayScreen {
    hud_components: Vec<UIComponent>,
    menu_visible: bool,
}

pub struct BattleScreen {
    pokemon_info: Vec<UIComponent>,
    move_buttons: Vec<UIComponent>,
    battle_log: Vec<String>,
}

pub struct InventoryScreen {
    item_list: Vec<UIComponent>,
    selected_item: Option<usize>,
    categories: Vec<String>,
}

pub struct PokedexScreen {
    pokemon_list: Vec<UIComponent>,
    detail_view: Option<UIComponent>,
    search_filter: String,
}

pub struct SettingsScreen {
    setting_groups: HashMap<String, Vec<UIComponent>>,
}

// 实现
impl UIManager {
    pub fn new() -> Self {
        Self {
            screens: HashMap::new(),
            current_screen: None,
            screen_stack: Vec::new(),
            
            current_theme: UITheme::default(),
            themes: HashMap::new(),
            
            layout_manager: LayoutManager::new(),
            animation_manager: AnimationManager::new(),
            input_handler: InputHandler::new(),
            
            is_initialized: false,
            scale_factor: 1.0,
            screen_size: (1280, 720),
        }
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        // 初始化默认主题
        self.load_default_themes();
        
        // 注册默认屏幕
        self.register_default_screens()?;
        
        self.is_initialized = true;
        info!("UI系统初始化完成");
        Ok(())
    }
    
    pub fn register_screen(&mut self, screen: Box<dyn UIScreen>) {
        let name = screen.get_name().to_string();
        self.screens.insert(name, screen);
    }
    
    pub fn show_screen(&mut self, screen_name: &str) -> Result<()> {
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                screen.on_exit()?;
            }
        }
        
        if let Some(screen) = self.screens.get_mut(screen_name) {
            screen.on_enter()?;
            self.current_screen = Some(screen_name.to_string());
            info!("切换到UI屏幕: {}", screen_name);
            Ok(())
        } else {
            Err(GameError::UIError(format!("UI屏幕不存在: {}", screen_name)))
        }
    }
    
    pub fn push_screen(&mut self, screen_name: &str) -> Result<()> {
        if let Some(current) = &self.current_screen {
            self.screen_stack.push(current.clone());
        }
        self.show_screen(screen_name)
    }
    
    pub fn pop_screen(&mut self) -> Result<()> {
        if let Some(previous) = self.screen_stack.pop() {
            self.show_screen(&previous)
        } else {
            Err(GameError::UIError("没有可返回的屏幕".to_string()))
        }
    }
    
    pub fn update(&mut self, delta_time: Duration) -> Result<()> {
        // 更新动画
        self.animation_manager.update(delta_time);
        
        // 更新当前屏幕
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                screen.update(delta_time)?;
            }
        }
        
        Ok(())
    }
    
    pub fn render(&self, renderer: &mut dyn UIRenderer) -> Result<()> {
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get(current) {
                screen.render(renderer)?;
            }
        }
        
        Ok(())
    }
    
    pub fn handle_input(&mut self, input: &InputEvent) -> Result<UIResponse> {
        // 更新输入状态
        self.input_handler.handle_input(input);
        
        // 传递给当前屏幕
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                return screen.handle_input(input);
            }
        }
        
        Ok(UIResponse::None)
    }
    
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if let Some(theme) = self.themes.get(theme_name).cloned() {
            self.current_theme = theme;
            info!("切换UI主题: {}", theme_name);
            Ok(())
        } else {
            Err(GameError::UIError(format!("主题不存在: {}", theme_name)))
        }
    }
    
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale.clamp(0.5, 3.0);
        debug!("UI缩放比例设置为: {}", self.scale_factor);
    }
    
    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.screen_size = (width, height);
        debug!("UI屏幕尺寸设置为: {}x{}", width, height);
    }
    
    // 私有方法
    fn load_default_themes(&mut self) {
        // 默认主题
        let default_theme = UITheme::default();
        self.themes.insert("default".to_string(), default_theme.clone());
        self.current_theme = default_theme;
        
        // 暗色主题
        let mut dark_theme = UITheme::default();
        dark_theme.name = "Dark".to_string();
        dark_theme.colors.background = Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 };
        dark_theme.colors.surface = Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
        dark_theme.colors.text_primary = Color { r: 0.9, g: 0.9, b: 0.9, a: 1.0 };
        self.themes.insert("dark".to_string(), dark_theme);
        
        // Pokemon主题
        let mut pokemon_theme = UITheme::default();
        pokemon_theme.name = "Pokemon".to_string();
        pokemon_theme.colors.primary = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }; // 红色
        pokemon_theme.colors.secondary = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }; // 蓝色
        self.themes.insert("pokemon".to_string(), pokemon_theme);
    }
    
    fn register_default_screens(&mut self) -> Result<()> {
        // 注册各种默认屏幕
        self.register_screen(Box::new(MainMenuScreen::new()));
        self.register_screen(Box::new(GameplayScreen::new()));
        self.register_screen(Box::new(BattleScreen::new()));
        self.register_screen(Box::new(InventoryScreen::new()));
        self.register_screen(Box::new(PokedexScreen::new()));
        self.register_screen(Box::new(SettingsScreen::new()));
        
        Ok(())
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
            current_layout: None,
        }
    }
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_id: 1,
        }
    }
    
    pub fn start_animation(&mut self, animation: UIAnimation) -> u32 {
        let id = self.next_id;
        let mut anim = animation;
        anim.id = id;
        self.animations.push(anim);
        self.next_id += 1;
        id
    }
    
    pub fn update(&mut self, delta_time: Duration) {
        for animation in &mut self.animations {
            if !animation.completed {
                animation.elapsed += delta_time;
                
                if animation.elapsed >= animation.duration {
                    animation.elapsed = animation.duration;
                    animation.completed = true;
                }
            }
        }
        
        // 移除完成的动画
        self.animations.retain(|anim| !anim.completed);
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2 { x: 0.0, y: 0.0 },
            mouse_buttons: HashMap::new(),
            keyboard_keys: HashMap::new(),
            focused_component: None,
            hovered_component: None,
        }
    }
    
    pub fn handle_input(&mut self, input: &InputEvent) {
        match input {
            InputEvent::MouseMove(pos) => {
                self.mouse_position = *pos;
            },
            InputEvent::MouseButton { button, pressed, .. } => {
                self.mouse_buttons.insert(*button, *pressed);
            },
            InputEvent::KeyBoard { key, pressed } => {
                self.keyboard_keys.insert(*key, *pressed);
            },
            _ => {}
        }
    }
}

// 默认实现
impl Default for UITheme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            colors: UIColorPalette::default(),
            fonts: UIFontSet::default(),
            spacing: UISpacing::default(),
            animations: UIAnimationSettings::default(),
        }
    }
}

impl Default for UIColorPalette {
    fn default() -> Self {
        Self {
            primary: Color { r: 0.2, g: 0.6, b: 1.0, a: 1.0 },
            secondary: Color { r: 0.8, g: 0.4, b: 0.0, a: 1.0 },
            background: Color { r: 0.95, g: 0.95, b: 0.95, a: 1.0 },
            surface: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            text_primary: Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            text_secondary: Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
            accent: Color { r: 1.0, g: 0.2, b: 0.6, a: 1.0 },
            error: Color { r: 1.0, g: 0.2, b: 0.2, a: 1.0 },
            warning: Color { r: 1.0, g: 0.8, b: 0.0, a: 1.0 },
            success: Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 },
            disabled: Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
        }
    }
}

impl Default for UIFontSet {
    fn default() -> Self {
        Self {
            title: UIFont {
                family: "Arial".to_string(),
                size: 24.0,
                weight: FontWeight::Bold,
                style: FontStyle::Normal,
            },
            body: UIFont {
                family: "Arial".to_string(),
                size: 14.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            caption: UIFont {
                family: "Arial".to_string(),
                size: 12.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            button: UIFont {
                family: "Arial".to_string(),
                size: 16.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
        }
    }
}

impl Default for UISpacing {
    fn default() -> Self {
        Self {
            tiny: 4.0,
            small: 8.0,
            medium: 16.0,
            large: 24.0,
            huge: 32.0,
        }
    }
}

impl Default for UIAnimationSettings {
    fn default() -> Self {
        Self {
            duration_fast: Duration::from_millis(150),
            duration_normal: Duration::from_millis(300),
            duration_slow: Duration::from_millis(600),
            easing: EasingType::EaseInOut,
        }
    }
}

// 屏幕实现
impl MainMenuScreen {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            background_texture: None,
        }
    }
}

impl UIScreen for MainMenuScreen {
    fn initialize(&mut self) -> Result<()> {
        // 初始化主菜单组件
        Ok(())
    }
    
    fn update(&mut self, _delta_time: Duration) -> Result<()> {
        Ok(())
    }
    
    fn render(&self, renderer: &mut dyn UIRenderer) -> Result<()> {
        // 渲染主菜单
        Ok(())
    }
    
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> {
        Ok(UIResponse::None)
    }
    
    fn on_enter(&mut self) -> Result<()> {
        debug!("进入主菜单屏幕");
        Ok(())
    }
    
    fn on_exit(&mut self) -> Result<()> {
        debug!("退出主菜单屏幕");
        Ok(())
    }
    
    fn get_name(&self) -> &str {
        "main_menu"
    }
}

impl GameplayScreen {
    pub fn new() -> Self {
        Self {
            hud_components: Vec::new(),
            menu_visible: false,
        }
    }
}

impl UIScreen for GameplayScreen {
    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn update(&mut self, _delta_time: Duration) -> Result<()> { Ok(()) }
    fn render(&self, _renderer: &mut dyn UIRenderer) -> Result<()> { Ok(()) }
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> { Ok(UIResponse::None) }
    fn on_enter(&mut self) -> Result<()> { Ok(()) }
    fn on_exit(&mut self) -> Result<()> { Ok(()) }
    fn get_name(&self) -> &str { "gameplay" }
}

impl BattleScreen {
    pub fn new() -> Self {
        Self {
            pokemon_info: Vec::new(),
            move_buttons: Vec::new(),
            battle_log: Vec::new(),
        }
    }
}

impl UIScreen for BattleScreen {
    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn update(&mut self, _delta_time: Duration) -> Result<()> { Ok(()) }
    fn render(&self, _renderer: &mut dyn UIRenderer) -> Result<()> { Ok(()) }
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> { Ok(UIResponse::None) }
    fn on_enter(&mut self) -> Result<()> { Ok(()) }
    fn on_exit(&mut self) -> Result<()> { Ok(()) }
    fn get_name(&self) -> &str { "battle" }
}

impl InventoryScreen {
    pub fn new() -> Self {
        Self {
            item_list: Vec::new(),
            selected_item: None,
            categories: vec![
                "物品".to_string(),
                "药品".to_string(),
                "精灵球".to_string(),
                "招式学习器".to_string(),
                "树果".to_string(),
                "关键道具".to_string(),
            ],
        }
    }
}

impl UIScreen for InventoryScreen {
    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn update(&mut self, _delta_time: Duration) -> Result<()> { Ok(()) }
    fn render(&self, _renderer: &mut dyn UIRenderer) -> Result<()> { Ok(()) }
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> { Ok(UIResponse::None) }
    fn on_enter(&mut self) -> Result<()> { Ok(()) }
    fn on_exit(&mut self) -> Result<()> { Ok(()) }
    fn get_name(&self) -> &str { "inventory" }
}

impl PokedexScreen {
    pub fn new() -> Self {
        Self {
            pokemon_list: Vec::new(),
            detail_view: None,
            search_filter: String::new(),
        }
    }
}

impl UIScreen for PokedexScreen {
    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn update(&mut self, _delta_time: Duration) -> Result<()> { Ok(()) }
    fn render(&self, _renderer: &mut dyn UIRenderer) -> Result<()> { Ok(()) }
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> { Ok(UIResponse::None) }
    fn on_enter(&mut self) -> Result<()> { Ok(()) }
    fn on_exit(&mut self) -> Result<()> { Ok(()) }
    fn get_name(&self) -> &str { "pokedex" }
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            setting_groups: HashMap::new(),
        }
    }
}

impl UIScreen for SettingsScreen {
    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn update(&mut self, _delta_time: Duration) -> Result<()> { Ok(()) }
    fn render(&self, _renderer: &mut dyn UIRenderer) -> Result<()> { Ok(()) }
    fn handle_input(&mut self, _input: &InputEvent) -> Result<UIResponse> { Ok(UIResponse::None) }
    fn on_enter(&mut self) -> Result<()> { Ok(()) }
    fn on_exit(&mut self) -> Result<()> { Ok(()) }
    fn get_name(&self) -> &str { "settings" }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_manager_creation() {
        let manager = UIManager::new();
        assert!(!manager.is_initialized);
        assert_eq!(manager.scale_factor, 1.0);
    }
    
    #[test]
    fn test_theme_system() {
        let mut manager = UIManager::new();
        manager.load_default_themes();
        
        assert!(manager.themes.contains_key("default"));
        assert!(manager.themes.contains_key("dark"));
        assert!(manager.themes.contains_key("pokemon"));
    }
}