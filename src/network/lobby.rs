/*
 * 大厅系统 - Lobby System
 * 
 * 开发心理过程：
 * 设计完善的游戏大厅系统，支持房间管理、玩家交互、观战功能等
 * 需要考虑用户体验、实时通信、状态同步和社交功能
 * 重点关注功能完整性和用户友好性
 */

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::core::error::{GameResult, GameError};

// 大厅状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LobbyState {
    Online,
    Maintenance,
    Offline,
}

// 房间状态
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,        // 等待玩家
    Starting,       // 准备开始
    InProgress,     // 进行中
    Finished,       // 已结束
    Cancelled,      // 已取消
}

// 玩家状态
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerStatus {
    Online,
    Away,
    Busy,
    InGame,
    Offline,
}

// 房间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    Public,         // 公开房间
    Private,        // 私人房间
    Ranked,         // 排位房间
    Tournament,     // 锦标赛房间
    Custom,         // 自定义房间
    Training,       // 训练房间
}

// 权限等级
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PermissionLevel {
    Guest = 0,      // 客人
    Member = 1,     // 成员
    Moderator = 2,  // 管理员
    Admin = 3,      // 管理员
    Owner = 4,      // 房主
}

// 玩家信息
#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub id: Uuid,
    pub name: String,
    pub level: u32,
    pub rating: u32,
    pub title: Option<String>,
    pub avatar: String,
    pub status: PlayerStatus,
    pub current_room: Option<String>,
    pub permissions: PermissionLevel,
    pub muted_until: Option<Instant>,
    pub banned_until: Option<Instant>,
    pub join_time: Instant,
    pub last_activity: Instant,
    pub statistics: PlayerStatistics,
}

// 玩家统计
#[derive(Debug, Clone, Default)]
pub struct PlayerStatistics {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_games: u32,
    pub win_streak: u32,
    pub best_win_streak: u32,
    pub favorite_pokemon: Vec<u16>,
    pub playtime_hours: u32,
}

// 房间信息
#[derive(Debug, Clone)]
pub struct LobbyRoom {
    pub id: String,
    pub name: String,
    pub description: String,
    pub room_type: RoomType,
    pub state: RoomState,
    pub owner_id: Uuid,
    pub password: Option<String>,
    pub max_players: u8,
    pub max_spectators: u8,
    pub settings: RoomSettings,
    pub players: HashMap<Uuid, RoomPlayerInfo>,
    pub spectators: Vec<Uuid>,
    pub moderators: HashSet<Uuid>,
    pub banned_players: HashSet<Uuid>,
    pub created_at: Instant,
    pub started_at: Option<Instant>,
    pub last_activity: Instant,
    pub chat_history: VecDeque<ChatMessage>,
}

// 房间设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub game_mode: GameMode,
    pub time_limit: Option<u32>,        // 每回合时间限制
    pub total_time_limit: Option<u32>,  // 总游戏时间限制
    pub level_cap: Option<u8>,          // 等级限制
    pub battle_format: BattleFormat,
    pub allow_spectators: bool,
    pub spectator_chat: bool,
    pub password_protected: bool,
    pub auto_start: bool,
    pub region_lock: Option<String>,
    pub language_filter: Option<String>,
    pub custom_rules: HashMap<String, String>,
}

// 游戏模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Single,         // 单打
    Double,         // 双打
    Multi,          // 多人
    Tournament,     // 锦标赛
    Draft,          // 轮抽
}

// 对战格式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BattleFormat {
    OU,             // 标准
    Ubers,          // 超级
    UU,             // 次级
    RU,             // 稀有级
    NU,             // 未使用级
    LC,             // 小杯
    Monotype,       // 单属性
    Random,         // 随机
    Custom,         // 自定义
}

// 房间玩家信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPlayerInfo {
    pub player_id: Uuid,
    pub team_id: Option<u8>,
    pub ready: bool,
    pub joined_at: Instant,
    pub ping: u32,
    pub spectator: bool,
}

// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub timestamp: u64,
    pub message_type: ChatMessageType,
    pub target_id: Option<Uuid>, // 私聊目标
    pub metadata: HashMap<String, String>,
}

// 聊天消息类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChatMessageType {
    Public,         // 公开消息
    Team,           // 队伍消息
    Private,        // 私人消息
    System,         // 系统消息
    Moderator,      // 管理员消息
    Emote,          // 表情
    Notification,   // 通知
}

// 大厅事件
#[derive(Debug, Clone)]
pub enum LobbyEvent {
    PlayerJoined { player_id: Uuid, room_id: Option<String> },
    PlayerLeft { player_id: Uuid, room_id: Option<String> },
    RoomCreated { room_id: String, owner_id: Uuid },
    RoomUpdated { room_id: String },
    RoomDestroyed { room_id: String },
    ChatMessage { room_id: Option<String>, message: ChatMessage },
    PlayerStatusChanged { player_id: Uuid, status: PlayerStatus },
    PlayerKicked { player_id: Uuid, room_id: String, reason: String },
    PlayerBanned { player_id: Uuid, room_id: String, duration: Duration },
    GameStarted { room_id: String, players: Vec<Uuid> },
    GameEnded { room_id: String, result: GameResult },
}

// 游戏结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner_id: Option<Uuid>,
    pub loser_id: Option<Uuid>,
    pub result_type: GameResultType,
    pub duration: Duration,
    pub statistics: HashMap<Uuid, GameStats>,
}

// 游戏结果类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameResultType {
    Victory,
    Defeat,
    Draw,
    Forfeit,
    Timeout,
    Disconnect,
}

// 游戏统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStats {
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub pokemon_ko: u8,
    pub pokemon_lost: u8,
    pub turns_played: u16,
    pub items_used: u8,
    pub critical_hits: u8,
}

// 大厅配置
#[derive(Debug, Clone)]
pub struct LobbyConfig {
    pub max_players: usize,
    pub max_rooms: usize,
    pub max_room_players: u8,
    pub max_room_spectators: u8,
    pub room_timeout: Duration,
    pub player_timeout: Duration,
    pub chat_history_limit: usize,
    pub message_rate_limit: u32,     // 每分钟最大消息数
    pub auto_kick_inactive: bool,
    pub enable_spectator_mode: bool,
    pub enable_private_rooms: bool,
    pub enable_tournaments: bool,
}

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            max_players: 10000,
            max_rooms: 1000,
            max_room_players: 8,
            max_room_spectators: 20,
            room_timeout: Duration::from_secs(1800), // 30分钟
            player_timeout: Duration::from_secs(300), // 5分钟
            chat_history_limit: 100,
            message_rate_limit: 30,
            auto_kick_inactive: true,
            enable_spectator_mode: true,
            enable_private_rooms: true,
            enable_tournaments: true,
        }
    }
}

// 大厅管理器
pub struct LobbyManager {
    config: LobbyConfig,
    state: LobbyState,
    
    // 玩家和房间管理
    players: HashMap<Uuid, LobbyPlayer>,
    rooms: HashMap<String, LobbyRoom>,
    player_connections: HashMap<Uuid, Instant>,
    
    // 聊天和消息系统
    global_chat: VecDeque<ChatMessage>,
    message_rate_tracker: HashMap<Uuid, VecDeque<Instant>>,
    
    // 统计和监控
    statistics: LobbyStatistics,
    
    // 事件处理
    event_sender: tokio::sync::mpsc::UnboundedSender<LobbyEvent>,
}

// 大厅统计
#[derive(Debug, Default)]
pub struct LobbyStatistics {
    pub total_players: usize,
    pub online_players: usize,
    pub total_rooms: usize,
    pub active_rooms: usize,
    pub total_messages: u64,
    pub total_games: u64,
    pub concurrent_games: usize,
    pub average_room_duration: Duration,
    pub most_popular_mode: Option<GameMode>,
}

impl LobbyManager {
    // 创建大厅管理器
    pub fn new(config: LobbyConfig) -> GameResult<Self> {
        let (event_sender, _) = tokio::sync::mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            state: LobbyState::Online,
            players: HashMap::new(),
            rooms: HashMap::new(),
            player_connections: HashMap::new(),
            global_chat: VecDeque::new(),
            message_rate_tracker: HashMap::new(),
            statistics: LobbyStatistics::default(),
            event_sender,
        })
    }

    // 初始化大厅系统
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化大厅系统...");
        
        self.state = LobbyState::Online;
        
        info!("大厅系统初始化完成");
        Ok(())
    }

    // 玩家加入大厅
    pub fn join_lobby(&mut self, player: LobbyPlayer) -> GameResult<()> {
        if self.players.len() >= self.config.max_players {
            return Err(GameError::Lobby("大厅已满".to_string()));
        }

        let player_id = player.id;
        
        self.players.insert(player_id, player);
        self.player_connections.insert(player_id, Instant::now());

        self.statistics.total_players += 1;
        self.statistics.online_players += 1;

        // 发送欢迎消息
        self.send_system_message(
            None,
            &format!("欢迎 {} 加入大厅！", 
                    self.players.get(&player_id).unwrap().name)
        );

        let _ = self.event_sender.send(LobbyEvent::PlayerJoined {
            player_id,
            room_id: None,
        });

        info!("玩家 {} 加入大厅", player_id);
        Ok(())
    }

    // 玩家离开大厅
    pub fn leave_lobby(&mut self, player_id: Uuid) -> GameResult<()> {
        // 如果玩家在房间中，先让其离开房间
        if let Some(player) = self.players.get(&player_id) {
            if let Some(room_id) = &player.current_room {
                self.leave_room(player_id, room_id.clone())?;
            }
        }

        if let Some(player) = self.players.remove(&player_id) {
            self.player_connections.remove(&player_id);
            self.message_rate_tracker.remove(&player_id);
            
            self.statistics.online_players = self.statistics.online_players.saturating_sub(1);

            let _ = self.event_sender.send(LobbyEvent::PlayerLeft {
                player_id,
                room_id: None,
            });

            info!("玩家 {} 离开大厅", player.name);
        }

        Ok(())
    }

    // 创建房间
    pub fn create_room(
        &mut self,
        owner_id: Uuid,
        name: String,
        room_type: RoomType,
        settings: RoomSettings,
        password: Option<String>
    ) -> GameResult<String> {
        
        if self.rooms.len() >= self.config.max_rooms {
            return Err(GameError::Lobby("房间数量已达上限".to_string()));
        }

        let room_id = Uuid::new_v4().to_string();
        
        let room = LobbyRoom {
            id: room_id.clone(),
            name,
            description: String::new(),
            room_type,
            state: RoomState::Waiting,
            owner_id,
            password,
            max_players: self.config.max_room_players,
            max_spectators: self.config.max_room_spectators,
            settings,
            players: HashMap::new(),
            spectators: Vec::new(),
            moderators: HashSet::new(),
            banned_players: HashSet::new(),
            created_at: Instant::now(),
            started_at: None,
            last_activity: Instant::now(),
            chat_history: VecDeque::new(),
        };

        self.rooms.insert(room_id.clone(), room);
        
        // 让房主自动加入房间
        self.join_room(owner_id, room_id.clone(), password)?;

        self.statistics.total_rooms += 1;
        self.statistics.active_rooms += 1;

        let _ = self.event_sender.send(LobbyEvent::RoomCreated {
            room_id: room_id.clone(),
            owner_id,
        });

        info!("创建房间: {} (房主: {})", room_id, owner_id);
        Ok(room_id)
    }

    // 加入房间
    pub fn join_room(&mut self, player_id: Uuid, room_id: String, password: Option<String>) -> GameResult<()> {
        // 验证玩家存在
        if !self.players.contains_key(&player_id) {
            return Err(GameError::Lobby("玩家不存在".to_string()));
        }

        // 验证房间存在
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| GameError::Lobby("房间不存在".to_string()))?;

        // 检查房间状态
        if room.state == RoomState::InProgress {
            return Err(GameError::Lobby("游戏正在进行中".to_string()));
        }

        // 检查是否被禁止
        if room.banned_players.contains(&player_id) {
            return Err(GameError::Lobby("您已被此房间禁止进入".to_string()));
        }

        // 验证密码
        if let Some(room_password) = &room.password {
            if password.as_ref() != Some(room_password) {
                return Err(GameError::Lobby("密码错误".to_string()));
            }
        }

        // 检查房间容量
        if room.players.len() >= room.max_players as usize {
            return Err(GameError::Lobby("房间已满".to_string()));
        }

        // 添加玩家到房间
        let player_info = RoomPlayerInfo {
            player_id,
            team_id: None,
            ready: false,
            joined_at: Instant::now(),
            ping: 0, // TODO: 获取实际ping值
            spectator: false,
        };

        room.players.insert(player_id, player_info);
        room.last_activity = Instant::now();

        // 更新玩家状态
        if let Some(player) = self.players.get_mut(&player_id) {
            player.current_room = Some(room_id.clone());
        }

        // 发送房间消息
        self.send_room_message(
            &room_id,
            ChatMessage {
                id: Uuid::new_v4().to_string(),
                sender_id: Uuid::nil(),
                sender_name: "系统".to_string(),
                content: format!("{} 加入了房间", 
                    self.players.get(&player_id).unwrap().name),
                timestamp: Self::get_timestamp(),
                message_type: ChatMessageType::System,
                target_id: None,
                metadata: HashMap::new(),
            }
        )?;

        let _ = self.event_sender.send(LobbyEvent::PlayerJoined {
            player_id,
            room_id: Some(room_id.clone()),
        });

        info!("玩家 {} 加入房间 {}", player_id, room_id);
        Ok(())
    }

    // 离开房间
    pub fn leave_room(&mut self, player_id: Uuid, room_id: String) -> GameResult<()> {
        let should_destroy_room = {
            if let Some(room) = self.rooms.get_mut(&room_id) {
                room.players.remove(&player_id);
                room.spectators.retain(|&id| id != player_id);
                room.last_activity = Instant::now();

                // 如果房主离开了，转移房主权限
                if room.owner_id == player_id && !room.players.is_empty() {
                    let new_owner = *room.players.keys().next().unwrap();
                    room.owner_id = new_owner;
                    
                    self.send_room_message(
                        &room_id,
                        ChatMessage {
                            id: Uuid::new_v4().to_string(),
                            sender_id: Uuid::nil(),
                            sender_name: "系统".to_string(),
                            content: format!("{} 成为了新房主", 
                                self.players.get(&new_owner).map(|p| &p.name).unwrap_or(&"未知玩家".to_string())),
                            timestamp: Self::get_timestamp(),
                            message_type: ChatMessageType::System,
                            target_id: None,
                            metadata: HashMap::new(),
                        }
                    )?;
                }

                // 发送离开消息
                if let Some(player_name) = self.players.get(&player_id).map(|p| p.name.clone()) {
                    self.send_room_message(
                        &room_id,
                        ChatMessage {
                            id: Uuid::new_v4().to_string(),
                            sender_id: Uuid::nil(),
                            sender_name: "系统".to_string(),
                            content: format!("{} 离开了房间", player_name),
                            timestamp: Self::get_timestamp(),
                            message_type: ChatMessageType::System,
                            target_id: None,
                            metadata: HashMap::new(),
                        }
                    )?;
                }

                // 检查是否需要销毁房间
                room.players.is_empty() && room.spectators.is_empty()
            } else {
                false
            }
        };

        // 更新玩家状态
        if let Some(player) = self.players.get_mut(&player_id) {
            player.current_room = None;
        }

        if should_destroy_room {
            self.destroy_room(room_id.clone())?;
        }

        let _ = self.event_sender.send(LobbyEvent::PlayerLeft {
            player_id,
            room_id: Some(room_id.clone()),
        });

        info!("玩家 {} 离开房间 {}", player_id, room_id);
        Ok(())
    }

    // 销毁房间
    pub fn destroy_room(&mut self, room_id: String) -> GameResult<()> {
        if let Some(room) = self.rooms.remove(&room_id) {
            // 让所有玩家离开房间
            for &player_id in room.players.keys() {
                if let Some(player) = self.players.get_mut(&player_id) {
                    player.current_room = None;
                }
            }

            self.statistics.active_rooms = self.statistics.active_rooms.saturating_sub(1);

            let _ = self.event_sender.send(LobbyEvent::RoomDestroyed {
                room_id: room_id.clone(),
            });

            info!("销毁房间: {}", room_id);
        }

        Ok(())
    }

    // 发送聊天消息
    pub fn send_chat_message(&mut self, message: ChatMessage) -> GameResult<()> {
        // 检查消息发送频率限制
        if !self.check_message_rate_limit(message.sender_id) {
            return Err(GameError::Lobby("消息发送过于频繁".to_string()));
        }

        // 检查玩家是否被禁言
        if let Some(player) = self.players.get(&message.sender_id) {
            if let Some(muted_until) = player.muted_until {
                if Instant::now() < muted_until {
                    return Err(GameError::Lobby("您已被禁言".to_string()));
                }
            }
        }

        match message.message_type {
            ChatMessageType::Public => {
                // 发送到玩家当前所在房间或全局聊天
                if let Some(player) = self.players.get(&message.sender_id) {
                    if let Some(room_id) = &player.current_room {
                        self.send_room_message(room_id, message)?;
                    } else {
                        self.send_global_message(message)?;
                    }
                }
            }
            ChatMessageType::Private => {
                // 私聊消息
                self.send_private_message(message)?;
            }
            _ => {
                return Err(GameError::Lobby("不支持的消息类型".to_string()));
            }
        }

        self.statistics.total_messages += 1;
        Ok(())
    }

    // 发送房间消息
    fn send_room_message(&mut self, room_id: &str, message: ChatMessage) -> GameResult<()> {
        if let Some(room) = self.rooms.get_mut(room_id) {
            // 添加到聊天历史
            if room.chat_history.len() >= self.config.chat_history_limit {
                room.chat_history.pop_front();
            }
            room.chat_history.push_back(message.clone());

            let _ = self.event_sender.send(LobbyEvent::ChatMessage {
                room_id: Some(room_id.to_string()),
                message,
            });
        }

        Ok(())
    }

    // 发送全局消息
    fn send_global_message(&mut self, message: ChatMessage) -> GameResult<()> {
        // 添加到全局聊天历史
        if self.global_chat.len() >= self.config.chat_history_limit {
            self.global_chat.pop_front();
        }
        self.global_chat.push_back(message.clone());

        let _ = self.event_sender.send(LobbyEvent::ChatMessage {
            room_id: None,
            message,
        });

        Ok(())
    }

    // 发送私人消息
    fn send_private_message(&mut self, message: ChatMessage) -> GameResult<()> {
        let target_id = message.target_id
            .ok_or_else(|| GameError::Lobby("私聊消息需要指定目标".to_string()))?;

        // 验证目标玩家存在且在线
        if !self.players.contains_key(&target_id) {
            return Err(GameError::Lobby("目标玩家不存在或不在线".to_string()));
        }

        let _ = self.event_sender.send(LobbyEvent::ChatMessage {
            room_id: None,
            message,
        });

        Ok(())
    }

    // 发送系统消息
    fn send_system_message(&mut self, room_id: Option<&str>, content: &str) {
        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender_id: Uuid::nil(),
            sender_name: "系统".to_string(),
            content: content.to_string(),
            timestamp: Self::get_timestamp(),
            message_type: ChatMessageType::System,
            target_id: None,
            metadata: HashMap::new(),
        };

        if let Some(room_id) = room_id {
            let _ = self.send_room_message(room_id, message);
        } else {
            let _ = self.send_global_message(message);
        }
    }

    // 设置玩家准备状态
    pub fn set_player_ready(&mut self, player_id: Uuid, ready: bool) -> GameResult<()> {
        let room_id = self.players.get(&player_id)
            .and_then(|p| p.current_room.clone())
            .ok_or_else(|| GameError::Lobby("玩家不在房间中".to_string()))?;

        if let Some(room) = self.rooms.get_mut(&room_id) {
            if let Some(player_info) = room.players.get_mut(&player_id) {
                player_info.ready = ready;
                room.last_activity = Instant::now();

                // 检查是否所有玩家都准备好了
                if self.all_players_ready(&room_id) {
                    self.start_game(&room_id)?;
                }
            }
        }

        Ok(())
    }

    // 踢出玩家
    pub fn kick_player(
        &mut self,
        kicker_id: Uuid,
        target_id: Uuid,
        room_id: String,
        reason: String
    ) -> GameResult<()> {
        
        // 检查权限
        if !self.has_kick_permission(kicker_id, &room_id) {
            return Err(GameError::Lobby("没有踢人权限".to_string()));
        }

        self.leave_room(target_id, room_id.clone())?;

        let _ = self.event_sender.send(LobbyEvent::PlayerKicked {
            player_id: target_id,
            room_id,
            reason,
        });

        Ok(())
    }

    // 禁止玩家
    pub fn ban_player(
        &mut self,
        banner_id: Uuid,
        target_id: Uuid,
        room_id: String,
        duration: Duration
    ) -> GameResult<()> {
        
        // 检查权限
        if !self.has_ban_permission(banner_id, &room_id) {
            return Err(GameError::Lobby("没有禁止权限".to_string()));
        }

        if let Some(room) = self.rooms.get_mut(&room_id) {
            room.banned_players.insert(target_id);
            
            // 如果玩家在房间中，踢出
            if room.players.contains_key(&target_id) {
                self.leave_room(target_id, room_id.clone())?;
            }
        }

        let _ = self.event_sender.send(LobbyEvent::PlayerBanned {
            player_id: target_id,
            room_id,
            duration,
        });

        Ok(())
    }

    // 开始游戏
    fn start_game(&mut self, room_id: &str) -> GameResult<()> {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.state != RoomState::Waiting {
                return Err(GameError::Lobby("房间状态不允许开始游戏".to_string()));
            }

            room.state = RoomState::InProgress;
            room.started_at = Some(Instant::now());

            let players: Vec<Uuid> = room.players.keys().copied().collect();

            self.send_room_message(
                room_id,
                ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    sender_id: Uuid::nil(),
                    sender_name: "系统".to_string(),
                    content: "游戏开始！".to_string(),
                    timestamp: Self::get_timestamp(),
                    message_type: ChatMessageType::System,
                    target_id: None,
                    metadata: HashMap::new(),
                }
            )?;

            let _ = self.event_sender.send(LobbyEvent::GameStarted {
                room_id: room_id.to_string(),
                players: players.clone(),
            });

            self.statistics.total_games += 1;
            self.statistics.concurrent_games += 1;

            info!("房间 {} 开始游戏，玩家: {:?}", room_id, players);
        }

        Ok(())
    }

    // 结束游戏
    pub fn end_game(&mut self, room_id: String, result: GameResult) -> GameResult<()> {
        if let Some(room) = self.rooms.get_mut(&room_id) {
            room.state = RoomState::Finished;

            // 更新玩家统计
            for (player_id, stats) in &result.statistics {
                if let Some(player) = self.players.get_mut(player_id) {
                    match result.result_type {
                        GameResultType::Victory if Some(*player_id) == result.winner_id => {
                            player.statistics.wins += 1;
                            player.statistics.win_streak += 1;
                            if player.statistics.win_streak > player.statistics.best_win_streak {
                                player.statistics.best_win_streak = player.statistics.win_streak;
                            }
                        }
                        GameResultType::Defeat => {
                            player.statistics.losses += 1;
                            player.statistics.win_streak = 0;
                        }
                        GameResultType::Draw => {
                            player.statistics.draws += 1;
                        }
                        _ => {}
                    }
                    player.statistics.total_games += 1;
                }
            }

            self.statistics.concurrent_games = self.statistics.concurrent_games.saturating_sub(1);

            let _ = self.event_sender.send(LobbyEvent::GameEnded {
                room_id: room_id.clone(),
                result,
            });

            info!("房间 {} 游戏结束", room_id);
        }

        Ok(())
    }

    // 更新大厅
    pub fn update(&mut self) -> GameResult<()> {
        // 清理超时的玩家
        self.cleanup_inactive_players()?;
        
        // 清理空闲的房间
        self.cleanup_idle_rooms()?;
        
        // 更新统计信息
        self.update_statistics();

        Ok(())
    }

    // 辅助方法

    // 检查所有玩家是否准备好
    fn all_players_ready(&self, room_id: &str) -> bool {
        if let Some(room) = self.rooms.get(room_id) {
            room.players.len() >= 2 && room.players.values().all(|p| p.ready)
        } else {
            false
        }
    }

    // 检查踢人权限
    fn has_kick_permission(&self, player_id: Uuid, room_id: &str) -> bool {
        if let Some(room) = self.rooms.get(room_id) {
            room.owner_id == player_id || room.moderators.contains(&player_id) ||
            self.players.get(&player_id)
                .map(|p| p.permissions >= PermissionLevel::Moderator)
                .unwrap_or(false)
        } else {
            false
        }
    }

    // 检查禁止权限
    fn has_ban_permission(&self, player_id: Uuid, room_id: &str) -> bool {
        if let Some(room) = self.rooms.get(room_id) {
            room.owner_id == player_id ||
            self.players.get(&player_id)
                .map(|p| p.permissions >= PermissionLevel::Admin)
                .unwrap_or(false)
        } else {
            false
        }
    }

    // 检查消息频率限制
    fn check_message_rate_limit(&mut self, player_id: Uuid) -> bool {
        let now = Instant::now();
        let tracker = self.message_rate_tracker.entry(player_id).or_insert_with(VecDeque::new);
        
        // 清理一分钟前的记录
        while let Some(&front_time) = tracker.front() {
            if now.duration_since(front_time) > Duration::from_secs(60) {
                tracker.pop_front();
            } else {
                break;
            }
        }

        if tracker.len() >= self.config.message_rate_limit as usize {
            false
        } else {
            tracker.push_back(now);
            true
        }
    }

    // 清理不活跃玩家
    fn cleanup_inactive_players(&mut self) -> GameResult<()> {
        if !self.config.auto_kick_inactive {
            return Ok(());
        }

        let now = Instant::now();
        let inactive_players: Vec<Uuid> = self.players
            .iter()
            .filter(|(_, player)| {
                now.duration_since(player.last_activity) > self.config.player_timeout
            })
            .map(|(&id, _)| id)
            .collect();

        for player_id in inactive_players {
            self.leave_lobby(player_id)?;
        }

        Ok(())
    }

    // 清理空闲房间
    fn cleanup_idle_rooms(&mut self) -> GameResult<()> {
        let now = Instant::now();
        let idle_rooms: Vec<String> = self.rooms
            .iter()
            .filter(|(_, room)| {
                room.players.is_empty() && 
                now.duration_since(room.last_activity) > self.config.room_timeout
            })
            .map(|(id, _)| id.clone())
            .collect();

        for room_id in idle_rooms {
            self.destroy_room(room_id)?;
        }

        Ok(())
    }

    // 更新统计信息
    fn update_statistics(&mut self) {
        self.statistics.online_players = self.players.len();
        self.statistics.active_rooms = self.rooms.len();
        
        // 计算最受欢迎的游戏模式
        let mut mode_counts = HashMap::new();
        for room in self.rooms.values() {
            *mode_counts.entry(room.settings.game_mode).or_insert(0) += 1;
        }
        
        self.statistics.most_popular_mode = mode_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(mode, _)| mode);
    }

    // 获取时间戳
    fn get_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    // 公共接口方法

    // 获取玩家信息
    pub fn get_player(&self, player_id: Uuid) -> Option<&LobbyPlayer> {
        self.players.get(&player_id)
    }

    // 获取房间信息
    pub fn get_room(&self, room_id: &str) -> Option<&LobbyRoom> {
        self.rooms.get(room_id)
    }

    // 获取房间列表
    pub fn get_room_list(&self, room_type: Option<RoomType>) -> Vec<&LobbyRoom> {
        match room_type {
            Some(rt) => self.rooms.values().filter(|room| room.room_type == rt).collect(),
            None => self.rooms.values().collect(),
        }
    }

    // 获取在线玩家列表
    pub fn get_online_players(&self) -> Vec<&LobbyPlayer> {
        self.players.values().collect()
    }

    // 获取统计信息
    pub fn get_statistics(&self) -> &LobbyStatistics {
        &self.statistics
    }

    // 设置维护模式
    pub fn set_maintenance_mode(&mut self, enabled: bool) {
        self.state = if enabled {
            LobbyState::Maintenance
        } else {
            LobbyState::Online
        };
    }

    // 搜索房间
    pub fn search_rooms(&self, query: &str) -> Vec<&LobbyRoom> {
        self.rooms.values()
            .filter(|room| {
                room.name.contains(query) || room.description.contains(query)
            })
            .collect()
    }
}

// Bevy系统实现
pub fn lobby_system(
    mut lobby_manager: ResMut<LobbyManager>,
) {
    let _ = lobby_manager.update();
}

// 便捷函数
impl LobbyManager {
    // 创建快速匹配房间
    pub fn create_quick_match_room(&mut self, owner_id: Uuid, game_mode: GameMode) -> GameResult<String> {
        let settings = RoomSettings {
            game_mode,
            time_limit: Some(60),
            total_time_limit: Some(1800),
            level_cap: None,
            battle_format: BattleFormat::OU,
            allow_spectators: true,
            spectator_chat: true,
            password_protected: false,
            auto_start: true,
            region_lock: None,
            language_filter: None,
            custom_rules: HashMap::new(),
        };

        self.create_room(
            owner_id,
            "快速匹配".to_string(),
            RoomType::Public,
            settings,
            None
        )
    }

    // 获取玩家当前房间
    pub fn get_player_room(&self, player_id: Uuid) -> Option<&LobbyRoom> {
        self.players.get(&player_id)
            .and_then(|player| player.current_room.as_ref())
            .and_then(|room_id| self.rooms.get(room_id))
    }

    // 检查玩家是否在线
    pub fn is_player_online(&self, player_id: Uuid) -> bool {
        self.players.contains_key(&player_id)
    }

    // 获取房间聊天历史
    pub fn get_room_chat_history(&self, room_id: &str) -> Option<&VecDeque<ChatMessage>> {
        self.rooms.get(room_id).map(|room| &room.chat_history)
    }

    // 获取全局聊天历史
    pub fn get_global_chat_history(&self) -> &VecDeque<ChatMessage> {
        &self.global_chat
    }
}