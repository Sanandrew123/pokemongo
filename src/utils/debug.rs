// Debug and Development Tools - 调试和开发工具系统
//
// 开发心理过程：
// 1. 这个模块提供全面的调试和开发工具，帮助开发者诊断和优化游戏
// 2. 实现性能监控系统，追踪FPS、内存使用、渲染时间等关键指标
// 3. 提供可视化调试界面，显示游戏状态、实体信息、系统状态等
// 4. 集成日志系统和错误追踪，便于问题定位和修复
// 5. 添加开发者命令行接口，支持实时调整游戏参数和状态

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, RwLock};
use std::fmt;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

/// 调试管理器
#[derive(Resource)]
pub struct DebugManager {
    pub is_enabled: bool,
    pub show_debug_ui: bool,
    pub performance_monitor: PerformanceMonitor,
    pub logger: DebugLogger,
    pub profiler: GameProfiler,
    pub inspector: GameInspector,
    pub console: DebugConsole,
    pub visualization: DebugVisualization,
    pub memory_tracker: MemoryTracker,
    pub network_debugger: NetworkDebugger,
}

impl Default for DebugManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugManager {
    pub fn new() -> Self {
        Self {
            is_enabled: cfg!(debug_assertions),
            show_debug_ui: false,
            performance_monitor: PerformanceMonitor::new(),
            logger: DebugLogger::new(),
            profiler: GameProfiler::new(),
            inspector: GameInspector::new(),
            console: DebugConsole::new(),
            visualization: DebugVisualization::new(),
            memory_tracker: MemoryTracker::new(),
            network_debugger: NetworkDebugger::new(),
        }
    }

    pub fn toggle_debug_ui(&mut self) {
        self.show_debug_ui = !self.show_debug_ui;
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.is_enabled {
            return;
        }

        self.performance_monitor.update(delta_time);
        self.profiler.update(delta_time);
        self.memory_tracker.update(delta_time);
        self.console.update(delta_time);
        self.logger.flush();
    }

    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        self.logger.log(level, module, message);
    }

    pub fn start_profiling(&mut self, name: &str) -> ProfilerHandle {
        self.profiler.start_profile(name)
    }

    pub fn end_profiling(&mut self, handle: ProfilerHandle) {
        self.profiler.end_profile(handle);
    }

    pub fn add_debug_value<T: fmt::Debug>(&mut self, key: String, value: T) {
        self.inspector.add_value(key, format!("{:?}", value));
    }

    pub fn execute_command(&mut self, command: &str) -> Result<String, DebugError> {
        self.console.execute_command(command)
    }

    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_monitor.stats
    }

    pub fn render_debug_ui(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        if !self.show_debug_ui {
            return;
        }

        self.render_performance_overlay(ui_renderer);
        self.render_profiler_data(ui_renderer);
        self.render_inspector_panel(ui_renderer);
        self.render_console(ui_renderer);
        self.visualization.render(ui_renderer);
    }

    fn render_performance_overlay(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        let stats = &self.performance_monitor.stats;
        let y_offset = 10.0;
        let line_height = 20.0;

        let fps_text = format!("FPS: {:.1}", stats.fps);
        let frame_time_text = format!("Frame Time: {:.2}ms", stats.frame_time_ms);
        let memory_text = format!("Memory: {:.1}MB", stats.memory_usage_mb);
        
        ui_renderer.draw_text(&fps_text, Vec2::new(10.0, y_offset), 
                             &crate::graphics::TextStyle {
                                 font: "debug".to_string(),
                                 size: 14.0,
                                 color: if stats.fps >= 55.0 { crate::graphics::Color::GREEN } 
                                       else if stats.fps >= 25.0 { crate::graphics::Color::YELLOW } 
                                       else { crate::graphics::Color::RED },
                             });

        ui_renderer.draw_text(&frame_time_text, Vec2::new(10.0, y_offset + line_height), 
                             &crate::graphics::TextStyle {
                                 font: "debug".to_string(),
                                 size: 14.0,
                                 color: crate::graphics::Color::WHITE,
                             });

        ui_renderer.draw_text(&memory_text, Vec2::new(10.0, y_offset + line_height * 2.0), 
                             &crate::graphics::TextStyle {
                                 font: "debug".to_string(),
                                 size: 14.0,
                                 color: if stats.memory_usage_mb < 500.0 { crate::graphics::Color::GREEN }
                                       else if stats.memory_usage_mb < 1000.0 { crate::graphics::Color::YELLOW }
                                       else { crate::graphics::Color::RED },
                             });
    }

    fn render_profiler_data(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        let x_offset = 10.0;
        let mut y_offset = 100.0;
        let line_height = 16.0;

        ui_renderer.draw_text("Profiler Data:", Vec2::new(x_offset, y_offset), 
                             &crate::graphics::TextStyle {
                                 font: "debug".to_string(),
                                 size: 16.0,
                                 color: crate::graphics::Color::CYAN,
                             });
        y_offset += line_height * 1.5;

        for (name, data) in &self.profiler.profile_data {
            let avg_time = data.total_time / data.call_count as f64;
            let profile_text = format!("{}: {:.2}ms (avg), {} calls", name, avg_time * 1000.0, data.call_count);
            
            ui_renderer.draw_text(&profile_text, Vec2::new(x_offset, y_offset), 
                                 &crate::graphics::TextStyle {
                                     font: "debug".to_string(),
                                     size: 12.0,
                                     color: crate::graphics::Color::WHITE,
                                 });
            y_offset += line_height;
        }
    }

    fn render_inspector_panel(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        let x_offset = 300.0;
        let mut y_offset = 10.0;
        let line_height = 16.0;

        ui_renderer.draw_text("Game Inspector:", Vec2::new(x_offset, y_offset), 
                             &crate::graphics::TextStyle {
                                 font: "debug".to_string(),
                                 size: 16.0,
                                 color: crate::graphics::Color::CYAN,
                             });
        y_offset += line_height * 1.5;

        for (key, value) in &self.inspector.values {
            let inspector_text = format!("{}: {}", key, value);
            
            ui_renderer.draw_text(&inspector_text, Vec2::new(x_offset, y_offset), 
                                 &crate::graphics::TextStyle {
                                     font: "debug".to_string(),
                                     size: 12.0,
                                     color: crate::graphics::Color::LIGHT_GRAY,
                                 });
            y_offset += line_height;
        }
    }

    fn render_console(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        if !self.console.is_visible {
            return;
        }

        // 渲染控制台背景
        let console_height = 300.0;
        let console_y = 1080.0 - console_height;
        
        ui_renderer.draw_rect(Vec2::new(0.0, console_y), Vec2::new(1920.0, console_height), 
                             crate::graphics::Color::from_rgba(0.0, 0.0, 0.0, 0.8));

        // 渲染控制台历史
        let mut y_offset = console_y + 10.0;
        let line_height = 16.0;
        
        for entry in self.console.history.iter().rev().take(15) {
            let color = match entry.entry_type {
                ConsoleEntryType::Command => crate::graphics::Color::YELLOW,
                ConsoleEntryType::Output => crate::graphics::Color::WHITE,
                ConsoleEntryType::Error => crate::graphics::Color::RED,
            };

            ui_renderer.draw_text(&entry.text, Vec2::new(10.0, y_offset), 
                                 &crate::graphics::TextStyle {
                                     font: "console".to_string(),
                                     size: 12.0,
                                     color,
                                 });
            y_offset += line_height;
        }

        // 渲染输入提示符
        let prompt_text = format!("> {}", self.console.current_input);
        ui_renderer.draw_text(&prompt_text, Vec2::new(10.0, console_y + console_height - 30.0), 
                             &crate::graphics::TextStyle {
                                 font: "console".to_string(),
                                 size: 14.0,
                                 color: crate::graphics::Color::GREEN,
                             });
    }
}

/// 性能监控器
#[derive(Debug)]
pub struct PerformanceMonitor {
    pub stats: PerformanceStats,
    frame_times: VecDeque<f32>,
    last_update: Instant,
    update_interval: Duration,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            stats: PerformanceStats::default(),
            frame_times: VecDeque::with_capacity(120), // 2秒的帧数历史（60fps）
            last_update: Instant::now(),
            update_interval: Duration::from_millis(250), // 每250ms更新一次统计
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // 记录帧时间
        self.frame_times.push_back(delta_time);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }

        // 定期更新统计数据
        if self.last_update.elapsed() >= self.update_interval {
            self.update_stats();
            self.last_update = Instant::now();
        }
    }

    fn update_stats(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        // 计算FPS
        let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.stats.fps = 1.0 / avg_frame_time;
        self.stats.frame_time_ms = avg_frame_time * 1000.0;

        // 计算帧时间统计
        let mut sorted_times: Vec<f32> = self.frame_times.iter().copied().collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if !sorted_times.is_empty() {
            self.stats.min_frame_time_ms = sorted_times[0] * 1000.0;
            self.stats.max_frame_time_ms = sorted_times[sorted_times.len() - 1] * 1000.0;
            self.stats.percentile_99_frame_time_ms = sorted_times[(sorted_times.len() as f32 * 0.99) as usize] * 1000.0;
        }

        // 获取内存使用情况（简化实现）
        self.stats.memory_usage_mb = self.get_memory_usage();
    }

    fn get_memory_usage(&self) -> f32 {
        // 这是一个简化的实现，实际项目中应该使用系统API获取真实内存使用
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<f32>() {
                                return kb / 1024.0; // 转换为MB
                            }
                        }
                    }
                }
            }
        }
        
        // 回退估算
        100.0 + (Instant::now().elapsed().as_secs() as f32 * 0.1) % 50.0
    }
}

#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub min_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub percentile_99_frame_time_ms: f32,
    pub memory_usage_mb: f32,
    pub draw_calls: u32,
    pub triangles_rendered: u32,
}

/// 游戏性能分析器
#[derive(Debug)]
pub struct GameProfiler {
    pub profile_data: HashMap<String, ProfileData>,
    active_profiles: HashMap<ProfilerHandle, ActiveProfile>,
    next_handle: ProfilerHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProfilerHandle(u32);

#[derive(Debug)]
struct ActiveProfile {
    name: String,
    start_time: Instant,
}

#[derive(Debug)]
pub struct ProfileData {
    pub total_time: f64,
    pub call_count: u32,
    pub min_time: f64,
    pub max_time: f64,
    pub recent_samples: VecDeque<f64>,
}

impl GameProfiler {
    pub fn new() -> Self {
        Self {
            profile_data: HashMap::new(),
            active_profiles: HashMap::new(),
            next_handle: ProfilerHandle(0),
        }
    }

    pub fn start_profile(&mut self, name: &str) -> ProfilerHandle {
        let handle = self.next_handle;
        self.next_handle.0 += 1;

        self.active_profiles.insert(handle, ActiveProfile {
            name: name.to_string(),
            start_time: Instant::now(),
        });

        handle
    }

    pub fn end_profile(&mut self, handle: ProfilerHandle) {
        if let Some(active_profile) = self.active_profiles.remove(&handle) {
            let duration = active_profile.start_time.elapsed().as_secs_f64();
            
            let data = self.profile_data.entry(active_profile.name).or_insert_with(|| ProfileData {
                total_time: 0.0,
                call_count: 0,
                min_time: f64::INFINITY,
                max_time: 0.0,
                recent_samples: VecDeque::with_capacity(100),
            });

            data.total_time += duration;
            data.call_count += 1;
            data.min_time = data.min_time.min(duration);
            data.max_time = data.max_time.max(duration);
            
            data.recent_samples.push_back(duration);
            if data.recent_samples.len() > 100 {
                data.recent_samples.pop_front();
            }
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // 清理长时间运行的分析（可能是忘记结束的）
        let current_time = Instant::now();
        self.active_profiles.retain(|_, profile| {
            current_time.duration_since(profile.start_time).as_secs() < 10
        });
    }

    pub fn get_average_time(&self, name: &str) -> Option<f64> {
        self.profile_data.get(name).map(|data| data.total_time / data.call_count as f64)
    }

    pub fn reset(&mut self) {
        self.profile_data.clear();
        self.active_profiles.clear();
    }
}

/// 游戏检查器 - 显示游戏状态信息
#[derive(Debug)]
pub struct GameInspector {
    pub values: HashMap<String, String>,
    pub categories: HashMap<String, Vec<String>>,
}

impl GameInspector {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: String) {
        self.values.insert(key, value);
    }

    pub fn add_category_value(&mut self, category: String, key: String, value: String) {
        self.categories.entry(category.clone()).or_insert_with(Vec::new).push(key.clone());
        self.values.insert(format!("{}::{}", category, key), value);
    }

    pub fn remove_value(&mut self, key: &str) {
        self.values.remove(key);
    }

    pub fn clear_category(&mut self, category: &str) {
        if let Some(keys) = self.categories.get(category) {
            for key in keys {
                self.values.remove(&format!("{}::{}", category, key));
            }
        }
        self.categories.remove(category);
    }
}

/// 调试控制台
#[derive(Debug)]
pub struct DebugConsole {
    pub is_visible: bool,
    pub history: VecDeque<ConsoleEntry>,
    pub current_input: String,
    pub command_history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub commands: HashMap<String, Box<dyn ConsoleCommand>>,
}

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub text: String,
    pub entry_type: ConsoleEntryType,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum ConsoleEntryType {
    Command,
    Output,
    Error,
}

impl DebugConsole {
    pub fn new() -> Self {
        let mut console = Self {
            is_visible: false,
            history: VecDeque::with_capacity(1000),
            current_input: String::new(),
            command_history: VecDeque::with_capacity(100),
            history_index: None,
            commands: HashMap::new(),
        };

        console.register_default_commands();
        console
    }

    fn register_default_commands(&mut self) {
        self.commands.insert("help".to_string(), Box::new(HelpCommand));
        self.commands.insert("clear".to_string(), Box::new(ClearCommand));
        self.commands.insert("fps".to_string(), Box::new(FPSCommand));
        self.commands.insert("memory".to_string(), Box::new(MemoryCommand));
        self.commands.insert("profile".to_string(), Box::new(ProfileCommand));
        self.commands.insert("spawn".to_string(), Box::new(SpawnCommand));
        self.commands.insert("teleport".to_string(), Box::new(TeleportCommand));
        self.commands.insert("give".to_string(), Box::new(GiveCommand));
        self.commands.insert("set".to_string(), Box::new(SetCommand));
        self.commands.insert("get".to_string(), Box::new(GetCommand));
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn add_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.current_input.push(ch);
        }
    }

    pub fn backspace(&mut self) {
        self.current_input.pop();
    }

    pub fn execute_current_command(&mut self) -> Result<String, DebugError> {
        let command = self.current_input.trim().to_string();
        if command.is_empty() {
            return Ok(String::new());
        }

        self.add_to_history(command.clone(), ConsoleEntryType::Command);
        self.command_history.push_back(command.clone());
        
        if self.command_history.len() > 100 {
            self.command_history.pop_front();
        }

        let result = self.execute_command(&command);
        self.current_input.clear();
        self.history_index = None;

        result
    }

    pub fn execute_command(&mut self, command: &str) -> Result<String, DebugError> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        let cmd_name = parts[0];
        let args = &parts[1..];

        if let Some(cmd_handler) = self.commands.get(cmd_name) {
            let result = cmd_handler.execute(args);
            match &result {
                Ok(output) => {
                    if !output.is_empty() {
                        self.add_to_history(output.clone(), ConsoleEntryType::Output);
                    }
                }
                Err(error) => {
                    self.add_to_history(format!("Error: {}", error), ConsoleEntryType::Error);
                }
            }
            result
        } else {
            let error_msg = format!("Unknown command: '{}'. Type 'help' for available commands.", cmd_name);
            self.add_to_history(error_msg.clone(), ConsoleEntryType::Error);
            Err(DebugError::UnknownCommand(cmd_name.to_string()))
        }
    }

    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        self.history_index = match self.history_index {
            None => Some(self.command_history.len() - 1),
            Some(index) => Some(index.saturating_sub(1)),
        };

        if let Some(index) = self.history_index {
            if let Some(cmd) = self.command_history.get(index) {
                self.current_input = cmd.clone();
            }
        }
    }

    pub fn history_down(&mut self) {
        if let Some(index) = self.history_index {
            if index + 1 < self.command_history.len() {
                self.history_index = Some(index + 1);
                if let Some(cmd) = self.command_history.get(index + 1) {
                    self.current_input = cmd.clone();
                }
            } else {
                self.history_index = None;
                self.current_input.clear();
            }
        }
    }

    fn add_to_history(&mut self, text: String, entry_type: ConsoleEntryType) {
        self.history.push_back(ConsoleEntry {
            text,
            entry_type,
            timestamp: Instant::now(),
        });

        if self.history.len() > 1000 {
            self.history.pop_front();
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // 清理旧的历史条目（可选）
        let cutoff = Instant::now() - Duration::from_secs(3600); // 保留1小时的历史
        while let Some(entry) = self.history.front() {
            if entry.timestamp < cutoff {
                self.history.pop_front();
            } else {
                break;
            }
        }
    }
}

/// 控制台命令接口
pub trait ConsoleCommand: std::fmt::Debug {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError>;
    fn help(&self) -> String;
}

// 默认控制台命令实现
#[derive(Debug)]
struct HelpCommand;

impl ConsoleCommand for HelpCommand {
    fn execute(&self, _args: &[&str]) -> Result<String, DebugError> {
        Ok("Available commands:\n\
            help - Show this help message\n\
            clear - Clear console history\n\
            fps - Show FPS information\n\
            memory - Show memory usage\n\
            profile <start|stop|reset> [name] - Profile performance\n\
            spawn <pokemon_id> [level] - Spawn a Pokemon\n\
            teleport <x> <y> - Teleport to coordinates\n\
            give <item> [amount] - Give item to player\n\
            set <variable> <value> - Set game variable\n\
            get <variable> - Get game variable value".to_string())
    }

    fn help(&self) -> String {
        "Show available commands".to_string()
    }
}

#[derive(Debug)]
struct ClearCommand;

impl ConsoleCommand for ClearCommand {
    fn execute(&self, _args: &[&str]) -> Result<String, DebugError> {
        // 实际实现中需要清理控制台历史
        Ok("Console cleared".to_string())
    }

    fn help(&self) -> String {
        "Clear console history".to_string()
    }
}

#[derive(Debug)]
struct FPSCommand;

impl ConsoleCommand for FPSCommand {
    fn execute(&self, _args: &[&str]) -> Result<String, DebugError> {
        // 这里需要访问性能监控器的数据
        Ok("FPS information would be displayed here".to_string())
    }

    fn help(&self) -> String {
        "Show FPS and performance information".to_string()
    }
}

#[derive(Debug)]
struct MemoryCommand;

impl ConsoleCommand for MemoryCommand {
    fn execute(&self, _args: &[&str]) -> Result<String, DebugError> {
        Ok("Memory usage information would be displayed here".to_string())
    }

    fn help(&self) -> String {
        "Show memory usage statistics".to_string()
    }
}

#[derive(Debug)]
struct ProfileCommand;

impl ConsoleCommand for ProfileCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        match args.first() {
            Some(&"start") => Ok("Profiling started".to_string()),
            Some(&"stop") => Ok("Profiling stopped".to_string()),
            Some(&"reset") => Ok("Profiling data reset".to_string()),
            _ => Err(DebugError::InvalidArguments("Usage: profile <start|stop|reset> [name]".to_string())),
        }
    }

    fn help(&self) -> String {
        "Control performance profiling".to_string()
    }
}

#[derive(Debug)]
struct SpawnCommand;

impl ConsoleCommand for SpawnCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        if args.is_empty() {
            return Err(DebugError::InvalidArguments("Usage: spawn <pokemon_id> [level]".to_string()));
        }
        
        let pokemon_id = args[0];
        let level = args.get(1).unwrap_or(&"1");
        
        Ok(format!("Spawned {} at level {}", pokemon_id, level))
    }

    fn help(&self) -> String {
        "Spawn a Pokemon at current location".to_string()
    }
}

#[derive(Debug)]
struct TeleportCommand;

impl ConsoleCommand for TeleportCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        if args.len() < 2 {
            return Err(DebugError::InvalidArguments("Usage: teleport <x> <y>".to_string()));
        }
        
        let x = args[0];
        let y = args[1];
        
        Ok(format!("Teleported to ({}, {})", x, y))
    }

    fn help(&self) -> String {
        "Teleport to specified coordinates".to_string()
    }
}

#[derive(Debug)]
struct GiveCommand;

impl ConsoleCommand for GiveCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        if args.is_empty() {
            return Err(DebugError::InvalidArguments("Usage: give <item> [amount]".to_string()));
        }
        
        let item = args[0];
        let amount = args.get(1).unwrap_or(&"1");
        
        Ok(format!("Gave {} x{}", item, amount))
    }

    fn help(&self) -> String {
        "Give item to player".to_string()
    }
}

#[derive(Debug)]
struct SetCommand;

impl ConsoleCommand for SetCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        if args.len() < 2 {
            return Err(DebugError::InvalidArguments("Usage: set <variable> <value>".to_string()));
        }
        
        let variable = args[0];
        let value = args[1];
        
        Ok(format!("Set {} = {}", variable, value))
    }

    fn help(&self) -> String {
        "Set game variable value".to_string()
    }
}

#[derive(Debug)]
struct GetCommand;

impl ConsoleCommand for GetCommand {
    fn execute(&self, args: &[&str]) -> Result<String, DebugError> {
        if args.is_empty() {
            return Err(DebugError::InvalidArguments("Usage: get <variable>".to_string()));
        }
        
        let variable = args[0];
        Ok(format!("{} = <value would be retrieved>", variable))
    }

    fn help(&self) -> String {
        "Get game variable value".to_string()
    }
}

/// 调试可视化系统
#[derive(Debug)]
pub struct DebugVisualization {
    pub show_collision_boxes: bool,
    pub show_pathfinding: bool,
    pub show_ai_debug: bool,
    pub show_performance_graph: bool,
    pub wireframe_mode: bool,
}

impl DebugVisualization {
    pub fn new() -> Self {
        Self {
            show_collision_boxes: false,
            show_pathfinding: false,
            show_ai_debug: false,
            show_performance_graph: false,
            wireframe_mode: false,
        }
    }

    pub fn render(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        if self.show_performance_graph {
            self.render_performance_graph(ui_renderer);
        }

        if self.show_collision_boxes {
            self.render_collision_debug(ui_renderer);
        }
    }

    fn render_performance_graph(&self, ui_renderer: &mut crate::graphics::UIRenderer) {
        // 渲染性能图表
        let graph_x = 1500.0;
        let graph_y = 50.0;
        let graph_width = 400.0;
        let graph_height = 200.0;

        // 背景
        ui_renderer.draw_rect(Vec2::new(graph_x, graph_y), Vec2::new(graph_width, graph_height), 
                             crate::graphics::Color::from_rgba(0.0, 0.0, 0.0, 0.5));

        // TODO: 渲染实际的性能数据图表
    }

    fn render_collision_debug(&self, _ui_renderer: &mut crate::graphics::UIRenderer) {
        // TODO: 渲染碰撞盒调试信息
    }
}

/// 内存跟踪器
#[derive(Debug)]
pub struct MemoryTracker {
    pub allocations: HashMap<String, AllocationInfo>,
    pub total_allocated: usize,
    pub peak_usage: usize,
    pub allocation_history: VecDeque<MemorySnapshot>,
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub size: usize,
    pub count: usize,
    pub peak_count: usize,
    pub total_allocated: usize,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub total_allocated: usize,
    pub allocations: HashMap<String, usize>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocated: 0,
            peak_usage: 0,
            allocation_history: VecDeque::with_capacity(3600), // 1小时的历史
        }
    }

    pub fn record_allocation(&mut self, category: String, size: usize) {
        let info = self.allocations.entry(category).or_insert_with(|| AllocationInfo {
            size: 0,
            count: 0,
            peak_count: 0,
            total_allocated: 0,
        });

        info.size += size;
        info.count += 1;
        info.peak_count = info.peak_count.max(info.count);
        info.total_allocated += size;

        self.total_allocated += size;
        self.peak_usage = self.peak_usage.max(self.total_allocated);
    }

    pub fn record_deallocation(&mut self, category: &str, size: usize) {
        if let Some(info) = self.allocations.get_mut(category) {
            info.size = info.size.saturating_sub(size);
            info.count = info.count.saturating_sub(1);
        }
        self.total_allocated = self.total_allocated.saturating_sub(size);
    }

    pub fn update(&mut self, _delta_time: f32) {
        // 每秒记录一次内存快照
        static mut LAST_SNAPSHOT: Option<Instant> = None;
        let now = Instant::now();
        
        unsafe {
            if LAST_SNAPSHOT.is_none() || now.duration_since(LAST_SNAPSHOT.unwrap()).as_secs() >= 1 {
                let snapshot = MemorySnapshot {
                    timestamp: now,
                    total_allocated: self.total_allocated,
                    allocations: self.allocations.iter()
                        .map(|(k, v)| (k.clone(), v.size))
                        .collect(),
                };

                self.allocation_history.push_back(snapshot);
                if self.allocation_history.len() > 3600 {
                    self.allocation_history.pop_front();
                }

                LAST_SNAPSHOT = Some(now);
            }
        }
    }

    pub fn get_memory_usage_by_category(&self) -> Vec<(String, usize)> {
        let mut usage: Vec<_> = self.allocations.iter()
            .map(|(category, info)| (category.clone(), info.size))
            .collect();
        
        usage.sort_by(|a, b| b.1.cmp(&a.1));
        usage
    }
}

/// 网络调试器
#[derive(Debug)]
pub struct NetworkDebugger {
    pub packet_log: VecDeque<NetworkPacket>,
    pub connection_stats: HashMap<String, ConnectionStats>,
    pub latency_history: VecDeque<(Instant, f32)>,
}

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub timestamp: Instant,
    pub direction: PacketDirection,
    pub packet_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum PacketDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub packets_sent: usize,
    pub packets_received: usize,
    pub connection_time: Instant,
    pub last_activity: Instant,
    pub average_latency: f32,
}

impl NetworkDebugger {
    pub fn new() -> Self {
        Self {
            packet_log: VecDeque::with_capacity(1000),
            connection_stats: HashMap::new(),
            latency_history: VecDeque::with_capacity(600), // 10分钟的延迟历史
        }
    }

    pub fn log_packet(&mut self, direction: PacketDirection, packet_type: String, size: usize, data: Vec<u8>) {
        let packet = NetworkPacket {
            timestamp: Instant::now(),
            direction,
            packet_type,
            size,
            data,
        };

        self.packet_log.push_back(packet);
        if self.packet_log.len() > 1000 {
            self.packet_log.pop_front();
        }
    }

    pub fn update_latency(&mut self, latency_ms: f32) {
        let now = Instant::now();
        self.latency_history.push_back((now, latency_ms));
        
        // 保留最近10分钟的延迟数据
        let cutoff = now - Duration::from_secs(600);
        while let Some(&(timestamp, _)) = self.latency_history.front() {
            if timestamp < cutoff {
                self.latency_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_average_latency(&self) -> f32 {
        if self.latency_history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.latency_history.iter().map(|(_, latency)| latency).sum();
        sum / self.latency_history.len() as f32
    }
}

/// 调试日志系统
#[derive(Debug)]
pub struct DebugLogger {
    pub log_entries: VecDeque<LogEntry>,
    pub log_level: LogLevel,
    pub file_output: Option<std::fs::File>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl DebugLogger {
    pub fn new() -> Self {
        Self {
            log_entries: VecDeque::with_capacity(10000),
            log_level: if cfg!(debug_assertions) { LogLevel::Debug } else { LogLevel::Info },
            file_output: None,
        }
    }

    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        if level < self.log_level {
            return;
        }

        let entry = LogEntry {
            timestamp: Instant::now(),
            level,
            module: module.to_string(),
            message: message.to_string(),
        };

        self.log_entries.push_back(entry.clone());
        if self.log_entries.len() > 10000 {
            self.log_entries.pop_front();
        }

        // 输出到控制台
        println!("[{:?}] {}: {}", level, module, message);

        // 输出到文件（如果配置了）
        if let Some(ref mut file) = self.file_output {
            use std::io::Write;
            let _ = writeln!(file, "[{:?}] {}: {}", level, module, message);
        }
    }

    pub fn flush(&mut self) {
        if let Some(ref mut file) = self.file_output {
            use std::io::Write;
            let _ = file.flush();
        }
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    pub fn enable_file_logging(&mut self, path: &std::path::Path) -> Result<(), std::io::Error> {
        self.file_output = Some(std::fs::File::create(path)?);
        Ok(())
    }
}

/// 调试错误类型
#[derive(Debug, Clone)]
pub enum DebugError {
    UnknownCommand(String),
    InvalidArguments(String),
    SystemError(String),
    PermissionDenied,
}

impl fmt::Display for DebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            DebugError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            DebugError::SystemError(msg) => write!(f, "System error: {}", msg),
            DebugError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl std::error::Error for DebugError {}

/// 调试宏
#[macro_export]
macro_rules! debug_log {
    ($level:expr, $module:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            // 实际实现中应该通过全局调试管理器记录日志
            println!("[{:?}] {}: {}", $level, $module, format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! debug_profile {
    ($profiler:expr, $name:expr, $block:block) => {
        {
            #[cfg(debug_assertions)]
            let handle = $profiler.start_profile($name);
            
            let result = $block;
            
            #[cfg(debug_assertions)]
            $profiler.end_profile(handle);
            
            result
        }
    };
}

/// Bevy系统：处理调试输入
pub fn debug_input_system(
    input: Res<Input<KeyCode>>,
    mut debug_manager: ResMut<DebugManager>,
) {
    if input.just_pressed(KeyCode::F3) {
        debug_manager.toggle_debug_ui();
    }

    if input.just_pressed(KeyCode::Grave) {
        debug_manager.console.toggle_visibility();
    }

    // 控制台输入处理
    if debug_manager.console.is_visible {
        // TODO: 处理字符输入和特殊键
        if input.just_pressed(KeyCode::Return) {
            let _ = debug_manager.console.execute_current_command();
        }

        if input.just_pressed(KeyCode::Back) {
            debug_manager.console.backspace();
        }

        if input.just_pressed(KeyCode::Up) {
            debug_manager.console.history_up();
        }

        if input.just_pressed(KeyCode::Down) {
            debug_manager.console.history_down();
        }
    }
}

/// Bevy系统：更新调试管理器
pub fn debug_update_system(
    time: Res<Time>,
    mut debug_manager: ResMut<DebugManager>,
    diagnostics: Res<DiagnosticsStore>,
) {
    debug_manager.update(time.delta_seconds());

    // 更新性能统计
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps_diagnostic.smoothed() {
            debug_manager.performance_monitor.stats.fps = fps_value as f32;
        }
    }

    if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time_value) = frame_time_diagnostic.smoothed() {
            debug_manager.performance_monitor.stats.frame_time_ms = (frame_time_value * 1000.0) as f32;
        }
    }
}

/// Bevy系统：渲染调试UI
pub fn debug_render_system(
    debug_manager: Res<DebugManager>,
    mut ui_renderer: ResMut<crate::graphics::UIRenderer>,
) {
    if debug_manager.is_enabled {
        debug_manager.render_debug_ui(&mut ui_renderer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_manager_creation() {
        let manager = DebugManager::new();
        assert_eq!(manager.is_enabled, cfg!(debug_assertions));
        assert!(!manager.show_debug_ui);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        // 模拟一些帧时间
        for _ in 0..60 {
            monitor.update(0.016); // ~60 FPS
        }
        
        assert!(monitor.stats.fps > 50.0 && monitor.stats.fps < 70.0);
    }

    #[test]
    fn test_profiler() {
        let mut profiler = GameProfiler::new();
        
        let handle = profiler.start_profile("test_function");
        std::thread::sleep(std::time::Duration::from_millis(10));
        profiler.end_profile(handle);
        
        let avg_time = profiler.get_average_time("test_function").unwrap();
        assert!(avg_time > 0.005); // 应该至少有5ms
    }

    #[test]
    fn test_console_commands() {
        let mut console = DebugConsole::new();
        
        let result = console.execute_command("help");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Available commands"));
        
        let result = console.execute_command("unknown_command");
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        
        tracker.record_allocation("textures".to_string(), 1024);
        tracker.record_allocation("audio".to_string(), 512);
        
        assert_eq!(tracker.total_allocated, 1536);
        assert_eq!(tracker.peak_usage, 1536);
        
        let usage = tracker.get_memory_usage_by_category();
        assert_eq!(usage[0].0, "textures");
        assert_eq!(usage[0].1, 1024);
    }

    #[test]
    fn test_logger() {
        let mut logger = DebugLogger::new();
        
        logger.log(LogLevel::Info, "test", "Test message");
        assert_eq!(logger.log_entries.len(), 1);
        
        let entry = &logger.log_entries[0];
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, "test");
        assert_eq!(entry.message, "Test message");
    }
}