/*
 * Pokemon Go - Evolution Tree System
 * 开发心理过程:
 * 1. 设计复杂的进化树系统,支持多分支、条件进化和特殊进化
 * 2. 实现图论算法管理进化关系,支持循环检测和路径查找
 * 3. 集成多种进化条件:等级、道具、时间、地点、统计值等
 * 4. 提供进化预测和可视化功能,便于玩家理解进化路径
 * 5. 支持动态进化条件和事件驱动的特殊进化
 */

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};
use petgraph::{Graph, Directed, graph::NodeIndex, algo::dijkstra};
use chrono::{DateTime, Utc, TimeZone, Local, Timelike};

use super::{CreatureEngineError, CreatureEngineResult, GeneratedCreature, EvolutionRequirement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRequirements {
    pub level_requirements: HashMap<String, u8>,
    pub item_requirements: HashMap<String, Vec<String>>,
    pub stat_thresholds: HashMap<String, HashMap<String, u32>>,
    pub friendship_levels: HashMap<String, u32>,
    pub location_requirements: HashMap<String, Vec<String>>,
    pub time_windows: HashMap<String, Vec<TimeWindow>>,
    pub weather_conditions: HashMap<String, Vec<String>>,
    pub special_conditions: HashMap<String, Vec<SpecialCondition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_hour: u8,
    pub end_hour: u8,
    pub days_of_week: Vec<u8>,
    pub seasonal_modifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialCondition {
    pub condition_type: ConditionType,
    pub parameters: HashMap<String, String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    BattleVictories(u32),
    StepsWalked(u32),
    ItemsUsed(Vec<String>),
    TrainerLevel(u8),
    CompanionDuration(u32),
    GymBadges(Vec<String>),
    QuestCompletion(String),
    SocialInteraction(String),
    CustomEvent(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPath {
    pub creature_id: String,
    pub target_id: String,
    pub requirements: EvolutionRequirement,
    pub cost: EvolutionCost,
    pub success_rate: f64,
    pub time_estimate: Option<u32>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCost {
    pub energy_cost: u32,
    pub items_consumed: Vec<ItemRequirement>,
    pub stat_changes: HashMap<String, i32>,
    pub trait_changes: Vec<TraitChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRequirement {
    pub item_id: String,
    pub quantity: u32,
    pub quality_threshold: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitChange {
    pub action: TraitAction,
    pub trait_id: String,
    pub parameters: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitAction {
    Add,
    Remove,
    Modify,
    Replace(String),
}

#[derive(Debug)]
pub struct EvolutionTree {
    graph: Graph<EvolutionNode, EvolutionEdge, Directed>,
    node_map: HashMap<String, NodeIndex>,
    requirements: EvolutionRequirements,
    evolution_history: Vec<EvolutionRecord>,
    cached_paths: HashMap<String, Vec<EvolutionPath>>,
}

#[derive(Debug, Clone)]
struct EvolutionNode {
    creature_id: String,
    species_name: String,
    stage: u8,
    base_requirements: Vec<EvolutionRequirement>,
}

#[derive(Debug, Clone)]
struct EvolutionEdge {
    requirements: EvolutionRequirement,
    success_rate: f64,
    cost: EvolutionCost,
    unlock_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRecord {
    pub creature_id: String,
    pub original_form: String,
    pub evolved_form: String,
    pub evolution_time: DateTime<Utc>,
    pub conditions_met: Vec<String>,
    pub success: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionForecast {
    pub creature_id: String,
    pub available_evolutions: Vec<EvolutionOption>,
    pub blocked_evolutions: Vec<BlockedEvolution>,
    pub optimal_path: Option<EvolutionPath>,
    pub time_sensitive_opportunities: Vec<TimeSensitiveEvolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionOption {
    pub target_id: String,
    pub target_name: String,
    pub requirements_met: Vec<String>,
    pub missing_requirements: Vec<MissingRequirement>,
    pub estimated_completion_time: Option<u32>,
    pub difficulty_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedEvolution {
    pub target_id: String,
    pub blocking_conditions: Vec<String>,
    pub alternative_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSensitiveEvolution {
    pub target_id: String,
    pub time_window: TimeWindow,
    pub urgency_level: UrgencyLevel,
    pub special_rewards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingRequirement {
    pub requirement_type: String,
    pub current_value: String,
    pub required_value: String,
    pub progress_percentage: f32,
    pub estimated_time: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl Default for EvolutionRequirements {
    fn default() -> Self {
        Self {
            level_requirements: HashMap::new(),
            item_requirements: HashMap::new(),
            stat_thresholds: HashMap::new(),
            friendship_levels: HashMap::new(),
            location_requirements: HashMap::new(),
            time_windows: HashMap::new(),
            weather_conditions: HashMap::new(),
            special_conditions: HashMap::new(),
        }
    }
}

impl EvolutionTree {
    pub fn new(requirements: &EvolutionRequirements) -> CreatureEngineResult<Self> {
        Ok(Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
            requirements: requirements.clone(),
            evolution_history: Vec::new(),
            cached_paths: HashMap::new(),
        })
    }

    pub fn add_evolution_chain(
        &mut self,
        chain: &[String],
        chain_requirements: Vec<EvolutionRequirement>
    ) -> CreatureEngineResult<()> {
        if chain.len() < 2 {
            return Err(CreatureEngineError::EvolutionError("Evolution chain must have at least 2 creatures".to_string()));
        }

        for (i, creature_id) in chain.iter().enumerate() {
            let node = EvolutionNode {
                creature_id: creature_id.clone(),
                species_name: format!("Species_{}", creature_id),
                stage: i as u8,
                base_requirements: if i < chain_requirements.len() {
                    vec![chain_requirements[i].clone()]
                } else {
                    Vec::new()
                },
            };

            let node_idx = self.graph.add_node(node);
            self.node_map.insert(creature_id.clone(), node_idx);
        }

        for i in 0..chain.len()-1 {
            let from_idx = self.node_map[&chain[i]];
            let to_idx = self.node_map[&chain[i+1]];
            
            let edge = EvolutionEdge {
                requirements: if i < chain_requirements.len() {
                    chain_requirements[i].clone()
                } else {
                    EvolutionRequirement::default()
                },
                success_rate: 0.95,
                cost: EvolutionCost::default(),
                unlock_conditions: Vec::new(),
            };

            self.graph.add_edge(from_idx, to_idx, edge);
        }

        self.cached_paths.clear();
        Ok(())
    }

    pub fn add_branching_evolution(
        &mut self,
        base_creature: &str,
        branches: Vec<(String, EvolutionRequirement)>
    ) -> CreatureEngineResult<()> {
        let base_idx = self.node_map.get(base_creature)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Base creature '{}' not found", base_creature)))?;

        for (branch_creature, requirement) in branches {
            let branch_node = EvolutionNode {
                creature_id: branch_creature.clone(),
                species_name: format!("Species_{}", branch_creature),
                stage: self.graph[*base_idx].stage + 1,
                base_requirements: vec![requirement.clone()],
            };

            let branch_idx = self.graph.add_node(branch_node);
            self.node_map.insert(branch_creature, branch_idx);

            let edge = EvolutionEdge {
                requirements: requirement,
                success_rate: 0.90,
                cost: EvolutionCost::default(),
                unlock_conditions: Vec::new(),
            };

            self.graph.add_edge(*base_idx, branch_idx, edge);
        }

        self.cached_paths.clear();
        Ok(())
    }

    pub fn can_evolve(&self, creature: &GeneratedCreature) -> CreatureEngineResult<bool> {
        let node_idx = self.node_map.get(&creature.template_id)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Creature '{}' not in evolution tree", creature.template_id)))?;

        let outgoing_edges = self.graph.edges(*node_idx);
        
        for edge in outgoing_edges {
            if self.check_evolution_requirements(creature, &edge.weight().requirements)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get_next_evolution(&self, creature_id: &str) -> CreatureEngineResult<String> {
        let node_idx = self.node_map.get(creature_id)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Creature '{}' not in evolution tree", creature_id)))?;

        let mut edges: Vec<_> = self.graph.edges(*node_idx).collect();
        edges.sort_by(|a, b| b.weight().success_rate.partial_cmp(&a.weight().success_rate).unwrap());

        if let Some(edge) = edges.first() {
            let target_node = &self.graph[edge.1];
            Ok(target_node.creature_id.clone())
        } else {
            Err(CreatureEngineError::EvolutionError("No evolution available".to_string()))
        }
    }

    pub fn get_all_possible_evolutions(&self, creature_id: &str) -> CreatureEngineResult<Vec<String>> {
        let node_idx = self.node_map.get(creature_id)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Creature '{}' not in evolution tree", creature_id)))?;

        let mut evolutions = Vec::new();
        let outgoing_edges = self.graph.edges(*node_idx);
        
        for edge in outgoing_edges {
            let target_node = &self.graph[edge.1];
            evolutions.push(target_node.creature_id.clone());
        }

        Ok(evolutions)
    }

    pub fn get_evolution_path(&self, from: &str, to: &str) -> CreatureEngineResult<Option<Vec<EvolutionPath>>> {
        let cache_key = format!("{}_{}", from, to);
        if let Some(cached) = self.cached_paths.get(&cache_key) {
            return Ok(Some(cached.clone()));
        }

        let from_idx = self.node_map.get(from)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Source creature '{}' not found", from)))?;
        let to_idx = self.node_map.get(to)
            .ok_or_else(|| CreatureEngineError::EvolutionError(format!("Target creature '{}' not found", to)))?;

        let path_map = dijkstra(&self.graph, *from_idx, Some(*to_idx), |edge| {
            (edge.weight().cost.energy_cost as f32 * (1.0 - edge.weight().success_rate as f32)) as i32
        });

        if !path_map.contains_key(to_idx) {
            return Ok(None);
        }

        let path = self.reconstruct_path(*from_idx, *to_idx, &path_map)?;
        Ok(Some(path))
    }

    pub fn generate_evolution_forecast(&self, creature: &GeneratedCreature) -> CreatureEngineResult<EvolutionForecast> {
        let available_evolutions = self.get_available_evolutions(creature)?;
        let blocked_evolutions = self.get_blocked_evolutions(creature)?;
        let optimal_path = self.find_optimal_evolution_path(creature)?;
        let time_sensitive = self.get_time_sensitive_evolutions(creature)?;

        Ok(EvolutionForecast {
            creature_id: creature.id.clone(),
            available_evolutions,
            blocked_evolutions,
            optimal_path,
            time_sensitive_opportunities: time_sensitive,
        })
    }

    pub fn check_time_based_evolution(&self, creature_id: &str) -> CreatureEngineResult<Vec<String>> {
        let mut available = Vec::new();
        
        if let Some(time_windows) = self.requirements.time_windows.get(creature_id) {
            let now = Local::now();
            let current_hour = now.time().hour() as u8;
            let current_day = now.weekday().num_days_from_monday() as u8;

            for window in time_windows {
                if self.is_time_window_active(window, current_hour, current_day) {
                    if let Ok(evolutions) = self.get_all_possible_evolutions(creature_id) {
                        available.extend(evolutions);
                    }
                }
            }
        }

        Ok(available)
    }

    pub fn record_evolution(&mut self, record: EvolutionRecord) {
        self.evolution_history.push(record);
        
        if self.evolution_history.len() > 10000 {
            self.evolution_history.drain(0..1000);
        }
    }

    pub fn get_evolution_statistics(&self, creature_id: &str) -> EvolutionStatistics {
        let total_attempts = self.evolution_history.iter()
            .filter(|r| r.original_form == creature_id)
            .count();
        
        let successful_attempts = self.evolution_history.iter()
            .filter(|r| r.original_form == creature_id && r.success)
            .count();

        let success_rate = if total_attempts > 0 {
            successful_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };

        let most_common_evolution = self.evolution_history.iter()
            .filter(|r| r.original_form == creature_id && r.success)
            .fold(HashMap::new(), |mut acc, r| {
                *acc.entry(r.evolved_form.clone()).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(form, _)| form);

        EvolutionStatistics {
            creature_id: creature_id.to_string(),
            total_attempts,
            successful_attempts,
            success_rate,
            most_common_evolution,
        }
    }

    pub fn chain_count(&self) -> usize {
        let mut chains = HashSet::new();
        
        for node_idx in self.graph.node_indices() {
            let mut visited = HashSet::new();
            let mut current = node_idx;
            let mut chain_id = Vec::new();
            
            while !visited.contains(&current) {
                visited.insert(current);
                chain_id.push(self.graph[current].creature_id.clone());
                
                let outgoing: Vec<_> = self.graph.edges(current).collect();
                if let Some(edge) = outgoing.first() {
                    current = edge.1;
                } else {
                    break;
                }
            }
            
            if chain_id.len() > 1 {
                chain_id.sort();
                chains.insert(chain_id.join("->"));
            }
        }
        
        chains.len()
    }

    fn check_evolution_requirements(
        &self,
        creature: &GeneratedCreature,
        requirements: &EvolutionRequirement
    ) -> CreatureEngineResult<bool> {
        if let Some(level_req) = requirements.level_requirement {
            if creature.level < level_req {
                return Ok(false);
            }
        }

        for (stat_name, threshold) in &requirements.stat_requirements {
            if let Some(stat_value) = creature.base_stats.get(stat_name) {
                if *stat_value < *threshold {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_available_evolutions(&self, creature: &GeneratedCreature) -> CreatureEngineResult<Vec<EvolutionOption>> {
        let mut options = Vec::new();
        
        if let Ok(possible_evolutions) = self.get_all_possible_evolutions(&creature.template_id) {
            for evolution_id in possible_evolutions {
                let node_idx = self.node_map.get(&creature.template_id).unwrap();
                
                if let Some(edge) = self.graph.edges(*node_idx)
                    .find(|e| self.graph[e.1].creature_id == evolution_id) {
                    
                    let requirements_analysis = self.analyze_requirements(creature, &edge.weight().requirements)?;
                    
                    options.push(EvolutionOption {
                        target_id: evolution_id.clone(),
                        target_name: format!("Species_{}", evolution_id),
                        requirements_met: requirements_analysis.met,
                        missing_requirements: requirements_analysis.missing,
                        estimated_completion_time: requirements_analysis.estimated_time,
                        difficulty_score: requirements_analysis.difficulty,
                    });
                }
            }
        }
        
        Ok(options)
    }

    fn get_blocked_evolutions(&self, creature: &GeneratedCreature) -> CreatureEngineResult<Vec<BlockedEvolution>> {
        let mut blocked = Vec::new();
        
        // Implementation would analyze which evolutions are blocked and why
        // This is a simplified version
        
        Ok(blocked)
    }

    fn find_optimal_evolution_path(&self, creature: &GeneratedCreature) -> CreatureEngineResult<Option<EvolutionPath>> {
        // Find the evolution path with highest success rate and lowest cost
        if let Ok(available) = self.get_available_evolutions(creature) {
            if let Some(best_option) = available.iter()
                .min_by(|a, b| a.difficulty_score.partial_cmp(&b.difficulty_score).unwrap()) {
                
                return Ok(Some(EvolutionPath {
                    creature_id: creature.id.clone(),
                    target_id: best_option.target_id.clone(),
                    requirements: EvolutionRequirement::default(),
                    cost: EvolutionCost::default(),
                    success_rate: 0.95,
                    time_estimate: best_option.estimated_completion_time,
                    prerequisites: Vec::new(),
                }));
            }
        }
        
        Ok(None)
    }

    fn get_time_sensitive_evolutions(&self, creature: &GeneratedCreature) -> CreatureEngineResult<Vec<TimeSensitiveEvolution>> {
        let mut time_sensitive = Vec::new();
        
        if let Some(time_windows) = self.requirements.time_windows.get(&creature.template_id) {
            for window in time_windows {
                time_sensitive.push(TimeSensitiveEvolution {
                    target_id: "time_evolution".to_string(),
                    time_window: window.clone(),
                    urgency_level: UrgencyLevel::Medium,
                    special_rewards: vec!["Rare Trait".to_string()],
                });
            }
        }
        
        Ok(time_sensitive)
    }

    fn is_time_window_active(&self, window: &TimeWindow, current_hour: u8, current_day: u8) -> bool {
        let hour_match = current_hour >= window.start_hour && current_hour <= window.end_hour;
        let day_match = window.days_of_week.is_empty() || window.days_of_week.contains(&current_day);
        
        hour_match && day_match
    }

    fn reconstruct_path(
        &self,
        from: NodeIndex,
        to: NodeIndex,
        path_map: &HashMap<NodeIndex, i32>
    ) -> CreatureEngineResult<Vec<EvolutionPath>> {
        // Simplified path reconstruction
        let mut path = Vec::new();
        
        path.push(EvolutionPath {
            creature_id: self.graph[from].creature_id.clone(),
            target_id: self.graph[to].creature_id.clone(),
            requirements: EvolutionRequirement::default(),
            cost: EvolutionCost::default(),
            success_rate: 0.90,
            time_estimate: Some(3600),
            prerequisites: Vec::new(),
        });
        
        Ok(path)
    }

    fn analyze_requirements(&self, creature: &GeneratedCreature, requirements: &EvolutionRequirement) -> CreatureEngineResult<RequirementAnalysis> {
        let mut met = Vec::new();
        let mut missing = Vec::new();
        let mut difficulty = 0.0f32;

        if let Some(level_req) = requirements.level_requirement {
            if creature.level >= level_req {
                met.push(format!("Level requirement: {}/{}", creature.level, level_req));
            } else {
                missing.push(MissingRequirement {
                    requirement_type: "Level".to_string(),
                    current_value: creature.level.to_string(),
                    required_value: level_req.to_string(),
                    progress_percentage: (creature.level as f32 / level_req as f32) * 100.0,
                    estimated_time: Some((level_req - creature.level) as u32 * 3600),
                });
                difficulty += (level_req - creature.level) as f32 * 0.1;
            }
        }

        Ok(RequirementAnalysis {
            met,
            missing,
            difficulty,
            estimated_time: if missing.is_empty() { Some(0) } else { Some(3600) },
        })
    }
}

#[derive(Debug)]
struct RequirementAnalysis {
    met: Vec<String>,
    missing: Vec<MissingRequirement>,
    difficulty: f32,
    estimated_time: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionStatistics {
    pub creature_id: String,
    pub total_attempts: usize,
    pub successful_attempts: usize,
    pub success_rate: f64,
    pub most_common_evolution: Option<String>,
}

impl Default for EvolutionRequirement {
    fn default() -> Self {
        Self {
            target_id: String::new(),
            level_requirement: None,
            item_requirement: None,
            stat_requirements: HashMap::new(),
            special_conditions: Vec::new(),
            time_of_day: None,
            location_requirement: None,
        }
    }
}

impl Default for EvolutionCost {
    fn default() -> Self {
        Self {
            energy_cost: 100,
            items_consumed: Vec::new(),
            stat_changes: HashMap::new(),
            trait_changes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_tree_creation() {
        let requirements = EvolutionRequirements::default();
        let tree = EvolutionTree::new(&requirements);
        assert!(tree.is_ok());
    }

    #[test]
    fn test_add_evolution_chain() {
        let requirements = EvolutionRequirements::default();
        let mut tree = EvolutionTree::new(&requirements).unwrap();
        
        let chain = vec!["creature1".to_string(), "creature2".to_string(), "creature3".to_string()];
        let chain_reqs = vec![EvolutionRequirement::default(), EvolutionRequirement::default()];
        
        let result = tree.add_evolution_chain(&chain, chain_reqs);
        assert!(result.is_ok());
        assert_eq!(tree.chain_count(), 1);
    }

    #[test]
    fn test_evolution_path_finding() {
        let requirements = EvolutionRequirements::default();
        let mut tree = EvolutionTree::new(&requirements).unwrap();
        
        let chain = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let _ = tree.add_evolution_chain(&chain, vec![EvolutionRequirement::default(); 2]);
        
        let path = tree.get_evolution_path("a", "c").unwrap();
        assert!(path.is_some());
    }
}