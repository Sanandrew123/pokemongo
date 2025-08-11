/*
* 开发心理过程：
* 1. 设计灵活的瓦片地图系统，支持多层渲染和复杂地形
* 2. 实现高效的瓦片索引和批量渲染优化
* 3. 支持动态加载和卸载地图块，优化内存使用
* 4. 集成碰撞检测、可行走性、触发区域等游戏逻辑
* 5. 提供瓦片动画、特效、环境交互等高级功能
* 6. 支持地图编辑器的实时预览和修改
* 7. 实现LOD（细节层次）系统优化大地图性能
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::math::{Vec2, IVec2};

use crate::{
    core::error::{GameError, GameResult},
    graphics::texture::TextureId,
    world::collision::CollisionShape,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Component)]
pub struct TileMap {
    /// 地图尺寸（以瓦片为单位）
    pub size: IVec2,
    /// 瓦片大小（像素）
    pub tile_size: IVec2,
    /// 地图层
    pub layers: Vec<TileLayer>,
    /// 瓦片集
    pub tilesets: Vec<Tileset>,
    /// 地图属性
    pub properties: HashMap<String, String>,
    /// 地图ID
    pub id: String,
    /// 地图名称
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    /// 层名称
    pub name: String,
    /// 层ID
    pub id: u32,
    /// 层级（渲染顺序）
    pub z_order: i32,
    /// 是否可见
    pub visible: bool,
    /// 透明度
    pub opacity: f32,
    /// 瓦片数据
    pub tiles: Vec<Vec<Tile>>,
    /// 层属性
    pub properties: HashMap<String, String>,
    /// 层类型
    pub layer_type: LayerType,
    /// 滚动速度（视差效果）
    pub scroll_speed: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Background,      // 背景层
    Terrain,         // 地形层
    Decoration,      // 装饰层
    Collision,       // 碰撞层
    Interactive,     // 交互层
    Foreground,      // 前景层
    Overlay,         // 覆盖层
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    /// 瓦片ID（0表示空瓦片）
    pub id: u32,
    /// 瓦片集ID
    pub tileset_id: u16,
    /// 翻转标志
    pub flip: TileFlip,
    /// 旋转角度
    pub rotation: TileRotation,
    /// 瓦片属性
    pub properties: TileProperties,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TileFlip {
    pub horizontal: bool,
    pub vertical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileRotation {
    None,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TileProperties {
    /// 是否可碰撞
    pub collidable: bool,
    /// 是否可行走
    pub walkable: bool,
    /// 移动速度修正
    pub speed_modifier: f32,
    /// 是否触发遇敌
    pub encounter_trigger: bool,
    /// 地形类型
    pub terrain_type: TerrainType,
    /// 高度层级
    pub elevation: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Normal,          // 普通地面
    Grass,           // 草地
    Water,           // 水面
    Sand,            // 沙地
    Rock,            // 岩石
    Ice,             // 冰面
    Lava,            // 熔岩
    Swamp,           // 沼泽
    Bridge,          // 桥梁
    Cave,            // 洞穴
    Building,        // 建筑物
    Special,         // 特殊地形
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tileset {
    /// 瓦片集ID
    pub id: u16,
    /// 瓦片集名称
    pub name: String,
    /// 纹理资源ID
    pub texture_id: TextureId,
    /// 瓦片大小
    pub tile_size: IVec2,
    /// 纹理中的瓦片数量
    pub tile_count: IVec2,
    /// 瓦片间距
    pub spacing: IVec2,
    /// 边距
    pub margin: IVec2,
    /// 瓦片元数据
    pub tile_metadata: HashMap<u32, TileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMetadata {
    /// 瓦片名称
    pub name: String,
    /// 瓦片属性
    pub properties: TileProperties,
    /// 碰撞形状
    pub collision_shape: Option<CollisionShape>,
    /// 动画信息
    pub animation: Option<TileAnimation>,
    /// 触发事件
    pub trigger_events: Vec<TriggerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileAnimation {
    /// 动画帧
    pub frames: Vec<AnimationFrame>,
    /// 是否循环
    pub looping: bool,
    /// 播放速度
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// 瓦片ID
    pub tile_id: u32,
    /// 持续时间（毫秒）
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 事件数据
    pub event_data: String,
    /// 触发条件
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    OnEnter,         // 进入时触发
    OnExit,          // 离开时触发
    OnStep,          // 踩踏时触发
    OnInteract,      // 交互时触发
    OnTouch,         // 接触时触发
}

#[derive(Debug, Clone)]
pub struct TileMapRenderer {
    /// 视口位置
    pub viewport: Vec2,
    /// 视口大小
    pub viewport_size: Vec2,
    /// 缩放级别
    pub zoom: f32,
    /// 可见瓦片缓存
    pub visible_tiles_cache: HashMap<u32, Vec<VisibleTile>>,
    /// 渲染批次
    pub render_batches: Vec<RenderBatch>,
    /// LOD级别
    pub lod_level: u8,
}

#[derive(Debug, Clone)]
pub struct VisibleTile {
    /// 瓦片位置
    pub position: IVec2,
    /// 屏幕位置
    pub screen_position: Vec2,
    /// 瓦片数据
    pub tile: Tile,
    /// 层级
    pub layer_id: u32,
}

#[derive(Debug, Clone)]
pub struct RenderBatch {
    /// 纹理ID
    pub texture_id: TextureId,
    /// 瓦片列表
    pub tiles: Vec<BatchedTile>,
    /// 层级
    pub z_order: i32,
}

#[derive(Debug, Clone)]
pub struct BatchedTile {
    /// 源矩形（纹理坐标）
    pub source_rect: bevy::math::Rect,
    /// 目标矩形（屏幕坐标）
    pub dest_rect: bevy::math::Rect,
    /// 颜色
    pub color: Color,
    /// 变换
    pub transform: Transform,
}

#[derive(Debug, Clone)]
pub struct TileMapManager {
    /// 已加载的地图
    pub loaded_maps: HashMap<String, TileMap>,
    /// 地图缓存配置
    pub cache_config: CacheConfig,
    /// 性能统计
    pub performance_stats: PerformanceStats,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存地图数量
    pub max_cached_maps: usize,
    /// 瓦片缓存大小
    pub tile_cache_size: usize,
    /// 自动清理阈值
    pub cleanup_threshold: f32,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// 渲染的瓦片数量
    pub rendered_tiles: u32,
    /// 剔除的瓦片数量
    pub culled_tiles: u32,
    /// 批次数量
    pub batch_count: u32,
    /// 渲染时间（毫秒）
    pub render_time_ms: f32,
}

impl TileMap {
    pub fn new(id: String, name: String, size: IVec2, tile_size: IVec2) -> Self {
        Self {
            size,
            tile_size,
            layers: Vec::new(),
            tilesets: Vec::new(),
            properties: HashMap::new(),
            id,
            name,
        }
    }

    /// 添加图层
    pub fn add_layer(&mut self, layer: TileLayer) {
        self.layers.push(layer);
        // 按z_order排序
        self.layers.sort_by_key(|l| l.z_order);
    }

    /// 获取指定位置的瓦片
    pub fn get_tile(&self, layer_id: u32, position: IVec2) -> Option<&Tile> {
        let layer = self.layers.iter().find(|l| l.id == layer_id)?;
        
        if position.x < 0 || position.y < 0 || 
           position.x >= self.size.x || position.y >= self.size.y {
            return None;
        }

        layer.tiles.get(position.y as usize)?
            .get(position.x as usize)
    }

    /// 设置指定位置的瓦片
    pub fn set_tile(&mut self, layer_id: u32, position: IVec2, tile: Tile) -> GameResult<()> {
        if position.x < 0 || position.y < 0 || 
           position.x >= self.size.x || position.y >= self.size.y {
            return Err(GameError::World("瓦片位置超出地图范围".to_string()));
        }

        let layer = self.layers.iter_mut()
            .find(|l| l.id == layer_id)
            .ok_or_else(|| GameError::World("找不到指定图层".to_string()))?;

        if let Some(row) = layer.tiles.get_mut(position.y as usize) {
            if let Some(cell) = row.get_mut(position.x as usize) {
                *cell = tile;
                return Ok(());
            }
        }

        Err(GameError::World("无效的瓦片位置".to_string()))
    }

    /// 检查位置是否可行走
    pub fn is_walkable(&self, world_position: Vec2) -> bool {
        let tile_pos = self.world_to_tile_position(world_position);
        
        // 检查所有相关图层
        for layer in &self.layers {
            if let Some(tile) = self.get_tile(layer.id, tile_pos) {
                if !tile.properties.walkable {
                    return false;
                }
            }
        }
        
        true
    }

    /// 检查位置是否有碰撞
    pub fn has_collision(&self, world_position: Vec2) -> bool {
        let tile_pos = self.world_to_tile_position(world_position);
        
        for layer in &self.layers {
            if layer.layer_type == LayerType::Collision {
                if let Some(tile) = self.get_tile(layer.id, tile_pos) {
                    if tile.properties.collidable {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// 获取位置的地形类型
    pub fn get_terrain_type(&self, world_position: Vec2) -> TerrainType {
        let tile_pos = self.world_to_tile_position(world_position);
        
        for layer in &self.layers {
            if layer.layer_type == LayerType::Terrain {
                if let Some(tile) = self.get_tile(layer.id, tile_pos) {
                    return tile.properties.terrain_type;
                }
            }
        }
        
        TerrainType::Normal
    }

    /// 世界坐标转瓦片坐标
    pub fn world_to_tile_position(&self, world_position: Vec2) -> IVec2 {
        IVec2::new(
            (world_position.x / self.tile_size.x as f32).floor() as i32,
            (world_position.y / self.tile_size.y as f32).floor() as i32,
        )
    }

    /// 瓦片坐标转世界坐标
    pub fn tile_to_world_position(&self, tile_position: IVec2) -> Vec2 {
        Vec2::new(
            tile_position.x as f32 * self.tile_size.x as f32,
            tile_position.y as f32 * self.tile_size.y as f32,
        )
    }

    /// 获取周围的瓦片
    pub fn get_surrounding_tiles(&self, center: IVec2, radius: i32) -> Vec<(IVec2, &Tile)> {
        let mut tiles = Vec::new();
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let pos = center + IVec2::new(dx, dy);
                
                // 检查所有图层
                for layer in &self.layers {
                    if let Some(tile) = self.get_tile(layer.id, pos) {
                        if tile.id != 0 { // 跳过空瓦片
                            tiles.push((pos, tile));
                        }
                    }
                }
            }
        }
        
        tiles
    }

    /// 查找特定类型的瓦片
    pub fn find_tiles_by_type(&self, terrain_type: TerrainType) -> Vec<IVec2> {
        let mut positions = Vec::new();
        
        for layer in &self.layers {
            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    if tile.properties.terrain_type == terrain_type {
                        positions.push(IVec2::new(x as i32, y as i32));
                    }
                }
            }
        }
        
        positions
    }

    /// 生成随机地图
    pub fn generate_random_map(
        size: IVec2,
        tile_size: IVec2,
        tileset: &Tileset,
        rng: &mut RandomGenerator,
    ) -> GameResult<Self> {
        let mut map = Self::new("random_map".to_string(), "Random Map".to_string(), size, tile_size);
        
        // 添加基础地形层
        let mut terrain_layer = TileLayer::new("terrain".to_string(), 0, LayerType::Terrain);
        terrain_layer.tiles = vec![vec![Tile::default(); size.x as usize]; size.y as usize];
        
        // 生成地形
        for y in 0..size.y {
            for x in 0..size.x {
                let noise_value = rng.noise_2d(x as f32 * 0.1, y as f32 * 0.1);
                let tile_id = if noise_value > 0.5 { 2 } else { 1 }; // 草地或泥土
                
                terrain_layer.tiles[y as usize][x as usize] = Tile {
                    id: tile_id,
                    tileset_id: tileset.id,
                    flip: TileFlip::default(),
                    rotation: TileRotation::None,
                    properties: TileProperties {
                        walkable: true,
                        terrain_type: if tile_id == 2 { TerrainType::Grass } else { TerrainType::Normal },
                        encounter_trigger: tile_id == 2, // 草地触发遇敌
                        speed_modifier: 1.0,
                        ..Default::default()
                    },
                };
            }
        }
        
        map.add_layer(terrain_layer);
        map.tilesets.push(tileset.clone());
        
        Ok(map)
    }
}

impl TileLayer {
    pub fn new(name: String, id: u32, layer_type: LayerType) -> Self {
        Self {
            name,
            id,
            z_order: match layer_type {
                LayerType::Background => -100,
                LayerType::Terrain => 0,
                LayerType::Decoration => 10,
                LayerType::Collision => 20,
                LayerType::Interactive => 30,
                LayerType::Foreground => 50,
                LayerType::Overlay => 100,
            },
            visible: true,
            opacity: 1.0,
            tiles: Vec::new(),
            properties: HashMap::new(),
            layer_type,
            scroll_speed: Vec2::ONE,
        }
    }

    /// 初始化图层数据
    pub fn initialize(&mut self, size: IVec2) {
        self.tiles = vec![vec![Tile::default(); size.x as usize]; size.y as usize];
    }

    /// 填充图层
    pub fn fill(&mut self, tile: Tile) {
        for row in &mut self.tiles {
            for cell in row {
                *cell = tile;
            }
        }
    }

    /// 应用图案
    pub fn apply_pattern(&mut self, pattern: &TilePattern, offset: IVec2) -> GameResult<()> {
        for (y, row) in pattern.tiles.iter().enumerate() {
            for (x, &tile_id) in row.iter().enumerate() {
                let pos = offset + IVec2::new(x as i32, y as i32);
                
                if pos.x >= 0 && pos.y >= 0 && 
                   pos.x < self.tiles[0].len() as i32 && pos.y < self.tiles.len() as i32 {
                    if tile_id != 0 {
                        self.tiles[pos.y as usize][pos.x as usize] = Tile {
                            id: tile_id,
                            tileset_id: pattern.tileset_id,
                            ..Default::default()
                        };
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl TileMapRenderer {
    pub fn new() -> Self {
        Self {
            viewport: Vec2::ZERO,
            viewport_size: Vec2::new(800.0, 600.0),
            zoom: 1.0,
            visible_tiles_cache: HashMap::new(),
            render_batches: Vec::new(),
            lod_level: 0,
        }
    }

    /// 更新可见瓦片缓存
    pub fn update_visible_tiles(&mut self, tile_map: &TileMap) {
        self.visible_tiles_cache.clear();
        
        // 计算可见区域
        let tile_bounds = self.calculate_visible_tile_bounds(tile_map);
        
        for layer in &tile_map.layers {
            if !layer.visible || layer.opacity <= 0.0 {
                continue;
            }
            
            let mut visible_tiles = Vec::new();
            
            for y in tile_bounds.min.y..=tile_bounds.max.y {
                for x in tile_bounds.min.x..=tile_bounds.max.x {
                    if let Some(tile) = tile_map.get_tile(layer.id, IVec2::new(x, y)) {
                        if tile.id != 0 { // 跳过空瓦片
                            let world_pos = tile_map.tile_to_world_position(IVec2::new(x, y));
                            let screen_pos = self.world_to_screen_position(world_pos);
                            
                            visible_tiles.push(VisibleTile {
                                position: IVec2::new(x, y),
                                screen_position: screen_pos,
                                tile: *tile,
                                layer_id: layer.id,
                            });
                        }
                    }
                }
            }
            
            self.visible_tiles_cache.insert(layer.id, visible_tiles);
        }
    }

    /// 生成渲染批次
    pub fn generate_render_batches(&mut self, tile_map: &TileMap) {
        self.render_batches.clear();
        
        // 按纹理和层级分组
        let mut batches: HashMap<(TextureId, i32), Vec<BatchedTile>> = HashMap::new();
        
        for layer in &tile_map.layers {
            if let Some(visible_tiles) = self.visible_tiles_cache.get(&layer.id) {
                for visible_tile in visible_tiles {
                    if let Some(tileset) = tile_map.tilesets.iter()
                        .find(|ts| ts.id == visible_tile.tile.tileset_id) {
                        
                        let batched_tile = self.create_batched_tile(
                            &visible_tile.tile,
                            visible_tile.screen_position,
                            tileset,
                            tile_map.tile_size,
                            layer.opacity,
                        );
                        
                        batches.entry((tileset.texture_id, layer.z_order))
                            .or_default()
                            .push(batched_tile);
                    }
                }
            }
        }
        
        // 转换为渲染批次
        for ((texture_id, z_order), tiles) in batches {
            self.render_batches.push(RenderBatch {
                texture_id,
                tiles,
                z_order,
            });
        }
        
        // 按z_order排序
        self.render_batches.sort_by_key(|batch| batch.z_order);
    }

    fn calculate_visible_tile_bounds(&self, tile_map: &TileMap) -> TileBounds {
        let viewport_world_min = self.screen_to_world_position(Vec2::ZERO);
        let viewport_world_max = self.screen_to_world_position(self.viewport_size);
        
        let tile_min = tile_map.world_to_tile_position(viewport_world_min);
        let tile_max = tile_map.world_to_tile_position(viewport_world_max);
        
        TileBounds {
            min: IVec2::new(
                (tile_min.x - 1).max(0),
                (tile_min.y - 1).max(0),
            ),
            max: IVec2::new(
                (tile_max.x + 1).min(tile_map.size.x - 1),
                (tile_max.y + 1).min(tile_map.size.y - 1),
            ),
        }
    }

    fn world_to_screen_position(&self, world_position: Vec2) -> Vec2 {
        (world_position - self.viewport) * self.zoom
    }

    fn screen_to_world_position(&self, screen_position: Vec2) -> Vec2 {
        screen_position / self.zoom + self.viewport
    }

    fn create_batched_tile(
        &self,
        tile: &Tile,
        screen_position: Vec2,
        tileset: &Tileset,
        tile_size: IVec2,
        opacity: f32,
    ) -> BatchedTile {
        // 计算源矩形（纹理坐标）
        let tiles_per_row = tileset.tile_count.x;
        let tile_x = (tile.id as i32 - 1) % tiles_per_row;
        let tile_y = (tile.id as i32 - 1) / tiles_per_row;
        
        let source_rect = bevy::math::Rect::new(
            (tileset.margin.x + tile_x * (tileset.tile_size.x + tileset.spacing.x)) as f32,
            (tileset.margin.y + tile_y * (tileset.tile_size.y + tileset.spacing.y)) as f32,
            tileset.tile_size.x as f32,
            tileset.tile_size.y as f32,
        );
        
        // 计算目标矩形（屏幕坐标）
        let dest_rect = bevy::math::Rect::new(
            screen_position.x,
            screen_position.y,
            (tile_size.x as f32 * self.zoom),
            (tile_size.y as f32 * self.zoom),
        );
        
        // 处理翻转和旋转
        let mut transform = Transform::from_translation(Vec3::new(
            screen_position.x + dest_rect.width() * 0.5,
            screen_position.y + dest_rect.height() * 0.5,
            0.0,
        ));
        
        match tile.rotation {
            TileRotation::Rotate90 => transform.rotate_z(std::f32::consts::FRAC_PI_2),
            TileRotation::Rotate180 => transform.rotate_z(std::f32::consts::PI),
            TileRotation::Rotate270 => transform.rotate_z(3.0 * std::f32::consts::FRAC_PI_2),
            _ => {},
        }
        
        if tile.flip.horizontal {
            transform.scale.x = -1.0;
        }
        if tile.flip.vertical {
            transform.scale.y = -1.0;
        }
        
        BatchedTile {
            source_rect,
            dest_rect,
            color: Color::rgba(1.0, 1.0, 1.0, opacity),
            transform,
        }
    }

    /// 设置视口
    pub fn set_viewport(&mut self, position: Vec2, size: Vec2) {
        self.viewport = position;
        self.viewport_size = size;
    }

    /// 设置缩放
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }
}

impl TileMapManager {
    pub fn new() -> Self {
        Self {
            loaded_maps: HashMap::new(),
            cache_config: CacheConfig::default(),
            performance_stats: PerformanceStats::default(),
        }
    }

    /// 加载地图
    pub fn load_map(&mut self, map: TileMap) {
        if self.loaded_maps.len() >= self.cache_config.max_cached_maps {
            // 清理最少使用的地图
            self.cleanup_unused_maps();
        }
        
        self.loaded_maps.insert(map.id.clone(), map);
    }

    /// 获取地图
    pub fn get_map(&self, map_id: &str) -> Option<&TileMap> {
        self.loaded_maps.get(map_id)
    }

    /// 卸载地图
    pub fn unload_map(&mut self, map_id: &str) -> bool {
        self.loaded_maps.remove(map_id).is_some()
    }

    fn cleanup_unused_maps(&mut self) {
        // 简化实现：移除第一个地图
        if let Some(first_key) = self.loaded_maps.keys().next().cloned() {
            self.loaded_maps.remove(&first_key);
        }
    }
}

// 辅助结构
#[derive(Debug, Clone)]
struct TileBounds {
    min: IVec2,
    max: IVec2,
}

#[derive(Debug, Clone)]
pub struct TilePattern {
    pub tiles: Vec<Vec<u32>>,
    pub tileset_id: u16,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            id: 0,
            tileset_id: 0,
            flip: TileFlip::default(),
            rotation: TileRotation::None,
            properties: TileProperties::default(),
        }
    }
}

impl Default for TileProperties {
    fn default() -> Self {
        Self {
            collidable: false,
            walkable: true,
            speed_modifier: 1.0,
            encounter_trigger: false,
            terrain_type: TerrainType::Normal,
            elevation: 0,
        }
    }
}

impl Default for TileRotation {
    fn default() -> Self {
        TileRotation::None
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_maps: 5,
            tile_cache_size: 10000,
            cleanup_threshold: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_map_creation() {
        let map = TileMap::new(
            "test_map".to_string(),
            "Test Map".to_string(),
            IVec2::new(10, 10),
            IVec2::new(32, 32),
        );
        
        assert_eq!(map.size, IVec2::new(10, 10));
        assert_eq!(map.tile_size, IVec2::new(32, 32));
        assert_eq!(map.id, "test_map");
    }

    #[test]
    fn test_coordinate_conversion() {
        let map = TileMap::new(
            "test".to_string(),
            "Test".to_string(),
            IVec2::new(10, 10),
            IVec2::new(32, 32),
        );
        
        let world_pos = Vec2::new(64.0, 96.0);
        let tile_pos = map.world_to_tile_position(world_pos);
        assert_eq!(tile_pos, IVec2::new(2, 3));
        
        let converted_back = map.tile_to_world_position(tile_pos);
        assert_eq!(converted_back, Vec2::new(64.0, 96.0));
    }

    #[test]
    fn test_tile_layer() {
        let mut layer = TileLayer::new("terrain".to_string(), 0, LayerType::Terrain);
        layer.initialize(IVec2::new(5, 5));
        
        assert_eq!(layer.tiles.len(), 5);
        assert_eq!(layer.tiles[0].len(), 5);
        assert_eq!(layer.z_order, 0);
    }

    #[test]
    fn test_tile_properties() {
        let props = TileProperties {
            walkable: false,
            terrain_type: TerrainType::Water,
            encounter_trigger: true,
            ..Default::default()
        };
        
        assert!(!props.walkable);
        assert_eq!(props.terrain_type, TerrainType::Water);
        assert!(props.encounter_trigger);
    }
}