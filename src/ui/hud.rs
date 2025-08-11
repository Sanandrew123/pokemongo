/*
 * HUD界面系统 - Heads-Up Display System
 * 
 * 开发心理过程：
 * 设计游戏内的实时信息显示界面，包括血量条、经验条、小地图、快捷操作等
 * 需要考虑信息的实时更新、动画效果、布局适配和性能优化
 * 重点关注用户体验，提供直观而不干扰的信息展示
 */

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::pokemon::{Individual, PokemonId};
use crate::player::{Party, Trainer, TrainerId};
use crate::world::{Location, Weather, WeatherType};
use crate::ui::menu::{UiTheme, MenuStyle};
use crate::core::error::GameResult;

// HUD状态
#[derive(Debug, Clone, PartialEq)]
pub enum HudState {
    Hidden,         // 隐藏
    Minimal,        // 最小化显示
    Normal,         // 正常显示
    Extended,       // 扩展显示
}

// HUD组件类型
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum HudComponent {
    PartyStatus,    // 队伍状态
    PlayerInfo,     // 玩家信息
    MiniMap,        // 小地图
    WeatherInfo,    // 天气信息
    TimeInfo,       // 时间信息
    QuickActions,   // 快捷操作
    BattleHints,    // 战斗提示
    Notifications,  // 通知区域
}

// HUD主组件
#[derive(Component)]
pub struct GameHud {
    pub state: HudState,
    pub visible_components: HashMap<HudComponent, bool>,
    pub animation_progress: HashMap<HudComponent, f32>,
    pub update_timers: HashMap<HudComponent, Timer>,
    pub layout_config: HudLayoutConfig,
}

// HUD布局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudLayoutConfig {
    pub opacity: f32,
    pub scale: f32,
    pub auto_hide_delay: f32,
    pub compact_mode: bool,
    pub show_animations: bool,
    pub position_anchors: HashMap<HudComponent, AnchorPoint>,
}

// 锚点位置
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnchorPoint {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

// 队伍状态显示
#[derive(Debug, Clone)]
pub struct PartyStatusDisplay {
    pub pokemon_slots: Vec<PokemonSlotDisplay>,
    pub active_pokemon: Option<usize>,
    pub animation_timer: Timer,
}

// 宝可梦槽位显示
#[derive(Debug, Clone)]
pub struct PokemonSlotDisplay {
    pub pokemon: Option<Individual>,
    pub health_percentage: f32,
    pub status_icon: Option<String>,
    pub level: u8,
    pub is_active: bool,
    pub is_fainted: bool,
    pub animation_state: SlotAnimationState,
}

// 槽位动画状态
#[derive(Debug, Clone, PartialEq)]
pub enum SlotAnimationState {
    Normal,
    Damaged,
    Healing,
    LevelUp,
    Fainted,
}

// 玩家信息显示
#[derive(Debug, Clone)]
pub struct PlayerInfoDisplay {
    pub trainer_name: String,
    pub trainer_level: u8,
    pub money: u32,
    pub badge_count: u8,
    pub current_location: String,
    pub play_time: String,
}

// 小地图显示
#[derive(Debug, Clone)]
pub struct MiniMapDisplay {
    pub zoom_level: f32,
    pub player_position: Vec2,
    pub visible_radius: f32,
    pub show_npcs: bool,
    pub show_items: bool,
    pub show_pokemon: bool,
    pub map_data: Vec<MapTile>,
}

// 地图瓦片
#[derive(Debug, Clone)]
pub struct MapTile {
    pub position: Vec2,
    pub tile_type: TileType,
    pub color: Color,
}

// 瓦片类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Grass,
    Water,
    Road,
    Building,
    Tree,
    Rock,
    Sand,
}

// 天气信息显示
#[derive(Debug, Clone)]
pub struct WeatherDisplay {
    pub current_weather: WeatherType,
    pub temperature: f32,
    pub humidity: f32,
    pub wind_speed: f32,
    pub forecast: Vec<WeatherForecast>,
}

// 天气预报
#[derive(Debug, Clone)]
pub struct WeatherForecast {
    pub time: String,
    pub weather: WeatherType,
    pub temperature: f32,
}

// 通知系统
#[derive(Debug, Clone)]
pub struct NotificationDisplay {
    pub notifications: Vec<Notification>,
    pub max_notifications: usize,
    pub auto_clear_time: f32,
}

// 通知信息
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_time: f32,
    pub lifetime: f32,
    pub is_persistent: bool,
}

// 通知类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    Achievement,
    ItemFound,
    PokemonCaught,
}

impl Default for GameHud {
    fn default() -> Self {
        let mut visible_components = HashMap::new();
        visible_components.insert(HudComponent::PartyStatus, true);
        visible_components.insert(HudComponent::PlayerInfo, true);
        visible_components.insert(HudComponent::MiniMap, true);
        visible_components.insert(HudComponent::WeatherInfo, false);
        visible_components.insert(HudComponent::TimeInfo, true);
        visible_components.insert(HudComponent::QuickActions, true);
        visible_components.insert(HudComponent::BattleHints, false);
        visible_components.insert(HudComponent::Notifications, true);

        Self {
            state: HudState::Normal,
            visible_components,
            animation_progress: HashMap::new(),
            update_timers: HashMap::new(),
            layout_config: HudLayoutConfig::default(),
        }
    }
}

impl Default for HudLayoutConfig {
    fn default() -> Self {
        let mut anchors = HashMap::new();
        anchors.insert(HudComponent::PartyStatus, AnchorPoint::BottomLeft);
        anchors.insert(HudComponent::PlayerInfo, AnchorPoint::TopLeft);
        anchors.insert(HudComponent::MiniMap, AnchorPoint::TopRight);
        anchors.insert(HudComponent::WeatherInfo, AnchorPoint::TopCenter);
        anchors.insert(HudComponent::TimeInfo, AnchorPoint::TopCenter);
        anchors.insert(HudComponent::QuickActions, AnchorPoint::BottomRight);
        anchors.insert(HudComponent::BattleHints, AnchorPoint::Center);
        anchors.insert(HudComponent::Notifications, AnchorPoint::CenterRight);

        Self {
            opacity: 0.9,
            scale: 1.0,
            auto_hide_delay: 5.0,
            compact_mode: false,
            show_animations: true,
            position_anchors: anchors,
        }
    }
}

// HUD系统
pub struct HudSystem {
    theme: UiTheme,
    ui_entities: HashMap<HudComponent, Entity>,
    party_display: PartyStatusDisplay,
    player_display: PlayerInfoDisplay,
    minimap_display: MiniMapDisplay,
    weather_display: WeatherDisplay,
    notification_display: NotificationDisplay,
    root_entity: Option<Entity>,
}

impl HudSystem {
    pub fn new(theme: UiTheme) -> Self {
        Self {
            theme,
            ui_entities: HashMap::new(),
            party_display: PartyStatusDisplay {
                pokemon_slots: Vec::new(),
                active_pokemon: None,
                animation_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            player_display: PlayerInfoDisplay {
                trainer_name: "训练师".to_string(),
                trainer_level: 1,
                money: 0,
                badge_count: 0,
                current_location: "家乡镇".to_string(),
                play_time: "00:00:00".to_string(),
            },
            minimap_display: MiniMapDisplay {
                zoom_level: 1.0,
                player_position: Vec2::ZERO,
                visible_radius: 100.0,
                show_npcs: true,
                show_items: true,
                show_pokemon: true,
                map_data: Vec::new(),
            },
            weather_display: WeatherDisplay {
                current_weather: WeatherType::Clear,
                temperature: 20.0,
                humidity: 50.0,
                wind_speed: 5.0,
                forecast: Vec::new(),
            },
            notification_display: NotificationDisplay {
                notifications: Vec::new(),
                max_notifications: 5,
                auto_clear_time: 5.0,
            },
            root_entity: None,
        }
    }

    // 初始化HUD
    pub fn initialize(&mut self, commands: &mut Commands) -> GameResult<()> {
        // 创建根容器
        let root = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        }).id();
        
        self.root_entity = Some(root);

        // 创建各个组件
        self.create_party_status(commands, root)?;
        self.create_player_info(commands, root)?;
        self.create_minimap(commands, root)?;
        self.create_weather_info(commands, root)?;
        self.create_time_info(commands, root)?;
        self.create_quick_actions(commands, root)?;
        self.create_notifications(commands, root)?;

        Ok(())
    }

    // 创建队伍状态显示
    fn create_party_status(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let party_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(80.0),
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.8).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 创建6个宝可梦槽位
        for i in 0..6 {
            let slot = self.create_pokemon_slot(commands, i)?;
            commands.entity(party_container).add_child(slot);
        }

        commands.entity(parent).add_child(party_container);
        self.ui_entities.insert(HudComponent::PartyStatus, party_container);
        Ok(())
    }

    // 创建宝可梦槽位
    fn create_pokemon_slot(&self, commands: &mut Commands, index: usize) -> GameResult<Entity> {
        let slot = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(45.0),
                height: Val::Px(65.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::horizontal(Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.2, 0.2, 0.2, 0.9).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 宝可梦图标
        let icon = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(35.0),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }).id();

        // 血量条
        let health_bg = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(90.0),
                height: Val::Px(4.0),
                margin: UiRect::horizontal(Val::Percent(5.0)),
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        }).id();

        let health_bar = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::GREEN.into(),
            ..default()
        }).id();

        // 状态指示器
        let status_indicator = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).id();

        let level_text = commands.spawn(TextBundle::from_section(
            format!("L{}", 1),
            TextStyle {
                font_size: 10.0,
                color: Color::WHITE,
                ..default()
            }
        )).id();

        commands.entity(health_bg).add_child(health_bar);
        commands.entity(status_indicator).add_child(level_text);
        commands.entity(slot).add_child(icon);
        commands.entity(slot).add_child(health_bg);
        commands.entity(slot).add_child(status_indicator);

        Ok(slot)
    }

    // 创建玩家信息
    fn create_player_info(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let info_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.8).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 训练师名称
        let name_text = commands.spawn(TextBundle::from_section(
            &self.player_display.trainer_name,
            TextStyle {
                font_size: 16.0,
                color: self.theme.text_color,
                ..default()
            }
        )).id();

        // 金钱
        let money_text = commands.spawn(TextBundle::from_section(
            format!("¥{}", self.player_display.money),
            TextStyle {
                font_size: 14.0,
                color: Color::YELLOW,
                ..default()
            }
        )).id();

        // 徽章数量
        let badges_text = commands.spawn(TextBundle::from_section(
            format!("徽章: {}/8", self.player_display.badge_count),
            TextStyle {
                font_size: 12.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).id();

        // 当前位置
        let location_text = commands.spawn(TextBundle::from_section(
            &self.player_display.current_location,
            TextStyle {
                font_size: 12.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).id();

        commands.entity(info_container).add_child(name_text);
        commands.entity(info_container).add_child(money_text);
        commands.entity(info_container).add_child(badges_text);
        commands.entity(info_container).add_child(location_text);
        commands.entity(parent).add_child(info_container);
        self.ui_entities.insert(HudComponent::PlayerInfo, info_container);
        Ok(())
    }

    // 创建小地图
    fn create_minimap(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let minimap_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(150.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.9).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 地图区域
        let map_area = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::rgba(0.2, 0.4, 0.2, 1.0).into(),
            ..default()
        }).id();

        // 玩家位置指示器
        let player_marker = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(6.0),
                height: Val::Px(6.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                ..default()
            },
            background_color: Color::RED.into(),
            ..default()
        }).id();

        commands.entity(minimap_container).add_child(map_area);
        commands.entity(minimap_container).add_child(player_marker);
        commands.entity(parent).add_child(minimap_container);
        self.ui_entities.insert(HudComponent::MiniMap, minimap_container);
        Ok(())
    }

    // 创建天气信息
    fn create_weather_info(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let weather_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(120.0),
                height: Val::Px(60.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(40.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.7).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 天气图标
        let weather_icon = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::YELLOW.into(),
            ..default()
        }).id();

        // 温度文本
        let temp_text = commands.spawn(TextBundle::from_section(
            format!("{}°C", self.weather_display.temperature as i32),
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            }
        )).id();

        commands.entity(weather_container).add_child(weather_icon);
        commands.entity(weather_container).add_child(temp_text);
        commands.entity(parent).add_child(weather_container);
        self.ui_entities.insert(HudComponent::WeatherInfo, weather_container);
        Ok(())
    }

    // 创建时间信息
    fn create_time_info(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let time_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(30.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(45.0),
                top: Val::Px(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.7).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        let time_text = commands.spawn(TextBundle::from_section(
            &self.player_display.play_time,
            TextStyle {
                font_size: 12.0,
                color: Color::WHITE,
                ..default()
            }
        )).id();

        commands.entity(time_container).add_child(time_text);
        commands.entity(parent).add_child(time_container);
        self.ui_entities.insert(HudComponent::TimeInfo, time_container);
        Ok(())
    }

    // 创建快捷操作
    fn create_quick_actions(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let actions_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(60.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).id();

        let actions = [
            ("🎒", "背包"),
            ("📱", "宝可梦"),
            ("💾", "保存"),
            ("⚙️", "设置"),
        ];

        for (icon, tooltip) in actions.iter() {
            let action_button = commands.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(50.0),
                    height: Val::Px(40.0),
                    margin: UiRect::bottom(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: Color::rgba(0.1, 0.1, 0.1, 0.8).into(),
                border_color: self.theme.border_color.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    *icon,
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    }
                ));
            }).id();

            commands.entity(actions_container).add_child(action_button);
        }

        commands.entity(parent).add_child(actions_container);
        self.ui_entities.insert(HudComponent::QuickActions, actions_container);
        Ok(())
    }

    // 创建通知区域
    fn create_notifications(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let notifications_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).id();

        commands.entity(parent).add_child(notifications_container);
        self.ui_entities.insert(HudComponent::Notifications, notifications_container);
        Ok(())
    }

    // 更新队伍状态
    pub fn update_party_status(&mut self, party: &Party) -> GameResult<()> {
        self.party_display.pokemon_slots.clear();
        
        for (index, slot) in party.slots.iter().enumerate() {
            let display = match slot {
                crate::player::party::PartySlot::Occupied(pokemon) => {
                    let current_hp = pokemon.current_hp as f32;
                    let max_hp = pokemon.get_stats().hp as f32;
                    let health_percentage = if max_hp > 0.0 { current_hp / max_hp } else { 0.0 };

                    let animation_state = if pokemon.current_hp == 0 {
                        SlotAnimationState::Fainted
                    } else if health_percentage < 0.25 {
                        SlotAnimationState::Damaged
                    } else {
                        SlotAnimationState::Normal
                    };

                    PokemonSlotDisplay {
                        pokemon: Some(pokemon.clone()),
                        health_percentage,
                        status_icon: None, // TODO: 获取状态异常图标
                        level: pokemon.level,
                        is_active: index == 0,
                        is_fainted: pokemon.current_hp == 0,
                        animation_state,
                    }
                },
                crate::player::party::PartySlot::Empty => {
                    PokemonSlotDisplay {
                        pokemon: None,
                        health_percentage: 0.0,
                        status_icon: None,
                        level: 0,
                        is_active: false,
                        is_fainted: false,
                        animation_state: SlotAnimationState::Normal,
                    }
                },
            };
            
            self.party_display.pokemon_slots.push(display);
        }

        Ok(())
    }

    // 更新玩家信息
    pub fn update_player_info(&mut self, trainer: &Trainer, location: &str, play_time: &str) -> GameResult<()> {
        self.player_display.trainer_name = trainer.name.clone();
        self.player_display.trainer_level = trainer.level.level;
        self.player_display.money = trainer.money;
        self.player_display.badge_count = trainer.badges.len() as u8;
        self.player_display.current_location = location.to_string();
        self.player_display.play_time = play_time.to_string();
        Ok(())
    }

    // 更新天气信息
    pub fn update_weather_info(&mut self, weather: &Weather) -> GameResult<()> {
        self.weather_display.current_weather = weather.current_weather;
        self.weather_display.temperature = weather.temperature;
        self.weather_display.humidity = weather.humidity;
        self.weather_display.wind_speed = weather.wind_speed;
        Ok(())
    }

    // 添加通知
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) -> GameResult<()> {
        let notification = Notification {
            id: self.notification_display.notifications.len() as u32,
            message,
            notification_type,
            created_time: 0.0, // TODO: 获取当前时间
            lifetime: match notification_type {
                NotificationType::Achievement => 10.0,
                NotificationType::Error => 8.0,
                _ => self.notification_display.auto_clear_time,
            },
            is_persistent: matches!(notification_type, NotificationType::Error),
        };

        self.notification_display.notifications.push(notification);

        // 限制通知数量
        if self.notification_display.notifications.len() > self.notification_display.max_notifications {
            self.notification_display.notifications.remove(0);
        }

        Ok(())
    }

    // 清除过期通知
    pub fn clear_expired_notifications(&mut self, current_time: f32) {
        self.notification_display.notifications.retain(|notification| {
            if notification.is_persistent {
                true
            } else {
                current_time - notification.created_time < notification.lifetime
            }
        });
    }

    // 设置HUD状态
    pub fn set_state(&mut self, state: HudState) {
        // TODO: 实现状态切换动画
        // self.state = state;
    }

    // 切换组件可见性
    pub fn toggle_component(&mut self, component: HudComponent) {
        if let Some(visible) = self.visible_components.get_mut(&component) {
            *visible = !*visible;
        }
    }

    // 设置组件可见性
    pub fn set_component_visible(&mut self, component: HudComponent, visible: bool) {
        self.visible_components.insert(component, visible);
    }

    // 更新小地图
    pub fn update_minimap(&mut self, player_pos: Vec2, map_data: Vec<MapTile>) {
        self.minimap_display.player_position = player_pos;
        self.minimap_display.map_data = map_data;
    }

    // 清理HUD
    pub fn cleanup(&mut self, commands: &mut Commands) {
        if let Some(root) = self.root_entity.take() {
            commands.entity(root).despawn_recursive();
        }
        self.ui_entities.clear();
    }

    // 获取通知颜色
    fn get_notification_color(&self, notification_type: NotificationType) -> Color {
        match notification_type {
            NotificationType::Info => self.theme.text_color,
            NotificationType::Warning => Color::YELLOW,
            NotificationType::Error => Color::RED,
            NotificationType::Success => Color::GREEN,
            NotificationType::Achievement => Color::GOLD,
            NotificationType::ItemFound => Color::CYAN,
            NotificationType::PokemonCaught => Color::LIME_GREEN,
        }
    }
}

// HUD事件
#[derive(Debug, Clone)]
pub enum HudEvent {
    QuickActionPressed(String),     // 快捷操作按下
    NotificationDismissed(u32),     // 通知被关闭
    ComponentToggled(HudComponent), // 组件可见性切换
    StateChanged(HudState),         // 状态改变
}

// Bevy系统实现
pub fn update_hud_system(
    mut hud_system: ResMut<HudSystem>,
    time: Res<Time>,
    party: Res<Party>,
    trainer: Res<Trainer>,
    // weather: Res<Weather>,
) {
    // 更新队伍状态
    let _ = hud_system.update_party_status(&party);

    // 更新玩家信息
    let play_time = format!("{:02}:{:02}:{:02}", 
        (time.elapsed_seconds() / 3600.0) as u32,
        ((time.elapsed_seconds() % 3600.0) / 60.0) as u32,
        (time.elapsed_seconds() % 60.0) as u32
    );
    let _ = hud_system.update_player_info(&trainer, "当前位置", &play_time);

    // 清除过期通知
    hud_system.clear_expired_notifications(time.elapsed_seconds());
}

pub fn handle_hud_events_system(
    mut event_reader: EventReader<HudEvent>,
    mut hud_system: ResMut<HudSystem>,
) {
    for event in event_reader.iter() {
        match event {
            HudEvent::QuickActionPressed(action) => {
                info!("Quick action pressed: {}", action);
            },
            HudEvent::ComponentToggled(component) => {
                hud_system.toggle_component(*component);
            },
            HudEvent::StateChanged(state) => {
                hud_system.set_state(state.clone());
            },
            _ => {}
        }
    }
}