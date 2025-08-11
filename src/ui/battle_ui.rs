// Battle UI System - 战斗界面系统
//
// 开发心理过程：
// 1. 战斗UI是游戏的核心界面之一，需要实时显示Pokemon状态、技能菜单等
// 2. 设计动态HP条、经验条动画效果，提供直观的视觉反馈
// 3. 实现技能选择界面，支持技能信息展示和PP显示
// 4. 集成战斗动画系统，确保UI与战斗逻辑同步
// 5. 考虑不同设备的显示适配和交互体验优化

use std::collections::{HashMap, VecDeque};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::battle::{BattleState, BattlePhase, Turn, DamageResult, StatusCondition};
use crate::pokemon::{Individual, Type, Stats, MoveData};
use crate::graphics::{UIRenderer, Texture, Animation, Sprite};
use crate::audio::{SoundManager, SoundEffect};
use crate::input::{InputState, InputAction};
use crate::ui::menu::{Menu, MenuManager, MenuItem, MenuAction};
use crate::player::{PartySlot};

/// 战斗UI管理器
#[derive(Resource)]
pub struct BattleUIManager {
    pub current_state: BattleUIState,
    pub ui_elements: HashMap<String, BattleUIElement>,
    pub animations: Vec<UIAnimation>,
    pub message_queue: VecDeque<BattleMessage>,
    pub active_menus: Vec<BattleMenu>,
    pub health_bars: HashMap<PartySlot, HealthBar>,
    pub experience_bars: HashMap<PartySlot, ExperienceBar>,
    pub turn_display: TurnDisplay,
    pub damage_numbers: Vec<DamageNumber>,
    pub status_indicators: HashMap<PartySlot, Vec<StatusIndicator>>,
    pub ui_settings: BattleUISettings,
}

impl Default for BattleUIManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BattleUIManager {
    pub fn new() -> Self {
        Self {
            current_state: BattleUIState::Hidden,
            ui_elements: HashMap::new(),
            animations: Vec::new(),
            message_queue: VecDeque::new(),
            active_menus: Vec::new(),
            health_bars: HashMap::new(),
            experience_bars: HashMap::new(),
            turn_display: TurnDisplay::new(),
            damage_numbers: Vec::new(),
            status_indicators: HashMap::new(),
            ui_settings: BattleUISettings::default(),
        }
    }

    pub fn initialize_battle(&mut self, player_pokemon: &[Individual], enemy_pokemon: &[Individual]) {
        self.current_state = BattleUIState::Entering;
        
        // 初始化玩家Pokemon的UI元素
        for (slot, pokemon) in player_pokemon.iter().enumerate() {
            self.create_health_bar(slot, pokemon, true);
            self.create_experience_bar(slot, pokemon);
            self.create_status_indicators(slot);
        }

        // 初始化敌方Pokemon的UI元素
        for (slot, pokemon) in enemy_pokemon.iter().enumerate() {
            self.create_health_bar(slot + 10, pokemon, false); // 敌方使用不同的slot范围
        }

        // 创建主要UI元素
        self.create_battle_interface();
        
        // 播放进入动画
        self.start_battle_entrance_animation();
    }

    fn create_health_bar(&mut self, slot: PartySlot, pokemon: &Individual, is_player: bool) {
        let max_hp = pokemon.calculate_stat(crate::pokemon::StatType::HP);
        let current_hp = max_hp; // 战斗开始时满血
        
        let health_bar = HealthBar {
            pokemon_name: pokemon.species.name.clone(),
            level: pokemon.level,
            current_hp,
            max_hp,
            is_player,
            position: if is_player {
                Vec2::new(50.0, 50.0 + slot as f32 * 100.0)
            } else {
                Vec2::new(1200.0, 500.0 + (slot - 10) as f32 * 100.0)
            },
            animation_target_hp: current_hp,
            animation_speed: 0.0,
            is_animating: false,
            bar_color: HealthBarColor::Green,
        };
        
        self.health_bars.insert(slot, health_bar);
    }

    fn create_experience_bar(&mut self, slot: PartySlot, pokemon: &Individual) {
        let exp_bar = ExperienceBar {
            current_exp: pokemon.experience,
            level_up_exp: pokemon.get_experience_for_level(pokemon.level + 1),
            previous_level_exp: pokemon.get_experience_for_level(pokemon.level),
            position: Vec2::new(50.0, 30.0 + slot as f32 * 100.0),
            animation_target_exp: pokemon.experience,
            animation_speed: 0.0,
            is_animating: false,
        };
        
        self.experience_bars.insert(slot, exp_bar);
    }

    fn create_status_indicators(&mut self, slot: PartySlot) {
        self.status_indicators.insert(slot, Vec::new());
    }

    fn create_battle_interface(&mut self) {
        // 创建技能菜单UI元素
        self.ui_elements.insert("move_menu_background".to_string(), BattleUIElement {
            element_type: UIElementType::Panel,
            position: Vec2::new(100.0, 850.0),
            size: Vec2::new(800.0, 200.0),
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.8),
            texture: None,
            is_visible: false,
            animation: None,
        });

        // 创建消息框UI元素
        self.ui_elements.insert("message_box".to_string(), BattleUIElement {
            element_type: UIElementType::TextBox,
            position: Vec2::new(50.0, 900.0),
            size: Vec2::new(1820.0, 150.0),
            color: Color::WHITE,
            texture: Some("message_box_bg.png".to_string()),
            is_visible: true,
            animation: None,
        });

        // 创建Pokemon信息面板
        self.ui_elements.insert("pokemon_info_panel".to_string(), BattleUIElement {
            element_type: UIElementType::Panel,
            position: Vec2::new(1400.0, 50.0),
            size: Vec2::new(450.0, 300.0),
            color: Color::from_rgba(0.2, 0.2, 0.2, 0.9),
            texture: Some("pokemon_info_bg.png".to_string()),
            is_visible: false,
            animation: None,
        });
    }

    fn start_battle_entrance_animation(&mut self) {
        // 创建入场动画
        for health_bar in self.health_bars.values_mut() {
            let slide_animation = UIAnimation {
                animation_type: AnimationType::SlideIn,
                target_element: if health_bar.is_player { "player_health_bar" } else { "enemy_health_bar" }.to_string(),
                duration: 1.0,
                elapsed: 0.0,
                from_position: Vec2::new(-200.0, health_bar.position.y),
                to_position: health_bar.position,
                easing: EasingType::EaseOut,
            };
            
            self.animations.push(slide_animation);
        }
    }

    pub fn update_battle_state(&mut self, battle_state: &BattleState) {
        match battle_state.phase {
            BattlePhase::SelectingMoves => {
                self.current_state = BattleUIState::SelectingMove;
                self.show_move_selection_menu(&battle_state.player_pokemon[0]);
            }
            BattlePhase::ExecutingTurns => {
                self.current_state = BattleUIState::ExecutingTurn;
                self.hide_move_selection_menu();
            }
            BattlePhase::CheckingResults => {
                self.current_state = BattleUIState::ShowingResults;
            }
            BattlePhase::BattleEnded => {
                self.current_state = BattleUIState::BattleEnded;
                self.show_battle_results(battle_state);
            }
            _ => {}
        }
    }

    pub fn show_move_selection_menu(&mut self, pokemon: &Individual) {
        // 显示技能选择菜单
        self.ui_elements.get_mut("move_menu_background").unwrap().is_visible = true;
        
        // 创建技能菜单
        let mut move_menu = BattleMenu {
            menu_type: BattleMenuType::MoveSelection,
            items: Vec::new(),
            selected_index: 0,
            is_visible: true,
            position: Vec2::new(120.0, 870.0),
        };

        for (i, move_data) in pokemon.moves.iter().enumerate() {
            let move_item = BattleMenuItem {
                id: i as u32,
                text: move_data.move_info.name.clone(),
                move_type: Some(move_data.move_info.move_type),
                pp_current: move_data.current_pp,
                pp_max: move_data.move_info.pp,
                is_enabled: move_data.current_pp > 0,
                position: Vec2::new(130.0 + (i % 2) as f32 * 350.0, 880.0 + (i / 2) as f32 * 50.0),
                size: Vec2::new(320.0, 40.0),
            };
            move_menu.items.push(move_item);
        }

        self.active_menus.push(move_menu);
    }

    pub fn hide_move_selection_menu(&mut self) {
        self.ui_elements.get_mut("move_menu_background").unwrap().is_visible = false;
        self.active_menus.retain(|menu| menu.menu_type != BattleMenuType::MoveSelection);
    }

    pub fn handle_input(&mut self, input_state: &InputState) -> Vec<BattleUIAction> {
        let mut actions = Vec::new();

        if let Some(active_menu) = self.active_menus.last_mut() {
            match active_menu.menu_type {
                BattleMenuType::MoveSelection => {
                    actions.extend(self.handle_move_selection_input(active_menu, input_state));
                }
                BattleMenuType::TargetSelection => {
                    actions.extend(self.handle_target_selection_input(active_menu, input_state));
                }
                BattleMenuType::ItemSelection => {
                    actions.extend(self.handle_item_selection_input(active_menu, input_state));
                }
            }
        }

        // 处理通用输入
        if input_state.just_pressed(InputAction::BattleInfo) {
            self.toggle_pokemon_info_panel();
        }

        if input_state.just_pressed(InputAction::BattleSkip) && !self.message_queue.is_empty() {
            self.skip_current_message();
        }

        actions
    }

    fn handle_move_selection_input(&mut self, menu: &mut BattleMenu, input_state: &InputState) -> Vec<BattleUIAction> {
        let mut actions = Vec::new();

        if input_state.just_pressed(InputAction::MenuUp) && menu.selected_index >= 2 {
            menu.selected_index -= 2;
        } else if input_state.just_pressed(InputAction::MenuDown) && menu.selected_index < menu.items.len() - 2 {
            menu.selected_index += 2;
        } else if input_state.just_pressed(InputAction::MenuLeft) && menu.selected_index % 2 == 1 {
            menu.selected_index -= 1;
        } else if input_state.just_pressed(InputAction::MenuRight) && menu.selected_index % 2 == 0 && menu.selected_index + 1 < menu.items.len() {
            menu.selected_index += 1;
        }

        if input_state.just_pressed(InputAction::MenuConfirm) {
            if menu.selected_index < menu.items.len() && menu.items[menu.selected_index].is_enabled {
                actions.push(BattleUIAction::MoveSelected(menu.selected_index));
            }
        }

        if input_state.just_pressed(InputAction::MenuCancel) {
            actions.push(BattleUIAction::BackToPreviousMenu);
        }

        actions
    }

    fn handle_target_selection_input(&mut self, menu: &mut BattleMenu, input_state: &InputState) -> Vec<BattleUIAction> {
        let mut actions = Vec::new();

        if input_state.just_pressed(InputAction::MenuUp) && menu.selected_index > 0 {
            menu.selected_index -= 1;
        } else if input_state.just_pressed(InputAction::MenuDown) && menu.selected_index < menu.items.len() - 1 {
            menu.selected_index += 1;
        }

        if input_state.just_pressed(InputAction::MenuConfirm) {
            if menu.selected_index < menu.items.len() {
                actions.push(BattleUIAction::TargetSelected(menu.selected_index));
            }
        }

        if input_state.just_pressed(InputAction::MenuCancel) {
            actions.push(BattleUIAction::BackToPreviousMenu);
        }

        actions
    }

    fn handle_item_selection_input(&mut self, menu: &mut BattleMenu, input_state: &InputState) -> Vec<BattleUIAction> {
        // TODO: 实现道具选择输入处理
        Vec::new()
    }

    pub fn execute_turn_animation(&mut self, turn: &Turn, damage_results: &[DamageResult]) {
        // 播放技能使用动画
        self.add_message(format!("{} used {}!", turn.pokemon_name, turn.move_name));
        
        // 创建技能动画
        let move_animation = UIAnimation {
            animation_type: AnimationType::MoveEffect,
            target_element: "battle_field".to_string(),
            duration: 1.5,
            elapsed: 0.0,
            from_position: Vec2::ZERO,
            to_position: Vec2::ZERO,
            easing: EasingType::Linear,
        };
        self.animations.push(move_animation);

        // 处理伤害结果
        for damage_result in damage_results {
            self.apply_damage_to_ui(damage_result);
        }
    }

    fn apply_damage_to_ui(&mut self, damage_result: &DamageResult) {
        let target_slot = damage_result.target_slot;
        
        // 更新血条
        if let Some(health_bar) = self.health_bars.get_mut(&target_slot) {
            health_bar.animation_target_hp = health_bar.current_hp.saturating_sub(damage_result.damage);
            health_bar.is_animating = true;
            health_bar.animation_speed = 2.0; // HP/秒的动画速度
            
            // 更新血条颜色
            let hp_ratio = health_bar.animation_target_hp as f32 / health_bar.max_hp as f32;
            health_bar.bar_color = if hp_ratio > 0.5 {
                HealthBarColor::Green
            } else if hp_ratio > 0.2 {
                HealthBarColor::Yellow
            } else {
                HealthBarColor::Red
            };
        }

        // 创建伤害数字动画
        let damage_number = DamageNumber {
            value: damage_result.damage,
            position: self.get_pokemon_position(target_slot),
            velocity: Vec2::new(0.0, -50.0), // 向上飘动
            lifetime: 2.0,
            elapsed: 0.0,
            color: if damage_result.is_critical_hit {
                Color::YELLOW
            } else {
                Color::WHITE
            },
            scale: if damage_result.is_critical_hit { 1.5 } else { 1.0 },
        };
        self.damage_numbers.push(damage_number);

        // 添加战斗消息
        if damage_result.is_critical_hit {
            self.add_message("A critical hit!".to_string());
        }

        if damage_result.effectiveness > 1.0 {
            self.add_message("It's super effective!".to_string());
        } else if damage_result.effectiveness < 1.0 && damage_result.effectiveness > 0.0 {
            self.add_message("It's not very effective...".to_string());
        } else if damage_result.effectiveness == 0.0 {
            self.add_message("It had no effect!".to_string());
        }
    }

    pub fn apply_status_condition(&mut self, pokemon_slot: PartySlot, condition: StatusCondition) {
        let indicator = StatusIndicator {
            condition,
            position: self.get_status_indicator_position(pokemon_slot),
            animation_timer: 0.0,
            blink_state: false,
        };

        self.status_indicators.entry(pokemon_slot).or_insert_with(Vec::new).push(indicator);
        
        let condition_name = match condition {
            StatusCondition::Burned => "burned",
            StatusCondition::Frozen => "frozen", 
            StatusCondition::Paralyzed => "paralyzed",
            StatusCondition::Poisoned => "poisoned",
            StatusCondition::Asleep => "asleep",
            _ => "affected by a status condition",
        };
        
        self.add_message(format!("The Pokemon was {}!", condition_name));
    }

    pub fn show_battle_results(&mut self, battle_state: &BattleState) {
        // 显示战斗结果
        match battle_state.winner {
            Some(winner) => {
                if winner == 0 {
                    self.add_message("Victory! You won the battle!".to_string());
                } else {
                    self.add_message("Defeat... You lost the battle.".to_string());
                }
            }
            None => {
                self.add_message("The battle ended in a draw.".to_string());
            }
        }

        // 显示经验值获得
        for (slot, exp_gained) in &battle_state.experience_gained {
            if let Some(exp_bar) = self.experience_bars.get_mut(slot) {
                exp_bar.animation_target_exp += exp_gained;
                exp_bar.is_animating = true;
                exp_bar.animation_speed = 100.0; // 经验/秒
            }
        }
    }

    pub fn add_message(&mut self, message: String) {
        let battle_message = BattleMessage {
            text: message,
            display_time: 3.0,
            elapsed: 0.0,
            is_auto_advance: true,
        };
        self.message_queue.push_back(battle_message);
    }

    pub fn skip_current_message(&mut self) {
        if let Some(current_message) = self.message_queue.front_mut() {
            current_message.elapsed = current_message.display_time;
        }
    }

    fn toggle_pokemon_info_panel(&mut self) {
        if let Some(panel) = self.ui_elements.get_mut("pokemon_info_panel") {
            panel.is_visible = !panel.is_visible;
            
            // 创建淡入/淡出动画
            let fade_animation = UIAnimation {
                animation_type: if panel.is_visible { AnimationType::FadeIn } else { AnimationType::FadeOut },
                target_element: "pokemon_info_panel".to_string(),
                duration: 0.3,
                elapsed: 0.0,
                from_position: Vec2::ZERO,
                to_position: Vec2::ZERO,
                easing: EasingType::EaseInOut,
            };
            self.animations.push(fade_animation);
        }
    }

    fn get_pokemon_position(&self, slot: PartySlot) -> Vec2 {
        if slot < 10 {
            // 玩家Pokemon
            Vec2::new(400.0, 300.0 + slot as f32 * 100.0)
        } else {
            // 敌方Pokemon
            Vec2::new(1200.0, 300.0 + (slot - 10) as f32 * 100.0)
        }
    }

    fn get_status_indicator_position(&self, slot: PartySlot) -> Vec2 {
        let base_pos = self.get_pokemon_position(slot);
        Vec2::new(base_pos.x + 50.0, base_pos.y - 30.0)
    }

    pub fn update(&mut self, delta_time: f32) {
        // 更新动画
        self.update_animations(delta_time);
        
        // 更新血条动画
        self.update_health_bars(delta_time);
        
        // 更新经验条动画
        self.update_experience_bars(delta_time);
        
        // 更新伤害数字
        self.update_damage_numbers(delta_time);
        
        // 更新状态指示器
        self.update_status_indicators(delta_time);
        
        // 更新消息
        self.update_messages(delta_time);
    }

    fn update_animations(&mut self, delta_time: f32) {
        self.animations.retain_mut(|animation| {
            animation.elapsed += delta_time;
            animation.elapsed < animation.duration
        });
    }

    fn update_health_bars(&mut self, delta_time: f32) {
        for health_bar in self.health_bars.values_mut() {
            if health_bar.is_animating {
                let hp_diff = health_bar.animation_target_hp as f32 - health_bar.current_hp as f32;
                
                if hp_diff.abs() > 1.0 {
                    let change = hp_diff.signum() * health_bar.animation_speed * delta_time;
                    health_bar.current_hp = (health_bar.current_hp as f32 + change).round() as u32;
                } else {
                    health_bar.current_hp = health_bar.animation_target_hp;
                    health_bar.is_animating = false;
                }
            }
        }
    }

    fn update_experience_bars(&mut self, delta_time: f32) {
        for exp_bar in self.experience_bars.values_mut() {
            if exp_bar.is_animating {
                let exp_diff = exp_bar.animation_target_exp as f32 - exp_bar.current_exp as f32;
                
                if exp_diff.abs() > 10.0 {
                    let change = exp_diff.signum() * exp_bar.animation_speed * delta_time;
                    exp_bar.current_exp = (exp_bar.current_exp as f32 + change).round() as u64;
                } else {
                    exp_bar.current_exp = exp_bar.animation_target_exp;
                    exp_bar.is_animating = false;
                }
            }
        }
    }

    fn update_damage_numbers(&mut self, delta_time: f32) {
        for damage_number in &mut self.damage_numbers {
            damage_number.elapsed += delta_time;
            damage_number.position += damage_number.velocity * delta_time;
            damage_number.velocity.y -= 100.0 * delta_time; // 重力效果
        }
        
        self.damage_numbers.retain(|damage_number| damage_number.elapsed < damage_number.lifetime);
    }

    fn update_status_indicators(&mut self, delta_time: f32) {
        for indicators in self.status_indicators.values_mut() {
            for indicator in indicators {
                indicator.animation_timer += delta_time;
                indicator.blink_state = (indicator.animation_timer * 2.0).sin() > 0.0;
            }
        }
    }

    fn update_messages(&mut self, delta_time: f32) {
        if let Some(current_message) = self.message_queue.front_mut() {
            current_message.elapsed += delta_time;
            
            if current_message.is_auto_advance && current_message.elapsed >= current_message.display_time {
                self.message_queue.pop_front();
            }
        }
    }

    pub fn render(&self, ui_renderer: &mut UIRenderer) {
        match self.current_state {
            BattleUIState::Hidden => return,
            _ => {}
        }

        // 渲染UI元素
        self.render_ui_elements(ui_renderer);
        
        // 渲染血条
        self.render_health_bars(ui_renderer);
        
        // 渲染经验条
        self.render_experience_bars(ui_renderer);
        
        // 渲染菜单
        self.render_menus(ui_renderer);
        
        // 渲染伤害数字
        self.render_damage_numbers(ui_renderer);
        
        // 渲染状态指示器
        self.render_status_indicators(ui_renderer);
        
        // 渲染消息
        self.render_messages(ui_renderer);
        
        // 渲染动画效果
        self.render_animations(ui_renderer);
    }

    fn render_ui_elements(&self, ui_renderer: &mut UIRenderer) {
        for element in self.ui_elements.values() {
            if !element.is_visible {
                continue;
            }

            match element.element_type {
                UIElementType::Panel => {
                    ui_renderer.draw_rect(element.position, element.size, element.color);
                }
                UIElementType::TextBox => {
                    if let Some(ref texture) = element.texture {
                        ui_renderer.draw_texture(texture, element.position, element.size);
                    } else {
                        ui_renderer.draw_rect(element.position, element.size, element.color);
                    }
                }
                UIElementType::Image => {
                    if let Some(ref texture) = element.texture {
                        ui_renderer.draw_texture(texture, element.position, element.size);
                    }
                }
            }
        }
    }

    fn render_health_bars(&self, ui_renderer: &mut UIRenderer) {
        for health_bar in self.health_bars.values() {
            self.render_health_bar(health_bar, ui_renderer);
        }
    }

    fn render_health_bar(&self, health_bar: &HealthBar, ui_renderer: &mut UIRenderer) {
        let bar_width = 200.0;
        let bar_height = 20.0;
        let hp_ratio = health_bar.current_hp as f32 / health_bar.max_hp as f32;
        
        // 背景
        ui_renderer.draw_rect(health_bar.position, Vec2::new(bar_width, bar_height), Color::BLACK);
        
        // HP条
        let hp_width = bar_width * hp_ratio;
        let hp_color = match health_bar.bar_color {
            HealthBarColor::Green => Color::GREEN,
            HealthBarColor::Yellow => Color::YELLOW,
            HealthBarColor::Red => Color::RED,
        };
        ui_renderer.draw_rect(health_bar.position, Vec2::new(hp_width, bar_height), hp_color);
        
        // 边框
        ui_renderer.draw_rect_outline(health_bar.position, Vec2::new(bar_width, bar_height), Color::WHITE, 2.0);
        
        // Pokemon名称和等级
        let name_text = format!("{} Lv.{}", health_bar.pokemon_name, health_bar.level);
        ui_renderer.draw_text(&name_text, Vec2::new(health_bar.position.x, health_bar.position.y - 25.0), 
                             &crate::graphics::TextStyle {
                                 font: "default".to_string(),
                                 size: 18.0,
                                 color: crate::graphics::Color::WHITE,
                             });
        
        // HP数值
        let hp_text = format!("{}/{}", health_bar.current_hp, health_bar.max_hp);
        ui_renderer.draw_text(&hp_text, Vec2::new(health_bar.position.x + bar_width - 50.0, health_bar.position.y + 5.0),
                             &crate::graphics::TextStyle {
                                 font: "small".to_string(),
                                 size: 14.0,
                                 color: crate::graphics::Color::WHITE,
                             });
    }

    fn render_experience_bars(&self, ui_renderer: &mut UIRenderer) {
        for exp_bar in self.experience_bars.values() {
            self.render_experience_bar(exp_bar, ui_renderer);
        }
    }

    fn render_experience_bar(&self, exp_bar: &ExperienceBar, ui_renderer: &mut UIRenderer) {
        let bar_width = 200.0;
        let bar_height = 8.0;
        
        let level_exp = exp_bar.level_up_exp - exp_bar.previous_level_exp;
        let current_level_exp = exp_bar.current_exp - exp_bar.previous_level_exp;
        let exp_ratio = if level_exp > 0 {
            current_level_exp as f32 / level_exp as f32
        } else {
            1.0
        };
        
        // 背景
        ui_renderer.draw_rect(exp_bar.position, Vec2::new(bar_width, bar_height), Color::DARK_GRAY);
        
        // 经验条
        let exp_width = bar_width * exp_ratio.min(1.0);
        ui_renderer.draw_rect(exp_bar.position, Vec2::new(exp_width, bar_height), Color::BLUE);
        
        // 边框
        ui_renderer.draw_rect_outline(exp_bar.position, Vec2::new(bar_width, bar_height), Color::WHITE, 1.0);
        
        // EXP标签
        ui_renderer.draw_text("EXP", Vec2::new(exp_bar.position.x - 40.0, exp_bar.position.y),
                             &crate::graphics::TextStyle {
                                 font: "small".to_string(),
                                 size: 12.0,
                                 color: crate::graphics::Color::WHITE,
                             });
    }

    fn render_menus(&self, ui_renderer: &mut UIRenderer) {
        for menu in &self.active_menus {
            self.render_battle_menu(menu, ui_renderer);
        }
    }

    fn render_battle_menu(&self, menu: &BattleMenu, ui_renderer: &mut UIRenderer) {
        if !menu.is_visible {
            return;
        }

        for (index, item) in menu.items.iter().enumerate() {
            let is_selected = index == menu.selected_index;
            let background_color = if is_selected {
                Color::from_rgba(0.0, 0.5, 1.0, 0.8)
            } else if item.is_enabled {
                Color::from_rgba(0.2, 0.2, 0.2, 0.8)
            } else {
                Color::from_rgba(0.1, 0.1, 0.1, 0.5)
            };

            // 背景
            ui_renderer.draw_rect(item.position, item.size, background_color);
            
            // 边框
            if is_selected {
                ui_renderer.draw_rect_outline(item.position, item.size, Color::WHITE, 2.0);
            }

            // 技能名称
            let text_color = if item.is_enabled {
                Color::WHITE
            } else {
                Color::GRAY
            };
            
            ui_renderer.draw_text(&item.text, Vec2::new(item.position.x + 10.0, item.position.y + 5.0),
                                 &crate::graphics::TextStyle {
                                     font: "default".to_string(),
                                     size: 16.0,
                                     color: text_color.into(),
                                 });

            // 属性类型（如果有）
            if let Some(move_type) = item.move_type {
                let type_color = self.get_type_color(move_type);
                let type_pos = Vec2::new(item.position.x + item.size.x - 80.0, item.position.y + 5.0);
                ui_renderer.draw_rect(type_pos, Vec2::new(70.0, 15.0), type_color);
                
                ui_renderer.draw_text(&format!("{:?}", move_type), Vec2::new(type_pos.x + 5.0, type_pos.y + 2.0),
                                     &crate::graphics::TextStyle {
                                         font: "small".to_string(),
                                         size: 12.0,
                                         color: crate::graphics::Color::WHITE,
                                     });
            }

            // PP显示
            let pp_text = format!("{}/{}", item.pp_current, item.pp_max);
            ui_renderer.draw_text(&pp_text, Vec2::new(item.position.x + item.size.x - 50.0, item.position.y + item.size.y - 20.0),
                                 &crate::graphics::TextStyle {
                                     font: "small".to_string(),
                                     size: 12.0,
                                     color: if item.pp_current > 0 { 
                                         crate::graphics::Color::WHITE 
                                     } else { 
                                         crate::graphics::Color::RED 
                                     },
                                 });
        }
    }

    fn get_type_color(&self, pokemon_type: Type) -> Color {
        match pokemon_type {
            Type::Normal => Color::from_rgba(0.7, 0.7, 0.7, 1.0),
            Type::Fire => Color::from_rgba(1.0, 0.3, 0.3, 1.0),
            Type::Water => Color::from_rgba(0.3, 0.3, 1.0, 1.0),
            Type::Electric => Color::from_rgba(1.0, 1.0, 0.3, 1.0),
            Type::Grass => Color::from_rgba(0.3, 1.0, 0.3, 1.0),
            Type::Ice => Color::from_rgba(0.7, 0.9, 1.0, 1.0),
            Type::Fighting => Color::from_rgba(0.8, 0.3, 0.3, 1.0),
            Type::Poison => Color::from_rgba(0.8, 0.3, 0.8, 1.0),
            Type::Ground => Color::from_rgba(0.9, 0.7, 0.3, 1.0),
            Type::Flying => Color::from_rgba(0.7, 0.7, 1.0, 1.0),
            Type::Psychic => Color::from_rgba(1.0, 0.3, 0.7, 1.0),
            Type::Bug => Color::from_rgba(0.7, 0.8, 0.3, 1.0),
            Type::Rock => Color::from_rgba(0.7, 0.6, 0.3, 1.0),
            Type::Ghost => Color::from_rgba(0.5, 0.3, 0.7, 1.0),
            Type::Dragon => Color::from_rgba(0.5, 0.3, 1.0, 1.0),
            Type::Dark => Color::from_rgba(0.3, 0.3, 0.3, 1.0),
            Type::Steel => Color::from_rgba(0.7, 0.7, 0.8, 1.0),
            Type::Fairy => Color::from_rgba(1.0, 0.7, 0.9, 1.0),
        }
    }

    fn render_damage_numbers(&self, ui_renderer: &mut UIRenderer) {
        for damage_number in &self.damage_numbers {
            let alpha = 1.0 - (damage_number.elapsed / damage_number.lifetime);
            let mut color = damage_number.color;
            color.set_a(alpha);

            ui_renderer.draw_text(&damage_number.value.to_string(), damage_number.position,
                                 &crate::graphics::TextStyle {
                                     font: "damage".to_string(),
                                     size: 24.0 * damage_number.scale,
                                     color: color.into(),
                                 });
        }
    }

    fn render_status_indicators(&self, ui_renderer: &mut UIRenderer) {
        for indicators in self.status_indicators.values() {
            for indicator in indicators {
                if indicator.blink_state {
                    let icon_name = match indicator.condition {
                        StatusCondition::Burned => "status_burn.png",
                        StatusCondition::Frozen => "status_freeze.png",
                        StatusCondition::Paralyzed => "status_paralyze.png",
                        StatusCondition::Poisoned => "status_poison.png",
                        StatusCondition::Asleep => "status_sleep.png",
                        _ => "status_unknown.png",
                    };

                    ui_renderer.draw_texture(icon_name, indicator.position, Vec2::new(24.0, 24.0));
                }
            }
        }
    }

    fn render_messages(&self, ui_renderer: &mut UIRenderer) {
        if let Some(current_message) = self.message_queue.front() {
            ui_renderer.draw_text(&current_message.text, Vec2::new(100.0, 920.0),
                                 &crate::graphics::TextStyle {
                                     font: "default".to_string(),
                                     size: 20.0,
                                     color: crate::graphics::Color::BLACK,
                                 });
        }
    }

    fn render_animations(&self, ui_renderer: &mut UIRenderer) {
        for animation in &self.animations {
            match animation.animation_type {
                AnimationType::MoveEffect => {
                    // 渲染技能效果动画
                    let progress = animation.elapsed / animation.duration;
                    let alpha = (progress * std::f32::consts::PI * 2.0).sin().abs();
                    
                    // 闪烁效果
                    ui_renderer.draw_rect(Vec2::ZERO, Vec2::new(1920.0, 1080.0), 
                                         Color::from_rgba(1.0, 1.0, 1.0, alpha * 0.3));
                }
                _ => {}
            }
        }
    }
}

/// 战斗UI状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleUIState {
    Hidden,
    Entering,
    SelectingMove,
    ExecutingTurn,
    ShowingResults,
    BattleEnded,
    Transitioning,
}

/// UI元素类型
#[derive(Debug, Clone)]
pub struct BattleUIElement {
    pub element_type: UIElementType,
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub texture: Option<String>,
    pub is_visible: bool,
    pub animation: Option<UIAnimation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIElementType {
    Panel,
    TextBox,
    Image,
}

/// 血条组件
#[derive(Debug, Clone)]
pub struct HealthBar {
    pub pokemon_name: String,
    pub level: u8,
    pub current_hp: u32,
    pub max_hp: u32,
    pub is_player: bool,
    pub position: Vec2,
    pub animation_target_hp: u32,
    pub animation_speed: f32,
    pub is_animating: bool,
    pub bar_color: HealthBarColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthBarColor {
    Green,
    Yellow,
    Red,
}

/// 经验条组件
#[derive(Debug, Clone)]
pub struct ExperienceBar {
    pub current_exp: u64,
    pub level_up_exp: u64,
    pub previous_level_exp: u64,
    pub position: Vec2,
    pub animation_target_exp: u64,
    pub animation_speed: f32,
    pub is_animating: bool,
}

/// 战斗菜单
#[derive(Debug, Clone)]
pub struct BattleMenu {
    pub menu_type: BattleMenuType,
    pub items: Vec<BattleMenuItem>,
    pub selected_index: usize,
    pub is_visible: bool,
    pub position: Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleMenuType {
    MoveSelection,
    TargetSelection,
    ItemSelection,
}

/// 战斗菜单项
#[derive(Debug, Clone)]
pub struct BattleMenuItem {
    pub id: u32,
    pub text: String,
    pub move_type: Option<Type>,
    pub pp_current: u32,
    pub pp_max: u32,
    pub is_enabled: bool,
    pub position: Vec2,
    pub size: Vec2,
}

/// UI动画
#[derive(Debug, Clone)]
pub struct UIAnimation {
    pub animation_type: AnimationType,
    pub target_element: String,
    pub duration: f32,
    pub elapsed: f32,
    pub from_position: Vec2,
    pub to_position: Vec2,
    pub easing: EasingType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationType {
    SlideIn,
    SlideOut,
    FadeIn,
    FadeOut,
    Scale,
    MoveEffect,
    Shake,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// 伤害数字
#[derive(Debug, Clone)]
pub struct DamageNumber {
    pub value: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub elapsed: f32,
    pub color: Color,
    pub scale: f32,
}

/// 状态指示器
#[derive(Debug, Clone)]
pub struct StatusIndicator {
    pub condition: StatusCondition,
    pub position: Vec2,
    pub animation_timer: f32,
    pub blink_state: bool,
}

/// 战斗消息
#[derive(Debug, Clone)]
pub struct BattleMessage {
    pub text: String,
    pub display_time: f32,
    pub elapsed: f32,
    pub is_auto_advance: bool,
}

/// 回合显示
#[derive(Debug, Clone)]
pub struct TurnDisplay {
    pub current_turn: u32,
    pub phase_text: String,
    pub position: Vec2,
}

impl TurnDisplay {
    pub fn new() -> Self {
        Self {
            current_turn: 1,
            phase_text: "Battle Start".to_string(),
            position: Vec2::new(1700.0, 50.0),
        }
    }
}

/// 战斗UI动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleUIAction {
    MoveSelected(usize),
    TargetSelected(usize),
    ItemSelected(usize),
    BackToPreviousMenu,
    ShowPokemonInfo,
    SkipAnimation,
    SwitchPokemon(PartySlot),
}

/// 战斗UI设置
#[derive(Debug, Clone)]
pub struct BattleUISettings {
    pub animation_speed: f32,
    pub auto_advance_messages: bool,
    pub show_damage_numbers: bool,
    pub show_type_effectiveness: bool,
    pub battle_camera_shake: bool,
}

impl Default for BattleUISettings {
    fn default() -> Self {
        Self {
            animation_speed: 1.0,
            auto_advance_messages: true,
            show_damage_numbers: true,
            show_type_effectiveness: true,
            battle_camera_shake: true,
        }
    }
}

/// Bevy系统：处理战斗UI输入
pub fn battle_ui_input_system(
    input_state: Res<InputState>,
    mut battle_ui_manager: ResMut<BattleUIManager>,
    mut sound_manager: ResMut<SoundManager>,
) {
    let actions = battle_ui_manager.handle_input(&input_state);
    
    for action in actions {
        match action {
            BattleUIAction::MoveSelected(move_index) => {
                sound_manager.play_sound_effect(SoundEffect::MenuConfirm);
                // TODO: 通知战斗系统选择了技能
            }
            BattleUIAction::BackToPreviousMenu => {
                sound_manager.play_sound_effect(SoundEffect::MenuCancel);
                // TODO: 返回上一级菜单
            }
            _ => {}
        }
    }
}

/// Bevy系统：更新战斗UI
pub fn battle_ui_update_system(
    time: Res<Time>,
    mut battle_ui_manager: ResMut<BattleUIManager>,
) {
    battle_ui_manager.update(time.delta_seconds());
}

/// Bevy系统：渲染战斗UI
pub fn battle_ui_render_system(
    battle_ui_manager: Res<BattleUIManager>,
    mut ui_renderer: ResMut<UIRenderer>,
) {
    battle_ui_manager.render(&mut ui_renderer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::{PokemonSpecies, Nature};

    fn create_test_pokemon() -> Individual {
        let species = PokemonSpecies {
            id: crate::pokemon::PokemonId(25),
            name: "Pikachu".to_string(),
            types: vec![Type::Electric],
            base_stats: crate::pokemon::Stats {
                hp: 35,
                attack: 55,
                defense: 40,
                special_attack: 50,
                special_defense: 50,
                speed: 90,
            },
            abilities: vec![],
            moves_learned: vec![],
            evolution_chain: None,
        };

        Individual::new(species, 25, Nature::Hardy)
    }

    #[test]
    fn test_battle_ui_manager_creation() {
        let manager = BattleUIManager::new();
        assert_eq!(manager.current_state, BattleUIState::Hidden);
        assert!(manager.ui_elements.is_empty());
        assert!(manager.health_bars.is_empty());
    }

    #[test]
    fn test_battle_initialization() {
        let mut manager = BattleUIManager::new();
        let player_pokemon = vec![create_test_pokemon()];
        let enemy_pokemon = vec![create_test_pokemon()];
        
        manager.initialize_battle(&player_pokemon, &enemy_pokemon);
        
        assert_eq!(manager.current_state, BattleUIState::Entering);
        assert!(!manager.health_bars.is_empty());
        assert!(!manager.ui_elements.is_empty());
    }

    #[test]
    fn test_damage_application() {
        let mut manager = BattleUIManager::new();
        let pokemon = create_test_pokemon();
        manager.create_health_bar(0, &pokemon, true);
        
        let damage_result = DamageResult {
            damage: 20,
            target_slot: 0,
            is_critical_hit: false,
            effectiveness: 1.0,
            status_effects: vec![],
        };
        
        manager.apply_damage_to_ui(&damage_result);
        
        let health_bar = manager.health_bars.get(&0).unwrap();
        assert!(health_bar.is_animating);
        assert_eq!(health_bar.animation_target_hp, health_bar.current_hp - 20);
    }

    #[test]
    fn test_message_system() {
        let mut manager = BattleUIManager::new();
        
        manager.add_message("Test message".to_string());
        assert_eq!(manager.message_queue.len(), 1);
        
        manager.skip_current_message();
        let message = manager.message_queue.front().unwrap();
        assert_eq!(message.elapsed, message.display_time);
    }
}