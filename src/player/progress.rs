// 游戏进度系统
// 开发心理：进度系统跟踪玩家游戏历程、解锁内容、成就记录
// 设计原则：渐进解锁、成就激励、里程碑记录、数据持久

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;

// 游戏进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProgress {
    // 故事进度
    pub story_progress: StoryProgress,
    
    // 徽章收集
    pub badges: HashMap<u32, Badge>,
    
    // 成就系统
    pub achievements: HashMap<u32, Achievement>,
    
    // 解锁内容
    pub unlocked_features: Vec<String>,
    pub unlocked_areas: Vec<String>,
    pub unlocked_pokemon: Vec<u32>,
    
    // 任务系统
    pub active_quests: Vec<Quest>,
    pub completed_quests: Vec<u32>,
    
    // 里程碑
    pub milestones: Vec<Milestone>,
}

// 故事进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryProgress {
    pub current_chapter: u32,
    pub completed_chapters: Vec<u32>,
    pub story_flags: HashMap<String, bool>,
    pub last_checkpoint: String,
}

// 徽章
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub obtained_date: std::time::SystemTime,
    pub gym_leader: String,
    pub location: String,
}

// 成就
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub progress: u32,
    pub target: u32,
    pub completed: bool,
    pub obtained_date: Option<std::time::SystemTime>,
    pub reward_coins: u32,
    pub reward_items: Vec<(u32, u32)>, // (item_id, quantity)
}

// 成就分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementCategory {
    Collector,      // 收集类
    Battler,        // 战斗类
    Explorer,       // 探索类
    Trainer,        // 训练类
    Social,         // 社交类
    Special,        // 特殊类
}

// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub quest_type: QuestType,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub started_date: std::time::SystemTime,
    pub deadline: Option<std::time::SystemTime>,
    pub completed: bool,
}

// 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestType {
    Main,           // 主线任务
    Side,           // 支线任务
    Daily,          // 日常任务
    Weekly,         // 周常任务
    Event,          // 活动任务
}

// 任务目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub objective_type: String,    // "catch_pokemon", "win_battles", etc.
    pub target_value: u32,
    pub current_value: u32,
    pub completed: bool,
}

// 任务奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestReward {
    pub reward_type: String,       // "coins", "items", "experience", etc.
    pub value: u32,
    pub item_id: Option<u32>,
}

// 里程碑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub achieved_date: std::time::SystemTime,
    pub special_reward: Option<String>,
}

impl GameProgress {
    pub fn new() -> Self {
        let mut progress = Self {
            story_progress: StoryProgress {
                current_chapter: 1,
                completed_chapters: Vec::new(),
                story_flags: HashMap::new(),
                last_checkpoint: "start".to_string(),
            },
            badges: HashMap::new(),
            achievements: HashMap::new(),
            unlocked_features: vec!["basic_catching".to_string()],
            unlocked_areas: vec!["starting_town".to_string()],
            unlocked_pokemon: Vec::new(),
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
            milestones: Vec::new(),
        };
        
        progress.initialize_achievements();
        progress.add_starter_quests();
        
        progress
    }
    
    // 初始化成就
    fn initialize_achievements(&mut self) {
        let achievements = vec![
            Achievement {
                id: 1,
                name: "初出茅庐".to_string(),
                description: "捕获第一只Pokemon".to_string(),
                category: AchievementCategory::Collector,
                progress: 0,
                target: 1,
                completed: false,
                obtained_date: None,
                reward_coins: 100,
                reward_items: vec![(1, 5)], // 5个精灵球
            },
            Achievement {
                id: 2,
                name: "Pokemon收集家".to_string(),
                description: "捕获10只不同的Pokemon".to_string(),
                category: AchievementCategory::Collector,
                progress: 0,
                target: 10,
                completed: false,
                obtained_date: None,
                reward_coins: 500,
                reward_items: vec![(2, 3)], // 3个超级球
            },
            Achievement {
                id: 3,
                name: "初级训练师".to_string(),
                description: "赢得第一场战斗".to_string(),
                category: AchievementCategory::Battler,
                progress: 0,
                target: 1,
                completed: false,
                obtained_date: None,
                reward_coins: 200,
                reward_items: vec![(101, 3)], // 3个伤药
            },
        ];
        
        for achievement in achievements {
            self.achievements.insert(achievement.id, achievement);
        }
        
        debug!("初始化成就系统: {} 个成就", self.achievements.len());
    }
    
    // 添加新手任务
    fn add_starter_quests(&mut self) {
        let starter_quest = Quest {
            id: 1,
            name: "Pokemon训练师的第一步".to_string(),
            description: "完成Pokemon训练师的基础训练".to_string(),
            quest_type: QuestType::Main,
            objectives: vec![
                QuestObjective {
                    description: "捕获一只野生Pokemon".to_string(),
                    objective_type: "catch_pokemon".to_string(),
                    target_value: 1,
                    current_value: 0,
                    completed: false,
                },
                QuestObjective {
                    description: "赢得一场Pokemon战斗".to_string(),
                    objective_type: "win_battle".to_string(),
                    target_value: 1,
                    current_value: 0,
                    completed: false,
                },
            ],
            rewards: vec![
                QuestReward {
                    reward_type: "experience".to_string(),
                    value: 500,
                    item_id: None,
                },
                QuestReward {
                    reward_type: "coins".to_string(),
                    value: 1000,
                    item_id: None,
                },
            ],
            started_date: std::time::SystemTime::now(),
            deadline: None,
            completed: false,
        };
        
        self.active_quests.push(starter_quest);
        debug!("添加新手任务");
    }
    
    // 更新成就进度
    pub fn update_achievement_progress(&mut self, achievement_id: u32, progress: u32) -> Result<bool, GameError> {
        if let Some(achievement) = self.achievements.get_mut(&achievement_id) {
            if achievement.completed {
                return Ok(false); // 已完成的成就不再更新
            }
            
            let old_progress = achievement.progress;
            achievement.progress = achievement.progress.max(progress);
            
            if achievement.progress >= achievement.target {
                achievement.completed = true;
                achievement.obtained_date = Some(std::time::SystemTime::now());
                
                debug!("完成成就: {} ({})", achievement.name, achievement_id);
                return Ok(true); // 返回true表示成就刚完成
            } else if achievement.progress > old_progress {
                debug!("更新成就进度: {} ({}/{}/{})", 
                    achievement.name, achievement.progress, achievement.target, achievement_id);
            }
        }
        
        Ok(false)
    }
    
    // 更新任务进度
    pub fn update_quest_progress(&mut self, objective_type: &str, value: u32) -> Vec<u32> {
        let mut completed_quests = Vec::new();
        
        for quest in &mut self.active_quests {
            if quest.completed {
                continue;
            }
            
            let mut all_objectives_complete = true;
            
            for objective in &mut quest.objectives {
                if !objective.completed && objective.objective_type == objective_type {
                    objective.current_value += value;
                    
                    if objective.current_value >= objective.target_value {
                        objective.completed = true;
                        debug!("完成任务目标: {} (任务: {})", objective.description, quest.name);
                    }
                }
                
                if !objective.completed {
                    all_objectives_complete = false;
                }
            }
            
            if all_objectives_complete {
                quest.completed = true;
                completed_quests.push(quest.id);
                debug!("完成任务: {} ({})", quest.name, quest.id);
            }
        }
        
        // 移动完成的任务
        self.active_quests.retain(|q| !q.completed);
        for &quest_id in &completed_quests {
            self.completed_quests.push(quest_id);
        }
        
        completed_quests
    }
    
    // 解锁新功能
    pub fn unlock_feature(&mut self, feature: String) -> bool {
        if !self.unlocked_features.contains(&feature) {
            self.unlocked_features.push(feature.clone());
            debug!("解锁新功能: {}", feature);
            true
        } else {
            false
        }
    }
    
    // 解锁新区域
    pub fn unlock_area(&mut self, area: String) -> bool {
        if !self.unlocked_areas.contains(&area) {
            self.unlocked_areas.push(area.clone());
            debug!("解锁新区域: {}", area);
            true
        } else {
            false
        }
    }
    
    // 获得徽章
    pub fn earn_badge(&mut self, badge: Badge) -> Result<(), GameError> {
        if self.badges.contains_key(&badge.id) {
            return Err(GameError::Progress("徽章已获得".to_string()));
        }
        
        self.badges.insert(badge.id, badge.clone());
        debug!("获得徽章: {} ({})", badge.name, badge.id);
        
        // 记录里程碑
        let milestone = Milestone {
            id: self.milestones.len() as u32 + 1000, // 徽章里程碑从1000开始编号
            name: format!("获得{}徽章", badge.name),
            description: format!("在{}击败了{}", badge.location, badge.gym_leader),
            achieved_date: badge.obtained_date,
            special_reward: Some("gym_badge".to_string()),
        };
        
        self.milestones.push(milestone);
        
        Ok(())
    }
    
    // 检查是否已解锁功能
    pub fn is_feature_unlocked(&self, feature: &str) -> bool {
        self.unlocked_features.contains(&feature.to_string())
    }
    
    // 检查是否已解锁区域
    pub fn is_area_unlocked(&self, area: &str) -> bool {
        self.unlocked_areas.contains(&area.to_string())
    }
    
    // 获取完成的成就
    pub fn get_completed_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().filter(|a| a.completed).collect()
    }
    
    // 获取进行中的成就
    pub fn get_active_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().filter(|a| !a.completed).collect()
    }
    
    // 计算总体进度百分比
    pub fn calculate_overall_progress(&self) -> f32 {
        let total_badges = 8; // 假设总共8个徽章
        let total_achievements = self.achievements.len();
        let total_story_chapters = 20; // 假设总共20章
        
        let badge_progress = (self.badges.len() as f32 / total_badges as f32) * 100.0;
        let achievement_progress = (self.get_completed_achievements().len() as f32 / total_achievements as f32) * 100.0;
        let story_progress = (self.story_progress.completed_chapters.len() as f32 / total_story_chapters as f32) * 100.0;
        
        (badge_progress + achievement_progress + story_progress) / 3.0
    }
    
    // 获取进度统计
    pub fn get_progress_stats(&self) -> ProgressStats {
        ProgressStats {
            badges_earned: self.badges.len(),
            achievements_completed: self.get_completed_achievements().len(),
            total_achievements: self.achievements.len(),
            chapters_completed: self.story_progress.completed_chapters.len(),
            active_quests: self.active_quests.len(),
            completed_quests: self.completed_quests.len(),
            features_unlocked: self.unlocked_features.len(),
            areas_unlocked: self.unlocked_areas.len(),
            overall_progress: self.calculate_overall_progress(),
        }
    }
}

// 进度统计
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub badges_earned: usize,
    pub achievements_completed: usize,
    pub total_achievements: usize,
    pub chapters_completed: usize,
    pub active_quests: usize,
    pub completed_quests: usize,
    pub features_unlocked: usize,
    pub areas_unlocked: usize,
    pub overall_progress: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_progress_creation() {
        let progress = GameProgress::new();
        assert_eq!(progress.story_progress.current_chapter, 1);
        assert!(!progress.achievements.is_empty());
        assert!(!progress.active_quests.is_empty());
        assert!(progress.is_feature_unlocked("basic_catching"));
    }
    
    #[test]
    fn test_achievement_progress() {
        let mut progress = GameProgress::new();
        
        // 更新成就进度
        let completed = progress.update_achievement_progress(1, 1).unwrap();
        assert!(completed); // 应该完成了第一个成就
        
        let achievement = progress.achievements.get(&1).unwrap();
        assert!(achievement.completed);
        assert!(achievement.obtained_date.is_some());
    }
    
    #[test]
    fn test_feature_unlock() {
        let mut progress = GameProgress::new();
        
        assert!(!progress.is_feature_unlocked("trading"));
        
        let unlocked = progress.unlock_feature("trading".to_string());
        assert!(unlocked);
        assert!(progress.is_feature_unlocked("trading"));
        
        // 重复解锁应该返回false
        let unlocked_again = progress.unlock_feature("trading".to_string());
        assert!(!unlocked_again);
    }
    
    #[test]
    fn test_quest_progress() {
        let mut progress = GameProgress::new();
        
        let completed = progress.update_quest_progress("catch_pokemon", 1);
        assert!(!completed.is_empty()); // 应该完成了一些任务
    }
}