/*
 * 匹配系统 - Matchmaking System
 * 
 * 开发心理过程：
 * 设计智能的玩家匹配系统，支持技能等级匹配、延迟优化、队伍平衡等功能
 * 需要考虑匹配公平性、等待时间、地理位置和玩家偏好
 * 重点关注匹配质量和用户满意度
 */

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::core::error::{GameResult, GameError};

// 匹配状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchmakingState {
    Idle,
    Searching,
    MatchFound,
    Connecting,
    InGame,
    Failed,
}

// 游戏模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    Casual1v1,      // 休闲1v1
    Ranked1v1,      // 排位1v1
    Double2v2,      // 2v2双打
    Multi4Player,   // 4人混战
    Tournament,     // 锦标赛
    CustomMatch,    // 自定义匹配
}

// 玩家等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerTier {
    Beginner = 1,   // 新手 (0-999)
    Bronze = 2,     // 青铜 (1000-1199)
    Silver = 3,     // 白银 (1200-1399)  
    Gold = 4,       // 黄金 (1400-1599)
    Platinum = 5,   // 白金 (1600-1799)
    Diamond = 6,    // 钻石 (1800-1999)
    Master = 7,     // 大师 (2000-2199)
    Grandmaster = 8, // 宗师 (2200+)
}

// 匹配请求
#[derive(Debug, Clone)]
pub struct MatchmakingRequest {
    pub player_id: Uuid,
    pub game_mode: GameMode,
    pub rating: u32,
    pub tier: PlayerTier,
    pub preferences: MatchmakingPreferences,
    pub team: Option<Vec<Uuid>>, // 组队匹配
    pub timestamp: Instant,
    pub expanded_search_time: Duration,
}

// 匹配偏好
#[derive(Debug, Clone)]
pub struct MatchmakingPreferences {
    pub max_rating_difference: u32,
    pub preferred_regions: Vec<String>,
    pub max_ping: u32,
    pub language: String,
    pub avoid_players: Vec<Uuid>,
    pub allow_cross_platform: bool,
    pub priority_queue: bool,
}

impl Default for MatchmakingPreferences {
    fn default() -> Self {
        Self {
            max_rating_difference: 200,
            preferred_regions: vec!["auto".to_string()],
            max_ping: 150,
            language: "zh".to_string(),
            avoid_players: Vec::new(),
            allow_cross_platform: true,
            priority_queue: false,
        }
    }
}

// 匹配结果
#[derive(Debug, Clone)]
pub struct MatchmakingResult {
    pub match_id: String,
    pub players: Vec<MatchedPlayer>,
    pub game_mode: GameMode,
    pub server_region: String,
    pub estimated_ping: u32,
    pub quality_score: f32,
    pub created_at: Instant,
}

// 匹配的玩家
#[derive(Debug, Clone)]
pub struct MatchedPlayer {
    pub player_id: Uuid,
    pub rating: u32,
    pub tier: PlayerTier,
    pub team_id: u8,
    pub ping: u32,
    pub region: String,
}

// 匹配队列
#[derive(Debug)]
pub struct MatchmakingQueue {
    pub game_mode: GameMode,
    pub requests: VecDeque<MatchmakingRequest>,
    pub priority_requests: VecDeque<MatchmakingRequest>,
    pub average_wait_time: Duration,
    pub peak_hours: Vec<u8>, // 高峰时段(小时)
}

impl MatchmakingQueue {
    pub fn new(game_mode: GameMode) -> Self {
        Self {
            game_mode,
            requests: VecDeque::new(),
            priority_requests: VecDeque::new(),
            average_wait_time: Duration::from_secs(60),
            peak_hours: vec![19, 20, 21, 22], // 默认晚高峰
        }
    }

    pub fn add_request(&mut self, request: MatchmakingRequest) {
        if request.preferences.priority_queue {
            self.priority_requests.push_back(request);
        } else {
            self.requests.push_back(request);
        }
    }

    pub fn remove_request(&mut self, player_id: Uuid) -> bool {
        let removed_normal = self.requests.iter()
            .position(|r| r.player_id == player_id)
            .map(|pos| self.requests.remove(pos))
            .is_some();
            
        let removed_priority = self.priority_requests.iter()
            .position(|r| r.player_id == player_id)
            .map(|pos| self.priority_requests.remove(pos))
            .is_some();
            
        removed_normal || removed_priority
    }

    pub fn get_next_request(&mut self) -> Option<MatchmakingRequest> {
        self.priority_requests.pop_front()
            .or_else(|| self.requests.pop_front())
    }

    pub fn len(&self) -> usize {
        self.requests.len() + self.priority_requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.priority_requests.is_empty()
    }
}

// 匹配算法配置
#[derive(Debug, Clone)]
pub struct MatchmakingConfig {
    pub max_search_time: Duration,
    pub rating_expansion_rate: f32, // 每秒评级范围扩展速度
    pub ping_weight: f32,
    pub rating_weight: f32,
    pub wait_time_weight: f32,
    pub region_bonus: f32,
    pub language_bonus: f32,
    pub avoid_penalty: f32,
    pub team_balance_weight: f32,
    pub min_players_per_mode: HashMap<GameMode, usize>,
}

impl Default for MatchmakingConfig {
    fn default() -> Self {
        let mut min_players = HashMap::new();
        min_players.insert(GameMode::Casual1v1, 2);
        min_players.insert(GameMode::Ranked1v1, 2);
        min_players.insert(GameMode::Double2v2, 4);
        min_players.insert(GameMode::Multi4Player, 4);
        min_players.insert(GameMode::Tournament, 8);
        min_players.insert(GameMode::CustomMatch, 2);

        Self {
            max_search_time: Duration::from_secs(300), // 5分钟
            rating_expansion_rate: 50.0, // 每秒扩展50评级点
            ping_weight: 0.3,
            rating_weight: 0.4,
            wait_time_weight: 0.2,
            region_bonus: 0.1,
            language_bonus: 0.05,
            avoid_penalty: 0.5,
            team_balance_weight: 0.3,
            min_players_per_mode: min_players,
        }
    }
}

// 匹配统计
#[derive(Debug, Default)]
pub struct MatchmakingStats {
    pub total_requests: u64,
    pub successful_matches: u64,
    pub failed_matches: u64,
    pub average_wait_time: Duration,
    pub average_quality_score: f32,
    pub queue_sizes: HashMap<GameMode, usize>,
    pub peak_concurrent_searches: usize,
    pub regional_distribution: HashMap<String, u32>,
}

// 匹配事件
#[derive(Debug, Clone)]
pub enum MatchmakingEvent {
    PlayerJoinedQueue { player_id: Uuid, game_mode: GameMode },
    PlayerLeftQueue { player_id: Uuid, game_mode: GameMode },
    MatchFound { match_id: String, players: Vec<Uuid> },
    MatchFailed { reason: String },
    QueueUpdated { game_mode: GameMode, size: usize },
}

// 匹配服务器信息
#[derive(Debug, Clone)]
pub struct GameServer {
    pub id: String,
    pub region: String,
    pub capacity: u32,
    pub current_load: u32,
    pub average_ping: HashMap<String, u32>, // 到各地区的平均延迟
    pub status: ServerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerStatus {
    Online,
    Maintenance,
    Overloaded,
    Offline,
}

// 匹配管理器
pub struct MatchmakingManager {
    config: MatchmakingConfig,
    queues: HashMap<GameMode, MatchmakingQueue>,
    active_searches: HashMap<Uuid, MatchmakingState>,
    completed_matches: HashMap<String, MatchmakingResult>,
    servers: HashMap<String, GameServer>,
    stats: MatchmakingStats,
    event_sender: tokio::sync::mpsc::UnboundedSender<MatchmakingEvent>,
    last_matchmaking_run: Instant,
}

impl MatchmakingManager {
    // 创建匹配管理器
    pub fn new() -> GameResult<Self> {
        let (event_sender, _) = tokio::sync::mpsc::unbounded_channel();
        
        let mut queues = HashMap::new();
        for &mode in &[
            GameMode::Casual1v1, GameMode::Ranked1v1, GameMode::Double2v2,
            GameMode::Multi4Player, GameMode::Tournament, GameMode::CustomMatch
        ] {
            queues.insert(mode, MatchmakingQueue::new(mode));
        }

        Ok(Self {
            config: MatchmakingConfig::default(),
            queues,
            active_searches: HashMap::new(),
            completed_matches: HashMap::new(),
            servers: HashMap::new(),
            stats: MatchmakingStats::default(),
            event_sender,
            last_matchmaking_run: Instant::now(),
        })
    }

    // 初始化匹配系统
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化匹配系统...");
        
        // 注册默认服务器
        self.register_default_servers()?;
        
        info!("匹配系统初始化完成");
        Ok(())
    }

    // 加入匹配队列
    pub fn join_queue(
        &mut self, 
        player_id: Uuid, 
        game_mode: GameMode,
        rating: u32,
        preferences: MatchmakingPreferences,
        team: Option<Vec<Uuid>>
    ) -> GameResult<()> {
        
        // 检查玩家是否已在队列中
        if self.active_searches.contains_key(&player_id) {
            return Err(GameError::Matchmaking("玩家已在匹配队列中".to_string()));
        }

        let tier = Self::rating_to_tier(rating);
        
        let request = MatchmakingRequest {
            player_id,
            game_mode,
            rating,
            tier,
            preferences,
            team,
            timestamp: Instant::now(),
            expanded_search_time: Duration::ZERO,
        };

        // 添加到对应队列
        if let Some(queue) = self.queues.get_mut(&game_mode) {
            queue.add_request(request);
            self.active_searches.insert(player_id, MatchmakingState::Searching);
            
            // 发送事件
            let _ = self.event_sender.send(MatchmakingEvent::PlayerJoinedQueue {
                player_id,
                game_mode,
            });
            
            info!("玩家 {} 加入 {:?} 匹配队列 (评级: {})", player_id, game_mode, rating);
            self.stats.total_requests += 1;
        }

        Ok(())
    }

    // 离开匹配队列
    pub fn leave_queue(&mut self, player_id: Uuid) -> GameResult<()> {
        // 从所有队列中移除玩家
        let mut removed_from_mode = None;
        
        for (mode, queue) in &mut self.queues {
            if queue.remove_request(player_id) {
                removed_from_mode = Some(*mode);
                break;
            }
        }

        if let Some(mode) = removed_from_mode {
            self.active_searches.remove(&player_id);
            
            let _ = self.event_sender.send(MatchmakingEvent::PlayerLeftQueue {
                player_id,
                game_mode: mode,
            });
            
            info!("玩家 {} 离开 {:?} 匹配队列", player_id, mode);
        }

        Ok(())
    }

    // 更新匹配系统
    pub fn update(&mut self) -> GameResult<()> {
        let now = Instant::now();
        
        // 限制匹配频率
        if now.duration_since(self.last_matchmaking_run) < Duration::from_millis(500) {
            return Ok(());
        }

        self.last_matchmaking_run = now;

        // 处理每个游戏模式的队列
        for (&game_mode, queue) in &mut self.queues {
            if !queue.is_empty() {
                self.process_queue(game_mode, queue)?;
            }
        }

        // 清理过期的搜索
        self.cleanup_expired_searches()?;
        
        // 更新统计信息
        self.update_statistics();

        Ok(())
    }

    // 处理队列匹配
    fn process_queue(&mut self, game_mode: GameMode, queue: &mut MatchmakingQueue) -> GameResult<()> {
        let min_players = *self.config.min_players_per_mode.get(&game_mode).unwrap_or(&2);
        
        if queue.len() < min_players {
            return Ok(());
        }

        // 收集潜在的匹配候选者
        let mut candidates = Vec::new();
        let max_candidates = std::cmp::min(queue.len(), min_players * 3); // 避免处理太多候选者
        
        for _ in 0..max_candidates {
            if let Some(request) = queue.get_next_request() {
                candidates.push(request);
            } else {
                break;
            }
        }

        // 尝试创建匹配
        if let Some(match_result) = self.find_best_match(game_mode, &candidates)? {
            self.create_match(match_result)?;
        } else {
            // 如果没有找到合适的匹配，将候选者放回队列
            for candidate in candidates {
                queue.add_request(candidate);
            }
        }

        Ok(())
    }

    // 寻找最佳匹配
    fn find_best_match(
        &self,
        game_mode: GameMode,
        candidates: &[MatchmakingRequest]
    ) -> GameResult<Option<MatchmakingResult>> {
        
        let min_players = *self.config.min_players_per_mode.get(&game_mode).unwrap_or(&2);
        
        if candidates.len() < min_players {
            return Ok(None);
        }

        match game_mode {
            GameMode::Casual1v1 | GameMode::Ranked1v1 => {
                self.find_1v1_match(candidates)
            }
            GameMode::Double2v2 => {
                self.find_2v2_match(candidates)
            }
            GameMode::Multi4Player => {
                self.find_multi_match(candidates)
            }
            GameMode::Tournament => {
                self.find_tournament_match(candidates)
            }
            GameMode::CustomMatch => {
                self.find_custom_match(candidates)
            }
        }
    }

    // 1v1匹配
    fn find_1v1_match(&self, candidates: &[MatchmakingRequest]) -> GameResult<Option<MatchmakingResult>> {
        let mut best_match = None;
        let mut best_quality = 0.0f32;

        // 尝试所有可能的组合
        for i in 0..candidates.len() {
            for j in i+1..candidates.len() {
                let player1 = &candidates[i];
                let player2 = &candidates[j];

                // 检查基本兼容性
                if !self.are_players_compatible(player1, player2) {
                    continue;
                }

                let quality = self.calculate_match_quality(&[player1.clone(), player2.clone()]);
                
                if quality > best_quality {
                    best_quality = quality;
                    
                    let server = self.select_best_server(&[player1, player2])?;
                    
                    best_match = Some(MatchmakingResult {
                        match_id: Uuid::new_v4().to_string(),
                        players: vec![
                            MatchedPlayer {
                                player_id: player1.player_id,
                                rating: player1.rating,
                                tier: player1.tier,
                                team_id: 1,
                                ping: self.estimate_ping(player1, &server),
                                region: self.get_player_region(player1),
                            },
                            MatchedPlayer {
                                player_id: player2.player_id,
                                rating: player2.rating,
                                tier: player2.tier,
                                team_id: 2,
                                ping: self.estimate_ping(player2, &server),
                                region: self.get_player_region(player2),
                            }
                        ],
                        game_mode: player1.game_mode,
                        server_region: server.region.clone(),
                        estimated_ping: (self.estimate_ping(player1, &server) + self.estimate_ping(player2, &server)) / 2,
                        quality_score: quality,
                        created_at: Instant::now(),
                    });
                }
            }
        }

        // 只有质量足够高的匹配才会被接受
        if best_quality > 0.3 {
            Ok(best_match)
        } else {
            Ok(None)
        }
    }

    // 2v2匹配
    fn find_2v2_match(&self, candidates: &[MatchmakingRequest]) -> GameResult<Option<MatchmakingResult>> {
        // TODO: 实现2v2匹配逻辑
        // 需要考虑队伍平衡、组队玩家等
        Ok(None)
    }

    // 多人匹配
    fn find_multi_match(&self, candidates: &[MatchmakingRequest]) -> GameResult<Option<MatchmakingResult>> {
        // TODO: 实现多人匹配逻辑
        Ok(None)
    }

    // 锦标赛匹配
    fn find_tournament_match(&self, candidates: &[MatchmakingRequest]) -> GameResult<Option<MatchmakingResult>> {
        // TODO: 实现锦标赛匹配逻辑
        Ok(None)
    }

    // 自定义匹配
    fn find_custom_match(&self, candidates: &[MatchmakingRequest]) -> GameResult<Option<MatchmakingResult>> {
        // TODO: 实现自定义匹配逻辑
        Ok(None)
    }

    // 创建匹配
    fn create_match(&mut self, match_result: MatchmakingResult) -> GameResult<()> {
        let match_id = match_result.match_id.clone();
        let player_ids: Vec<Uuid> = match_result.players.iter().map(|p| p.player_id).collect();

        // 更新玩家状态
        for player_id in &player_ids {
            self.active_searches.insert(*player_id, MatchmakingState::MatchFound);
        }

        // 存储匹配结果
        self.completed_matches.insert(match_id.clone(), match_result);

        // 发送匹配成功事件
        let _ = self.event_sender.send(MatchmakingEvent::MatchFound {
            match_id,
            players: player_ids,
        });

        self.stats.successful_matches += 1;
        info!("创建匹配成功: 玩家数量 {}", player_ids.len());

        Ok(())
    }

    // 检查玩家兼容性
    fn are_players_compatible(&self, player1: &MatchmakingRequest, player2: &MatchmakingRequest) -> bool {
        // 检查避免列表
        if player1.preferences.avoid_players.contains(&player2.player_id) ||
           player2.preferences.avoid_players.contains(&player1.player_id) {
            return false;
        }

        // 检查评级差异
        let rating_diff = (player1.rating as i32 - player2.rating as i32).abs() as u32;
        let max_diff = self.get_expanded_rating_range(player1).max(self.get_expanded_rating_range(player2));
        
        if rating_diff > max_diff {
            return false;
        }

        // 检查语言偏好
        if player1.preferences.language != player2.preferences.language &&
           player1.preferences.language != "any" && player2.preferences.language != "any" {
            return false;
        }

        true
    }

    // 计算匹配质量
    fn calculate_match_quality(&self, players: &[MatchmakingRequest]) -> f32 {
        if players.len() < 2 {
            return 0.0;
        }

        let mut total_quality = 0.0f32;
        let mut comparisons = 0;

        // 计算所有玩家对之间的质量
        for i in 0..players.len() {
            for j in i+1..players.len() {
                let player1 = &players[i];
                let player2 = &players[j];
                
                let quality = self.calculate_pair_quality(player1, player2);
                total_quality += quality;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            total_quality / comparisons as f32
        } else {
            0.0
        }
    }

    // 计算配对质量
    fn calculate_pair_quality(&self, player1: &MatchmakingRequest, player2: &MatchmakingRequest) -> f32 {
        let mut quality = 1.0f32;

        // 评级差异惩罚
        let rating_diff = (player1.rating as i32 - player2.rating as i32).abs() as u32;
        let rating_penalty = (rating_diff as f32 / 1000.0) * self.config.rating_weight;
        quality -= rating_penalty;

        // 等待时间奖励
        let avg_wait_time = (player1.timestamp.elapsed().as_secs_f32() + 
                           player2.timestamp.elapsed().as_secs_f32()) / 2.0;
        let wait_bonus = (avg_wait_time / 300.0) * self.config.wait_time_weight;
        quality += wait_bonus.min(0.3);

        // 地区奖励
        if self.get_player_region(player1) == self.get_player_region(player2) {
            quality += self.config.region_bonus;
        }

        // 语言奖励
        if player1.preferences.language == player2.preferences.language {
            quality += self.config.language_bonus;
        }

        quality.clamp(0.0, 1.0)
    }

    // 选择最佳服务器
    fn select_best_server(&self, players: &[&MatchmakingRequest]) -> GameResult<GameServer> {
        let mut best_server = None;
        let mut best_score = f32::MIN;

        for server in self.servers.values() {
            if server.status != ServerStatus::Online {
                continue;
            }

            let mut score = 0.0f32;
            let mut total_ping = 0u32;

            // 计算所有玩家到这个服务器的总延迟
            for player in players {
                let ping = self.estimate_ping(player, server);
                total_ping += ping;
                
                // 延迟惩罚
                score -= (ping as f32 / 300.0) * self.config.ping_weight;
            }

            // 服务器负载惩罚
            let load_ratio = server.current_load as f32 / server.capacity as f32;
            score -= load_ratio * 0.5;

            // 地区匹配奖励
            let player_regions: Vec<String> = players.iter()
                .map(|p| self.get_player_region(p))
                .collect();
                
            if player_regions.iter().any(|region| region == &server.region) {
                score += self.config.region_bonus;
            }

            if score > best_score {
                best_score = score;
                best_server = Some(server.clone());
            }
        }

        best_server.ok_or_else(|| GameError::Matchmaking("没有可用的服务器".to_string()))
    }

    // 估算延迟
    fn estimate_ping(&self, player: &MatchmakingRequest, server: &GameServer) -> u32 {
        let player_region = self.get_player_region(player);
        server.average_ping.get(&player_region).copied().unwrap_or(100)
    }

    // 获取玩家地区
    fn get_player_region(&self, player: &MatchmakingRequest) -> String {
        player.preferences.preferred_regions
            .first()
            .unwrap_or(&"unknown".to_string())
            .clone()
    }

    // 获取扩展评级范围
    fn get_expanded_rating_range(&self, player: &MatchmakingRequest) -> u32 {
        let base_range = player.preferences.max_rating_difference;
        let expansion = (player.timestamp.elapsed().as_secs_f32() * self.config.rating_expansion_rate) as u32;
        base_range + expansion
    }

    // 评级转等级
    fn rating_to_tier(rating: u32) -> PlayerTier {
        match rating {
            0..=999 => PlayerTier::Beginner,
            1000..=1199 => PlayerTier::Bronze,
            1200..=1399 => PlayerTier::Silver,
            1400..=1599 => PlayerTier::Gold,
            1600..=1799 => PlayerTier::Platinum,
            1800..=1999 => PlayerTier::Diamond,
            2000..=2199 => PlayerTier::Master,
            _ => PlayerTier::Grandmaster,
        }
    }

    // 清理过期搜索
    fn cleanup_expired_searches(&mut self) -> GameResult<()> {
        let expired_players: Vec<Uuid> = self.active_searches
            .iter()
            .filter_map(|(player_id, state)| {
                if *state == MatchmakingState::Searching {
                    // 检查是否超时
                    // 这里需要从队列中获取请求时间
                    Some(*player_id)
                } else {
                    None
                }
            })
            .collect();

        for player_id in expired_players {
            self.leave_queue(player_id)?;
            self.active_searches.insert(player_id, MatchmakingState::Failed);
            
            let _ = self.event_sender.send(MatchmakingEvent::MatchFailed {
                reason: "搜索超时".to_string(),
            });
            
            self.stats.failed_matches += 1;
        }

        Ok(())
    }

    // 注册默认服务器
    fn register_default_servers(&mut self) -> GameResult<()> {
        let servers = vec![
            GameServer {
                id: "asia-east-1".to_string(),
                region: "Asia East".to_string(),
                capacity: 1000,
                current_load: 0,
                average_ping: {
                    let mut map = HashMap::new();
                    map.insert("China".to_string(), 30);
                    map.insert("Japan".to_string(), 60);
                    map.insert("Korea".to_string(), 80);
                    map
                },
                status: ServerStatus::Online,
            },
            GameServer {
                id: "us-west-1".to_string(),
                region: "US West".to_string(),
                capacity: 800,
                current_load: 0,
                average_ping: {
                    let mut map = HashMap::new();
                    map.insert("US".to_string(), 40);
                    map.insert("Canada".to_string(), 60);
                    map.insert("Mexico".to_string(), 80);
                    map
                },
                status: ServerStatus::Online,
            },
            GameServer {
                id: "eu-central-1".to_string(),
                region: "EU Central".to_string(),
                capacity: 600,
                current_load: 0,
                average_ping: {
                    let mut map = HashMap::new();
                    map.insert("Germany".to_string(), 20);
                    map.insert("France".to_string(), 40);
                    map.insert("UK".to_string(), 50);
                    map
                },
                status: ServerStatus::Online,
            },
        ];

        for server in servers {
            self.servers.insert(server.id.clone(), server);
        }

        Ok(())
    }

    // 更新统计信息
    fn update_statistics(&mut self) {
        // 更新队列大小统计
        for (mode, queue) in &self.queues {
            self.stats.queue_sizes.insert(*mode, queue.len());
        }

        // 更新并发搜索数量
        let current_searches = self.active_searches.values()
            .filter(|&&state| state == MatchmakingState::Searching)
            .count();
            
        if current_searches > self.stats.peak_concurrent_searches {
            self.stats.peak_concurrent_searches = current_searches;
        }

        // 计算平均匹配质量
        if !self.completed_matches.is_empty() {
            let total_quality: f32 = self.completed_matches.values()
                .map(|m| m.quality_score)
                .sum();
            self.stats.average_quality_score = total_quality / self.completed_matches.len() as f32;
        }
    }

    // 获取匹配状态
    pub fn get_player_state(&self, player_id: Uuid) -> Option<MatchmakingState> {
        self.active_searches.get(&player_id).copied()
    }

    // 获取队列信息
    pub fn get_queue_info(&self, game_mode: GameMode) -> Option<(usize, Duration)> {
        self.queues.get(&game_mode)
            .map(|queue| (queue.len(), queue.average_wait_time))
    }

    // 获取匹配结果
    pub fn get_match_result(&self, match_id: &str) -> Option<&MatchmakingResult> {
        self.completed_matches.get(match_id)
    }

    // 获取统计信息
    pub fn get_stats(&self) -> &MatchmakingStats {
        &self.stats
    }

    // 设置服务器状态
    pub fn set_server_status(&mut self, server_id: &str, status: ServerStatus) -> GameResult<()> {
        if let Some(server) = self.servers.get_mut(server_id) {
            server.status = status;
            info!("服务器 {} 状态更新为 {:?}", server_id, status);
        }
        Ok(())
    }

    // 更新服务器负载
    pub fn update_server_load(&mut self, server_id: &str, current_load: u32) -> GameResult<()> {
        if let Some(server) = self.servers.get_mut(server_id) {
            server.current_load = current_load;
        }
        Ok(())
    }
}

// Bevy系统实现
pub fn matchmaking_system(
    mut matchmaking_manager: ResMut<MatchmakingManager>,
) {
    let _ = matchmaking_manager.update();
}

// 便捷函数
impl MatchmakingManager {
    // 快速匹配（使用默认偏好）
    pub fn quick_match(&mut self, player_id: Uuid, game_mode: GameMode, rating: u32) -> GameResult<()> {
        self.join_queue(player_id, game_mode, rating, MatchmakingPreferences::default(), None)
    }

    // 组队匹配
    pub fn team_match(
        &mut self, 
        team_leader: Uuid,
        team_members: Vec<Uuid>,
        game_mode: GameMode,
        rating: u32
    ) -> GameResult<()> {
        let mut team = vec![team_leader];
        team.extend(team_members);
        
        self.join_queue(team_leader, game_mode, rating, MatchmakingPreferences::default(), Some(team))
    }

    // 获取估算等待时间
    pub fn get_estimated_wait_time(&self, game_mode: GameMode, rating: u32) -> Duration {
        let base_time = self.queues.get(&game_mode)
            .map(|queue| queue.average_wait_time)
            .unwrap_or(Duration::from_secs(60));

        // 根据评级调整等待时间
        let tier = Self::rating_to_tier(rating);
        match tier {
            PlayerTier::Grandmaster | PlayerTier::Master => base_time * 2, // 高手等待时间更长
            PlayerTier::Beginner => base_time / 2, // 新手更容易匹配
            _ => base_time,
        }
    }

    // 获取活跃玩家数量
    pub fn get_active_players(&self) -> usize {
        self.queues.values().map(|queue| queue.len()).sum()
    }

    // 检查是否是高峰时段
    pub fn is_peak_hours(&self) -> bool {
        let current_hour = chrono::Local::now().hour() as u8;
        self.queues.values().any(|queue| queue.peak_hours.contains(&current_hour))
    }
}