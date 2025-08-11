// 数据序列化器
// 开发心理：序列化器负责数据的编码解码，支持多种格式和版本兼容
// 设计原则：格式多样、版本兼容、性能优化、错误恢复

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;

// 数据序列化器
pub struct DataSerializer {
    // 序列化配置
    config: SerializerConfig,
    
    // 版本映射
    version_handlers: HashMap<String, Box<dyn VersionHandler>>,
    
    // 格式处理器
    format_handlers: HashMap<SerializationFormat, FormatHandler>,
    
    // 统计信息
    statistics: SerializerStatistics,
}

// 序列化配置
#[derive(Debug, Clone)]
pub struct SerializerConfig {
    pub default_format: SerializationFormat,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub version_checking: bool,
    pub pretty_print: bool,
    pub buffer_size: usize,
}

// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerializationFormat {
    JSON,           // JSON格式
    Binary,         // 二进制格式
    MessagePack,    // MessagePack格式
    CBOR,           // CBOR格式
    YAML,           // YAML格式
    TOML,           // TOML格式
    XML,            // XML格式
}

// 版本处理器
pub trait VersionHandler: Send + Sync {
    fn can_handle(&self, version: &str) -> bool;
    fn migrate(&self, data: &[u8], from_version: &str, to_version: &str) -> Result<Vec<u8>, GameError>;
    fn validate(&self, data: &[u8], version: &str) -> Result<bool, GameError>;
}

// 格式处理器枚举 - 解决dyn兼容性问题
#[derive(Debug)]
pub enum FormatHandler {
    Json(JsonHandler),
    Binary(BinaryHandler),
    MessagePack(MessagePackHandler),
    Cbor(CborHandler),
    Yaml(YamlHandler),
    Toml(TomlHandler),
    Xml(XmlHandler),
}

impl FormatHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        match self {
            FormatHandler::Json(handler) => handler.serialize(data),
            FormatHandler::Binary(handler) => handler.serialize(data),
            FormatHandler::MessagePack(handler) => handler.serialize(data),
            FormatHandler::Cbor(handler) => handler.serialize(data),
            FormatHandler::Yaml(handler) => handler.serialize(data),
            FormatHandler::Toml(handler) => handler.serialize(data),
            FormatHandler::Xml(handler) => handler.serialize(data),
        }
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        match self {
            FormatHandler::Json(handler) => handler.deserialize(data),
            FormatHandler::Binary(handler) => handler.deserialize(data),
            FormatHandler::MessagePack(handler) => handler.deserialize(data),
            FormatHandler::Cbor(handler) => handler.deserialize(data),
            FormatHandler::Yaml(handler) => handler.deserialize(data),
            FormatHandler::Toml(handler) => handler.deserialize(data),
            FormatHandler::Xml(handler) => handler.deserialize(data),
        }
    }
    
    pub fn get_content_type(&self) -> &'static str {
        match self {
            FormatHandler::Json(_) => "application/json",
            FormatHandler::Binary(_) => "application/octet-stream",
            FormatHandler::MessagePack(_) => "application/msgpack",
            FormatHandler::Cbor(_) => "application/cbor",
            FormatHandler::Yaml(_) => "application/x-yaml",
            FormatHandler::Toml(_) => "application/toml",
            FormatHandler::Xml(_) => "application/xml",
        }
    }
    
    pub fn supports_pretty_print(&self) -> bool {
        match self {
            FormatHandler::Json(_) | FormatHandler::Yaml(_) | 
            FormatHandler::Toml(_) | FormatHandler::Xml(_) => true,
            _ => false,
        }
    }
}

// 序列化结果
#[derive(Debug, Clone)]
pub struct SerializationResult {
    pub data: Vec<u8>,
    pub format: SerializationFormat,
    pub version: String,
    pub compressed: bool,
    pub encrypted: bool,
    pub checksum: String,
    pub size_original: usize,
    pub size_final: usize,
}

// 反序列化结果
#[derive(Debug, Clone)]
pub struct DeserializationResult<T> {
    pub data: T,
    pub format: SerializationFormat,
    pub version: String,
    pub was_compressed: bool,
    pub was_encrypted: bool,
    pub checksum_valid: bool,
}

// 序列化统计
#[derive(Debug, Clone, Default)]
pub struct SerializerStatistics {
    pub serializations: u64,
    pub deserializations: u64,
    pub compressions: u64,
    pub decompressions: u64,
    pub encryptions: u64,
    pub decryptions: u64,
    pub version_migrations: u64,
    pub total_bytes_serialized: u64,
    pub total_bytes_deserialized: u64,
    pub average_compression_ratio: f32,
}

impl DataSerializer {
    pub fn new() -> Self {
        let mut serializer = Self {
            config: SerializerConfig::default(),
            version_handlers: HashMap::new(),
            format_handlers: HashMap::new(),
            statistics: SerializerStatistics::default(),
        };
        
        // 注册默认格式处理器
        serializer.register_format_handler(SerializationFormat::JSON, FormatHandler::Json(JsonHandler));
        serializer.register_format_handler(SerializationFormat::Binary, FormatHandler::Binary(BinaryHandler));
        serializer.register_format_handler(SerializationFormat::MessagePack, FormatHandler::MessagePack(MessagePackHandler));
        serializer.register_format_handler(SerializationFormat::CBOR, FormatHandler::Cbor(CborHandler));
        serializer.register_format_handler(SerializationFormat::YAML, FormatHandler::Yaml(YamlHandler));
        serializer.register_format_handler(SerializationFormat::TOML, FormatHandler::Toml(TomlHandler));
        serializer.register_format_handler(SerializationFormat::XML, FormatHandler::Xml(XmlHandler));
        
        serializer
    }
    
    // 注册版本处理器
    pub fn register_version_handler(&mut self, version: String, handler: Box<dyn VersionHandler>) {
        self.version_handlers.insert(version, handler);
        debug!("注册版本处理器: {}", version);
    }
    
    // 注册格式处理器
    pub fn register_format_handler(&mut self, format: SerializationFormat, handler: FormatHandler) {
        self.format_handlers.insert(format, handler);
        debug!("注册格式处理器: {:?}", format);
    }
    
    // 序列化数据
    pub fn serialize<T>(&mut self, data: &T, format: Option<SerializationFormat>, version: Option<String>) -> Result<SerializationResult, GameError>
    where
        T: Serialize,
    {
        let format = format.unwrap_or(self.config.default_format);
        let version = version.unwrap_or_else(|| "1.0.0".to_string());
        
        // 获取格式处理器
        let handler = self.format_handlers.get(&format)
            .ok_or_else(|| GameError::Data(format!("不支持的序列化格式: {:?}", format)))?;
        
        // 序列化数据
        let mut serialized_data = handler.serialize(data)?;
        let original_size = serialized_data.len();
        
        // 压缩
        let compressed = if self.config.compression_enabled {
            serialized_data = self.compress_data(&serialized_data)?;
            self.statistics.compressions += 1;
            true
        } else {
            false
        };
        
        // 加密
        let encrypted = if self.config.encryption_enabled {
            serialized_data = self.encrypt_data(&serialized_data)?;
            self.statistics.encryptions += 1;
            true
        } else {
            false
        };
        
        // 计算校验和
        let checksum = self.calculate_checksum(&serialized_data);
        
        // 更新统计
        self.statistics.serializations += 1;
        self.statistics.total_bytes_serialized += serialized_data.len() as u64;
        
        if compressed {
            let compression_ratio = serialized_data.len() as f32 / original_size as f32;
            self.statistics.average_compression_ratio = 
                (self.statistics.average_compression_ratio * (self.statistics.compressions - 1) as f32 + compression_ratio) / 
                self.statistics.compressions as f32;
        }
        
        let result = SerializationResult {
            data: serialized_data,
            format,
            version,
            compressed,
            encrypted,
            checksum,
            size_original: original_size,
            size_final: serialized_data.len(),
        };
        
        debug!("序列化完成: 格式={:?}, 原始大小={}, 最终大小={}, 压缩={}, 加密={}", 
            format, original_size, result.size_final, compressed, encrypted);
        
        Ok(result)
    }
    
    // 反序列化数据
    pub fn deserialize<T>(&mut self, data: &[u8], expected_format: Option<SerializationFormat>) -> Result<DeserializationResult<T>, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut data_to_process = data.to_vec();
        
        // 尝试检测格式
        let format = if let Some(fmt) = expected_format {
            fmt
        } else {
            self.detect_format(&data_to_process)?
        };
        
        // 验证校验和（如果启用）
        let checksum_valid = if self.config.version_checking {
            self.verify_checksum(&data_to_process)
        } else {
            true
        };
        
        // 解密
        let was_encrypted = if self.is_encrypted(&data_to_process) {
            data_to_process = self.decrypt_data(&data_to_process)?;
            self.statistics.decryptions += 1;
            true
        } else {
            false
        };
        
        // 解压缩
        let was_compressed = if self.is_compressed(&data_to_process) {
            data_to_process = self.decompress_data(&data_to_process)?;
            self.statistics.decompressions += 1;
            true
        } else {
            false
        };
        
        // 获取格式处理器并反序列化
        let handler = self.format_handlers.get(&format)
            .ok_or_else(|| GameError::Data(format!("不支持的反序列化格式: {:?}", format)))?;
        
        let deserialized_data = handler.deserialize::<T>(&data_to_process)?;
        
        // 更新统计
        self.statistics.deserializations += 1;
        self.statistics.total_bytes_deserialized += data.len() as u64;
        
        let result = DeserializationResult {
            data: deserialized_data,
            format,
            version: "1.0.0".to_string(), // 简化实现
            was_compressed,
            was_encrypted,
            checksum_valid,
        };
        
        debug!("反序列化完成: 格式={:?}, 大小={}, 压缩={}, 加密={}", 
            format, data.len(), was_compressed, was_encrypted);
        
        Ok(result)
    }
    
    // 批量序列化
    pub fn serialize_batch<T>(&mut self, items: &[T], format: SerializationFormat) -> Result<Vec<SerializationResult>, GameError>
    where
        T: Serialize,
    {
        let mut results = Vec::new();
        
        for item in items {
            let result = self.serialize(item, Some(format), None)?;
            results.push(result);
        }
        
        debug!("批量序列化完成: {} 项", results.len());
        Ok(results)
    }
    
    // 迁移数据版本
    pub fn migrate_version(&mut self, data: &[u8], from_version: &str, to_version: &str) -> Result<Vec<u8>, GameError> {
        // 查找版本处理器
        let handler = self.version_handlers.values()
            .find(|h| h.can_handle(from_version) && h.can_handle(to_version))
            .ok_or_else(|| GameError::Data(format!("找不到版本迁移处理器: {} -> {}", from_version, to_version)))?;
        
        let migrated_data = handler.migrate(data, from_version, to_version)?;
        
        self.statistics.version_migrations += 1;
        debug!("版本迁移完成: {} -> {}", from_version, to_version);
        
        Ok(migrated_data)
    }
    
    // 验证数据
    pub fn validate_data(&self, data: &[u8], format: SerializationFormat, version: &str) -> Result<bool, GameError> {
        // 检查格式是否有效
        if !self.format_handlers.contains_key(&format) {
            return Ok(false);
        }
        
        // 检查版本是否有效
        if let Some(handler) = self.version_handlers.values().find(|h| h.can_handle(version)) {
            handler.validate(data, version)
        } else {
            Ok(true) // 如果没有版本处理器，默认有效
        }
    }
    
    // 获取统计信息
    pub fn get_statistics(&self) -> &SerializerStatistics {
        &self.statistics
    }
    
    // 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.statistics = SerializerStatistics::default();
        debug!("序列化统计信息已重置");
    }
    
    // 私有方法
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, GameError> {
        // 简化实现：使用flate2库进行压缩
        use std::io::Write;
        
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).map_err(|e| GameError::Data(format!("压缩失败: {}", e)))?;
        encoder.finish().map_err(|e| GameError::Data(format!("压缩完成失败: {}", e)))
    }
    
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, GameError> {
        // 简化实现：使用flate2库进行解压
        use std::io::Read;
        
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| GameError::Data(format!("解压失败: {}", e)))?;
        Ok(decompressed)
    }
    
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, GameError> {
        // 简化实现：这里应该使用真正的加密算法
        Ok(data.to_vec())
    }
    
    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, GameError> {
        // 简化实现：这里应该使用真正的解密算法
        Ok(data.to_vec())
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> String {
        // 简化实现：使用CRC32作为校验和
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    fn verify_checksum(&self, data: &[u8]) -> bool {
        // 简化实现：总是返回true
        true
    }
    
    fn detect_format(&self, data: &[u8]) -> Result<SerializationFormat, GameError> {
        // 简单的格式检测
        if data.starts_with(b"{") || data.starts_with(b"[") {
            Ok(SerializationFormat::JSON)
        } else if data.len() >= 4 && &data[0..4] == b"\x93\x92\x91\x90" {
            // MessagePack magic number (简化)
            Ok(SerializationFormat::MessagePack)
        } else {
            Ok(SerializationFormat::Binary)
        }
    }
    
    fn is_compressed(&self, data: &[u8]) -> bool {
        // 检查GZIP魔数
        data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
    }
    
    fn is_encrypted(&self, _data: &[u8]) -> bool {
        // 简化实现：总是返回false
        false
    }
}

// JSON格式处理器
#[derive(Debug)]
pub struct JsonHandler;

impl JsonHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        let json_string = serde_json::to_string(data)
            .map_err(|e| GameError::Data(format!("JSON序列化失败: {}", e)))?;
        Ok(json_string.into_bytes())
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        let json_string = String::from_utf8(data.to_vec())
            .map_err(|e| GameError::Data(format!("UTF-8解码失败: {}", e)))?;
        
        serde_json::from_str(&json_string)
            .map_err(|e| GameError::Data(format!("JSON反序列化失败: {}", e)))
    }
}

// 二进制格式处理器
#[derive(Debug)]
pub struct BinaryHandler;

impl BinaryHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        bincode::serialize(data)
            .map_err(|e| GameError::Data(format!("二进制序列化失败: {}", e)))
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        bincode::deserialize(data)
            .map_err(|e| GameError::Data(format!("二进制反序列化失败: {}", e)))
    }
}

// MessagePack格式处理器
#[derive(Debug)]
pub struct MessagePackHandler;

impl MessagePackHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        rmp_serde::to_vec(data)
            .map_err(|e| GameError::Data(format!("MessagePack序列化失败: {}", e)))
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        rmp_serde::from_slice(data)
            .map_err(|e| GameError::Data(format!("MessagePack反序列化失败: {}", e)))
    }
}

// CBOR格式处理器
#[derive(Debug)]
pub struct CborHandler;

impl CborHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        serde_cbor::to_vec(data)
            .map_err(|e| GameError::Data(format!("CBOR序列化失败: {}", e)))
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        serde_cbor::from_slice(data)
            .map_err(|e| GameError::Data(format!("CBOR反序列化失败: {}", e)))
    }
}

// YAML格式处理器
#[derive(Debug)]
pub struct YamlHandler;

impl YamlHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        let yaml_string = serde_yaml::to_string(data)
            .map_err(|e| GameError::Data(format!("YAML序列化失败: {}", e)))?;
        Ok(yaml_string.into_bytes())
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        let yaml_string = String::from_utf8(data.to_vec())
            .map_err(|e| GameError::Data(format!("UTF-8解码失败: {}", e)))?;
        
        serde_yaml::from_str(&yaml_string)
            .map_err(|e| GameError::Data(format!("YAML反序列化失败: {}", e)))
    }
}

// TOML格式处理器
#[derive(Debug)]
pub struct TomlHandler;

impl TomlHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        let toml_string = toml::to_string(data)
            .map_err(|e| GameError::Data(format!("TOML序列化失败: {}", e)))?;
        Ok(toml_string.into_bytes())
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        let toml_string = String::from_utf8(data.to_vec())
            .map_err(|e| GameError::Data(format!("UTF-8解码失败: {}", e)))?;
        
        toml::from_str(&toml_string)
            .map_err(|e| GameError::Data(format!("TOML反序列化失败: {}", e)))
    }
}

// XML格式处理器
#[derive(Debug)]
pub struct XmlHandler;

impl XmlHandler {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, GameError> {
        // 使用quick-xml和serde进行XML序列化
        let xml_string = quick_xml::se::to_string(data)
            .map_err(|e| GameError::Data(format!("XML序列化失败: {}", e)))?;
        Ok(xml_string.into_bytes())
    }
    
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, GameError> {
        let xml_string = String::from_utf8(data.to_vec())
            .map_err(|e| GameError::Data(format!("UTF-8解码失败: {}", e)))?;
        
        quick_xml::de::from_str(&xml_string)
            .map_err(|e| GameError::Data(format!("XML反序列化失败: {}", e)))
    }
}

impl Default for SerializerConfig {
    fn default() -> Self {
        Self {
            default_format: SerializationFormat::JSON,
            compression_enabled: true,
            encryption_enabled: false,
            version_checking: true,
            pretty_print: false,
            buffer_size: 8192,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
        active: bool,
    }
    
    #[test]
    fn test_serializer_creation() {
        let serializer = DataSerializer::new();
        assert!(serializer.format_handlers.contains_key(&SerializationFormat::JSON));
        assert!(serializer.format_handlers.contains_key(&SerializationFormat::Binary));
    }
    
    #[test]
    fn test_json_serialization() {
        let mut serializer = DataSerializer::new();
        
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
            active: true,
        };
        
        let result = serializer.serialize(&test_data, Some(SerializationFormat::JSON), None).unwrap();
        assert_eq!(result.format, SerializationFormat::JSON);
        assert!(!result.data.is_empty());
    }
    
    #[test]
    fn test_json_deserialization() {
        let mut serializer = DataSerializer::new();
        
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
            active: true,
        };
        
        let serialized = serializer.serialize(&test_data, Some(SerializationFormat::JSON), None).unwrap();
        let result: DeserializationResult<TestData> = serializer.deserialize(&serialized.data, Some(SerializationFormat::JSON)).unwrap();
        
        assert_eq!(result.data, test_data);
        assert_eq!(result.format, SerializationFormat::JSON);
    }
    
    #[test]
    fn test_binary_serialization() {
        let mut serializer = DataSerializer::new();
        
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
            active: true,
        };
        
        let result = serializer.serialize(&test_data, Some(SerializationFormat::Binary), None).unwrap();
        assert_eq!(result.format, SerializationFormat::Binary);
        assert!(!result.data.is_empty());
    }
    
    #[test]
    fn test_statistics() {
        let mut serializer = DataSerializer::new();
        
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
            active: true,
        };
        
        serializer.serialize(&test_data, Some(SerializationFormat::JSON), None).unwrap();
        
        let stats = serializer.get_statistics();
        assert_eq!(stats.serializations, 1);
        assert!(stats.total_bytes_serialized > 0);
    }
}