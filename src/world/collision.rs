/*
* 开发心理过程：
* 1. 设计高效的2D碰撞检测系统，支持各种几何形状
* 2. 实现空间分割算法优化大量对象的碰撞检测
* 3. 支持静态和动态碰撞体，分离检测和响应逻辑
* 4. 集成连续碰撞检测防止快速移动物体穿透
* 5. 提供碰撞层和过滤系统，精确控制交互对象
* 6. 优化性能，支持大世界和大量NPC同时碰撞检测
* 7. 实现触发器系统，支持进入/离开事件
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::math::{Vec2, IVec2};
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    world::tile::TileMap,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Component)]
pub struct CollisionWorld {
    /// 空间哈希网格
    pub spatial_grid: SpatialHashGrid,
    /// 静态碰撞体
    pub static_bodies: HashMap<Uuid, CollisionBody>,
    /// 动态碰撞体
    pub dynamic_bodies: HashMap<Uuid, CollisionBody>,
    /// 触发器
    pub triggers: HashMap<Uuid, TriggerZone>,
    /// 碰撞配置
    pub config: CollisionConfig,
    /// 碰撞事件缓冲区
    pub collision_events: Vec<CollisionEvent>,
    /// 触发事件缓冲区
    pub trigger_events: Vec<TriggerEvent>,
}

#[derive(Debug, Clone)]
pub struct CollisionConfig {
    /// 网格大小
    pub grid_cell_size: f32,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 碰撞容忍度
    pub collision_tolerance: f32,
    /// 是否启用连续碰撞检测
    pub enable_continuous_detection: bool,
    /// 性能模式
    pub performance_mode: PerformanceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    Precise,    // 精确模式：高精度，适合小场景
    Balanced,   // 平衡模式：中等精度和性能
    Fast,       // 快速模式：低精度，适合大场景
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionBody {
    pub id: Uuid,
    pub position: Vec2,
    pub velocity: Vec2,
    pub shape: CollisionShape,
    pub body_type: BodyType,
    pub collision_layers: CollisionLayers,
    pub material: PhysicsMaterial,
    pub is_enabled: bool,
    pub is_trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyType {
    Static,     // 静态：不移动，不受力影响
    Kinematic,  // 运动学：可移动，不受物理影响
    Dynamic,    // 动态：受物理引擎完全控制
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionShape {
    Circle {
        radius: f32,
    },
    Rectangle {
        width: f32,
        height: f32,
    },
    Capsule {
        radius: f32,
        height: f32,
    },
    Polygon {
        vertices: Vec<Vec2>,
    },
    Compound {
        shapes: Vec<(CollisionShape, Vec2)>, // (形状, 相对位置)
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CollisionLayers {
    /// 碰撞层掩码
    pub layers: u32,
    /// 碰撞检测掩码
    pub mask: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    /// 摩擦系数
    pub friction: f32,
    /// 弹性系数
    pub restitution: f32,
    /// 密度
    pub density: f32,
}

#[derive(Debug, Clone)]
pub struct TriggerZone {
    pub id: Uuid,
    pub position: Vec2,
    pub shape: CollisionShape,
    pub trigger_layers: CollisionLayers,
    pub events: Vec<TriggerEventType>,
    pub is_enabled: bool,
    pub entered_entities: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub enum TriggerEventType {
    OnEnter(String),    // 进入时触发
    OnExit(String),     // 离开时触发
    OnStay(String),     // 停留时触发
}

#[derive(Debug, Clone)]
pub struct SpatialHashGrid {
    /// 网格大小
    pub cell_size: f32,
    /// 网格数据
    pub grid: HashMap<IVec2, Vec<Uuid>>,
    /// 对象位置缓存
    pub object_cells: HashMap<Uuid, Vec<IVec2>>,
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Uuid,
    pub entity_b: Uuid,
    pub contact_point: Vec2,
    pub contact_normal: Vec2,
    pub penetration_depth: f32,
    pub event_type: CollisionEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventType {
    Started,    // 碰撞开始
    Continuing, // 碰撞持续
    Ended,      // 碰撞结束
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub trigger_id: Uuid,
    pub entity_id: Uuid,
    pub event_type: TriggerEventType,
    pub position: Vec2,
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub entity_id: Uuid,
    pub position: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct MovementQuery {
    pub from: Vec2,
    pub to: Vec2,
    pub shape: CollisionShape,
    pub layers: CollisionLayers,
}

#[derive(Debug, Clone)]
pub struct MovementResult {
    pub final_position: Vec2,
    pub collisions: Vec<CollisionInfo>,
    pub was_blocked: bool,
}

#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub entity_id: Uuid,
    pub contact_point: Vec2,
    pub contact_normal: Vec2,
    pub penetration: f32,
}

impl CollisionWorld {
    pub fn new(config: CollisionConfig) -> Self {
        Self {
            spatial_grid: SpatialHashGrid::new(config.grid_cell_size),
            static_bodies: HashMap::new(),
            dynamic_bodies: HashMap::new(),
            triggers: HashMap::new(),
            config,
            collision_events: Vec::new(),
            trigger_events: Vec::new(),
        }
    }

    /// 添加碰撞体
    pub fn add_collision_body(&mut self, body: CollisionBody) -> GameResult<()> {
        let id = body.id;
        
        // 添加到空间网格
        self.spatial_grid.insert(id, body.position, &body.shape)?;
        
        // 根据类型添加到相应集合
        match body.body_type {
            BodyType::Static => {
                self.static_bodies.insert(id, body);
            },
            BodyType::Kinematic | BodyType::Dynamic => {
                self.dynamic_bodies.insert(id, body);
            },
        }
        
        Ok(())
    }

    /// 移除碰撞体
    pub fn remove_collision_body(&mut self, id: Uuid) -> GameResult<()> {
        // 从空间网格移除
        self.spatial_grid.remove(id);
        
        // 从集合中移除
        self.static_bodies.remove(&id);
        self.dynamic_bodies.remove(&id);
        
        Ok(())
    }

    /// 更新碰撞体位置
    pub fn update_body_position(&mut self, id: Uuid, new_position: Vec2) -> GameResult<()> {
        // 更新动态体
        if let Some(body) = self.dynamic_bodies.get_mut(&id) {
            let old_position = body.position;
            body.position = new_position;
            
            // 更新空间网格
            self.spatial_grid.update(id, old_position, new_position, &body.shape)?;
            return Ok(());
        }
        
        // 更新静态体（较少见）
        if let Some(body) = self.static_bodies.get_mut(&id) {
            let old_position = body.position;
            body.position = new_position;
            
            self.spatial_grid.update(id, old_position, new_position, &body.shape)?;
            return Ok(());
        }
        
        Err(GameError::World(format!("找不到碰撞体: {}", id)))
    }

    /// 移动碰撞体并检测碰撞
    pub fn move_body(&mut self, id: Uuid, delta: Vec2) -> GameResult<MovementResult> {
        let body = self.dynamic_bodies.get(&id)
            .ok_or_else(|| GameError::World("找不到动态碰撞体".to_string()))?
            .clone();
        
        let movement_query = MovementQuery {
            from: body.position,
            to: body.position + delta,
            shape: body.shape.clone(),
            layers: body.collision_layers,
        };
        
        self.test_movement(movement_query, Some(id))
    }

    /// 测试移动路径
    pub fn test_movement(&self, query: MovementQuery, exclude_id: Option<Uuid>) -> GameResult<MovementResult> {
        let mut result = MovementResult {
            final_position: query.to,
            collisions: Vec::new(),
            was_blocked: false,
        };
        
        // 获取路径上可能碰撞的对象
        let potential_colliders = self.get_potential_colliders_along_path(&query, exclude_id)?;
        
        if self.config.enable_continuous_detection {
            // 连续碰撞检测
            result = self.continuous_collision_detection(query, &potential_colliders)?;
        } else {
            // 离散碰撞检测
            result = self.discrete_collision_detection(query, &potential_colliders)?;
        }
        
        Ok(result)
    }

    /// 射线检测
    pub fn raycast(&self, start: Vec2, end: Vec2, layers: CollisionLayers) -> Option<RaycastHit> {
        let direction = (end - start).normalize();
        let max_distance = start.distance(end);
        
        // 获取射线路径上的潜在对象
        let cells = self.spatial_grid.get_cells_along_line(start, end);
        let mut potential_hits = Vec::new();
        
        for cell in cells {
            if let Some(objects) = self.spatial_grid.grid.get(&cell) {
                potential_hits.extend(objects);
            }
        }
        
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = max_distance;
        
        // 检测每个潜在对象
        for &object_id in &potential_hits {
            if let Some(body) = self.get_body(object_id) {
                if !self.should_collide(&layers, &body.collision_layers) {
                    continue;
                }
                
                if let Some(hit_distance) = self.ray_intersect_shape(start, direction, &body.shape, body.position) {
                    if hit_distance < closest_distance {
                        closest_distance = hit_distance;
                        let hit_position = start + direction * hit_distance;
                        let normal = self.calculate_surface_normal(&body.shape, body.position, hit_position);
                        
                        closest_hit = Some(RaycastHit {
                            entity_id: object_id,
                            position: hit_position,
                            normal,
                            distance: hit_distance,
                        });
                    }
                }
            }
        }
        
        closest_hit
    }

    /// 区域查询
    pub fn query_area(&self, shape: &CollisionShape, position: Vec2, layers: CollisionLayers) -> Vec<Uuid> {
        let mut results = Vec::new();
        let bounds = self.calculate_shape_bounds(shape, position);
        let cells = self.spatial_grid.get_cells_in_bounds(bounds);
        
        for cell in cells {
            if let Some(objects) = self.spatial_grid.grid.get(&cell) {
                for &object_id in objects {
                    if let Some(body) = self.get_body(object_id) {
                        if self.should_collide(&layers, &body.collision_layers) &&
                           self.shapes_intersect(shape, position, &body.shape, body.position) {
                            results.push(object_id);
                        }
                    }
                }
            }
        }
        
        results
    }

    /// 添加触发区域
    pub fn add_trigger(&mut self, trigger: TriggerZone) -> GameResult<()> {
        self.spatial_grid.insert(trigger.id, trigger.position, &trigger.shape)?;
        self.triggers.insert(trigger.id, trigger);
        Ok(())
    }

    /// 更新物理系统
    pub fn update(&mut self, delta_time: f32) -> GameResult<()> {
        self.collision_events.clear();
        self.trigger_events.clear();
        
        // 更新动态体位置
        self.update_dynamic_bodies(delta_time)?;
        
        // 检测碰撞
        self.detect_collisions()?;
        
        // 检测触发器事件
        self.detect_trigger_events()?;
        
        Ok(())
    }

    fn update_dynamic_bodies(&mut self, delta_time: f32) -> GameResult<()> {
        let body_ids: Vec<Uuid> = self.dynamic_bodies.keys().copied().collect();
        
        for id in body_ids {
            if let Some(body) = self.dynamic_bodies.get(&id).cloned() {
                if body.body_type == BodyType::Dynamic && body.velocity.length() > 0.0 {
                    let new_position = body.position + body.velocity * delta_time;
                    self.update_body_position(id, new_position)?;
                }
            }
        }
        
        Ok(())
    }

    fn detect_collisions(&mut self) -> GameResult<()> {
        let dynamic_ids: Vec<Uuid> = self.dynamic_bodies.keys().copied().collect();
        
        for id in dynamic_ids {
            let potential_colliders = self.spatial_grid.get_nearby_objects(id);
            
            for &other_id in &potential_colliders {
                if id != other_id {
                    if let Some(collision) = self.test_collision(id, other_id)? {
                        self.collision_events.push(collision);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn detect_trigger_events(&mut self) -> GameResult<()> {
        for (trigger_id, trigger) in &mut self.triggers {
            if !trigger.is_enabled {
                continue;
            }
            
            let current_entities = self.query_area(&trigger.shape, trigger.position, trigger.trigger_layers);
            let current_set: HashSet<Uuid> = current_entities.into_iter().collect();
            
            // 检测进入事件
            for &entity_id in &current_set {
                if !trigger.entered_entities.contains(&entity_id) {
                    for event in &trigger.events {
                        if matches!(event, TriggerEventType::OnEnter(_)) {
                            self.trigger_events.push(TriggerEvent {
                                trigger_id: *trigger_id,
                                entity_id,
                                event_type: event.clone(),
                                position: trigger.position,
                            });
                        }
                    }
                }
            }
            
            // 检测离开事件
            for &entity_id in &trigger.entered_entities {
                if !current_set.contains(&entity_id) {
                    for event in &trigger.events {
                        if matches!(event, TriggerEventType::OnExit(_)) {
                            self.trigger_events.push(TriggerEvent {
                                trigger_id: *trigger_id,
                                entity_id,
                                event_type: event.clone(),
                                position: trigger.position,
                            });
                        }
                    }
                }
            }
            
            // 更新进入列表
            trigger.entered_entities = current_set;
        }
        
        Ok(())
    }

    // 碰撞检测算法实现
    fn shapes_intersect(&self, shape1: &CollisionShape, pos1: Vec2, shape2: &CollisionShape, pos2: Vec2) -> bool {
        match (shape1, shape2) {
            (CollisionShape::Circle { radius: r1 }, CollisionShape::Circle { radius: r2 }) => {
                let distance = pos1.distance(pos2);
                distance <= r1 + r2
            },
            (CollisionShape::Rectangle { width: w1, height: h1 }, CollisionShape::Rectangle { width: w2, height: h2 }) => {
                self.aabb_intersect(pos1, *w1, *h1, pos2, *w2, *h2)
            },
            (CollisionShape::Circle { radius }, CollisionShape::Rectangle { width, height }) => {
                self.circle_aabb_intersect(pos1, *radius, pos2, *width, *height)
            },
            (CollisionShape::Rectangle { width, height }, CollisionShape::Circle { radius }) => {
                self.circle_aabb_intersect(pos2, *radius, pos1, *width, *height)
            },
            _ => {
                // 更复杂的形状组合，使用SAT或GJK算法
                false // 简化实现
            },
        }
    }

    fn aabb_intersect(&self, pos1: Vec2, w1: f32, h1: f32, pos2: Vec2, w2: f32, h2: f32) -> bool {
        let left1 = pos1.x - w1 * 0.5;
        let right1 = pos1.x + w1 * 0.5;
        let top1 = pos1.y - h1 * 0.5;
        let bottom1 = pos1.y + h1 * 0.5;
        
        let left2 = pos2.x - w2 * 0.5;
        let right2 = pos2.x + w2 * 0.5;
        let top2 = pos2.y - h2 * 0.5;
        let bottom2 = pos2.y + h2 * 0.5;
        
        !(left1 > right2 || right1 < left2 || top1 > bottom2 || bottom1 < top2)
    }

    fn circle_aabb_intersect(&self, circle_pos: Vec2, radius: f32, aabb_pos: Vec2, width: f32, height: f32) -> bool {
        let left = aabb_pos.x - width * 0.5;
        let right = aabb_pos.x + width * 0.5;
        let top = aabb_pos.y - height * 0.5;
        let bottom = aabb_pos.y + height * 0.5;
        
        let closest_x = circle_pos.x.clamp(left, right);
        let closest_y = circle_pos.y.clamp(top, bottom);
        
        let distance = circle_pos.distance(Vec2::new(closest_x, closest_y));
        distance <= radius
    }

    // 辅助方法
    fn get_body(&self, id: Uuid) -> Option<&CollisionBody> {
        self.static_bodies.get(&id).or_else(|| self.dynamic_bodies.get(&id))
    }

    fn should_collide(&self, layers1: &CollisionLayers, layers2: &CollisionLayers) -> bool {
        (layers1.layers & layers2.mask) != 0 && (layers2.layers & layers1.mask) != 0
    }

    fn calculate_shape_bounds(&self, shape: &CollisionShape, position: Vec2) -> BoundingBox {
        match shape {
            CollisionShape::Circle { radius } => {
                BoundingBox {
                    min: position - Vec2::splat(*radius),
                    max: position + Vec2::splat(*radius),
                }
            },
            CollisionShape::Rectangle { width, height } => {
                let half_size = Vec2::new(*width * 0.5, *height * 0.5);
                BoundingBox {
                    min: position - half_size,
                    max: position + half_size,
                }
            },
            _ => {
                // 简化实现，返回大致边界
                let size = 50.0;
                BoundingBox {
                    min: position - Vec2::splat(size),
                    max: position + Vec2::splat(size),
                }
            },
        }
    }

    // 其他复杂方法的简化实现
    fn get_potential_colliders_along_path(&self, query: &MovementQuery, exclude_id: Option<Uuid>) -> GameResult<Vec<Uuid>> {
        let cells = self.spatial_grid.get_cells_along_line(query.from, query.to);
        let mut potential_colliders = Vec::new();
        
        for cell in cells {
            if let Some(objects) = self.spatial_grid.grid.get(&cell) {
                for &id in objects {
                    if Some(id) != exclude_id {
                        potential_colliders.push(id);
                    }
                }
            }
        }
        
        Ok(potential_colliders)
    }

    fn continuous_collision_detection(&self, query: MovementQuery, potential_colliders: &[Uuid]) -> GameResult<MovementResult> {
        // 简化实现：使用多个离散检测点
        let steps = 10;
        let delta = (query.to - query.from) / steps as f32;
        
        for i in 1..=steps {
            let test_pos = query.from + delta * i as f32;
            
            for &collider_id in potential_colliders {
                if let Some(body) = self.get_body(collider_id) {
                    if self.should_collide(&query.layers, &body.collision_layers) &&
                       self.shapes_intersect(&query.shape, test_pos, &body.shape, body.position) {
                        return Ok(MovementResult {
                            final_position: query.from + delta * (i - 1) as f32,
                            collisions: vec![CollisionInfo {
                                entity_id: collider_id,
                                contact_point: test_pos,
                                contact_normal: Vec2::ZERO, // 简化
                                penetration: 0.0,
                            }],
                            was_blocked: true,
                        });
                    }
                }
            }
        }
        
        Ok(MovementResult {
            final_position: query.to,
            collisions: Vec::new(),
            was_blocked: false,
        })
    }

    fn discrete_collision_detection(&self, query: MovementQuery, potential_colliders: &[Uuid]) -> GameResult<MovementResult> {
        let mut collisions = Vec::new();
        
        for &collider_id in potential_colliders {
            if let Some(body) = self.get_body(collider_id) {
                if self.should_collide(&query.layers, &body.collision_layers) &&
                   self.shapes_intersect(&query.shape, query.to, &body.shape, body.position) {
                    collisions.push(CollisionInfo {
                        entity_id: collider_id,
                        contact_point: query.to,
                        contact_normal: Vec2::ZERO, // 简化
                        penetration: 0.0,
                    });
                }
            }
        }
        
        Ok(MovementResult {
            final_position: if collisions.is_empty() { query.to } else { query.from },
            collisions,
            was_blocked: !collisions.is_empty(),
        })
    }

    fn test_collision(&self, id1: Uuid, id2: Uuid) -> GameResult<Option<CollisionEvent>> {
        let body1 = self.get_body(id1).ok_or_else(|| GameError::World("找不到碰撞体1".to_string()))?;
        let body2 = self.get_body(id2).ok_or_else(|| GameError::World("找不到碰撞体2".to_string()))?;
        
        if !self.should_collide(&body1.collision_layers, &body2.collision_layers) {
            return Ok(None);
        }
        
        if self.shapes_intersect(&body1.shape, body1.position, &body2.shape, body2.position) {
            Ok(Some(CollisionEvent {
                entity_a: id1,
                entity_b: id2,
                contact_point: (body1.position + body2.position) * 0.5, // 简化
                contact_normal: (body2.position - body1.position).normalize_or_zero(),
                penetration_depth: 0.0, // 简化
                event_type: CollisionEventType::Started,
            }))
        } else {
            Ok(None)
        }
    }

    fn ray_intersect_shape(&self, start: Vec2, direction: Vec2, shape: &CollisionShape, position: Vec2) -> Option<f32> {
        match shape {
            CollisionShape::Circle { radius } => {
                self.ray_circle_intersect(start, direction, position, *radius)
            },
            CollisionShape::Rectangle { width, height } => {
                self.ray_aabb_intersect(start, direction, position, *width, *height)
            },
            _ => None, // 简化实现
        }
    }

    fn ray_circle_intersect(&self, start: Vec2, direction: Vec2, center: Vec2, radius: f32) -> Option<f32> {
        let to_center = center - start;
        let projection = to_center.dot(direction);
        
        if projection < 0.0 {
            return None;
        }
        
        let closest_point = start + direction * projection;
        let distance_to_center = closest_point.distance(center);
        
        if distance_to_center > radius {
            return None;
        }
        
        let half_chord = (radius * radius - distance_to_center * distance_to_center).sqrt();
        Some(projection - half_chord)
    }

    fn ray_aabb_intersect(&self, start: Vec2, direction: Vec2, center: Vec2, width: f32, height: f32) -> Option<f32> {
        let half_size = Vec2::new(width * 0.5, height * 0.5);
        let min = center - half_size;
        let max = center + half_size;
        
        let inv_dir = Vec2::new(1.0 / direction.x, 1.0 / direction.y);
        
        let t1 = (min.x - start.x) * inv_dir.x;
        let t2 = (max.x - start.x) * inv_dir.x;
        let t3 = (min.y - start.y) * inv_dir.y;
        let t4 = (max.y - start.y) * inv_dir.y;
        
        let tmin = t1.min(t2).max(t3.min(t4));
        let tmax = t1.max(t2).min(t3.max(t4));
        
        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }

    fn calculate_surface_normal(&self, shape: &CollisionShape, position: Vec2, hit_point: Vec2) -> Vec2 {
        match shape {
            CollisionShape::Circle { .. } => {
                (hit_point - position).normalize_or_zero()
            },
            _ => Vec2::Y, // 简化实现
        }
    }

    /// 获取碰撞事件
    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }

    /// 获取触发事件
    pub fn get_trigger_events(&self) -> &[TriggerEvent] {
        &self.trigger_events
    }
}

impl SpatialHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            object_cells: HashMap::new(),
        }
    }

    pub fn insert(&mut self, object_id: Uuid, position: Vec2, shape: &CollisionShape) -> GameResult<()> {
        let cells = self.get_cells_for_shape(position, shape);
        
        for cell in &cells {
            self.grid.entry(*cell).or_default().push(object_id);
        }
        
        self.object_cells.insert(object_id, cells);
        Ok(())
    }

    pub fn remove(&mut self, object_id: Uuid) {
        if let Some(cells) = self.object_cells.remove(&object_id) {
            for cell in cells {
                if let Some(objects) = self.grid.get_mut(&cell) {
                    objects.retain(|&id| id != object_id);
                    if objects.is_empty() {
                        self.grid.remove(&cell);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, object_id: Uuid, old_position: Vec2, new_position: Vec2, shape: &CollisionShape) -> GameResult<()> {
        self.remove(object_id);
        self.insert(object_id, new_position, shape)
    }

    pub fn get_nearby_objects(&self, object_id: Uuid) -> Vec<Uuid> {
        let mut nearby = Vec::new();
        
        if let Some(cells) = self.object_cells.get(&object_id) {
            for cell in cells {
                if let Some(objects) = self.grid.get(cell) {
                    nearby.extend(objects);
                }
            }
        }
        
        nearby.sort();
        nearby.dedup();
        nearby.retain(|&id| id != object_id);
        nearby
    }

    fn get_cells_for_shape(&self, position: Vec2, shape: &CollisionShape) -> Vec<IVec2> {
        let bounds = self.calculate_shape_bounds(shape, position);
        self.get_cells_in_bounds(bounds)
    }

    fn calculate_shape_bounds(&self, shape: &CollisionShape, position: Vec2) -> BoundingBox {
        match shape {
            CollisionShape::Circle { radius } => {
                BoundingBox {
                    min: position - Vec2::splat(*radius),
                    max: position + Vec2::splat(*radius),
                }
            },
            CollisionShape::Rectangle { width, height } => {
                let half_size = Vec2::new(*width * 0.5, *height * 0.5);
                BoundingBox {
                    min: position - half_size,
                    max: position + half_size,
                }
            },
            _ => {
                BoundingBox {
                    min: position - Vec2::splat(50.0),
                    max: position + Vec2::splat(50.0),
                }
            },
        }
    }

    fn get_cells_in_bounds(&self, bounds: BoundingBox) -> Vec<IVec2> {
        let min_cell = IVec2::new(
            (bounds.min.x / self.cell_size).floor() as i32,
            (bounds.min.y / self.cell_size).floor() as i32,
        );
        let max_cell = IVec2::new(
            (bounds.max.x / self.cell_size).floor() as i32,
            (bounds.max.y / self.cell_size).floor() as i32,
        );
        
        let mut cells = Vec::new();
        for y in min_cell.y..=max_cell.y {
            for x in min_cell.x..=max_cell.x {
                cells.push(IVec2::new(x, y));
            }
        }
        cells
    }

    pub fn get_cells_along_line(&self, start: Vec2, end: Vec2) -> Vec<IVec2> {
        let mut cells = Vec::new();
        let direction = end - start;
        let steps = (direction.length() / self.cell_size * 2.0) as i32;
        
        if steps == 0 {
            return vec![IVec2::new(
                (start.x / self.cell_size).floor() as i32,
                (start.y / self.cell_size).floor() as i32,
            )];
        }
        
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let point = start + direction * t;
            let cell = IVec2::new(
                (point.x / self.cell_size).floor() as i32,
                (point.y / self.cell_size).floor() as i32,
            );
            
            if !cells.contains(&cell) {
                cells.push(cell);
            }
        }
        
        cells
    }
}

// 辅助结构
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            friction: 0.6,
            restitution: 0.0,
            density: 1.0,
        }
    }
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            grid_cell_size: 64.0,
            max_iterations: 10,
            collision_tolerance: 0.01,
            enable_continuous_detection: false,
            performance_mode: PerformanceMode::Balanced,
        }
    }
}

impl CollisionLayers {
    pub const LAYER_1: u32 = 1 << 0;
    pub const LAYER_2: u32 = 1 << 1;
    pub const LAYER_3: u32 = 1 << 2;
    pub const LAYER_4: u32 = 1 << 3;
    pub const ALL: u32 = 0xFFFFFFFF;
    
    pub fn new(layers: u32, mask: u32) -> Self {
        Self { layers, mask }
    }
    
    pub fn all() -> Self {
        Self { layers: Self::ALL, mask: Self::ALL }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_world_creation() {
        let world = CollisionWorld::new(CollisionConfig::default());
        assert_eq!(world.static_bodies.len(), 0);
        assert_eq!(world.dynamic_bodies.len(), 0);
    }

    #[test]
    fn test_circle_collision() {
        let world = CollisionWorld::new(CollisionConfig::default());
        
        let shape1 = CollisionShape::Circle { radius: 10.0 };
        let shape2 = CollisionShape::Circle { radius: 10.0 };
        
        // 相交的圆
        assert!(world.shapes_intersect(&shape1, Vec2::ZERO, &shape2, Vec2::new(15.0, 0.0)));
        
        // 不相交的圆
        assert!(!world.shapes_intersect(&shape1, Vec2::ZERO, &shape2, Vec2::new(25.0, 0.0)));
    }

    #[test]
    fn test_spatial_hash_grid() {
        let mut grid = SpatialHashGrid::new(32.0);
        let id = Uuid::new_v4();
        let shape = CollisionShape::Circle { radius: 5.0 };
        
        grid.insert(id, Vec2::new(16.0, 16.0), &shape).unwrap();
        assert!(grid.object_cells.contains_key(&id));
        
        grid.remove(id);
        assert!(!grid.object_cells.contains_key(&id));
    }

    #[test]
    fn test_collision_layers() {
        let layers1 = CollisionLayers::new(CollisionLayers::LAYER_1, CollisionLayers::LAYER_2);
        let layers2 = CollisionLayers::new(CollisionLayers::LAYER_2, CollisionLayers::LAYER_1);
        
        let world = CollisionWorld::new(CollisionConfig::default());
        assert!(world.should_collide(&layers1, &layers2));
        
        let layers3 = CollisionLayers::new(CollisionLayers::LAYER_3, CollisionLayers::LAYER_4);
        assert!(!world.should_collide(&layers1, &layers3));
    }

    #[test]
    fn test_raycast() {
        let mut world = CollisionWorld::new(CollisionConfig::default());
        
        let body = CollisionBody {
            id: Uuid::new_v4(),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            shape: CollisionShape::Circle { radius: 10.0 },
            body_type: BodyType::Static,
            collision_layers: CollisionLayers::all(),
            material: PhysicsMaterial::default(),
            is_enabled: true,
            is_trigger: false,
        };
        
        world.add_collision_body(body).unwrap();
        
        let hit = world.raycast(Vec2::ZERO, Vec2::new(100.0, 50.0), CollisionLayers::all());
        assert!(hit.is_some());
        
        if let Some(hit) = hit {
            assert!(hit.distance > 0.0);
            assert!(hit.distance < 100.0);
        }
    }
}