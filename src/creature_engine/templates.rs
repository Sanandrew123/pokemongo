/*
 * Pokemon Go - Template System
 * 开发心理过程:
 * 1. 设计高度灵活的模板系统,支持JSON/TOML/YAML多种格式
 * 2. 实现模板继承和组合机制,减少重复配置
 * 3. 集成热重载功能,支持开发时实时调整
 * 4. 提供强大的模板验证和错误检测机制
 * 5. 支持模块化设计,便于管理大型生物数据库
 */

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json;
use toml;
use notify::{Watcher, RecursiveMode, Event as NotifyEvent, EventKind, RecommendedWatcher};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use super::{CreatureEngineError, CreatureEngineResult, CreatureRarity, CreatureTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub base_stats: HashMap<String, u32>,
    pub types: Vec<String>,
    pub abilities: Vec<String>,
    pub possible_traits: Vec<String>,
    pub evolution_chain: Vec<EvolutionRequirement>,
    pub spawn_data: SpawnData,
    pub visual_data: VisualData,
    pub behavioral_data: BehavioralData,
    pub inheritance: Option<TemplateInheritance>,
    pub tags: Vec<String>,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRequirement {
    pub target_id: String,
    pub level_requirement: Option<u8>,
    pub item_requirement: Option<String>,
    pub stat_requirements: HashMap<String, u32>,
    pub special_conditions: Vec<String>,
    pub time_of_day: Option<String>,
    pub location_requirement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnData {
    pub biomes: Vec<String>,
    pub rarity_weights: HashMap<CreatureRarity, f64>,
    pub level_range: (u8, u8),
    pub seasonal_modifiers: HashMap<String, f64>,
    pub weather_preferences: HashMap<String, f64>,
    pub time_preferences: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualData {
    pub model_path: String,
    pub texture_variants: Vec<String>,
    pub animation_sets: Vec<String>,
    pub size_range: (f32, f32),
    pub color_variants: Vec<ColorVariant>,
    pub particle_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorVariant {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub rarity_modifier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralData {
    pub aggression_level: f32,
    pub intelligence: f32,
    pub social_tendency: f32,
    pub activity_patterns: Vec<ActivityPattern>,
    pub diet_type: DietType,
    pub habitat_preferences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPattern {
    pub name: String,
    pub time_range: (u8, u8),
    pub activity_level: f32,
    pub preferred_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DietType {
    Herbivore,
    Carnivore,
    Omnivore,
    Energy,
    Mineral,
    Parasitic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInheritance {
    pub parent_template: String,
    pub override_fields: Vec<String>,
    pub merge_arrays: bool,
    pub inheritance_mode: InheritanceMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InheritanceMode {
    Override,
    Merge,
    Extend,
    Custom(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCollection {
    pub name: String,
    pub version: String,
    pub description: String,
    pub templates: Vec<CreatureTemplate>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TemplateManager {
    templates: Arc<RwLock<HashMap<String, CreatureTemplate>>>,
    collections: Arc<RwLock<HashMap<String, TemplateCollection>>>,
    template_paths: Vec<PathBuf>,
    file_watcher: Option<notify::RecommendedWatcher>,
    reload_receiver: Option<Receiver<NotifyEvent>>,
    inheritance_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    cache: Arc<RwLock<HashMap<String, CachedTemplate>>>,
}

#[derive(Debug, Clone)]
struct CachedTemplate {
    template: CreatureTemplate,
    file_hash: u64,
    last_modified: std::time::SystemTime,
}

impl TemplateManager {
    pub fn new() -> CreatureEngineResult<Self> {
        let templates = Arc::new(RwLock::new(HashMap::new()));
        let collections = Arc::new(RwLock::new(HashMap::new()));
        let inheritance_graph = Arc::new(RwLock::new(HashMap::new()));
        let cache = Arc::new(RwLock::new(HashMap::new()));

        let mut manager = Self {
            templates,
            collections,
            template_paths: Vec::new(),
            file_watcher: None,
            reload_receiver: None,
            inheritance_graph,
            cache,
        };

        manager.setup_default_paths()?;
        manager.load_all_templates()?;
        manager.setup_file_watcher()?;

        Ok(manager)
    }

    pub fn add_template_path<P: AsRef<Path>>(&mut self, path: P) -> CreatureEngineResult<()> {
        self.template_paths.push(path.as_ref().to_path_buf());
        self.load_templates_from_path(&path)?;
        Ok(())
    }

    pub fn get_template(&self, id: &str) -> CreatureEngineResult<CreatureTemplate> {
        let templates = self.templates.read().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        if let Some(template) = templates.get(id) {
            Ok(self.resolve_inheritance(template.clone())?)
        } else {
            Err(CreatureEngineError::InvalidTemplate(format!("Template '{}' not found", id)))
        }
    }

    pub fn get_all_templates(&self) -> CreatureEngineResult<Vec<CreatureTemplate>> {
        let templates = self.templates.read().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        let mut result = Vec::new();
        for template in templates.values() {
            result.push(self.resolve_inheritance(template.clone())?);
        }
        
        Ok(result)
    }

    pub fn get_templates_by_category(&self, category: &str) -> CreatureEngineResult<Vec<CreatureTemplate>> {
        let templates = self.templates.read().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        let mut result = Vec::new();
        for template in templates.values() {
            if template.category == category {
                result.push(self.resolve_inheritance(template.clone())?);
            }
        }
        
        Ok(result)
    }

    pub fn get_templates_by_tags(&self, tags: &[String]) -> CreatureEngineResult<Vec<CreatureTemplate>> {
        let templates = self.templates.read().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        let mut result = Vec::new();
        for template in templates.values() {
            if tags.iter().any(|tag| template.tags.contains(tag)) {
                result.push(self.resolve_inheritance(template.clone())?);
            }
        }
        
        Ok(result)
    }

    pub fn add_template(&mut self, template: CreatureTemplate) -> CreatureEngineResult<()> {
        self.validate_template(&template)?;
        
        let mut templates = self.templates.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        templates.insert(template.id.clone(), template.clone());
        drop(templates);
        
        self.update_inheritance_graph(&template)?;
        Ok(())
    }

    pub fn remove_template(&mut self, id: &str) -> CreatureEngineResult<()> {
        let mut templates = self.templates.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        templates.remove(id);
        drop(templates);
        
        self.cleanup_inheritance_graph(id)?;
        Ok(())
    }

    pub fn reload_template(&mut self, id: &str) -> CreatureEngineResult<()> {
        // Find the template file and reload it
        for path in &self.template_paths.clone() {
            if let Ok(template) = self.load_template_from_file(path, id) {
                self.add_template(template)?;
                return Ok(());
            }
        }
        
        Err(CreatureEngineError::ResourceError(format!("Template '{}' not found in any path", id)))
    }

    pub fn reload_all_templates(&mut self) -> CreatureEngineResult<()> {
        let mut templates = self.templates.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        templates.clear();
        drop(templates);
        
        let mut inheritance_graph = self.inheritance_graph.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire inheritance graph lock".to_string())
        })?;
        
        inheritance_graph.clear();
        drop(inheritance_graph);
        
        self.load_all_templates()
    }

    pub fn validate_all_templates(&self) -> CreatureEngineResult<Vec<ValidationError>> {
        let templates = self.templates.read().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
        })?;
        
        let mut errors = Vec::new();
        for (id, template) in templates.iter() {
            if let Err(err) = self.validate_template(template) {
                errors.push(ValidationError {
                    template_id: id.clone(),
                    error_type: ValidationErrorType::Invalid,
                    message: err.to_string(),
                });
            }
        }
        
        Ok(errors)
    }

    pub fn template_count(&self) -> usize {
        self.templates.read().map(|t| t.len()).unwrap_or(0)
    }

    pub fn check_for_updates(&mut self) -> CreatureEngineResult<Vec<String>> {
        let mut updated_templates = Vec::new();
        
        if let Some(receiver) = &self.reload_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    Ok(event) if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) => {
                        let path = event.paths.get(0).unwrap();
                        if let Some(template_id) = self.extract_template_id_from_path(&path) {
                            if self.reload_template(&template_id).is_ok() {
                                updated_templates.push(template_id);
                            }
                        }
                    }
                    Ok(event) if matches!(event.kind, EventKind::Remove(_)) => {
                        let path = event.paths.get(0).unwrap();
                        if let Some(template_id) = self.extract_template_id_from_path(&path) {
                            let _ = self.remove_template(&template_id);
                            updated_templates.push(template_id);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(updated_templates)
    }

    fn setup_default_paths(&mut self) -> CreatureEngineResult<()> {
        let default_paths = [
            "data/templates/",
            "assets/creatures/",
            "config/templates/",
        ];
        
        for path in &default_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                self.template_paths.push(path_buf);
            }
        }
        
        Ok(())
    }

    fn setup_file_watcher(&mut self) -> CreatureEngineResult<()> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }, notify::Config::default()).map_err(|e| {
            CreatureEngineError::ConfigError(format!("Failed to create file watcher: {}", e))
        })?;

        for path in &self.template_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive).map_err(|e| {
                    CreatureEngineError::ConfigError(format!("Failed to watch path {:?}: {}", path, e))
                })?;
            }
        }

        self.file_watcher = Some(watcher);
        self.reload_receiver = Some(rx);
        Ok(())
    }

    fn load_all_templates(&mut self) -> CreatureEngineResult<()> {
        for path in &self.template_paths.clone() {
            self.load_templates_from_path(path)?;
        }
        Ok(())
    }

    fn load_templates_from_path<P: AsRef<Path>>(&mut self, path: P) -> CreatureEngineResult<()> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(());
        }

        if path.is_file() {
            if let Some(extension) = path.extension() {
                match extension.to_str() {
                    Some("json") | Some("toml") | Some("yaml") | Some("yml") => {
                        let template = self.load_single_template_file(path)?;
                        self.add_template(template)?;
                    }
                    _ => {}
                }
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path).map_err(|e| {
                CreatureEngineError::ResourceError(format!("Failed to read directory {:?}: {}", path, e))
            })? {
                let entry = entry.map_err(|e| {
                    CreatureEngineError::ResourceError(format!("Failed to read directory entry: {}", e))
                })?;
                
                self.load_templates_from_path(entry.path())?;
            }
        }

        Ok(())
    }

    fn load_single_template_file<P: AsRef<Path>>(&self, path: P) -> CreatureEngineResult<CreatureTemplate> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| {
            CreatureEngineError::ResourceError(format!("Failed to read template file {:?}: {}", path, e))
        })?;

        let template = match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                serde_json::from_str::<CreatureTemplate>(&content).map_err(|e| {
                    CreatureEngineError::InvalidTemplate(format!("Invalid JSON in {:?}: {}", path, e))
                })?
            }
            Some("toml") => {
                toml::from_str::<CreatureTemplate>(&content).map_err(|e| {
                    CreatureEngineError::InvalidTemplate(format!("Invalid TOML in {:?}: {}", path, e))
                })?
            }
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str::<CreatureTemplate>(&content).map_err(|e| {
                    CreatureEngineError::InvalidTemplate(format!("Invalid YAML in {:?}: {}", path, e))
                })?
            }
            _ => {
                return Err(CreatureEngineError::InvalidTemplate(
                    format!("Unsupported template file format: {:?}", path)
                ));
            }
        };

        Ok(template)
    }

    fn load_template_from_file<P: AsRef<Path>>(&self, path: P, id: &str) -> CreatureEngineResult<CreatureTemplate> {
        let path = path.as_ref();
        let template_path = path.join(format!("{}.json", id))
            .or_else(|| path.join(format!("{}.toml", id)))
            .or_else(|| path.join(format!("{}.yaml", id)))
            .or_else(|| path.join(format!("{}.yml", id)));

        if let Some(file_path) = template_path {
            if file_path.exists() {
                return self.load_single_template_file(file_path);
            }
        }

        Err(CreatureEngineError::ResourceError(format!("Template file for '{}' not found", id)))
    }

    fn resolve_inheritance(&self, mut template: CreatureTemplate) -> CreatureEngineResult<CreatureTemplate> {
        if let Some(inheritance) = &template.inheritance.clone() {
            let parent_id = &inheritance.parent_template;
            
            let templates = self.templates.read().map_err(|_| {
                CreatureEngineError::ConfigError("Failed to acquire template lock".to_string())
            })?;
            
            if let Some(parent_template) = templates.get(parent_id) {
                let resolved_parent = self.resolve_inheritance(parent_template.clone())?;
                template = self.merge_templates(resolved_parent, template, inheritance)?;
            } else {
                return Err(CreatureEngineError::InvalidTemplate(
                    format!("Parent template '{}' not found", parent_id)
                ));
            }
        }
        
        Ok(template)
    }

    fn merge_templates(
        &self,
        parent: CreatureTemplate,
        mut child: CreatureTemplate,
        inheritance: &TemplateInheritance
    ) -> CreatureEngineResult<CreatureTemplate> {
        match inheritance.inheritance_mode {
            InheritanceMode::Override => {
                for field in &inheritance.override_fields {
                    match field.as_str() {
                        "base_stats" => child.base_stats = parent.base_stats.clone(),
                        "types" => child.types = parent.types.clone(),
                        "abilities" => child.abilities = parent.abilities.clone(),
                        _ => {}
                    }
                }
            }
            InheritanceMode::Merge => {
                for (stat, value) in parent.base_stats {
                    child.base_stats.entry(stat).or_insert(value);
                }
                
                if inheritance.merge_arrays {
                    child.types.extend(parent.types);
                    child.types.sort();
                    child.types.dedup();
                    
                    child.abilities.extend(parent.abilities);
                    child.abilities.sort();
                    child.abilities.dedup();
                }
            }
            InheritanceMode::Extend => {
                child.base_stats.extend(parent.base_stats);
                child.types.extend(parent.types);
                child.abilities.extend(parent.abilities);
            }
            InheritanceMode::Custom(_) => {
                // Custom inheritance rules would be implemented here
            }
        }
        
        Ok(child)
    }

    fn validate_template(&self, template: &CreatureTemplate) -> CreatureEngineResult<()> {
        if template.id.is_empty() {
            return Err(CreatureEngineError::ValidationError("Template ID cannot be empty".to_string()));
        }

        if template.name.is_empty() {
            return Err(CreatureEngineError::ValidationError("Template name cannot be empty".to_string()));
        }

        if template.base_stats.is_empty() {
            return Err(CreatureEngineError::ValidationError("Template must have base stats".to_string()));
        }

        for (stat_name, stat_value) in &template.base_stats {
            if *stat_value == 0 || *stat_value > 255 {
                return Err(CreatureEngineError::ValidationError(
                    format!("Invalid stat value for {}: {}", stat_name, stat_value)
                ));
            }
        }

        Ok(())
    }

    fn update_inheritance_graph(&mut self, template: &CreatureTemplate) -> CreatureEngineResult<()> {
        let mut graph = self.inheritance_graph.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire inheritance graph lock".to_string())
        })?;

        if let Some(inheritance) = &template.inheritance {
            graph.entry(inheritance.parent_template.clone())
                .or_insert_with(Vec::new)
                .push(template.id.clone());
        }

        Ok(())
    }

    fn cleanup_inheritance_graph(&mut self, id: &str) -> CreatureEngineResult<()> {
        let mut graph = self.inheritance_graph.write().map_err(|_| {
            CreatureEngineError::ConfigError("Failed to acquire inheritance graph lock".to_string())
        })?;

        graph.remove(id);
        
        for children in graph.values_mut() {
            children.retain(|child_id| child_id != id);
        }

        Ok(())
    }

    fn extract_template_id_from_path(&self, path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub template_id: String,
    pub error_type: ValidationErrorType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    Invalid,
    MissingDependency,
    CircularInheritance,
    DuplicateId,
}

impl Default for SpawnData {
    fn default() -> Self {
        Self {
            biomes: vec!["grassland".to_string()],
            rarity_weights: {
                let mut weights = HashMap::new();
                weights.insert(CreatureRarity::Common, 0.5);
                weights.insert(CreatureRarity::Uncommon, 0.3);
                weights.insert(CreatureRarity::Rare, 0.15);
                weights.insert(CreatureRarity::Epic, 0.04);
                weights.insert(CreatureRarity::Legendary, 0.009);
                weights.insert(CreatureRarity::Mythical, 0.001);
                weights
            },
            level_range: (1, 30),
            seasonal_modifiers: HashMap::new(),
            weather_preferences: HashMap::new(),
            time_preferences: HashMap::new(),
        }
    }
}

impl Default for VisualData {
    fn default() -> Self {
        Self {
            model_path: "models/default.obj".to_string(),
            texture_variants: vec!["default.png".to_string()],
            animation_sets: vec!["idle".to_string(), "walk".to_string()],
            size_range: (1.0, 1.0),
            color_variants: Vec::new(),
            particle_effects: Vec::new(),
        }
    }
}

impl Default for BehavioralData {
    fn default() -> Self {
        Self {
            aggression_level: 0.5,
            intelligence: 0.5,
            social_tendency: 0.5,
            activity_patterns: Vec::new(),
            diet_type: DietType::Omnivore,
            habitat_preferences: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_template_validation() {
        let manager = TemplateManager::new().unwrap();
        
        let invalid_template = CreatureTemplate {
            id: "".to_string(),
            name: "Test".to_string(),
            base_stats: HashMap::new(),
            // ... other default fields
            description: "Test description".to_string(),
            category: "test".to_string(),
            types: Vec::new(),
            abilities: Vec::new(),
            possible_traits: Vec::new(),
            evolution_chain: Vec::new(),
            spawn_data: SpawnData::default(),
            visual_data: VisualData::default(),
            behavioral_data: BehavioralData::default(),
            inheritance: None,
            tags: Vec::new(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        assert!(manager.validate_template(&invalid_template).is_err());
    }

    #[test]
    fn test_inheritance_resolution() {
        // Would need setup of parent and child templates for full testing
    }
}