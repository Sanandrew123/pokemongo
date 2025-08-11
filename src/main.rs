/*
* 开发心理过程：
* 1. 这是游戏的主入口文件，需要初始化Bevy游戏引擎
* 2. 设置游戏窗口、基本配置和插件系统
* 3. 添加游戏状态管理和主要系统
* 4. 使用feature flags控制不同模块的编译
* 5. 集成自定义模块：Pokemon系统、战斗系统、世界系统等
* 6. 提供清晰的应用程序生命周期管理
* 7. 确保高性能启动和资源加载
*/

use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy::window::{WindowPlugin, WindowResolution};
use bevy::asset::AssetPlugin;

// 核心模块
mod core;
mod utils;
mod ecs;
mod data;
mod assets;
mod audio;
mod graphics;
mod input;
mod states;
mod save;
mod game_modes;
mod creature_engine;

// 游戏系统模块
#[cfg(feature = "pokemon-wip")]
mod pokemon;

#[cfg(feature = "battle-wip")]
mod battle;

mod world;
mod player;
mod ui;

#[cfg(feature = "network-wip")]
mod network;

// FFI接口
#[cfg(feature = "native")]
mod ffi;

// Engine模块
#[cfg(feature = "custom-engine")]
mod engine;

#[cfg(feature = "custom-engine")]
use crate::core::app::PokemonApp;
use crate::states::{loading::LoadingPlugin, menu::MenuPlugin};
// 注意：以下插件需要实现后再启用
// use crate::graphics::renderer::PokemonRendererPlugin;
// use crate::input::PokemonInputPlugin;
// use crate::audio::PokemonAudioPlugin;
// use crate::data::PokemonDataPlugin;
// use crate::player::PokemonPlayerPlugin;

#[cfg(feature = "pokemon-wip")]
use crate::pokemon::PokemonSystemPlugin;

#[cfg(feature = "battle-wip")]
use crate::battle::PokemonBattlePlugin;

// use crate::world::PokemonWorldPlugin;
// use crate::ui::PokemonUIPlugin;

#[cfg(feature = "network-wip")]
use crate::network::PokemonNetworkPlugin;

fn main() {
    tracing::info!("启动高性能Pokemon游戏引擎 v1.0.0");
    
    let mut app = App::new();
    
    // 基础Bevy插件
    app.add_plugins(DefaultPlugins.set(
        WindowPlugin {
            primary_window: Some(Window {
                title: "Pokemon Adventure - Rust Engine".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                resizable: true,
                ..default()
            }),
            ..default()
        }
    ).set(
        AssetPlugin {
            mode: bevy::asset::AssetMode::Processed,
            ..default()
        }
    ).set(
        LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "pokemongo=trace,wgpu_core=warn,wgpu_hal=warn,naga=info".into(),
            ..default()
        }
    ));

    // 游戏状态管理
    app.init_state::<states::GameState>()
        .enable_state_scoped_entities::<states::GameState>();

    // 核心游戏插件
    #[cfg(feature = "custom-engine")]
    app.add_plugins(PokemonApp);
    
    app.add_plugins(LoadingPlugin)
        .add_plugins(MenuPlugin);
        // 注意：以下插件需要实现后再启用
        // .add_plugins(PokemonRendererPlugin)
        // .add_plugins(PokemonInputPlugin)
        // .add_plugins(PokemonAudioPlugin)
        // .add_plugins(PokemonDataPlugin)
        // .add_plugins(PokemonPlayerPlugin)
        // .add_plugins(PokemonWorldPlugin)
        // .add_plugins(PokemonUIPlugin);

    // 条件编译的模块插件
    #[cfg(feature = "pokemon-wip")]
    app.add_plugins(PokemonSystemPlugin);

    #[cfg(feature = "battle-wip")]
    app.add_plugins(PokemonBattlePlugin);

    #[cfg(feature = "network-wip")]
    app.add_plugins(PokemonNetworkPlugin);

    // 系统配置
    app.add_systems(Startup, setup_game)
        .add_systems(Update, (
            handle_exit_conditions,
            performance_monitoring,
        ));

    // 启动游戏
    tracing::info!("游戏引擎初始化完成，开始运行");
    app.run();
}

fn setup_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<states::GameState>>,
) {
    tracing::info!("设置游戏初始状态");
    
    // 添加游戏配置资源
    commands.insert_resource(core::config::GameConfig::default());
    commands.insert_resource(core::time::GameTimer::default());
    
    // 初始化ECS世界
    commands.insert_resource(crate::ecs::ECSWorld::new());
    
    // 启动加载状态
    next_state.set(states::GameState::Loading);
    
    tracing::info!("游戏初始状态设置完成");
}

fn handle_exit_conditions(
    input: Res<ButtonInput<KeyCode>>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    if input.just_pressed(KeyCode::Escape) && input.pressed(KeyCode::AltLeft) {
        tracing::info!("用户请求退出游戏");
        app_exit.send(bevy::app::AppExit::Success);
    }
}

fn performance_monitoring(
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_seconds();
    
    if *timer >= 60.0 {
        let fps = 1.0 / time.delta_seconds();
        tracing::debug!("性能监控 - FPS: {:.1}, 帧时间: {:.2}ms", 
            fps, time.delta_seconds() * 1000.0);
        *timer = 0.0;
    }
}

// 导出公共API
pub use crate::core::*;
pub use crate::utils::*;

#[cfg(feature = "pokemon-wip")]
pub use crate::pokemon::*;

#[cfg(feature = "battle-wip")]
pub use crate::battle::*;

pub use crate::world::*;
pub use crate::player::*;

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(app.world().entities().len() == 0);
    }

    #[test]
    fn test_game_state_initialization() {
        let mut app = App::new();
        app.init_state::<states::GameState>();
        
        let current_state = app.world().resource::<State<states::GameState>>();
        assert_eq!(**current_state, states::GameState::Loading);
    }

    #[test]
    fn test_performance_monitoring_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, performance_monitoring);
        
        app.update();
    }
}