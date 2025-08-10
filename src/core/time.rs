// 时间系统模块 - 游戏时间管理
// 开发心理：时间是游戏逻辑的基础，需要统一的时间管理系统
// 设计原则：精确计时、时间缩放、暂停恢复、帧率无关

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

// 游戏时间
#[derive(Debug, Clone)]
pub struct GameTime {
    pub total_time: Duration,
    pub delta_time: Duration,
    pub time_scale: f64,
    pub is_paused: bool,
    start_time: Instant,
    last_frame_time: Instant,
    real_delta_time: Duration,
}

impl GameTime {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            total_time: Duration::ZERO,
            delta_time: Duration::ZERO,
            time_scale: 1.0,
            is_paused: false,
            start_time: now,
            last_frame_time: now,
            real_delta_time: Duration::ZERO,
        }
    }
    
    pub fn update(&mut self, delta: Duration) {
        let now = Instant::now();
        self.real_delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        // 使用传入的delta时间，支持Bevy的时间系统
        if !self.is_paused {
            let scaled_delta = Duration::from_secs_f64(
                delta.as_secs_f64() * self.time_scale
            );
            self.delta_time = scaled_delta;
            self.total_time += scaled_delta;
        } else {
            self.delta_time = Duration::ZERO;
        }
    }
    
    // 新增方法用于App
    pub fn frame_count(&self) -> u64 {
        (self.total_time.as_secs_f64() * 60.0) as u64 // 假设60FPS
    }
    
    pub fn average_frame_time(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }
    
    pub fn fps(&self) -> f32 {
        if self.delta_time.is_zero() {
            0.0
        } else {
            1.0 / self.delta_time.as_secs_f32()
        }
    }
    
    pub fn pause(&mut self) {
        self.is_paused = true;
    }
    
    pub fn resume(&mut self) {
        self.is_paused = false;
        self.last_frame_time = Instant::now();
    }
    
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }
    
    pub fn get_real_delta_time(&self) -> Duration {
        self.real_delta_time
    }
    
    pub fn get_fps(&self) -> f64 {
        if self.real_delta_time.is_zero() {
            0.0
        } else {
            1.0 / self.real_delta_time.as_secs_f64()
        }
    }
    
    pub fn elapsed_real_time(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

// 计时器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    duration: Duration,
    elapsed: Duration,
    repeating: bool,
    active: bool,
}

impl Timer {
    pub fn new(duration: Duration, repeating: bool) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating,
            active: true,
        }
    }
    
    pub fn from_seconds(seconds: f64, repeating: bool) -> Self {
        Self::new(Duration::from_secs_f64(seconds), repeating)
    }
    
    pub fn update(&mut self, delta_time: Duration) -> bool {
        if !self.active {
            return false;
        }
        
        self.elapsed += delta_time;
        
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.active = false;
            }
            true
        } else {
            false
        }
    }
    
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.active = true;
    }
    
    pub fn stop(&mut self) {
        self.active = false;
    }
    
    pub fn is_finished(&self) -> bool {
        !self.active && self.elapsed >= self.duration
    }
    
    pub fn progress(&self) -> f64 {
        if self.duration.is_zero() {
            1.0
        } else {
            (self.elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0)
        }
    }
    
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed)
    }
    
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
}

// 时间工具函数
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let milliseconds = duration.subsec_millis();
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
    } else {
        format!("{:02}:{:02}.{:03}", minutes, seconds, milliseconds)
    }
}

pub fn lerp_duration(from: Duration, to: Duration, t: f64) -> Duration {
    let t = t.clamp(0.0, 1.0);
    let from_secs = from.as_secs_f64();
    let to_secs = to.as_secs_f64();
    let result_secs = from_secs + (to_secs - from_secs) * t;
    Duration::from_secs_f64(result_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer() {
        let mut timer = Timer::from_seconds(1.0, false);
        assert!(!timer.update(Duration::from_millis(500)));
        assert!(timer.is_active());
        
        assert!(timer.update(Duration::from_millis(600)));
        assert!(!timer.is_active());
        assert!(timer.is_finished());
    }
    
    #[test]
    fn test_repeating_timer() {
        let mut timer = Timer::from_seconds(0.5, true);
        assert!(timer.update(Duration::from_millis(600)));
        assert!(timer.is_active());
        assert!(!timer.is_finished());
    }
    
    #[test]
    fn test_game_time() {
        let mut game_time = GameTime::new();
        game_time.set_time_scale(2.0);
        
        // 模拟16ms帧时间（约60FPS）
        std::thread::sleep(Duration::from_millis(16));
        game_time.update();
        
        // 由于时间缩放为2.0，游戏时间应该比实际时间快
        assert!(game_time.delta_time > game_time.real_delta_time);
    }
    
    #[test]
    fn test_format_duration() {
        let duration = Duration::from_millis(125500); // 2分5.5秒
        let formatted = format_duration(duration);
        assert_eq!(formatted, "02:05.500");
    }
}