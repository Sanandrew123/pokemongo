// 纹理系统
// 开发心理：纹理是游戏视觉效果的基础，需要高效的内存管理和GPU上传
// 设计原则：异步加载、格式自适应、内存池管理、压缩纹理支持

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceHandle, ResourceManager};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use log::{info, debug, warn, error};
use std::path::{Path, PathBuf};

pub type TextureId = u32;

// 纹理格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    // 无压缩格式
    R8,
    RG8,
    RGB8,
    RGBA8,
    R16,
    RG16,
    RGB16,
    RGBA16,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
    
    // 深度/模板格式
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    Depth32FStencil8,
    
    // 压缩格式
    DXT1,      // BC1
    DXT3,      // BC2
    DXT5,      // BC3
    RGTC1,     // BC4
    RGTC2,     // BC5
    BPTC,      // BC6H/BC7
    ETC2_RGB8,
    ETC2_RGBA8,
    ASTC_4x4,
    ASTC_8x8,
    
    // 特殊格式
    sRGB8,
    sRGBA8,
}

// 纹理过滤模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

// 纹理包装模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureWrap {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

// 纹理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureType {
    Texture2D,
    Texture3D,
    TextureCube,
    Texture2DArray,
    TextureCubeArray,
}

// 纹理使用标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureUsage {
    pub read: bool,
    pub write: bool,
    pub render_target: bool,
    pub depth_stencil: bool,
    pub generate_mipmaps: bool,
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            render_target: false,
            depth_stencil: false,
            generate_mipmaps: true,
        }
    }
}

// 纹理描述符
#[derive(Debug, Clone)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub format: TextureFormat,
    pub texture_type: TextureType,
    pub usage: TextureUsage,
    pub min_filter: TextureFilter,
    pub mag_filter: TextureFilter,
    pub wrap_s: TextureWrap,
    pub wrap_t: TextureWrap,
    pub wrap_r: TextureWrap,
    pub border_color: [f32; 4],
    pub anisotropy: f32,
}

impl Default for TextureDesc {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
            mip_levels: 1,
            array_layers: 1,
            format: TextureFormat::RGBA8,
            texture_type: TextureType::Texture2D,
            usage: TextureUsage::default(),
            min_filter: TextureFilter::Linear,
            mag_filter: TextureFilter::Linear,
            wrap_s: TextureWrap::Repeat,
            wrap_t: TextureWrap::Repeat,
            wrap_r: TextureWrap::Repeat,
            border_color: [0.0, 0.0, 0.0, 1.0],
            anisotropy: 1.0,
        }
    }
}

// 纹理数据
#[derive(Debug, Clone)]
pub struct TextureData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_level: u32,
    pub array_layer: u32,
}

// 纹理对象
#[derive(Debug)]
pub struct Texture {
    pub id: TextureId,
    pub name: String,
    pub desc: TextureDesc,
    pub native_handle: Option<u32>, // OpenGL/Vulkan等的原生句柄
    pub size_bytes: u64,
    pub last_used: std::time::Instant,
    pub ref_count: u32,
    pub file_path: Option<PathBuf>,
}

impl Texture {
    pub fn new(id: TextureId, name: String, desc: TextureDesc) -> Self {
        let size_bytes = Self::calculate_size_bytes(&desc);
        
        Self {
            id,
            name,
            desc,
            native_handle: None,
            size_bytes,
            last_used: std::time::Instant::now(),
            ref_count: 1,
            file_path: None,
        }
    }
    
    fn calculate_size_bytes(desc: &TextureDesc) -> u64 {
        let pixel_size = match desc.format {
            TextureFormat::R8 => 1,
            TextureFormat::RG8 => 2,
            TextureFormat::RGB8 => 3,
            TextureFormat::RGBA8 => 4,
            TextureFormat::R16 => 2,
            TextureFormat::RG16 => 4,
            TextureFormat::RGB16 => 6,
            TextureFormat::RGBA16 => 8,
            TextureFormat::R32F => 4,
            TextureFormat::RG32F => 8,
            TextureFormat::RGB32F => 12,
            TextureFormat::RGBA32F => 16,
            _ => 4, // 默认4字节
        };
        
        let mut total_size = 0u64;
        let mut width = desc.width as u64;
        let mut height = desc.height as u64;
        let depth = desc.depth as u64;
        
        // 计算所有mip层级的大小
        for _ in 0..desc.mip_levels {
            total_size += width * height * depth * pixel_size;
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }
        
        total_size * desc.array_layers as u64
    }
    
    pub fn get_dimensions(&self) -> (u32, u32, u32) {
        (self.desc.width, self.desc.height, self.desc.depth)
    }
    
    pub fn is_compressed(&self) -> bool {
        matches!(self.desc.format, 
                TextureFormat::DXT1 | TextureFormat::DXT3 | TextureFormat::DXT5 |
                TextureFormat::RGTC1 | TextureFormat::RGTC2 | TextureFormat::BPTC |
                TextureFormat::ETC2_RGB8 | TextureFormat::ETC2_RGBA8 |
                TextureFormat::ASTC_4x4 | TextureFormat::ASTC_8x8)
    }
    
    pub fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
        self.ref_count += 1;
    }
}

// 纹理管理器
pub struct TextureManager {
    textures: HashMap<TextureId, Texture>,
    texture_cache: HashMap<String, TextureId>,
    next_id: TextureId,
    max_texture_memory: u64,
    current_texture_memory: u64,
    default_texture_id: Option<TextureId>,
    white_texture_id: Option<TextureId>,
    black_texture_id: Option<TextureId>,
    normal_texture_id: Option<TextureId>,
    loading_tasks: HashMap<String, tokio::task::JoinHandle<Result<TextureData>>>,
    texture_atlas: Option<TextureAtlas>,
}

// 纹理图集
#[derive(Debug)]
pub struct TextureAtlas {
    pub texture_id: TextureId,
    pub regions: HashMap<String, AtlasRegion>,
    pub width: u32,
    pub height: u32,
    pub free_space: Vec<AtlasRect>,
}

#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub u1: f32,
    pub v1: f32,
    pub u2: f32,
    pub v2: f32,
}

#[derive(Debug, Clone)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texture_cache: HashMap::new(),
            next_id: 1,
            max_texture_memory: 512 * 1024 * 1024, // 512MB default
            current_texture_memory: 0,
            default_texture_id: None,
            white_texture_id: None,
            black_texture_id: None,
            normal_texture_id: None,
            loading_tasks: HashMap::new(),
            texture_atlas: None,
        }
    }
    
    // 设置最大纹理内存
    pub fn set_max_memory(&mut self, max_bytes: u64) {
        self.max_texture_memory = max_bytes;
        info!("纹理最大内存设置为: {} MB", max_bytes / 1024 / 1024);
    }
    
    // 初始化默认纹理
    pub fn initialize_default_textures(&mut self) -> Result<()> {
        info!("初始化默认纹理");
        
        // 白色纹理
        let white_data = vec![255u8; 4]; // 1x1 RGBA白色
        let white_id = self.create_texture_from_data(
            "default_white",
            &white_data,
            1, 1,
            TextureFormat::RGBA8
        )?;
        self.white_texture_id = Some(white_id);
        
        // 黑色纹理
        let black_data = vec![0u8, 0u8, 0u8, 255u8]; // 1x1 RGBA黑色
        let black_id = self.create_texture_from_data(
            "default_black",
            &black_data,
            1, 1,
            TextureFormat::RGBA8
        )?;
        self.black_texture_id = Some(black_id);
        
        // 默认法线贴图 (0.5, 0.5, 1.0, 1.0) 映射到 (128, 128, 255, 255)
        let normal_data = vec![128u8, 128u8, 255u8, 255u8];
        let normal_id = self.create_texture_from_data(
            "default_normal",
            &normal_data,
            1, 1,
            TextureFormat::RGBA8
        )?;
        self.normal_texture_id = Some(normal_id);
        
        // 默认纹理设为白色纹理
        self.default_texture_id = self.white_texture_id;
        
        Ok(())
    }
    
    // 从文件加载纹理
    pub async fn load_from_file<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<TextureId> {
        let path = path.as_ref();
        
        // 检查缓存
        if let Some(&texture_id) = self.texture_cache.get(name) {
            if let Some(texture) = self.textures.get_mut(&texture_id) {
                texture.touch();
                return Ok(texture_id);
            }
        }
        
        // 检查是否已在加载
        let path_str = path.to_string_lossy().to_string();
        if self.loading_tasks.contains_key(&path_str) {
            // 等待加载完成
            if let Some(task) = self.loading_tasks.remove(&path_str) {
                let texture_data = task.await.map_err(|e| GameError::ResourceError(format!("纹理加载任务失败: {}", e)))??;
                return self.create_texture_from_texture_data(name, texture_data, Some(path.to_path_buf()));
            }
        }
        
        // 开始异步加载
        info!("开始加载纹理: {} 从文件: {:?}", name, path);
        let path_clone = path.to_path_buf();
        let loading_task = tokio::spawn(async move {
            Self::load_texture_data_from_file(path_clone).await
        });
        
        self.loading_tasks.insert(path_str.clone(), loading_task);
        
        // 等待加载完成
        if let Some(task) = self.loading_tasks.remove(&path_str) {
            let texture_data = task.await.map_err(|e| GameError::ResourceError(format!("纹理加载任务失败: {}", e)))??;
            self.create_texture_from_texture_data(name, texture_data, Some(path.to_path_buf()))
        } else {
            Err(GameError::ResourceError("纹理加载任务丢失".to_string()))
        }
    }
    
    // 从内存数据创建纹理
    pub fn create_texture_from_data(
        &mut self,
        name: &str,
        data: &[u8],
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Result<TextureId> {
        let texture_data = TextureData {
            data: data.to_vec(),
            width,
            height,
            format,
            mip_level: 0,
            array_layer: 0,
        };
        
        self.create_texture_from_texture_data(name, texture_data, None)
    }
    
    // 创建空纹理（用作渲染目标）
    pub fn create_render_target(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Result<TextureId> {
        let mut desc = TextureDesc::default();
        desc.width = width;
        desc.height = height;
        desc.format = format;
        desc.usage = TextureUsage {
            read: true,
            write: true,
            render_target: true,
            depth_stencil: matches!(format, 
                TextureFormat::Depth16 | TextureFormat::Depth24 | TextureFormat::Depth32F |
                TextureFormat::Depth24Stencil8 | TextureFormat::Depth32FStencil8),
            generate_mipmaps: false,
        };
        
        let texture_id = self.next_id;
        self.next_id += 1;
        
        let mut texture = Texture::new(texture_id, name.to_string(), desc);
        
        // 创建GPU纹理
        texture.native_handle = Some(self.create_gpu_texture(&texture)?);
        
        // 更新内存统计
        self.current_texture_memory += texture.size_bytes;
        
        self.textures.insert(texture_id, texture);
        self.texture_cache.insert(name.to_string(), texture_id);
        
        info!("创建渲染目标纹理: {} ({}x{}, {:?})", name, width, height, format);
        Ok(texture_id)
    }
    
    // 获取纹理
    pub fn get_texture(&self, texture_id: TextureId) -> Option<&Texture> {
        self.textures.get(&texture_id)
    }
    
    // 获取纹理（可变引用）
    pub fn get_texture_mut(&mut self, texture_id: TextureId) -> Option<&mut Texture> {
        self.textures.get_mut(&texture_id)
    }
    
    // 根据名称获取纹理ID
    pub fn get_texture_id(&self, name: &str) -> Option<TextureId> {
        self.texture_cache.get(name).copied()
    }
    
    // 获取默认纹理ID
    pub fn get_default_texture_id(&self) -> TextureId {
        self.default_texture_id.unwrap_or(1)
    }
    
    // 获取白色纹理ID
    pub fn get_white_texture_id(&self) -> Option<TextureId> {
        self.white_texture_id
    }
    
    // 获取黑色纹理ID
    pub fn get_black_texture_id(&self) -> Option<TextureId> {
        self.black_texture_id
    }
    
    // 获取默认法线贴图ID
    pub fn get_normal_texture_id(&self) -> Option<TextureId> {
        self.normal_texture_id
    }
    
    // 绑定纹理到槽位
    pub fn bind_texture(&mut self, texture_id: TextureId, slot: u32) -> Result<()> {
        if let Some(texture) = self.textures.get_mut(&texture_id) {
            texture.touch();
            
            // TODO: 实际的纹理绑定调用
            debug!("绑定纹理: {} (ID: {}) 到槽位: {}", texture.name, texture_id, slot);
            Ok(())
        } else {
            // 使用默认纹理
            debug!("纹理不存在，使用默认纹理: {}", texture_id);
            Ok(())
        }
    }
    
    // 更新纹理数据
    pub fn update_texture(&mut self, texture_id: TextureId, data: &TextureData) -> Result<()> {
        if let Some(texture) = self.textures.get_mut(&texture_id) {
            // TODO: 实际的纹理更新调用
            texture.touch();
            
            info!("更新纹理数据: {} ({}x{})", texture.name, data.width, data.height);
            Ok(())
        } else {
            Err(GameError::RenderError(format!("纹理不存在: {}", texture_id)))
        }
    }
    
    // 删除纹理
    pub fn delete_texture(&mut self, texture_id: TextureId) -> Result<()> {
        if let Some(texture) = self.textures.remove(&texture_id) {
            // 从缓存中移除
            self.texture_cache.remove(&texture.name);
            
            // 更新内存统计
            self.current_texture_memory = self.current_texture_memory.saturating_sub(texture.size_bytes);
            
            // TODO: 释放GPU资源
            debug!("删除纹理: {} (ID: {})", texture.name, texture_id);
            Ok(())
        } else {
            Err(GameError::RenderError(format!("纹理不存在: {}", texture_id)))
        }
    }
    
    // 内存清理
    pub fn cleanup_unused_textures(&mut self, max_unused_time: std::time::Duration) -> usize {
        let now = std::time::Instant::now();
        let mut to_remove = Vec::new();
        
        for (texture_id, texture) in &self.textures {
            if texture.ref_count == 0 && now.duration_since(texture.last_used) > max_unused_time {
                to_remove.push(*texture_id);
            }
        }
        
        let removed_count = to_remove.len();
        for texture_id in to_remove {
            let _ = self.delete_texture(texture_id);
        }
        
        if removed_count > 0 {
            info!("清理了{}个未使用的纹理", removed_count);
        }
        
        removed_count
    }
    
    // 强制内存清理
    pub fn force_cleanup(&mut self) -> u64 {
        let initial_memory = self.current_texture_memory;
        
        if self.current_texture_memory > self.max_texture_memory {
            // 按最后使用时间排序
            let mut textures_by_usage: Vec<_> = self.textures.iter()
                .filter(|(id, _)| {
                    // 不删除默认纹理
                    !matches!(self.default_texture_id, Some(default_id) if **id == default_id) &&
                    !matches!(self.white_texture_id, Some(white_id) if **id == white_id) &&
                    !matches!(self.black_texture_id, Some(black_id) if **id == black_id) &&
                    !matches!(self.normal_texture_id, Some(normal_id) if **id == normal_id)
                })
                .collect();
            
            textures_by_usage.sort_by_key(|(_, texture)| texture.last_used);
            
            // 删除最旧的纹理直到内存使用低于限制
            let mut deleted = 0;
            for (texture_id, _) in textures_by_usage {
                if self.current_texture_memory <= self.max_texture_memory * 8 / 10 { // 保留80%的空间
                    break;
                }
                
                if let Ok(()) = self.delete_texture(*texture_id) {
                    deleted += 1;
                }
            }
            
            warn!("强制清理了{}个纹理以释放内存", deleted);
        }
        
        initial_memory - self.current_texture_memory
    }
    
    // 获取内存使用统计
    pub fn get_memory_stats(&self) -> (u64, u64, usize) {
        (self.current_texture_memory, self.max_texture_memory, self.textures.len())
    }
    
    // 获取所有纹理信息
    pub fn get_all_textures(&self) -> Vec<(&String, TextureId, &Texture)> {
        self.texture_cache.iter()
            .filter_map(|(name, &id)| {
                self.textures.get(&id).map(|texture| (name, id, texture))
            })
            .collect()
    }
    
    // 清理资源
    pub fn cleanup(&mut self) {
        info!("清理纹理资源，共{}个纹理，内存使用: {} MB", 
              self.textures.len(), 
              self.current_texture_memory / 1024 / 1024);
        
        // TODO: 释放所有GPU资源
        for (texture_id, texture) in &self.textures {
            debug!("释放纹理: {} (ID: {})", texture.name, texture_id);
        }
        
        // 等待所有加载任务完成
        let loading_tasks = std::mem::take(&mut self.loading_tasks);
        for (path, task) in loading_tasks {
            task.abort();
            debug!("取消纹理加载任务: {}", path);
        }
        
        self.textures.clear();
        self.texture_cache.clear();
        self.current_texture_memory = 0;
        self.next_id = 1;
        self.default_texture_id = None;
        self.white_texture_id = None;
        self.black_texture_id = None;
        self.normal_texture_id = None;
    }
    
    // 私有方法
    async fn load_texture_data_from_file(path: PathBuf) -> Result<TextureData> {
        let data = tokio::fs::read(&path).await
            .map_err(|e| GameError::FileNotFound(format!("无法读取纹理文件 {:?}: {}", path, e)))?;
        
        // 根据文件扩展名选择解码器
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "png" => Self::decode_png(&data),
            "jpg" | "jpeg" => Self::decode_jpeg(&data),
            "tga" => Self::decode_tga(&data),
            "dds" => Self::decode_dds(&data),
            "ktx" => Self::decode_ktx(&data),
            _ => {
                warn!("不支持的纹理格式: {}", extension);
                // 尝试使用image库自动检测
                Self::decode_with_image_crate(&data)
            }
        }
    }
    
    fn decode_png(data: &[u8]) -> Result<TextureData> {
        // TODO: 实际的PNG解码
        // 这里应该使用png库或image库
        Ok(TextureData {
            data: vec![255, 0, 255, 255; 64], // 8x8 magenta placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn decode_jpeg(data: &[u8]) -> Result<TextureData> {
        // TODO: 实际的JPEG解码
        Ok(TextureData {
            data: vec![255, 255, 0, 255; 64], // 8x8 yellow placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn decode_tga(data: &[u8]) -> Result<TextureData> {
        // TODO: 实际的TGA解码
        Ok(TextureData {
            data: vec![0, 255, 255, 255; 64], // 8x8 cyan placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn decode_dds(data: &[u8]) -> Result<TextureData> {
        // TODO: 实际的DDS解码（支持压缩格式）
        Ok(TextureData {
            data: vec![255, 128, 0, 255; 64], // 8x8 orange placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn decode_ktx(data: &[u8]) -> Result<TextureData> {
        // TODO: 实际的KTX解码
        Ok(TextureData {
            data: vec![128, 0, 128, 255; 64], // 8x8 purple placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn decode_with_image_crate(data: &[u8]) -> Result<TextureData> {
        // TODO: 使用image库进行通用解码
        Ok(TextureData {
            data: vec![128, 128, 128, 255; 64], // 8x8 gray placeholder
            width: 8,
            height: 8,
            format: TextureFormat::RGBA8,
            mip_level: 0,
            array_layer: 0,
        })
    }
    
    fn create_texture_from_texture_data(
        &mut self,
        name: &str,
        texture_data: TextureData,
        file_path: Option<PathBuf>,
    ) -> Result<TextureId> {
        let texture_id = self.next_id;
        self.next_id += 1;
        
        let mut desc = TextureDesc::default();
        desc.width = texture_data.width;
        desc.height = texture_data.height;
        desc.format = texture_data.format;
        
        let mut texture = Texture::new(texture_id, name.to_string(), desc);
        texture.file_path = file_path;
        
        // 创建GPU纹理并上传数据
        texture.native_handle = Some(self.create_gpu_texture(&texture)?);
        self.upload_texture_data(&texture, &texture_data)?;
        
        // 生成mipmap（如果启用）
        if texture.desc.usage.generate_mipmaps {
            self.generate_mipmaps(&texture)?;
        }
        
        // 更新内存统计
        self.current_texture_memory += texture.size_bytes;
        
        // 检查内存使用
        if self.current_texture_memory > self.max_texture_memory {
            warn!("纹理内存使用超过限制: {} / {} MB", 
                  self.current_texture_memory / 1024 / 1024,
                  self.max_texture_memory / 1024 / 1024);
        }
        
        self.textures.insert(texture_id, texture);
        self.texture_cache.insert(name.to_string(), texture_id);
        
        info!("纹理创建成功: {} (ID: {}, {}x{}, {:?})", 
              name, texture_id, texture_data.width, texture_data.height, texture_data.format);
        
        Ok(texture_id)
    }
    
    fn create_gpu_texture(&self, _texture: &Texture) -> Result<u32> {
        // TODO: 实际的GPU纹理创建
        Ok(fastrand::u32(1000..9999)) // 返回模拟的句柄
    }
    
    fn upload_texture_data(&self, _texture: &Texture, _data: &TextureData) -> Result<()> {
        // TODO: 实际的纹理数据上传
        Ok(())
    }
    
    fn generate_mipmaps(&self, _texture: &Texture) -> Result<()> {
        // TODO: 实际的mipmap生成
        Ok(())
    }
}

impl Drop for TextureManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// 纹理工具函数
pub fn create_checker_texture(size: u32, checker_size: u32) -> TextureData {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let is_white = (checker_x + checker_y) % 2 == 0;
            
            if is_white {
                data.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                data.extend_from_slice(&[128, 128, 128, 255]);
            }
        }
    }
    
    TextureData {
        data,
        width: size,
        height: size,
        format: TextureFormat::RGBA8,
        mip_level: 0,
        array_layer: 0,
    }
}

pub fn create_noise_texture(width: u32, height: u32, seed: u64) -> TextureData {
    fastrand::seed(seed);
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    
    for _ in 0..(width * height) {
        let value = fastrand::u8(..);
        data.extend_from_slice(&[value, value, value, 255]);
    }
    
    TextureData {
        data,
        width,
        height,
        format: TextureFormat::RGBA8,
        mip_level: 0,
        array_layer: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_texture_manager_creation() {
        let manager = TextureManager::new();
        assert_eq!(manager.next_id, 1);
        assert!(manager.textures.is_empty());
        assert_eq!(manager.max_texture_memory, 512 * 1024 * 1024);
    }
    
    #[test]
    fn test_texture_size_calculation() {
        let desc = TextureDesc {
            width: 256,
            height: 256,
            format: TextureFormat::RGBA8,
            mip_levels: 1,
            array_layers: 1,
            ..Default::default()
        };
        
        let size = Texture::calculate_size_bytes(&desc);
        assert_eq!(size, 256 * 256 * 4);
    }
    
    #[test]
    fn test_checker_texture_creation() {
        let texture_data = create_checker_texture(8, 2);
        assert_eq!(texture_data.width, 8);
        assert_eq!(texture_data.height, 8);
        assert_eq!(texture_data.data.len(), 8 * 8 * 4);
        assert_eq!(texture_data.format, TextureFormat::RGBA8);
    }
    
    #[test]
    fn test_texture_format_compression() {
        let texture = Texture::new(1, "test".to_string(), TextureDesc {
            format: TextureFormat::DXT1,
            ..Default::default()
        });
        assert!(texture.is_compressed());
        
        let texture2 = Texture::new(2, "test2".to_string(), TextureDesc {
            format: TextureFormat::RGBA8,
            ..Default::default()
        });
        assert!(!texture2.is_compressed());
    }
}