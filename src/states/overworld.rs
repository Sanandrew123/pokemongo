// 大地图状态
// 开发心理：大地图是玩家主要活动区域，需要流畅移动、丰富交互、地图系统
// 设计原则：开放探索、NPC交互、Pokemon遭遇、地图切换

use log::{debug, warn, error};
use crate::core::error::GameError;
use crate::graphics::renderer::Renderer2D;
use crate::graphics::ui::UIManager;
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
use super::{GameState, GameStateType, StateTransition};
use glam::{Vec2, Vec4};

// 大地图状态
pub struct OverworldState {
    name: String,
    ui_manager: UIManager,
    
    // 玩家状态
    player_position: Vec2,
    player_direction: Vec2,
    player_speed: f32,
    
    // 相机
    camera_position: Vec2,
    camera_target: Vec2,
    camera_smooth: f32,
    
    // 地图数据
    current_map: String,
    map_size: Vec2,
    
    // 移动状态
    movement_keys: [bool; 4], // [up, down, left, right]
}

impl OverworldState {
    pub fn new() -> Self {
        Self {
            name: "OverworldState".to_string(),
            ui_manager: UIManager::new(Vec2::new(800.0, 600.0)),
            player_position: Vec2::new(400.0, 300.0),
            player_direction: Vec2::new(0.0, -1.0),
            player_speed: 200.0,
            camera_position: Vec2::new(400.0, 300.0),
            camera_target: Vec2::new(400.0, 300.0),
            camera_smooth: 5.0,
            current_map: "starting_town".to_string(),
            map_size: Vec2::new(2000.0, 1500.0),
            movement_keys: [false; 4],
        }
    }
    
    // 更新玩家移动
    fn update_movement(&mut self, delta_time: f32) {
        let mut movement = Vec2::ZERO;
        
        if self.movement_keys[0] { movement.y -= 1.0; } // Up
        if self.movement_keys[1] { movement.y += 1.0; } // Down  
        if self.movement_keys[2] { movement.x -= 1.0; } // Left
        if self.movement_keys[3] { movement.x += 1.0; } // Right
        
        if movement.length() > 0.0 {
            movement = movement.normalize();
            self.player_direction = movement;
            
            let new_position = self.player_position + movement * self.player_speed * delta_time;
            
            // 边界检查
            self.player_position = Vec2::new(
                new_position.x.clamp(0.0, self.map_size.x),
                new_position.y.clamp(0.0, self.map_size.y),
            );
        }
    }
    
    // 更新相机
    fn update_camera(&mut self, delta_time: f32) {
        self.camera_target = self.player_position;
        self.camera_position = self.camera_position.lerp(
            self.camera_target, 
            self.camera_smooth * delta_time
        );
    }
}

impl GameState for OverworldState {
    fn get_type(&self) -> GameStateType {
        GameStateType::Overworld
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("进入大地图状态");
        Ok(())
    }
    
    fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("退出大地图状态");
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), GameError> {
        debug!("暂停大地图状态");
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), GameError> {
        debug!("恢复大地图状态");
        Ok(())
    }
    
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError> {
        self.update_movement(delta_time);
        self.update_camera(delta_time);
        
        // 检查随机遭遇
        if fastrand::f32() < 0.001 { // 0.1% 每帧
            return Ok(StateTransition::Push(GameStateType::Battle));
        }
        
        Ok(StateTransition::None)
    }
    
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 设置相机
        let camera_offset = self.camera_position - Vec2::new(400.0, 300.0);
        
        // 渲染地图背景
        renderer.draw_quad(
            Vec2::ZERO - camera_offset,
            self.map_size,
            1,
            Vec4::new(0.2, 0.8, 0.3, 1.0),
            0.0,
        )?;
        
        // 渲染玩家
        renderer.draw_quad(
            self.player_position - camera_offset,
            Vec2::new(32.0, 32.0),
            1,
            Vec4::new(0.8, 0.2, 0.2, 1.0),
            0.0,
        )?;
        
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, _event: &MouseEvent) -> Result<bool, GameError> {
        Ok(false)
    }
    
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        match key {
            "w" | "W" | "ArrowUp" => {
                self.movement_keys[0] = pressed;
                Ok(true)
            },
            "s" | "S" | "ArrowDown" => {
                self.movement_keys[1] = pressed;
                Ok(true)
            },
            "a" | "A" | "ArrowLeft" => {
                self.movement_keys[2] = pressed;
                Ok(true)
            },
            "d" | "D" | "ArrowRight" => {
                self.movement_keys[3] = pressed;
                Ok(true)
            },
            "Escape" => {
                if pressed {
                    return Ok(false); // 让上层处理菜单
                }
                Ok(true)
            },
            _ => Ok(false),
        }
    }
    
    fn handle_gamepad_event(&mut self, _event: &GamepadEvent) -> Result<bool, GameError> {
        Ok(false)
    }
    
    fn is_transparent(&self) -> bool {
        false
    }
    
    fn blocks_input(&self) -> bool {
        true
    }
}