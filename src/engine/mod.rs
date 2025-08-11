/*
 * 游戏引擎模块 - Game Engine Module
 * 
 * 开发心理过程：
 * 设计高性能游戏引擎的核心模块，提供渲染、输入、音频、资源管理等基础功能
 * 需要考虑模块间的解耦、性能优化和扩展性
 * 重点关注引擎架构的清晰性和使用便利性
 */

pub mod renderer;
pub mod input;
pub mod audio;
pub mod resource;
pub mod scene;

use bevy::prelude::*;
use crate::core::error::{GameResult, GameError};
use std::collections::HashMap;

// 引擎配置
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub target_fps: u32,
    pub enable_debug: bool,
    pub asset_path: String,
    pub shader_path: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "Pokemon Game Engine".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            msaa_samples: 4,
            target_fps: 60,
            enable_debug: false,
            asset_path: "assets/".to_string(),
            shader_path: "assets/shaders/".to_string(),
        }
    }
}

// 引擎状态
#[derive(Debug, Clone, PartialEq)]
pub enum EngineState {
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

// 引擎事件
#[derive(Debug, Clone)]
pub enum EngineEvent {
    WindowResized(u32, u32),
    WindowClosed,
    FocusGained,
    FocusLost,
    MinimizeRequested,
    RestoreRequested,
    PerformanceWarning(String),
    ResourceLoaded(String),
    ResourceFailed(String),
}

// 引擎统计信息
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub frame_count: u64,
    pub fps: f32,
    pub frame_time: f32,
    pub render_time: f32,
    pub update_time: f32,
    pub memory_usage: usize,
    pub draw_calls: u32,
    pub triangles: u32,
    pub texture_memory: usize,
    pub audio_active_sources: u32,
}

impl Default for EngineStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            fps: 0.0,
            frame_time: 0.0,
            render_time: 0.0,
            update_time: 0.0,
            memory_usage: 0,
            draw_calls: 0,
            triangles: 0,
            texture_memory: 0,
            audio_active_sources: 0,
        }
    }
}

// 引擎主结构
pub struct GameEngine {
    pub config: EngineConfig,
    pub state: EngineState,
    pub stats: EngineStats,
    pub renderer: renderer::Renderer,
    pub input_manager: input::InputManager,
    pub audio_manager: audio::AudioManager,
    pub resource_manager: resource::ResourceManager,
    pub scene_manager: scene::SceneManager,
    // Camera system moved to graphics module
    start_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    frame_times: Vec<f32>,
}

impl GameEngine {
    // 创建新的引擎实例
    pub fn new(config: EngineConfig) -> GameResult<Self> {
        info!("初始化游戏引擎...");
        
        let renderer = renderer::Renderer::new(&config)?;
        let input_manager = input::InputManager::new()?;
        let audio_manager = audio::AudioManager::new(&config)?;
        let resource_manager = resource::ResourceManager::new(&config.asset_path)?;
        let scene_manager = scene::SceneManager::new()?;
        // Camera system initialization moved to graphics module

        Ok(Self {
            config,
            state: EngineState::Initializing,
            stats: EngineStats::default(),
            renderer,
            input_manager,
            audio_manager,
            resource_manager,
            scene_manager,
            start_time: std::time::Instant::now(),
            last_frame_time: std::time::Instant::now(),
            frame_times: Vec::with_capacity(60),
        })
    }

    // 初始化引擎
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("引擎初始化开始");
        
        // 初始化各个子系统
        self.renderer.initialize()?;
        self.input_manager.initialize()?;
        self.audio_manager.initialize()?;
        self.resource_manager.initialize()?;
        self.scene_manager.initialize()?;
        // Camera system initialization moved to graphics module

        self.state = EngineState::Running;
        info!("引擎初始化完成");
        
        Ok(())
    }

    // 主循环更新
    pub fn update(&mut self, delta_time: f32) -> GameResult<()> {
        if self.state != EngineState::Running {
            return Ok(());
        }

        let update_start = std::time::Instant::now();

        // 更新统计信息
        self.update_stats(delta_time)?;

        // 更新输入系统
        self.input_manager.update(delta_time)?;

        // 更新场景管理器
        self.scene_manager.update(delta_time)?;

        // 更新音频系统
        self.audio_manager.update(delta_time)?;

        // Camera system update moved to graphics module

        // 更新资源管理器（异步加载）
        self.resource_manager.update()?;

        self.stats.update_time = update_start.elapsed().as_secs_f32() * 1000.0;
        
        Ok(())
    }

    // 渲染
    pub fn render(&mut self) -> GameResult<()> {
        if self.state != EngineState::Running {
            return Ok(());
        }

        let render_start = std::time::Instant::now();

        // 开始渲染帧
        self.renderer.begin_frame()?;

        // Camera handling moved to graphics module

        // 渲染当前场景
        if let Some(active_scene) = self.scene_manager.get_active_scene_mut() {
            self.renderer.render_scene(active_scene)?;
        }

        // 渲染调试信息（如果启用）
        if self.config.enable_debug {
            self.render_debug_info()?;
        }

        // 结束渲染帧
        self.renderer.end_frame()?;

        self.stats.render_time = render_start.elapsed().as_secs_f32() * 1000.0;
        self.stats.draw_calls = self.renderer.get_draw_calls();
        self.stats.triangles = self.renderer.get_triangle_count();

        Ok(())
    }

    // 处理引擎事件
    pub fn handle_event(&mut self, event: &EngineEvent) -> GameResult<()> {
        match event {
            EngineEvent::WindowResized(width, height) => {
                self.config.width = *width;
                self.config.height = *height;
                self.renderer.resize(*width, *height)?;
                // Camera aspect ratio handling moved to graphics module
            },
            EngineEvent::WindowClosed => {
                self.shutdown()?;
            },
            EngineEvent::FocusLost => {
                if self.state == EngineState::Running {
                    self.pause()?;
                }
            },
            EngineEvent::FocusGained => {
                if self.state == EngineState::Paused {
                    self.resume()?;
                }
            },
            EngineEvent::PerformanceWarning(message) => {
                warn!("性能警告: {}", message);
            },
            EngineEvent::ResourceLoaded(path) => {
                info!("资源加载完成: {}", path);
            },
            EngineEvent::ResourceFailed(path) => {
                error!("资源加载失败: {}", path);
            },
            _ => {}
        }
        Ok(())
    }

    // 暂停引擎
    pub fn pause(&mut self) -> GameResult<()> {
        if self.state == EngineState::Running {
            self.state = EngineState::Paused;
            self.audio_manager.pause_all()?;
            info!("引擎已暂停");
        }
        Ok(())
    }

    // 恢复引擎
    pub fn resume(&mut self) -> GameResult<()> {
        if self.state == EngineState::Paused {
            self.state = EngineState::Running;
            self.audio_manager.resume_all()?;
            self.last_frame_time = std::time::Instant::now();
            info!("引擎已恢复");
        }
        Ok(())
    }

    // 关闭引擎
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("引擎开始关闭");
        
        self.state = EngineState::Stopping;

        // 关闭各个子系统
        self.scene_manager.shutdown()?;
        self.audio_manager.shutdown()?;
        self.resource_manager.shutdown()?;
        self.renderer.shutdown()?;
        self.input_manager.shutdown()?;
        // Camera system shutdown moved to graphics module

        self.state = EngineState::Stopped;
        info!("引擎关闭完成");
        
        Ok(())
    }

    // 更新统计信息
    fn update_stats(&mut self, delta_time: f32) -> GameResult<()> {
        self.stats.frame_count += 1;
        
        // 更新帧时间
        let current_time = std::time::Instant::now();
        self.stats.frame_time = current_time.duration_since(self.last_frame_time).as_secs_f32() * 1000.0;
        self.last_frame_time = current_time;

        // 保存最近的帧时间用于FPS计算
        self.frame_times.push(self.stats.frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // 计算FPS
        if !self.frame_times.is_empty() {
            let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.stats.fps = if avg_frame_time > 0.0 { 1000.0 / avg_frame_time } else { 0.0 };
        }

        // 更新内存使用情况（简化版）
        self.stats.memory_usage = self.get_memory_usage();
        self.stats.texture_memory = self.renderer.get_texture_memory_usage();
        self.stats.audio_active_sources = self.audio_manager.get_active_source_count();

        // 性能警告
        if self.stats.fps < self.config.target_fps as f32 * 0.8 {
            if self.stats.frame_count % 300 == 0 { // 每5秒检查一次
                let warning = format!("FPS过低: {:.1} (目标: {})", self.stats.fps, self.config.target_fps);
                self.handle_event(&EngineEvent::PerformanceWarning(warning))?;
            }
        }

        Ok(())
    }

    // 获取内存使用情况（简化实现）
    fn get_memory_usage(&self) -> usize {
        // 实际实现需要系统特定的API
        // 这里返回一个估计值
        std::mem::size_of::<Self>() + 
        self.renderer.get_memory_usage() +
        self.resource_manager.get_memory_usage() +
        self.audio_manager.get_memory_usage()
    }

    // 渲染调试信息
    fn render_debug_info(&mut self) -> GameResult<()> {
        let debug_text = format!(
            "FPS: {:.1} | Frame: {:.2}ms | Render: {:.2}ms | Update: {:.2}ms\nDraw Calls: {} | Triangles: {} | Memory: {:.1}MB",
            self.stats.fps,
            self.stats.frame_time,
            self.stats.render_time,
            self.stats.update_time,
            self.stats.draw_calls,
            self.stats.triangles,
            self.stats.memory_usage as f32 / 1024.0 / 1024.0
        );

        self.renderer.render_debug_text(&debug_text, 10.0, 10.0)?;
        Ok(())
    }

    // 获取引擎运行时间
    pub fn get_runtime(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    // 获取引擎状态
    pub fn get_state(&self) -> EngineState {
        self.state.clone()
    }

    // 获取引擎统计信息
    pub fn get_stats(&self) -> &EngineStats {
        &self.stats
    }

    // 设置目标FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.config.target_fps = fps;
    }

    // 切换全屏模式
    pub fn toggle_fullscreen(&mut self) -> GameResult<()> {
        self.config.fullscreen = !self.config.fullscreen;
        self.renderer.set_fullscreen(self.config.fullscreen)?;
        Ok(())
    }

    // 切换VSync
    pub fn toggle_vsync(&mut self) -> GameResult<()> {
        self.config.vsync = !self.config.vsync;
        self.renderer.set_vsync(self.config.vsync)?;
        Ok(())
    }

    // 设置MSAA样本数
    pub fn set_msaa_samples(&mut self, samples: u32) -> GameResult<()> {
        self.config.msaa_samples = samples;
        self.renderer.set_msaa_samples(samples)?;
        Ok(())
    }

    // 获取渲染器引用
    pub fn renderer(&mut self) -> &mut renderer::Renderer {
        &mut self.renderer
    }

    // 获取输入管理器引用
    pub fn input_manager(&mut self) -> &mut input::InputManager {
        &mut self.input_manager
    }

    // 获取音频管理器引用
    pub fn audio_manager(&mut self) -> &mut audio::AudioManager {
        &mut self.audio_manager
    }

    // 获取资源管理器引用
    pub fn resource_manager(&mut self) -> &mut resource::ResourceManager {
        &mut self.resource_manager
    }

    // 获取场景管理器引用
    pub fn scene_manager(&mut self) -> &mut scene::SceneManager {
        &mut self.scene_manager
    }

    // Camera system methods moved to graphics module

    // 检查是否应该退出
    pub fn should_quit(&self) -> bool {
        matches!(self.state, EngineState::Stopping | EngineState::Stopped | EngineState::Error)
    }

    // 设置错误状态
    pub fn set_error_state(&mut self, error: &str) {
        error!("引擎错误: {}", error);
        self.state = EngineState::Error;
    }

    // 获取配置
    pub fn get_config(&self) -> &EngineConfig {
        &self.config
    }

    // 获取可变配置引用
    pub fn get_config_mut(&mut self) -> &mut EngineConfig {
        &mut self.config
    }
}

impl Drop for GameEngine {
    fn drop(&mut self) {
        if !self.should_quit() {
            let _ = self.shutdown();
        }
    }
}

// 引擎构建器
pub struct EngineBuilder {
    config: EngineConfig,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.config.title = title.to_string();
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.config.fullscreen = fullscreen;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.vsync = vsync;
        self
    }

    pub fn msaa_samples(mut self, samples: u32) -> Self {
        self.config.msaa_samples = samples;
        self
    }

    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.target_fps = fps;
        self
    }

    pub fn enable_debug(mut self, debug: bool) -> Self {
        self.config.enable_debug = debug;
        self
    }

    pub fn asset_path(mut self, path: &str) -> Self {
        self.config.asset_path = path.to_string();
        self
    }

    pub fn build(self) -> GameResult<GameEngine> {
        GameEngine::new(self.config)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// 便捷函数
pub fn create_default_engine() -> GameResult<GameEngine> {
    EngineBuilder::new()
        .title("Pokemon Game")
        .window_size(1280, 720)
        .vsync(true)
        .msaa_samples(4)
        .target_fps(60)
        .enable_debug(cfg!(debug_assertions))
        .build()
}

pub fn create_high_performance_engine() -> GameResult<GameEngine> {
    EngineBuilder::new()
        .title("Pokemon Game - High Performance")
        .window_size(1920, 1080)
        .fullscreen(false)
        .vsync(false)
        .msaa_samples(2)
        .target_fps(120)
        .enable_debug(false)
        .build()
}

pub fn create_low_spec_engine() -> GameResult<GameEngine> {
    EngineBuilder::new()
        .title("Pokemon Game - Low Spec")
        .window_size(800, 600)
        .fullscreen(false)
        .vsync(true)
        .msaa_samples(1)
        .target_fps(30)
        .enable_debug(false)
        .build()
}