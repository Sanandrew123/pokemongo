/*
* 开发心理过程：
* 1. 创建应用程序核心结构，负责整个游戏的生命周期管理
* 2. 实现Bevy插件接口，提供模块化的架构
* 3. 管理游戏配置、资源初始化和清理
* 4. 提供统一的错误处理和日志记录
* 5. 支持热重载和开发者工具
* 6. 集成性能分析和监控系统
* 7. 提供扩展接口供其他模块使用
*/

use bevy::prelude::*;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

use crate::core::{config::GameConfig, error::GameResult, time::GameTimer};

#[derive(Resource, Debug)]
pub struct PokemonAppState {
    pub start_time: Instant,
    pub total_frames: u64,
    pub frame_time_history: Vec<Duration>,
    pub performance_stats: PerformanceStats,
    pub debug_info: DebugInfo,
}

#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub avg_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub frame_time_ms: f32,
    pub memory_usage_mb: f32,
    pub draw_calls: u32,
    pub entities_count: u32,
    pub systems_time_ms: f32,
}

#[derive(Debug, Default)]
pub struct DebugInfo {
    pub show_fps: bool,
    pub show_entity_count: bool,
    pub show_memory_usage: bool,
    pub show_draw_calls: bool,
    pub show_profiler: bool,
    pub wireframe_mode: bool,
    pub pause_simulation: bool,
}

impl Default for PokemonAppState {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            total_frames: 0,
            frame_time_history: Vec::with_capacity(120),
            performance_stats: PerformanceStats::default(),
            debug_info: DebugInfo::default(),
        }
    }
}

#[derive(Event)]
pub struct AppInitializedEvent;

#[derive(Event)]
pub struct AppShutdownEvent;

#[derive(Event)]
pub struct PerformanceUpdateEvent(pub PerformanceStats);

pub struct PokemonApp;

impl Plugin for PokemonApp {
    fn build(&self, app: &mut App) {
        info!("初始化Pokemon应用程序核心");
        
        app.init_resource::<PokemonAppState>()
           .add_event::<AppInitializedEvent>()
           .add_event::<AppShutdownEvent>()
           .add_event::<PerformanceUpdateEvent>()
           .add_systems(Startup, (
               initialize_app,
               setup_debug_systems,
               load_initial_resources,
           ).chain())
           .add_systems(Update, (
               update_performance_stats,
               handle_debug_input,
               monitor_memory_usage,
               update_frame_timing,
           ))
           .add_systems(Last, (
               cleanup_expired_data,
               send_performance_events,
           ));

        #[cfg(debug_assertions)]
        {
            app.add_systems(Update, (
                debug_entity_inspector,
                debug_system_profiler,
            ));
        }

        info!("Pokemon应用程序核心初始化完成");
    }
}

fn initialize_app(
    mut commands: Commands,
    mut app_state: ResMut<PokemonAppState>,
    mut events: EventWriter<AppInitializedEvent>,
    config: Res<GameConfig>,
) {
    info!("开始应用程序初始化");
    
    app_state.start_time = Instant::now();
    app_state.debug_info.show_fps = config.debug_mode;
    
    commands.insert_resource(GameTimer::new());
    
    events.send(AppInitializedEvent);
    
    info!("应用程序初始化完成，运行时间: {:?}", 
        app_state.start_time.elapsed());
}

fn setup_debug_systems(
    config: Res<GameConfig>,
    mut app_state: ResMut<PokemonAppState>,
) {
    if config.debug_mode {
        info!("启用调试模式");
        
        app_state.debug_info.show_fps = true;
        app_state.debug_info.show_entity_count = true;
        app_state.debug_info.show_memory_usage = true;
        
        #[cfg(debug_assertions)]
        {
            app_state.debug_info.show_profiler = true;
        }
    }
}

fn load_initial_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("加载初始资源");
    
    let default_font = asset_server.load("fonts/default.ttf");
    commands.insert_resource(DefaultFont(default_font));
}

fn update_performance_stats(
    time: Res<Time>,
    mut app_state: ResMut<PokemonAppState>,
    query: Query<Entity>,
) {
    let frame_time = time.delta();
    app_state.total_frames += 1;
    
    app_state.frame_time_history.push(frame_time);
    if app_state.frame_time_history.len() > 120 {
        app_state.frame_time_history.remove(0);
    }
    
    let fps = 1.0 / frame_time.as_secs_f32();
    
    let avg_frame_time: Duration = app_state.frame_time_history.iter().sum::<Duration>() 
        / app_state.frame_time_history.len() as u32;
    let avg_fps = 1.0 / avg_frame_time.as_secs_f32();
    
    let min_frame_time = app_state.frame_time_history.iter().min().unwrap_or(&frame_time);
    let max_frame_time = app_state.frame_time_history.iter().max().unwrap_or(&frame_time);
    
    app_state.performance_stats.avg_fps = avg_fps;
    app_state.performance_stats.min_fps = 1.0 / max_frame_time.as_secs_f32();
    app_state.performance_stats.max_fps = 1.0 / min_frame_time.as_secs_f32();
    app_state.performance_stats.frame_time_ms = frame_time.as_secs_f32() * 1000.0;
    app_state.performance_stats.entities_count = query.iter().count() as u32;
}

fn handle_debug_input(
    input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<PokemonAppState>,
) {
    if input.just_pressed(KeyCode::F1) {
        app_state.debug_info.show_fps = !app_state.debug_info.show_fps;
    }
    
    if input.just_pressed(KeyCode::F2) {
        app_state.debug_info.show_entity_count = !app_state.debug_info.show_entity_count;
    }
    
    if input.just_pressed(KeyCode::F3) {
        app_state.debug_info.show_memory_usage = !app_state.debug_info.show_memory_usage;
    }
    
    if input.just_pressed(KeyCode::F4) {
        app_state.debug_info.wireframe_mode = !app_state.debug_info.wireframe_mode;
    }
    
    if input.just_pressed(KeyCode::F5) {
        app_state.debug_info.pause_simulation = !app_state.debug_info.pause_simulation;
    }
}

fn monitor_memory_usage(
    mut app_state: ResMut<PokemonAppState>,
) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(pages) = statm.split_whitespace().next() {
                if let Ok(pages) = pages.parse::<u64>() {
                    let page_size = 4096;
                    let memory_bytes = pages * page_size;
                    app_state.performance_stats.memory_usage_mb = 
                        (memory_bytes as f32) / (1024.0 * 1024.0);
                }
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        use std::alloc::{GlobalAlloc, System};
        app_state.performance_stats.memory_usage_mb = 64.0;
    }
}

fn update_frame_timing(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
) {
    timer.update(time.delta());
}

fn cleanup_expired_data(
    mut app_state: ResMut<PokemonAppState>,
) {
    const MAX_HISTORY_SIZE: usize = 300;
    
    if app_state.frame_time_history.len() > MAX_HISTORY_SIZE {
        let remove_count = app_state.frame_time_history.len() - MAX_HISTORY_SIZE;
        app_state.frame_time_history.drain(0..remove_count);
    }
}

fn send_performance_events(
    app_state: Res<PokemonAppState>,
    mut events: EventWriter<PerformanceUpdateEvent>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();
    
    if *timer >= 1.0 {
        events.send(PerformanceUpdateEvent(app_state.performance_stats.clone()));
        *timer = 0.0;
        
        if app_state.performance_stats.avg_fps < 30.0 {
            warn!("低帧率检测: {:.1} FPS", app_state.performance_stats.avg_fps);
        }
        
        if app_state.performance_stats.memory_usage_mb > 512.0 {
            warn!("高内存使用: {:.1} MB", app_state.performance_stats.memory_usage_mb);
        }
    }
}

#[cfg(debug_assertions)]
fn debug_entity_inspector(
    query: Query<Entity, Added<Transform>>,
    mut app_state: ResMut<PokemonAppState>,
) {
    if !app_state.debug_info.show_entity_count {
        return;
    }
    
    let new_entities = query.iter().count();
    if new_entities > 0 {
        info!("新增实体数量: {}", new_entities);
    }
}

#[cfg(debug_assertions)]
fn debug_system_profiler(
    time: Res<Time>,
    app_state: Res<PokemonAppState>,
) {
    if !app_state.debug_info.show_profiler {
        return;
    }
    
    if app_state.total_frames % 300 == 0 {
        info!("系统性能分析 - 总帧数: {}, 运行时间: {:?}", 
            app_state.total_frames, 
            app_state.start_time.elapsed());
    }
}

#[derive(Resource)]
pub struct DefaultFont(pub Handle<Font>);

impl Clone for PerformanceStats {
    fn clone(&self) -> Self {
        Self {
            avg_fps: self.avg_fps,
            min_fps: self.min_fps,
            max_fps: self.max_fps,
            frame_time_ms: self.frame_time_ms,
            memory_usage_mb: self.memory_usage_mb,
            draw_calls: self.draw_calls,
            entities_count: self.entities_count,
            systems_time_ms: self.systems_time_ms,
        }
    }
}

pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_build_timestamp() -> &'static str {
    env!("BUILD_TIMESTAMP")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_pokemon_app_plugin() {
        let mut app = App::new();
        app.add_plugins(PokemonApp);
        
        assert!(app.world().contains_resource::<PokemonAppState>());
    }

    #[test]
    fn test_performance_stats_default() {
        let stats = PerformanceStats::default();
        assert_eq!(stats.avg_fps, 0.0);
        assert_eq!(stats.entities_count, 0);
    }

    #[test]
    fn test_debug_info_toggle() {
        let mut debug_info = DebugInfo::default();
        assert!(!debug_info.show_fps);
        
        debug_info.show_fps = true;
        assert!(debug_info.show_fps);
    }
}