// 游戏引擎核心 - 统一管理所有子系统
// 开发心理：Engine是整个游戏的心脏，负责协调各个子系统的运行
// 设计模式：使用状态机管理游戏状态，事件驱动的架构

use crate::core::{GameError, Result, GameConfig};
use crate::graphics::Renderer;
use crate::audio::AudioManager;
use crate::input::InputManager;
use crate::world::WorldManager;
use crate::battle::BattleEngine;
use crate::network::NetworkManager;
use std::time::{Duration, Instant};
use log::{info, warn, error};

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Loading,
    MainMenu,
    Gameplay,
    Battle,
    Inventory,
    Settings,
    Paused,
    Shutdown,
}

pub struct Engine {
    config: GameConfig,
    state: GameState,
    previous_state: GameState,
    
    // 子系统
    renderer: Option<Renderer>,
    audio_manager: Option<AudioManager>,
    input_manager: Option<InputManager>,
    world_manager: Option<WorldManager>,
    battle_engine: Option<BattleEngine>,
    network_manager: Option<NetworkManager>,
    
    // 时间管理
    last_frame_time: Instant,
    delta_time: Duration,
    frame_count: u64,
    fps: f64,
    
    // 性能监控
    frame_time_buffer: Vec<Duration>,
    avg_frame_time: Duration,
}

impl Engine {
    pub fn new(config: GameConfig) -> Result<Self> {
        info!("初始化游戏引擎...");
        
        Ok(Engine {
            config,
            state: GameState::Loading,
            previous_state: GameState::Loading,
            
            renderer: None,
            audio_manager: None,
            input_manager: None,
            world_manager: None,
            battle_engine: None,
            network_manager: None,
            
            last_frame_time: Instant::now(),
            delta_time: Duration::from_secs(0),
            frame_count: 0,
            fps: 0.0,
            
            frame_time_buffer: Vec::with_capacity(120),
            avg_frame_time: Duration::from_secs(0),
        })
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        info!("初始化引擎子系统...");
        
        // 初始化渲染器
        self.renderer = Some(Renderer::new(&self.config.graphics)?);
        
        // 初始化音频管理器
        if self.config.audio.enabled {
            self.audio_manager = Some(AudioManager::new(&self.config.audio)?);
        }
        
        // 初始化输入管理器
        self.input_manager = Some(InputManager::new()?);
        
        // 初始化世界管理器
        self.world_manager = Some(WorldManager::new()?);
        
        // 初始化战斗引擎
        self.battle_engine = Some(BattleEngine::new()?);
        
        // 初始化网络管理器（如果启用）
        if self.config.features.multiplayer {
            self.network_manager = Some(NetworkManager::new(&self.config.network)?);
        }
        
        self.change_state(GameState::MainMenu);
        info!("引擎初始化完成");
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<()> {
        info!("开始游戏主循环");
        
        while self.state != GameState::Shutdown {
            self.update_timing();
            
            // 处理输入
            if let Some(ref mut input) = self.input_manager {
                input.update()?;
                self.handle_input(input)?;
            }
            
            // 更新游戏逻辑
            self.update(self.delta_time)?;
            
            // 渲染
            if let Some(ref mut renderer) = self.renderer {
                renderer.begin_frame()?;
                self.render(renderer)?;
                renderer.end_frame()?;
            }
            
            // 性能统计
            self.update_performance_stats();
            
            // 垂直同步
            if self.config.graphics.vsync {
                self.limit_framerate();
            }
        }
        
        info!("游戏主循环结束");
        Ok(())
    }
    
    fn update_timing(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.frame_count += 1;
    }
    
    fn handle_input(&mut self, input: &InputManager) -> Result<()> {
        // 全局输入处理
        if input.is_key_just_pressed("escape") {
            match self.state {
                GameState::MainMenu => self.change_state(GameState::Shutdown),
                GameState::Paused => self.change_state(self.previous_state.clone()),
                _ => self.change_state(GameState::Paused),
            }
        }
        
        if input.is_key_just_pressed("f11") {
            self.toggle_fullscreen()?;
        }
        
        // 状态特定输入处理
        match self.state {
            GameState::MainMenu => self.handle_menu_input(input)?,
            GameState::Gameplay => self.handle_gameplay_input(input)?,
            GameState::Battle => self.handle_battle_input(input)?,
            _ => {}
        }
        
        Ok(())
    }
    
    fn update(&mut self, delta_time: Duration) -> Result<()> {
        match self.state {
            GameState::Loading => self.update_loading()?,
            GameState::MainMenu => self.update_menu(delta_time)?,
            GameState::Gameplay => self.update_gameplay(delta_time)?,
            GameState::Battle => self.update_battle(delta_time)?,
            GameState::Inventory => self.update_inventory(delta_time)?,
            GameState::Settings => self.update_settings(delta_time)?,
            GameState::Paused => {}, // 暂停时不更新
            GameState::Shutdown => {},
        }
        
        // 更新音频系统
        if let Some(ref mut audio) = self.audio_manager {
            audio.update(delta_time)?;
        }
        
        // 更新网络系统
        if let Some(ref mut network) = self.network_manager {
            network.update(delta_time)?;
        }
        
        Ok(())
    }
    
    fn render(&mut self, renderer: &mut Renderer) -> Result<()> {
        match self.state {
            GameState::Loading => self.render_loading(renderer)?,
            GameState::MainMenu => self.render_menu(renderer)?,
            GameState::Gameplay => self.render_gameplay(renderer)?,
            GameState::Battle => self.render_battle(renderer)?,
            GameState::Inventory => self.render_inventory(renderer)?,
            GameState::Settings => self.render_settings(renderer)?,
            GameState::Paused => {
                // 先渲染暂停前的画面
                match self.previous_state {
                    GameState::Gameplay => self.render_gameplay(renderer)?,
                    GameState::Battle => self.render_battle(renderer)?,
                    _ => {}
                }
                // 然后渲染暂停覆盖层
                self.render_pause_overlay(renderer)?;
            }
            GameState::Shutdown => {},
        }
        
        // 渲染性能信息（调试模式）
        if self.config.debug_mode {
            self.render_debug_info(renderer)?;
        }
        
        Ok(())
    }
    
    pub fn change_state(&mut self, new_state: GameState) {
        if self.state != new_state {
            info!("状态切换: {:?} -> {:?}", self.state, new_state);
            self.previous_state = self.state.clone();
            self.state = new_state;
            self.on_state_enter();
        }
    }
    
    fn on_state_enter(&mut self) {
        match self.state {
            GameState::MainMenu => {
                if let Some(ref mut audio) = self.audio_manager {
                    audio.play_music("menu_theme", true);
                }
            }
            GameState::Gameplay => {
                if let Some(ref mut audio) = self.audio_manager {
                    audio.play_music("overworld_theme", true);
                }
            }
            GameState::Battle => {
                if let Some(ref mut audio) = self.audio_manager {
                    audio.play_music("battle_theme", true);
                }
            }
            _ => {}
        }
    }
    
    fn update_performance_stats(&mut self) {
        // 更新帧时间缓冲区
        self.frame_time_buffer.push(self.delta_time);
        if self.frame_time_buffer.len() > 120 {
            self.frame_time_buffer.remove(0);
        }
        
        // 计算平均帧时间和FPS
        if self.frame_count % 60 == 0 {
            let total_time: Duration = self.frame_time_buffer.iter().sum();
            self.avg_frame_time = total_time / self.frame_time_buffer.len() as u32;
            self.fps = 1.0 / self.avg_frame_time.as_secs_f64();
            
            if self.config.debug_mode {
                info!("FPS: {:.1}, 平均帧时间: {:.2}ms", 
                      self.fps, 
                      self.avg_frame_time.as_secs_f64() * 1000.0);
            }
        }
    }
    
    // 各状态的具体实现方法
    fn update_loading(&mut self) -> Result<()> {
        // 加载资源
        info!("加载游戏资源...");
        self.change_state(GameState::MainMenu);
        Ok(())
    }
    
    fn update_menu(&mut self, _delta_time: Duration) -> Result<()> {
        // 菜单逻辑更新
        Ok(())
    }
    
    fn update_gameplay(&mut self, delta_time: Duration) -> Result<()> {
        if let Some(ref mut world) = self.world_manager {
            world.update(delta_time)?;
        }
        Ok(())
    }
    
    fn update_battle(&mut self, delta_time: Duration) -> Result<()> {
        if let Some(ref mut battle) = self.battle_engine {
            battle.update(delta_time)?;
        }
        Ok(())
    }
    
    fn update_inventory(&mut self, _delta_time: Duration) -> Result<()> {
        Ok(())
    }
    
    fn update_settings(&mut self, _delta_time: Duration) -> Result<()> {
        Ok(())
    }
    
    // 渲染方法实现
    fn render_loading(&self, renderer: &mut Renderer) -> Result<()> {
        renderer.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw_text("加载中...", 640, 360, 32, (1.0, 1.0, 1.0, 1.0));
        Ok(())
    }
    
    fn render_menu(&self, renderer: &mut Renderer) -> Result<()> {
        renderer.clear_color(0.2, 0.3, 0.8, 1.0);
        renderer.draw_text("宝可梦游戏", 640, 200, 64, (1.0, 1.0, 1.0, 1.0));
        renderer.draw_text("按回车开始", 640, 400, 32, (0.8, 0.8, 0.8, 1.0));
        Ok(())
    }
    
    fn render_gameplay(&self, renderer: &mut Renderer) -> Result<()> {
        if let Some(ref world) = self.world_manager {
            world.render(renderer)?;
        }
        Ok(())
    }
    
    fn render_battle(&self, renderer: &mut Renderer) -> Result<()> {
        if let Some(ref battle) = self.battle_engine {
            battle.render(renderer)?;
        }
        Ok(())
    }
    
    fn render_inventory(&self, renderer: &mut Renderer) -> Result<()> {
        renderer.clear_color(0.1, 0.1, 0.2, 1.0);
        renderer.draw_text("背包", 640, 100, 48, (1.0, 1.0, 1.0, 1.0));
        Ok(())
    }
    
    fn render_settings(&self, renderer: &mut Renderer) -> Result<()> {
        renderer.clear_color(0.15, 0.15, 0.25, 1.0);
        renderer.draw_text("设置", 640, 100, 48, (1.0, 1.0, 1.0, 1.0));
        Ok(())
    }
    
    fn render_pause_overlay(&self, renderer: &mut Renderer) -> Result<()> {
        renderer.draw_rect(0, 0, 1280, 720, (0.0, 0.0, 0.0, 0.5));
        renderer.draw_text("暂停", 640, 360, 64, (1.0, 1.0, 1.0, 1.0));
        Ok(())
    }
    
    fn render_debug_info(&self, renderer: &mut Renderer) -> Result<()> {
        let fps_text = format!("FPS: {:.1}", self.fps);
        let frame_time_text = format!("帧时间: {:.2}ms", self.avg_frame_time.as_secs_f64() * 1000.0);
        
        renderer.draw_text(&fps_text, 10, 10, 16, (0.0, 1.0, 0.0, 1.0));
        renderer.draw_text(&frame_time_text, 10, 30, 16, (0.0, 1.0, 0.0, 1.0));
        
        Ok(())
    }
    
    // 输入处理方法
    fn handle_menu_input(&mut self, input: &InputManager) -> Result<()> {
        if input.is_key_just_pressed("return") {
            self.change_state(GameState::Gameplay);
        }
        Ok(())
    }
    
    fn handle_gameplay_input(&mut self, input: &InputManager) -> Result<()> {
        if input.is_key_just_pressed("tab") {
            self.change_state(GameState::Inventory);
        }
        Ok(())
    }
    
    fn handle_battle_input(&mut self, _input: &InputManager) -> Result<()> {
        // 战斗输入处理
        Ok(())
    }
    
    // 工具方法
    fn toggle_fullscreen(&mut self) -> Result<()> {
        self.config.graphics.fullscreen = !self.config.graphics.fullscreen;
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_fullscreen(self.config.graphics.fullscreen)?;
        }
        Ok(())
    }
    
    fn limit_framerate(&self) {
        let target_frame_time = Duration::from_secs_f64(1.0 / self.config.graphics.target_fps as f64);
        if self.delta_time < target_frame_time {
            let sleep_time = target_frame_time - self.delta_time;
            std::thread::sleep(sleep_time);
        }
    }
    
    pub fn shutdown(&mut self) {
        info!("关闭游戏引擎...");
        self.change_state(GameState::Shutdown);
        
        // 清理子系统
        self.network_manager = None;
        self.battle_engine = None;
        self.world_manager = None;
        self.input_manager = None;
        self.audio_manager = None;
        self.renderer = None;
        
        info!("游戏引擎已关闭");
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if self.state != GameState::Shutdown {
            self.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let config = GameConfig::default();
        let engine = Engine::new(config);
        assert!(engine.is_ok());
        
        let mut engine = engine.unwrap();
        assert_eq!(engine.state, GameState::Loading);
        assert_eq!(engine.frame_count, 0);
    }
    
    #[test]
    fn test_state_changes() {
        let config = GameConfig::default();
        let mut engine = Engine::new(config).unwrap();
        
        assert_eq!(engine.state, GameState::Loading);
        
        engine.change_state(GameState::MainMenu);
        assert_eq!(engine.state, GameState::MainMenu);
        assert_eq!(engine.previous_state, GameState::Loading);
    }
}