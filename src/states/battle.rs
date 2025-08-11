// 战斗状态
// 开发心理：战斗是游戏核心体验，需要流畅动画、策略深度、视觉冲击
// 设计原则：回合制逻辑、动画表现、AI智能、用户交互

use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::ui::{UIManager, ElementType};
use super::Renderer2D;
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
// Pokemon相关类型暂时注释掉，等待pokemon模块启用
// use crate::pokemon::stats::PokemonStats;
// use crate::pokemon::types::{PokemonType, DualType};
// use crate::battle::animation::BattleAnimationManager;
// use crate::battle::status_effects::StatusEffectManager;

// 临时类型定义
#[derive(Debug, Clone, Copy)]
pub enum PokemonType { Normal, Fire, Water, Electric, Grass, Ice, Fighting, Poison, Ground, Flying, Psychic, Bug, Rock, Ghost, Dragon, Dark, Steel, Fairy }

#[derive(Debug, Clone)]
pub struct PokemonStats { pub hp: u32, pub attack: u32, pub defense: u32, pub sp_attack: u32, pub sp_defense: u32, pub speed: u32 }

#[derive(Debug, Clone)]
pub struct DualType(pub Option<PokemonType>, pub Option<PokemonType>);
use super::{StateHandler, GameStateType, StateTransition};
use glam::{Vec2, Vec4};
use std::collections::HashMap;

// 战斗动画管理器
#[derive(Debug)]
pub struct BattleAnimationManager {
    animations: HashMap<String, String>,
    current_animation: Option<String>,
    animation_timer: f32,
}

impl BattleAnimationManager {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            current_animation: None,
            animation_timer: 0.0,
        }
    }
    
    pub fn play_animation(&mut self, animation: &str, target: &str) -> Result<(), GameError> {
        self.current_animation = Some(format!("{}-{}", animation, target));
        self.animation_timer = 0.0;
        Ok(())
    }
    
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        if self.current_animation.is_some() {
            self.animation_timer += delta_time;
            if self.animation_timer > 2.0 { // 2秒动画时长
                self.current_animation = None;
                self.animation_timer = 0.0;
            }
        }
        Ok(())
    }
    
    pub fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 渲染当前动画效果
        if let Some(_) = &self.current_animation {
            // 简单的闪烁效果
            let alpha = (self.animation_timer * 10.0).sin().abs();
            renderer.draw_sprite(
                Vec2::new(400.0, 300.0),
                Vec2::new(50.0, 50.0),
                [1.0, 1.0, 0.0, alpha],
            );
        }
        Ok(())
    }
    
    pub fn is_playing(&self) -> bool {
        self.current_animation.is_some()
    }
}

// 状态效果管理器
#[derive(Debug)]
pub struct StatusEffectManager {
    effects: HashMap<u32, Vec<StatusEffect>>,
}

#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub effect_type: u32,
    pub duration: f32,
    pub intensity: f32,
}

impl StatusEffectManager {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        // 更新状态效果持续时间
        for (_, effects) in self.effects.iter_mut() {
            effects.retain_mut(|effect| {
                effect.duration -= delta_time;
                effect.duration > 0.0
            });
        }
        Ok(())
    }
    
    pub fn add_effect(&mut self, pokemon_id: u32, effect: StatusEffect) {
        self.effects.entry(pokemon_id).or_insert_with(Vec::new).push(effect);
    }
    
    pub fn remove_effect(&mut self, pokemon_id: u32, effect_type: u32) {
        if let Some(effects) = self.effects.get_mut(&pokemon_id) {
            effects.retain(|e| e.effect_type != effect_type);
        }
    }
}

// 战斗阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlePhase {
    Initializing,   // 初始化
    PlayerTurn,     // 玩家回合
    EnemyTurn,      // 敌方回合
    Animation,      // 动画播放中
    Victory,        // 胜利
    Defeat,         // 失败
    Escape,         // 逃跑
}

// 战斗行动类型
#[derive(Debug, Clone, PartialEq)]
pub enum BattleAction {
    Attack { move_id: u32, target: usize },
    Item { item_id: u32, target: Option<usize> },
    Switch { pokemon_index: usize },
    Escape,
}

// 战斗结果
#[derive(Debug, Clone)]
pub struct BattleResult {
    pub victory: bool,
    pub experience_gained: u32,
    pub items_obtained: Vec<u32>,
    pub pokemon_caught: Option<u32>,
}

// 战斗Pokemon数据
#[derive(Debug, Clone)]
pub struct BattlePokemon {
    pub species_id: u32,
    pub name: String,
    pub level: u8,
    pub stats: PokemonStats,
    pub types: DualType,
    pub current_hp: u32,
    pub status_effects: Vec<u32>,
    pub moves: Vec<u32>,
    pub sprite_id: Option<u32>,
    pub position: Vec2,
    pub is_player: bool,
}

// 战斗状态
pub struct BattleState {
    name: String,
    ui_manager: UIManager,
    animation_manager: BattleAnimationManager,
    status_manager: StatusEffectManager,
    
    // 战斗数据
    phase: BattlePhase,
    turn_count: u32,
    player_team: Vec<BattlePokemon>,
    enemy_team: Vec<BattlePokemon>,
    active_player: usize,
    active_enemy: usize,
    
    // UI元素
    player_hp_bar: Option<u32>,
    enemy_hp_bar: Option<u32>,
    battle_menu: Option<u32>,
    move_buttons: Vec<u32>,
    battle_log: Vec<String>,
    
    // 动作队列
    action_queue: Vec<BattleAction>,
    current_action: Option<BattleAction>,
    
    // 动画状态
    animation_playing: bool,
    animation_timer: f32,
    
    // 相机震动
    screen_shake: Vec2,
    shake_intensity: f32,
    shake_duration: f32,
    
    // 战斗配置
    can_escape: bool,
    can_catch: bool,
    background_id: u32,
    
    // 统计
    damage_dealt: u32,
    damage_received: u32,
    moves_used: u32,
}

impl BattleState {
    pub fn new() -> Self {
        Self {
            name: "BattleState".to_string(),
            ui_manager: UIManager::new(Vec2::new(800.0, 600.0)),
            animation_manager: BattleAnimationManager::new(),
            status_manager: StatusEffectManager::new(),
            phase: BattlePhase::Initializing,
            turn_count: 0,
            player_team: Vec::new(),
            enemy_team: Vec::new(),
            active_player: 0,
            active_enemy: 0,
            player_hp_bar: None,
            enemy_hp_bar: None,
            battle_menu: None,
            move_buttons: Vec::new(),
            battle_log: Vec::new(),
            action_queue: Vec::new(),
            current_action: None,
            animation_playing: false,
            animation_timer: 0.0,
            screen_shake: Vec2::ZERO,
            shake_intensity: 0.0,
            shake_duration: 0.0,
            can_escape: true,
            can_catch: false,
            background_id: 1,
            damage_dealt: 0,
            damage_received: 0,
            moves_used: 0,
        }
    }
    
    // 初始化战斗
    pub fn start_battle(
        &mut self,
        player_team: Vec<BattlePokemon>,
        enemy_team: Vec<BattlePokemon>,
        can_escape: bool,
        can_catch: bool,
    ) -> Result<(), GameError> {
        self.player_team = player_team;
        self.enemy_team = enemy_team;
        self.can_escape = can_escape;
        self.can_catch = can_catch;
        self.active_player = 0;
        self.active_enemy = 0;
        self.turn_count = 0;
        
        // 设置Pokemon位置
        if let Some(player_pokemon) = self.player_team.get_mut(0) {
            player_pokemon.position = Vec2::new(200.0, 400.0);
        }
        if let Some(enemy_pokemon) = self.enemy_team.get_mut(0) {
            enemy_pokemon.position = Vec2::new(600.0, 200.0);
        }
        
        self.setup_battle_ui()?;
        self.phase = BattlePhase::PlayerTurn;
        
        self.add_battle_log("战斗开始！".to_string());
        debug!("战斗初始化完成");
        
        Ok(())
    }
    
    // 设置战斗UI
    fn setup_battle_ui(&mut self) -> Result<(), GameError> {
        // 玩家HP条
        self.player_hp_bar = Some(self.ui_manager.create_element(
            "player_hp".to_string(),
            ElementType::ProgressBar,
            None,
        )?);
        
        if let Some(hp_id) = self.player_hp_bar {
            self.ui_manager.set_element_position(hp_id, Vec2::new(50.0, 450.0))?;
            self.ui_manager.set_element_size(hp_id, Vec2::new(200.0, 20.0))?;
        }
        
        // 敌方HP条
        self.enemy_hp_bar = Some(self.ui_manager.create_element(
            "enemy_hp".to_string(),
            ElementType::ProgressBar,
            None,
        )?);
        
        if let Some(hp_id) = self.enemy_hp_bar {
            self.ui_manager.set_element_position(hp_id, Vec2::new(550.0, 150.0))?;
            self.ui_manager.set_element_size(hp_id, Vec2::new(200.0, 20.0))?;
        }
        
        // 战斗菜单
        self.battle_menu = Some(self.ui_manager.create_element(
            "battle_menu".to_string(),
            ElementType::Panel,
            None,
        )?);
        
        if let Some(menu_id) = self.battle_menu {
            self.ui_manager.set_element_position(menu_id, Vec2::new(50.0, 500.0))?;
            self.ui_manager.set_element_size(menu_id, Vec2::new(700.0, 80.0))?;
        }
        
        // 创建招式按钮
        self.create_move_buttons()?;
        
        debug!("战斗UI初始化完成");
        Ok(())
    }
    
    // 创建招式按钮
    fn create_move_buttons(&mut self) -> Result<(), GameError> {
        if let Some(player_pokemon) = self.player_team.get(self.active_player) {
            self.move_buttons.clear();
            
            for (i, &move_id) in player_pokemon.moves.iter().take(4).enumerate() {
                let button_id = self.ui_manager.create_element(
                    format!("move_button_{}", i),
                    ElementType::Button,
                    self.battle_menu,
                )?;
                
                let x = 10.0 + (i % 2) as f32 * 180.0;
                let y = 10.0 + (i / 2) as f32 * 30.0;
                
                self.ui_manager.set_element_position(button_id, Vec2::new(x, y))?;
                self.ui_manager.set_element_size(button_id, Vec2::new(170.0, 25.0))?;
                self.ui_manager.set_element_text(button_id, format!("招式 {}", move_id))?;
                
                self.move_buttons.push(button_id);
            }
        }
        
        Ok(())
    }
    
    // 处理玩家行动
    fn handle_player_action(&mut self, action: BattleAction) -> Result<(), GameError> {
        self.action_queue.push(action.clone());
        self.phase = BattlePhase::EnemyTurn;
        
        debug!("玩家行动: {:?}", action);
        
        // 生成敌方行动
        let enemy_action = self.generate_enemy_action()?;
        self.action_queue.push(enemy_action);
        
        // 处理行动队列
        self.process_actions()?;
        
        Ok(())
    }
    
    // 生成敌方AI行动
    fn generate_enemy_action(&self) -> Result<BattleAction, GameError> {
        if let Some(enemy_pokemon) = self.enemy_team.get(self.active_enemy) {
            if !enemy_pokemon.moves.is_empty() {
                let move_id = enemy_pokemon.moves[fastrand::usize(0..enemy_pokemon.moves.len())];
                return Ok(BattleAction::Attack { move_id, target: self.active_player });
            }
        }
        
        Ok(BattleAction::Attack { move_id: 1, target: self.active_player })
    }
    
    // 处理行动队列
    fn process_actions(&mut self) -> Result<(), GameError> {
        if self.action_queue.is_empty() {
            self.end_turn()?;
            return Ok(());
        }
        
        // 按速度排序行动
        self.sort_actions_by_priority();
        
        self.phase = BattlePhase::Animation;
        self.animation_playing = true;
        
        Ok(())
    }
    
    // 按优先级排序行动
    fn sort_actions_by_priority(&mut self) {
        // 简化实现：随机顺序
        fastrand::shuffle(&mut self.action_queue);
    }
    
    // 执行下一个行动
    fn execute_next_action(&mut self) -> Result<(), GameError> {
        if let Some(action) = self.action_queue.pop() {
            self.current_action = Some(action.clone());
            self.execute_action(action)?;
        } else {
            self.end_turn()?;
        }
        
        Ok(())
    }
    
    // 执行具体行动
    fn execute_action(&mut self, action: BattleAction) -> Result<(), GameError> {
        match action {
            BattleAction::Attack { move_id, target } => {
                self.execute_attack(move_id, target)?;
            },
            BattleAction::Item { item_id, target } => {
                self.execute_item_use(item_id, target)?;
            },
            BattleAction::Switch { pokemon_index } => {
                self.execute_pokemon_switch(pokemon_index)?;
            },
            BattleAction::Escape => {
                self.attempt_escape()?;
            },
        }
        
        Ok(())
    }
    
    // 执行攻击
    fn execute_attack(&mut self, move_id: u32, target_index: usize) -> Result<(), GameError> {
        // 简化的伤害计算
        let damage = self.calculate_damage(move_id, target_index);
        
        // 应用伤害
        if target_index < self.player_team.len() {
            if let Some(target) = self.player_team.get_mut(target_index) {
                target.current_hp = target.current_hp.saturating_sub(damage);
                self.damage_received += damage;
                
                self.add_battle_log(format!("{}受到了{}点伤害！", target.name, damage));
                
                if target.current_hp == 0 {
                    self.add_battle_log(format!("{}倒下了！", target.name));
                }
            }
        } else if let Some(target) = self.enemy_team.get_mut(target_index - self.player_team.len()) {
            target.current_hp = target.current_hp.saturating_sub(damage);
            self.damage_dealt += damage;
            
            self.add_battle_log(format!("{}受到了{}点伤害！", target.name, damage));
            
            if target.current_hp == 0 {
                self.add_battle_log(format!("{}倒下了！", target.name));
            }
        }
        
        // 启动屏幕震动
        self.start_screen_shake(5.0, 0.5);
        
        // 播放攻击动画
        self.animation_manager.play_animation(
            "attack_basic",
            "attacker",
        ).ok();
        
        self.moves_used += 1;
        
        Ok(())
    }
    
    // 计算伤害
    fn calculate_damage(&self, move_id: u32, target_index: usize) -> u32 {
        // 简化的伤害计算公式
        let base_damage = 50;
        let level_modifier = 1.0;
        let random_factor = 0.85 + fastrand::f32() * 0.3; // 85% - 115%
        
        (base_damage as f32 * level_modifier * random_factor) as u32
    }
    
    // 使用道具
    fn execute_item_use(&mut self, item_id: u32, target: Option<usize>) -> Result<(), GameError> {
        self.add_battle_log(format!("使用了道具 {}！", item_id));
        // 道具效果实现
        Ok(())
    }
    
    // 切换Pokemon
    fn execute_pokemon_switch(&mut self, pokemon_index: usize) -> Result<(), GameError> {
        if pokemon_index < self.player_team.len() {
            self.active_player = pokemon_index;
            self.create_move_buttons()?;
            
            if let Some(pokemon) = self.player_team.get(pokemon_index) {
                self.add_battle_log(format!("上场吧，{}！", pokemon.name));
            }
        }
        
        Ok(())
    }
    
    // 尝试逃跑
    fn attempt_escape(&mut self) -> Result<(), GameError> {
        if self.can_escape {
            let escape_chance = 0.8; // 80%逃跑成功率
            if fastrand::f32() < escape_chance {
                self.phase = BattlePhase::Escape;
                self.add_battle_log("成功逃跑了！".to_string());
            } else {
                self.add_battle_log("逃跑失败！".to_string());
            }
        } else {
            self.add_battle_log("无法逃跑！".to_string());
        }
        
        Ok(())
    }
    
    // 结束回合
    fn end_turn(&mut self) -> Result<(), GameError> {
        self.turn_count += 1;
        self.current_action = None;
        
        // 检查战斗结束条件
        if self.check_battle_end() {
            return Ok(());
        }
        
        // 处理状态效果
        self.process_status_effects()?;
        
        // 开始新回合
        self.phase = BattlePhase::PlayerTurn;
        self.animation_playing = false;
        
        Ok(())
    }
    
    // 检查战斗结束
    fn check_battle_end(&mut self) -> bool {
        let player_alive = self.player_team.iter().any(|p| p.current_hp > 0);
        let enemy_alive = self.enemy_team.iter().any(|p| p.current_hp > 0);
        
        if !player_alive {
            self.phase = BattlePhase::Defeat;
            self.add_battle_log("战斗失败！".to_string());
            return true;
        }
        
        if !enemy_alive {
            self.phase = BattlePhase::Victory;
            self.add_battle_log("战斗胜利！".to_string());
            return true;
        }
        
        false
    }
    
    // 处理状态效果
    fn process_status_effects(&mut self) -> Result<(), GameError> {
        // 处理毒、燃烧等持续伤害状态
        for pokemon in &mut self.player_team {
            for &status_id in &pokemon.status_effects {
                // 应用状态效果
            }
        }
        
        for pokemon in &mut self.enemy_team {
            for &status_id in &pokemon.status_effects {
                // 应用状态效果
            }
        }
        
        Ok(())
    }
    
    // 启动屏幕震动
    fn start_screen_shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
    }
    
    // 更新屏幕震动
    fn update_screen_shake(&mut self, delta_time: f32) {
        if self.shake_duration > 0.0 {
            self.shake_duration -= delta_time;
            
            let shake_amount = self.shake_intensity * (self.shake_duration / 0.5);
            self.screen_shake = Vec2::new(
                (fastrand::f32() - 0.5) * shake_amount,
                (fastrand::f32() - 0.5) * shake_amount,
            );
            
            if self.shake_duration <= 0.0 {
                self.screen_shake = Vec2::ZERO;
            }
        }
    }
    
    // 添加战斗日志
    fn add_battle_log(&mut self, message: String) {
        self.battle_log.push(message);
        if self.battle_log.len() > 10 {
            self.battle_log.remove(0);
        }
        debug!("战斗日志: {}", self.battle_log.last().unwrap_or(&String::new()));
    }
    
    // 更新HP条
    fn update_hp_bars(&mut self) -> Result<(), GameError> {
        if let (Some(player_pokemon), Some(hp_id)) = 
            (self.player_team.get(self.active_player), self.player_hp_bar) {
            let hp_percentage = player_pokemon.current_hp as f32 / player_pokemon.stats.hp as f32;
            self.ui_manager.set_element_value(hp_id, format!("{:.0}%", hp_percentage * 100.0))?;
        }
        
        if let (Some(enemy_pokemon), Some(hp_id)) = 
            (self.enemy_team.get(self.active_enemy), self.enemy_hp_bar) {
            let hp_percentage = enemy_pokemon.current_hp as f32 / enemy_pokemon.stats.hp as f32;
            self.ui_manager.set_element_value(hp_id, format!("{:.0}%", hp_percentage * 100.0))?;
        }
        
        Ok(())
    }
}

impl StateHandler for BattleState {
    fn get_type(&self) -> GameStateType {
        GameStateType::Battle
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("进入战斗状态");
        Ok(())
    }
    
    fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("退出战斗状态");
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), GameError> {
        debug!("暂停战斗状态");
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), GameError> {
        debug!("恢复战斗状态");
        Ok(())
    }
    
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError> {
        // 更新动画
        self.animation_manager.update(delta_time)?;
        
        // 更新状态效果
        self.status_manager.update(delta_time)?;
        
        // 更新屏幕震动
        self.update_screen_shake(delta_time);
        
        // 更新UI
        self.ui_manager.update(delta_time)?;
        
        // 更新HP条
        self.update_hp_bars()?;
        
        // 处理动画状态
        if self.animation_playing {
            self.animation_timer += delta_time;
            
            if !self.animation_manager.is_playing() {
                self.animation_playing = false;
                self.execute_next_action()?;
            }
        }
        
        // 检查战斗结束
        match self.phase {
            BattlePhase::Victory | BattlePhase::Defeat | BattlePhase::Escape => {
                return Ok(StateTransition::Pop);
            },
            _ => {}
        }
        
        Ok(StateTransition::None)
    }
    
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 应用屏幕震动
        let shake_camera = super::Camera {
            position: glam::Vec2::new(self.screen_shake.x, self.screen_shake.y),
            zoom: 1.0,
            rotation: 0.0,
            projection_matrix: glam::Mat4::orthographic_rh(
                -400.0, 400.0, -300.0, 300.0, -1000.0, 1000.0
            ),
        };
        renderer.push_camera(shake_camera);
        
        // 渲染背景
        renderer.draw_sprite(
            Vec2::ZERO,
            Vec2::new(800.0, 600.0),
            [0.1, 0.2, 0.4, 1.0], // 背景色
        );
        
        // 渲染Pokemon
        if let Some(player_pokemon) = self.player_team.get(self.active_player) {
            if let Some(sprite_id) = player_pokemon.sprite_id {
                renderer.draw_sprite(
                    player_pokemon.position,
                    Vec2::new(100.0, 100.0),
                    [1.0, 1.0, 1.0, 1.0], // 白色
                );
            }
        }
        
        if let Some(enemy_pokemon) = self.enemy_team.get(self.active_enemy) {
            if let Some(sprite_id) = enemy_pokemon.sprite_id {
                renderer.draw_sprite(
                    enemy_pokemon.position,
                    Vec2::new(100.0, 100.0),
                    [1.0, 1.0, 1.0, 1.0], // 白色
                );
            }
        }
        
        // 渲染动画
        self.animation_manager.render(renderer)?;
        
        // 渲染UI
        self.ui_manager.render(renderer)?;
        
        renderer.pop_camera();
        
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError> {
        if self.phase != BattlePhase::PlayerTurn {
            return Ok(false);
        }
        
        // 检查是否点击了招式按钮
        for (i, &button_id) in self.move_buttons.iter().enumerate() {
            // 简化的点击检测
            if event.state == crate::input::mouse::MouseState::Pressed {
                if let Some(player_pokemon) = self.player_team.get(self.active_player) {
                    if i < player_pokemon.moves.len() {
                        let move_id = player_pokemon.moves[i];
                        self.handle_player_action(BattleAction::Attack {
                            move_id,
                            target: self.active_enemy,
                        })?;
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        if !pressed || self.phase != BattlePhase::PlayerTurn {
            return Ok(false);
        }
        
        match key {
            "1" | "2" | "3" | "4" => {
                if let Ok(move_index) = key.parse::<usize>() {
                    if move_index > 0 && move_index <= 4 {
                        if let Some(player_pokemon) = self.player_team.get(self.active_player) {
                            if move_index - 1 < player_pokemon.moves.len() {
                                let move_id = player_pokemon.moves[move_index - 1];
                                self.handle_player_action(BattleAction::Attack {
                                    move_id,
                                    target: self.active_enemy,
                                })?;
                                return Ok(true);
                            }
                        }
                    }
                }
            },
            "Escape" => {
                if self.can_escape {
                    self.handle_player_action(BattleAction::Escape)?;
                    return Ok(true);
                }
            },
            _ => {}
        }
        
        Ok(false)
    }
    
    fn handle_gamepad_event(&mut self, event: &GamepadEvent) -> Result<bool, GameError> {
        Ok(false) // 简化实现
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
    // use crate::pokemon::stats::*; // 注释掉直到pokemon模块可用
    
    #[test]
    fn test_battle_state_creation() {
        let battle = BattleState::new();
        assert_eq!(battle.get_type(), GameStateType::Battle);
        assert_eq!(battle.phase, BattlePhase::Initializing);
        assert_eq!(battle.turn_count, 0);
    }
    
    #[test]
    fn test_damage_calculation() {
        let battle = BattleState::new();
        let damage = battle.calculate_damage(1, 0);
        assert!(damage > 0);
        assert!(damage < 100); // 基于简化的伤害公式
    }
}