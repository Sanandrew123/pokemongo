// 加载状态
// 开发心理：加载状态需要展示进度、异步加载、用户友好的反馈
// 设计原则：异步处理、进度展示、错误处理、流畅过渡

use log::{debug, warn, error};
use crate::core::error::GameError;
#[cfg(feature = "graphics-wip")]
use crate::graphics::Renderer2D;
#[cfg(feature = "graphics-wip")]
use crate::graphics::ui::{UIManager, ElementType, UIEvent};

#[cfg(not(feature = "graphics-wip"))]
use crate::states::Renderer2D;
#[cfg(not(feature = "graphics-wip"))]
use crate::ui::{UIManager, ElementType, UIEvent};
use crate::input::mouse::MouseEvent;
use crate::input::gamepad::GamepadEvent;
use super::{StateHandler, GameStateType, StateTransition};
use glam::{Vec2, Vec4};
use std::collections::HashMap;

// 加载任务
#[derive(Debug, Clone)]
pub struct LoadingTask {
    pub name: String,
    pub description: String,
    pub progress: f32,      // 0.0 - 1.0
    pub completed: bool,
    pub error: Option<String>,
}

// 加载状态
pub struct LoadingState {
    name: String,
    ui_manager: UIManager,
    
    // 加载任务
    tasks: Vec<LoadingTask>,
    current_task_index: usize,
    total_progress: f32,
    
    // UI元素ID
    progress_bar_id: Option<u32>,
    status_label_id: Option<u32>,
    title_label_id: Option<u32>,
    
    // 状态
    loading_complete: bool,
    next_state: GameStateType,
    
    // 计时
    load_start_time: std::time::Instant,
    min_loading_time: f32,      // 最小加载时间
    
    // 动画
    spinner_rotation: f32,
    dots_animation: f32,
    fade_alpha: f32,
    
    // 配置
    show_detailed_progress: bool,
    background_color: Vec4,
}

impl LoadingState {
    pub fn new() -> Self {
        Self {
            name: "LoadingState".to_string(),
            ui_manager: UIManager::new(Vec2::new(800.0, 600.0)),
            tasks: Vec::new(),
            current_task_index: 0,
            total_progress: 0.0,
            progress_bar_id: None,
            status_label_id: None,
            title_label_id: None,
            loading_complete: false,
            next_state: GameStateType::MainMenu,
            load_start_time: std::time::Instant::now(),
            min_loading_time: 2.0,
            spinner_rotation: 0.0,
            dots_animation: 0.0,
            fade_alpha: 0.0,
            show_detailed_progress: true,
            background_color: Vec4::new(0.1, 0.1, 0.15, 1.0),
        }
    }
    
    // 设置加载任务
    pub fn set_loading_tasks(&mut self, tasks: Vec<LoadingTask>) {
        self.tasks = tasks;
        self.current_task_index = 0;
        self.total_progress = 0.0;
        self.loading_complete = false;
        debug!("设置加载任务: {} 个", self.tasks.len());
    }
    
    // 设置下一个状态
    pub fn set_next_state(&mut self, state: GameStateType) {
        self.next_state = state;
        debug!("设置下一状态: {:?}", state);
    }
    
    // 创建默认加载任务
    fn create_default_tasks(&mut self) {
        let tasks = vec![
            LoadingTask {
                name: "textures".to_string(),
                description: "加载纹理资源...".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
            LoadingTask {
                name: "audio".to_string(),
                description: "加载音频资源...".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
            LoadingTask {
                name: "pokemon_data".to_string(),
                description: "加载Pokemon数据...".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
            LoadingTask {
                name: "game_data".to_string(),
                description: "初始化游戏数据...".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
            LoadingTask {
                name: "save_data".to_string(),
                description: "检查存档数据...".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
        ];
        
        self.set_loading_tasks(tasks);
    }
    
    // 初始化UI
    fn setup_ui(&mut self) -> Result<(), GameError> {
        // 标题
        self.title_label_id = Some(self.ui_manager.create_element(
            "title_label".to_string(),
            ElementType::Label,
            None,
        )?);
        
        if let Some(title_id) = self.title_label_id {
            self.ui_manager.set_element_text(title_id, "Pokemon GO - 加载中...".to_string())?;
            self.ui_manager.set_element_position(title_id, Vec2::new(400.0, 150.0))?;
            self.ui_manager.set_element_size(title_id, Vec2::new(400.0, 60.0))?;
        }
        
        // 进度条容器
        let progress_container = self.ui_manager.create_element(
            "progress_container".to_string(),
            ElementType::Panel,
            None,
        )?;
        self.ui_manager.set_element_position(progress_container, Vec2::new(200.0, 400.0))?;
        self.ui_manager.set_element_size(progress_container, Vec2::new(400.0, 40.0))?;
        
        // 进度条
        self.progress_bar_id = Some(self.ui_manager.create_element(
            "progress_bar".to_string(),
            ElementType::ProgressBar,
            Some(progress_container),
        )?);
        
        if let Some(progress_id) = self.progress_bar_id {
            self.ui_manager.set_element_position(progress_id, Vec2::new(0.0, 0.0))?;
            self.ui_manager.set_element_size(progress_id, Vec2::new(400.0, 40.0))?;
        }
        
        // 状态标签
        self.status_label_id = Some(self.ui_manager.create_element(
            "status_label".to_string(),
            ElementType::Label,
            None,
        )?);
        
        if let Some(status_id) = self.status_label_id {
            self.ui_manager.set_element_text(status_id, "初始化...".to_string())?;
            self.ui_manager.set_element_position(status_id, Vec2::new(400.0, 480.0))?;
            self.ui_manager.set_element_size(status_id, Vec2::new(400.0, 30.0))?;
        }
        
        debug!("加载状态UI初始化完成");
        Ok(())
    }
    
    // 更新进度
    fn update_progress(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        
        // 计算总进度
        let completed_tasks = self.tasks.iter().filter(|t| t.completed).count() as f32;
        let current_task_progress = if self.current_task_index < self.tasks.len() {
            self.tasks[self.current_task_index].progress
        } else {
            1.0
        };
        
        self.total_progress = (completed_tasks + current_task_progress) / self.tasks.len() as f32;
        self.total_progress = self.total_progress.clamp(0.0, 1.0);
        
        // 更新UI
        if let Some(progress_id) = self.progress_bar_id {
            let progress_text = format!("{:.0}%", self.total_progress * 100.0);
            self.ui_manager.set_element_value(progress_id, progress_text).ok();
        }
        
        if let Some(status_id) = self.status_label_id {
            if self.current_task_index < self.tasks.len() {
                let current_task = &self.tasks[self.current_task_index];
                let status_text = if self.show_detailed_progress {
                    format!("{} ({:.0}%)", current_task.description, current_task.progress * 100.0)
                } else {
                    current_task.description.clone()
                };
                self.ui_manager.set_element_text(status_id, status_text).ok();
            } else {
                self.ui_manager.set_element_text(status_id, "加载完成!".to_string()).ok();
            }
        }
    }
    
    // 模拟加载任务
    fn simulate_loading(&mut self, delta_time: f32) {
        if self.current_task_index >= self.tasks.len() {
            if !self.loading_complete {
                self.loading_complete = true;
                debug!("所有加载任务完成");
            }
            return;
        }
        
        let current_task = &mut self.tasks[self.current_task_index];
        if current_task.completed {
            self.current_task_index += 1;
            return;
        }
        
        // 模拟加载进度
        let load_speed = match current_task.name.as_str() {
            "textures" => 0.3,      // 较慢
            "audio" => 0.5,         // 中等
            "pokemon_data" => 0.4,  // 较慢
            "game_data" => 0.8,     // 较快
            "save_data" => 1.2,     // 最快
            _ => 0.5,
        };
        
        current_task.progress += delta_time * load_speed;
        current_task.progress = current_task.progress.clamp(0.0, 1.0);
        
        if current_task.progress >= 1.0 {
            current_task.completed = true;
            debug!("完成加载任务: {}", current_task.name);
            
            // 模拟偶尔的错误
            if current_task.name == "save_data" && fastrand::f32() < 0.1 {
                current_task.error = Some("存档文件损坏".to_string());
                warn!("加载任务出错: {} - {:?}", current_task.name, current_task.error);
            }
        }
    }
    
    // 更新动画
    fn update_animations(&mut self, delta_time: f32) {
        // 旋转动画
        self.spinner_rotation += delta_time * 180.0; // 每秒180度
        if self.spinner_rotation >= 360.0 {
            self.spinner_rotation -= 360.0;
        }
        
        // 点点动画
        self.dots_animation += delta_time * 2.0;
        if self.dots_animation >= 3.0 {
            self.dots_animation = 0.0;
        }
        
        // 淡入动画
        if self.fade_alpha < 1.0 {
            self.fade_alpha += delta_time * 2.0;
            self.fade_alpha = self.fade_alpha.min(1.0);
        }
    }
    
    // 检查是否可以切换到下一状态
    fn should_transition(&self) -> bool {
        if !self.loading_complete {
            return false;
        }
        
        // 确保最小加载时间
        let elapsed = self.load_start_time.elapsed().as_secs_f32();
        if elapsed < self.min_loading_time {
            return false;
        }
        
        // 检查是否有错误
        let has_errors = self.tasks.iter().any(|t| t.error.is_some());
        if has_errors {
            warn!("加载过程中发现错误，但继续进行");
        }
        
        true
    }
}

impl StateHandler for LoadingState {
    fn get_type(&self) -> GameStateType {
        GameStateType::Loading
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn enter(&mut self, _previous_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("进入加载状态");
        
        // 重置状态
        self.load_start_time = std::time::Instant::now();
        self.loading_complete = false;
        self.current_task_index = 0;
        self.total_progress = 0.0;
        self.fade_alpha = 0.0;
        
        // 创建加载任务
        if self.tasks.is_empty() {
            self.create_default_tasks();
        }
        
        // 初始化UI
        self.setup_ui()?;
        
        Ok(())
    }
    
    fn exit(&mut self, _next_state: Option<GameStateType>) -> Result<(), GameError> {
        debug!("退出加载状态");
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), GameError> {
        debug!("暂停加载状态");
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), GameError> {
        debug!("恢复加载状态");
        Ok(())
    }
    
    fn update(&mut self, delta_time: f32) -> Result<StateTransition, GameError> {
        // 模拟加载过程
        self.simulate_loading(delta_time);
        
        // 更新进度显示
        self.update_progress();
        
        // 更新动画
        self.update_animations(delta_time);
        
        // 更新UI
        self.ui_manager.update(delta_time)?;
        
        // 检查是否完成
        if self.should_transition() {
            return Ok(StateTransition::Replace(self.next_state));
        }
        
        Ok(StateTransition::None)
    }
    
    fn render(&mut self, renderer: &mut Renderer2D) -> Result<(), GameError> {
        // 清屏
        renderer.clear(self.background_color * self.fade_alpha)?;
        
        // 渲染UI
        self.ui_manager.render(renderer)?;
        
        // 渲染旋转指示器 (简化实现)
        let spinner_pos = Vec2::new(100.0, 400.0);
        let spinner_size = Vec2::new(20.0, 20.0);
        renderer.draw_quad(
            spinner_pos,
            spinner_size,
            1,
            Vec4::new(1.0, 1.0, 1.0, self.fade_alpha),
            self.spinner_rotation.to_radians(),
        )?;
        
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<bool, GameError> {
        // 加载状态通常不响应鼠标事件，但可以提供取消选项
        Ok(false)
    }
    
    fn handle_keyboard_event(&mut self, key: &str, pressed: bool) -> Result<bool, GameError> {
        if pressed {
            match key {
                "Escape" => {
                    // 可以提供跳过加载的选项（仅调试模式）
                    #[cfg(debug_assertions)]
                    {
                        warn!("调试模式: 跳过加载");
                        self.loading_complete = true;
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }
        
        Ok(false)
    }
    
    fn handle_gamepad_event(&mut self, _event: &GamepadEvent) -> Result<bool, GameError> {
        Ok(false)
    }
    
    fn load_resources(&mut self) -> Result<(), GameError> {
        debug!("加载状态资源");
        Ok(())
    }
    
    fn unload_resources(&mut self) -> Result<(), GameError> {
        debug!("卸载状态资源");
        Ok(())
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
    
    #[test]
    fn test_loading_state_creation() {
        let state = LoadingState::new();
        assert_eq!(state.get_type(), GameStateType::Loading);
        assert_eq!(state.get_name(), "LoadingState");
        assert_eq!(state.total_progress, 0.0);
        assert!(!state.loading_complete);
    }
    
    #[test]
    fn test_loading_tasks() {
        let mut state = LoadingState::new();
        
        let tasks = vec![
            LoadingTask {
                name: "test_task".to_string(),
                description: "测试任务".to_string(),
                progress: 0.0,
                completed: false,
                error: None,
            },
        ];
        
        state.set_loading_tasks(tasks);
        assert_eq!(state.tasks.len(), 1);
        assert_eq!(state.current_task_index, 0);
    }
    
    #[test]
    fn test_progress_calculation() {
        let mut state = LoadingState::new();
        
        // 设置测试任务
        let mut tasks = vec![
            LoadingTask {
                name: "task1".to_string(),
                description: "任务1".to_string(),
                progress: 1.0,
                completed: true,
                error: None,
            },
            LoadingTask {
                name: "task2".to_string(),
                description: "任务2".to_string(),
                progress: 0.5,
                completed: false,
                error: None,
            },
        ];
        
        state.set_loading_tasks(tasks);
        state.current_task_index = 1;
        state.update_progress();
        
        // 总进度应该是 (1.0 + 0.5) / 2 = 0.75
        assert!((state.total_progress - 0.75).abs() < 0.01);
    }
}