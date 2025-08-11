// Pathfinding System - AI 移动路径计算和寻路算法
//
// 开发心理过程：
// 1. 这是游戏中AI角色移动的核心算法模块，需要支持多种寻路算法
// 2. A*算法适合精确寻路，JPS适合大地图快速搜索，流场适合多单位移动
// 3. 考虑到Pokemon游戏特性，需要支持不同地形类型和移动成本
// 4. 实现分层寻路系统，支持世界地图到局部地图的多级导航
// 5. 为NPC巡逻、玩家导航、AI战斗定位等场景提供不同的寻路策略

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Ordering;
use std::f32::consts::SQRT_2;
use serde::{Deserialize, Serialize};
use nalgebra::{Vector2, Vector3};

#[cfg(feature = "pathfinding-wip")]
use rayon::prelude::*;

/// 坐标点结构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn manhattan_distance(&self, other: &Point) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    pub fn neighbors(&self) -> Vec<Point> {
        vec![
            Point::new(self.x - 1, self.y),
            Point::new(self.x + 1, self.y),
            Point::new(self.x, self.y - 1),
            Point::new(self.x, self.y + 1),
        ]
    }

    pub fn neighbors_8(&self) -> Vec<Point> {
        vec![
            Point::new(self.x - 1, self.y - 1), Point::new(self.x, self.y - 1), Point::new(self.x + 1, self.y - 1),
            Point::new(self.x - 1, self.y),     Point::new(self.x + 1, self.y),
            Point::new(self.x - 1, self.y + 1), Point::new(self.x, self.y + 1), Point::new(self.x + 1, self.y + 1),
        ]
    }
}

/// 地形类型定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Grass,
    Water,
    Sand,
    Rock,
    Ice,
    Lava,
    Building,
    Blocked,
    Bridge,
    Cave,
}

impl TerrainType {
    pub fn movement_cost(&self) -> f32 {
        match self {
            TerrainType::Grass => 1.0,
            TerrainType::Water => 2.0,
            TerrainType::Sand => 1.5,
            TerrainType::Rock => 1.2,
            TerrainType::Ice => 0.8,
            TerrainType::Lava => 10.0,
            TerrainType::Building => f32::INFINITY,
            TerrainType::Blocked => f32::INFINITY,
            TerrainType::Bridge => 1.0,
            TerrainType::Cave => 1.3,
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self, TerrainType::Building | TerrainType::Blocked)
    }
}

/// 寻路节点
#[derive(Debug, Clone)]
struct PathNode {
    position: Point,
    g_cost: f32,
    h_cost: f32,
    parent: Option<Point>,
}

impl PathNode {
    fn new(position: Point, g_cost: f32, h_cost: f32, parent: Option<Point>) -> Self {
        Self { position, g_cost, h_cost, parent }
    }

    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost().partial_cmp(&self.f_cost()).unwrap_or(Ordering::Equal)
    }
}

/// 寻路地图
pub struct PathfindingMap {
    width: usize,
    height: usize,
    terrain: Vec<Vec<TerrainType>>,
    obstacles: HashSet<Point>,
    dynamic_costs: HashMap<Point, f32>,
}

impl PathfindingMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            terrain: vec![vec![TerrainType::Grass; width]; height],
            obstacles: HashSet::new(),
            dynamic_costs: HashMap::new(),
        }
    }

    pub fn set_terrain(&mut self, point: Point, terrain: TerrainType) {
        if self.is_valid_point(point) {
            self.terrain[point.y as usize][point.x as usize] = terrain;
        }
    }

    pub fn get_terrain(&self, point: Point) -> Option<TerrainType> {
        if self.is_valid_point(point) {
            Some(self.terrain[point.y as usize][point.x as usize])
        } else {
            None
        }
    }

    pub fn add_obstacle(&mut self, point: Point) {
        if self.is_valid_point(point) {
            self.obstacles.insert(point);
        }
    }

    pub fn remove_obstacle(&mut self, point: Point) {
        self.obstacles.remove(&point);
    }

    pub fn set_dynamic_cost(&mut self, point: Point, cost: f32) {
        if self.is_valid_point(point) {
            self.dynamic_costs.insert(point, cost);
        }
    }

    pub fn is_valid_point(&self, point: Point) -> bool {
        point.x >= 0 && point.y >= 0 && 
        point.x < self.width as i32 && point.y < self.height as i32
    }

    pub fn is_walkable(&self, point: Point) -> bool {
        if !self.is_valid_point(point) || self.obstacles.contains(&point) {
            return false;
        }
        
        if let Some(terrain) = self.get_terrain(point) {
            terrain.is_walkable()
        } else {
            false
        }
    }

    pub fn get_movement_cost(&self, from: Point, to: Point) -> f32 {
        if !self.is_walkable(to) {
            return f32::INFINITY;
        }

        let base_cost = if let Some(terrain) = self.get_terrain(to) {
            terrain.movement_cost()
        } else {
            1.0
        };

        let dynamic_cost = self.dynamic_costs.get(&to).unwrap_or(&1.0);
        
        let distance_multiplier = if (from.x - to.x).abs() + (from.y - to.y).abs() == 2 {
            SQRT_2  // 对角线移动
        } else {
            1.0
        };

        base_cost * dynamic_cost * distance_multiplier
    }
}

/// A*寻路算法实现
pub struct AStarPathfinder {
    heuristic_weight: f32,
    allow_diagonal: bool,
}

impl AStarPathfinder {
    pub fn new() -> Self {
        Self {
            heuristic_weight: 1.0,
            allow_diagonal: false,
        }
    }

    pub fn with_diagonal(mut self) -> Self {
        self.allow_diagonal = true;
        self
    }

    pub fn with_heuristic_weight(mut self, weight: f32) -> Self {
        self.heuristic_weight = weight;
        self
    }

    pub fn find_path(&self, map: &PathfindingMap, start: Point, goal: Point) -> Option<Vec<Point>> {
        if start == goal {
            return Some(vec![start]);
        }

        if !map.is_walkable(goal) {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        g_score.insert(start, 0.0);
        open_set.push(PathNode::new(
            start, 
            0.0, 
            self.heuristic(start, goal) * self.heuristic_weight, 
            None
        ));

        while let Some(current_node) = open_set.pop() {
            let current = current_node.position;

            if current == goal {
                return Some(self.reconstruct_path(&came_from, current));
            }

            closed_set.insert(current);

            let neighbors = if self.allow_diagonal {
                current.neighbors_8()
            } else {
                current.neighbors()
            };

            for neighbor in neighbors {
                if closed_set.contains(&neighbor) || !map.is_walkable(neighbor) {
                    continue;
                }

                let tentative_g_score = g_score.get(&current).unwrap_or(&f32::INFINITY) 
                    + map.get_movement_cost(current, neighbor);

                let current_g_score = g_score.get(&neighbor).unwrap_or(&f32::INFINITY);

                if tentative_g_score < *current_g_score {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    let h_score = self.heuristic(neighbor, goal) * self.heuristic_weight;
                    open_set.push(PathNode::new(neighbor, tentative_g_score, h_score, Some(current)));
                }
            }
        }

        None
    }

    fn heuristic(&self, a: Point, b: Point) -> f32 {
        if self.allow_diagonal {
            a.distance_to(&b)
        } else {
            a.manhattan_distance(&b) as f32
        }
    }

    fn reconstruct_path(&self, came_from: &HashMap<Point, Point>, mut current: Point) -> Vec<Point> {
        let mut path = vec![current];
        
        while let Some(&parent) = came_from.get(&current) {
            current = parent;
            path.push(current);
        }
        
        path.reverse();
        path
    }
}

/// Jump Point Search (JPS) 算法实现 - 适用于大地图快速寻路
pub struct JPSPathfinder {
    allow_corner_cutting: bool,
}

impl JPSPathfinder {
    pub fn new() -> Self {
        Self {
            allow_corner_cutting: false,
        }
    }

    pub fn with_corner_cutting(mut self) -> Self {
        self.allow_corner_cutting = true;
        self
    }

    pub fn find_path(&self, map: &PathfindingMap, start: Point, goal: Point) -> Option<Vec<Point>> {
        if start == goal {
            return Some(vec![start]);
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        g_score.insert(start, 0.0);
        open_set.push(PathNode::new(start, 0.0, start.distance_to(&goal), None));

        while let Some(current_node) = open_set.pop() {
            let current = current_node.position;

            if current == goal {
                return Some(self.reconstruct_path(&came_from, current));
            }

            closed_set.insert(current);

            // JPS 特有的跳跃点搜索逻辑
            let successors = self.get_successors(map, current, goal);
            
            for successor in successors {
                if closed_set.contains(&successor) {
                    continue;
                }

                let tentative_g_score = g_score.get(&current).unwrap_or(&f32::INFINITY) 
                    + current.distance_to(&successor);

                let current_g_score = g_score.get(&successor).unwrap_or(&f32::INFINITY);

                if tentative_g_score < *current_g_score {
                    came_from.insert(successor, current);
                    g_score.insert(successor, tentative_g_score);
                    
                    let h_score = successor.distance_to(&goal);
                    open_set.push(PathNode::new(successor, tentative_g_score, h_score, Some(current)));
                }
            }
        }

        None
    }

    fn get_successors(&self, map: &PathfindingMap, current: Point, goal: Point) -> Vec<Point> {
        let mut successors = Vec::new();

        // 获取自然邻居（根据父节点方向）和强制邻居
        let directions = [
            (-1, -1), (0, -1), (1, -1),
            (-1, 0),           (1, 0),
            (-1, 1),  (0, 1),  (1, 1),
        ];

        for &(dx, dy) in &directions {
            let neighbor = Point::new(current.x + dx, current.y + dy);
            
            if let Some(jump_point) = self.jump(map, neighbor, current, goal) {
                successors.push(jump_point);
            }
        }

        successors
    }

    fn jump(&self, map: &PathfindingMap, current: Point, parent: Point, goal: Point) -> Option<Point> {
        if !map.is_walkable(current) {
            return None;
        }

        if current == goal {
            return Some(current);
        }

        let dx = current.x - parent.x;
        let dy = current.y - parent.y;

        // 对角线移动的跳跃逻辑
        if dx != 0 && dy != 0 {
            // 检查强制邻居
            if self.has_forced_neighbor(map, current, dx, dy) {
                return Some(current);
            }

            // 递归检查水平和垂直方向
            if self.jump(map, Point::new(current.x + dx, current.y), current, goal).is_some() ||
               self.jump(map, Point::new(current.x, current.y + dy), current, goal).is_some() {
                return Some(current);
            }
        } else {
            // 水平或垂直移动的跳跃逻辑
            if self.has_forced_neighbor(map, current, dx, dy) {
                return Some(current);
            }
        }

        // 继续在同一方向搜索
        self.jump(map, Point::new(current.x + dx, current.y + dy), current, goal)
    }

    fn has_forced_neighbor(&self, map: &PathfindingMap, current: Point, dx: i32, dy: i32) -> bool {
        if dx != 0 && dy != 0 {
            // 对角线移动的强制邻居检查
            let blocked_x = Point::new(current.x - dx, current.y);
            let blocked_y = Point::new(current.x, current.y - dy);
            let forced_x = Point::new(current.x - dx, current.y + dy);
            let forced_y = Point::new(current.x + dx, current.y - dy);

            (!map.is_walkable(blocked_x) && map.is_walkable(forced_x)) ||
            (!map.is_walkable(blocked_y) && map.is_walkable(forced_y))
        } else if dx != 0 {
            // 水平移动的强制邻居检查
            let blocked_up = Point::new(current.x, current.y - 1);
            let blocked_down = Point::new(current.x, current.y + 1);
            let forced_up = Point::new(current.x + dx, current.y - 1);
            let forced_down = Point::new(current.x + dx, current.y + 1);

            (!map.is_walkable(blocked_up) && map.is_walkable(forced_up)) ||
            (!map.is_walkable(blocked_down) && map.is_walkable(forced_down))
        } else {
            // 垂直移动的强制邻居检查
            let blocked_left = Point::new(current.x - 1, current.y);
            let blocked_right = Point::new(current.x + 1, current.y);
            let forced_left = Point::new(current.x - 1, current.y + dy);
            let forced_right = Point::new(current.x + 1, current.y + dy);

            (!map.is_walkable(blocked_left) && map.is_walkable(forced_left)) ||
            (!map.is_walkable(blocked_right) && map.is_walkable(forced_right))
        }
    }

    fn reconstruct_path(&self, came_from: &HashMap<Point, Point>, mut current: Point) -> Vec<Point> {
        let mut path = vec![current];
        
        while let Some(&parent) = came_from.get(&current) {
            // JPS需要在跳跃点之间插值生成完整路径
            let interpolated = self.interpolate_path(parent, current);
            for point in interpolated.into_iter().rev().skip(1) {
                path.push(point);
            }
            current = parent;
            path.push(current);
        }
        
        path.reverse();
        path
    }

    fn interpolate_path(&self, start: Point, end: Point) -> Vec<Point> {
        let mut path = Vec::new();
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        
        let steps = dx.abs().max(dy.abs()) as usize;
        
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = start.x + (dx as f32 * t) as i32;
            let y = start.y + (dy as f32 * t) as i32;
            path.push(Point::new(x, y));
        }
        
        path
    }
}

/// 流场寻路算法 - 适用于多单位同时寻路
pub struct FlowFieldPathfinder {
    integration_field: HashMap<Point, f32>,
    flow_field: HashMap<Point, Vector2<f32>>,
}

impl FlowFieldPathfinder {
    pub fn new() -> Self {
        Self {
            integration_field: HashMap::new(),
            flow_field: HashMap::new(),
        }
    }

    pub fn generate_flow_field(&mut self, map: &PathfindingMap, goal: Point) {
        self.integration_field.clear();
        self.flow_field.clear();

        // 生成积分场（从目标点开始的Dijkstra）
        self.generate_integration_field(map, goal);
        
        // 基于积分场生成流场
        self.generate_flow_vectors(map);
    }

    fn generate_integration_field(&mut self, map: &PathfindingMap, goal: Point) {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        self.integration_field.insert(goal, 0.0);
        queue.push_back(goal);
        visited.insert(goal);

        while let Some(current) = queue.pop_front() {
            let current_cost = *self.integration_field.get(&current).unwrap();

            for neighbor in current.neighbors() {
                if visited.contains(&neighbor) || !map.is_walkable(neighbor) {
                    continue;
                }

                let movement_cost = map.get_movement_cost(current, neighbor);
                if movement_cost == f32::INFINITY {
                    continue;
                }

                let new_cost = current_cost + movement_cost;
                
                if let Some(&existing_cost) = self.integration_field.get(&neighbor) {
                    if new_cost >= existing_cost {
                        continue;
                    }
                }

                self.integration_field.insert(neighbor, new_cost);
                queue.push_back(neighbor);
                visited.insert(neighbor);
            }
        }
    }

    fn generate_flow_vectors(&mut self, map: &PathfindingMap) {
        for (&point, _) in &self.integration_field {
            let mut best_direction = Vector2::new(0.0, 0.0);
            let mut best_cost = f32::INFINITY;

            for neighbor in point.neighbors() {
                if let Some(&neighbor_cost) = self.integration_field.get(&neighbor) {
                    if neighbor_cost < best_cost {
                        best_cost = neighbor_cost;
                        best_direction = Vector2::new(
                            (neighbor.x - point.x) as f32,
                            (neighbor.y - point.y) as f32
                        );
                    }
                }
            }

            if best_direction.magnitude() > 0.0 {
                best_direction = best_direction.normalize();
            }

            self.flow_field.insert(point, best_direction);
        }
    }

    pub fn get_flow_direction(&self, point: Point) -> Option<Vector2<f32>> {
        self.flow_field.get(&point).copied()
    }

    pub fn sample_path(&self, start: Point, max_steps: usize) -> Vec<Point> {
        let mut path = Vec::new();
        let mut current = start;
        path.push(current);

        for _ in 0..max_steps {
            if let Some(direction) = self.get_flow_direction(current) {
                if direction.magnitude() < 0.1 {
                    break; // 已到达目标附近
                }

                let next_x = current.x + direction.x.round() as i32;
                let next_y = current.y + direction.y.round() as i32;
                let next = Point::new(next_x, next_y);

                if next == current {
                    break;
                }

                current = next;
                path.push(current);
            } else {
                break;
            }
        }

        path
    }
}

/// 分层寻路系统 - 支持世界地图到局部地图的多级导航
pub struct HierarchicalPathfinder {
    clusters: HashMap<Point, ClusterNode>,
    cluster_size: usize,
    inter_cluster_graph: HashMap<Point, Vec<ClusterConnection>>,
}

#[derive(Debug, Clone)]
struct ClusterNode {
    id: Point,
    bounds: (Point, Point),
    entrances: Vec<Point>,
    internal_paths: HashMap<(Point, Point), Vec<Point>>,
}

#[derive(Debug, Clone)]
struct ClusterConnection {
    to_cluster: Point,
    entrance_from: Point,
    entrance_to: Point,
    cost: f32,
}

impl HierarchicalPathfinder {
    pub fn new(cluster_size: usize) -> Self {
        Self {
            clusters: HashMap::new(),
            cluster_size,
            inter_cluster_graph: HashMap::new(),
        }
    }

    pub fn build_hierarchy(&mut self, map: &PathfindingMap) {
        self.create_clusters(map);
        self.find_cluster_entrances(map);
        self.build_inter_cluster_connections(map);
        self.precompute_internal_paths(map);
    }

    fn create_clusters(&mut self, map: &PathfindingMap) {
        let clusters_x = (map.width + self.cluster_size - 1) / self.cluster_size;
        let clusters_y = (map.height + self.cluster_size - 1) / self.cluster_size;

        for cy in 0..clusters_y {
            for cx in 0..clusters_x {
                let cluster_id = Point::new(cx as i32, cy as i32);
                let min_x = cx * self.cluster_size;
                let min_y = cy * self.cluster_size;
                let max_x = ((cx + 1) * self.cluster_size).min(map.width) - 1;
                let max_y = ((cy + 1) * self.cluster_size).min(map.height) - 1;

                let cluster = ClusterNode {
                    id: cluster_id,
                    bounds: (Point::new(min_x as i32, min_y as i32), Point::new(max_x as i32, max_y as i32)),
                    entrances: Vec::new(),
                    internal_paths: HashMap::new(),
                };

                self.clusters.insert(cluster_id, cluster);
            }
        }
    }

    fn find_cluster_entrances(&mut self, map: &PathfindingMap) {
        for cluster in self.clusters.values_mut() {
            let (min_bound, max_bound) = cluster.bounds;
            let mut entrances = Vec::new();

            // 检查集群边界上的可通行点
            for x in min_bound.x..=max_bound.x {
                // 上边界
                if min_bound.y > 0 {
                    let point = Point::new(x, min_bound.y);
                    let above = Point::new(x, min_bound.y - 1);
                    if map.is_walkable(point) && map.is_walkable(above) {
                        entrances.push(point);
                    }
                }
                
                // 下边界
                if max_bound.y < (map.height as i32) - 1 {
                    let point = Point::new(x, max_bound.y);
                    let below = Point::new(x, max_bound.y + 1);
                    if map.is_walkable(point) && map.is_walkable(below) {
                        entrances.push(point);
                    }
                }
            }

            for y in min_bound.y..=max_bound.y {
                // 左边界
                if min_bound.x > 0 {
                    let point = Point::new(min_bound.x, y);
                    let left = Point::new(min_bound.x - 1, y);
                    if map.is_walkable(point) && map.is_walkable(left) {
                        entrances.push(point);
                    }
                }
                
                // 右边界
                if max_bound.x < (map.width as i32) - 1 {
                    let point = Point::new(max_bound.x, y);
                    let right = Point::new(max_bound.x + 1, y);
                    if map.is_walkable(point) && map.is_walkable(right) {
                        entrances.push(point);
                    }
                }
            }

            cluster.entrances = entrances;
        }
    }

    fn build_inter_cluster_connections(&mut self, map: &PathfindingMap) {
        for (&cluster_id, cluster) in &self.clusters {
            let mut connections = Vec::new();

            // 检查相邻集群的连接
            let adjacent_clusters = [
                Point::new(cluster_id.x - 1, cluster_id.y),
                Point::new(cluster_id.x + 1, cluster_id.y),
                Point::new(cluster_id.x, cluster_id.y - 1),
                Point::new(cluster_id.x, cluster_id.y + 1),
            ];

            for adj_cluster_id in adjacent_clusters {
                if let Some(adj_cluster) = self.clusters.get(&adj_cluster_id) {
                    // 找到两个集群之间的连接点
                    for &entrance_from in &cluster.entrances {
                        for &entrance_to in &adj_cluster.entrances {
                            if entrance_from.manhattan_distance(&entrance_to) == 1 {
                                let cost = map.get_movement_cost(entrance_from, entrance_to);
                                if cost < f32::INFINITY {
                                    connections.push(ClusterConnection {
                                        to_cluster: adj_cluster_id,
                                        entrance_from,
                                        entrance_to,
                                        cost,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            self.inter_cluster_graph.insert(cluster_id, connections);
        }
    }

    fn precompute_internal_paths(&mut self, map: &PathfindingMap) {
        let pathfinder = AStarPathfinder::new();

        for cluster in self.clusters.values_mut() {
            // 预计算集群内所有入口点之间的路径
            for (i, &entrance_a) in cluster.entrances.iter().enumerate() {
                for (j, &entrance_b) in cluster.entrances.iter().enumerate() {
                    if i != j {
                        if let Some(path) = pathfinder.find_path(map, entrance_a, entrance_b) {
                            cluster.internal_paths.insert((entrance_a, entrance_b), path);
                        }
                    }
                }
            }
        }
    }

    pub fn find_hierarchical_path(&self, map: &PathfindingMap, start: Point, goal: Point) -> Option<Vec<Point>> {
        let start_cluster = self.get_cluster_for_point(start)?;
        let goal_cluster = self.get_cluster_for_point(goal)?;

        if start_cluster == goal_cluster {
            // 同一集群内的路径查找
            let pathfinder = AStarPathfinder::new();
            return pathfinder.find_path(map, start, goal);
        }

        // 多集群路径查找
        let cluster_path = self.find_cluster_path(start_cluster, goal_cluster)?;
        let mut full_path = Vec::new();

        // 从起点到第一个集群出口
        let first_cluster = self.clusters.get(&start_cluster)?;
        let start_entrance = self.find_best_entrance(first_cluster, start, cluster_path.get(1).copied())?;
        
        let pathfinder = AStarPathfinder::new();
        if let Some(path_to_entrance) = pathfinder.find_path(map, start, start_entrance) {
            full_path.extend(path_to_entrance);
        }

        // 集群间的路径
        for window in cluster_path.windows(2) {
            let from_cluster = self.clusters.get(&window[0])?;
            let to_cluster = self.clusters.get(&window[1])?;
            
            if let Some(connection) = self.find_connection(window[0], window[1]) {
                // 集群内部路径
                if let Some(internal_path) = from_cluster.internal_paths.get(&(start_entrance, connection.entrance_from)) {
                    full_path.extend(internal_path.iter().skip(1));
                }
                
                // 集群间跳跃
                full_path.push(connection.entrance_to);
            }
        }

        // 从最后一个集群入口到目标点
        let last_cluster = self.clusters.get(&goal_cluster)?;
        let goal_entrance = self.find_best_entrance(last_cluster, goal, cluster_path.get(cluster_path.len() - 2).copied())?;
        
        if let Some(path_from_entrance) = pathfinder.find_path(map, goal_entrance, goal) {
            full_path.extend(path_from_entrance.iter().skip(1));
        }

        Some(full_path)
    }

    fn get_cluster_for_point(&self, point: Point) -> Option<Point> {
        let cluster_x = point.x / self.cluster_size as i32;
        let cluster_y = point.y / self.cluster_size as i32;
        let cluster_id = Point::new(cluster_x, cluster_y);
        
        if self.clusters.contains_key(&cluster_id) {
            Some(cluster_id)
        } else {
            None
        }
    }

    fn find_cluster_path(&self, start_cluster: Point, goal_cluster: Point) -> Option<Vec<Point>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        g_score.insert(start_cluster, 0.0);
        open_set.push(PathNode::new(start_cluster, 0.0, start_cluster.distance_to(&goal_cluster), None));

        while let Some(current_node) = open_set.pop() {
            let current = current_node.position;

            if current == goal_cluster {
                let mut path = vec![current];
                let mut curr = current;
                while let Some(&parent) = came_from.get(&curr) {
                    curr = parent;
                    path.push(curr);
                }
                path.reverse();
                return Some(path);
            }

            closed_set.insert(current);

            if let Some(connections) = self.inter_cluster_graph.get(&current) {
                for connection in connections {
                    let neighbor = connection.to_cluster;
                    
                    if closed_set.contains(&neighbor) {
                        continue;
                    }

                    let tentative_g_score = g_score.get(&current).unwrap_or(&f32::INFINITY) + connection.cost;
                    let current_g_score = g_score.get(&neighbor).unwrap_or(&f32::INFINITY);

                    if tentative_g_score < *current_g_score {
                        came_from.insert(neighbor, current);
                        g_score.insert(neighbor, tentative_g_score);
                        
                        let h_score = neighbor.distance_to(&goal_cluster);
                        open_set.push(PathNode::new(neighbor, tentative_g_score, h_score, Some(current)));
                    }
                }
            }
        }

        None
    }

    fn find_best_entrance(&self, cluster: &ClusterNode, point: Point, next_cluster: Option<Point>) -> Option<Point> {
        if cluster.entrances.is_empty() {
            return None;
        }

        if let Some(next) = next_cluster {
            // 找到通向下一个集群的最佳入口
            if let Some(connections) = self.inter_cluster_graph.get(&cluster.id) {
                for connection in connections {
                    if connection.to_cluster == next {
                        return Some(connection.entrance_from);
                    }
                }
            }
        }

        // 否则返回距离最近的入口
        cluster.entrances.iter()
            .min_by(|&&a, &&b| {
                a.distance_to(&point).partial_cmp(&b.distance_to(&point)).unwrap_or(Ordering::Equal)
            })
            .copied()
    }

    fn find_connection(&self, from_cluster: Point, to_cluster: Point) -> Option<&ClusterConnection> {
        self.inter_cluster_graph.get(&from_cluster)?
            .iter()
            .find(|conn| conn.to_cluster == to_cluster)
    }
}

/// 路径平滑处理
pub struct PathSmoother;

impl PathSmoother {
    pub fn smooth_path(map: &PathfindingMap, path: &[Point]) -> Vec<Point> {
        if path.len() <= 2 {
            return path.to_vec();
        }

        let mut smoothed = vec![path[0]];
        let mut current_index = 0;

        while current_index < path.len() - 1 {
            let start = path[current_index];
            let mut farthest_visible = current_index + 1;

            // 找到从当前点能直接看到的最远点
            for i in (current_index + 2)..path.len() {
                if Self::has_line_of_sight(map, start, path[i]) {
                    farthest_visible = i;
                } else {
                    break;
                }
            }

            smoothed.push(path[farthest_visible]);
            current_index = farthest_visible;
        }

        smoothed
    }

    fn has_line_of_sight(map: &PathfindingMap, from: Point, to: Point) -> bool {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = dx.abs().max(dy.abs());

        if steps == 0 {
            return true;
        }

        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let x = from.x + (dx as f32 * t) as i32;
            let y = from.y + (dy as f32 * t) as i32;
            let point = Point::new(x, y);

            if !map.is_walkable(point) {
                return false;
            }
        }

        true
    }

    pub fn bezier_smooth_path(path: &[Point], control_strength: f32) -> Vec<Vector2<f32>> {
        if path.len() <= 2 {
            return path.iter().map(|p| Vector2::new(p.x as f32, p.y as f32)).collect();
        }

        let mut smoothed = Vec::new();
        let points: Vec<Vector2<f32>> = path.iter()
            .map(|p| Vector2::new(p.x as f32, p.y as f32))
            .collect();

        for i in 0..points.len() - 1 {
            let p0 = points[i];
            let p1 = points[i + 1];
            
            let control1 = if i > 0 {
                let direction = (points[i + 1] - points[i - 1]).normalize();
                p0 + direction * control_strength
            } else {
                p0
            };

            let control2 = if i + 2 < points.len() {
                let direction = (points[i + 2] - points[i]).normalize();
                p1 - direction * control_strength
            } else {
                p1
            };

            // 生成贝塞尔曲线上的点
            for t in 0..10 {
                let t = t as f32 / 10.0;
                let point = Self::cubic_bezier(p0, control1, control2, p1, t);
                smoothed.push(point);
            }
        }

        smoothed.push(*points.last().unwrap());
        smoothed
    }

    fn cubic_bezier(p0: Vector2<f32>, p1: Vector2<f32>, p2: Vector2<f32>, p3: Vector2<f32>, t: f32) -> Vector2<f32> {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        p0 * uuu + p1 * (3.0 * uu * t) + p2 * (3.0 * u * tt) + p3 * ttt
    }
}

#[cfg(feature = "pathfinding-wip")]
/// 并行寻路管理器 - 支持多个寻路请求的并发处理
pub struct ParallelPathfindingManager {
    thread_pool_size: usize,
}

#[cfg(feature = "pathfinding-wip")]
impl ParallelPathfindingManager {
    pub fn new(thread_pool_size: usize) -> Self {
        Self { thread_pool_size }
    }

    pub fn find_multiple_paths(
        &self, 
        map: &PathfindingMap, 
        requests: Vec<(Point, Point)>
    ) -> Vec<Option<Vec<Point>>> {
        requests.into_par_iter()
            .map(|(start, goal)| {
                let pathfinder = AStarPathfinder::new();
                pathfinder.find_path(map, start, goal)
            })
            .collect()
    }

    pub fn generate_multiple_flow_fields(
        &self,
        map: &PathfindingMap,
        goals: Vec<Point>
    ) -> Vec<FlowFieldPathfinder> {
        goals.into_par_iter()
            .map(|goal| {
                let mut pathfinder = FlowFieldPathfinder::new();
                pathfinder.generate_flow_field(map, goal);
                pathfinder
            })
            .collect()
    }
}

/// 寻路缓存系统
pub struct PathfindingCache {
    cache: HashMap<(Point, Point), Vec<Point>>,
    max_entries: usize,
    access_order: VecDeque<(Point, Point)>,
}

impl PathfindingCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            access_order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, start: Point, goal: Point) -> Option<&Vec<Point>> {
        let key = (start, goal);
        
        if let Some(path) = self.cache.get(&key) {
            // 更新访问顺序 (LRU)
            if let Some(pos) = self.access_order.iter().position(|&x| x == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key);
            Some(path)
        } else {
            None
        }
    }

    pub fn insert(&mut self, start: Point, goal: Point, path: Vec<Point>) {
        let key = (start, goal);

        // 如果缓存已满，删除最少使用的条目
        if self.cache.len() >= self.max_entries {
            if let Some(lru_key) = self.access_order.pop_front() {
                self.cache.remove(&lru_key);
            }
        }

        self.cache.insert(key, path);
        self.access_order.push_back(key);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    pub fn invalidate_region(&mut self, min_point: Point, max_point: Point) {
        let keys_to_remove: Vec<_> = self.cache.keys()
            .filter(|(start, goal)| {
                Self::point_in_region(*start, min_point, max_point) ||
                Self::point_in_region(*goal, min_point, max_point)
            })
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            if let Some(pos) = self.access_order.iter().position(|&x| x == key) {
                self.access_order.remove(pos);
            }
        }
    }

    fn point_in_region(point: Point, min: Point, max: Point) -> bool {
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astar_pathfinding() {
        let mut map = PathfindingMap::new(10, 10);
        
        // 创建一个简单的障碍物
        map.add_obstacle(Point::new(2, 2));
        map.add_obstacle(Point::new(2, 3));
        map.add_obstacle(Point::new(2, 4));

        let pathfinder = AStarPathfinder::new();
        let start = Point::new(0, 3);
        let goal = Point::new(5, 3);
        
        let path = pathfinder.find_path(&map, start, goal);
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
        
        // 确保路径不经过障碍物
        for point in &path {
            assert!(!map.obstacles.contains(point));
        }
    }

    #[test]
    fn test_flow_field_generation() {
        let map = PathfindingMap::new(5, 5);
        let mut pathfinder = FlowFieldPathfinder::new();
        let goal = Point::new(2, 2);
        
        pathfinder.generate_flow_field(&map, goal);
        
        // 测试流场方向
        let direction = pathfinder.get_flow_direction(Point::new(0, 0));
        assert!(direction.is_some());
        
        let sample_path = pathfinder.sample_path(Point::new(0, 0), 10);
        assert!(!sample_path.is_empty());
    }

    #[test]
    fn test_hierarchical_pathfinding() {
        let map = PathfindingMap::new(20, 20);
        let mut pathfinder = HierarchicalPathfinder::new(5);
        
        pathfinder.build_hierarchy(&map);
        
        let start = Point::new(1, 1);
        let goal = Point::new(18, 18);
        
        let path = pathfinder.find_hierarchical_path(&map, start, goal);
        assert!(path.is_some());
    }

    #[test]
    fn test_path_smoothing() {
        let map = PathfindingMap::new(10, 10);
        let path = vec![
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(2, 0),
            Point::new(3, 1),
            Point::new(4, 2),
            Point::new(5, 2),
        ];
        
        let smoothed = PathSmoother::smooth_path(&map, &path);
        assert!(smoothed.len() <= path.len());
        assert_eq!(smoothed.first(), path.first());
        assert_eq!(smoothed.last(), path.last());
    }

    #[test]
    fn test_pathfinding_cache() {
        let mut cache = PathfindingCache::new(3);
        let start = Point::new(0, 0);
        let goal = Point::new(5, 5);
        let path = vec![start, Point::new(2, 2), goal];
        
        cache.insert(start, goal, path.clone());
        
        let cached_path = cache.get(start, goal);
        assert!(cached_path.is_some());
        assert_eq!(cached_path.unwrap(), &path);
    }
}