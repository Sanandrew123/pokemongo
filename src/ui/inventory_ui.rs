/*
 * 背包界面系统 - Inventory UI System
 * 
 * 开发心理过程：
 * 设计一个功能完整的背包界面，包括物品分类、搜索过滤、使用操作等
 * 需要考虑不同物品类型的显示和交互方式，以及与游戏核心系统的集成
 * 重点关注用户体验，提供直观的物品管理功能
 */

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::player::inventory::{Inventory, Item, ItemType, ItemId, ItemStack};
use crate::ui::menu::{UiTheme, MenuStyle};
use crate::core::error::GameResult;

// 背包界面状态
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryUiState {
    Closed,
    Items,      // 道具页面
    KeyItems,   // 关键道具页面
    Pokeballs,  // 精灵球页面
    Berries,    // 树果页面
}

// 背包界面组件
#[derive(Component)]
pub struct InventoryUi {
    pub state: InventoryUiState,
    pub selected_category: ItemType,
    pub selected_item_index: usize,
    pub search_query: String,
    pub sort_mode: SortMode,
    pub show_details: bool,
    pub animation_timer: Timer,
    pub scroll_offset: f32,
}

// 排序模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortMode {
    Name,       // 按名称排序
    Type,       // 按类型排序
    Quantity,   // 按数量排序
    Recent,     // 按最近获得排序
    Value,      // 按价值排序
}

// 物品显示信息
#[derive(Debug, Clone)]
pub struct ItemDisplay {
    pub item: Item,
    pub stack: ItemStack,
    pub position: Vec2,
    pub size: Vec2,
    pub is_selected: bool,
    pub is_usable: bool,
    pub animation_progress: f32,
}

// 背包界面事件
#[derive(Debug, Clone)]
pub enum InventoryUiEvent {
    CategoryChanged(ItemType),
    ItemSelected(ItemId),
    ItemUsed(ItemId),
    ItemDropped(ItemId, u32),
    SortChanged(SortMode),
    SearchChanged(String),
    DetailsToggled,
}

// 物品操作菜单
#[derive(Debug, Clone)]
pub struct ItemActionMenu {
    pub item_id: ItemId,
    pub actions: Vec<ItemAction>,
    pub selected_action: usize,
    pub position: Vec2,
    pub visible: bool,
}

// 物品操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum ItemAction {
    Use,        // 使用
    Give,       // 给予宝可梦
    Drop,       // 丢弃
    Details,    // 查看详情
    Sort,       // 排序到前面
}

impl Default for InventoryUi {
    fn default() -> Self {
        Self {
            state: InventoryUiState::Closed,
            selected_category: ItemType::Item,
            selected_item_index: 0,
            search_query: String::new(),
            sort_mode: SortMode::Type,
            show_details: false,
            animation_timer: Timer::from_seconds(0.3, TimerMode::Once),
            scroll_offset: 0.0,
        }
    }
}

// 背包界面系统
pub struct InventoryUiSystem {
    theme: UiTheme,
    item_displays: Vec<ItemDisplay>,
    action_menu: Option<ItemActionMenu>,
    category_buttons: HashMap<ItemType, Entity>,
    search_input: Option<Entity>,
    sort_dropdown: Option<Entity>,
}

impl InventoryUiSystem {
    pub fn new(theme: UiTheme) -> Self {
        Self {
            theme,
            item_displays: Vec::new(),
            action_menu: None,
            category_buttons: HashMap::new(),
            search_input: None,
            sort_dropdown: None,
        }
    }

    // 打开背包界面
    pub fn open(&mut self, commands: &mut Commands) -> GameResult<()> {
        self.create_ui(commands)?;
        Ok(())
    }

    // 关闭背包界面
    pub fn close(&mut self, commands: &mut Commands) -> GameResult<()> {
        self.cleanup_ui(commands);
        Ok(())
    }

    // 创建UI界面
    fn create_ui(&mut self, commands: &mut Commands) -> GameResult<()> {
        // 创建主容器
        let container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(90.0),
                height: Val::Percent(85.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(5.0),
                top: Val::Percent(7.5),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: self.theme.panel_color.into(),
            ..default()
        }).id();

        // 创建标题栏
        self.create_header(commands, container)?;

        // 创建分类选项卡
        self.create_category_tabs(commands, container)?;

        // 创建搜索和排序控件
        self.create_controls(commands, container)?;

        // 创建物品网格
        self.create_item_grid(commands, container)?;

        // 创建详情面板
        self.create_details_panel(commands, container)?;

        Ok(())
    }

    // 创建标题栏
    fn create_header(&self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let header = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: self.theme.header_color.into(),
            ..default()
        }).id();

        // 标题文本
        commands.spawn(TextBundle::from_section(
            "背包",
            TextStyle {
                font_size: 32.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(header);

        // 关闭按钮
        commands.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: self.theme.button_color.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "×",
                TextStyle {
                    font_size: 24.0,
                    color: self.theme.text_color,
                    ..default()
                }
            ));
        }).set_parent(header);

        commands.entity(parent).add_child(header);
        Ok(())
    }

    // 创建分类选项卡
    fn create_category_tabs(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let tabs_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).id();

        let categories = vec![
            (ItemType::Item, "道具"),
            (ItemType::KeyItem, "关键道具"),
            (ItemType::Pokeball, "精灵球"),
            (ItemType::Berry, "树果"),
        ];

        for (item_type, label) in categories {
            let button = commands.spawn(ButtonBundle {
                style: Style {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: self.theme.button_color.into(),
                border_color: self.theme.border_color.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: 18.0,
                        color: self.theme.text_color,
                        ..default()
                    }
                ));
            }).id();

            self.category_buttons.insert(item_type, button);
            commands.entity(tabs_container).add_child(button);
        }

        commands.entity(parent).add_child(tabs_container);
        Ok(())
    }

    // 创建控制面板
    fn create_controls(&mut self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let controls = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            ..default()
        }).id();

        // 搜索输入框
        let search_input = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(35.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::WHITE.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLACK,
                    ..default()
                }
            ));
        }).id();

        self.search_input = Some(search_input);

        // 排序下拉框
        let sort_dropdown = commands.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(120.0),
                height: Val::Px(35.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: self.theme.button_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "按类型排序",
                TextStyle {
                    font_size: 16.0,
                    color: self.theme.text_color,
                    ..default()
                }
            ));
        }).id();

        self.sort_dropdown = Some(sort_dropdown);

        commands.entity(controls).add_child(search_input);
        commands.entity(controls).add_child(sort_dropdown);
        commands.entity(parent).add_child(controls);
        Ok(())
    }

    // 创建物品网格
    fn create_item_grid(&self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let grid_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            ..default()
        }).id();

        // 滚动容器
        let scroll_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        }).id();

        commands.entity(grid_container).add_child(scroll_container);
        commands.entity(parent).add_child(grid_container);
        Ok(())
    }

    // 创建详情面板
    fn create_details_panel(&self, commands: &mut Commands, parent: Entity) -> GameResult<()> {
        let details_panel = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 物品图标
        commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                margin: UiRect::right(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }).set_parent(details_panel);

        // 物品信息
        let info_container = commands.spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        }).id();

        // 物品名称
        commands.spawn(TextBundle::from_section(
            "选择一个物品查看详情",
            TextStyle {
                font_size: 20.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(info_container);

        // 物品描述
        commands.spawn(TextBundle::from_section(
            "",
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(info_container);

        // 数量和价值
        commands.spawn(TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(info_container);

        commands.entity(details_panel).add_child(info_container);
        commands.entity(parent).add_child(details_panel);
        Ok(())
    }

    // 更新物品显示
    pub fn update_items(&mut self, inventory: &Inventory, category: ItemType) -> GameResult<()> {
        self.item_displays.clear();

        let mut items: Vec<_> = inventory.get_items_by_type(category)
            .into_iter()
            .collect();

        // 排序
        match self.sort_mode {
            SortMode::Name => items.sort_by(|(_, a), (_, b)| a.item.name.cmp(&b.item.name)),
            SortMode::Type => items.sort_by(|(_, a), (_, b)| a.item.item_type.cmp(&b.item.item_type)),
            SortMode::Quantity => items.sort_by(|(_, a), (_, b)| b.quantity.cmp(&a.quantity)),
            SortMode::Value => items.sort_by(|(_, a), (_, b)| 
                (b.item.value * b.quantity as u32).cmp(&(a.item.value * a.quantity as u32))),
            SortMode::Recent => items.sort_by(|(_, a), (_, b)| b.last_used.cmp(&a.last_used)),
        }

        // 搜索过滤
        if !self.search_query.is_empty() {
            items.retain(|(_, stack)| {
                stack.item.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                stack.item.description.to_lowercase().contains(&self.search_query.to_lowercase())
            });
        }

        // 创建显示项
        for (index, (item_id, stack)) in items.iter().enumerate() {
            let x = (index % 8) as f32 * 80.0;
            let y = (index / 8) as f32 * 80.0;

            self.item_displays.push(ItemDisplay {
                item: stack.item.clone(),
                stack: (*stack).clone(),
                position: Vec2::new(x, y),
                size: Vec2::new(75.0, 75.0),
                is_selected: index == self.selected_item_index,
                is_usable: stack.item.is_usable(),
                animation_progress: 0.0,
            });
        }

        Ok(())
    }

    // 处理输入
    pub fn handle_input(&mut self, input: &Input<KeyCode>) -> Option<InventoryUiEvent> {
        if input.just_pressed(KeyCode::Escape) {
            return Some(InventoryUiEvent::DetailsToggled);
        }

        if input.just_pressed(KeyCode::Tab) {
            let categories = [ItemType::Item, ItemType::KeyItem, ItemType::Pokeball, ItemType::Berry];
            let current_index = categories.iter().position(|&t| t == self.selected_category).unwrap_or(0);
            let next_index = (current_index + 1) % categories.len();
            self.selected_category = categories[next_index];
            return Some(InventoryUiEvent::CategoryChanged(self.selected_category));
        }

        if input.just_pressed(KeyCode::Return) {
            if !self.item_displays.is_empty() && self.selected_item_index < self.item_displays.len() {
                let selected_item = &self.item_displays[self.selected_item_index];
                return Some(InventoryUiEvent::ItemSelected(selected_item.stack.item.id));
            }
        }

        // 方向键导航
        if input.just_pressed(KeyCode::Left) {
            if self.selected_item_index > 0 {
                self.selected_item_index -= 1;
            }
        } else if input.just_pressed(KeyCode::Right) {
            if self.selected_item_index < self.item_displays.len().saturating_sub(1) {
                self.selected_item_index += 1;
            }
        } else if input.just_pressed(KeyCode::Up) {
            if self.selected_item_index >= 8 {
                self.selected_item_index -= 8;
            }
        } else if input.just_pressed(KeyCode::Down) {
            if self.selected_item_index + 8 < self.item_displays.len() {
                self.selected_item_index += 8;
            }
        }

        None
    }

    // 使用物品
    pub fn use_item(&mut self, item_id: ItemId) -> GameResult<()> {
        // 实现物品使用逻辑
        Ok(())
    }

    // 显示物品操作菜单
    pub fn show_action_menu(&mut self, item_id: ItemId, position: Vec2) {
        let mut actions = vec![ItemAction::Details];

        // 根据物品类型添加可用操作
        if let Some(display) = self.item_displays.iter().find(|d| d.stack.item.id == item_id) {
            if display.is_usable {
                actions.insert(0, ItemAction::Use);
            }

            if display.item.item_type != ItemType::KeyItem {
                actions.push(ItemAction::Drop);
            }

            if display.item.can_give_to_pokemon() {
                actions.insert(1, ItemAction::Give);
            }
        }

        self.action_menu = Some(ItemActionMenu {
            item_id,
            actions,
            selected_action: 0,
            position,
            visible: true,
        });
    }

    // 隐藏操作菜单
    pub fn hide_action_menu(&mut self) {
        self.action_menu = None;
    }

    // 清理UI
    fn cleanup_ui(&mut self, commands: &mut Commands) {
        // 清理UI实体
        self.category_buttons.clear();
        self.search_input = None;
        self.sort_dropdown = None;
        self.action_menu = None;
        self.item_displays.clear();
    }

    // 更新动画
    pub fn update_animations(&mut self, delta_time: f32) {
        self.animation_timer.tick(std::time::Duration::from_secs_f32(delta_time));

        for display in &mut self.item_displays {
            if display.is_selected {
                display.animation_progress = (display.animation_progress + delta_time * 3.0) % 1.0;
            } else {
                display.animation_progress = 0.0;
            }
        }
    }

    // 获取选中的物品
    pub fn get_selected_item(&self) -> Option<&ItemDisplay> {
        if self.selected_item_index < self.item_displays.len() {
            Some(&self.item_displays[self.selected_item_index])
        } else {
            None
        }
    }

    // 设置搜索查询
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    // 设置排序模式
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }
}

// Bevy系统实现
pub fn update_inventory_ui_system(
    mut inventory_ui: ResMut<InventoryUiSystem>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut event_writer: EventWriter<InventoryUiEvent>,
) {
    // 更新动画
    inventory_ui.update_animations(time.delta_seconds());

    // 处理输入
    if let Some(event) = inventory_ui.handle_input(&input) {
        event_writer.send(event);
    }
}

pub fn handle_inventory_events_system(
    mut event_reader: EventReader<InventoryUiEvent>,
    mut inventory_ui: ResMut<InventoryUiSystem>,
    inventory: Res<Inventory>,
) {
    for event in event_reader.iter() {
        match event {
            InventoryUiEvent::CategoryChanged(category) => {
                let _ = inventory_ui.update_items(&inventory, *category);
            },
            InventoryUiEvent::ItemSelected(item_id) => {
                info!("Selected item: {:?}", item_id);
            },
            InventoryUiEvent::ItemUsed(item_id) => {
                let _ = inventory_ui.use_item(*item_id);
            },
            InventoryUiEvent::SortChanged(sort_mode) => {
                inventory_ui.set_sort_mode(*sort_mode);
                let _ = inventory_ui.update_items(&inventory, inventory_ui.selected_category);
            },
            InventoryUiEvent::SearchChanged(query) => {
                inventory_ui.set_search_query(query.clone());
                let _ = inventory_ui.update_items(&inventory, inventory_ui.selected_category);
            },
            _ => {},
        }
    }
}