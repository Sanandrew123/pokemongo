// 应用程序核心 - 游戏主循环和状态管理
// 开发心理：基于Bevy ECS架构的清晰状态管理，确保游戏循环的稳定性和可扩展性
// 使用状态机模式管理不同的游戏场景，便于添加新功能和调试

use crate::core::{
    config::{GameConfig, ConfigManager},
    error::{GameError, Result},
    time::GameTime,
};
use bevy::prelude::*;
use std::time::{Duration, Instant};

// 游戏状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    Loading,
    MainMenu,
    InGame,
    Battle,
    Paused,
    Settings,
    Connecting,
    Error,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Loading
    }
}

// 应用程序主结构
pub struct App {
    bevy_app: bevy::prelude::App,
    config_manager: ConfigManager,
    game_time: GameTime,
    target_fps: u32,
    frame_time: Duration,
    last_frame: Instant,
}

impl App {
    // 创建新的应用程序实例
    pub fn new(config: GameConfig) -> Result<Self> {
        log::info!("初始化游戏应用程序");
        
        let config_manager = ConfigManager::new()?;
        let target_fps = config.graphics.max_fps;
        let frame_time = Duration::from_secs_f64(1.0 / target_fps as f64);
        
        let mut bevy_app = bevy::prelude::App::new();
        
        // 添加基础插件
        bevy_app
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pokemon GO - 高性能游戏".to_string(),
                    resolution: (config.graphics.width as f32, config.graphics.height as f32).into(),
                    present_mode: if config.graphics.vsync {
                        bevy::window::PresentMode::AutoVsync
                    } else {
                        bevy::window::PresentMode::AutoNoVsync
                    },
                    mode: if config.graphics.fullscreen {
                        bevy::window::WindowMode::BorderlessFullscreen
                    } else {
                        bevy::window::WindowMode::Windowed
                    },
                    ..default()
                }),
                ..default()
            }))
            .insert_resource(Time::<Fixed>::from_hz(target_fps as f64))
            .init_state::<GameState>()
            .insert_resource(GameConfig::load_or_default()?)
            .insert_resource(GameTime::new())
            
            // 添加系统
            .add_systems(Startup, setup_system)
            .add_systems(Update, (
                config_hot_reload_system,
                fps_counter_system,
                state_transition_system,
                exit_system,
            ))
            
            // 按状态添加系统
            .add_systems(OnEnter(GameState::Loading), enter_loading_state)
            .add_systems(Update, loading_system.run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), exit_loading_state)
            
            .add_systems(OnEnter(GameState::MainMenu), enter_main_menu_state)
            .add_systems(Update, main_menu_system.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), exit_main_menu_state)
            
            .add_systems(OnEnter(GameState::InGame), enter_game_state)
            .add_systems(Update, game_update_system.run_if(in_state(GameState::InGame)))
            .add_systems(OnExit(GameState::InGame), exit_game_state)
            
            .add_systems(OnEnter(GameState::Battle), enter_battle_state)
            .add_systems(Update, battle_system.run_if(in_state(GameState::Battle)))
            .add_systems(OnExit(GameState::Battle), exit_battle_state)
            
            .add_systems(OnEnter(GameState::Error), enter_error_state)
            .add_systems(Update, error_system.run_if(in_state(GameState::Error)));
        
        Ok(Self {
            bevy_app,
            config_manager,
            game_time: GameTime::new(),
            target_fps,
            frame_time,
            last_frame: Instant::now(),
        })
    }
    
    // 运行游戏主循环
    pub fn run(mut self) -> Result<()> {
        log::info!("启动游戏循环");
        
        // 运行Bevy应用程序
        self.bevy_app.run();
        
        Ok(())
    }
    
    // 更新配置
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut GameConfig),
    {
        self.config_manager.update_config(updater)
    }
    
    // 获取当前配置
    pub fn get_config(&self) -> &GameConfig {
        self.config_manager.get_config()
    }
}

// 初始化系统
fn setup_system(mut commands: Commands) {
    log::info!("设置游戏场景");
    
    // 添加默认相机
    commands.spawn(Camera2dBundle::default());
    
    // 添加游戏时间资源
    commands.insert_resource(GameTime::new());
}

// 配置热重载系统
fn config_hot_reload_system(
    mut config_manager: ResMut<ConfigManager>,
    mut config: ResMut<GameConfig>,
) {
    if let Ok(reloaded) = config_manager.check_reload() {
        if reloaded {
            *config = config_manager.get_config().clone();
            log::info!("配置已热重载");
        }
    }
}

// FPS计数器系统
fn fps_counter_system(
    mut game_time: ResMut<GameTime>,
    time: Res<Time>,
) {
    game_time.update(time.delta());
    
    // 每秒输出一次FPS信息
    if game_time.frame_count() % 60 == 0 {
        log::debug!("FPS: {:.1}, Frame Time: {:.2}ms", 
                   game_time.fps(), 
                   game_time.average_frame_time() * 1000.0);
    }
}

// 状态转换系统
fn state_transition_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match **current_state {
        GameState::Loading => {
            // 加载完成后进入主菜单
            // 这里应该检查实际的加载进度
        },
        GameState::MainMenu => {
            if keyboard.just_pressed(KeyCode::Enter) {
                next_state.set(GameState::InGame);
            }
        },
        GameState::InGame => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
            }
            if keyboard.just_pressed(KeyCode::KeyB) {
                next_state.set(GameState::Battle);
            }
        },
        GameState::Battle => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::InGame);
            }
        },
        GameState::Paused => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::InGame);
            }
        },
        _ => {}
    }
}

// 退出系统
fn exit_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::F4) && keyboard.pressed(KeyCode::AltLeft) {
        log::info!("用户请求退出游戏");
        exit.send(AppExit);
    }
}

// 加载状态系统
fn enter_loading_state() {
    log::info!("进入加载状态");
}

fn loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
    }
    
    if let Some(ref mut t) = timer.as_mut() {
        t.tick(time.delta());
        if t.finished() {
            next_state.set(GameState::MainMenu);
        }
    }
}

fn exit_loading_state() {
    log::info!("退出加载状态");
}

// 主菜单状态系统
fn enter_main_menu_state() {
    log::info!("进入主菜单状态");
}

fn main_menu_system() {
    // 主菜单逻辑
}

fn exit_main_menu_state() {
    log::info!("退出主菜单状态");
}

// 游戏状态系统
fn enter_game_state() {
    log::info!("进入游戏状态");
}

fn game_update_system(
    mut game_time: ResMut<GameTime>,
    time: Res<Time>,
) {
    game_time.update(time.delta());
    
    // 游戏核心更新逻辑
    // 这里会调用世界更新、宝可梦AI、物理系统等
}

fn exit_game_state() {
    log::info!("退出游戏状态");
}

// 战斗状态系统
fn enter_battle_state() {
    log::info!("进入战斗状态");
}

fn battle_system() {
    // 战斗系统逻辑
}

fn exit_battle_state() {
    log::info!("退出战斗状态");
}

// 错误状态系统
fn enter_error_state() {
    log::error!("进入错误状态");
}

fn error_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // 错误状态处理，允许用户返回主菜单
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

// 性能统计结构
#[derive(Resource, Debug)]
pub struct PerformanceStats {
    pub frame_count: u64,
    pub total_time: Duration,
    pub min_frame_time: Duration,
    pub max_frame_time: Duration,
    pub avg_frame_time: Duration,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            total_time: Duration::ZERO,
            min_frame_time: Duration::from_secs(1),
            max_frame_time: Duration::ZERO,
            avg_frame_time: Duration::ZERO,
        }
    }
}

impl PerformanceStats {
    pub fn update(&mut self, frame_time: Duration) {
        self.frame_count += 1;
        self.total_time += frame_time;
        self.min_frame_time = self.min_frame_time.min(frame_time);
        self.max_frame_time = self.max_frame_time.max(frame_time);
        self.avg_frame_time = self.total_time / self.frame_count as u32;
    }
    
    pub fn fps(&self) -> f64 {
        if self.avg_frame_time.is_zero() {
            0.0
        } else {
            1.0 / self.avg_frame_time.as_secs_f64()
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_state_transitions() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
    
    #[test]
    fn test_performance_stats() {
        let mut stats = PerformanceStats::default();
        
        let frame_time = Duration::from_millis(16); // ~60 FPS
        stats.update(frame_time);
        
        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.min_frame_time, frame_time);
        assert_eq!(stats.max_frame_time, frame_time);
        
        let fps = stats.fps();
        assert!((fps - 62.5).abs() < 1.0); // 约等于62.5 FPS
    }
}