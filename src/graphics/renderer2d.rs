// 2D渲染器 - 高性能批量渲染系统
// 开发心理：专为2D游戏优化的渲染管线，支持精灵批处理和图层管理
// 设计原则：批处理优化、GPU友好、状态缓存、可调试

use crate::core::{GameError, Result};
use crate::graphics::{Texture, Sprite, RenderQueue, RenderCommand, RenderLayer};
use crate::utils::Color;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Renderer2D {
    // 渲染队列和批处理
    render_queue: RenderQueue,
    sprite_batches: HashMap<Handle<Image>, Vec<SpriteInstance>>,
    
    // 渲染状态
    current_texture: Option<Handle<Image>>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    
    // 统计信息
    stats: RenderStats,
    
    // 设置
    max_sprites_per_batch: usize,
    enable_depth_sorting: bool,
}

#[derive(Debug, Clone)]
struct SpriteInstance {
    pub transform: Transform,
    pub uv: Rect,
    pub color: Color4,
    pub layer: RenderLayer,
}

#[derive(Debug, Clone, Copy)]
struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for Color4 {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

impl From<glam::Vec4> for Color4 {
    fn from(vec: glam::Vec4) -> Self {
        Self {
            r: vec.x,
            g: vec.y,
            b: vec.z,
            a: vec.w,
        }
    }
}

#[derive(Debug, Default)]
pub struct RenderStats {
    pub frame_count: u64,
    pub draw_calls: u32,
    pub sprites_rendered: u32,
    pub vertices_processed: u32,
    pub texture_switches: u32,
    pub batch_count: u32,
}

impl Renderer2D {
    pub fn new() -> Result<Self> {
        info!("初始化2D渲染器");
        
        Ok(Self {
            render_queue: RenderQueue::new(),
            sprite_batches: HashMap::new(),
            current_texture: None,
            vertices: Vec::with_capacity(10000), // 预分配空间
            indices: Vec::with_capacity(15000),
            stats: RenderStats::default(),
            max_sprites_per_batch: 2048,
            enable_depth_sorting: true,
        })
    }
    
    // 开始新的渲染帧
    pub fn begin_frame(&mut self) {
        self.render_queue.clear();
        self.sprite_batches.clear();
        self.vertices.clear();
        self.indices.clear();
        self.current_texture = None;
        
        // 重置帧统计
        self.stats.draw_calls = 0;
        self.stats.sprites_rendered = 0;
        self.stats.vertices_processed = 0;
        self.stats.texture_switches = 0;
        self.stats.batch_count = 0;
    }
    
    // 添加精灵到渲染队列
    pub fn draw_sprite(
        &mut self,
        texture: Handle<Image>,
        position: Vec2,
        size: Vec2,
        rotation: f32,
        color: Color4,
        uv_rect: Option<Rect>,
        layer: RenderLayer,
    ) {
        let transform = Transform::from_translation(position.extend(layer as f32))
            .with_rotation(Quat::from_rotation_z(rotation))
            .with_scale(size.extend(1.0));
        
        let uv = uv_rect.unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0));
        
        let sprite_instance = SpriteInstance {
            transform,
            uv,
            color,
            layer,
        };
        
        // 添加到对应纹理的批处理中
        self.sprite_batches
            .entry(texture)
            .or_insert_with(Vec::new)
            .push(sprite_instance);
    }
    
    // 绘制文本（简化版本）
    pub fn draw_text(
        &mut self,
        text: &str,
        position: Vec2,
        size: f32,
        color: Color4,
        layer: RenderLayer,
    ) -> Result<()> {
        // 简化的文本渲染 - 实际应该使用字体纹理
        debug!("渲染文本: '{}' 在位置 {:?}", text, position);
        Ok(())
    }
    
    // 绘制矩形（调试用）
    pub fn draw_rect(
        &mut self,
        position: Vec2,
        size: Vec2,
        color: Color4,
        filled: bool,
        layer: RenderLayer,
    ) {
        // 使用白色1x1纹理绘制矩形
        // 这里需要一个默认的白色纹理句柄
        debug!("绘制矩形: pos={:?}, size={:?}, filled={}", position, size, filled);
    }

    // 绘制四边形
    pub fn draw_quad(
        &mut self,
        position: Vec2,
        size: Vec2,
        rotation: f32,
        color: Color4,
        layer: RenderLayer,
    ) {
        // 创建一个默认的白色纹理句柄用于纯色四边形
        // 在实际实现中，这应该是一个预加载的1x1白色纹理
        let white_texture = Handle::default(); // 占位符
        
        self.draw_sprite(
            white_texture,
            position,
            size,
            rotation,
            color,
            None, // 使用整个纹理
            layer,
        );
    }
    
    // 结束帧并提交渲染
    pub fn end_frame(&mut self, commands: &mut Commands) -> Result<()> {
        // 按层级和纹理排序批处理
        self.sort_batches();
        
        // 生成渲染命令
        self.generate_render_commands();
        
        // 创建顶点和索引缓冲
        self.build_vertex_buffer();
        
        // 提交到GPU
        self.submit_to_gpu(commands)?;
        
        self.stats.frame_count += 1;
        
        Ok(())
    }
    
    // 按渲染顺序排序批处理
    fn sort_batches(&mut self) {
        if !self.enable_depth_sorting {
            return;
        }
        
        for sprites in self.sprite_batches.values_mut() {
            sprites.sort_by(|a, b| {
                // 先按层级排序，再按Z坐标排序
                a.layer.cmp(&b.layer)
                    .then_with(|| a.transform.translation.z.partial_cmp(&b.transform.translation.z).unwrap())
            });
        }
    }
    
    // 生成渲染命令
    fn generate_render_commands(&mut self) {
        for (texture_handle, sprites) in &self.sprite_batches {
            if sprites.is_empty() {
                continue;
            }
            
            // 将大批次分割成小批次
            for chunk in sprites.chunks(self.max_sprites_per_batch) {
                self.render_queue.commands.push(RenderCommand::DrawSprites {
                    texture: texture_handle.clone(),
                    sprite_count: chunk.len() as u32,
                    start_vertex: self.vertices.len() as u32,
                });
                
                // 为这个批次生成顶点数据
                self.generate_vertices_for_sprites(chunk);
                self.stats.batch_count += 1;
            }
        }
    }
    
    // 为精灵批次生成顶点数据
    fn generate_vertices_for_sprites(&mut self, sprites: &[SpriteInstance]) {
        for sprite in sprites {
            self.generate_quad_vertices(sprite);
            self.stats.sprites_rendered += 1;
        }
    }
    
    // 生成四边形顶点
    fn generate_quad_vertices(&mut self, sprite: &SpriteInstance) {
        let transform = &sprite.transform;
        let uv = &sprite.uv;
        let color = [sprite.color.r, sprite.color.g, sprite.color.b, sprite.color.a];
        
        // 计算四个顶点的世界坐标
        let half_size = transform.scale.truncate() * 0.5;
        let rotation = transform.rotation;
        let position = transform.translation.truncate();
        
        // 本地空间顶点
        let local_vertices = [
            Vec2::new(-half_size.x, -half_size.y), // 左下
            Vec2::new( half_size.x, -half_size.y), // 右下
            Vec2::new( half_size.x,  half_size.y), // 右上
            Vec2::new(-half_size.x,  half_size.y), // 左上
        ];
        
        // UV坐标
        let uvs = [
            Vec2::new(uv.min.x, uv.max.y), // 左下
            Vec2::new(uv.max.x, uv.max.y), // 右下
            Vec2::new(uv.max.x, uv.min.y), // 右上
            Vec2::new(uv.min.x, uv.min.y), // 左上
        ];
        
        let base_index = self.vertices.len() as u32;
        
        // 转换到世界空间并添加顶点
        for (i, &local_pos) in local_vertices.iter().enumerate() {
            let world_pos = position + rotation * local_pos;
            
            self.vertices.push(Vertex {
                position: world_pos.extend(transform.translation.z),
                uv: uvs[i],
                color,
            });
        }
        
        // 添加索引（两个三角形）
        let indices = [
            base_index,     base_index + 1, base_index + 2, // 第一个三角形
            base_index,     base_index + 2, base_index + 3, // 第二个三角形
        ];
        
        self.indices.extend_from_slice(&indices);
        self.stats.vertices_processed += 4;
    }
    
    // 构建顶点缓冲区
    fn build_vertex_buffer(&mut self) {
        // 在实际的Bevy实现中，这里会创建GPU缓冲区
        debug!("构建顶点缓冲区: {} 顶点, {} 索引", self.vertices.len(), self.indices.len());
    }
    
    // 提交到GPU
    fn submit_to_gpu(&mut self, _commands: &mut Commands) -> Result<()> {
        // 在实际的Bevy实现中，这里会提交渲染命令到GPU
        self.stats.draw_calls = self.render_queue.commands.len() as u32;
        
        debug!("提交渲染: {} 绘制调用, {} 精灵", 
               self.stats.draw_calls, self.stats.sprites_rendered);
        
        Ok(())
    }
    
    // 获取渲染统计信息
    pub fn get_stats(&self) -> &RenderStats {
        &self.stats
    }
    
    // 设置最大批处理大小
    pub fn set_max_sprites_per_batch(&mut self, max_sprites: usize) {
        self.max_sprites_per_batch = max_sprites;
    }
    
    // 启用/禁用深度排序
    pub fn set_depth_sorting(&mut self, enable: bool) {
        self.enable_depth_sorting = enable;
    }
    
    // 清空渲染队列
    pub fn clear(&mut self) {
        self.render_queue.clear();
        self.sprite_batches.clear();
    }
    
    // 设置视口
    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        debug!("设置视口: {}x{} at ({}, {})", width, height, x, y);
    }
}

// 渲染层级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderLayer {
    Background = 0,
    Terrain = 10,
    Objects = 20,
    Characters = 30,
    Effects = 40,
    UI = 50,
    Debug = 60,
}

impl Default for RenderLayer {
    fn default() -> Self {
        RenderLayer::Objects
    }
}

// 扩展RenderCommand以支持精灵批处理
#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawMesh { shader_id: u32, texture_id: Option<u32> },
    DrawSprites { texture: Handle<Image>, sprite_count: u32, start_vertex: u32 },
    SetRenderTarget { target_id: Option<u32> },
    Clear { color: Color4 },
    SetViewport { x: i32, y: i32, width: u32, height: u32 },
}

// Bevy系统：2D渲染系统
pub fn sprite_rendering_system(
    mut renderer: ResMut<Renderer2D>,
    mut commands: Commands,
    sprite_query: Query<(
        &Transform,
        &Handle<Image>,
        Option<&Sprite>,
        Option<&RenderLayer>,
    )>,
) {
    renderer.begin_frame();
    
    // 收集所有需要渲染的精灵
    for (transform, texture_handle, sprite_component, layer) in sprite_query.iter() {
        let layer = layer.copied().unwrap_or_default();
        let color = sprite_component
            .map(|s| Color4::from(s.color))
            .unwrap_or(Color4 { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
        
        let uv_rect = sprite_component
            .and_then(|s| s.rect)
            .map(|rect| Rect::new(
                rect.min.x / rect.max.x,
                rect.min.y / rect.max.y,
                rect.max.x / rect.max.x,
                rect.max.y / rect.max.y,
            ));
        
        renderer.draw_sprite(
            texture_handle.clone(),
            transform.translation.truncate(),
            transform.scale.truncate(),
            transform.rotation.to_euler(EulerRot::ZYX).0,
            color,
            uv_rect,
            layer,
        );
    }
    
    // 结束帧并提交渲染
    if let Err(e) = renderer.end_frame(&mut commands) {
        error!("渲染帧失败: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_renderer2d_creation() {
        let renderer = Renderer2D::new().unwrap();
        assert_eq!(renderer.stats.frame_count, 0);
        assert_eq!(renderer.max_sprites_per_batch, 2048);
    }
    
    #[test]
    fn test_render_layer_ordering() {
        assert!(RenderLayer::Background < RenderLayer::UI);
        assert!(RenderLayer::Characters < RenderLayer::Effects);
    }
    
    #[test]
    fn test_color_conversion() {
        let color = Color::RED;
        let color4 = Color4::from(color);
        assert_eq!(color4.r, 1.0);
        assert_eq!(color4.g, 0.0);
        assert_eq!(color4.b, 0.0);
        assert_eq!(color4.a, 1.0);
    }
}