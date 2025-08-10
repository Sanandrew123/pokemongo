// 着色器系统
// 开发心理：着色器是现代图形编程的核心，需要高效的编译和管理系统
// 设计原则：平台无关、热重载支持、统一的接口抽象

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceHandle, ResourceManager};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use log::{info, debug, warn, error};
use std::path::{Path, PathBuf};

pub type ShaderId = u32;

// 着色器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    Compute,
    TessellationControl,
    TessellationEvaluation,
}

// 着色器阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex = 0x00000001,
    Fragment = 0x00000010,
    Geometry = 0x00000100,
    Compute = 0x00001000,
    TessellationControl = 0x00010000,
    TessellationEvaluation = 0x00100000,
}

// 着色器源码
#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub vertex_source: String,
    pub fragment_source: String,
    pub geometry_source: Option<String>,
    pub compute_source: Option<String>,
    pub includes: Vec<String>,
}

// 着色器程序
#[derive(Debug)]
pub struct ShaderProgram {
    pub id: ShaderId,
    pub name: String,
    pub stages: u32, // 位掩码表示包含的阶段
    pub uniforms: HashMap<String, UniformInfo>,
    pub attributes: HashMap<String, AttributeInfo>,
    pub native_handle: Option<u32>, // OpenGL/Vulkan等的原生句柄
    pub last_modified: std::time::SystemTime,
    pub file_paths: Vec<PathBuf>,
}

// Uniform信息
#[derive(Debug, Clone)]
pub struct UniformInfo {
    pub name: String,
    pub location: i32,
    pub uniform_type: UniformType,
    pub size: usize,
    pub count: usize,
}

// Attribute信息
#[derive(Debug, Clone)]
pub struct AttributeInfo {
    pub name: String,
    pub location: i32,
    pub attribute_type: AttributeType,
    pub size: usize,
}

// Uniform类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniformType {
    Bool,
    Int,
    UInt,
    Float,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
    Sampler3D,
    SamplerArray,
}

// Attribute类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    IVec2,
    IVec3,
    IVec4,
    UInt,
    UVec2,
    UVec3,
    UVec4,
}

// Uniform值
#[derive(Debug, Clone)]
pub enum UniformValue {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    IVec2(glam::IVec2),
    IVec3(glam::IVec3),
    IVec4(glam::IVec4),
    UVec2(glam::UVec2),
    UVec3(glam::UVec3),
    UVec4(glam::UVec4),
    Mat2(glam::Mat2),
    Mat3(glam::Mat3),
    Mat4(glam::Mat4),
    Texture(u32),
    TextureArray(Vec<u32>),
}

// 着色器编译错误
#[derive(Debug, Clone)]
pub struct ShaderCompileError {
    pub shader_type: ShaderType,
    pub error_message: String,
    pub line_number: Option<u32>,
    pub source_file: Option<PathBuf>,
}

// 着色器管理器
pub struct ShaderManager {
    shaders: HashMap<ShaderId, ShaderProgram>,
    shader_cache: HashMap<String, ShaderId>,
    next_id: ShaderId,
    shader_root_path: PathBuf,
    include_cache: HashMap<String, String>,
    watch_files: bool,
    file_watcher: Option<FileWatcher>,
}

// 文件监视器（简化实现）
pub struct FileWatcher {
    watched_files: HashMap<PathBuf, std::time::SystemTime>,
}

impl FileWatcher {
    fn new() -> Self {
        Self {
            watched_files: HashMap::new(),
        }
    }
    
    fn watch_file(&mut self, path: PathBuf) -> Result<()> {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.watched_files.insert(path, modified);
            }
        }
        Ok(())
    }
    
    fn check_changes(&self) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();
        
        for (path, &old_time) in &self.watched_files {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(new_time) = metadata.modified() {
                    if new_time > old_time {
                        changed_files.push(path.clone());
                    }
                }
            }
        }
        
        changed_files
    }
}

impl ShaderManager {
    pub fn new<P: AsRef<Path>>(shader_root_path: P) -> Self {
        Self {
            shaders: HashMap::new(),
            shader_cache: HashMap::new(),
            next_id: 1,
            shader_root_path: shader_root_path.as_ref().to_path_buf(),
            include_cache: HashMap::new(),
            watch_files: cfg!(debug_assertions),
            file_watcher: if cfg!(debug_assertions) {
                Some(FileWatcher::new())
            } else {
                None
            },
        }
    }
    
    // 从文件加载着色器
    pub fn load_from_file<P: AsRef<Path>>(&mut self, name: &str, vertex_path: P, fragment_path: P) -> Result<ShaderId> {
        let vertex_path = vertex_path.as_ref();
        let fragment_path = fragment_path.as_ref();
        
        // 检查缓存
        if let Some(&shader_id) = self.shader_cache.get(name) {
            return Ok(shader_id);
        }
        
        // 读取着色器源码
        let vertex_source = self.read_shader_file(vertex_path)?;
        let fragment_source = self.read_shader_file(fragment_path)?;
        
        let shader_source = ShaderSource {
            vertex_source,
            fragment_source,
            geometry_source: None,
            compute_source: None,
            includes: Vec::new(),
        };
        
        // 编译着色器
        let shader_id = self.create_shader_program(name, shader_source)?;
        
        // 设置文件路径用于热重载
        if let Some(shader) = self.shaders.get_mut(&shader_id) {
            shader.file_paths = vec![vertex_path.to_path_buf(), fragment_path.to_path_buf()];
            
            // 添加到文件监视器
            if let Some(ref mut watcher) = self.file_watcher {
                watcher.watch_file(vertex_path.to_path_buf())?;
                watcher.watch_file(fragment_path.to_path_buf())?;
            }
        }
        
        // 缓存着色器ID
        self.shader_cache.insert(name.to_string(), shader_id);
        
        info!("着色器加载成功: {} (ID: {})", name, shader_id);
        Ok(shader_id)
    }
    
    // 从源码创建着色器
    pub fn create_from_source(&mut self, name: &str, source: ShaderSource) -> Result<ShaderId> {
        if let Some(&shader_id) = self.shader_cache.get(name) {
            return Ok(shader_id);
        }
        
        let shader_id = self.create_shader_program(name, source)?;
        self.shader_cache.insert(name.to_string(), shader_id);
        
        info!("着色器创建成功: {} (ID: {})", name, shader_id);
        Ok(shader_id)
    }
    
    // 获取着色器程序
    pub fn get_shader(&self, shader_id: ShaderId) -> Option<&ShaderProgram> {
        self.shaders.get(&shader_id)
    }
    
    // 获取着色器程序（可变引用）
    pub fn get_shader_mut(&mut self, shader_id: ShaderId) -> Option<&mut ShaderProgram> {
        self.shaders.get_mut(&shader_id)
    }
    
    // 根据名称获取着色器ID
    pub fn get_shader_id(&self, name: &str) -> Option<ShaderId> {
        self.shader_cache.get(name).copied()
    }
    
    // 使用着色器
    pub fn use_shader(&self, shader_id: ShaderId) -> Result<()> {
        if let Some(shader) = self.shaders.get(&shader_id) {
            // TODO: 实际的着色器绑定调用
            debug!("使用着色器: {} (ID: {})", shader.name, shader_id);
            Ok(())
        } else {
            Err(GameError::RenderError(format!("着色器不存在: {}", shader_id)))
        }
    }
    
    // 设置uniform值
    pub fn set_uniform(&mut self, shader_id: ShaderId, name: &str, value: UniformValue) -> Result<()> {
        if let Some(shader) = self.shaders.get(&shader_id) {
            if let Some(uniform_info) = shader.uniforms.get(name) {
                // 检查类型匹配
                if !self.is_uniform_type_compatible(&uniform_info.uniform_type, &value) {
                    return Err(GameError::RenderError(
                        format!("Uniform类型不匹配: {} 期望 {:?}，得到 {:?}",
                               name, uniform_info.uniform_type, self.get_uniform_value_type(&value))
                    ));
                }
                
                // TODO: 实际的uniform设置调用
                debug!("设置uniform: {} = {:?}", name, value);
                Ok(())
            } else {
                Err(GameError::RenderError(format!("Uniform不存在: {}", name)))
            }
        } else {
            Err(GameError::RenderError(format!("着色器不存在: {}", shader_id)))
        }
    }
    
    // 批量设置uniform
    pub fn set_uniforms(&mut self, shader_id: ShaderId, uniforms: &[(String, UniformValue)]) -> Result<()> {
        for (name, value) in uniforms {
            self.set_uniform(shader_id, name, value.clone())?;
        }
        Ok(())
    }
    
    // 检查文件更改并重新加载
    pub fn check_for_changes(&mut self) -> Result<Vec<ShaderId>> {
        let mut reloaded_shaders = Vec::new();
        
        if !self.watch_files {
            return Ok(reloaded_shaders);
        }
        
        if let Some(ref watcher) = self.file_watcher {
            let changed_files = watcher.check_changes();
            
            if !changed_files.is_empty() {
                debug!("检测到着色器文件更改: {:?}", changed_files);
                
                // 找到需要重新加载的着色器
                let mut shaders_to_reload = Vec::new();
                for (shader_id, shader) in &self.shaders {
                    for changed_file in &changed_files {
                        if shader.file_paths.contains(changed_file) {
                            shaders_to_reload.push(*shader_id);
                            break;
                        }
                    }
                }
                
                // 重新加载着色器
                for shader_id in shaders_to_reload {
                    if let Some(shader) = self.shaders.get(&shader_id) {
                        let name = shader.name.clone();
                        let file_paths = shader.file_paths.clone();
                        
                        if file_paths.len() >= 2 {
                            // 移除旧的缓存条目
                            self.shader_cache.remove(&name);
                            
                            // 重新加载
                            match self.load_from_file(&name, &file_paths[0], &file_paths[1]) {
                                Ok(_) => {
                                    info!("着色器热重载成功: {}", name);
                                    reloaded_shaders.push(shader_id);
                                },
                                Err(e) => {
                                    error!("着色器热重载失败: {}: {}", name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(reloaded_shaders)
    }
    
    // 删除着色器
    pub fn delete_shader(&mut self, shader_id: ShaderId) -> Result<()> {
        if let Some(shader) = self.shaders.remove(&shader_id) {
            // 从缓存中移除
            self.shader_cache.remove(&shader.name);
            
            // TODO: 释放GPU资源
            debug!("删除着色器: {} (ID: {})", shader.name, shader_id);
            Ok(())
        } else {
            Err(GameError::RenderError(format!("着色器不存在: {}", shader_id)))
        }
    }
    
    // 获取所有着色器信息
    pub fn get_all_shaders(&self) -> Vec<(&String, ShaderId)> {
        self.shader_cache.iter().map(|(name, &id)| (name, id)).collect()
    }
    
    // 清理资源
    pub fn cleanup(&mut self) {
        info!("清理着色器资源，共{}个着色器", self.shaders.len());
        
        // TODO: 释放所有GPU资源
        for (shader_id, shader) in &self.shaders {
            debug!("释放着色器: {} (ID: {})", shader.name, shader_id);
        }
        
        self.shaders.clear();
        self.shader_cache.clear();
        self.include_cache.clear();
        self.next_id = 1;
    }
    
    // 私有方法
    fn read_shader_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let path = path.as_ref();
        let full_path = if path.is_relative() {
            self.shader_root_path.join(path)
        } else {
            path.to_path_buf()
        };
        
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| GameError::FileNotFound(format!("无法读取着色器文件 {:?}: {}", full_path, e)))?;
        
        // 处理#include指令
        self.process_includes(&content, &full_path.parent().unwrap_or(Path::new(".")))
    }
    
    fn process_includes(&mut self, source: &str, base_path: &Path) -> Result<String> {
        let mut result = String::new();
        
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                // 解析include路径
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed.rfind('"') {
                        if start < end {
                            let include_path = &trimmed[start + 1..end];
                            let full_include_path = base_path.join(include_path);
                            
                            // 检查include缓存
                            let cache_key = full_include_path.to_string_lossy().to_string();
                            let include_content = if let Some(cached) = self.include_cache.get(&cache_key) {
                                cached.clone()
                            } else {
                                let content = std::fs::read_to_string(&full_include_path)
                                    .map_err(|e| GameError::FileNotFound(
                                        format!("无法读取include文件 {:?}: {}", full_include_path, e)))?;
                                
                                self.include_cache.insert(cache_key, content.clone());
                                content
                            };
                            
                            result.push_str(&format!("// === Include: {} ===\n", include_path));
                            result.push_str(&include_content);
                            result.push_str("\n// === End Include ===\n");
                            continue;
                        }
                    }
                }
                
                warn!("无效的#include指令: {}", line);
            }
            
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }
    
    fn create_shader_program(&mut self, name: &str, source: ShaderSource) -> Result<ShaderId> {
        let shader_id = self.next_id;
        self.next_id += 1;
        
        // 编译各个阶段
        let mut stages = 0u32;
        
        // 顶点着色器
        if !source.vertex_source.is_empty() {
            self.compile_shader_stage(ShaderType::Vertex, &source.vertex_source)?;
            stages |= ShaderStage::Vertex as u32;
        }
        
        // 片段着色器
        if !source.fragment_source.is_empty() {
            self.compile_shader_stage(ShaderType::Fragment, &source.fragment_source)?;
            stages |= ShaderStage::Fragment as u32;
        }
        
        // 几何着色器
        if let Some(ref geo_source) = source.geometry_source {
            if !geo_source.is_empty() {
                self.compile_shader_stage(ShaderType::Geometry, geo_source)?;
                stages |= ShaderStage::Geometry as u32;
            }
        }
        
        // 计算着色器
        if let Some(ref compute_source) = source.compute_source {
            if !compute_source.is_empty() {
                self.compile_shader_stage(ShaderType::Compute, compute_source)?;
                stages |= ShaderStage::Compute as u32;
            }
        }
        
        // 链接程序
        let native_handle = self.link_shader_program(stages)?;
        
        // 反射uniform和attribute信息
        let (uniforms, attributes) = self.reflect_shader_interface(native_handle)?;
        
        // 创建着色器程序对象
        let shader_program = ShaderProgram {
            id: shader_id,
            name: name.to_string(),
            stages,
            uniforms,
            attributes,
            native_handle: Some(native_handle),
            last_modified: std::time::SystemTime::now(),
            file_paths: Vec::new(),
        };
        
        self.shaders.insert(shader_id, shader_program);
        
        debug!("着色器程序创建成功: {} (ID: {}, 句柄: {})", name, shader_id, native_handle);
        Ok(shader_id)
    }
    
    fn compile_shader_stage(&self, shader_type: ShaderType, source: &str) -> Result<u32> {
        // TODO: 实际的着色器编译
        // 这里应该调用OpenGL/Vulkan等API进行编译
        
        debug!("编译着色器阶段: {:?}", shader_type);
        
        // 模拟编译过程
        if source.contains("ERROR") {
            return Err(GameError::RenderError("着色器编译错误".to_string()));
        }
        
        Ok(fastrand::u32(1000..9999)) // 返回模拟的句柄
    }
    
    fn link_shader_program(&self, _stages: u32) -> Result<u32> {
        // TODO: 实际的着色器链接
        debug!("链接着色器程序");
        Ok(fastrand::u32(1000..9999)) // 返回模拟的程序句柄
    }
    
    fn reflect_shader_interface(&self, _program_handle: u32) -> Result<(HashMap<String, UniformInfo>, HashMap<String, AttributeInfo>)> {
        // TODO: 实际的着色器反射
        let mut uniforms = HashMap::new();
        let mut attributes = HashMap::new();
        
        // 添加一些常用的uniform
        uniforms.insert("u_mvp_matrix".to_string(), UniformInfo {
            name: "u_mvp_matrix".to_string(),
            location: 0,
            uniform_type: UniformType::Mat4,
            size: std::mem::size_of::<glam::Mat4>(),
            count: 1,
        });
        
        uniforms.insert("u_texture".to_string(), UniformInfo {
            name: "u_texture".to_string(),
            location: 1,
            uniform_type: UniformType::Sampler2D,
            size: std::mem::size_of::<u32>(),
            count: 1,
        });
        
        // 添加一些常用的attribute
        attributes.insert("a_position".to_string(), AttributeInfo {
            name: "a_position".to_string(),
            location: 0,
            attribute_type: AttributeType::Vec3,
            size: std::mem::size_of::<glam::Vec3>(),
        });
        
        attributes.insert("a_texcoord".to_string(), AttributeInfo {
            name: "a_texcoord".to_string(),
            location: 1,
            attribute_type: AttributeType::Vec2,
            size: std::mem::size_of::<glam::Vec2>(),
        });
        
        Ok((uniforms, attributes))
    }
    
    fn is_uniform_type_compatible(&self, expected: &UniformType, value: &UniformValue) -> bool {
        match (expected, value) {
            (UniformType::Bool, UniformValue::Bool(_)) => true,
            (UniformType::Int, UniformValue::Int(_)) => true,
            (UniformType::UInt, UniformValue::UInt(_)) => true,
            (UniformType::Float, UniformValue::Float(_)) => true,
            (UniformType::Vec2, UniformValue::Vec2(_)) => true,
            (UniformType::Vec3, UniformValue::Vec3(_)) => true,
            (UniformType::Vec4, UniformValue::Vec4(_)) => true,
            (UniformType::Mat4, UniformValue::Mat4(_)) => true,
            (UniformType::Sampler2D, UniformValue::Texture(_)) => true,
            _ => false,
        }
    }
    
    fn get_uniform_value_type(&self, value: &UniformValue) -> &'static str {
        match value {
            UniformValue::Bool(_) => "Bool",
            UniformValue::Int(_) => "Int",
            UniformValue::UInt(_) => "UInt",
            UniformValue::Float(_) => "Float",
            UniformValue::Vec2(_) => "Vec2",
            UniformValue::Vec3(_) => "Vec3",
            UniformValue::Vec4(_) => "Vec4",
            UniformValue::Mat4(_) => "Mat4",
            UniformValue::Texture(_) => "Texture",
            _ => "Unknown",
        }
    }
}

impl Drop for ShaderManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// 便利函数
pub fn create_basic_vertex_fragment_source(vertex_code: &str, fragment_code: &str) -> ShaderSource {
    ShaderSource {
        vertex_source: vertex_code.to_string(),
        fragment_source: fragment_code.to_string(),
        geometry_source: None,
        compute_source: None,
        includes: Vec::new(),
    }
}

// 内置着色器源码
pub mod builtin_shaders {
    use super::*;
    
    pub const BASIC_2D_VERTEX: &str = r#"
#version 330 core

layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_texcoord;
layout (location = 2) in vec4 a_color;

uniform mat4 u_mvp_matrix;

out vec2 v_texcoord;
out vec4 v_color;

void main() {
    gl_Position = u_mvp_matrix * vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord;
    v_color = a_color;
}
"#;

    pub const BASIC_2D_FRAGMENT: &str = r#"
#version 330 core

in vec2 v_texcoord;
in vec4 v_color;

uniform sampler2D u_texture;

out vec4 fragColor;

void main() {
    vec4 texColor = texture(u_texture, v_texcoord);
    fragColor = texColor * v_color;
}
"#;

    pub const SOLID_COLOR_VERTEX: &str = r#"
#version 330 core

layout (location = 0) in vec2 a_position;

uniform mat4 u_mvp_matrix;

void main() {
    gl_Position = u_mvp_matrix * vec4(a_position, 0.0, 1.0);
}
"#;

    pub const SOLID_COLOR_FRAGMENT: &str = r#"
#version 330 core

uniform vec4 u_color;

out vec4 fragColor;

void main() {
    fragColor = u_color;
}
"#;

    pub fn load_builtin_shaders(manager: &mut ShaderManager) -> Result<HashMap<String, ShaderId>> {
        let mut builtin_shader_ids = HashMap::new();
        
        // 基础2D着色器
        let basic_2d_source = create_basic_vertex_fragment_source(BASIC_2D_VERTEX, BASIC_2D_FRAGMENT);
        let basic_2d_id = manager.create_from_source("basic_2d", basic_2d_source)?;
        builtin_shader_ids.insert("basic_2d".to_string(), basic_2d_id);
        
        // 纯色着色器
        let solid_color_source = create_basic_vertex_fragment_source(SOLID_COLOR_VERTEX, SOLID_COLOR_FRAGMENT);
        let solid_color_id = manager.create_from_source("solid_color", solid_color_source)?;
        builtin_shader_ids.insert("solid_color".to_string(), solid_color_id);
        
        info!("内置着色器加载完成，共{}个", builtin_shader_ids.len());
        Ok(builtin_shader_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_manager_creation() {
        let manager = ShaderManager::new("assets/shaders");
        assert_eq!(manager.next_id, 1);
        assert!(manager.shaders.is_empty());
    }
    
    #[test]
    fn test_uniform_type_compatibility() {
        let manager = ShaderManager::new(".");
        
        assert!(manager.is_uniform_type_compatible(&UniformType::Float, &UniformValue::Float(1.0)));
        assert!(manager.is_uniform_type_compatible(&UniformType::Vec3, &UniformValue::Vec3(glam::Vec3::ZERO)));
        assert!(manager.is_uniform_type_compatible(&UniformType::Mat4, &UniformValue::Mat4(glam::Mat4::IDENTITY)));
        
        assert!(!manager.is_uniform_type_compatible(&UniformType::Float, &UniformValue::Int(1)));
        assert!(!manager.is_uniform_type_compatible(&UniformType::Vec2, &UniformValue::Vec3(glam::Vec3::ZERO)));
    }
    
    #[test]
    fn test_shader_source_creation() {
        let source = create_basic_vertex_fragment_source("vertex code", "fragment code");
        assert_eq!(source.vertex_source, "vertex code");
        assert_eq!(source.fragment_source, "fragment code");
        assert!(source.geometry_source.is_none());
    }
}