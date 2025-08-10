// 网络客户端
// 开发心理：客户端负责与服务器通信，处理网络请求和响应
// 设计原则：异步通信、重连机制、消息队列、错误处理

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use log::{debug, warn, error, info};
use crate::core::error::GameError;

// 网络客户端
pub struct NetworkClient {
    // 客户端状态
    state: ClientState,
    
    // 连接配置
    config: ClientConfig,
    
    // 消息队列
    outgoing_messages: VecDeque<NetworkMessage>,
    incoming_messages: VecDeque<NetworkMessage>,
    
    // 统计信息
    statistics: ClientStatistics,
}

// 客户端状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_address: String,
    pub server_port: u16,
    pub connection_timeout: std::time::Duration,
    pub reconnect_attempts: u32,
    pub reconnect_delay: std::time::Duration,
    pub keep_alive_interval: std::time::Duration,
    pub message_buffer_size: usize,
}

// 网络消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: u64,
    pub message_type: String,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
    pub priority: MessagePriority,
}

// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

// 客户端统计
#[derive(Debug, Clone, Default)]
pub struct ClientStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_attempts: u32,
    pub successful_connections: u32,
    pub connection_errors: u32,
    pub last_ping: Option<std::time::Duration>,
}

impl NetworkClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            state: ClientState::Disconnected,
            config,
            outgoing_messages: VecDeque::new(),
            incoming_messages: VecDeque::new(),
            statistics: ClientStatistics::default(),
        }
    }
    
    // 连接到服务器
    pub fn connect(&mut self) -> Result<(), GameError> {
        if self.state == ClientState::Connected {
            return Ok(());
        }
        
        self.state = ClientState::Connecting;
        self.statistics.connection_attempts += 1;
        
        // 实际的连接逻辑应该在这里实现
        // 简化实现：直接标记为已连接
        self.state = ClientState::Connected;
        self.statistics.successful_connections += 1;
        
        info!("连接到服务器: {}:{}", self.config.server_address, self.config.server_port);
        Ok(())
    }
    
    // 断开连接
    pub fn disconnect(&mut self) -> Result<(), GameError> {
        if self.state == ClientState::Disconnected {
            return Ok(());
        }
        
        // 发送剩余消息
        self.flush_outgoing_messages()?;
        
        self.state = ClientState::Disconnected;
        
        info!("断开服务器连接");
        Ok(())
    }
    
    // 发送消息
    pub fn send_message(&mut self, message_type: String, data: Vec<u8>, priority: MessagePriority) -> Result<u64, GameError> {
        let message_id = self.generate_message_id();
        
        let message = NetworkMessage {
            id: message_id,
            message_type,
            data,
            timestamp: std::time::SystemTime::now(),
            priority,
        };
        
        // 根据优先级插入到队列中
        self.insert_message_by_priority(message);
        
        debug!("添加消息到发送队列: ID={} 优先级={:?}", message_id, priority);
        Ok(message_id)
    }
    
    // 接收消息
    pub fn receive_message(&mut self) -> Option<NetworkMessage> {
        self.incoming_messages.pop_front()
    }
    
    // 更新客户端
    pub fn update(&mut self, _delta_time: f32) -> Result<(), GameError> {
        match self.state {
            ClientState::Connected => {
                // 处理传出消息
                self.process_outgoing_messages()?;
                
                // 处理传入消息
                self.process_incoming_messages()?;
                
                // 发送心跳
                self.send_keep_alive()?;
            },
            ClientState::Connecting => {
                // 检查连接状态
            },
            ClientState::Reconnecting => {
                // 尝试重连
                self.attempt_reconnect()?;
            },
            _ => {}
        }
        
        Ok(())
    }
    
    // 获取连接状态
    pub fn get_state(&self) -> ClientState {
        self.state
    }
    
    // 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.state == ClientState::Connected
    }
    
    // 获取统计信息
    pub fn get_statistics(&self) -> &ClientStatistics {
        &self.statistics
    }
    
    // 获取待发送消息数量
    pub fn get_pending_message_count(&self) -> usize {
        self.outgoing_messages.len()
    }
    
    // 清空消息队列
    pub fn clear_message_queues(&mut self) {
        self.outgoing_messages.clear();
        self.incoming_messages.clear();
        debug!("消息队列已清空");
    }
    
    // 私有方法
    fn generate_message_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    fn insert_message_by_priority(&mut self, message: NetworkMessage) {
        let position = self.outgoing_messages
            .iter()
            .position(|msg| msg.priority < message.priority)
            .unwrap_or(self.outgoing_messages.len());
        
        self.outgoing_messages.insert(position, message);
    }
    
    fn process_outgoing_messages(&mut self) -> Result<(), GameError> {
        let max_messages = 10; // 每帧最多处理10条消息
        
        for _ in 0..max_messages {
            if let Some(message) = self.outgoing_messages.pop_front() {
                // 实际发送消息的逻辑
                self.send_message_to_server(&message)?;
                
                self.statistics.messages_sent += 1;
                self.statistics.bytes_sent += message.data.len() as u64;
                
                debug!("发送消息: ID={} 类型={}", message.id, message.message_type);
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    fn process_incoming_messages(&mut self) -> Result<(), GameError> {
        // 简化实现：模拟接收消息
        // 实际实现应该从网络读取数据
        
        Ok(())
    }
    
    fn send_message_to_server(&self, _message: &NetworkMessage) -> Result<(), GameError> {
        // 简化实现：实际应该通过网络发送
        Ok(())
    }
    
    fn send_keep_alive(&mut self) -> Result<(), GameError> {
        // 发送心跳包
        let keep_alive_data = b"ping".to_vec();
        self.send_message("keep_alive".to_string(), keep_alive_data, MessagePriority::Low)?;
        
        Ok(())
    }
    
    fn attempt_reconnect(&mut self) -> Result<(), GameError> {
        if self.statistics.connection_attempts < self.config.reconnect_attempts {
            debug!("尝试重连: 第 {} 次", self.statistics.connection_attempts + 1);
            self.connect()
        } else {
            self.state = ClientState::Error;
            Err(GameError::Network("重连尝试次数已达上限".to_string()))
        }
    }
    
    fn flush_outgoing_messages(&mut self) -> Result<(), GameError> {
        while let Some(message) = self.outgoing_messages.pop_front() {
            self.send_message_to_server(&message)?;
        }
        Ok(())
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            server_port: 8080,
            connection_timeout: std::time::Duration::from_secs(30),
            reconnect_attempts: 3,
            reconnect_delay: std::time::Duration::from_secs(5),
            keep_alive_interval: std::time::Duration::from_secs(30),
            message_buffer_size: 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let client = NetworkClient::new(config);
        
        assert_eq!(client.get_state(), ClientState::Disconnected);
        assert_eq!(client.get_pending_message_count(), 0);
    }
    
    #[test]
    fn test_message_priority() {
        let mut client = NetworkClient::new(ClientConfig::default());
        
        client.send_message("low".to_string(), b"low".to_vec(), MessagePriority::Low).unwrap();
        client.send_message("high".to_string(), b"high".to_vec(), MessagePriority::High).unwrap();
        client.send_message("normal".to_string(), b"normal".to_vec(), MessagePriority::Normal).unwrap();
        
        // 高优先级消息应该排在前面
        assert_eq!(client.outgoing_messages[0].message_type, "high");
        assert_eq!(client.outgoing_messages[1].message_type, "normal");
        assert_eq!(client.outgoing_messages[2].message_type, "low");
    }
    
    #[test]
    fn test_connection_state() {
        let mut client = NetworkClient::new(ClientConfig::default());
        
        assert_eq!(client.get_state(), ClientState::Disconnected);
        assert!(!client.is_connected());
        
        client.connect().unwrap();
        
        assert_eq!(client.get_state(), ClientState::Connected);
        assert!(client.is_connected());
    }
}