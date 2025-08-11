/*
 * 渲染系统 - Renderer System
 * 
 * 开发心理过程：
 * 设计高性能的2D/3D混合渲染系统，支持批量渲染、纹理管理、着色器系统等
 * 需要考虑性能优化、内存管理和跨平台兼容性
 * 重点关注渲染管线的效率和扩展性
 */

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment};
use std::collections::HashMap;
use crate::core::error::{GameResult, GameError};
use crate::core::math::{Vec2, Vec3, Matrix4};
use crate::engine::EngineConfig;

// 渲染器配置
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub clear_color: Color,
    pub max_sprites: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
    pub enable_depth_test: bool,
    pub enable_multisampling: bool,
    pub texture_filter: FilterMode,
    pub anisotropy: Option<u8>,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            clear_color: Color::rgb(0.1, 0.1, 0.1),
            max_sprites: 10000,
            max_vertices: 40000,
            max_indices: 60000,
            enable_depth_test: true,
            enable_multisampling: true,
            texture_filter: FilterMode::Linear,
            anisotropy: Some(16),
        }
    }
}

// 顶点数据结构
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

// 渲染批次
#[derive(Debug)]
pub struct RenderBatch {
    pub texture_id: Option<u32>,
    pub shader_id: u32,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub blend_mode: BlendMode,
    pub depth_test: bool,
}

// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    None,
    Alpha,
    Additive,
    Multiply,
    Screen,
}

// 纹理信息
#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub handle: Handle<Image>,
    pub memory_size: usize,
}

// 着色器信息
#[derive(Debug)]
pub struct ShaderInfo {
    pub id: u32,
    pub vertex_shader: Handle<Shader>,
    pub fragment_shader: Handle<Shader>,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

// 渲染统计信息
#[derive(Debug, Default)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub texture_switches: u32,
    pub shader_switches: u32,
    pub batches_merged: u32,
}

// 渲染器主结构
pub struct Renderer {
    config: RendererConfig,
    stats: RenderStats,
    
    // 渲染资源
    textures: HashMap<u32, TextureInfo>,
    shaders: HashMap<u32, ShaderInfo>,
    render_batches: Vec<RenderBatch>,
    
    // 当前状态
    current_texture: Option<u32>,
    current_shader: u32,
    current_blend_mode: BlendMode,
    
    // 矩阵
    view_matrix: Matrix4,
    projection_matrix: Matrix4,
    model_matrix: Matrix4,
    
    // 顶点缓冲区
    vertex_buffer_2d: Vec<Vertex2D>,
    vertex_buffer_3d: Vec<Vertex3D>,
    index_buffer: Vec<u32>,
    
    // 渲染管线状态
    render_pipeline_2d: Option<RenderPipeline>,
    render_pipeline_3d: Option<RenderPipeline>,
    uniform_buffer: Option<Buffer>,
    bind_group_layout: Option<BindGroupLayout>,
    
    // 内存使用统计
    texture_memory: usize,
    buffer_memory: usize,
    
    next_texture_id: u32,
    next_shader_id: u32,
}

impl Renderer {
    // 创建新的渲染器
    pub fn new(engine_config: &EngineConfig) -> GameResult<Self> {
        let config = RendererConfig {
            clear_color: Color::rgb(0.0, 0.0, 0.0),
            ..Default::default()
        };

        Ok(Self {
            config,
            stats: RenderStats::default(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            render_batches: Vec::new(),
            current_texture: None,
            current_shader: 0,
            current_blend_mode: BlendMode::Alpha,
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::orthographic(
                0.0, engine_config.width as f32,
                0.0, engine_config.height as f32,
                -1.0, 1.0
            ),
            model_matrix: Matrix4::identity(),
            vertex_buffer_2d: Vec::with_capacity(config.max_vertices),
            vertex_buffer_3d: Vec::with_capacity(config.max_vertices),
            index_buffer: Vec::with_capacity(config.max_indices),
            render_pipeline_2d: None,
            render_pipeline_3d: None,
            uniform_buffer: None,
            bind_group_layout: None,
            texture_memory: 0,
            buffer_memory: 0,
            next_texture_id: 1,
            next_shader_id: 1,
        })
    }

    // 初始化渲染器
    pub fn initialize(&mut self) -> GameResult<()> {
        info!("初始化渲染器...");
        
        // 创建默认着色器
        self.create_default_shaders()?;
        
        // 设置默认状态
        self.current_shader = 1; // 默认2D着色器
        
        info!("渲染器初始化完成");
        Ok(())
    }

    // 关闭渲染器
    pub fn shutdown(&mut self) -> GameResult<()> {
        info!("关闭渲染器...");
        
        // 清理资源
        self.textures.clear();
        self.shaders.clear();
        self.render_batches.clear();
        
        // 清空缓冲区
        self.vertex_buffer_2d.clear();
        self.vertex_buffer_3d.clear();
        self.index_buffer.clear();
        
        info!("渲染器已关闭");
        Ok(())
    }

    // 开始渲染帧
    pub fn begin_frame(&mut self) -> GameResult<()> {
        // 重置统计信息
        self.stats = RenderStats::default();
        
        // 清空批次
        self.render_batches.clear();
        
        // 重置缓冲区
        self.vertex_buffer_2d.clear();
        self.vertex_buffer_3d.clear();
        self.index_buffer.clear();
        
        // 重置状态
        self.current_texture = None;
        self.current_blend_mode = BlendMode::Alpha;
        
        Ok(())
    }

    // 结束渲染帧
    pub fn end_frame(&mut self) -> GameResult<()> {
        // 提交剩余的批次
        self.flush_batches()?;
        
        // 更新内存统计
        self.update_memory_stats();
        
        Ok(())
    }

    // 设置视图矩阵
    pub fn set_view_matrix(&mut self, matrix: Matrix4) -> GameResult<()> {
        self.view_matrix = matrix;
        Ok(())
    }

    // 设置投影矩阵
    pub fn set_projection_matrix(&mut self, matrix: Matrix4) -> GameResult<()> {
        self.projection_matrix = matrix;
        Ok(())
    }

    // 设置模型矩阵
    pub fn set_model_matrix(&mut self, matrix: Matrix4) -> GameResult<()> {
        self.model_matrix = matrix;
        Ok(())
    }

    // 渲染精灵
    pub fn draw_sprite(&mut self, 
        position: Vec2, 
        size: Vec2, 
        texture_id: Option<u32>,
        color: Color,
        rotation: f32,
        uv_rect: Option<[f32; 4]>
    ) -> GameResult<()> {
        
        // 检查是否需要开始新批次
        if self.should_start_new_batch(texture_id, BlendMode::Alpha) {
            self.flush_current_batch()?;
            self.current_texture = texture_id;
            self.current_blend_mode = BlendMode::Alpha;
        }

        let uv = uv_rect.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let color_array = [color.r(), color.g(), color.b(), color.a()];
        
        // 计算旋转后的顶点位置
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        let half_w = size.x * 0.5;
        let half_h = size.y * 0.5;
        
        let vertices = [
            // 左下
            Vertex2D {
                position: [
                    position.x + (-half_w * cos_r - -half_h * sin_r),
                    position.y + (-half_w * sin_r + -half_h * cos_r)
                ],
                tex_coords: [uv[0], uv[3]],
                color: color_array,
            },
            // 右下
            Vertex2D {
                position: [
                    position.x + (half_w * cos_r - -half_h * sin_r),
                    position.y + (half_w * sin_r + -half_h * cos_r)
                ],
                tex_coords: [uv[2], uv[3]],
                color: color_array,
            },
            // 右上
            Vertex2D {
                position: [
                    position.x + (half_w * cos_r - half_h * sin_r),
                    position.y + (half_w * sin_r + half_h * cos_r)
                ],
                tex_coords: [uv[2], uv[1]],
                color: color_array,
            },
            // 左上
            Vertex2D {
                position: [
                    position.x + (-half_w * cos_r - half_h * sin_r),
                    position.y + (-half_w * sin_r + half_h * cos_r)
                ],
                tex_coords: [uv[0], uv[1]],
                color: color_array,
            },
        ];

        let base_index = self.vertex_buffer_2d.len() as u32;
        let indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];

        // 添加到缓冲区
        self.vertex_buffer_2d.extend_from_slice(&vertices);
        self.index_buffer.extend_from_slice(&indices);

        // 更新统计信息
        self.stats.vertices_rendered += 4;
        self.stats.triangles_rendered += 2;

        Ok(())
    }

    // 渲染矩形
    pub fn draw_rect(&mut self, 
        position: Vec2, 
        size: Vec2, 
        color: Color,
        filled: bool
    ) -> GameResult<()> {
        
        if filled {
            self.draw_sprite(position, size, None, color, 0.0, None)
        } else {
            // 绘制矩形边框
            let line_width = 1.0;
            
            // 上边
            self.draw_sprite(
                Vec2::new(position.x, position.y + size.y - line_width * 0.5),
                Vec2::new(size.x, line_width),
                None, color, 0.0, None
            )?;
            
            // 下边
            self.draw_sprite(
                Vec2::new(position.x, position.y + line_width * 0.5),
                Vec2::new(size.x, line_width),
                None, color, 0.0, None
            )?;
            
            // 左边
            self.draw_sprite(
                Vec2::new(position.x + line_width * 0.5, position.y + size.y * 0.5),
                Vec2::new(line_width, size.y),
                None, color, 0.0, None
            )?;
            
            // 右边
            self.draw_sprite(
                Vec2::new(position.x + size.x - line_width * 0.5, position.y + size.y * 0.5),
                Vec2::new(line_width, size.y),
                None, color, 0.0, None
            )?;
            
            Ok(())
        }
    }

    // 渲染圆形
    pub fn draw_circle(&mut self, 
        center: Vec2, 
        radius: f32, 
        color: Color,
        segments: u32,
        filled: bool
    ) -> GameResult<()> {
        
        let segments = segments.max(8).min(64);
        let angle_step = std::f32::consts::TAU / segments as f32;
        
        if filled {
            // 扇形填充
            let center_vertex = Vertex2D {
                position: [center.x, center.y],
                tex_coords: [0.5, 0.5],
                color: [color.r(), color.g(), color.b(), color.a()],
            };
            
            let base_index = self.vertex_buffer_2d.len() as u32;
            self.vertex_buffer_2d.push(center_vertex);
            
            for i in 0..=segments {
                let angle = i as f32 * angle_step;
                let x = center.x + radius * angle.cos();
                let y = center.y + radius * angle.sin();
                
                let vertex = Vertex2D {
                    position: [x, y],
                    tex_coords: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
                    color: [color.r(), color.g(), color.b(), color.a()],
                };
                
                self.vertex_buffer_2d.push(vertex);
                
                if i > 0 {
                    self.index_buffer.extend_from_slice(&[
                        base_index,
                        base_index + i,
                        base_index + i + 1,
                    ]);
                    self.stats.triangles_rendered += 1;
                }
            }
            
            self.stats.vertices_rendered += segments + 1;
        } else {
            // 线框圆
            for i in 0..segments {
                let angle1 = i as f32 * angle_step;
                let angle2 = ((i + 1) % segments) as f32 * angle_step;
                
                let x1 = center.x + radius * angle1.cos();
                let y1 = center.y + radius * angle1.sin();
                let x2 = center.x + radius * angle2.cos();
                let y2 = center.y + radius * angle2.sin();
                
                self.draw_line(Vec2::new(x1, y1), Vec2::new(x2, y2), 1.0, color)?;
            }
        }
        
        Ok(())
    }

    // 渲染线段
    pub fn draw_line(&mut self, 
        start: Vec2, 
        end: Vec2, 
        width: f32, 
        color: Color
    ) -> GameResult<()> {
        
        let direction = end - start;
        let length = direction.length();
        let normalized = direction / length;
        let perpendicular = Vec2::new(-normalized.y, normalized.x) * (width * 0.5);
        
        let vertices = [
            Vertex2D {
                position: [(start - perpendicular).x, (start - perpendicular).y],
                tex_coords: [0.0, 0.0],
                color: [color.r(), color.g(), color.b(), color.a()],
            },
            Vertex2D {
                position: [(start + perpendicular).x, (start + perpendicular).y],
                tex_coords: [1.0, 0.0],
                color: [color.r(), color.g(), color.b(), color.a()],
            },
            Vertex2D {
                position: [(end + perpendicular).x, (end + perpendicular).y],
                tex_coords: [1.0, 1.0],
                color: [color.r(), color.g(), color.b(), color.a()],
            },
            Vertex2D {
                position: [(end - perpendicular).x, (end - perpendicular).y],
                tex_coords: [0.0, 1.0],
                color: [color.r(), color.g(), color.b(), color.a()],
            },
        ];

        let base_index = self.vertex_buffer_2d.len() as u32;
        let indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];

        self.vertex_buffer_2d.extend_from_slice(&vertices);
        self.index_buffer.extend_from_slice(&indices);

        self.stats.vertices_rendered += 4;
        self.stats.triangles_rendered += 2;

        Ok(())
    }

    // 渲染文本
    pub fn draw_text(&mut self, 
        text: &str, 
        position: Vec2, 
        font_size: f32, 
        color: Color
    ) -> GameResult<()> {
        // 简化实现：渲染每个字符为小矩形
        let char_width = font_size * 0.6;
        let char_spacing = font_size * 0.1;
        
        for (i, ch) in text.chars().enumerate() {
            let char_pos = Vec2::new(
                position.x + i as f32 * (char_width + char_spacing),
                position.y
            );
            
            // 这里应该使用字体纹理，简化为矩形
            self.draw_rect(
                char_pos,
                Vec2::new(char_width, font_size),
                color,
                true
            )?;
        }
        
        Ok(())
    }

    // 渲染调试文本
    pub fn render_debug_text(&mut self, text: &str, x: f32, y: f32) -> GameResult<()> {
        self.draw_text(text, Vec2::new(x, y), 16.0, Color::WHITE)
    }

    // 渲染场景
    pub fn render_scene(&mut self, scene: &mut crate::engine::scene::Scene) -> GameResult<()> {
        // 遍历场景中的所有渲染组件
        // 这里简化实现
        info!("渲染场景");
        Ok(())
    }

    // 加载纹理
    pub fn load_texture(&mut self, path: &str, image_handle: Handle<Image>) -> GameResult<u32> {
        let texture_id = self.next_texture_id;
        self.next_texture_id += 1;

        let texture_info = TextureInfo {
            id: texture_id,
            width: 256, // 这里应该从实际图像获取
            height: 256,
            format: TextureFormat::Rgba8UnormSrgb,
            handle: image_handle,
            memory_size: 256 * 256 * 4,
        };

        self.texture_memory += texture_info.memory_size;
        self.textures.insert(texture_id, texture_info);

        Ok(texture_id)
    }

    // 卸载纹理
    pub fn unload_texture(&mut self, texture_id: u32) -> GameResult<()> {
        if let Some(texture_info) = self.textures.remove(&texture_id) {
            self.texture_memory -= texture_info.memory_size;
        }
        Ok(())
    }

    // 设置混合模式
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) -> GameResult<()> {
        if self.current_blend_mode != blend_mode {
            self.flush_current_batch()?;
            self.current_blend_mode = blend_mode;
        }
        Ok(())
    }

    // 设置着色器
    pub fn set_shader(&mut self, shader_id: u32) -> GameResult<()> {
        if self.current_shader != shader_id {
            self.flush_current_batch()?;
            self.current_shader = shader_id;
            self.stats.shader_switches += 1;
        }
        Ok(())
    }

    // 设置窗口大小
    pub fn resize(&mut self, width: u32, height: u32) -> GameResult<()> {
        self.projection_matrix = Matrix4::orthographic(
            0.0, width as f32,
            0.0, height as f32,
            -1.0, 1.0
        );
        Ok(())
    }

    // 设置全屏
    pub fn set_fullscreen(&mut self, _fullscreen: bool) -> GameResult<()> {
        // 实现全屏切换
        Ok(())
    }

    // 设置垂直同步
    pub fn set_vsync(&mut self, _vsync: bool) -> GameResult<()> {
        // 实现垂直同步设置
        Ok(())
    }

    // 设置多重采样
    pub fn set_msaa_samples(&mut self, _samples: u32) -> GameResult<()> {
        // 实现MSAA设置
        Ok(())
    }

    // 获取渲染统计信息
    pub fn get_draw_calls(&self) -> u32 {
        self.stats.draw_calls
    }

    pub fn get_triangle_count(&self) -> u32 {
        self.stats.triangles_rendered
    }

    pub fn get_texture_memory_usage(&self) -> usize {
        self.texture_memory
    }

    pub fn get_memory_usage(&self) -> usize {
        self.texture_memory + self.buffer_memory
    }

    // 私有辅助方法
    fn should_start_new_batch(&self, texture_id: Option<u32>, blend_mode: BlendMode) -> bool {
        self.current_texture != texture_id || 
        self.current_blend_mode != blend_mode ||
        self.vertex_buffer_2d.len() + 4 > self.config.max_vertices ||
        self.index_buffer.len() + 6 > self.config.max_indices
    }

    fn flush_current_batch(&mut self) -> GameResult<()> {
        if !self.vertex_buffer_2d.is_empty() {
            // 这里应该提交当前批次到GPU
            self.stats.draw_calls += 1;
            
            // 清空缓冲区准备下一批次
            self.vertex_buffer_2d.clear();
            self.index_buffer.clear();
        }
        Ok(())
    }

    fn flush_batches(&mut self) -> GameResult<()> {
        self.flush_current_batch()?;
        Ok(())
    }

    fn create_default_shaders(&mut self) -> GameResult<()> {
        // 创建默认的2D和3D着色器
        info!("创建默认着色器");
        Ok(())
    }

    fn update_memory_stats(&mut self) {
        self.buffer_memory = 
            self.vertex_buffer_2d.len() * std::mem::size_of::<Vertex2D>() +
            self.vertex_buffer_3d.len() * std::mem::size_of::<Vertex3D>() +
            self.index_buffer.len() * std::mem::size_of::<u32>();
    }
}

// 实现顶点布局
impl Vertex2D {
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex2D>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Vertex3D {
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex3D>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 3]>() * 2) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 3]>() * 2 + mem::size_of::<[f32; 2]>()) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// 混合模式转换
impl From<BlendMode> for BlendState {
    fn from(mode: BlendMode) -> Self {
        match mode {
            BlendMode::None => BlendState::REPLACE,
            BlendMode::Alpha => BlendState::ALPHA_BLENDING,
            BlendMode::Additive => BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            },
            BlendMode::Multiply => BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::Dst,
                    dst_factor: BlendFactor::Zero,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            },
            BlendMode::Screen => BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::OneMinusDst,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            },
        }
    }
}

// 渲染器扩展
impl Renderer {
    // 批量渲染精灵
    pub fn draw_sprite_batch(&mut self, sprites: &[SpriteInstance]) -> GameResult<()> {
        for sprite in sprites {
            self.draw_sprite(
                sprite.position,
                sprite.size,
                sprite.texture_id,
                sprite.color,
                sprite.rotation,
                sprite.uv_rect,
            )?;
        }
        Ok(())
    }

    // 渲染带遮罩的精灵
    pub fn draw_masked_sprite(&mut self, 
        position: Vec2,
        size: Vec2,
        texture_id: u32,
        mask_id: u32,
        color: Color
    ) -> GameResult<()> {
        // 实现遮罩渲染
        self.draw_sprite(position, size, Some(texture_id), color, 0.0, None)
    }

    // 渲染九宫格精灵
    pub fn draw_nine_patch(&mut self,
        position: Vec2,
        size: Vec2,
        texture_id: u32,
        border_widths: [f32; 4], // top, right, bottom, left
        color: Color
    ) -> GameResult<()> {
        // 实现九宫格渲染逻辑
        let texture_info = self.textures.get(&texture_id).ok_or_else(|| {
            GameError::Render("纹理不存在".to_string())
        })?;

        let tex_w = texture_info.width as f32;
        let tex_h = texture_info.height as f32;

        // 分割成9个区域进行渲染
        // 这里简化实现
        self.draw_sprite(position, size, Some(texture_id), color, 0.0, None)
    }
}

// 精灵实例
#[derive(Debug, Clone)]
pub struct SpriteInstance {
    pub position: Vec2,
    pub size: Vec2,
    pub texture_id: Option<u32>,
    pub color: Color,
    pub rotation: f32,
    pub uv_rect: Option<[f32; 4]>,
}

impl Default for SpriteInstance {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            texture_id: None,
            color: Color::WHITE,
            rotation: 0.0,
            uv_rect: None,
        }
    }
}