/*
 * 网络同步系统 - Network Synchronization System
 * 
 * 开发心理过程：
 * 设计高精度的网络同步机制，支持状态预测、回滚补偿、延迟补偿等技术
 * 需要考虑网络抖动、时钟同步、状态一致性和用户体验
 * 重点关注同步精度和网络适应性
 */

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::core::error::{GameResult, GameError};
use crate::core::math::{Vec2, Vec3};

// 同步模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncMode {
    Authoritative,  // 权威服务器模式
    P2P,           // P2P对等模式
    Hybrid,        // 混合模式
}

// 同步状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncState {
    Disconnected,
    Connecting,
    Synchronizing,
    Synchronized,
    Desynchronized,
    Reconnecting,
}

// 时间戳
pub type NetworkTimestamp = u64;
pub type LocalTimestamp = u64;

// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub mode: SyncMode,
    pub tick_rate: u32,              // 同步频率 (Hz)
    pub buffer_size: usize,          // 历史状态缓冲区大小
    pub prediction_frames: u32,      // 预测帧数
    pub rollback_frames: u32,        // 回滚帧数
    pub interpolation_enabled: bool, // 启用插值
    pub lag_compensation: bool,      // 延迟补偿
    pub jitter_buffer_size: u32,     // 抖动缓冲区大小
    pub max_desync_time: Duration,   // 最大失同步时间
    pub clock_sync_interval: Duration, // 时钟同步间隔
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Authoritative,
            tick_rate: 60,
            buffer_size: 256,
            prediction_frames: 8,
            rollback_frames: 8,
            interpolation_enabled: true,
            lag_compensation: true,
            jitter_buffer_size: 16,
            max_desync_time: Duration::from_secs(5),
            clock_sync_interval: Duration::from_secs(30),
        }
    }
}

// 网络实体
#[derive(Debug, Clone, Component)]
pub struct NetworkEntity {
    pub network_id: u32,
    pub owner_id: Option<u32>,
    pub sync_priority: SyncPriority,
    pub last_sync: NetworkTimestamp,
    pub predicted_state: Option<EntityState>,
    pub interpolation_data: InterpolationData,
}

// 同步优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncPriority {
    Critical = 3,  // 关键实体（玩家、重要NPC）
    High = 2,      // 高优先级（战斗中的宝可梦）
    Normal = 1,    // 普通优先级（世界对象）
    Low = 0,       // 低优先级（装饰性对象）
}

// 实体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub position: Vec3,
    pub rotation: f32,
    pub velocity: Vec3,
    pub animation_state: u16,
    pub health: Option<f32>,
    pub custom_data: HashMap<String, f32>,
    pub timestamp: NetworkTimestamp,
}

// 插值数据
#[derive(Debug, Clone)]
pub struct InterpolationData {
    pub start_state: EntityState,
    pub target_state: EntityState,
    pub progress: f32,
    pub duration: Duration,
    pub start_time: Instant,
}

// 同步包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPacket {
    pub timestamp: NetworkTimestamp,
    pub frame_number: u32,
    pub entities: Vec<EntityUpdate>,
    pub world_state: Option<WorldState>,
    pub checksum: u32,
}

// 实体更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityUpdate {
    pub network_id: u32,
    pub state: EntityState,
    pub update_flags: EntityUpdateFlags,
}

// 更新标志
bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct EntityUpdateFlags: u8 {
        const POSITION = 0x01;
        const ROTATION = 0x02;
        const VELOCITY = 0x04;
        const ANIMATION = 0x08;
        const HEALTH = 0x10;
        const CUSTOM_DATA = 0x20;
        const CREATED = 0x40;
        const DESTROYED = 0x80;
    }
}

// 世界状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub weather: u8,
    pub time_of_day: f32,
    pub global_effects: Vec<GlobalEffect>,
    pub battle_state: Option<BattleStateSync>,
}

// 全局效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEffect {
    pub effect_id: u16,
    pub duration: f32,
    pub intensity: f32,
    pub parameters: HashMap<String, f32>,
}

// 战斗状态同步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleStateSync {
    pub turn_number: u32,
    pub active_pokemon: Vec<u32>,
    pub field_effects: Vec<FieldEffectSync>,
    pub turn_timer: f32,
}

// 场地效果同步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEffectSync {
    pub effect_type: u16,
    pub duration: u16,
    pub affects_team: u8,
}

// 时钟同步数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSyncRequest {
    pub client_timestamp: NetworkTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSyncResponse {
    pub client_timestamp: NetworkTimestamp,
    pub server_timestamp: NetworkTimestamp,
    pub processing_time: u32, // 微秒
}

// 网络统计
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub ping: u32,
    pub jitter: u32,
    pub packet_loss: f32,
    pub bandwidth_up: u32,
    pub bandwidth_down: u32,
    pub prediction_accuracy: f32,
    pub rollback_frequency: f32,
    pub sync_drift: i64, // 时钟偏差（毫秒）
}

// 同步管理器
pub struct SyncManager {
    config: SyncConfig,
    state: SyncState,
    
    // 时间同步
    local_time_offset: i64,
    network_time: NetworkTimestamp,
    last_clock_sync: Instant,
    
    // 状态管理
    current_frame: u32,
    entity_states: HashMap<u32, VecDeque<EntityState>>,
    confirmed_states: BTreeMap<u32, SyncPacket>,
    predicted_states: HashMap<u32, EntityState>,
    
    // 网络统计
    stats: NetworkStats,
    ping_samples: VecDeque<u32>,
    jitter_samples: VecDeque<u32>,
    
    // 事件处理
    pending_updates: VecDeque<SyncPacket>,
    rollback_queue: VecDeque<RollbackEvent>,
    
    // 插值系统
    interpolation_targets: HashMap<u32, InterpolationTarget>,
}

// 回滚事件
#[derive(Debug, Clone)]
pub struct RollbackEvent {
    pub frame_number: u32,
    pub entity_id: u32,
    pub corrected_state: EntityState,
    pub prediction_error: f32,
}

// 插值目标
#[derive(Debug, Clone)]
pub struct InterpolationTarget {
    pub entity_id: u32,
    pub start_state: EntityState,
    pub end_state: EntityState,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing_type: EasingType,
}

// 缓动类型
#[derive(Debug, Clone, Copy)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl SyncManager {
    // 创建同步管理器
    pub fn new(config: SyncConfig) -> GameResult<Self> {
        Ok(Self {
            config,
            state: SyncState::Disconnected,
            local_time_offset: 0,
            network_time: 0,
            last_clock_sync: Instant::now(),
            current_frame: 0,
            entity_states: HashMap::new(),
            confirmed_states: BTreeMap::new(),
            predicted_states: HashMap::new(),
            stats: NetworkStats::default(),
            ping_samples: VecDeque::with_capacity(60),
            jitter_samples: VecDeque::with_capacity(60),
            pending_updates: VecDeque::new(),
            rollback_queue: VecDeque::new(),
            interpolation_targets: HashMap::new(),
        })
    }

    // 初始化同步系统
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化网络同步系统 (模式: {:?}, 频率: {}Hz)", 
              self.config.mode, self.config.tick_rate);
        
        self.state = SyncState::Connecting;
        self.network_time = Self::get_current_timestamp();
        
        Ok(())
    }

    // 更新同步系统
    pub fn update(&mut self, delta_time: Duration) -> GameResult<()> {
        match self.state {
            SyncState::Connecting => {
                // 尝试连接并开始时钟同步
                self.request_clock_sync()?;
            }
            SyncState::Synchronizing => {
                // 等待初始同步完成
                self.check_initial_sync()?;
            }
            SyncState::Synchronized => {
                // 正常同步更新
                self.update_synchronized(delta_time)?;
            }
            SyncState::Desynchronized => {
                // 尝试重新同步
                self.attempt_resync()?;
            }
            SyncState::Reconnecting => {
                // 重连处理
                self.handle_reconnect()?;
            }
            _ => {}
        }

        // 更新网络时间
        self.update_network_time(delta_time)?;
        
        // 处理pending更新
        self.process_pending_updates()?;
        
        // 处理插值
        if self.config.interpolation_enabled {
            self.update_interpolation(delta_time)?;
        }
        
        // 更新网络统计
        self.update_network_stats()?;

        Ok(())
    }

    // 处理同步数据包
    pub fn handle_sync_packet(&mut self, packet: SyncPacket) -> GameResult<()> {
        // 验证数据包
        if !self.validate_sync_packet(&packet) {
            warn!("收到无效的同步包，帧号: {}", packet.frame_number);
            return Err(GameError::Sync("无效的同步数据包".to_string()));
        }

        // 检查是否需要回滚
        if self.should_rollback(&packet) {
            self.perform_rollback(&packet)?;
        } else {
            // 正常应用更新
            self.apply_sync_packet(&packet)?;
        }

        // 存储确认的状态
        self.confirmed_states.insert(packet.frame_number, packet);

        // 清理过期的状态
        self.cleanup_old_states();

        Ok(())
    }

    // 创建同步数据包
    pub fn create_sync_packet(&self, entities: &[u32]) -> GameResult<SyncPacket> {
        let mut entity_updates = Vec::new();

        for &entity_id in entities {
            if let Some(states) = self.entity_states.get(&entity_id) {
                if let Some(latest_state) = states.back() {
                    entity_updates.push(EntityUpdate {
                        network_id: entity_id,
                        state: latest_state.clone(),
                        update_flags: self.determine_update_flags(entity_id, latest_state),
                    });
                }
            }
        }

        let packet = SyncPacket {
            timestamp: self.get_network_time(),
            frame_number: self.current_frame,
            entities: entity_updates,
            world_state: None, // TODO: 实现世界状态
            checksum: 0,       // TODO: 计算校验和
        };

        Ok(packet)
    }

    // 预测实体状态
    pub fn predict_entity_state(&mut self, entity_id: u32, frames_ahead: u32) -> Option<EntityState> {
        if let Some(states) = self.entity_states.get(&entity_id) {
            if let Some(current_state) = states.back() {
                // 简单的线性预测
                let predicted_state = self.linear_predict(current_state, frames_ahead);
                self.predicted_states.insert(entity_id, predicted_state.clone());
                return Some(predicted_state);
            }
        }
        None
    }

    // 设置实体状态
    pub fn set_entity_state(&mut self, entity_id: u32, state: EntityState) -> GameResult<()> {
        let states = self.entity_states.entry(entity_id).or_insert_with(VecDeque::new);
        
        // 维护缓冲区大小
        if states.len() >= self.config.buffer_size {
            states.pop_front();
        }
        
        states.push_back(state);
        Ok(())
    }

    // 获取实体状态
    pub fn get_entity_state(&self, entity_id: u32) -> Option<&EntityState> {
        self.entity_states.get(&entity_id)?.back()
    }

    // 获取预测状态
    pub fn get_predicted_state(&self, entity_id: u32) -> Option<&EntityState> {
        self.predicted_states.get(&entity_id)
    }

    // 插值到目标状态
    pub fn interpolate_to_state(
        &mut self,
        entity_id: u32,
        target_state: EntityState,
        duration: Duration,
        easing: EasingType
    ) -> GameResult<()> {
        
        if let Some(current_state) = self.get_entity_state(entity_id) {
            let interpolation_target = InterpolationTarget {
                entity_id,
                start_state: current_state.clone(),
                end_state: target_state,
                start_time: Instant::now(),
                duration,
                easing_type: easing,
            };
            
            self.interpolation_targets.insert(entity_id, interpolation_target);
        }
        
        Ok(())
    }

    // 处理时钟同步响应
    pub fn handle_clock_sync_response(&mut self, response: ClockSyncResponse) -> GameResult<()> {
        let round_trip_time = (self.get_current_timestamp() - response.client_timestamp) as i64;
        let network_latency = (round_trip_time - response.processing_time as i64) / 2;
        
        // 更新时间偏移
        let server_adjusted_time = response.server_timestamp + network_latency as u64;
        let local_time = self.get_current_timestamp();
        self.local_time_offset = server_adjusted_time as i64 - local_time as i64;
        
        // 更新ping统计
        self.update_ping_stats(network_latency as u32);
        
        info!("时钟同步完成，偏移: {}ms, 延迟: {}ms", 
              self.local_time_offset, network_latency);

        if self.state == SyncState::Connecting {
            self.state = SyncState::Synchronizing;
        }

        Ok(())
    }

    // 检测失同步
    pub fn detect_desync(&mut self) -> bool {
        // 检查时钟偏差
        if self.local_time_offset.abs() > 1000 { // 1秒
            warn!("检测到严重的时钟偏差: {}ms", self.local_time_offset);
            return true;
        }

        // 检查预测精度
        if self.stats.prediction_accuracy < 0.7 {
            warn!("预测精度过低: {:.2}", self.stats.prediction_accuracy);
            return true;
        }

        // 检查网络质量
        if self.stats.packet_loss > 0.1 {
            warn!("丢包率过高: {:.2}%", self.stats.packet_loss * 100.0);
            return true;
        }

        false
    }

    // 私有方法

    // 获取网络时间
    pub fn get_network_time(&self) -> NetworkTimestamp {
        (self.get_current_timestamp() as i64 + self.local_time_offset) as u64
    }

    // 获取当前时间戳
    fn get_current_timestamp() -> NetworkTimestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    // 验证同步数据包
    fn validate_sync_packet(&self, packet: &SyncPacket) -> bool {
        // 检查时间戳合理性
        let current_time = self.get_network_time();
        let time_diff = (packet.timestamp as i64 - current_time as i64).abs();
        
        if time_diff > 5000 { // 5秒
            return false;
        }

        // 检查帧号序列
        if let Some(last_confirmed) = self.confirmed_states.keys().last() {
            if packet.frame_number <= *last_confirmed && 
               packet.frame_number + 100 < *last_confirmed {
                return false; // 太老的包
            }
        }

        // TODO: 验证校验和
        true
    }

    // 判断是否需要回滚
    fn should_rollback(&self, packet: &SyncPacket) -> bool {
        // 如果收到的包的帧号小于当前帧，且存在状态差异，则需要回滚
        if packet.frame_number < self.current_frame {
            if let Some(confirmed_packet) = self.confirmed_states.get(&packet.frame_number) {
                return !self.states_equal(&packet.entities, &confirmed_packet.entities);
            }
            return true;
        }
        false
    }

    // 执行回滚
    fn perform_rollback(&mut self, packet: &SyncPacket) -> GameResult<()> {
        info!("执行回滚到帧 {}", packet.frame_number);

        // 恢复到回滚帧的状态
        for entity_update in &packet.entities {
            if let Some(states) = self.entity_states.get_mut(&entity_update.network_id) {
                // 移除回滚帧之后的所有状态
                while let Some(state) = states.back() {
                    if state.timestamp > packet.timestamp {
                        states.pop_back();
                    } else {
                        break;
                    }
                }
                
                // 应用正确的状态
                states.push_back(entity_update.state.clone());
            }
        }

        // 记录回滚事件
        let rollback_event = RollbackEvent {
            frame_number: packet.frame_number,
            entity_id: 0, // TODO: 具体的实体ID
            corrected_state: packet.entities.first().unwrap().state.clone(),
            prediction_error: 0.0, // TODO: 计算预测误差
        };
        
        self.rollback_queue.push_back(rollback_event);

        // 更新统计
        self.stats.rollback_frequency += 1.0;

        Ok(())
    }

    // 应用同步数据包
    fn apply_sync_packet(&mut self, packet: &SyncPacket) -> GameResult<()> {
        for entity_update in &packet.entities {
            self.set_entity_state(entity_update.network_id, entity_update.state.clone())?;
            
            // 设置插值目标
            if self.config.interpolation_enabled {
                if let Some(current_state) = self.get_entity_state(entity_update.network_id) {
                    let interpolation_duration = Duration::from_millis(1000 / self.config.tick_rate as u64);
                    self.interpolate_to_state(
                        entity_update.network_id,
                        entity_update.state.clone(),
                        interpolation_duration,
                        EasingType::Linear
                    )?;
                }
            }
        }

        // 更新当前帧
        if packet.frame_number > self.current_frame {
            self.current_frame = packet.frame_number;
        }

        Ok(())
    }

    // 线性预测
    fn linear_predict(&self, current_state: &EntityState, frames_ahead: u32) -> EntityState {
        let frame_time = 1.0 / self.config.tick_rate as f32;
        let prediction_time = frames_ahead as f32 * frame_time;

        EntityState {
            position: current_state.position + current_state.velocity * prediction_time,
            rotation: current_state.rotation,
            velocity: current_state.velocity,
            animation_state: current_state.animation_state,
            health: current_state.health,
            custom_data: current_state.custom_data.clone(),
            timestamp: current_state.timestamp + (prediction_time * 1000.0) as u64,
        }
    }

    // 确定更新标志
    fn determine_update_flags(&self, _entity_id: u32, _state: &EntityState) -> EntityUpdateFlags {
        // 简化实现，实际应该比较状态差异
        EntityUpdateFlags::POSITION | EntityUpdateFlags::VELOCITY
    }

    // 比较状态是否相等
    fn states_equal(&self, states1: &[EntityUpdate], states2: &[EntityUpdate]) -> bool {
        if states1.len() != states2.len() {
            return false;
        }

        for (s1, s2) in states1.iter().zip(states2.iter()) {
            if s1.network_id != s2.network_id || !self.entity_states_equal(&s1.state, &s2.state) {
                return false;
            }
        }

        true
    }

    // 比较实体状态是否相等
    fn entity_states_equal(&self, state1: &EntityState, state2: &EntityState) -> bool {
        const EPSILON: f32 = 0.001;

        (state1.position - state2.position).length() < EPSILON &&
        (state1.rotation - state2.rotation).abs() < EPSILON &&
        (state1.velocity - state2.velocity).length() < EPSILON
    }

    // 更新网络时间
    fn update_network_time(&mut self, delta_time: Duration) -> GameResult<()> {
        self.network_time += delta_time.as_millis() as u64;

        // 定期请求时钟同步
        if self.last_clock_sync.elapsed() > self.config.clock_sync_interval {
            self.request_clock_sync()?;
            self.last_clock_sync = Instant::now();
        }

        Ok(())
    }

    // 请求时钟同步
    fn request_clock_sync(&mut self) -> GameResult<()> {
        let request = ClockSyncRequest {
            client_timestamp: self.get_current_timestamp(),
        };

        // TODO: 发送时钟同步请求到服务器
        debug!("请求时钟同步: {}", request.client_timestamp);
        
        Ok(())
    }

    // 处理pending更新
    fn process_pending_updates(&mut self) -> GameResult<()> {
        while let Some(packet) = self.pending_updates.pop_front() {
            self.handle_sync_packet(packet)?;
        }
        Ok(())
    }

    // 更新插值
    fn update_interpolation(&mut self, delta_time: Duration) -> GameResult<()> {
        let current_time = Instant::now();
        let mut completed_interpolations = Vec::new();

        for (entity_id, target) in &mut self.interpolation_targets {
            let elapsed = current_time.duration_since(target.start_time);
            let progress = (elapsed.as_secs_f32() / target.duration.as_secs_f32()).clamp(0.0, 1.0);
            
            let eased_progress = match target.easing_type {
                EasingType::Linear => progress,
                EasingType::EaseIn => progress * progress,
                EasingType::EaseOut => 1.0 - (1.0 - progress).powi(2),
                EasingType::EaseInOut => {
                    if progress < 0.5 {
                        2.0 * progress * progress
                    } else {
                        1.0 - 2.0 * (1.0 - progress).powi(2)
                    }
                }
            };

            // 插值状态
            let interpolated_state = EntityState {
                position: target.start_state.position.lerp(target.end_state.position, eased_progress),
                rotation: lerp_angle(target.start_state.rotation, target.end_state.rotation, eased_progress),
                velocity: target.start_state.velocity.lerp(target.end_state.velocity, eased_progress),
                animation_state: target.end_state.animation_state,
                health: target.end_state.health,
                custom_data: target.end_state.custom_data.clone(),
                timestamp: target.end_state.timestamp,
            };

            // 更新实体状态
            self.set_entity_state(*entity_id, interpolated_state)?;

            if progress >= 1.0 {
                completed_interpolations.push(*entity_id);
            }
        }

        // 移除完成的插值
        for entity_id in completed_interpolations {
            self.interpolation_targets.remove(&entity_id);
        }

        Ok(())
    }

    // 更新网络统计
    fn update_network_stats(&mut self) -> GameResult<()> {
        // 计算平均ping
        if !self.ping_samples.is_empty() {
            self.stats.ping = self.ping_samples.iter().sum::<u32>() / self.ping_samples.len() as u32;
        }

        // 计算抖动
        if self.ping_samples.len() >= 2 {
            let mut jitter_sum = 0u32;
            for window in self.ping_samples.windows(2) {
                jitter_sum += (window[1] as i32 - window[0] as i32).abs() as u32;
            }
            self.stats.jitter = jitter_sum / (self.ping_samples.len() - 1) as u32;
        }

        // 更新时钟偏差
        self.stats.sync_drift = self.local_time_offset;

        Ok(())
    }

    // 更新ping统计
    fn update_ping_stats(&mut self, ping: u32) {
        if self.ping_samples.len() >= 60 {
            self.ping_samples.pop_front();
        }
        self.ping_samples.push_back(ping);
    }

    // 清理过期状态
    fn cleanup_old_states(&mut self) {
        let cutoff_frame = self.current_frame.saturating_sub(self.config.rollback_frames);
        
        // 清理确认状态
        while let Some(&first_frame) = self.confirmed_states.keys().next() {
            if first_frame < cutoff_frame {
                self.confirmed_states.remove(&first_frame);
            } else {
                break;
            }
        }

        // 清理实体状态历史
        for states in self.entity_states.values_mut() {
            while let Some(state) = states.front() {
                // 简化的时间检查
                if states.len() > self.config.buffer_size {
                    states.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    // 检查初始同步
    fn check_initial_sync(&mut self) -> GameResult<()> {
        // 简化实现
        if !self.confirmed_states.is_empty() {
            self.state = SyncState::Synchronized;
            info!("初始同步完成");
        }
        Ok(())
    }

    // 正常同步更新
    fn update_synchronized(&mut self, _delta_time: Duration) -> GameResult<()> {
        // 检测失同步
        if self.detect_desync() {
            self.state = SyncState::Desynchronized;
            warn!("检测到失同步");
        }
        Ok(())
    }

    // 尝试重新同步
    fn attempt_resync(&mut self) -> GameResult<()> {
        // 清理状态并请求完整同步
        self.entity_states.clear();
        self.predicted_states.clear();
        self.confirmed_states.clear();
        
        // 请求时钟重新同步
        self.request_clock_sync()?;
        
        self.state = SyncState::Synchronizing;
        info!("开始重新同步");
        Ok(())
    }

    // 处理重连
    fn handle_reconnect(&mut self) -> GameResult<()> {
        // TODO: 实现重连逻辑
        self.state = SyncState::Connecting;
        Ok(())
    }

    // 获取同步状态
    pub fn get_sync_state(&self) -> SyncState {
        self.state
    }

    // 获取网络统计
    pub fn get_network_stats(&self) -> &NetworkStats {
        &self.stats
    }

    // 获取当前帧号
    pub fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    // 设置同步模式
    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.config.mode = mode;
        info!("同步模式切换为: {:?}", mode);
    }
}

// 辅助函数
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a + std::f32::consts::PI) % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    a + diff * t
}

// Bevy系统实现
pub fn sync_system(
    mut sync_manager: ResMut<SyncManager>,
    time: Res<Time>,
) {
    let _ = sync_manager.update(time.delta());
}

pub fn network_entity_sync_system(
    mut sync_manager: ResMut<SyncManager>,
    mut query: Query<(Entity, &mut Transform, &NetworkEntity)>,
) {
    // 同步网络实体的变换
    for (entity, mut transform, network_entity) in query.iter_mut() {
        if let Some(state) = sync_manager.get_entity_state(network_entity.network_id) {
            transform.translation = state.position;
            transform.rotation = Quat::from_rotation_z(state.rotation);
        }
        
        // 应用预测状态
        if let Some(predicted_state) = sync_manager.get_predicted_state(network_entity.network_id) {
            // 在权威服务器模式下，客户端可以显示预测状态
            if sync_manager.config.mode == SyncMode::Authoritative {
                transform.translation = predicted_state.position;
                transform.rotation = Quat::from_rotation_z(predicted_state.rotation);
            }
        }
    }
}

// 便捷函数
impl SyncManager {
    // 批量设置实体状态
    pub fn set_entity_states(&mut self, states: Vec<(u32, EntityState)>) -> GameResult<()> {
        for (entity_id, state) in states {
            self.set_entity_state(entity_id, state)?;
        }
        Ok(())
    }

    // 批量预测实体状态
    pub fn predict_entities(&mut self, entity_ids: &[u32], frames_ahead: u32) -> Vec<Option<EntityState>> {
        entity_ids.iter()
            .map(|&id| self.predict_entity_state(id, frames_ahead))
            .collect()
    }

    // 强制同步所有实体
    pub fn force_sync_all(&mut self) -> GameResult<()> {
        self.state = SyncState::Synchronizing;
        self.request_clock_sync()?;
        
        info!("强制同步所有实体");
        Ok(())
    }

    // 获取延迟补偿的时间
    pub fn get_lag_compensation_time(&self) -> Duration {
        Duration::from_millis(self.stats.ping as u64 / 2)
    }

    // 检查实体是否需要同步
    pub fn needs_sync(&self, entity_id: u32, threshold: f32) -> bool {
        if let (Some(current), Some(predicted)) = (
            self.get_entity_state(entity_id),
            self.get_predicted_state(entity_id)
        ) {
            let position_error = (current.position - predicted.position).length();
            let velocity_error = (current.velocity - predicted.velocity).length();
            
            position_error > threshold || velocity_error > threshold
        } else {
            true // 如果没有状态数据，需要同步
        }
    }
}