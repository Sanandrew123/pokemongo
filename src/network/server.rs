/*
 * 网络服务器系统 - Network Server System
 * 
 * 开发心理过程：
 * 设计高性能的游戏服务器，支持多人对战、实时同步、房间管理等功能
 * 需要考虑网络延迟、数据安全、负载均衡和可扩展性
 * 重点关注玩家体验和服务器稳定性
 */

use bevy::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::core::error::{GameResult, GameError};

// 服务器状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Maintenance,
}

// 客户端状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientState {
    Connected,
    Authenticating,
    InLobby,
    InBattle,
    Disconnected,
}

// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub timeout_seconds: u64,
    pub enable_tls: bool,
    pub tick_rate: u32,
    pub max_rooms: usize,
    pub max_players_per_room: usize,
    pub auth_required: bool,
    pub maintenance_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            timeout_seconds: 30,
            enable_tls: false,
            tick_rate: 60,
            max_rooms: 100,
            max_players_per_room: 4,
            auth_required: true,
            maintenance_mode: false,
        }
    }
}

// 客户端连接信息
#[derive(Debug)]
pub struct ClientConnection {
    pub id: Uuid,
    pub address: SocketAddr,
    pub state: ClientState,
    pub stream: Arc<Mutex<TcpStream>>,
    pub player_id: Option<String>,
    pub room_id: Option<String>,
    pub last_heartbeat: std::time::Instant,
    pub stats: ConnectionStats,
}

// 连接统计
#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u32,
    pub messages_received: u32,
    pub connection_time: std::time::Duration,
    pub last_ping: u32,
}

// 游戏房间
#[derive(Debug)]
pub struct GameRoom {
    pub id: String,
    pub name: String,
    pub password: Option<String>,
    pub max_players: usize,
    pub players: Vec<Uuid>,
    pub state: RoomState,
    pub created_at: std::time::SystemTime,
    pub settings: RoomSettings,
}

// 房间状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomState {
    Waiting,
    Starting,
    InProgress,
    Finished,
}

// 房间设置
#[derive(Debug, Clone)]
pub struct RoomSettings {
    pub battle_type: BattleType,
    pub time_limit: Option<u32>,
    pub level_cap: Option<u8>,
    pub allow_spectators: bool,
    pub private_room: bool,
}

// 对战类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BattleType {
    Single,     // 1v1
    Double,     // 2v2
    Multi,      // 多人混战
    Tournament, // 锦标赛
}

// 网络消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    // 连接管理
    Connect { version: String, player_name: String },
    Disconnect,
    Heartbeat,
    
    // 认证
    Authenticate { token: String },
    AuthResult { success: bool, error: Option<String> },
    
    // 房间管理
    CreateRoom { name: String, password: Option<String>, settings: RoomSettings },
    JoinRoom { room_id: String, password: Option<String> },
    LeaveRoom,
    RoomList { rooms: Vec<RoomInfo> },
    RoomUpdate { room: RoomInfo },
    
    // 游戏同步
    PlayerAction { action: String, data: Vec<u8> },
    GameState { state: Vec<u8> },
    BattleUpdate { battle_data: Vec<u8> },
    
    // 聊天
    ChatMessage { message: String, target: Option<String> },
    
    // 错误处理
    Error { code: u32, message: String },
}

// 房间信息（用于客户端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub player_count: usize,
    pub max_players: usize,
    pub state: RoomState,
    pub has_password: bool,
    pub battle_type: BattleType,
}

// 服务器事件
#[derive(Debug, Clone)]
pub enum ServerEvent {
    ClientConnected(Uuid),
    ClientDisconnected(Uuid),
    RoomCreated(String),
    RoomDestroyed(String),
    PlayerJoinedRoom(Uuid, String),
    PlayerLeftRoom(Uuid, String),
    BattleStarted(String),
    BattleEnded(String),
    ServerShutdown,
}

// 游戏服务器主结构
pub struct GameServer {
    config: ServerConfig,
    state: ServerState,
    
    // 连接管理
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
    rooms: Arc<RwLock<HashMap<String, GameRoom>>>,
    
    // 网络层
    listener: Option<TcpListener>,
    
    // 统计信息
    stats: Arc<RwLock<ServerStats>>,
    
    // 事件处理
    event_sender: tokio::sync::mpsc::UnboundedSender<ServerEvent>,
    event_receiver: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ServerEvent>>>,
}

// 服务器统计
#[derive(Debug, Default)]
pub struct ServerStats {
    pub total_connections: u32,
    pub current_connections: u32,
    pub total_rooms: u32,
    pub active_rooms: u32,
    pub messages_processed: u64,
    pub bytes_transferred: u64,
    pub uptime: std::time::Duration,
    pub start_time: Option<std::time::SystemTime>,
}

impl GameServer {
    // 创建新的游戏服务器
    pub fn new(config: ServerConfig) -> GameResult<Self> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            state: ServerState::Stopped,
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            stats: Arc::new(RwLock::new(ServerStats::default())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        })
    }

    // 启动服务器
    pub async fn start(&mut self) -> GameResult<()> {
        if self.state != ServerState::Stopped {
            return Err(GameError::Network("服务器已在运行".to_string()));
        }

        info!("启动游戏服务器...");
        self.state = ServerState::Starting;

        // 绑定监听器
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        self.listener = Some(TcpListener::bind(&bind_addr).await
            .map_err(|e| GameError::Network(format!("无法绑定到 {}: {}", bind_addr, e)))?);

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.start_time = Some(std::time::SystemTime::now());
        }

        self.state = ServerState::Running;
        info!("服务器已启动，监听地址: {}", bind_addr);

        // 启动接受连接的循环
        self.accept_connections().await?;

        Ok(())
    }

    // 停止服务器
    pub async fn stop(&mut self) -> GameResult<()> {
        info!("停止游戏服务器...");
        self.state = ServerState::Stopping;

        // 断开所有客户端
        self.disconnect_all_clients().await?;

        // 清理资源
        self.rooms.write().await.clear();
        
        self.state = ServerState::Stopped;
        self.listener = None;

        let _ = self.event_sender.send(ServerEvent::ServerShutdown);
        info!("服务器已停止");
        Ok(())
    }

    // 接受新连接
    async fn accept_connections(&self) -> GameResult<()> {
        let listener = self.listener.as_ref()
            .ok_or_else(|| GameError::Network("监听器未初始化".to_string()))?;

        loop {
            if self.state != ServerState::Running {
                break;
            }

            match listener.accept().await {
                Ok((stream, addr)) => {
                    if self.clients.read().await.len() >= self.config.max_connections {
                        warn!("连接数已达上限，拒绝新连接: {}", addr);
                        continue;
                    }

                    info!("新客户端连接: {}", addr);
                    self.handle_new_connection(stream, addr).await?;
                }
                Err(e) => {
                    error!("接受连接失败: {}", e);
                }
            }
        }

        Ok(())
    }

    // 处理新连接
    async fn handle_new_connection(&self, stream: TcpStream, addr: SocketAddr) -> GameResult<()> {
        let client_id = Uuid::new_v4();
        
        let client = ClientConnection {
            id: client_id,
            address: addr,
            state: ClientState::Connected,
            stream: Arc::new(Mutex::new(stream)),
            player_id: None,
            room_id: None,
            last_heartbeat: std::time::Instant::now(),
            stats: ConnectionStats::default(),
        };

        // 添加到客户端列表
        self.clients.write().await.insert(client_id, client);

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_connections += 1;
            stats.current_connections += 1;
        }

        // 发送事件
        let _ = self.event_sender.send(ServerEvent::ClientConnected(client_id));

        // 启动客户端处理任务
        let clients_clone = Arc::clone(&self.clients);
        let rooms_clone = Arc::clone(&self.rooms);
        let stats_clone = Arc::clone(&self.stats);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_client(
                client_id,
                clients_clone,
                rooms_clone,
                stats_clone,
                event_sender
            ).await {
                error!("处理客户端 {} 时发生错误: {}", client_id, e);
            }
        });

        Ok(())
    }

    // 处理客户端消息
    async fn handle_client(
        client_id: Uuid,
        clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: Arc<RwLock<HashMap<String, GameRoom>>>,
        stats: Arc<RwLock<ServerStats>>,
        event_sender: tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        let mut buffer = vec![0u8; 4096];
        
        loop {
            // 检查客户端是否仍然存在
            let stream = {
                let clients_guard = clients.read().await;
                if let Some(client) = clients_guard.get(&client_id) {
                    Arc::clone(&client.stream)
                } else {
                    break;
                }
            };

            // 读取数据
            let bytes_read = {
                let mut stream_guard = stream.lock().await;
                match stream_guard.read(&mut buffer).await {
                    Ok(0) => {
                        // 连接关闭
                        info!("客户端 {} 断开连接", client_id);
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        warn!("读取客户端 {} 数据失败: {}", client_id, e);
                        break;
                    }
                }
            };

            // 解析消息
            match Self::parse_message(&buffer[..bytes_read]) {
                Ok(message) => {
                    Self::process_message(
                        client_id,
                        message,
                        &clients,
                        &rooms,
                        &stats,
                        &event_sender
                    ).await?;
                }
                Err(e) => {
                    warn!("解析消息失败: {}", e);
                    Self::send_error_to_client(client_id, &clients, 400, "消息格式错误").await?;
                }
            }

            // 更新统计
            {
                let mut stats_guard = stats.write().await;
                stats_guard.messages_processed += 1;
                stats_guard.bytes_transferred += bytes_read as u64;
            }
        }

        // 清理客户端
        Self::cleanup_client(client_id, &clients, &rooms, &event_sender).await?;
        
        Ok(())
    }

    // 解析网络消息
    fn parse_message(data: &[u8]) -> GameResult<NetworkMessage> {
        serde_json::from_slice(data)
            .map_err(|e| GameError::Network(format!("解析消息失败: {}", e)))
    }

    // 处理消息
    async fn process_message(
        client_id: Uuid,
        message: NetworkMessage,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        _stats: &Arc<RwLock<ServerStats>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        match message {
            NetworkMessage::Connect { version, player_name } => {
                Self::handle_connect(client_id, version, player_name, clients).await?;
            }
            NetworkMessage::Disconnect => {
                Self::handle_disconnect(client_id, clients, rooms, event_sender).await?;
            }
            NetworkMessage::Heartbeat => {
                Self::handle_heartbeat(client_id, clients).await?;
            }
            NetworkMessage::CreateRoom { name, password, settings } => {
                Self::handle_create_room(client_id, name, password, settings, clients, rooms, event_sender).await?;
            }
            NetworkMessage::JoinRoom { room_id, password } => {
                Self::handle_join_room(client_id, room_id, password, clients, rooms, event_sender).await?;
            }
            NetworkMessage::LeaveRoom => {
                Self::handle_leave_room(client_id, clients, rooms, event_sender).await?;
            }
            NetworkMessage::PlayerAction { action, data } => {
                Self::handle_player_action(client_id, action, data, clients, rooms).await?;
            }
            NetworkMessage::ChatMessage { message, target } => {
                Self::handle_chat_message(client_id, message, target, clients, rooms).await?;
            }
            _ => {
                warn!("未处理的消息类型");
            }
        }

        Ok(())
    }

    // 处理连接请求
    async fn handle_connect(
        client_id: Uuid,
        _version: String,
        player_name: String,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>
    ) -> GameResult<()> {
        
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&client_id) {
            client.state = ClientState::InLobby;
            client.player_id = Some(player_name.clone());
            
            info!("玩家 {} 已连接", player_name);
            
            // 发送房间列表
            let response = NetworkMessage::RoomList { rooms: vec![] };
            Self::send_message_to_client_direct(client, response).await?;
        }

        Ok(())
    }

    // 处理断开连接
    async fn handle_disconnect(
        client_id: Uuid,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        Self::cleanup_client(client_id, clients, rooms, event_sender).await
    }

    // 处理心跳
    async fn handle_heartbeat(
        client_id: Uuid,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>
    ) -> GameResult<()> {
        
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&client_id) {
            client.last_heartbeat = std::time::Instant::now();
        }

        Ok(())
    }

    // 处理创建房间
    async fn handle_create_room(
        client_id: Uuid,
        name: String,
        password: Option<String>,
        settings: RoomSettings,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        let room_id = Uuid::new_v4().to_string();
        
        let room = GameRoom {
            id: room_id.clone(),
            name: name.clone(),
            password,
            max_players: settings.private_room.then(|| 2).unwrap_or(4),
            players: vec![client_id],
            state: RoomState::Waiting,
            created_at: std::time::SystemTime::now(),
            settings,
        };

        // 添加房间
        rooms.write().await.insert(room_id.clone(), room);

        // 更新客户端状态
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&client_id) {
                client.room_id = Some(room_id.clone());
            }
        }

        let _ = event_sender.send(ServerEvent::RoomCreated(room_id.clone()));
        let _ = event_sender.send(ServerEvent::PlayerJoinedRoom(client_id, room_id));

        info!("玩家 {} 创建房间: {}", client_id, name);
        Ok(())
    }

    // 处理加入房间
    async fn handle_join_room(
        client_id: Uuid,
        room_id: String,
        password: Option<String>,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        let can_join = {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                // 检查密码
                if let Some(room_password) = &room.password {
                    if password.as_ref() != Some(room_password) {
                        return Self::send_error_to_client(
                            client_id, 
                            clients, 
                            401, 
                            "密码错误"
                        ).await;
                    }
                }

                // 检查房间容量
                if room.players.len() >= room.max_players {
                    return Self::send_error_to_client(
                        client_id,
                        clients,
                        403,
                        "房间已满"
                    ).await;
                }

                true
            } else {
                return Self::send_error_to_client(
                    client_id,
                    clients,
                    404,
                    "房间不存在"
                ).await;
            }
        };

        if can_join {
            // 添加玩家到房间
            {
                let mut rooms_guard = rooms.write().await;
                if let Some(room) = rooms_guard.get_mut(&room_id) {
                    room.players.push(client_id);
                }
            }

            // 更新客户端状态
            {
                let mut clients_guard = clients.write().await;
                if let Some(client) = clients_guard.get_mut(&client_id) {
                    client.room_id = Some(room_id.clone());
                }
            }

            let _ = event_sender.send(ServerEvent::PlayerJoinedRoom(client_id, room_id));
            info!("玩家 {} 加入房间", client_id);
        }

        Ok(())
    }

    // 处理离开房间
    async fn handle_leave_room(
        client_id: Uuid,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        let room_id = {
            let clients_guard = clients.read().await;
            clients_guard.get(&client_id).and_then(|c| c.room_id.clone())
        };

        if let Some(room_id) = room_id {
            // 从房间移除玩家
            {
                let mut rooms_guard = rooms.write().await;
                if let Some(room) = rooms_guard.get_mut(&room_id) {
                    room.players.retain(|&id| id != client_id);
                    
                    // 如果房间为空，删除房间
                    if room.players.is_empty() {
                        rooms_guard.remove(&room_id);
                        let _ = event_sender.send(ServerEvent::RoomDestroyed(room_id.clone()));
                    }
                }
            }

            // 更新客户端状态
            {
                let mut clients_guard = clients.write().await;
                if let Some(client) = clients_guard.get_mut(&client_id) {
                    client.room_id = None;
                }
            }

            let _ = event_sender.send(ServerEvent::PlayerLeftRoom(client_id, room_id));
            info!("玩家 {} 离开房间", client_id);
        }

        Ok(())
    }

    // 处理玩家动作
    async fn handle_player_action(
        client_id: Uuid,
        _action: String,
        _data: Vec<u8>,
        _clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        _rooms: &Arc<RwLock<HashMap<String, GameRoom>>>
    ) -> GameResult<()> {
        
        // TODO: 实现具体的游戏逻辑
        debug!("处理玩家 {} 的动作", client_id);
        Ok(())
    }

    // 处理聊天消息
    async fn handle_chat_message(
        client_id: Uuid,
        message: String,
        _target: Option<String>,
        _clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        _rooms: &Arc<RwLock<HashMap<String, GameRoom>>>
    ) -> GameResult<()> {
        
        info!("玩家 {} 发送消息: {}", client_id, message);
        // TODO: 实现聊天功能
        Ok(())
    }

    // 发送消息给客户端
    async fn send_message_to_client_direct(
        client: &mut ClientConnection,
        message: NetworkMessage
    ) -> GameResult<()> {
        
        let data = serde_json::to_vec(&message)
            .map_err(|e| GameError::Network(format!("序列化消息失败: {}", e)))?;

        let mut stream = client.stream.lock().await;
        stream.write_all(&data).await
            .map_err(|e| GameError::Network(format!("发送消息失败: {}", e)))?;

        client.stats.bytes_sent += data.len() as u64;
        client.stats.messages_sent += 1;

        Ok(())
    }

    // 发送错误消息给客户端
    async fn send_error_to_client(
        client_id: Uuid,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        code: u32,
        message: &str
    ) -> GameResult<()> {
        
        let error_msg = NetworkMessage::Error {
            code,
            message: message.to_string(),
        };

        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&client_id) {
            Self::send_message_to_client_direct(client, error_msg).await?;
        }

        Ok(())
    }

    // 清理客户端
    async fn cleanup_client(
        client_id: Uuid,
        clients: &Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
        rooms: &Arc<RwLock<HashMap<String, GameRoom>>>,
        event_sender: &tokio::sync::mpsc::UnboundedSender<ServerEvent>
    ) -> GameResult<()> {
        
        // 从房间移除
        Self::handle_leave_room(client_id, clients, rooms, event_sender).await?;

        // 从客户端列表移除
        clients.write().await.remove(&client_id);

        let _ = event_sender.send(ServerEvent::ClientDisconnected(client_id));
        info!("客户端 {} 已清理", client_id);

        Ok(())
    }

    // 断开所有客户端
    async fn disconnect_all_clients(&self) -> GameResult<()> {
        let client_ids: Vec<Uuid> = {
            self.clients.read().await.keys().copied().collect()
        };

        for client_id in client_ids {
            Self::cleanup_client(
                client_id,
                &self.clients,
                &self.rooms,
                &self.event_sender
            ).await?;
        }

        Ok(())
    }

    // 获取服务器统计
    pub async fn get_stats(&self) -> ServerStats {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        
        if let Some(start_time) = result.start_time {
            result.uptime = std::time::SystemTime::now()
                .duration_since(start_time)
                .unwrap_or_default();
        }
        
        result.current_connections = self.clients.read().await.len() as u32;
        result.active_rooms = self.rooms.read().await.len() as u32;
        
        result
    }

    // 获取房间列表
    pub async fn get_room_list(&self) -> Vec<RoomInfo> {
        let rooms_guard = self.rooms.read().await;
        rooms_guard.values().map(|room| RoomInfo {
            id: room.id.clone(),
            name: room.name.clone(),
            player_count: room.players.len(),
            max_players: room.max_players,
            state: room.state,
            has_password: room.password.is_some(),
            battle_type: room.settings.battle_type,
        }).collect()
    }

    // 广播消息到房间
    pub async fn broadcast_to_room(&self, room_id: &str, message: NetworkMessage) -> GameResult<()> {
        let player_ids = {
            let rooms_guard = self.rooms.read().await;
            if let Some(room) = rooms_guard.get(room_id) {
                room.players.clone()
            } else {
                return Ok(());
            }
        };

        let mut clients_guard = self.clients.write().await;
        for player_id in player_ids {
            if let Some(client) = clients_guard.get_mut(&player_id) {
                let _ = Self::send_message_to_client_direct(client, message.clone()).await;
            }
        }

        Ok(())
    }

    // 设置维护模式
    pub fn set_maintenance_mode(&mut self, enabled: bool) {
        self.config.maintenance_mode = enabled;
        self.state = if enabled {
            ServerState::Maintenance
        } else {
            ServerState::Running
        };
        
        info!("维护模式: {}", if enabled { "开启" } else { "关闭" });
    }

    // 获取服务器状态
    pub fn get_state(&self) -> ServerState {
        self.state
    }

    // 处理服务器事件
    pub async fn process_events(&self) -> Vec<ServerEvent> {
        let mut events = Vec::new();
        let mut receiver = self.event_receiver.lock().await;
        
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        
        events
    }
}

// Bevy系统实现
pub fn network_server_system(
    mut server: ResMut<GameServer>,
) {
    // 处理服务器事件
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let events = server.process_events().await;
        for event in events {
            match event {
                ServerEvent::ClientConnected(id) => {
                    debug!("客户端连接事件: {}", id);
                }
                ServerEvent::ClientDisconnected(id) => {
                    debug!("客户端断开事件: {}", id);
                }
                ServerEvent::BattleStarted(room_id) => {
                    info!("战斗开始: {}", room_id);
                }
                ServerEvent::BattleEnded(room_id) => {
                    info!("战斗结束: {}", room_id);
                }
                _ => {}
            }
        }
    });
}

// 便捷函数
impl GameServer {
    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub async fn get_room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    pub async fn is_room_exists(&self, room_id: &str) -> bool {
        self.rooms.read().await.contains_key(room_id)
    }

    pub async fn get_room_players(&self, room_id: &str) -> Vec<Uuid> {
        let rooms_guard = self.rooms.read().await;
        rooms_guard.get(room_id)
            .map(|room| room.players.clone())
            .unwrap_or_default()
    }

    pub async fn kick_player(&self, client_id: Uuid) -> GameResult<()> {
        Self::cleanup_client(
            client_id,
            &self.clients,
            &self.rooms,
            &self.event_sender
        ).await
    }
}