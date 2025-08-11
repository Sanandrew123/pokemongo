/*
* 开发心理过程：
* 1. 设计完整的地点系统，包括城镇、道路、特殊地点
* 2. 实现地点之间的连接和传送机制
* 3. 支持动态加载和卸载地点，优化内存使用
* 4. 集成地点特有的功能：商店、治疗中心、道馆等
* 5. 提供地点事件系统，支持剧情触发和任务系统
* 6. 实现地点访问历史和解锁机制
* 7. 支持多层级地点结构，如建筑物内部
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    world::{
        tile::{TileMap, TerrainType},
        spawn::{SpawnManager, SpawnPoint},
        weather::{WeatherSystem, WeatherCondition},
        collision::CollisionWorld,
    },
    pokemon::species::SpeciesId,
    player::trainer::TrainerId,
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Component)]
pub struct LocationManager {
    /// 所有地点
    pub locations: HashMap<LocationId, Location>,
    /// 当前活跃地点
    pub active_location: Option<LocationId>,
    /// 地点连接图
    pub connections: HashMap<LocationId, Vec<LocationConnection>>,
    /// 加载的地点数据
    pub loaded_locations: HashMap<LocationId, LocationData>,
    /// 地点访问历史
    pub visit_history: Vec<LocationVisit>,
    /// 全局位置配置
    pub config: LocationConfig,
}

pub type LocationId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub location_type: LocationType,
    pub region: String,
    pub coordinates: Vec2,
    pub bounds: LocationBounds,
    pub services: Vec<LocationService>,
    pub npcs: Vec<NpcInfo>,
    pub items: Vec<LocationItem>,
    pub access_requirements: Vec<AccessRequirement>,
    pub is_unlocked: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    Town,           // 城镇
    City,           // 城市
    Route,          // 道路
    Cave,           // 洞穴
    Forest,         // 森林
    Beach,          // 海滩
    Mountain,       // 山脉
    Lake,           // 湖泊
    Gym,            // 道馆
    PokemonCenter,  // Pokemon中心
    Shop,           // 商店
    House,          // 房屋
    Building,       // 建筑物
    Dungeon,        // 地牢
    SpecialArea,    // 特殊区域
    Custom(u16),    // 自定义类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationBounds {
    Circle { center: Vec2, radius: f32 },
    Rectangle { min: Vec2, max: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationService {
    pub service_type: ServiceType,
    pub npc_id: Option<String>,
    pub is_available: bool,
    pub requirements: Vec<ServiceRequirement>,
    pub cost: Option<ServiceCost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    Healing,        // 治疗服务
    PokemonStorage, // Pokemon存储
    Shop,           // 商店
    Gym,            // 道馆挑战
    Transportation, // 传送服务
    Training,       // 训练服务
    Breeding,       // 繁殖服务
    Contest,        // 比赛
    GameCorner,     // 游戏角
    Custom(u16),    // 自定义服务
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirement {
    pub requirement_type: RequirementType,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementType {
    Badge,          // 徽章要求
    Pokemon,        // Pokemon要求
    Item,           // 道具要求
    Level,          // 等级要求
    QuestComplete,  // 任务完成要求
    Custom,         // 自定义要求
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCost {
    pub currency_type: CurrencyType,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrencyType {
    Money,          // 金钱
    BattlePoints,   // 对战点数
    Tokens,         // 代币
    Custom(u16),    // 自定义货币
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInfo {
    pub id: String,
    pub name: String,
    pub npc_type: NpcType,
    pub position: Vec2,
    pub dialogue: Vec<DialogueEntry>,
    pub is_active: bool,
    pub schedule: Option<NpcSchedule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcType {
    Citizen,        // 普通市民
    Trainer,        // 训练师
    GymLeader,      // 道馆馆主
    Nurse,          // 护士
    Shopkeeper,     // 店员
    Guard,          // 守卫
    Guide,          // 向导
    QuestGiver,     // 任务发布者
    Custom(u16),    // 自定义NPC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEntry {
    pub id: String,
    pub text: String,
    pub conditions: Vec<DialogueCondition>,
    pub actions: Vec<DialogueAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCondition {
    PlayerLevel(u8),
    HasPokemon(SpeciesId),
    HasItem(u32),
    QuestStatus(String, String), // quest_id, status
    TimeOfDay(crate::world::environment::TimeOfDay),
    Custom(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueAction {
    GiveItem(u32, u32), // item_id, quantity
    StartQuest(String),
    TakeMoney(u32),
    GiveMoney(u32),
    StartBattle(Vec<SpeciesId>),
    Heal,
    Custom(String, HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSchedule {
    pub entries: Vec<ScheduleEntry>,
    pub default_position: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time_range: (f32, f32), // 开始时间, 结束时间 (小时)
    pub position: Vec2,
    pub activity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationItem {
    pub item_id: u32,
    pub quantity: u32,
    pub position: Vec2,
    pub is_hidden: bool,
    pub respawn_time: Option<f32>,
    pub requirements: Vec<AccessRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessRequirement {
    Badge(String),
    Pokemon(SpeciesId),
    Item(u32),
    QuestCompleted(String),
    Level(u8),
    Time(crate::world::environment::TimeOfDay),
    Weather(WeatherCondition),
    Custom(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConnection {
    pub target_location: LocationId,
    pub connection_type: ConnectionType,
    pub entrance_point: Vec2,
    pub exit_point: Vec2,
    pub requirements: Vec<AccessRequirement>,
    pub is_bidirectional: bool,
    pub travel_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Walk,           // 步行连接
    Surf,           // 冲浪连接
    Fly,            // 飞行连接
    Teleport,       // 传送连接
    Door,           // 门连接
    Stairs,         // 楼梯连接
    Elevator,       // 电梯连接
    Warp,           // 扭曲连接
    Custom(u16),    // 自定义连接
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub tilemap: TileMap,
    pub spawn_manager: SpawnManager,
    pub weather_system: WeatherSystem,
    pub collision_world: CollisionWorld,
    pub is_loaded: bool,
    pub last_accessed: f64,
    pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct LocationVisit {
    pub location_id: LocationId,
    pub timestamp: f64,
    pub player_level: u8,
    pub duration: f32,
    pub actions_performed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LocationConfig {
    /// 最大同时加载的地点数量
    pub max_loaded_locations: usize,
    /// 自动卸载时间（秒）
    pub auto_unload_time: f32,
    /// 预加载相邻地点
    pub preload_adjacent: bool,
    /// 内存使用限制（MB）
    pub memory_limit_mb: f32,
}

impl LocationManager {
    pub fn new(config: LocationConfig) -> Self {
        Self {
            locations: HashMap::new(),
            active_location: None,
            connections: HashMap::new(),
            loaded_locations: HashMap::new(),
            visit_history: Vec::new(),
            config,
        }
    }

    /// 添加地点
    pub fn add_location(&mut self, location: Location) {
        let id = location.id;
        self.locations.insert(id, location);
    }

    /// 添加地点连接
    pub fn add_connection(&mut self, from: LocationId, connection: LocationConnection) {
        self.connections.entry(from).or_default().push(connection);
    }

    /// 获取地点信息
    pub fn get_location(&self, id: LocationId) -> Option<&Location> {
        self.locations.get(&id)
    }

    /// 检查地点是否可访问
    pub fn can_access_location(
        &self,
        location_id: LocationId,
        player_level: u8,
        player_items: &[u32],
        completed_quests: &[String],
    ) -> bool {
        let location = match self.locations.get(&location_id) {
            Some(loc) => loc,
            None => return false,
        };

        if !location.is_unlocked {
            return false;
        }

        for requirement in &location.access_requirements {
            if !self.check_requirement(requirement, player_level, player_items, completed_quests) {
                return false;
            }
        }

        true
    }

    fn check_requirement(
        &self,
        requirement: &AccessRequirement,
        player_level: u8,
        player_items: &[u32],
        completed_quests: &[String],
    ) -> bool {
        match requirement {
            AccessRequirement::Level(required_level) => player_level >= *required_level,
            AccessRequirement::Item(item_id) => player_items.contains(item_id),
            AccessRequirement::QuestCompleted(quest_id) => completed_quests.contains(quest_id),
            AccessRequirement::Badge(_badge_name) => {
                // 需要徽章系统检查
                true // 临时返回
            },
            AccessRequirement::Pokemon(_species_id) => {
                // 需要检查玩家队伍
                true // 临时返回
            },
            AccessRequirement::Time(_time) => {
                // 需要时间系统检查
                true // 临时返回
            },
            AccessRequirement::Weather(_weather) => {
                // 需要天气系统检查
                true // 临时返回
            },
            AccessRequirement::Custom(_name, _value) => {
                // 自定义要求检查
                true // 临时返回
            },
        }
    }

    /// 切换到指定地点
    pub fn travel_to_location(
        &mut self,
        location_id: LocationId,
        player_position: Vec2,
        current_time: f64,
    ) -> GameResult<TravelResult> {
        // 检查连接是否存在
        if let Some(current_id) = self.active_location {
            if !self.has_connection(current_id, location_id) {
                return Err(GameError::WorldError("没有到目标地点的连接".to_string()));
            }
        }

        // 记录访问历史
        if let Some(current_id) = self.active_location {
            if let Some(last_visit) = self.visit_history.last_mut() {
                if last_visit.location_id == current_id {
                    last_visit.duration = (current_time - last_visit.timestamp) as f32;
                }
            }
        }

        // 加载目标地点
        self.load_location(location_id)?;

        // 记录新的访问
        self.visit_history.push(LocationVisit {
            location_id,
            timestamp: current_time,
            player_level: 1, // 需要从玩家系统获取
            duration: 0.0,
            actions_performed: Vec::new(),
        });

        // 设置为当前地点
        self.active_location = Some(location_id);

        // 预加载相邻地点
        if self.config.preload_adjacent {
            self.preload_adjacent_locations(location_id)?;
        }

        // 清理旧的地点数据
        self.cleanup_old_locations(current_time)?;

        Ok(TravelResult {
            destination: location_id,
            spawn_position: self.get_spawn_position(location_id, player_position)?,
            travel_time: self.calculate_travel_time(self.active_location, Some(location_id))?,
        })
    }

    fn has_connection(&self, from: LocationId, to: LocationId) -> bool {
        if let Some(connections) = self.connections.get(&from) {
            connections.iter().any(|conn| conn.target_location == to)
        } else {
            false
        }
    }

    fn load_location(&mut self, location_id: LocationId) -> GameResult<()> {
        if self.loaded_locations.contains_key(&location_id) {
            return Ok(());
        }

        // 检查内存限制
        if self.loaded_locations.len() >= self.config.max_loaded_locations {
            self.unload_oldest_location()?;
        }

        // 创建地点数据
        let location_data = self.create_location_data(location_id)?;
        self.loaded_locations.insert(location_id, location_data);

        Ok(())
    }

    fn create_location_data(&self, location_id: LocationId) -> GameResult<LocationData> {
        let location = self.locations.get(&location_id)
            .ok_or_else(|| GameError::WorldError("地点不存在".to_string()))?;

        // 创建基础数据结构
        let tilemap = self.generate_tilemap_for_location(location)?;
        let spawn_manager = self.create_spawn_manager_for_location(location)?;
        let weather_system = self.create_weather_system_for_location(location)?;
        let collision_world = self.create_collision_world_for_location(location)?;

        Ok(LocationData {
            tilemap,
            spawn_manager,
            weather_system,
            collision_world,
            is_loaded: true,
            last_accessed: 0.0,
            memory_usage: 1024, // 简化的内存使用估算
        })
    }

    fn generate_tilemap_for_location(&self, location: &Location) -> GameResult<TileMap> {
        // 简化实现：根据地点类型生成基础地图
        use crate::world::tile::Tileset;
        use crate::utils::random::RandomGenerator;
        
        let tileset = Tileset {
            id: 1,
            name: "Default".to_string(),
            texture_id: 0,
            tile_size: bevy::math::IVec2::new(32, 32),
            tile_count: bevy::math::IVec2::new(16, 16),
            spacing: bevy::math::IVec2::ZERO,
            margin: bevy::math::IVec2::ZERO,
            tile_metadata: HashMap::new(),
        };

        let mut rng = RandomGenerator::new();
        let map_size = self.get_location_map_size(location);
        
        TileMap::generate_random_map(
            map_size,
            bevy::math::IVec2::new(32, 32),
            &tileset,
            &mut rng,
        )
    }

    fn get_location_map_size(&self, location: &Location) -> bevy::math::IVec2 {
        match location.location_type {
            LocationType::City => bevy::math::IVec2::new(200, 200),
            LocationType::Town => bevy::math::IVec2::new(100, 100),
            LocationType::Route => bevy::math::IVec2::new(50, 200),
            LocationType::Cave => bevy::math::IVec2::new(80, 80),
            LocationType::House => bevy::math::IVec2::new(20, 20),
            _ => bevy::math::IVec2::new(64, 64),
        }
    }

    fn create_spawn_manager_for_location(&self, location: &Location) -> GameResult<SpawnManager> {
        use crate::world::spawn::{GlobalSpawnConfig, SpawnManager};
        
        let config = GlobalSpawnConfig::default();
        let mut manager = SpawnManager::new(config);

        // 根据地点类型添加生成点
        match location.location_type {
            LocationType::Route => {
                let spawn_point = SpawnManager::create_grassland_spawn_point(Vec2::new(50.0, 100.0));
                manager.add_spawn_point(spawn_point);
            },
            LocationType::Cave => {
                // 添加洞穴Pokemon生成点
            },
            _ => {},
        }

        Ok(manager)
    }

    fn create_weather_system_for_location(&self, location: &Location) -> GameResult<WeatherSystem> {
        use crate::world::weather::{WeatherConfig, WeatherSystem};
        
        let mut config = WeatherConfig::default();
        
        // 根据地点类型调整天气配置
        match location.location_type {
            LocationType::Cave => {
                config.dynamic_weather = false; // 洞穴内天气稳定
            },
            LocationType::House | LocationType::Building => {
                config.dynamic_weather = false; // 建筑物内没有天气
            },
            _ => {},
        }

        Ok(WeatherSystem::new(config))
    }

    fn create_collision_world_for_location(&self, location: &Location) -> GameResult<CollisionWorld> {
        use crate::world::collision::{CollisionConfig, CollisionWorld};
        
        let config = CollisionConfig::default();
        let bounds = bevy::prelude::Rect::from_center_size(Vec2::ZERO, Vec2::splat(1000.0));
        Ok(CollisionWorld::new(bounds, config.grid_cell_size))
    }

    fn preload_adjacent_locations(&mut self, location_id: LocationId) -> GameResult<()> {
        if let Some(connections) = self.connections.get(&location_id).cloned() {
            for connection in connections {
                if !self.loaded_locations.contains_key(&connection.target_location) {
                    // 异步预加载
                    if let Err(e) = self.load_location(connection.target_location) {
                        warn!("预加载地点 {} 失败: {:?}", connection.target_location, e);
                    }
                }
            }
        }
        Ok(())
    }

    fn cleanup_old_locations(&mut self, current_time: f64) -> GameResult<()> {
        let locations_to_unload: Vec<LocationId> = self.loaded_locations
            .iter()
            .filter(|(id, data)| {
                **id != self.active_location.unwrap_or(0) &&
                current_time - data.last_accessed > self.config.auto_unload_time as f64
            })
            .map(|(id, _)| *id)
            .collect();

        for location_id in locations_to_unload {
            self.unload_location(location_id)?;
        }

        Ok(())
    }

    fn unload_oldest_location(&mut self) -> GameResult<()> {
        let oldest = self.loaded_locations
            .iter()
            .filter(|(id, _)| **id != self.active_location.unwrap_or(0))
            .min_by(|(_, a), (_, b)| a.last_accessed.partial_cmp(&b.last_accessed).unwrap())
            .map(|(id, _)| *id);

        if let Some(location_id) = oldest {
            self.unload_location(location_id)?;
        }

        Ok(())
    }

    fn unload_location(&mut self, location_id: LocationId) -> GameResult<()> {
        if let Some(location_data) = self.loaded_locations.remove(&location_id) {
            // 清理资源
            drop(location_data);
            info!("卸载地点: {}", location_id);
        }
        Ok(())
    }

    fn get_spawn_position(&self, location_id: LocationId, _player_pos: Vec2) -> GameResult<Vec2> {
        // 简化实现：返回地点中心位置
        if let Some(location) = self.locations.get(&location_id) {
            match &location.bounds {
                LocationBounds::Circle { center, .. } => Ok(*center),
                LocationBounds::Rectangle { min, max } => Ok((*min + *max) * 0.5),
                LocationBounds::Polygon { vertices } => {
                    if !vertices.is_empty() {
                        Ok(vertices[0])
                    } else {
                        Ok(Vec2::ZERO)
                    }
                },
            }
        } else {
            Ok(Vec2::ZERO)
        }
    }

    fn calculate_travel_time(&self, from: Option<LocationId>, to: Option<LocationId>) -> GameResult<f32> {
        // 简化实现
        Ok(1.0)
    }

    /// 获取地点的连接列表
    pub fn get_connections(&self, location_id: LocationId) -> Vec<&LocationConnection> {
        self.connections.get(&location_id).map_or(Vec::new(), |conns| conns.iter().collect())
    }

    /// 获取已访问的地点列表
    pub fn get_visited_locations(&self) -> Vec<LocationId> {
        self.visit_history.iter().map(|visit| visit.location_id).collect::<std::collections::HashSet<_>>().into_iter().collect()
    }

    /// 获取地点数据
    pub fn get_location_data(&self, location_id: LocationId) -> Option<&LocationData> {
        self.loaded_locations.get(&location_id)
    }

    /// 获取当前地点
    pub fn get_current_location(&self) -> Option<LocationId> {
        self.active_location
    }

    /// 解锁地点
    pub fn unlock_location(&mut self, location_id: LocationId) -> GameResult<()> {
        if let Some(location) = self.locations.get_mut(&location_id) {
            location.is_unlocked = true;
            Ok(())
        } else {
            Err(GameError::WorldError("地点不存在".to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TravelResult {
    pub destination: LocationId,
    pub spawn_position: Vec2,
    pub travel_time: f32,
}

impl LocationBounds {
    pub fn contains(&self, position: Vec2) -> bool {
        match self {
            LocationBounds::Circle { center, radius } => {
                center.distance(position) <= *radius
            },
            LocationBounds::Rectangle { min, max } => {
                position.x >= min.x && position.x <= max.x &&
                position.y >= min.y && position.y <= max.y
            },
            LocationBounds::Polygon { vertices } => {
                // 简化的点在多边形内检测
                false // 需要实现射线投射算法
            },
        }
    }

    pub fn get_center(&self) -> Vec2 {
        match self {
            LocationBounds::Circle { center, .. } => *center,
            LocationBounds::Rectangle { min, max } => (*min + *max) * 0.5,
            LocationBounds::Polygon { vertices } => {
                if vertices.is_empty() {
                    Vec2::ZERO
                } else {
                    let sum: Vec2 = vertices.iter().sum();
                    sum / vertices.len() as f32
                }
            },
        }
    }
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            max_loaded_locations: 5,
            auto_unload_time: 300.0, // 5分钟
            preload_adjacent: true,
            memory_limit_mb: 100.0,
        }
    }
}

// 预设地点创建函数
impl LocationManager {
    /// 创建城镇
    pub fn create_town(id: LocationId, name: String, coordinates: Vec2) -> Location {
        Location {
            id,
            name: name.clone(),
            display_name: name,
            description: "一个宁静的小镇".to_string(),
            location_type: LocationType::Town,
            region: "主要区域".to_string(),
            coordinates,
            bounds: LocationBounds::Rectangle {
                min: coordinates - Vec2::splat(50.0),
                max: coordinates + Vec2::splat(50.0),
            },
            services: vec![
                LocationService {
                    service_type: ServiceType::Healing,
                    npc_id: Some("nurse_joy".to_string()),
                    is_available: true,
                    requirements: vec![],
                    cost: None,
                },
                LocationService {
                    service_type: ServiceType::Shop,
                    npc_id: Some("shopkeeper".to_string()),
                    is_available: true,
                    requirements: vec![],
                    cost: None,
                },
            ],
            npcs: vec![],
            items: vec![],
            access_requirements: vec![],
            is_unlocked: true,
            metadata: HashMap::new(),
        }
    }

    /// 创建道路
    pub fn create_route(id: LocationId, name: String, start: Vec2, end: Vec2) -> Location {
        Location {
            id,
            name: name.clone(),
            display_name: name,
            description: "连接各地的道路".to_string(),
            location_type: LocationType::Route,
            region: "主要区域".to_string(),
            coordinates: (start + end) * 0.5,
            bounds: LocationBounds::Rectangle { min: start, max: end },
            services: vec![],
            npcs: vec![],
            items: vec![],
            access_requirements: vec![],
            is_unlocked: true,
            metadata: HashMap::new(),
        }
    }

    /// 创建Pokemon中心
    pub fn create_pokemon_center(id: LocationId, coordinates: Vec2) -> Location {
        Location {
            id,
            name: "pokemon_center".to_string(),
            display_name: "Pokemon中心".to_string(),
            description: "治疗Pokemon的地方".to_string(),
            location_type: LocationType::PokemonCenter,
            region: "服务区域".to_string(),
            coordinates,
            bounds: LocationBounds::Rectangle {
                min: coordinates - Vec2::new(10.0, 10.0),
                max: coordinates + Vec2::new(10.0, 10.0),
            },
            services: vec![
                LocationService {
                    service_type: ServiceType::Healing,
                    npc_id: Some("nurse_joy".to_string()),
                    is_available: true,
                    requirements: vec![],
                    cost: None,
                },
                LocationService {
                    service_type: ServiceType::PokemonStorage,
                    npc_id: Some("nurse_joy".to_string()),
                    is_available: true,
                    requirements: vec![],
                    cost: None,
                },
            ],
            npcs: vec![],
            items: vec![],
            access_requirements: vec![],
            is_unlocked: true,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_manager_creation() {
        let config = LocationConfig::default();
        let manager = LocationManager::new(config);
        
        assert_eq!(manager.locations.len(), 0);
        assert_eq!(manager.active_location, None);
    }

    #[test]
    fn test_location_creation() {
        let town = LocationManager::create_town(1, "新手镇".to_string(), Vec2::new(100.0, 100.0));
        
        assert_eq!(town.id, 1);
        assert_eq!(town.location_type, LocationType::Town);
        assert_eq!(town.services.len(), 2);
        assert!(town.is_unlocked);
    }

    #[test]
    fn test_location_bounds() {
        let circle_bounds = LocationBounds::Circle {
            center: Vec2::ZERO,
            radius: 50.0,
        };
        
        assert!(circle_bounds.contains(Vec2::new(25.0, 25.0)));
        assert!(!circle_bounds.contains(Vec2::new(100.0, 100.0)));
        
        let rect_bounds = LocationBounds::Rectangle {
            min: Vec2::new(-25.0, -25.0),
            max: Vec2::new(25.0, 25.0),
        };
        
        assert!(rect_bounds.contains(Vec2::ZERO));
        assert!(!rect_bounds.contains(Vec2::new(50.0, 50.0)));
    }

    #[test]
    fn test_location_access() {
        let config = LocationConfig::default();
        let manager = LocationManager::new(config);
        
        // 无限制地点
        let open_location = LocationManager::create_town(1, "开放镇".to_string(), Vec2::ZERO);
        
        assert!(manager.can_access_location(1, 5, &[1, 2, 3], &["quest1".to_string()]));
        
        // 有等级限制的地点
        let mut restricted_location = LocationManager::create_town(2, "限制镇".to_string(), Vec2::ZERO);
        restricted_location.access_requirements.push(AccessRequirement::Level(10));
        
        // 这个测试需要实际添加地点到manager才能测试
    }

    #[test]
    fn test_connections() {
        let mut manager = LocationManager::new(LocationConfig::default());
        
        let connection = LocationConnection {
            target_location: 2,
            connection_type: ConnectionType::Walk,
            entrance_point: Vec2::new(50.0, 0.0),
            exit_point: Vec2::new(0.0, 50.0),
            requirements: vec![],
            is_bidirectional: true,
            travel_time: 30.0,
        };
        
        manager.add_connection(1, connection);
        assert!(manager.has_connection(1, 2));
        assert!(!manager.has_connection(2, 1)); // 因为没有添加反向连接
    }
}