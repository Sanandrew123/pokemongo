// 地图系统
// 开发心理：地图是游戏世界的基础，需要分层渲染、碰撞检测、动态加载
// 设计原则：分块管理、层级渲染、碰撞优化、内存控制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
#[cfg(feature = "graphics-wip")]
use crate::graphics::renderer::Renderer2D;
#[cfg(feature = "graphics-wip")]
use crate::graphics::sprite::SpriteManager;

// 临时类型定义，直到graphics模块可用
#[cfg(not(feature = "graphics-wip"))]
#[derive(Debug)]
pub struct Renderer2D;

#[cfg(not(feature = "graphics-wip"))]
#[derive(Debug)]
pub struct SpriteManager;

#[cfg(not(feature = "graphics-wip"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlendMode;

#[cfg(not(feature = "graphics-wip"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[cfg(not(feature = "graphics-wip"))]
impl Default for BlendMode {
    fn default() -> Self {
        Self
    }
}

#[cfg(not(feature = "graphics-wip"))]
impl BlendMode {
    pub const Alpha: BlendMode = BlendMode;
}
use glam::{Vec2, Vec3, Vec4};

// 地图ID和坐标类型
pub type MapId = u32;
pub type ChunkId = u64;
pub type TileId = u32;

// 游戏地图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMap {
    pub id: MapId,
    pub name: String,
    pub description: String,
    pub size: Vec2,                     // 地图尺寸(世界坐标)
    pub tile_size: Vec2,                // 瓦片尺寸
    pub chunk_size: Vec2,               // 分块尺寸
    
    // 地图层级
    pub layers: Vec<MapLayer>,
    
    // 分块系统
    pub chunks: HashMap<ChunkId, MapChunk>,
    pub loaded_chunks: Vec<ChunkId>,
    
    // 碰撞系统
    pub collision_map: CollisionMap,
    
    // 传送点和连接
    pub warp_points: HashMap<String, WarpPoint>,
    pub connections: Vec<MapConnection>,
    
    // 地图属性
    pub properties: HashMap<String, String>,
    pub spawn_points: HashMap<String, Vec3>,
    
    // 环境设置
    pub background_color: Vec4,
    pub ambient_light: Vec4,
    pub music: Option<String>,
    pub weather_override: Option<crate::world::Weather>,
    
    // 动态内容
    pub dynamic_objects: HashMap<u64, DynamicObject>,
    pub next_object_id: u64,
}

// 地图层级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayer {
    pub id: u32,
    pub name: String,
    pub layer_type: LayerType,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: f32,
    pub parallax_factor: Vec2,          // 视差因子
    
    // 瓦片数据
    pub tiles: HashMap<(i32, i32), TileData>,
    pub tileset: Option<String>,        // 瓦片集名称
    
    // 渲染属性
    #[cfg(feature = "graphics-wip")]
    pub blend_mode: crate::graphics::renderer::BlendMode,
    #[cfg(not(feature = "graphics-wip"))]
    pub blend_mode: BlendMode,
    pub tint_color: Vec4,
    pub scroll_speed: Vec2,             // 自动滚动速度
}

// 层级类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Background,     // 背景层
    Terrain,        // 地形层
    Objects,        // 物体层
    Collision,      // 碰撞层
    Decoration,     // 装饰层
    Foreground,     // 前景层
    UI,            // UI层
}

// 瓦片数据
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TileData {
    pub tile_id: TileId,
    pub variation: u8,              // 变化版本
    pub rotation: u8,               // 旋转(0-3, 90度增量)
    pub flip_x: bool,
    pub flip_y: bool,
    pub properties: u32,            // 属性位掩码
}

// 地图分块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapChunk {
    pub id: ChunkId,
    pub position: Vec2,             // 分块位置
    pub size: Vec2,                 // 分块尺寸
    pub loaded: bool,
    pub dirty: bool,                // 需要重新渲染
    
    // 分块内容
    pub tiles: HashMap<u32, HashMap<(i32, i32), TileData>>, // layer_id -> tiles
    pub objects: Vec<u64>,          // 该分块内的动态对象ID
    pub collision_data: Vec<CollisionTile>,
    
    // 渲染优化
    #[cfg(feature = "graphics-wip")]
    pub vertex_buffer: Option<Vec<crate::graphics::renderer::Vertex>>,
    #[cfg(not(feature = "graphics-wip"))]
    pub vertex_buffer: Option<Vec<Vertex>>,
    pub texture_atlas: Option<u32>,
    pub last_rendered_frame: u64,
}

// 碰撞地图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionMap {
    pub tiles: HashMap<(i32, i32), CollisionTile>,
    pub grid_size: Vec2,            // 碰撞网格尺寸
    pub collision_layers: u32,      // 碰撞层位掩码
}

// 碰撞瓦片
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollisionTile {
    pub collision_type: CollisionType,
    pub solid: bool,
    pub one_way: bool,              // 单向碰撞
    pub trigger: bool,              // 触发器
    pub elevation: f32,             // 高度
    pub friction: f32,              // 摩擦力
    pub bounce: f32,                // 反弹力
}

// 碰撞类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionType {
    None,           // 无碰撞
    Solid,          // 实心
    Platform,       // 平台(可从下方穿过)
    Water,          // 水面
    Grass,          // 草地
    Sand,           // 沙地
    Ice,            // 冰面
    Lava,           // 岩浆
    Trigger,        // 触发区域
}

// 传送点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpPoint {
    pub name: String,
    pub position: Vec3,
    pub target_map: Option<MapId>,
    pub target_position: Vec3,
    pub direction: Vec2,            // 传送后朝向
    pub requirements: Vec<String>,   // 传送要求
    pub sound_effect: Option<String>,
    pub animation: Option<String>,
}

// 地图连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConnection {
    pub from_map: MapId,
    pub to_map: MapId,
    pub connection_type: ConnectionType,
    pub trigger_area: (Vec2, Vec2),  // 触发区域(位置, 尺寸)
    pub spawn_point: Vec3,           // 目标生成点
    pub transition_type: String,     // 过渡类型
}

// 连接类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Door,           // 门
    Stairs,         // 楼梯
    Cave,           // 洞穴
    Bridge,         // 桥梁
    Teleporter,     // 传送器
}

// 动态对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicObject {
    pub id: u64,
    pub object_type: String,
    pub position: Vec3,
    pub scale: Vec2,
    pub rotation: f32,
    pub sprite_id: Option<u32>,
    pub animation: Option<String>,
    pub properties: HashMap<String, String>,
    pub active: bool,
    pub persistent: bool,           // 是否持久保存
    pub spawn_conditions: Vec<String>, // 生成条件
    pub despawn_timer: Option<f32>, // 消失计时器
}

// 地图管理器
pub struct MapManager {
    // 当前加载的地图
    current_map: Option<GameMap>,
    
    // 地图缓存
    map_cache: HashMap<MapId, GameMap>,
    
    // 渲染系统
    sprite_manager: SpriteManager,
    
    // 分块管理
    chunk_load_radius: f32,         // 分块加载半径
    chunk_unload_radius: f32,       // 分块卸载半径
    max_chunks_per_frame: usize,    // 每帧最大处理分块数
    
    // 瓦片集管理
    tilesets: HashMap<String, Tileset>,
    
    // 渲染优化
    frustum_culling: bool,
    occlusion_culling: bool,
    batch_rendering: bool,
    
    // 统计信息
    chunks_loaded: usize,
    tiles_rendered: u32,
    draw_calls: u32,
    frame_count: u64,
}

// 瓦片集
#[derive(Debug, Clone)]
pub struct Tileset {
    pub name: String,
    pub texture_id: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub margin: u32,
    pub spacing: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub tile_properties: HashMap<TileId, TileProperties>,
}

// 瓦片属性
#[derive(Debug, Clone, Default)]
pub struct TileProperties {
    pub solid: bool,
    pub one_way: bool,
    pub animated: bool,
    pub animation_frames: Vec<AnimationFrame>,
    pub custom_properties: HashMap<String, String>,
}

// 动画帧
#[derive(Debug, Clone, Copy)]
pub struct AnimationFrame {
    pub tile_id: TileId,
    pub duration: f32,
}

impl GameMap {
    pub fn new(id: MapId, name: String, size: Vec2) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            size,
            tile_size: Vec2::new(32.0, 32.0),
            chunk_size: Vec2::new(512.0, 512.0),
            layers: Vec::new(),
            chunks: HashMap::new(),
            loaded_chunks: Vec::new(),
            collision_map: CollisionMap {
                tiles: HashMap::new(),
                grid_size: Vec2::new(32.0, 32.0),
                collision_layers: 0xFFFFFFFF,
            },
            warp_points: HashMap::new(),
            connections: Vec::new(),
            properties: HashMap::new(),
            spawn_points: HashMap::new(),
            background_color: Vec4::new(0.5, 0.7, 1.0, 1.0),
            ambient_light: Vec4::new(1.0, 1.0, 1.0, 1.0),
            music: None,
            weather_override: None,
            dynamic_objects: HashMap::new(),
            next_object_id: 1,
        }
    }
    
    // 添加地图层
    pub fn add_layer(&mut self, layer_type: LayerType, name: String, z_order: i32) -> u32 {
        let layer_id = self.layers.len() as u32;
        
        let layer = MapLayer {
            id: layer_id,
            name,
            layer_type,
            z_order,
            visible: true,
            opacity: 1.0,
            parallax_factor: Vec2::ONE,
            tiles: HashMap::new(),
            tileset: None,
            #[cfg(feature = "graphics-wip")]
            blend_mode: crate::graphics::renderer::BlendMode::Alpha,
            #[cfg(not(feature = "graphics-wip"))]
            blend_mode: BlendMode::Alpha,
            tint_color: Vec4::ONE,
            scroll_speed: Vec2::ZERO,
        };
        
        self.layers.push(layer);
        
        // 按Z顺序排序
        self.layers.sort_by_key(|l| l.z_order);
        
        debug!("添加地图层: '{}' 类型={:?} Z={}", self.layers[layer_id as usize].name, layer_type, z_order);
        layer_id
    }
    
    // 设置瓦片
    pub fn set_tile(&mut self, layer_id: u32, x: i32, y: i32, tile_data: TileData) -> Result<(), GameError> {
        if let Some(layer) = self.layers.get_mut(layer_id as usize) {
            layer.tiles.insert((x, y), tile_data);
            
            // 更新对应分块
            let chunk_id = self.get_chunk_id_for_position(Vec2::new(x as f32 * self.tile_size.x, y as f32 * self.tile_size.y));
            if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                chunk.dirty = true;
                chunk.tiles.entry(layer_id).or_insert_with(HashMap::new).insert((x, y), tile_data);
            }
            
            Ok(())
        } else {
            Err(GameError::Map(format!("图层不存在: {}", layer_id)))
        }
    }
    
    // 获取瓦片
    pub fn get_tile(&self, layer_id: u32, x: i32, y: i32) -> Option<TileData> {
        self.layers.get(layer_id as usize)?.tiles.get(&(x, y)).copied()
    }
    
    // 添加碰撞瓦片
    pub fn set_collision(&mut self, x: i32, y: i32, collision: CollisionTile) {
        self.collision_map.tiles.insert((x, y), collision);
        
        // 更新分块碰撞数据
        let chunk_id = self.get_chunk_id_for_position(Vec2::new(x as f32 * self.tile_size.x, y as f32 * self.tile_size.y));
        if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
            // 更新分块碰撞数据
            chunk.collision_data.push(collision);
        }
    }
    
    // 检查碰撞
    pub fn check_collision(&self, position: Vec2, size: Vec2) -> Option<CollisionTile> {
        let tile_x = (position.x / self.tile_size.x) as i32;
        let tile_y = (position.y / self.tile_size.y) as i32;
        
        // 检查占用的所有瓦片
        let tiles_x = ((size.x / self.tile_size.x).ceil() as i32).max(1);
        let tiles_y = ((size.y / self.tile_size.y).ceil() as i32).max(1);
        
        for dy in 0..tiles_y {
            for dx in 0..tiles_x {
                if let Some(collision) = self.collision_map.tiles.get(&(tile_x + dx, tile_y + dy)) {
                    if collision.solid {
                        return Some(*collision);
                    }
                }
            }
        }
        
        None
    }
    
    // 添加传送点
    pub fn add_warp_point(&mut self, name: String, warp: WarpPoint) {
        self.warp_points.insert(name.clone(), warp);
        debug!("添加传送点: '{}'", name);
    }
    
    // 检查传送点触发
    pub fn check_warp_trigger(&self, position: Vec3, radius: f32) -> Option<&WarpPoint> {
        for warp in self.warp_points.values() {
            let distance = (warp.position - position).length();
            if distance <= radius {
                return Some(warp);
            }
        }
        None
    }
    
    // 添加动态对象
    pub fn add_dynamic_object(&mut self, object_type: String, position: Vec3) -> u64 {
        let object_id = self.next_object_id;
        self.next_object_id += 1;
        
        let object = DynamicObject {
            id: object_id,
            object_type,
            position,
            scale: Vec2::ONE,
            rotation: 0.0,
            sprite_id: None,
            animation: None,
            properties: HashMap::new(),
            active: true,
            persistent: false,
            spawn_conditions: Vec::new(),
            despawn_timer: None,
        };
        
        self.dynamic_objects.insert(object_id, object);
        debug!("添加动态对象: ID={}", object_id);
        
        object_id
    }
    
    // 移除动态对象
    pub fn remove_dynamic_object(&mut self, object_id: u64) -> bool {
        if self.dynamic_objects.remove(&object_id).is_some() {
            debug!("移除动态对象: ID={}", object_id);
            true
        } else {
            false
        }
    }
    
    // 更新动态对象
    pub fn update_dynamic_objects(&mut self, delta_time: f32) {
        let mut objects_to_remove = Vec::new();
        
        for (object_id, object) in &mut self.dynamic_objects {
            if !object.active {
                continue;
            }
            
            // 更新消失计时器
            if let Some(ref mut timer) = object.despawn_timer {
                *timer -= delta_time;
                if *timer <= 0.0 {
                    objects_to_remove.push(*object_id);
                    continue;
                }
            }
            
            // 更新其他动态属性
            // 这里可以添加移动、动画等逻辑
        }
        
        // 移除过期对象
        for object_id in objects_to_remove {
            self.remove_dynamic_object(object_id);
        }
    }
    
    // 获取指定位置的分块ID
    fn get_chunk_id_for_position(&self, position: Vec2) -> ChunkId {
        let chunk_x = (position.x / self.chunk_size.x) as i64;
        let chunk_y = (position.y / self.chunk_size.y) as i64;
        
        // 将分块坐标编码为单个ID
        ((chunk_x as u64) << 32) | (chunk_y as u64 & 0xFFFFFFFF)
    }
    
    // 从分块ID解码坐标
    fn decode_chunk_id(&self, chunk_id: ChunkId) -> (i64, i64) {
        let chunk_x = (chunk_id >> 32) as i64;
        let chunk_y = (chunk_id & 0xFFFFFFFF) as i64;
        (chunk_x, chunk_y)
    }
    
    // 创建新分块
    fn create_chunk(&mut self, chunk_id: ChunkId) -> &mut MapChunk {
        let (chunk_x, chunk_y) = self.decode_chunk_id(chunk_id);
        let position = Vec2::new(chunk_x as f32 * self.chunk_size.x, chunk_y as f32 * self.chunk_size.y);
        
        let chunk = MapChunk {
            id: chunk_id,
            position,
            size: self.chunk_size,
            loaded: false,
            dirty: true,
            tiles: HashMap::new(),
            objects: Vec::new(),
            collision_data: Vec::new(),
            vertex_buffer: None,
            texture_atlas: None,
            last_rendered_frame: 0,
        };
        
        self.chunks.insert(chunk_id, chunk);
        self.chunks.get_mut(&chunk_id).unwrap()
    }
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            current_map: None,
            map_cache: HashMap::new(),
            sprite_manager: SpriteManager::new(),
            chunk_load_radius: 1000.0,
            chunk_unload_radius: 1500.0,
            max_chunks_per_frame: 4,
            tilesets: HashMap::new(),
            frustum_culling: true,
            occlusion_culling: false,
            batch_rendering: true,
            chunks_loaded: 0,
            tiles_rendered: 0,
            draw_calls: 0,
            frame_count: 0,
        }
    }
    
    // 加载地图
    pub fn load_map(&mut self, map_id: MapId) -> Result<(), GameError> {
        // 尝试从缓存加载
        if let Some(map) = self.map_cache.get(&map_id).cloned() {
            self.current_map = Some(map);
            debug!("从缓存加载地图: ID={}", map_id);
            return Ok(());
        }
        
        // 创建默认地图(实际应该从文件加载)
        let mut map = GameMap::new(map_id, format!("Map_{}", map_id), Vec2::new(2000.0, 2000.0));
        
        // 添加基础层级
        map.add_layer(LayerType::Background, "背景".to_string(), -100);
        map.add_layer(LayerType::Terrain, "地形".to_string(), 0);
        map.add_layer(LayerType::Objects, "物体".to_string(), 100);
        map.add_layer(LayerType::Foreground, "前景".to_string(), 200);
        
        self.current_map = Some(map.clone());
        self.map_cache.insert(map_id, map);
        
        debug!("加载地图: ID={}", map_id);
        Ok(())
    }
    
    // 更新分块加载
    pub fn update_chunk_loading(&mut self, camera_position: Vec2) -> Result<(), GameError> {
        if let Some(ref mut map) = self.current_map {
            // 计算需要加载的分块
            let chunks_to_load = self.calculate_chunks_in_radius(camera_position, self.chunk_load_radius, map);
            let chunks_to_unload = self.calculate_chunks_to_unload(camera_position, self.chunk_unload_radius, map);
            
            // 卸载远距离分块
            for chunk_id in chunks_to_unload {
                if let Some(chunk) = map.chunks.get_mut(&chunk_id) {
                    chunk.loaded = false;
                    chunk.vertex_buffer = None;
                    self.chunks_loaded -= 1;
                }
                map.loaded_chunks.retain(|&id| id != chunk_id);
            }
            
            // 加载新分块(限制每帧处理数量)
            let mut loaded_this_frame = 0;
            for chunk_id in chunks_to_load {
                if loaded_this_frame >= self.max_chunks_per_frame {
                    break;
                }
                
                if !map.chunks.contains_key(&chunk_id) {
                    map.create_chunk(chunk_id);
                }
                
                if let Some(chunk) = map.chunks.get_mut(&chunk_id) {
                    if !chunk.loaded {
                        self.load_chunk_data(chunk)?;
                        chunk.loaded = true;
                        self.chunks_loaded += 1;
                        loaded_this_frame += 1;
                        
                        if !map.loaded_chunks.contains(&chunk_id) {
                            map.loaded_chunks.push(chunk_id);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    // 渲染地图
    pub fn render(&mut self, renderer: &mut Renderer2D, camera_position: Vec2) -> Result<(), GameError> {
        self.frame_count += 1;
        self.tiles_rendered = 0;
        self.draw_calls = 0;
        
        if let Some(ref map) = self.current_map {
            // 清除背景
            renderer.clear(map.background_color)?;
            
            // 按层级顺序渲染
            for layer in &map.layers {
                if !layer.visible {
                    continue;
                }
                
                self.render_layer(renderer, layer, map, camera_position)?;
            }
            
            // 渲染动态对象
            self.render_dynamic_objects(renderer, map, camera_position)?;
        }
        
        Ok(())
    }
    
    // 更新地图系统
    pub fn update(&mut self, delta_time: f32, camera_position: Vec2) -> Result<(), GameError> {
        // 更新分块加载
        self.update_chunk_loading(camera_position)?;
        
        // 更新精灵管理器
        self.sprite_manager.update(delta_time)?;
        
        // 更新动态对象
        if let Some(ref mut map) = self.current_map {
            map.update_dynamic_objects(delta_time);
        }
        
        Ok(())
    }
    
    // 获取当前地图
    pub fn get_current_map(&self) -> Option<&GameMap> {
        self.current_map.as_ref()
    }
    
    // 获取当前地图(可变)
    pub fn get_current_map_mut(&mut self) -> Option<&mut GameMap> {
        self.current_map.as_mut()
    }
    
    // 私有方法
    fn calculate_chunks_in_radius(&self, center: Vec2, radius: f32, map: &GameMap) -> Vec<ChunkId> {
        let mut chunks = Vec::new();
        
        let chunks_x = (radius / map.chunk_size.x).ceil() as i32 + 1;
        let chunks_y = (radius / map.chunk_size.y).ceil() as i32 + 1;
        
        let center_chunk_x = (center.x / map.chunk_size.x) as i32;
        let center_chunk_y = (center.y / map.chunk_size.y) as i32;
        
        for dy in -chunks_y..=chunks_y {
            for dx in -chunks_x..=chunks_x {
                let chunk_x = center_chunk_x + dx;
                let chunk_y = center_chunk_y + dy;
                
                let chunk_center = Vec2::new(
                    chunk_x as f32 * map.chunk_size.x + map.chunk_size.x * 0.5,
                    chunk_y as f32 * map.chunk_size.y + map.chunk_size.y * 0.5,
                );
                
                if (chunk_center - center).length() <= radius {
                    let chunk_id = ((chunk_x as u64) << 32) | (chunk_y as u64 & 0xFFFFFFFF);
                    chunks.push(chunk_id);
                }
            }
        }
        
        chunks
    }
    
    fn calculate_chunks_to_unload(&self, center: Vec2, radius: f32, map: &GameMap) -> Vec<ChunkId> {
        let mut chunks_to_unload = Vec::new();
        
        for &chunk_id in &map.loaded_chunks {
            if let Some(chunk) = map.chunks.get(&chunk_id) {
                let chunk_center = chunk.position + chunk.size * 0.5;
                if (chunk_center - center).length() > radius {
                    chunks_to_unload.push(chunk_id);
                }
            }
        }
        
        chunks_to_unload
    }
    
    fn load_chunk_data(&self, chunk: &mut MapChunk) -> Result<(), GameError> {
        // 这里应该从文件或生成器加载分块数据
        // 简化实现：生成一些测试数据
        
        debug!("加载分块数据: ID={}", chunk.id);
        Ok(())
    }
    
    fn render_layer(
        &mut self,
        renderer: &mut Renderer2D,
        layer: &MapLayer,
        map: &GameMap,
        camera_position: Vec2,
    ) -> Result<(), GameError> {
        // 计算视差偏移
        let parallax_offset = camera_position * (Vec2::ONE - layer.parallax_factor);
        
        // 渲染可见的瓦片
        for (&(x, y), &tile_data) in &layer.tiles {
            let tile_position = Vec2::new(
                x as f32 * map.tile_size.x - parallax_offset.x,
                y as f32 * map.tile_size.y - parallax_offset.y,
            );
            
            // 视锥剔除
            if self.frustum_culling && !self.is_tile_visible(tile_position, map.tile_size, camera_position) {
                continue;
            }
            
            self.render_tile(renderer, tile_data, tile_position, map.tile_size, layer)?;
            self.tiles_rendered += 1;
        }
        
        self.draw_calls += 1;
        Ok(())
    }
    
    fn render_tile(
        &self,
        renderer: &mut Renderer2D,
        tile_data: TileData,
        position: Vec2,
        size: Vec2,
        layer: &MapLayer,
    ) -> Result<(), GameError> {
        if tile_data.tile_id == 0 {
            return Ok(()); // 空瓦片
        }
        
        // 简化的瓦片渲染
        let color = layer.tint_color * layer.opacity;
        
        renderer.draw_quad(
            position,
            size,
            1, // 默认纹理
            color,
            tile_data.rotation as f32 * std::f32::consts::PI / 2.0,
        )?;
        
        Ok(())
    }
    
    fn render_dynamic_objects(
        &mut self,
        renderer: &mut Renderer2D,
        map: &GameMap,
        camera_position: Vec2,
    ) -> Result<(), GameError> {
        for object in map.dynamic_objects.values() {
            if !object.active {
                continue;
            }
            
            let screen_position = Vec2::new(object.position.x - camera_position.x, object.position.z - camera_position.y);
            
            if let Some(sprite_id) = object.sprite_id {
                renderer.draw_sprite(
                    screen_position,
                    object.scale * 32.0, // 假设基础尺寸32x32
                    sprite_id,
                    None,
                    Vec4::ONE,
                    object.rotation,
                    false,
                    false,
                )?;
            } else {
                // 绘制默认表示
                renderer.draw_quad(
                    screen_position,
                    object.scale * 32.0,
                    1,
                    Vec4::new(1.0, 0.0, 1.0, 1.0),
                    object.rotation,
                )?;
            }
        }
        
        Ok(())
    }
    
    fn is_tile_visible(&self, tile_position: Vec2, tile_size: Vec2, camera_position: Vec2) -> bool {
        // 简化的视锥剔除
        let screen_bounds = (
            Vec2::new(-400.0, -300.0), // 假设屏幕尺寸800x600
            Vec2::new(400.0, 300.0)
        );
        
        let relative_pos = tile_position - camera_position;
        
        relative_pos.x + tile_size.x >= screen_bounds.0.x &&
        relative_pos.x <= screen_bounds.1.x &&
        relative_pos.y + tile_size.y >= screen_bounds.0.y &&
        relative_pos.y <= screen_bounds.1.y
    }
}

// 默认实现
impl Default for TileData {
    fn default() -> Self {
        Self {
            tile_id: 0,
            variation: 0,
            rotation: 0,
            flip_x: false,
            flip_y: false,
            properties: 0,
        }
    }
}

impl Default for CollisionTile {
    fn default() -> Self {
        Self {
            collision_type: CollisionType::None,
            solid: false,
            one_way: false,
            trigger: false,
            elevation: 0.0,
            friction: 1.0,
            bounce: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_map_creation() {
        let map = GameMap::new(1, "测试地图".to_string(), Vec2::new(1000.0, 1000.0));
        assert_eq!(map.id, 1);
        assert_eq!(map.name, "测试地图");
        assert_eq!(map.size, Vec2::new(1000.0, 1000.0));
        assert_eq!(map.layers.len(), 0);
    }
    
    #[test]
    fn test_map_layers() {
        let mut map = GameMap::new(1, "测试".to_string(), Vec2::new(1000.0, 1000.0));
        
        let layer_id = map.add_layer(LayerType::Terrain, "地形层".to_string(), 0);
        assert_eq!(layer_id, 0);
        assert_eq!(map.layers.len(), 1);
        assert_eq!(map.layers[0].name, "地形层");
        assert_eq!(map.layers[0].layer_type, LayerType::Terrain);
    }
    
    #[test]
    fn test_tile_operations() {
        let mut map = GameMap::new(1, "测试".to_string(), Vec2::new(1000.0, 1000.0));
        let layer_id = map.add_layer(LayerType::Terrain, "地形".to_string(), 0);
        
        let tile_data = TileData {
            tile_id: 1,
            variation: 0,
            rotation: 0,
            flip_x: false,
            flip_y: false,
            properties: 0,
        };
        
        map.set_tile(layer_id, 5, 10, tile_data).unwrap();
        
        let retrieved_tile = map.get_tile(layer_id, 5, 10);
        assert!(retrieved_tile.is_some());
        assert_eq!(retrieved_tile.unwrap().tile_id, 1);
    }
    
    #[test]
    fn test_collision_detection() {
        let mut map = GameMap::new(1, "测试".to_string(), Vec2::new(1000.0, 1000.0));
        
        let collision = CollisionTile {
            collision_type: CollisionType::Solid,
            solid: true,
            one_way: false,
            trigger: false,
            elevation: 0.0,
            friction: 1.0,
            bounce: 0.0,
        };
        
        map.set_collision(0, 0, collision);
        
        let result = map.check_collision(Vec2::new(16.0, 16.0), Vec2::new(32.0, 32.0));
        assert!(result.is_some());
        assert!(result.unwrap().solid);
    }
    
    #[test]
    fn test_chunk_id_encoding() {
        let map = GameMap::new(1, "测试".to_string(), Vec2::new(1000.0, 1000.0));
        
        let position = Vec2::new(1000.0, 2000.0);
        let chunk_id = map.get_chunk_id_for_position(position);
        let (chunk_x, chunk_y) = map.decode_chunk_id(chunk_id);
        
        assert_eq!(chunk_x, 1); // 1000 / 512 = ~1
        assert_eq!(chunk_y, 3); // 2000 / 512 = ~3
    }
}