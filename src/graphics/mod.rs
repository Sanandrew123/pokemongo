// 图形渲染系统模块 - 高性能2D/3D渲染引擎
// 开发心理：现代游戏需要美观的视觉效果和流畅的性能
// 设计原则：GPU驱动、批量渲染、资源高效利用、跨平台兼容

pub mod renderer;
pub mod shader;
pub mod texture;
pub mod sprite;
pub mod camera;
pub mod ui;

// 重新导出主要类型
pub use renderer::{Renderer, RenderCommand, RenderQueue};
pub use shader::{Shader, ShaderManager, ShaderProgram, UniformValue};
pub use texture::{Texture, TextureManager, TextureFormat, TextureFilter};
pub use sprite::{Sprite, SpriteRenderer, SpriteBatch, SpriteAnimation};
pub use camera::{Camera, CameraController, Projection};
pub use ui::{UIRenderer, UIElement, UIManager};

use crate::core::{GameError, Result};
use crate::core::resource_manager::{ResourceManager, ResourceHandle};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use log::{info, debug, warn, error};

// 渲染器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererType {
    OpenGL,
    Vulkan,
    DirectX11,
    DirectX12,
    Metal,
    WebGL,
}

// 渲染配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub renderer_type: RendererType,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub max_texture_size: u32,
    pub enable_debug: bool,
    pub enable_wireframe: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            renderer_type: RendererType::OpenGL,
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            vsync: true,
            msaa_samples: 4,
            max_texture_size: 4096,
            enable_debug: cfg!(debug_assertions),
            enable_wireframe: false,
        }
    }
}

// 渲染统计
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderStats {
    pub frame_count: u64,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub texture_switches: u32,
    pub shader_switches: u32,
    pub batches_merged: u32,
    pub gpu_memory_used: u64,
    pub fps: f64,
    pub frame_time_ms: f64,
}

// 渲染层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Background = 0,
    Terrain = 100,
    Objects = 200,
    Characters = 300,
    Effects = 400,
    UI = 500,
    Debug = 1000,
}

// 顶点格式定义
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

// 材质定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<ResourceHandle<Texture>>,
    pub normal_texture: Option<ResourceHandle<Texture>>,
    pub specular_texture: Option<ResourceHandle<Texture>>,
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub shader: ResourceHandle<Shader>,
}

// 网格数据
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub material_index: Option<usize>,
}

// 渲染对象
#[derive(Debug, Clone)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub transform: glam::Mat4,
    pub layer: RenderLayer,
    pub visible: bool,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
}

// 光照类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Light {
    Directional {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
    Point {
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
    },
    Spot {
        position: [f32; 3],
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
    },
}

// 渲染目标
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub color_attachments: Vec<Texture>,
    pub depth_attachment: Option<Texture>,
    pub samples: u32,
}

// 图形上下文
pub struct GraphicsContext {
    pub config: RenderConfig,
    pub stats: RenderStats,
    
    // 渲染组件
    pub renderer: Box<dyn Renderer>,
    pub shader_manager: ShaderManager,
    pub texture_manager: TextureManager,
    pub sprite_renderer: SpriteRenderer,
    pub ui_renderer: UIRenderer,
    
    // 场景数据
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub render_objects: Vec<RenderObject>,
    pub materials: Vec<Material>,
    
    // 渲染队列
    pub render_queue: RenderQueue,
    pub transparent_queue: Vec<RenderCommand>,
    
    // 帧缓冲
    pub main_framebuffer: Option<RenderTarget>,
    pub shadow_maps: HashMap<usize, RenderTarget>,
    
    // 时间相关
    last_frame_time: std::time::Instant,
    frame_times: Vec<f64>,
}

impl GraphicsContext {
    pub fn new(config: RenderConfig) -> Result<Self> {
        info!("初始化图形上下文: {:?}", config.renderer_type);
        
        // 创建渲染器
        let renderer = Self::create_renderer(&config)?;
        
        // 初始化管理器
        let shader_manager = ShaderManager::new();
        let texture_manager = TextureManager::new();
        let sprite_renderer = SpriteRenderer::new()?;
        let ui_renderer = UIRenderer::new()?;
        
        // 创建默认相机
        let camera = Camera::new(
            Projection::Perspective {
                fovy: 60.0_f32.to_radians(),
                aspect: config.window_width as f32 / config.window_height as f32,
                near: 0.1,
                far: 1000.0,
            }
        );
        
        Ok(Self {
            config,
            stats: RenderStats::default(),
            
            renderer,
            shader_manager,
            texture_manager,
            sprite_renderer,
            ui_renderer,
            
            camera,
            lights: Vec::new(),
            render_objects: Vec::new(),
            materials: Vec::new(),
            
            render_queue: RenderQueue::new(),
            transparent_queue: Vec::new(),
            
            main_framebuffer: None,
            shadow_maps: HashMap::new(),
            
            last_frame_time: std::time::Instant::now(),
            frame_times: Vec::with_capacity(120),
        })
    }
    
    fn create_renderer(config: &RenderConfig) -> Result<Box<dyn Renderer>> {
        match config.renderer_type {
            RendererType::OpenGL => {
                // TODO: 创建OpenGL渲染器
                Err(GameError::RenderError("OpenGL渲染器未实现".to_string()))
            },
            RendererType::Vulkan => {
                Err(GameError::RenderError("Vulkan渲染器未实现".to_string()))
            },
            _ => {
                Err(GameError::RenderError(format!("不支持的渲染器类型: {:?}", config.renderer_type)))
            }
        }
    }
    
    // 开始帧渲染
    pub fn begin_frame(&mut self) -> Result<()> {
        self.stats.frame_count += 1;
        self.stats.draw_calls = 0;
        self.stats.vertices_rendered = 0;
        self.stats.triangles_rendered = 0;
        self.stats.texture_switches = 0;
        self.stats.shader_switches = 0;
        self.stats.batches_merged = 0;
        
        // 清空渲染队列
        self.render_queue.clear();
        self.transparent_queue.clear();
        
        // 设置默认渲染状态
        self.renderer.clear_color(0.2, 0.3, 0.8, 1.0)?;
        self.renderer.clear()?;
        
        // 更新相机矩阵
        self.camera.update_matrices()?;
        
        Ok(())
    }
    
    // 结束帧渲染
    pub fn end_frame(&mut self) -> Result<()> {
        // 执行所有渲染命令
        self.execute_render_queue()?;
        
        // 渲染透明对象（从后往前）
        self.render_transparent_objects()?;
        
        // 渲染UI
        self.ui_renderer.render(&mut **self.renderer)?;
        
        // 呈现到屏幕
        self.renderer.present()?;
        
        // 更新性能统计
        self.update_performance_stats();
        
        Ok(())
    }
    
    // 添加渲染对象
    pub fn add_render_object(&mut self, object: RenderObject) {
        self.render_objects.push(object);
    }
    
    // 渲染精灵
    pub fn render_sprite(
        &mut self,
        texture: &ResourceHandle<Texture>,
        position: glam::Vec2,
        size: glam::Vec2,
        rotation: f32,
        color: glam::Vec4,
        layer: RenderLayer,
    ) -> Result<()> {
        self.sprite_renderer.add_sprite(Sprite {
            texture: texture.clone(),
            position,
            size,
            rotation,
            color,
            layer,
            uv_rect: glam::Vec4::new(0.0, 0.0, 1.0, 1.0),
            flip_x: false,
            flip_y: false,
        })?;
        
        Ok(())
    }
    
    // 渲染文本
    pub fn render_text(
        &mut self,
        text: &str,
        position: glam::Vec2,
        font_size: f32,
        color: glam::Vec4,
        layer: RenderLayer,
    ) -> Result<()> {
        self.ui_renderer.add_text(text, position, font_size, color, layer)?;
        Ok(())
    }
    
    // 添加光源
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }
    
    // 设置相机
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
    
    // 裁剪检测
    pub fn is_visible(&self, bounds: &glam::Vec4) -> bool {
        // 简单的视锥裁剪检测
        let view_matrix = self.camera.get_view_matrix();
        let proj_matrix = self.camera.get_projection_matrix();
        let vp_matrix = proj_matrix * view_matrix;
        
        // TODO: 实现更精确的裁剪检测
        true
    }
    
    // 批量处理
    fn execute_render_queue(&mut self) -> Result<()> {
        // 按层级和状态排序
        self.render_queue.sort();
        
        let mut current_shader: Option<u32> = None;
        let mut current_texture: Option<u32> = None;
        let mut batch_size = 0;
        
        for command in self.render_queue.commands.iter() {
            match command {
                RenderCommand::DrawMesh { shader_id, texture_id, .. } => {
                    // 检查是否需要切换状态
                    let mut state_changed = false;
                    
                    if current_shader != Some(*shader_id) {
                        if current_shader.is_some() {
                            self.stats.shader_switches += 1;
                        }
                        current_shader = Some(*shader_id);
                        state_changed = true;
                    }
                    
                    if current_texture != *texture_id {
                        if current_texture.is_some() {
                            self.stats.texture_switches += 1;
                        }
                        current_texture = *texture_id;
                        state_changed = true;
                    }
                    
                    // 如果状态改变且有积累的批次，先渲染
                    if state_changed && batch_size > 0 {
                        self.flush_batch(batch_size)?;
                        batch_size = 0;
                    }
                    
                    batch_size += 1;
                },
                _ => {
                    // 其他命令类型
                    if batch_size > 0 {
                        self.flush_batch(batch_size)?;
                        batch_size = 0;
                    }
                }
            }
        }
        
        // 渲染最后的批次
        if batch_size > 0 {
            self.flush_batch(batch_size)?;
        }
        
        Ok(())
    }
    
    fn flush_batch(&mut self, batch_size: usize) -> Result<()> {
        self.stats.draw_calls += 1;
        
        if batch_size > 1 {
            self.stats.batches_merged += 1;
        }
        
        // TODO: 实际的批量渲染调用
        Ok(())
    }
    
    fn render_transparent_objects(&mut self) -> Result<()> {
        // 按深度排序透明对象
        self.transparent_queue.sort_by(|a, b| {
            // TODO: 实现深度排序
            std::cmp::Ordering::Equal
        });
        
        // 渲染透明对象
        for command in &self.transparent_queue {
            // TODO: 执行透明对象渲染
        }
        
        Ok(())
    }
    
    fn update_performance_stats(&mut self) {
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;
        
        // 更新帧时间历史
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        
        // 计算平均FPS
        if !self.frame_times.is_empty() {
            let avg_frame_time: f64 = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
            self.stats.fps = 1.0 / avg_frame_time;
            self.stats.frame_time_ms = avg_frame_time * 1000.0;
        }
        
        // 每秒输出一次统计信息
        if self.stats.frame_count % 60 == 0 && self.config.enable_debug {
            debug!("渲染统计: FPS: {:.1}, 帧时间: {:.2}ms, 绘制调用: {}, 顶点: {}", 
                   self.stats.fps,
                   self.stats.frame_time_ms,
                   self.stats.draw_calls,
                   self.stats.vertices_rendered);
        }
    }
    
    // 截图功能
    pub fn capture_screenshot(&self) -> Result<Vec<u8>> {
        self.renderer.read_pixels()
    }
    
    // 窗口大小改变
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.config.window_width = width;
        self.config.window_height = height;
        
        // 更新相机宽高比
        if let Projection::Perspective { ref mut aspect, .. } = self.camera.projection {
            *aspect = width as f32 / height as f32;
        }
        
        // 重新创建帧缓冲
        if let Some(ref mut framebuffer) = self.main_framebuffer {
            framebuffer.width = width;
            framebuffer.height = height;
            // TODO: 重新创建纹理
        }
        
        self.renderer.viewport(0, 0, width, height)?;
        
        info!("窗口大小调整: {}x{}", width, height);
        Ok(())
    }
    
    // 设置全屏
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        self.config.fullscreen = fullscreen;
        // TODO: 实现全屏切换
        Ok(())
    }
    
    // 设置垂直同步
    pub fn set_vsync(&mut self, vsync: bool) -> Result<()> {
        self.config.vsync = vsync;
        self.renderer.set_vsync(vsync)?;
        Ok(())
    }
    
    // 获取渲染统计
    pub fn get_stats(&self) -> &RenderStats {
        &self.stats
    }
    
    // 获取GPU内存使用情况
    pub fn get_gpu_memory_usage(&self) -> u64 {
        // TODO: 实现GPU内存使用统计
        0
    }
    
    // 清理资源
    pub fn cleanup(&mut self) {
        info!("清理图形资源");
        
        self.render_objects.clear();
        self.lights.clear();
        self.materials.clear();
        
        self.shader_manager.cleanup();
        self.texture_manager.cleanup();
        
        // TODO: 清理其他GPU资源
    }
}

impl Drop for GraphicsContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// 全局图形上下文
static mut GRAPHICS_CONTEXT: Option<GraphicsContext> = None;
static GRAPHICS_INIT: std::sync::Once = std::sync::Once::new();

pub struct Graphics;

impl Graphics {
    pub fn init(config: RenderConfig) -> Result<()> {
        unsafe {
            GRAPHICS_INIT.call_once(|| {
                match GraphicsContext::new(config) {
                    Ok(context) => {
                        GRAPHICS_CONTEXT = Some(context);
                    },
                    Err(e) => {
                        error!("图形系统初始化失败: {}", e);
                    }
                }
            });
        }
        
        if unsafe { GRAPHICS_CONTEXT.is_none() } {
            return Err(GameError::InitializationFailed("图形系统初始化失败".to_string()));
        }
        
        Ok(())
    }
    
    pub fn instance() -> Result<&'static mut GraphicsContext> {
        unsafe {
            GRAPHICS_CONTEXT.as_mut()
                .ok_or_else(|| GameError::RenderError("图形系统未初始化".to_string()))
        }
    }
    
    pub fn cleanup() {
        unsafe {
            if let Some(ref mut context) = GRAPHICS_CONTEXT {
                context.cleanup();
            }
            GRAPHICS_CONTEXT = None;
        }
    }
}

// 渲染工具函数
pub fn create_quad_mesh() -> Mesh {
    let vertices = vec![
        Vertex3D {
            position: [-0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex3D {
            position: [0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex3D {
            position: [0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex3D {
            position: [-0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];
    
    let indices = vec![0, 1, 2, 2, 3, 0];
    
    Mesh {
        vertices,
        indices,
        material_index: None,
    }
}

pub fn create_cube_mesh() -> Mesh {
    // TODO: 实现立方体网格创建
    Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        material_index: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert_eq!(config.renderer_type, RendererType::OpenGL);
    }
    
    #[test]
    fn test_render_stats_default() {
        let stats = RenderStats::default();
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.fps, 0.0);
    }
    
    #[test]
    fn test_quad_mesh_creation() {
        let mesh = create_quad_mesh();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
    }
}