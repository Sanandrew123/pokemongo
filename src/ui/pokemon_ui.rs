/*
 * 宝可梦界面系统 - Pokemon UI System
 * 
 * 开发心理过程：
 * 设计完整的宝可梦管理界面，包括队伍查看、详细属性、技能管理、状态查看等
 * 需要提供直观的宝可梦信息展示，支持交互操作如换位置、查看详情、学习技能等
 * 重点关注数据的清晰呈现和用户操作的便捷性
 */

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::pokemon::{Individual, PokemonId, Species, Move, MoveId, PokemonType, Stats};
use crate::player::party::{Party, PartySlot};
use crate::ui::menu::{UiTheme, MenuStyle};
use crate::core::error::GameResult;

// 宝可梦UI状态
#[derive(Debug, Clone, PartialEq)]
pub enum PokemonUiState {
    Closed,
    PartyView,      // 队伍总览
    Details,        // 详细信息
    Stats,          // 属性详情
    Moves,          // 技能管理
    Summary,        // 总结页面
}

// 宝可梦界面组件
#[derive(Component)]
pub struct PokemonUi {
    pub state: PokemonUiState,
    pub selected_slot: usize,
    pub selected_move: usize,
    pub view_mode: ViewMode,
    pub show_animations: bool,
    pub animation_timer: Timer,
    pub transition_progress: f32,
    pub scroll_offset: Vec2,
}

// 查看模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Grid,       // 网格视图
    List,       // 列表视图
    Card,       // 卡片视图
}

// 宝可梦显示信息
#[derive(Debug, Clone)]
pub struct PokemonDisplay {
    pub pokemon: Individual,
    pub position: Vec2,
    pub size: Vec2,
    pub is_selected: bool,
    pub is_fainted: bool,
    pub health_percentage: f32,
    pub animation_progress: f32,
    pub status_effects: Vec<String>,
}

// 宝可梦UI事件
#[derive(Debug, Clone)]
pub enum PokemonUiEvent {
    SlotSelected(usize),
    StateChanged(PokemonUiState),
    SwapRequested(usize, usize),
    MoveSelected(MoveId),
    StatsRequested(PokemonId),
    SummaryRequested(PokemonId),
    ViewModeChanged(ViewMode),
}

// 属性显示组件
#[derive(Debug, Clone)]
pub struct StatDisplay {
    pub name: String,
    pub base_value: u16,
    pub current_value: u16,
    pub iv_value: u8,
    pub ev_value: u16,
    pub nature_modifier: f32,
    pub color: Color,
}

// 技能显示组件
#[derive(Debug, Clone)]
pub struct MoveDisplay {
    pub move_data: Move,
    pub pp_current: u8,
    pub pp_max: u8,
    pub is_disabled: bool,
    pub power_display: String,
    pub accuracy_display: String,
    pub type_color: Color,
}

impl Default for PokemonUi {
    fn default() -> Self {
        Self {
            state: PokemonUiState::Closed,
            selected_slot: 0,
            selected_move: 0,
            view_mode: ViewMode::Grid,
            show_animations: true,
            animation_timer: Timer::from_seconds(0.3, TimerMode::Once),
            transition_progress: 0.0,
            scroll_offset: Vec2::ZERO,
        }
    }
}

// 宝可梦界面系统
pub struct PokemonUiSystem {
    theme: UiTheme,
    pokemon_displays: Vec<PokemonDisplay>,
    stat_displays: HashMap<String, StatDisplay>,
    move_displays: Vec<MoveDisplay>,
    ui_entities: Vec<Entity>,
    background_entity: Option<Entity>,
}

impl PokemonUiSystem {
    pub fn new(theme: UiTheme) -> Self {
        Self {
            theme,
            pokemon_displays: Vec::new(),
            stat_displays: HashMap::new(),
            move_displays: Vec::new(),
            ui_entities: Vec::new(),
            background_entity: None,
        }
    }

    // 打开宝可梦界面
    pub fn open(&mut self, commands: &mut Commands, party: &Party) -> GameResult<()> {
        self.update_pokemon_displays(party)?;
        self.create_ui(commands)?;
        Ok(())
    }

    // 关闭宝可梦界面
    pub fn close(&mut self, commands: &mut Commands) -> GameResult<()> {
        self.cleanup_ui(commands);
        Ok(())
    }

    // 创建UI界面
    fn create_ui(&mut self, commands: &mut Commands) -> GameResult<()> {
        // 创建背景
        let background = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        }).id();
        self.background_entity = Some(background);

        // 创建主容器
        let main_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(95.0),
                height: Val::Percent(90.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(2.5),
                top: Val::Percent(5.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: self.theme.panel_color.into(),
            ..default()
        }).id();

        // 创建标题栏
        self.create_header(commands, main_container)?;

        // 创建视图切换按钮
        self.create_view_controls(commands, main_container)?;

        // 创建内容区域
        self.create_content_area(commands, main_container)?;

        // 创建底部操作栏
        self.create_action_bar(commands, main_container)?;

        commands.entity(background).add_child(main_container);
        self.ui_entities.push(background);
        Ok(())
    }

    // 创建标题栏
    fn create_header(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let header = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: self.theme.header_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 标题文本
        commands.spawn(TextBundle::from_section(
            "宝可梦",
            TextStyle {
                font_size: 36.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(header);

        // 状态指示器
        let status_container = commands.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).id();

        // 队伍数量
        commands.spawn(TextBundle::from_section(
            format!("队伍: {}/6", self.pokemon_displays.len()),
            TextStyle {
                font_size: 18.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(status_container);

        // 关闭按钮
        commands.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: self.theme.danger_color.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "×",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                }
            ));
        }).set_parent(header);

        commands.entity(header).add_child(status_container);
        commands.entity(parent).add_child(header);
        Ok(())
    }

    // 创建视图控制
    fn create_view_controls(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let controls = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            ..default()
        }).id();

        let view_modes = [
            (ViewMode::Grid, "网格"),
            (ViewMode::List, "列表"),
            (ViewMode::Card, "卡片"),
        ];

        for (mode, label) in view_modes.iter() {
            let button = commands.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::horizontal(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: self.theme.button_color.into(),
                border_color: self.theme.border_color.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    *label,
                    TextStyle {
                        font_size: 14.0,
                        color: self.theme.text_color,
                        ..default()
                    }
                ));
            }).id();

            commands.entity(controls).add_child(button);
        }

        commands.entity(parent).add_child(controls);
        Ok(())
    }

    // 创建内容区域
    fn create_content_area(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let content = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).id();

        // 左侧宝可梦列表
        self.create_pokemon_grid(commands, content)?;

        // 右侧详细信息
        self.create_details_panel(commands, content)?;

        commands.entity(parent).add_child(content);
        Ok(())
    }

    // 创建宝可梦网格
    fn create_pokemon_grid(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let grid_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::right(Val::Px(2.0)),
                ..default()
            },
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 滚动容器
        let scroll_area = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                overflow: Overflow::clip(),
                ..default()
            },
            ..default()
        }).id();

        // 创建宝可梦卡片
        for (index, display) in self.pokemon_displays.iter().enumerate() {
            self.create_pokemon_card(commands, scroll_area, index, display)?;
        }

        // 创建空槽位
        for slot in self.pokemon_displays.len()..6 {
            self.create_empty_slot(commands, scroll_area, slot)?;
        }

        commands.entity(grid_container).add_child(scroll_area);
        commands.entity(parent).add_child(grid_container);
        Ok(())
    }

    // 创建宝可梦卡片
    fn create_pokemon_card(&self, commands: &mut Commands, parent: Entity, index: usize, display: &PokemonDisplay) -> GameResult<()> {
        let card = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(if display.is_selected { 3.0 } else { 1.0 })),
                ..default()
            },
            background_color: if display.is_fainted { 
                Color::rgba(0.5, 0.5, 0.5, 0.8) 
            } else { 
                self.theme.card_color 
            }.into(),
            border_color: if display.is_selected {
                self.theme.accent_color
            } else {
                self.theme.border_color
            }.into(),
            ..default()
        }).id();

        // 宝可梦图像
        commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }).set_parent(card);

        // 宝可梦名称
        commands.spawn(TextBundle::from_section(
            &display.pokemon.species.name,
            TextStyle {
                font_size: 16.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(card);

        // 等级
        commands.spawn(TextBundle::from_section(
            format!("Lv.{}", display.pokemon.level),
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(card);

        // 血量条
        self.create_health_bar(commands, card, display.health_percentage)?;

        // 状态异常
        if !display.status_effects.is_empty() {
            let status_text = display.status_effects.join(", ");
            commands.spawn(TextBundle::from_section(
                status_text,
                TextStyle {
                    font_size: 12.0,
                    color: self.theme.warning_color,
                    ..default()
                }
            )).set_parent(card);
        }

        commands.entity(parent).add_child(card);
        Ok(())
    }

    // 创建血量条
    fn create_health_bar(&self, commands: &mut Commands, parent: Entity, health_percentage: f32) -> GameResult<()> {
        let health_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(8.0),
                margin: UiRect::vertical(Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::BLACK.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        let health_color = if health_percentage > 0.6 {
            Color::GREEN
        } else if health_percentage > 0.2 {
            Color::YELLOW
        } else {
            Color::RED
        };

        commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(health_percentage * 100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: health_color.into(),
            ..default()
        }).set_parent(health_container);

        commands.entity(parent).add_child(health_container);
        Ok(())
    }

    // 创建空槽位
    fn create_empty_slot(&self, commands: &mut Commands, parent: Entity, slot: usize) -> GameResult<()> {
        let empty_card = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0.2, 0.2, 0.2, 0.5).into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        commands.spawn(TextBundle::from_section(
            "空槽位",
            TextStyle {
                font_size: 18.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(empty_card);

        commands.spawn(TextBundle::from_section(
            format!("#{}", slot + 1),
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(empty_card);

        commands.entity(parent).add_child(empty_card);
        Ok(())
    }

    // 创建详细信息面板
    fn create_details_panel(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let details_panel = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(40.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            ..default()
        }).id();

        if let Some(selected_pokemon) = self.get_selected_pokemon() {
            // 宝可梦基本信息
            self.create_pokemon_summary(commands, details_panel, selected_pokemon)?;

            // 属性面板
            self.create_stats_panel(commands, details_panel, selected_pokemon)?;

            // 技能面板
            self.create_moves_panel(commands, details_panel, selected_pokemon)?;
        } else {
            // 空状态
            commands.spawn(TextBundle::from_section(
                "选择一个宝可梦查看详情",
                TextStyle {
                    font_size: 20.0,
                    color: self.theme.secondary_text_color,
                    ..default()
                }
            )).set_parent(details_panel);
        }

        commands.entity(parent).add_child(details_panel);
        Ok(())
    }

    // 创建宝可梦总结
    fn create_pokemon_summary(&self, commands: &mut Commands, parent: Entity, pokemon: &Individual) -> GameResult<()> {
        let summary = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: self.theme.card_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 名称和等级
        commands.spawn(TextBundle::from_section(
            format!("{} Lv.{}", pokemon.species.name, pokemon.level),
            TextStyle {
                font_size: 24.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(summary);

        // 属性类型
        let type_text = if pokemon.species.type2.is_some() {
            format!("{}/{}", 
                format!("{:?}", pokemon.species.type1), 
                format!("{:?}", pokemon.species.type2.unwrap())
            )
        } else {
            format!("{:?}", pokemon.species.type1)
        };

        commands.spawn(TextBundle::from_section(
            type_text,
            TextStyle {
                font_size: 16.0,
                color: self.get_type_color(pokemon.species.type1),
                ..default()
            }
        )).set_parent(summary);

        // 性别和性格
        commands.spawn(TextBundle::from_section(
            format!("性格: {:?}", pokemon.nature),
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(summary);

        // 经验值
        commands.spawn(TextBundle::from_section(
            format!("经验: {}/{}", pokemon.experience, pokemon.experience_to_next_level()),
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(summary);

        commands.entity(parent).add_child(summary);
        Ok(())
    }

    // 创建属性面板
    fn create_stats_panel(&self, commands: &mut Commands, parent: Entity, pokemon: &Individual) -> GameResult<()> {
        let stats_panel = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: self.theme.card_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 标题
        commands.spawn(TextBundle::from_section(
            "属性值",
            TextStyle {
                font_size: 18.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(stats_panel);

        // 各项属性
        let stat_names = ["HP", "攻击", "防御", "特攻", "特防", "速度"];
        let stats = pokemon.get_stats();
        let stat_values = [
            stats.hp, stats.attack, stats.defense, 
            stats.special_attack, stats.special_defense, stats.speed
        ];

        for (i, (name, value)) in stat_names.iter().zip(stat_values.iter()).enumerate() {
            let stat_row = commands.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            }).id();

            commands.spawn(TextBundle::from_section(
                *name,
                TextStyle {
                    font_size: 14.0,
                    color: self.theme.text_color,
                    ..default()
                }
            )).set_parent(stat_row);

            commands.spawn(TextBundle::from_section(
                value.to_string(),
                TextStyle {
                    font_size: 14.0,
                    color: self.theme.accent_color,
                    ..default()
                }
            )).set_parent(stat_row);

            commands.entity(stats_panel).add_child(stat_row);
        }

        commands.entity(parent).add_child(stats_panel);
        Ok(())
    }

    // 创建技能面板
    fn create_moves_panel(&self, commands: &mut Commands, parent: Entity, pokemon: &Individual) -> GameResult<()> {
        let moves_panel = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: self.theme.card_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 标题
        commands.spawn(TextBundle::from_section(
            "技能",
            TextStyle {
                font_size: 18.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(moves_panel);

        // 技能列表
        for (i, move_slot) in pokemon.moves.iter().enumerate() {
            if let Some(move_data) = move_slot {
                let move_row = commands.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        margin: UiRect::vertical(Val::Px(3.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: if i == self.selected_move {
                        self.theme.accent_color.with_a(0.3)
                    } else {
                        Color::NONE
                    }.into(),
                    border_color: self.theme.border_color.into(),
                    ..default()
                }).id();

                // 技能名称
                commands.spawn(TextBundle::from_section(
                    &move_data.name,
                    TextStyle {
                        font_size: 14.0,
                        color: self.theme.text_color,
                        ..default()
                    }
                )).set_parent(move_row);

                // PP
                commands.spawn(TextBundle::from_section(
                    format!("{}/{}", move_data.pp, move_data.max_pp),
                    TextStyle {
                        font_size: 12.0,
                        color: self.theme.secondary_text_color,
                        ..default()
                    }
                )).set_parent(move_row);

                commands.entity(moves_panel).add_child(move_row);
            } else {
                // 空技能槽
                let empty_slot = commands.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.2, 0.2, 0.2, 0.3).into(),
                    border_color: self.theme.border_color.into(),
                    ..default()
                }).id();

                commands.spawn(TextBundle::from_section(
                    "空技能槽",
                    TextStyle {
                        font_size: 12.0,
                        color: self.theme.secondary_text_color,
                        ..default()
                    }
                )).set_parent(empty_slot);

                commands.entity(moves_panel).add_child(empty_slot);
            }
        }

        commands.entity(parent).add_child(moves_panel);
        Ok(())
    }

    // 创建操作栏
    fn create_action_bar(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let action_bar = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        let actions = [
            ("交换位置", self.theme.button_color),
            ("查看详情", self.theme.accent_color),
            ("技能管理", self.theme.button_color),
            ("释放", self.theme.danger_color),
        ];

        for (label, color) in actions.iter() {
            let button = commands.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(100.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::horizontal(Val::Px(5.0)),
                    ..default()
                },
                background_color: (*color).into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    *label,
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    }
                ));
            }).id();

            commands.entity(action_bar).add_child(button);
        }

        commands.entity(parent).add_child(action_bar);
        Ok(())
    }

    // 更新宝可梦显示
    pub fn update_pokemon_displays(&mut self, party: &Party) -> GameResult<()> {
        self.pokemon_displays.clear();

        for (slot_index, slot) in party.slots.iter().enumerate() {
            if let PartySlot::Occupied(pokemon) = slot {
                let current_hp = pokemon.current_hp as f32;
                let max_hp = pokemon.get_stats().hp as f32;
                let health_percentage = if max_hp > 0.0 { current_hp / max_hp } else { 0.0 };

                let mut status_effects = Vec::new();
                if pokemon.is_poisoned() {
                    status_effects.push("中毒".to_string());
                }
                if pokemon.is_paralyzed() {
                    status_effects.push("麻痹".to_string());
                }
                if pokemon.is_sleeping() {
                    status_effects.push("睡眠".to_string());
                }

                self.pokemon_displays.push(PokemonDisplay {
                    pokemon: pokemon.clone(),
                    position: Vec2::ZERO,
                    size: Vec2::new(150.0, 180.0),
                    is_selected: slot_index == self.selected_slot,
                    is_fainted: pokemon.current_hp == 0,
                    health_percentage,
                    animation_progress: 0.0,
                    status_effects,
                });
            }
        }

        Ok(())
    }

    // 处理输入
    pub fn handle_input(&mut self, input: &Input<KeyCode>) -> Option<PokemonUiEvent> {
        if input.just_pressed(KeyCode::Escape) {
            return Some(PokemonUiEvent::StateChanged(PokemonUiState::Closed));
        }

        if input.just_pressed(KeyCode::Left) && self.selected_slot > 0 {
            self.selected_slot -= 1;
            return Some(PokemonUiEvent::SlotSelected(self.selected_slot));
        }

        if input.just_pressed(KeyCode::Right) && self.selected_slot < 5 {
            self.selected_slot += 1;
            return Some(PokemonUiEvent::SlotSelected(self.selected_slot));
        }

        if input.just_pressed(KeyCode::Return) {
            if let Some(pokemon) = self.get_selected_pokemon() {
                return Some(PokemonUiEvent::StatsRequested(pokemon.id));
            }
        }

        if input.just_pressed(KeyCode::Tab) {
            let new_mode = match self.view_mode {
                ViewMode::Grid => ViewMode::List,
                ViewMode::List => ViewMode::Card,
                ViewMode::Card => ViewMode::Grid,
            };
            return Some(PokemonUiEvent::ViewModeChanged(new_mode));
        }

        None
    }

    // 获取选中的宝可梦
    fn get_selected_pokemon(&self) -> Option<&Individual> {
        self.pokemon_displays.get(self.selected_slot).map(|d| &d.pokemon)
    }

    // 获取属性类型颜色
    fn get_type_color(&self, pokemon_type: PokemonType) -> Color {
        match pokemon_type {
            PokemonType::Fire => Color::RED,
            PokemonType::Water => Color::BLUE,
            PokemonType::Grass => Color::GREEN,
            PokemonType::Electric => Color::YELLOW,
            PokemonType::Psychic => Color::PURPLE,
            PokemonType::Ice => Color::CYAN,
            PokemonType::Dragon => Color::VIOLET,
            PokemonType::Dark => Color::rgb(0.2, 0.2, 0.2),
            PokemonType::Fighting => Color::MAROON,
            PokemonType::Poison => Color::rgb(0.8, 0.0, 0.8),
            PokemonType::Ground => Color::rgb(0.8, 0.6, 0.2),
            PokemonType::Flying => Color::rgb(0.6, 0.8, 1.0),
            PokemonType::Bug => Color::rgb(0.6, 0.8, 0.2),
            PokemonType::Rock => Color::rgb(0.6, 0.4, 0.2),
            PokemonType::Ghost => Color::rgb(0.4, 0.2, 0.6),
            PokemonType::Steel => Color::SILVER,
            PokemonType::Fairy => Color::PINK,
            PokemonType::Normal => Color::rgb(0.7, 0.7, 0.7),
        }
    }

    // 清理UI
    fn cleanup_ui(&mut self, commands: &mut Commands) {
        for entity in self.ui_entities.drain(..) {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(bg) = self.background_entity.take() {
            commands.entity(bg).despawn_recursive();
        }
        self.pokemon_displays.clear();
        self.stat_displays.clear();
        self.move_displays.clear();
    }

    // 更新动画
    pub fn update_animations(&mut self, delta_time: f32) {
        self.animation_timer.tick(std::time::Duration::from_secs_f32(delta_time));

        for display in &mut self.pokemon_displays {
            if display.is_selected {
                display.animation_progress = (display.animation_progress + delta_time * 2.0) % 1.0;
            } else {
                display.animation_progress = 0.0;
            }
        }

        // 更新过渡动画
        if self.animation_timer.just_finished() {
            self.transition_progress = 1.0;
        } else {
            self.transition_progress = self.animation_timer.percent();
        }
    }

    // 设置视图模式
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    // 选择槽位
    pub fn select_slot(&mut self, slot: usize) {
        if slot < 6 {
            self.selected_slot = slot;
            // 更新选中状态
            for (index, display) in self.pokemon_displays.iter_mut().enumerate() {
                display.is_selected = index == slot;
            }
        }
    }
}

// Bevy系统实现
pub fn update_pokemon_ui_system(
    mut pokemon_ui: ResMut<PokemonUiSystem>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut event_writer: EventWriter<PokemonUiEvent>,
    party: Res<Party>,
) {
    // 更新动画
    pokemon_ui.update_animations(time.delta_seconds());

    // 处理输入
    if let Some(event) = pokemon_ui.handle_input(&input) {
        event_writer.send(event);
    }
}

pub fn handle_pokemon_ui_events_system(
    mut event_reader: EventReader<PokemonUiEvent>,
    mut pokemon_ui: ResMut<PokemonUiSystem>,
    party: Res<Party>,
) {
    for event in event_reader.iter() {
        match event {
            PokemonUiEvent::SlotSelected(slot) => {
                pokemon_ui.select_slot(*slot);
                let _ = pokemon_ui.update_pokemon_displays(&party);
            },
            PokemonUiEvent::ViewModeChanged(mode) => {
                pokemon_ui.set_view_mode(*mode);
            },
            PokemonUiEvent::StatsRequested(pokemon_id) => {
                info!("Stats requested for pokemon: {:?}", pokemon_id);
            },
            _ => {},
        }
    }
}