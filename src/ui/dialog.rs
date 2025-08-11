/*
 * 对话系统 - Dialog System
 * 
 * 开发心理过程：
 * 设计一个功能完整的对话系统，支持NPC对话、剧情对话、选择分支等
 * 需要处理文本显示动画、音效同步、对话树结构和状态管理
 * 重点关注用户体验，提供流畅的对话交互和丰富的表现形式
 */

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::ui::menu::{UiTheme, MenuStyle};
use crate::core::error::GameResult;

// 对话状态
#[derive(Debug, Clone, PartialEq)]
pub enum DialogState {
    Hidden,         // 隐藏状态
    Showing,        // 正在显示
    WaitingInput,   // 等待用户输入
    Choosing,       // 等待选择
    Closing,        // 正在关闭
}

// 对话类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DialogType {
    NpcDialog,      // NPC对话
    SystemMessage,  // 系统消息
    StoryDialog,    // 剧情对话
    BattleDialog,   // 战斗对话
    ItemDialog,     // 物品获得对话
}

// 对话节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogNode {
    pub id: String,
    pub speaker: Option<String>,      // 说话者名称
    pub text: String,                 // 对话文本
    pub character_portrait: Option<String>, // 角色立绘
    pub voice_clip: Option<String>,   // 语音片段
    pub animation: Option<String>,    // 动画效果
    pub choices: Vec<DialogChoice>,   // 选择选项
    pub next_node: Option<String>,    // 下一个节点ID
    pub conditions: Vec<DialogCondition>, // 显示条件
    pub effects: Vec<DialogEffect>,   // 对话效果
    pub delay: f32,                   // 延迟时间
}

// 对话选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogChoice {
    pub id: String,
    pub text: String,
    pub next_node: Option<String>,
    pub conditions: Vec<DialogCondition>,
    pub effects: Vec<DialogEffect>,
    pub is_disabled: bool,
}

// 对话条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogCondition {
    HasItem(String, u32),           // 拥有指定数量物品
    PokemonInParty(String),         // 队伍中有指定宝可梦
    BadgeCount(u8),                 // 徽章数量
    VariableEquals(String, i32),    // 变量值等于
    VariableGreater(String, i32),   // 变量值大于
    FlagSet(String),                // 标志位设置
}

// 对话效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogEffect {
    SetVariable(String, i32),       // 设置变量
    SetFlag(String, bool),          // 设置标志
    GiveItem(String, u32),          // 给予物品
    GivePokemon(String),            // 给予宝可梦
    PlaySound(String),              // 播放音效
    StartBattle(String),            // 开始战斗
    ChangeScene(String),            // 切换场景
}

// 对话树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogTree {
    pub id: String,
    pub name: String,
    pub root_node: String,
    pub nodes: HashMap<String, DialogNode>,
    pub variables: HashMap<String, i32>,
    pub flags: HashMap<String, bool>,
}

// 对话组件
#[derive(Component)]
pub struct DialogUi {
    pub state: DialogState,
    pub current_tree: Option<DialogTree>,
    pub current_node: Option<String>,
    pub displayed_text: String,
    pub full_text: String,
    pub text_progress: f32,
    pub typing_speed: f32,
    pub selected_choice: usize,
    pub animation_timer: Timer,
    pub auto_advance_timer: Option<Timer>,
    pub dialog_type: DialogType,
}

// 对话显示配置
#[derive(Debug, Clone)]
pub struct DialogDisplayConfig {
    pub text_speed: f32,
    pub auto_advance_delay: f32,
    pub enable_voice: bool,
    pub enable_animations: bool,
    pub show_portraits: bool,
    pub text_size: f32,
    pub line_spacing: f32,
}

impl Default for DialogUi {
    fn default() -> Self {
        Self {
            state: DialogState::Hidden,
            current_tree: None,
            current_node: None,
            displayed_text: String::new(),
            full_text: String::new(),
            text_progress: 0.0,
            typing_speed: 50.0, // 字符/秒
            selected_choice: 0,
            animation_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            auto_advance_timer: None,
            dialog_type: DialogType::NpcDialog,
        }
    }
}

impl Default for DialogDisplayConfig {
    fn default() -> Self {
        Self {
            text_speed: 50.0,
            auto_advance_delay: 2.0,
            enable_voice: true,
            enable_animations: true,
            show_portraits: true,
            text_size: 18.0,
            line_spacing: 1.2,
        }
    }
}

// 对话系统
pub struct DialogSystem {
    theme: UiTheme,
    config: DialogDisplayConfig,
    dialog_trees: HashMap<String, DialogTree>,
    ui_entities: Vec<Entity>,
    dialog_box: Option<Entity>,
    portrait_entity: Option<Entity>,
    choices_container: Option<Entity>,
}

impl DialogSystem {
    pub fn new(theme: UiTheme) -> Self {
        Self {
            theme,
            config: DialogDisplayConfig::default(),
            dialog_trees: HashMap::new(),
            ui_entities: Vec::new(),
            dialog_box: None,
            portrait_entity: None,
            choices_container: None,
        }
    }

    // 开始对话
    pub fn start_dialog(&mut self, commands: &mut Commands, tree_id: &str, dialog_type: DialogType) -> GameResult<()> {
        if let Some(tree) = self.dialog_trees.get(tree_id).cloned() {
            self.create_dialog_ui(commands, dialog_type)?;
            
            let mut dialog_ui = DialogUi {
                current_tree: Some(tree.clone()),
                current_node: Some(tree.root_node.clone()),
                dialog_type,
                ..default()
            };

            // 加载第一个节点
            if let Some(node) = tree.nodes.get(&tree.root_node) {
                dialog_ui.full_text = node.text.clone();
                dialog_ui.state = DialogState::Showing;
                
                if node.delay > 0.0 {
                    dialog_ui.auto_advance_timer = Some(Timer::from_seconds(node.delay, TimerMode::Once));
                }
            }

            // 将DialogUi组件添加到实体
            if let Some(dialog_entity) = self.dialog_box {
                commands.entity(dialog_entity).insert(dialog_ui);
            }
        }
        Ok(())
    }

    // 结束对话
    pub fn end_dialog(&mut self, commands: &mut Commands) -> GameResult<()> {
        self.cleanup_ui(commands);
        Ok(())
    }

    // 创建对话UI
    fn create_dialog_ui(&mut self, commands: &mut Commands, dialog_type: DialogType) -> GameResult<()> {
        // 创建背景遮罩
        let overlay = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
            ..default()
        }).id();

        // 创建对话框
        let dialog_box = self.create_dialog_box(commands, dialog_type)?;
        commands.entity(overlay).add_child(dialog_box);
        
        self.dialog_box = Some(dialog_box);
        self.ui_entities.push(overlay);
        Ok(())
    }

    // 创建对话框
    fn create_dialog_box(&mut self, commands: &mut Commands, dialog_type: DialogType) -> GameResult<Entity> {
        let box_height = match dialog_type {
            DialogType::SystemMessage => 100.0,
            DialogType::BattleDialog => 120.0,
            _ => 200.0,
        };

        let dialog_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(90.0),
                height: Val::Px(box_height),
                position_type: PositionType::Absolute,
                left: Val::Percent(5.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            background_color: self.theme.panel_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        // 创建角色立绘区域
        if self.config.show_portraits && dialog_type != DialogType::SystemMessage {
            let portrait_area = self.create_portrait_area(commands)?;
            commands.entity(dialog_container).add_child(portrait_area);
        }

        // 创建文本区域
        let text_area = self.create_text_area(commands, dialog_type)?;
        commands.entity(dialog_container).add_child(text_area);

        Ok(dialog_container)
    }

    // 创建立绘区域
    fn create_portrait_area(&mut self, commands: &mut Commands) -> GameResult<Entity> {
        let portrait_container = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Percent(100.0),
                border: UiRect::right(Val::Px(2.0)),
                ..default()
            },
            background_color: self.theme.secondary_color.into(),
            border_color: self.theme.border_color.into(),
            ..default()
        }).id();

        let portrait = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }).id();

        commands.entity(portrait_container).add_child(portrait);
        self.portrait_entity = Some(portrait);
        Ok(portrait_container)
    }

    // 创建文本区域
    fn create_text_area(&mut self, commands: &mut Commands, dialog_type: DialogType) -> GameResult<Entity> {
        let text_container = commands.spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            ..default()
        }).id();

        // 说话者名称区域
        if dialog_type != DialogType::SystemMessage {
            let speaker_area = commands.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(25.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                ..default()
            }).id();

            commands.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    color: self.theme.accent_color,
                    ..default()
                }
            )).set_parent(speaker_area);

            commands.entity(text_container).add_child(speaker_area);
        }

        // 主要文本区域
        let main_text_area = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                overflow: Overflow::clip(),
                ..default()
            },
            ..default()
        }).id();

        commands.spawn(TextBundle::from_section(
            "",
            TextStyle {
                font_size: self.config.text_size,
                color: self.theme.text_color,
                ..default()
            }
        )).set_parent(main_text_area);

        commands.entity(text_container).add_child(main_text_area);

        // 选择区域
        let choices_area = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(0.0), // 初始隐藏
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
            ..default()
        }).id();

        commands.entity(text_container).add_child(choices_area);
        self.choices_container = Some(choices_area);

        // 继续提示
        let continue_hint = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).id();

        commands.spawn(TextBundle::from_section(
            "按任意键继续...",
            TextStyle {
                font_size: 14.0,
                color: self.theme.secondary_text_color,
                ..default()
            }
        )).set_parent(continue_hint);

        commands.entity(text_container).add_child(continue_hint);

        Ok(text_container)
    }

    // 更新文本显示
    pub fn update_text_display(&self, dialog_ui: &mut DialogUi, delta_time: f32) {
        if dialog_ui.state == DialogState::Showing {
            let chars_to_show = (dialog_ui.text_progress * self.config.text_speed * delta_time) as usize;
            let max_chars = dialog_ui.full_text.chars().count();
            
            if chars_to_show >= max_chars {
                dialog_ui.displayed_text = dialog_ui.full_text.clone();
                dialog_ui.state = DialogState::WaitingInput;
            } else {
                dialog_ui.displayed_text = dialog_ui.full_text.chars().take(chars_to_show).collect();
                dialog_ui.text_progress += delta_time;
            }
        }
    }

    // 处理用户输入
    pub fn handle_input(&self, dialog_ui: &mut DialogUi, input: &Input<KeyCode>) -> Option<DialogEvent> {
        match dialog_ui.state {
            DialogState::Showing => {
                // 加速或跳过文本显示
                if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                    dialog_ui.displayed_text = dialog_ui.full_text.clone();
                    dialog_ui.state = DialogState::WaitingInput;
                    return Some(DialogEvent::TextCompleted);
                }
            },
            DialogState::WaitingInput => {
                if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                    return Some(DialogEvent::Continue);
                }
                if input.just_pressed(KeyCode::Escape) {
                    return Some(DialogEvent::Skip);
                }
            },
            DialogState::Choosing => {
                if input.just_pressed(KeyCode::Up) {
                    dialog_ui.selected_choice = dialog_ui.selected_choice.saturating_sub(1);
                    return Some(DialogEvent::ChoiceChanged(dialog_ui.selected_choice));
                }
                if input.just_pressed(KeyCode::Down) {
                    // 需要获取选择数量来限制范围
                    dialog_ui.selected_choice += 1;
                    return Some(DialogEvent::ChoiceChanged(dialog_ui.selected_choice));
                }
                if input.just_pressed(KeyCode::Return) {
                    return Some(DialogEvent::ChoiceSelected(dialog_ui.selected_choice));
                }
            },
            _ => {}
        }
        None
    }

    // 进入下一个节点
    pub fn next_node(&mut self, dialog_ui: &mut DialogUi) -> GameResult<bool> {
        if let (Some(tree), Some(current_id)) = (&dialog_ui.current_tree, &dialog_ui.current_node) {
            if let Some(current_node) = tree.nodes.get(current_id) {
                // 检查是否有选择
                if !current_node.choices.is_empty() {
                    self.show_choices(dialog_ui, &current_node.choices)?;
                    dialog_ui.state = DialogState::Choosing;
                    return Ok(true);
                }

                // 移动到下一个节点
                if let Some(next_id) = &current_node.next_node {
                    if let Some(next_node) = tree.nodes.get(next_id) {
                        dialog_ui.current_node = Some(next_id.clone());
                        dialog_ui.full_text = next_node.text.clone();
                        dialog_ui.displayed_text.clear();
                        dialog_ui.text_progress = 0.0;
                        dialog_ui.state = DialogState::Showing;

                        // 执行节点效果
                        self.execute_effects(&next_node.effects)?;
                        return Ok(true);
                    }
                }
            }
        }
        
        // 没有下一个节点，对话结束
        dialog_ui.state = DialogState::Closing;
        Ok(false)
    }

    // 显示选择选项
    fn show_choices(&self, dialog_ui: &mut DialogUi, choices: &[DialogChoice]) -> GameResult<()> {
        // 这里需要更新UI来显示选择选项
        // 实际实现需要操作Bevy实体
        dialog_ui.selected_choice = 0;
        Ok(())
    }

    // 选择选项
    pub fn select_choice(&mut self, dialog_ui: &mut DialogUi, choice_index: usize) -> GameResult<bool> {
        if let (Some(tree), Some(current_id)) = (&dialog_ui.current_tree, &dialog_ui.current_node) {
            if let Some(current_node) = tree.nodes.get(current_id) {
                if let Some(choice) = current_node.choices.get(choice_index) {
                    // 执行选择效果
                    self.execute_effects(&choice.effects)?;

                    // 移动到选择的下一个节点
                    if let Some(next_id) = &choice.next_node {
                        if let Some(next_node) = tree.nodes.get(next_id) {
                            dialog_ui.current_node = Some(next_id.clone());
                            dialog_ui.full_text = next_node.text.clone();
                            dialog_ui.displayed_text.clear();
                            dialog_ui.text_progress = 0.0;
                            dialog_ui.state = DialogState::Showing;
                            return Ok(true);
                        }
                    } else {
                        // 选择没有下一个节点，对话结束
                        dialog_ui.state = DialogState::Closing;
                        return Ok(false);
                    }
                }
            }
        }
        Ok(false)
    }

    // 检查条件
    pub fn check_conditions(&self, conditions: &[DialogCondition], tree: &DialogTree) -> bool {
        for condition in conditions {
            match condition {
                DialogCondition::VariableEquals(var_name, value) => {
                    if tree.variables.get(var_name) != Some(value) {
                        return false;
                    }
                },
                DialogCondition::VariableGreater(var_name, value) => {
                    if tree.variables.get(var_name).unwrap_or(&0) <= value {
                        return false;
                    }
                },
                DialogCondition::FlagSet(flag_name) => {
                    if !tree.flags.get(flag_name).unwrap_or(&false) {
                        return false;
                    }
                },
                _ => {
                    // 其他条件需要访问游戏状态
                    // 这里简化处理
                }
            }
        }
        true
    }

    // 执行效果
    fn execute_effects(&self, effects: &[DialogEffect]) -> GameResult<()> {
        for effect in effects {
            match effect {
                DialogEffect::PlaySound(sound_name) => {
                    info!("Playing sound: {}", sound_name);
                    // 实际实现需要音频系统
                },
                DialogEffect::SetVariable(var_name, value) => {
                    info!("Setting variable {} to {}", var_name, value);
                    // 需要更新游戏状态
                },
                DialogEffect::SetFlag(flag_name, value) => {
                    info!("Setting flag {} to {}", flag_name, value);
                    // 需要更新游戏状态
                },
                _ => {
                    // 其他效果需要与游戏系统交互
                    info!("Executing effect: {:?}", effect);
                }
            }
        }
        Ok(())
    }

    // 加载对话树
    pub fn load_dialog_tree(&mut self, tree: DialogTree) {
        self.dialog_trees.insert(tree.id.clone(), tree);
    }

    // 清理UI
    fn cleanup_ui(&mut self, commands: &mut Commands) {
        for entity in self.ui_entities.drain(..) {
            commands.entity(entity).despawn_recursive();
        }
        self.dialog_box = None;
        self.portrait_entity = None;
        self.choices_container = None;
    }

    // 设置配置
    pub fn set_config(&mut self, config: DialogDisplayConfig) {
        self.config = config;
    }

    // 跳过对话
    pub fn skip_dialog(&mut self, dialog_ui: &mut DialogUi) {
        dialog_ui.state = DialogState::Closing;
    }

    // 暂停对话
    pub fn pause_dialog(&mut self, dialog_ui: &mut DialogUi) {
        if dialog_ui.state == DialogState::Showing {
            dialog_ui.state = DialogState::WaitingInput;
        }
    }

    // 恢复对话
    pub fn resume_dialog(&mut self, dialog_ui: &mut DialogUi) {
        if dialog_ui.state == DialogState::WaitingInput && dialog_ui.displayed_text.len() < dialog_ui.full_text.len() {
            dialog_ui.state = DialogState::Showing;
        }
    }
}

// 对话事件
#[derive(Debug, Clone)]
pub enum DialogEvent {
    Started(String),                // 对话开始
    TextCompleted,                  // 文本显示完成
    Continue,                       // 继续
    ChoiceChanged(usize),          // 选择改变
    ChoiceSelected(usize),         // 选择确认
    Skip,                          // 跳过
    Ended,                         // 对话结束
}

// Bevy系统实现
pub fn update_dialog_system(
    mut dialog_system: ResMut<DialogSystem>,
    mut query: Query<&mut DialogUi>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut event_writer: EventWriter<DialogEvent>,
    mut commands: Commands,
) {
    for mut dialog_ui in query.iter_mut() {
        // 更新文本显示
        dialog_system.update_text_display(&mut dialog_ui, time.delta_seconds());

        // 更新自动前进计时器
        if let Some(timer) = &mut dialog_ui.auto_advance_timer {
            timer.tick(time.delta());
            if timer.just_finished() && dialog_ui.state == DialogState::WaitingInput {
                if let Some(event) = dialog_system.handle_input(&mut dialog_ui, &input) {
                    event_writer.send(event);
                }
            }
        }

        // 处理输入
        if let Some(event) = dialog_system.handle_input(&mut dialog_ui, &input) {
            event_writer.send(event);
        }

        // 处理状态转换
        if dialog_ui.state == DialogState::Closing {
            let _ = dialog_system.end_dialog(&mut commands);
            event_writer.send(DialogEvent::Ended);
        }
    }
}

pub fn handle_dialog_events_system(
    mut event_reader: EventReader<DialogEvent>,
    mut dialog_system: ResMut<DialogSystem>,
    mut query: Query<&mut DialogUi>,
) {
    for event in event_reader.iter() {
        match event {
            DialogEvent::Continue => {
                for mut dialog_ui in query.iter_mut() {
                    let _ = dialog_system.next_node(&mut dialog_ui);
                }
            },
            DialogEvent::ChoiceSelected(choice_index) => {
                for mut dialog_ui in query.iter_mut() {
                    let _ = dialog_system.select_choice(&mut dialog_ui, *choice_index);
                }
            },
            DialogEvent::Skip => {
                for mut dialog_ui in query.iter_mut() {
                    dialog_system.skip_dialog(&mut dialog_ui);
                }
            },
            _ => {}
        }
    }
}

// 辅助函数：创建简单对话
pub fn create_simple_dialog(speaker: Option<String>, text: String) -> DialogTree {
    let node_id = "root".to_string();
    let mut nodes = HashMap::new();
    
    nodes.insert(node_id.clone(), DialogNode {
        id: node_id.clone(),
        speaker,
        text,
        character_portrait: None,
        voice_clip: None,
        animation: None,
        choices: Vec::new(),
        next_node: None,
        conditions: Vec::new(),
        effects: Vec::new(),
        delay: 0.0,
    });

    DialogTree {
        id: "simple_dialog".to_string(),
        name: "Simple Dialog".to_string(),
        root_node: node_id,
        nodes,
        variables: HashMap::new(),
        flags: HashMap::new(),
    }
}

// 辅助函数：创建选择对话
pub fn create_choice_dialog(
    speaker: Option<String>, 
    text: String, 
    choices: Vec<(String, String)>
) -> DialogTree {
    let node_id = "root".to_string();
    let mut nodes = HashMap::new();
    
    let dialog_choices: Vec<DialogChoice> = choices.into_iter().enumerate().map(|(i, (choice_text, _))| {
        DialogChoice {
            id: format!("choice_{}", i),
            text: choice_text,
            next_node: None,
            conditions: Vec::new(),
            effects: Vec::new(),
            is_disabled: false,
        }
    }).collect();

    nodes.insert(node_id.clone(), DialogNode {
        id: node_id.clone(),
        speaker,
        text,
        character_portrait: None,
        voice_clip: None,
        animation: None,
        choices: dialog_choices,
        next_node: None,
        conditions: Vec::new(),
        effects: Vec::new(),
        delay: 0.0,
    });

    DialogTree {
        id: "choice_dialog".to_string(),
        name: "Choice Dialog".to_string(),
        root_node: node_id,
        nodes,
        variables: HashMap::new(),
        flags: HashMap::new(),
    }
}