/*
 * P2P网络系统 - Peer-to-Peer Network System
 * 
 * 开发心理过程：
 * 设计去中心化的P2P网络系统，支持直连、NAT穿透、中继连接等功能
 * 需要考虑网络拓扑、连接质量、数据传输和安全性
 * 重点关注连接稳定性和数据传输效率
 */

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use tokio::net::UdpSocket;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::core::error::{GameResult, GameError};

// P2P连接状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    Disconnected,   // 未连接
    Connecting,     // 正在连接
    Connected,      // 已连接
    Reconnecting,   // 重连中
    Failed,         // 连接失败
}

// P2P节点类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Host,           // 主机节点
    Client,         // 客户端节点
    Relay,          // 中继节点
    Seed,           // 种子节点
}

// NAT类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NATType {
    Open,           // 开放型
    Moderate,       // 中等型
    Strict,         // 严格型
    Unknown,        // 未知
}

// 连接质量等级
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ConnectionQuality {
    Excellent,      // 优秀 (ping < 50ms, 丢包率 < 1%)
    Good,           // 良好 (ping < 100ms, 丢包率 < 3%)
    Fair,           // 一般 (ping < 200ms, 丢包率 < 5%)
    Poor,           // 较差 (ping > 200ms, 丢包率 > 5%)
}

// P2P消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    // 连接管理
    ConnectionRequest { sender_id: Uuid, public_addr: SocketAddr, private_addr: SocketAddr },
    ConnectionResponse { success: bool, session_id: String, relay_addr: Option<SocketAddr> },
    Disconnect { reason: String },
    
    // NAT穿透
    HolePunch { session_id: String, sequence: u32 },
    HolePunchAck { session_id: String, sequence: u32 },
    
    // 数据传输
    GameData { data: Vec<u8>, reliable: bool, sequence: u32 },
    DataAck { sequence: u32 },
    
    // 心跳和状态
    Heartbeat { timestamp: u64 },
    HeartbeatAck { timestamp: u64, ping: u32 },
    StatusUpdate { status: PeerStatus },
    
    // 中继
    RelayRequest { target_id: Uuid, data: Vec<u8> },
    RelayResponse { from_id: Uuid, data: Vec<u8> },
    
    // 错误处理
    Error { code: u32, message: String },
}

// 节点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub node_id: Uuid,
    pub node_type: NodeType,
    pub connection_count: u32,
    pub bandwidth_up: u32,
    pub bandwidth_down: u32,
    pub nat_type: NATType,
    pub relay_capable: bool,
}

// P2P连接信息
#[derive(Debug, Clone)]
pub struct P2PConnection {
    pub peer_id: Uuid,
    pub state: ConnectionState,
    pub session_id: String,
    pub direct_addr: Option<SocketAddr>,
    pub relay_addr: Option<SocketAddr>,
    pub using_relay: bool,
    pub quality: ConnectionQuality,
    pub rtt: u32,                    // 往返时延
    pub packet_loss: f32,            // 丢包率
    pub bandwidth: u32,              // 带宽 (bytes/s)
    pub connected_at: Instant,
    pub last_heartbeat: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u32,
    pub packets_received: u32,
    pub packets_lost: u32,
}

// P2P配置
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub node_id: Uuid,
    pub bind_port: u16,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub hole_punch_attempts: u32,
    pub hole_punch_timeout: Duration,
    pub relay_timeout: Duration,
    pub packet_timeout: Duration,
    pub max_packet_size: usize,
    pub enable_upnp: bool,
    pub enable_relay: bool,
    pub relay_servers: Vec<SocketAddr>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            node_id: Uuid::new_v4(),
            bind_port: 0, // 自动分配端口
            max_connections: 8,
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            hole_punch_attempts: 5,
            hole_punch_timeout: Duration::from_secs(5),
            relay_timeout: Duration::from_secs(30),
            packet_timeout: Duration::from_secs(10),
            max_packet_size: 1200, // MTU考虑
            enable_upnp: true,
            enable_relay: true,
            relay_servers: vec![
                "relay1.example.com:8080".parse().unwrap(),
                "relay2.example.com:8080".parse().unwrap(),
            ],
        }
    }
}

// 可靠传输管理
#[derive(Debug)]
pub struct ReliableTransport {
    send_buffer: HashMap<u32, (Vec<u8>, Instant)>,
    receive_buffer: HashMap<u32, Vec<u8>>,
    next_send_sequence: u32,
    expected_receive_sequence: u32,
    ack_bitfield: u64, // 用于批量确认
    retransmit_queue: VecDeque<u32>,
    max_retransmits: u32,
}

impl ReliableTransport {
    pub fn new() -> Self {
        Self {
            send_buffer: HashMap::new(),
            receive_buffer: HashMap::new(),
            next_send_sequence: 0,
            expected_receive_sequence: 0,
            ack_bitfield: 0,
            retransmit_queue: VecDeque::new(),
            max_retransmits: 5,
        }
    }

    // 发送可靠数据包
    pub fn send_reliable(&mut self, data: Vec<u8>) -> u32 {
        let sequence = self.next_send_sequence;
        self.next_send_sequence = self.next_send_sequence.wrapping_add(1);
        self.send_buffer.insert(sequence, (data, Instant::now()));
        sequence
    }

    // 处理数据包确认
    pub fn handle_ack(&mut self, sequence: u32) {
        self.send_buffer.remove(&sequence);
    }

    // 处理接收到的数据包
    pub fn handle_received(&mut self, sequence: u32, data: Vec<u8>) -> Option<Vec<u8>> {
        if sequence == self.expected_receive_sequence {
            self.expected_receive_sequence = self.expected_receive_sequence.wrapping_add(1);
            Some(data)
        } else if sequence > self.expected_receive_sequence {
            // 乱序包，存储等待
            self.receive_buffer.insert(sequence, data);
            None
        } else {
            // 重复包或过期包，忽略
            None
        }
    }

    // 检查需要重传的包
    pub fn get_retransmits(&mut self, timeout: Duration) -> Vec<(u32, Vec<u8>)> {
        let mut retransmits = Vec::new();
        let now = Instant::now();
        
        for (&sequence, (data, timestamp)) in &self.send_buffer {
            if now.duration_since(*timestamp) > timeout {
                retransmits.push((sequence, data.clone()));
            }
        }
        
        retransmits
    }
}

// STUN服务器信息
#[derive(Debug, Clone)]
pub struct STUNServer {
    pub address: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
    pub response_time: Option<Duration>,
}

// NAT穿透状态
#[derive(Debug)]
pub struct NATTraversal {
    pub external_addr: Option<SocketAddr>,
    pub internal_addr: SocketAddr,
    pub nat_type: NATType,
    pub stun_servers: Vec<STUNServer>,
    pub port_mapping: HashMap<u16, u16>, // 内部端口 -> 外部端口
}

// P2P管理器
pub struct P2PManager {
    config: P2PConfig,
    node_type: NodeType,
    
    // 网络层
    socket: Arc<UdpSocket>,
    local_addr: SocketAddr,
    external_addr: Option<SocketAddr>,
    
    // 连接管理
    connections: Arc<RwLock<HashMap<Uuid, P2PConnection>>>,
    pending_connections: HashMap<Uuid, Instant>,
    connection_attempts: HashMap<Uuid, u32>,
    
    // NAT穿透
    nat_traversal: NATTraversal,
    hole_punch_sessions: HashMap<String, HolePunchSession>,
    
    // 可靠传输
    reliable_transport: HashMap<Uuid, ReliableTransport>,
    
    // 统计信息
    statistics: P2PStatistics,
    
    // 事件处理
    event_sender: tokio::sync::mpsc::UnboundedSender<P2PEvent>,
}

// NAT穿透会话
#[derive(Debug)]
pub struct HolePunchSession {
    pub peer_id: Uuid,
    pub target_addr: SocketAddr,
    pub attempts: u32,
    pub max_attempts: u32,
    pub started_at: Instant,
    pub last_attempt: Instant,
}

// P2P事件
#[derive(Debug, Clone)]
pub enum P2PEvent {
    ConnectionEstablished { peer_id: Uuid, direct: bool },
    ConnectionLost { peer_id: Uuid, reason: String },
    DataReceived { peer_id: Uuid, data: Vec<u8> },
    NATTypeDetected { nat_type: NATType, external_addr: SocketAddr },
    RelayConnectionEstablished { relay_addr: SocketAddr },
    Error { error: String },
}

// P2P统计信息
#[derive(Debug, Default)]
pub struct P2PStatistics {
    pub total_connections: u32,
    pub active_connections: u32,
    pub direct_connections: u32,
    pub relay_connections: u32,
    pub failed_connections: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub average_rtt: f32,
    pub connection_success_rate: f32,
}

impl P2PManager {
    // 创建P2P管理器
    pub async fn new(config: P2PConfig) -> GameResult<Self> {
        // 绑定UDP套接字
        let socket = if config.bind_port == 0 {
            UdpSocket::bind("0.0.0.0:0").await
        } else {
            UdpSocket::bind(format!("0.0.0.0:{}", config.bind_port)).await
        }.map_err(|e| GameError::P2P(format!("无法绑定UDP套接字: {}", e)))?;

        let local_addr = socket.local_addr()
            .map_err(|e| GameError::P2P(format!("无法获取本地地址: {}", e)))?;

        let (event_sender, _) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            node_type: NodeType::Client,
            socket: Arc::new(socket),
            local_addr,
            external_addr: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_connections: HashMap::new(),
            connection_attempts: HashMap::new(),
            nat_traversal: NATTraversal {
                external_addr: None,
                internal_addr: local_addr,
                nat_type: NATType::Unknown,
                stun_servers: vec![
                    STUNServer {
                        address: "stun.l.google.com:19302".parse().unwrap(),
                        username: None,
                        password: None,
                        response_time: None,
                    },
                    STUNServer {
                        address: "stun1.l.google.com:19302".parse().unwrap(),
                        username: None,
                        password: None,
                        response_time: None,
                    },
                ],
                port_mapping: HashMap::new(),
            },
            hole_punch_sessions: HashMap::new(),
            reliable_transport: HashMap::new(),
            statistics: P2PStatistics::default(),
            event_sender,
            config,
        })
    }

    // 初始化P2P系统
    pub async fn initialize(&mut self) -> GameResult<()> {
        info!("初始化P2P系统...");
        
        // 检测NAT类型和外部地址
        self.detect_nat_type().await?;
        
        // 启动网络监听循环
        self.start_network_listener().await?;
        
        // 启动心跳循环
        self.start_heartbeat_loop().await;
        
        info!("P2P系统初始化完成，本地地址: {}", self.local_addr);
        if let Some(external_addr) = self.external_addr {
            info!("外部地址: {}, NAT类型: {:?}", external_addr, self.nat_traversal.nat_type);
        }
        
        Ok(())
    }

    // 连接到对等节点
    pub async fn connect_to_peer(&mut self, peer_id: Uuid, peer_addr: SocketAddr) -> GameResult<()> {
        if self.connections.read().await.contains_key(&peer_id) {
            return Err(GameError::P2P("已存在到该节点的连接".to_string()));
        }

        if self.connections.read().await.len() >= self.config.max_connections {
            return Err(GameError::P2P("连接数已达上限".to_string()));
        }

        info!("开始连接到节点 {} ({})", peer_id, peer_addr);
        
        // 记录连接尝试
        self.pending_connections.insert(peer_id, Instant::now());
        *self.connection_attempts.entry(peer_id).or_insert(0) += 1;

        // 尝试直连
        if self.attempt_direct_connection(peer_id, peer_addr).await? {
            return Ok(());
        }

        // 尝试NAT穿透
        if self.attempt_nat_traversal(peer_id, peer_addr).await? {
            return Ok(());
        }

        // 尝试中继连接
        if self.config.enable_relay {
            self.attempt_relay_connection(peer_id, peer_addr).await?;
        } else {
            return Err(GameError::P2P("无法建立连接".to_string()));
        }

        Ok(())
    }

    // 断开与节点的连接
    pub async fn disconnect_from_peer(&mut self, peer_id: Uuid, reason: String) -> GameResult<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(&peer_id) {
            // 发送断开消息
            let message = P2PMessage::Disconnect { reason: reason.clone() };
            self.send_message_to_connection(&connection, message).await?;

            // 清理相关数据
            self.reliable_transport.remove(&peer_id);
            self.pending_connections.remove(&peer_id);

            let _ = self.event_sender.send(P2PEvent::ConnectionLost {
                peer_id,
                reason,
            });

            self.statistics.active_connections = self.statistics.active_connections.saturating_sub(1);
            
            info!("断开与节点 {} 的连接", peer_id);
        }

        Ok(())
    }

    // 发送数据到节点
    pub async fn send_data(&mut self, peer_id: Uuid, data: Vec<u8>, reliable: bool) -> GameResult<()> {
        let connections = self.connections.read().await;
        let connection = connections.get(&peer_id)
            .ok_or_else(|| GameError::P2P("节点未连接".to_string()))?;

        if data.len() > self.config.max_packet_size {
            return Err(GameError::P2P("数据包过大".to_string()));
        }

        let sequence = if reliable {
            // 可靠传输
            let transport = self.reliable_transport.entry(peer_id).or_insert_with(ReliableTransport::new);
            transport.send_reliable(data.clone())
        } else {
            0 // 不可靠传输不需要序列号
        };

        let message = P2PMessage::GameData {
            data,
            reliable,
            sequence,
        };

        self.send_message_to_connection(connection, message).await?;
        self.statistics.bytes_sent += data.len() as u64;
        self.statistics.packets_sent += 1;

        Ok(())
    }

    // 广播数据到所有连接的节点
    pub async fn broadcast_data(&mut self, data: Vec<u8>, reliable: bool) -> GameResult<()> {
        let peer_ids: Vec<Uuid> = {
            let connections = self.connections.read().await;
            connections.keys().copied().collect()
        };

        for peer_id in peer_ids {
            let _ = self.send_data(peer_id, data.clone(), reliable).await;
        }

        Ok(())
    }

    // 更新P2P系统
    pub async fn update(&mut self) -> GameResult<()> {
        // 检查连接超时
        self.check_connection_timeouts().await?;
        
        // 处理可靠传输重传
        self.handle_reliable_retransmits().await?;
        
        // 清理过期的NAT穿透会话
        self.cleanup_hole_punch_sessions();
        
        // 更新连接质量
        self.update_connection_quality().await;
        
        // 更新统计信息
        self.update_statistics().await;

        Ok(())
    }

    // 私有方法

    // 检测NAT类型
    async fn detect_nat_type(&mut self) -> GameResult<()> {
        info!("检测NAT类型...");
        
        // 简化的STUN实现
        for stun_server in &self.nat_traversal.stun_servers {
            if let Ok(external_addr) = self.stun_request(stun_server.address).await {
                self.external_addr = Some(external_addr);
                self.nat_traversal.external_addr = Some(external_addr);
                
                // 简单的NAT类型判断
                self.nat_traversal.nat_type = if external_addr.ip() == self.local_addr.ip() {
                    NATType::Open
                } else {
                    NATType::Moderate // 简化判断
                };

                let _ = self.event_sender.send(P2PEvent::NATTypeDetected {
                    nat_type: self.nat_traversal.nat_type,
                    external_addr,
                });

                break;
            }
        }

        Ok(())
    }

    // STUN请求
    async fn stun_request(&self, server_addr: SocketAddr) -> GameResult<SocketAddr> {
        // 这是一个简化的STUN实现
        // 实际实现需要按照RFC 5389标准构造STUN消息
        
        let stun_message = b"STUN_REQUEST"; // 简化的消息格式
        
        match tokio::time::timeout(Duration::from_secs(5), async {
            self.socket.send_to(stun_message, server_addr).await?;
            
            let mut buffer = vec![0u8; 1024];
            let (len, addr) = self.socket.recv_from(&mut buffer).await?;
            
            // 解析STUN响应获取外部地址
            // 这里简化返回服务器地址作为示例
            Ok::<SocketAddr, std::io::Error>(server_addr)
        }).await {
            Ok(result) => result.map_err(|e| GameError::P2P(format!("STUN请求失败: {}", e))),
            Err(_) => Err(GameError::P2P("STUN请求超时".to_string())),
        }
    }

    // 启动网络监听循环
    async fn start_network_listener(&mut self) -> GameResult<()> {
        let socket = Arc::clone(&self.socket);
        let connections = Arc::clone(&self.connections);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 2048];
            
            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((len, addr)) => {
                        let data = buffer[..len].to_vec();
                        
                        if let Ok(message) = bincode::deserialize::<P2PMessage>(&data) {
                            // 处理收到的P2P消息
                            // TODO: 实现消息处理逻辑
                            debug!("收到来自 {} 的消息: {:?}", addr, message);
                        }
                    }
                    Err(e) => {
                        error!("UDP接收错误: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    // 启动心跳循环
    async fn start_heartbeat_loop(&mut self) {
        let connections = Arc::clone(&self.connections);
        let socket = Arc::clone(&self.socket);
        let interval = self.config.heartbeat_interval;

        tokio::spawn(async move {
            let mut heartbeat_interval = tokio::time::interval(interval);
            
            loop {
                heartbeat_interval.tick().await;
                
                let connections_guard = connections.read().await;
                for (peer_id, connection) in connections_guard.iter() {
                    let message = P2PMessage::Heartbeat {
                        timestamp: Self::get_timestamp(),
                    };
                    
                    if let Ok(data) = bincode::serialize(&message) {
                        let addr = connection.direct_addr.or(connection.relay_addr);
                        if let Some(addr) = addr {
                            let _ = socket.send_to(&data, addr).await;
                        }
                    }
                }
            }
        });
    }

    // 尝试直接连接
    async fn attempt_direct_connection(&mut self, peer_id: Uuid, peer_addr: SocketAddr) -> GameResult<bool> {
        let message = P2PMessage::ConnectionRequest {
            sender_id: self.config.node_id,
            public_addr: self.external_addr.unwrap_or(self.local_addr),
            private_addr: self.local_addr,
        };

        let data = bincode::serialize(&message)
            .map_err(|e| GameError::P2P(format!("消息序列化失败: {}", e)))?;

        self.socket.send_to(&data, peer_addr).await
            .map_err(|e| GameError::P2P(format!("发送连接请求失败: {}", e)))?;

        // 等待响应
        // TODO: 实现响应等待逻辑
        
        Ok(false) // 简化实现
    }

    // 尝试NAT穿透
    async fn attempt_nat_traversal(&mut self, peer_id: Uuid, peer_addr: SocketAddr) -> GameResult<bool> {
        let session_id = Uuid::new_v4().to_string();
        
        let session = HolePunchSession {
            peer_id,
            target_addr: peer_addr,
            attempts: 0,
            max_attempts: self.config.hole_punch_attempts,
            started_at: Instant::now(),
            last_attempt: Instant::now(),
        };

        self.hole_punch_sessions.insert(session_id.clone(), session);

        // 开始UDP打洞
        for sequence in 0..self.config.hole_punch_attempts {
            let message = P2PMessage::HolePunch {
                session_id: session_id.clone(),
                sequence,
            };

            let data = bincode::serialize(&message)
                .map_err(|e| GameError::P2P(format!("消息序列化失败: {}", e)))?;

            self.socket.send_to(&data, peer_addr).await
                .map_err(|e| GameError::P2P(format!("发送打洞包失败: {}", e)))?;

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // TODO: 等待打洞成功
        
        Ok(false) // 简化实现
    }

    // 尝试中继连接
    async fn attempt_relay_connection(&mut self, peer_id: Uuid, peer_addr: SocketAddr) -> GameResult<bool> {
        for relay_addr in &self.config.relay_servers {
            let message = P2PMessage::RelayRequest {
                target_id: peer_id,
                data: vec![], // 连接请求数据
            };

            let data = bincode::serialize(&message)
                .map_err(|e| GameError::P2P(format!("消息序列化失败: {}", e)))?;

            match self.socket.send_to(&data, *relay_addr).await {
                Ok(_) => {
                    info!("通过中继服务器 {} 尝试连接到 {}", relay_addr, peer_id);
                    // TODO: 处理中继响应
                    return Ok(true);
                }
                Err(e) => {
                    warn!("中继连接失败: {}", e);
                    continue;
                }
            }
        }

        Ok(false)
    }

    // 发送消息到连接
    async fn send_message_to_connection(
        &self,
        connection: &P2PConnection,
        message: P2PMessage
    ) -> GameResult<()> {
        let data = bincode::serialize(&message)
            .map_err(|e| GameError::P2P(format!("消息序列化失败: {}", e)))?;

        let addr = if connection.using_relay {
            connection.relay_addr
        } else {
            connection.direct_addr
        };

        if let Some(addr) = addr {
            self.socket.send_to(&data, addr).await
                .map_err(|e| GameError::P2P(format!("发送消息失败: {}", e)))?;
        }

        Ok(())
    }

    // 检查连接超时
    async fn check_connection_timeouts(&mut self) -> GameResult<()> {
        let now = Instant::now();
        let timeout = self.config.connection_timeout;
        
        let mut expired_connections = Vec::new();
        
        {
            let connections = self.connections.read().await;
            for (peer_id, connection) in connections.iter() {
                if now.duration_since(connection.last_heartbeat) > timeout {
                    expired_connections.push(*peer_id);
                }
            }
        }

        for peer_id in expired_connections {
            self.disconnect_from_peer(peer_id, "连接超时".to_string()).await?;
        }

        Ok(())
    }

    // 处理可靠传输重传
    async fn handle_reliable_retransmits(&mut self) -> GameResult<()> {
        let timeout = self.config.packet_timeout;
        
        for (peer_id, transport) in &mut self.reliable_transport {
            let retransmits = transport.get_retransmits(timeout);
            
            for (sequence, data) in retransmits {
                let message = P2PMessage::GameData {
                    data,
                    reliable: true,
                    sequence,
                };
                
                if let Some(connection) = self.connections.read().await.get(peer_id) {
                    self.send_message_to_connection(connection, message).await?;
                    self.statistics.packets_sent += 1;
                }
            }
        }

        Ok(())
    }

    // 清理NAT穿透会话
    fn cleanup_hole_punch_sessions(&mut self) {
        let now = Instant::now();
        let timeout = self.config.hole_punch_timeout;
        
        self.hole_punch_sessions.retain(|_, session| {
            now.duration_since(session.started_at) < timeout
        });
    }

    // 更新连接质量
    async fn update_connection_quality(&mut self) {
        let mut connections = self.connections.write().await;
        
        for connection in connections.values_mut() {
            // 根据ping和丢包率确定连接质量
            connection.quality = match (connection.rtt, connection.packet_loss) {
                (rtt, loss) if rtt < 50 && loss < 0.01 => ConnectionQuality::Excellent,
                (rtt, loss) if rtt < 100 && loss < 0.03 => ConnectionQuality::Good,
                (rtt, loss) if rtt < 200 && loss < 0.05 => ConnectionQuality::Fair,
                _ => ConnectionQuality::Poor,
            };
        }
    }

    // 更新统计信息
    async fn update_statistics(&mut self) {
        let connections = self.connections.read().await;
        
        self.statistics.active_connections = connections.len() as u32;
        self.statistics.direct_connections = connections.values()
            .filter(|c| !c.using_relay)
            .count() as u32;
        self.statistics.relay_connections = connections.values()
            .filter(|c| c.using_relay)
            .count() as u32;

        // 计算平均RTT
        let total_rtt: u32 = connections.values().map(|c| c.rtt).sum();
        self.statistics.average_rtt = if connections.is_empty() {
            0.0
        } else {
            total_rtt as f32 / connections.len() as f32
        };

        // 计算连接成功率
        if self.statistics.total_connections > 0 {
            self.statistics.connection_success_rate = 
                (self.statistics.total_connections - self.statistics.failed_connections) as f32 /
                self.statistics.total_connections as f32;
        }
    }

    // 获取时间戳
    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    // 公共接口方法

    // 获取连接信息
    pub async fn get_connection(&self, peer_id: Uuid) -> Option<P2PConnection> {
        let connections = self.connections.read().await;
        connections.get(&peer_id).cloned()
    }

    // 获取所有连接
    pub async fn get_all_connections(&self) -> Vec<P2PConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    // 获取统计信息
    pub fn get_statistics(&self) -> &P2PStatistics {
        &self.statistics
    }

    // 获取本地地址
    pub fn get_local_address(&self) -> SocketAddr {
        self.local_addr
    }

    // 获取外部地址
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        self.external_addr
    }

    // 获取NAT类型
    pub fn get_nat_type(&self) -> NATType {
        self.nat_traversal.nat_type
    }

    // 设置节点类型
    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
        info!("节点类型设置为: {:?}", node_type);
    }

    // 获取连接数量
    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    // 检查是否连接到节点
    pub async fn is_connected_to(&self, peer_id: Uuid) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(&peer_id)
    }
}

// Bevy系统实现
pub fn p2p_system(
    mut p2p_manager: ResMut<P2PManager>,
) {
    // P2P系统更新需要异步运行时
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let _ = p2p_manager.update().await;
    });
}

// 便捷函数
impl P2PManager {
    // 快速连接（尝试所有连接方法）
    pub async fn quick_connect(&mut self, peer_id: Uuid, peer_addr: SocketAddr) -> GameResult<()> {
        self.connect_to_peer(peer_id, peer_addr).await
    }

    // 获取最佳连接质量的节点
    pub async fn get_best_quality_peer(&self) -> Option<(Uuid, ConnectionQuality)> {
        let connections = self.connections.read().await;
        connections.iter()
            .max_by(|(_, a), (_, b)| a.quality.partial_cmp(&b.quality).unwrap())
            .map(|(id, conn)| (*id, conn.quality))
    }

    // 检查网络健康状况
    pub async fn check_network_health(&self) -> NetworkHealth {
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        
        if total_connections == 0 {
            return NetworkHealth::Disconnected;
        }

        let good_connections = connections.values()
            .filter(|c| matches!(c.quality, ConnectionQuality::Excellent | ConnectionQuality::Good))
            .count();

        let health_ratio = good_connections as f32 / total_connections as f32;

        match health_ratio {
            r if r >= 0.8 => NetworkHealth::Excellent,
            r if r >= 0.6 => NetworkHealth::Good,
            r if r >= 0.4 => NetworkHealth::Fair,
            _ => NetworkHealth::Poor,
        }
    }

    // 获取网络延迟统计
    pub async fn get_latency_stats(&self) -> LatencyStats {
        let connections = self.connections.read().await;
        let rtts: Vec<u32> = connections.values().map(|c| c.rtt).collect();
        
        if rtts.is_empty() {
            return LatencyStats::default();
        }

        let min = *rtts.iter().min().unwrap();
        let max = *rtts.iter().max().unwrap();
        let avg = rtts.iter().sum::<u32>() as f32 / rtts.len() as f32;
        
        // 计算标准差
        let variance = rtts.iter()
            .map(|&rtt| (rtt as f32 - avg).powi(2))
            .sum::<f32>() / rtts.len() as f32;
        let std_dev = variance.sqrt();

        LatencyStats {
            min,
            max,
            average: avg,
            std_deviation: std_dev,
        }
    }
}

// 网络健康状况
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkHealth {
    Excellent,      // 优秀
    Good,           // 良好
    Fair,           // 一般
    Poor,           // 较差
    Disconnected,   // 未连接
}

// 延迟统计
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    pub min: u32,
    pub max: u32,
    pub average: f32,
    pub std_deviation: f32,
}