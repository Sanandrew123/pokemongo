/*
 * 网络协议系统 - Network Protocol System
 * 
 * 开发心理过程：
 * 设计高效的网络通信协议，支持二进制序列化、压缩、加密和版本控制
 * 需要考虑数据包大小、网络带宽、延迟优化和协议兼容性
 * 重点关注网络性能和数据安全性
 */

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use crate::core::error::{GameResult, GameError};

// 协议版本
pub const PROTOCOL_VERSION: u16 = 1;
pub const MIN_SUPPORTED_VERSION: u16 = 1;
pub const MAX_PACKET_SIZE: usize = 8192; // 8KB

// 数据包类型
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    // 连接管理 (0x00-0x0F)
    Handshake = 0x00,
    HandshakeResponse = 0x01,
    Disconnect = 0x02,
    Heartbeat = 0x03,
    HeartbeatResponse = 0x04,
    
    // 认证 (0x10-0x1F)
    AuthRequest = 0x10,
    AuthResponse = 0x11,
    AuthChallenge = 0x12,
    
    // 房间管理 (0x20-0x2F)
    CreateRoom = 0x20,
    JoinRoom = 0x21,
    LeaveRoom = 0x22,
    RoomList = 0x23,
    RoomUpdate = 0x24,
    RoomDestroyed = 0x25,
    
    // 游戏同步 (0x30-0x3F)
    GameState = 0x30,
    PlayerAction = 0x31,
    BattleUpdate = 0x32,
    TurnData = 0x33,
    MoveSelection = 0x34,
    
    // 实时数据 (0x40-0x4F)
    PlayerPosition = 0x40,
    WorldUpdate = 0x41,
    EntityUpdate = 0x42,
    
    // 聊天通信 (0x50-0x5F)
    ChatMessage = 0x50,
    SystemMessage = 0x51,
    Whisper = 0x52,
    
    // 错误处理 (0xF0-0xFF)
    Error = 0xF0,
    Invalid = 0xFF,
}

impl TryFrom<u8> for PacketType {
    type Error = GameError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(PacketType::Handshake),
            0x01 => Ok(PacketType::HandshakeResponse),
            0x02 => Ok(PacketType::Disconnect),
            0x03 => Ok(PacketType::Heartbeat),
            0x04 => Ok(PacketType::HeartbeatResponse),
            0x10 => Ok(PacketType::AuthRequest),
            0x11 => Ok(PacketType::AuthResponse),
            0x12 => Ok(PacketType::AuthChallenge),
            0x20 => Ok(PacketType::CreateRoom),
            0x21 => Ok(PacketType::JoinRoom),
            0x22 => Ok(PacketType::LeaveRoom),
            0x23 => Ok(PacketType::RoomList),
            0x24 => Ok(PacketType::RoomUpdate),
            0x25 => Ok(PacketType::RoomDestroyed),
            0x30 => Ok(PacketType::GameState),
            0x31 => Ok(PacketType::PlayerAction),
            0x32 => Ok(PacketType::BattleUpdate),
            0x33 => Ok(PacketType::TurnData),
            0x34 => Ok(PacketType::MoveSelection),
            0x40 => Ok(PacketType::PlayerPosition),
            0x41 => Ok(PacketType::WorldUpdate),
            0x42 => Ok(PacketType::EntityUpdate),
            0x50 => Ok(PacketType::ChatMessage),
            0x51 => Ok(PacketType::SystemMessage),
            0x52 => Ok(PacketType::Whisper),
            0xF0 => Ok(PacketType::Error),
            _ => Ok(PacketType::Invalid),
        }
    }
}

// 数据包标志
bitflags::bitflags! {
    pub struct PacketFlags: u8 {
        const COMPRESSED = 0x01;
        const ENCRYPTED = 0x02;
        const RELIABLE = 0x04;
        const SEQUENCED = 0x08;
        const FRAGMENTED = 0x10;
        const PRIORITY_HIGH = 0x20;
        const PRIORITY_LOW = 0x40;
        const BROADCAST = 0x80;
    }
}

// 数据包头部
#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub packet_type: PacketType,
    pub flags: PacketFlags,
    pub sequence: u16,
    pub timestamp: u32,
    pub payload_size: u16,
    pub checksum: u32,
}

impl PacketHeader {
    pub const SIZE: usize = 12; // 固定头部大小

    pub fn new(packet_type: PacketType) -> Self {
        Self {
            packet_type,
            flags: PacketFlags::empty(),
            sequence: 0,
            timestamp: 0,
            payload_size: 0,
            checksum: 0,
        }
    }

    // 序列化头部
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::SIZE);
        
        buffer.push(self.packet_type as u8);
        buffer.push(self.flags.bits());
        buffer.extend_from_slice(&self.sequence.to_le_bytes());
        buffer.extend_from_slice(&self.timestamp.to_le_bytes());
        buffer.extend_from_slice(&self.payload_size.to_le_bytes());
        buffer.extend_from_slice(&self.checksum.to_le_bytes());
        
        buffer
    }

    // 反序列化头部
    pub fn deserialize(data: &[u8]) -> GameResult<Self> {
        if data.len() < Self::SIZE {
            return Err(GameError::Protocol("数据包头部大小不足".to_string()));
        }

        let packet_type = PacketType::try_from(data[0])?;
        let flags = PacketFlags::from_bits_truncate(data[1]);
        let sequence = u16::from_le_bytes([data[2], data[3]]);
        let timestamp = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let payload_size = u16::from_le_bytes([data[8], data[9]]);
        let checksum = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);

        Ok(Self {
            packet_type,
            flags,
            sequence,
            timestamp,
            payload_size,
            checksum,
        })
    }
}

// 完整数据包
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: PacketType, payload: Vec<u8>) -> Self {
        let mut header = PacketHeader::new(packet_type);
        header.payload_size = payload.len() as u16;
        header.timestamp = Self::get_timestamp();
        header.checksum = Self::calculate_checksum(&payload);

        Self { header, payload }
    }

    // 序列化整个数据包
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(PacketHeader::SIZE + self.payload.len());
        buffer.extend_from_slice(&self.header.serialize());
        buffer.extend_from_slice(&self.payload);
        buffer
    }

    // 反序列化数据包
    pub fn deserialize(data: &[u8]) -> GameResult<Self> {
        if data.len() < PacketHeader::SIZE {
            return Err(GameError::Protocol("数据包大小不足".to_string()));
        }

        let header = PacketHeader::deserialize(&data[..PacketHeader::SIZE])?;
        
        if data.len() < PacketHeader::SIZE + header.payload_size as usize {
            return Err(GameError::Protocol("载荷数据不完整".to_string()));
        }

        let payload_start = PacketHeader::SIZE;
        let payload_end = payload_start + header.payload_size as usize;
        let payload = data[payload_start..payload_end].to_vec();

        // 验证校验和
        let calculated_checksum = Self::calculate_checksum(&payload);
        if calculated_checksum != header.checksum {
            return Err(GameError::Protocol("校验和不匹配".to_string()));
        }

        Ok(Self { header, payload })
    }

    // 计算校验和
    fn calculate_checksum(data: &[u8]) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish() as u32
    }

    // 获取时间戳
    fn get_timestamp() -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32
    }
}

// 握手数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeData {
    pub protocol_version: u16,
    pub client_version: String,
    pub player_name: String,
    pub features: Vec<String>,
}

// 握手响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub accepted: bool,
    pub server_version: String,
    pub session_id: String,
    pub max_players: u32,
    pub features: Vec<String>,
    pub error_message: Option<String>,
}

// 认证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub auth_type: AuthType,
    pub credentials: HashMap<String, String>,
    pub client_data: HashMap<String, String>,
}

// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Guest,
    Token,
    OAuth,
    Certificate,
}

// 认证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub player_id: Option<String>,
    pub permissions: Vec<String>,
    pub session_data: HashMap<String, String>,
    pub error_code: Option<u32>,
    pub error_message: Option<String>,
}

// 房间创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub room_name: String,
    pub password: Option<String>,
    pub max_players: u8,
    pub battle_type: BattleType,
    pub settings: RoomSettings,
}

// 对战类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleType {
    Single,
    Double,
    Multi,
    Tournament,
}

// 房间设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub time_limit: Option<u32>,
    pub level_cap: Option<u8>,
    pub allow_spectators: bool,
    pub private_room: bool,
    pub custom_rules: HashMap<String, String>,
}

// 房间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub current_players: u8,
    pub max_players: u8,
    pub has_password: bool,
    pub battle_type: BattleType,
    pub state: RoomState,
}

// 房间状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,
    Starting,
    InProgress,
    Finished,
}

// 玩家动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: ActionType,
    pub target_id: Option<String>,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

// 动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    UseItem,
    SwitchPokemon,
    Surrender,
    Chat,
    Emote,
    SelectTarget,
}

// 游戏状态更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateUpdate {
    pub battle_id: String,
    pub turn_number: u32,
    pub current_player: Option<String>,
    pub pokemon_states: Vec<PokemonState>,
    pub field_effects: Vec<FieldEffect>,
    pub timer: Option<u32>,
}

// 宝可梦状态（网络传输用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonState {
    pub id: String,
    pub species_id: u16,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub status: Vec<StatusCondition>,
    pub position: BattlePosition,
    pub stats: PokemonStats,
}

// 状态条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCondition {
    pub effect_type: String,
    pub duration: Option<u32>,
    pub severity: u8,
}

// 战场位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BattlePosition {
    PlayerLeft,
    PlayerRight,
    OpponentLeft,
    OpponentRight,
    Bench(u8),
}

// 宝可梦属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonStats {
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
    pub accuracy: u8,
    pub evasion: u8,
}

// 场地效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEffect {
    pub effect_type: String,
    pub duration: Option<u32>,
    pub affects: Vec<BattlePosition>,
    pub parameters: HashMap<String, i32>,
}

// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender_id: String,
    pub sender_name: String,
    pub message: String,
    pub timestamp: u64,
    pub channel: ChatChannel,
    pub target: Option<String>,
}

// 聊天频道
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatChannel {
    Global,
    Room,
    Team,
    Private,
    System,
}

// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_code: u32,
    pub error_message: String,
    pub details: Option<HashMap<String, String>>,
    pub recoverable: bool,
}

// 协议处理器
pub struct ProtocolHandler {
    sequence_counter: u16,
    compression_enabled: bool,
    encryption_enabled: bool,
    max_packet_size: usize,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            sequence_counter: 0,
            compression_enabled: false,
            encryption_enabled: false,
            max_packet_size: MAX_PACKET_SIZE,
        }
    }

    // 创建数据包
    pub fn create_packet<T: Serialize>(&mut self, packet_type: PacketType, data: &T) -> GameResult<Packet> {
        let payload = self.serialize_payload(data)?;
        
        if payload.len() > self.max_packet_size {
            return Err(GameError::Protocol("载荷过大".to_string()));
        }

        let mut packet = Packet::new(packet_type, payload);
        packet.header.sequence = self.next_sequence();

        // 应用压缩
        if self.compression_enabled && packet.payload.len() > 128 {
            packet.payload = self.compress_data(&packet.payload)?;
            packet.header.flags |= PacketFlags::COMPRESSED;
            packet.header.payload_size = packet.payload.len() as u16;
        }

        // 应用加密
        if self.encryption_enabled {
            packet.payload = self.encrypt_data(&packet.payload)?;
            packet.header.flags |= PacketFlags::ENCRYPTED;
        }

        // 重新计算校验和
        packet.header.checksum = Packet::calculate_checksum(&packet.payload);

        Ok(packet)
    }

    // 解析数据包
    pub fn parse_packet<T: for<'de> Deserialize<'de>>(&self, packet: &Packet) -> GameResult<T> {
        let mut payload = packet.payload.clone();

        // 解密
        if packet.header.flags.contains(PacketFlags::ENCRYPTED) {
            payload = self.decrypt_data(&payload)?;
        }

        // 解压缩
        if packet.header.flags.contains(PacketFlags::COMPRESSED) {
            payload = self.decompress_data(&payload)?;
        }

        self.deserialize_payload(&payload)
    }

    // 序列化载荷
    fn serialize_payload<T: Serialize>(&self, data: &T) -> GameResult<Vec<u8>> {
        bincode::serialize(data)
            .map_err(|e| GameError::Protocol(format!("序列化失败: {}", e)))
    }

    // 反序列化载荷
    fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> GameResult<T> {
        bincode::deserialize(data)
            .map_err(|e| GameError::Protocol(format!("反序列化失败: {}", e)))
    }

    // 压缩数据
    fn compress_data(&self, data: &[u8]) -> GameResult<Vec<u8>> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| GameError::Protocol(format!("压缩失败: {}", e)))?;
        
        encoder.finish()
            .map_err(|e| GameError::Protocol(format!("压缩完成失败: {}", e)))
    }

    // 解压缩数据
    fn decompress_data(&self, data: &[u8]) -> GameResult<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| GameError::Protocol(format!("解压缩失败: {}", e)))?;
        
        Ok(decompressed)
    }

    // 加密数据（简化实现）
    fn encrypt_data(&self, data: &[u8]) -> GameResult<Vec<u8>> {
        // 简化的XOR加密，实际应用应使用更安全的加密算法
        let key = b"pokemon_game_key";
        let encrypted: Vec<u8> = data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect();
        
        Ok(encrypted)
    }

    // 解密数据（简化实现）
    fn decrypt_data(&self, data: &[u8]) -> GameResult<Vec<u8>> {
        // XOR加密的解密就是再次XOR
        self.encrypt_data(data)
    }

    // 获取下一个序列号
    fn next_sequence(&mut self) -> u16 {
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        self.sequence_counter
    }

    // 启用压缩
    pub fn enable_compression(&mut self) {
        self.compression_enabled = true;
        info!("协议压缩已启用");
    }

    // 启用加密
    pub fn enable_encryption(&mut self) {
        self.encryption_enabled = true;
        info!("协议加密已启用");
    }

    // 设置最大数据包大小
    pub fn set_max_packet_size(&mut self, size: usize) {
        self.max_packet_size = size;
    }

    // 验证数据包完整性
    pub fn validate_packet(&self, packet: &Packet) -> bool {
        // 检查版本兼容性
        if packet.header.timestamp == 0 {
            return false;
        }

        // 检查数据包大小
        if packet.payload.len() > self.max_packet_size {
            return false;
        }

        // 验证校验和
        let calculated = Packet::calculate_checksum(&packet.payload);
        calculated == packet.header.checksum
    }

    // 创建错误响应
    pub fn create_error_response(&mut self, code: u32, message: &str) -> GameResult<Packet> {
        let error = ErrorResponse {
            error_code: code,
            error_message: message.to_string(),
            details: None,
            recoverable: code < 500,
        };

        self.create_packet(PacketType::Error, &error)
    }

    // 创建握手响应
    pub fn create_handshake_response(
        &mut self, 
        accepted: bool, 
        session_id: String
    ) -> GameResult<Packet> {
        let response = HandshakeResponse {
            accepted,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            session_id,
            max_players: 1000,
            features: vec![
                "compression".to_string(),
                "encryption".to_string(),
                "reliable_delivery".to_string(),
            ],
            error_message: if accepted { None } else { Some("握手失败".to_string()) },
        };

        self.create_packet(PacketType::HandshakeResponse, &response)
    }
}

// 数据包碎片管理器
pub struct PacketFragmentManager {
    fragments: HashMap<String, Vec<Option<Vec<u8>>>>,
    fragment_timeout: std::time::Duration,
}

impl PacketFragmentManager {
    pub fn new() -> Self {
        Self {
            fragments: HashMap::new(),
            fragment_timeout: std::time::Duration::from_secs(30),
        }
    }

    // 分片大数据包
    pub fn fragment_packet(&self, packet: &Packet, max_fragment_size: usize) -> Vec<Packet> {
        if packet.payload.len() <= max_fragment_size {
            return vec![packet.clone()];
        }

        let fragment_count = (packet.payload.len() + max_fragment_size - 1) / max_fragment_size;
        let mut fragments = Vec::new();

        for i in 0..fragment_count {
            let start = i * max_fragment_size;
            let end = std::cmp::min(start + max_fragment_size, packet.payload.len());
            let fragment_data = packet.payload[start..end].to_vec();

            let mut fragment_header = packet.header.clone();
            fragment_header.flags |= PacketFlags::FRAGMENTED;
            fragment_header.sequence = i as u16;
            fragment_header.payload_size = fragment_data.len() as u16;
            fragment_header.checksum = Packet::calculate_checksum(&fragment_data);

            fragments.push(Packet {
                header: fragment_header,
                payload: fragment_data,
            });
        }

        fragments
    }

    // 重组分片数据包
    pub fn reassemble_packet(&mut self, fragment: Packet) -> Option<Packet> {
        if !fragment.header.flags.contains(PacketFlags::FRAGMENTED) {
            return Some(fragment);
        }

        let fragment_id = format!("{}_{}", 
            fragment.header.timestamp, 
            fragment.header.packet_type as u8
        );

        // TODO: 实现完整的分片重组逻辑
        // 这里需要处理分片的存储、排序和重组
        
        None
    }

    // 清理过期分片
    pub fn cleanup_expired_fragments(&mut self) {
        // TODO: 实现过期分片清理
    }
}

// 协议统计信息
#[derive(Debug, Default)]
pub struct ProtocolStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub compression_ratio: f32,
    pub packet_loss_rate: f32,
    pub average_latency: f32,
}

// 便捷函数
impl ProtocolHandler {
    // 创建常用数据包的便捷方法
    
    pub fn create_heartbeat(&mut self) -> GameResult<Packet> {
        self.create_packet(PacketType::Heartbeat, &())
    }

    pub fn create_disconnect(&mut self) -> GameResult<Packet> {
        self.create_packet(PacketType::Disconnect, &())
    }

    pub fn create_chat_message(&mut self, message: ChatMessage) -> GameResult<Packet> {
        self.create_packet(PacketType::ChatMessage, &message)
    }

    pub fn create_player_action(&mut self, action: PlayerAction) -> GameResult<Packet> {
        self.create_packet(PacketType::PlayerAction, &action)
    }

    pub fn create_game_state_update(&mut self, state: GameStateUpdate) -> GameResult<Packet> {
        self.create_packet(PacketType::GameState, &state)
    }

    // 解析常用数据包的便捷方法
    
    pub fn parse_handshake(&self, packet: &Packet) -> GameResult<HandshakeData> {
        self.parse_packet(packet)
    }

    pub fn parse_auth_request(&self, packet: &Packet) -> GameResult<AuthRequest> {
        self.parse_packet(packet)
    }

    pub fn parse_create_room(&self, packet: &Packet) -> GameResult<CreateRoomRequest> {
        self.parse_packet(packet)
    }

    pub fn parse_player_action(&self, packet: &Packet) -> GameResult<PlayerAction> {
        self.parse_packet(packet)
    }

    pub fn parse_chat_message(&self, packet: &Packet) -> GameResult<ChatMessage> {
        self.parse_packet(packet)
    }
}