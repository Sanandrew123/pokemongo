/*
* 开发心理过程：
* 1. 创建Pokemon数据加载系统，从多种数据源加载Pokemon信息
* 2. 支持JSON、TOML、数据库等多种数据格式
* 3. 实现缓存机制，提高数据访问性能
* 4. 提供数据验证和错误处理
* 5. 支持热重载和动态数据更新
* 6. 集成资源管理，确保数据一致性
* 7. 提供异步加载能力，避免阻塞游戏
*/

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};
use bevy::prelude::*;
use tokio::{fs, sync::RwLock};
use tracing::{info, warn, error, debug};

use crate::{
    core::error::{GameError, GameResult},
    pokemon::{
        species::{PokemonSpecies, SpeciesId},
        moves::{Move, MoveId},
        types::PokemonType,
        individual::IndividualPokemon,
    },
    data::{database::GameDatabase, cache::CacheManager},
    utils::random::RandomGenerator,
};

#[derive(Resource)]
pub struct PokemonDataLoader {
    species_cache: Arc<RwLock<HashMap<SpeciesId, Arc<PokemonSpecies>>>>,
    moves_cache: Arc<RwLock<HashMap<MoveId, Arc<Move>>>>,
    type_chart: Arc<RwLock<Option<TypeEffectivenessChart>>>,
    data_path: PathBuf,
    last_reload: SystemTime,
    database: Option<Arc<GameDatabase>>,
    cache_manager: Arc<CacheManager>,
    loading_tasks: Arc<RwLock<HashMap<String, LoadingTask>>>,
}

#[derive(Debug, Clone)]
struct LoadingTask {
    pub task_type: LoadingTaskType,
    pub progress: f32,
    pub started: SystemTime,
    pub estimated_completion: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadingTaskType {
    SpeciesData,
    MoveData,
    TypeChart,
    IndividualPokemon,
    ValidationCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEffectivenessChart {
    pub effectiveness: HashMap<(PokemonType, PokemonType), f32>,
    pub version: String,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PokemonDataManifest {
    pub version: String,
    pub species_count: u32,
    pub moves_count: u32,
    pub data_files: Vec<DataFileInfo>,
    pub checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataFileInfo {
    pub filename: String,
    pub file_type: DataFileType,
    pub priority: u8,
    pub size_bytes: u64,
    pub compressed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum DataFileType {
    Species,
    Moves,
    Types,
    Abilities,
    Items,
    Locations,
}

#[derive(Event)]
pub struct DataLoadedEvent {
    pub data_type: LoadingTaskType,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Event)]
pub struct DataReloadRequestEvent;

impl PokemonDataLoader {
    pub fn new(data_path: PathBuf) -> GameResult<Self> {
        info!("初始化Pokemon数据加载器: {:?}", data_path);
        
        if !data_path.exists() {
            return Err(GameError::DataError(format!("数据目录不存在: {:?}", data_path)));
        }

        Ok(Self {
            species_cache: Arc::new(RwLock::new(HashMap::new())),
            moves_cache: Arc::new(RwLock::new(HashMap::new())),
            type_chart: Arc::new(RwLock::new(None)),
            data_path,
            last_reload: SystemTime::now(),
            database: None,
            cache_manager: Arc::new(CacheManager::new()?),
            loading_tasks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&mut self) -> GameResult<()> {
        info!("开始初始化Pokemon数据");

        // 加载数据清单
        let manifest = self.load_manifest().await?;
        info!("数据清单加载完成 - 版本: {}, 种族数: {}, 招式数: {}", 
            manifest.version, manifest.species_count, manifest.moves_count);

        // 按优先级排序数据文件
        let mut files = manifest.data_files;
        files.sort_by_key(|f| f.priority);

        // 并发加载核心数据
        let load_tasks = vec![
            self.load_species_data(),
            self.load_moves_data(),
            self.load_type_effectiveness_chart(),
        ];

        let results = futures::future::join_all(load_tasks).await;
        
        // 检查加载结果
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                error!("数据加载失败 (任务 {}): {}", i, e);
                return Err(e);
            }
        }

        // 验证数据完整性
        self.validate_data_integrity().await?;

        info!("Pokemon数据初始化完成");
        Ok(())
    }

    async fn load_manifest(&self) -> GameResult<PokemonDataManifest> {
        let manifest_path = self.data_path.join("manifest.json");
        
        if !manifest_path.exists() {
            warn!("未找到数据清单，创建默认清单");
            return Ok(self.create_default_manifest().await?);
        }

        let content = fs::read_to_string(&manifest_path).await
            .map_err(|e| GameError::DataError(format!("读取清单失败: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| GameError::DataError(format!("解析清单失败: {}", e)))
    }

    async fn create_default_manifest(&self) -> GameResult<PokemonDataManifest> {
        let manifest = PokemonDataManifest {
            version: "1.0.0".to_string(),
            species_count: 151, // 第一代
            moves_count: 165,
            data_files: vec![
                DataFileInfo {
                    filename: "species.json".to_string(),
                    file_type: DataFileType::Species,
                    priority: 1,
                    size_bytes: 1024 * 1024, // 预估1MB
                    compressed: false,
                },
                DataFileInfo {
                    filename: "moves.json".to_string(),
                    file_type: DataFileType::Moves,
                    priority: 2,
                    size_bytes: 512 * 1024, // 预估512KB
                    compressed: false,
                },
                DataFileInfo {
                    filename: "type_chart.json".to_string(),
                    file_type: DataFileType::Types,
                    priority: 1,
                    size_bytes: 4 * 1024, // 预估4KB
                    compressed: false,
                },
            ],
            checksums: HashMap::new(),
        };

        // 保存默认清单
        let manifest_path = self.data_path.join("manifest.json");
        let content = serde_json::to_string_pretty(&manifest)
            .map_err(|e| GameError::DataError(format!("序列化清单失败: {}", e)))?;

        fs::write(&manifest_path, content).await
            .map_err(|e| GameError::DataError(format!("保存清单失败: {}", e)))?;

        Ok(manifest)
    }

    async fn load_species_data(&self) -> GameResult<()> {
        self.start_loading_task("species_data", LoadingTaskType::SpeciesData).await;

        let species_path = self.data_path.join("species.json");
        
        // 尝试从缓存加载
        if let Ok(cached_data) = self.cache_manager.get::<Vec<PokemonSpecies>>("species_all").await {
            let mut cache = self.species_cache.write().await;
            for species in cached_data {
                cache.insert(species.id, Arc::new(species));
            }
            self.complete_loading_task("species_data", true, None).await;
            return Ok(());
        }

        // 从文件加载
        let species_list = if species_path.exists() {
            let content = fs::read_to_string(&species_path).await
                .map_err(|e| GameError::DataError(format!("读取种族数据失败: {}", e)))?;

            serde_json::from_str::<Vec<PokemonSpecies>>(&content)
                .map_err(|e| GameError::DataError(format!("解析种族数据失败: {}", e)))?
        } else {
            warn!("种族数据文件不存在，使用默认数据");
            self.create_default_species_data().await?
        };

        // 更新缓存
        let mut cache = self.species_cache.write().await;
        for species in &species_list {
            cache.insert(species.id, Arc::new(species.clone()));
        }

        // 保存到缓存管理器
        self.cache_manager.set("species_all", &species_list, Duration::from_secs(3600)).await?;

        self.complete_loading_task("species_data", true, None).await;
        info!("种族数据加载完成: {} 个物种", species_list.len());
        Ok(())
    }

    async fn create_default_species_data(&self) -> GameResult<Vec<PokemonSpecies>> {
        // 创建一些默认的Pokemon种族数据
        let default_species = vec![
            PokemonSpecies {
                id: 1,
                name: "Bulbasaur".to_string(),
                types: vec![PokemonType::Grass, PokemonType::Poison],
                base_stats: crate::pokemon::stats::StatBlock {
                    hp: 45,
                    attack: 49,
                    defense: 49,
                    special_attack: 65,
                    special_defense: 65,
                    speed: 45,
                },
                height: 0.7,
                weight: 6.9,
                description: "Seed Pokemon".to_string(),
                ..Default::default()
            },
            PokemonSpecies {
                id: 4,
                name: "Charmander".to_string(),
                types: vec![PokemonType::Fire],
                base_stats: crate::pokemon::stats::StatBlock {
                    hp: 39,
                    attack: 52,
                    defense: 43,
                    special_attack: 60,
                    special_defense: 50,
                    speed: 65,
                },
                height: 0.6,
                weight: 8.5,
                description: "Lizard Pokemon".to_string(),
                ..Default::default()
            },
            PokemonSpecies {
                id: 7,
                name: "Squirtle".to_string(),
                types: vec![PokemonType::Water],
                base_stats: crate::pokemon::stats::StatBlock {
                    hp: 44,
                    attack: 48,
                    defense: 65,
                    special_attack: 50,
                    special_defense: 64,
                    speed: 43,
                },
                height: 0.5,
                weight: 9.0,
                description: "Tiny Turtle Pokemon".to_string(),
                ..Default::default()
            },
        ];

        // 保存默认数据到文件
        let species_path = self.data_path.join("species.json");
        let content = serde_json::to_string_pretty(&default_species)
            .map_err(|e| GameError::DataError(format!("序列化种族数据失败: {}", e)))?;

        fs::write(&species_path, content).await
            .map_err(|e| GameError::DataError(format!("保存种族数据失败: {}", e)))?;

        Ok(default_species)
    }

    async fn load_moves_data(&self) -> GameResult<()> {
        self.start_loading_task("moves_data", LoadingTaskType::MoveData).await;

        let moves_path = self.data_path.join("moves.json");
        
        // 尝试从缓存加载
        if let Ok(cached_data) = self.cache_manager.get::<Vec<Move>>("moves_all").await {
            let mut cache = self.moves_cache.write().await;
            for move_data in cached_data {
                cache.insert(move_data.id, Arc::new(move_data));
            }
            self.complete_loading_task("moves_data", true, None).await;
            return Ok(());
        }

        // 从文件加载
        let moves_list = if moves_path.exists() {
            let content = fs::read_to_string(&moves_path).await
                .map_err(|e| GameError::DataError(format!("读取招式数据失败: {}", e)))?;

            serde_json::from_str::<Vec<Move>>(&content)
                .map_err(|e| GameError::DataError(format!("解析招式数据失败: {}", e)))?
        } else {
            warn!("招式数据文件不存在，使用默认数据");
            self.create_default_moves_data().await?
        };

        // 更新缓存
        let mut cache = self.moves_cache.write().await;
        for move_data in &moves_list {
            cache.insert(move_data.id, Arc::new(move_data.clone()));
        }

        // 保存到缓存管理器
        self.cache_manager.set("moves_all", &moves_list, Duration::from_secs(3600)).await?;

        self.complete_loading_task("moves_data", true, None).await;
        info!("招式数据加载完成: {} 个招式", moves_list.len());
        Ok(())
    }

    async fn create_default_moves_data(&self) -> GameResult<Vec<Move>> {
        let default_moves = vec![
            Move {
                id: 1,
                name: "Tackle".to_string(),
                move_type: PokemonType::Normal,
                power: Some(40),
                accuracy: 100,
                pp: 35,
                description: "A physical attack in which the user charges and slams into the target.".to_string(),
                ..Default::default()
            },
            Move {
                id: 2,
                name: "Scratch".to_string(),
                move_type: PokemonType::Normal,
                power: Some(40),
                accuracy: 100,
                pp: 35,
                description: "Hard, pointed, sharp claws rake the target to inflict damage.".to_string(),
                ..Default::default()
            },
        ];

        // 保存默认数据到文件
        let moves_path = self.data_path.join("moves.json");
        let content = serde_json::to_string_pretty(&default_moves)
            .map_err(|e| GameError::DataError(format!("序列化招式数据失败: {}", e)))?;

        fs::write(&moves_path, content).await
            .map_err(|e| GameError::DataError(format!("保存招式数据失败: {}", e)))?;

        Ok(default_moves)
    }

    async fn load_type_effectiveness_chart(&self) -> GameResult<()> {
        self.start_loading_task("type_chart", LoadingTaskType::TypeChart).await;

        let chart_path = self.data_path.join("type_chart.json");
        
        let chart = if chart_path.exists() {
            let content = fs::read_to_string(&chart_path).await
                .map_err(|e| GameError::DataError(format!("读取属性相克表失败: {}", e)))?;

            serde_json::from_str::<TypeEffectivenessChart>(&content)
                .map_err(|e| GameError::DataError(format!("解析属性相克表失败: {}", e)))?
        } else {
            warn!("属性相克表不存在，创建默认相克表");
            self.create_default_type_chart().await?
        };

        *self.type_chart.write().await = Some(chart);

        self.complete_loading_task("type_chart", true, None).await;
        info!("属性相克表加载完成");
        Ok(())
    }

    async fn create_default_type_chart(&self) -> GameResult<TypeEffectivenessChart> {
        let mut effectiveness = HashMap::new();
        
        // 添加一些基本的属性相克关系
        // Fire vs Grass = 2.0 (Super effective)
        effectiveness.insert((PokemonType::Fire, PokemonType::Grass), 2.0);
        // Water vs Fire = 2.0 (Super effective)
        effectiveness.insert((PokemonType::Water, PokemonType::Fire), 2.0);
        // Grass vs Water = 2.0 (Super effective)
        effectiveness.insert((PokemonType::Grass, PokemonType::Water), 2.0);
        
        // Fire vs Water = 0.5 (Not very effective)
        effectiveness.insert((PokemonType::Fire, PokemonType::Water), 0.5);
        // Water vs Grass = 0.5 (Not very effective)
        effectiveness.insert((PokemonType::Water, PokemonType::Grass), 0.5);
        // Grass vs Fire = 0.5 (Not very effective)
        effectiveness.insert((PokemonType::Grass, PokemonType::Fire), 0.5);

        let chart = TypeEffectivenessChart {
            effectiveness,
            version: "1.0.0".to_string(),
            last_updated: SystemTime::now(),
        };

        // 保存到文件
        let chart_path = self.data_path.join("type_chart.json");
        let content = serde_json::to_string_pretty(&chart)
            .map_err(|e| GameError::DataError(format!("序列化属性相克表失败: {}", e)))?;

        fs::write(&chart_path, content).await
            .map_err(|e| GameError::DataError(format!("保存属性相克表失败: {}", e)))?;

        Ok(chart)
    }

    async fn validate_data_integrity(&self) -> GameResult<()> {
        self.start_loading_task("validation", LoadingTaskType::ValidationCheck).await;

        let species_cache = self.species_cache.read().await;
        let moves_cache = self.moves_cache.read().await;

        // 验证种族数据
        for (id, species) in species_cache.iter() {
            if species.id != *id {
                let error = format!("种族ID不匹配: 缓存ID {} vs 数据ID {}", id, species.id);
                self.complete_loading_task("validation", false, Some(error.clone())).await;
                return Err(GameError::DataError(error));
            }

            // 验证招式引用
            for (level, move_ids) in &species.level_up_moves {
                for &move_id in move_ids {
                    if !moves_cache.contains_key(&move_id) {
                        warn!("种族 {} 等级 {} 引用了不存在的招式 {}", species.id, level, move_id);
                    }
                }
            }
        }

        // 验证招式数据
        for (id, move_data) in moves_cache.iter() {
            if move_data.id != *id {
                let error = format!("招式ID不匹配: 缓存ID {} vs 数据ID {}", id, move_data.id);
                self.complete_loading_task("validation", false, Some(error.clone())).await;
                return Err(GameError::DataError(error));
            }
        }

        self.complete_loading_task("validation", true, None).await;
        info!("数据完整性验证通过");
        Ok(())
    }

    async fn start_loading_task(&self, task_id: &str, task_type: LoadingTaskType) {
        let task = LoadingTask {
            task_type,
            progress: 0.0,
            started: SystemTime::now(),
            estimated_completion: None,
        };

        self.loading_tasks.write().await.insert(task_id.to_string(), task);
        debug!("开始加载任务: {} ({:?})", task_id, task_type);
    }

    async fn complete_loading_task(&self, task_id: &str, success: bool, error: Option<String>) {
        self.loading_tasks.write().await.remove(task_id);
        
        if success {
            debug!("加载任务完成: {}", task_id);
        } else {
            error!("加载任务失败: {} - {:?}", task_id, error);
        }
    }

    // 公共API
    pub async fn get_species(&self, id: SpeciesId) -> Option<Arc<PokemonSpecies>> {
        self.species_cache.read().await.get(&id).cloned()
    }

    pub async fn get_move(&self, id: MoveId) -> Option<Arc<Move>> {
        self.moves_cache.read().await.get(&id).cloned()
    }

    pub async fn get_type_effectiveness(&self, attacking: PokemonType, defending: PokemonType) -> f32 {
        if let Some(chart) = self.type_chart.read().await.as_ref() {
            chart.effectiveness.get(&(attacking, defending)).copied().unwrap_or(1.0)
        } else {
            1.0
        }
    }

    pub async fn get_all_species(&self) -> Vec<Arc<PokemonSpecies>> {
        self.species_cache.read().await.values().cloned().collect()
    }

    pub async fn get_species_by_name(&self, name: &str) -> Option<Arc<PokemonSpecies>> {
        let cache = self.species_cache.read().await;
        cache.values().find(|species| species.name.eq_ignore_ascii_case(name)).cloned()
    }

    pub async fn create_random_pokemon(&self, level: u8, rng: &mut RandomGenerator) -> GameResult<IndividualPokemon> {
        let species_list = self.get_all_species().await;
        if species_list.is_empty() {
            return Err(GameError::DataError("没有可用的Pokemon种族数据".to_string()));
        }

        let species = &species_list[rng.range(0, species_list.len())];
        IndividualPokemon::new(species, level, rng)
    }

    pub async fn reload_data(&mut self) -> GameResult<()> {
        info!("重新加载Pokemon数据");
        
        // 清空缓存
        self.species_cache.write().await.clear();
        self.moves_cache.write().await.clear();
        *self.type_chart.write().await = None;

        // 重新初始化
        self.initialize().await?;
        self.last_reload = SystemTime::now();

        info!("数据重新加载完成");
        Ok(())
    }

    pub async fn get_loading_progress(&self) -> HashMap<String, f32> {
        let tasks = self.loading_tasks.read().await;
        tasks.iter().map(|(id, task)| (id.clone(), task.progress)).collect()
    }
}

// Bevy插件
pub struct PokemonLoaderPlugin;

impl Plugin for PokemonLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DataLoadedEvent>()
           .add_event::<DataReloadRequestEvent>()
           .add_systems(Update, (
               handle_data_reload_requests,
               monitor_loading_progress,
           ));
    }
}

fn handle_data_reload_requests(
    mut reload_events: EventReader<DataReloadRequestEvent>,
    mut loader: ResMut<PokemonDataLoader>,
) {
    for _event in reload_events.read() {
        info!("收到数据重载请求");
        // 在实际实现中，这里会启动异步任务
    }
}

fn monitor_loading_progress(
    loader: Res<PokemonDataLoader>,
    mut last_check: Local<SystemTime>,
    time: Res<Time>,
) {
    let now = SystemTime::now();
    if now.duration_since(*last_check).unwrap_or_default() < Duration::from_secs(1) {
        return;
    }

    *last_check = now;
    
    // 在实际实现中，这里会检查加载进度并发送事件
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_loader_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = PokemonDataLoader::new(temp_dir.path().to_path_buf()).unwrap();
        
        let result = loader.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_species_loading() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = PokemonDataLoader::new(temp_dir.path().to_path_buf()).unwrap();
        
        loader.initialize().await.unwrap();
        
        let species = loader.get_all_species().await;
        assert!(!species.is_empty());
    }

    #[tokio::test]
    async fn test_random_pokemon_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = PokemonDataLoader::new(temp_dir.path().to_path_buf()).unwrap();
        loader.initialize().await.unwrap();
        
        let mut rng = RandomGenerator::new();
        let pokemon = loader.create_random_pokemon(10, &mut rng).await;
        
        assert!(pokemon.is_ok());
        let pokemon = pokemon.unwrap();
        assert_eq!(pokemon.level, 10);
    }
}