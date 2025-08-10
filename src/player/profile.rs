// 玩家档案系统
// 开发心理：档案展示玩家成就、个性化设置、社交信息
// 设计原则：信息完整、隐私保护、个性展示、社交互动

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn};
use crate::core::error::GameError;

// 玩家档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    // 基础信息
    pub display_name: String,
    pub avatar_id: u32,
    pub title: Option<String>,          // 称号
    pub bio: String,                    // 个人简介
    pub favorite_pokemon: Option<u32>,  // 最爱Pokemon
    
    // 个性化设置
    pub theme: String,
    pub badge_showcase: Vec<u32>,       // 展示的徽章
    pub achievement_showcase: Vec<u32>, // 展示的成就
    
    // 社交设置
    pub friend_code: String,
    pub privacy_settings: PrivacySettings,
    
    // 统计展示
    pub public_stats: PublicStats,
    
    // 自定义字段
    pub custom_fields: HashMap<String, String>,
}

// 隐私设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub show_location: bool,
    pub show_last_online: bool,
    pub show_stats: bool,
    pub allow_friend_requests: bool,
    pub show_pokemon_team: bool,
    pub show_achievements: bool,
}

// 公开统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicStats {
    pub trainer_level: u32,
    pub pokemon_caught: u32,
    pub distance_walked: f64,
    pub battles_won: u32,
    pub favorite_type: Option<String>,
    pub start_date: std::time::SystemTime,
}

impl PlayerProfile {
    pub fn new(display_name: String) -> Self {
        Self {
            display_name,
            avatar_id: 1,
            title: None,
            bio: String::new(),
            favorite_pokemon: None,
            theme: "default".to_string(),
            badge_showcase: Vec::new(),
            achievement_showcase: Vec::new(),
            friend_code: Self::generate_friend_code(),
            privacy_settings: PrivacySettings::default(),
            public_stats: PublicStats::default(),
            custom_fields: HashMap::new(),
        }
    }
    
    // 生成好友码
    fn generate_friend_code() -> String {
        format!(
            "{:04}-{:04}-{:04}",
            fastrand::u16(1000..10000),
            fastrand::u16(1000..10000),
            fastrand::u16(1000..10000)
        )
    }
    
    // 设置展示徽章
    pub fn set_badge_showcase(&mut self, badges: Vec<u32>) -> Result<(), GameError> {
        if badges.len() > 6 {
            return Err(GameError::Player("最多只能展示6个徽章".to_string()));
        }
        self.badge_showcase = badges;
        Ok(())
    }
    
    // 设置展示成就
    pub fn set_achievement_showcase(&mut self, achievements: Vec<u32>) -> Result<(), GameError> {
        if achievements.len() > 3 {
            return Err(GameError::Player("最多只能展示3个成就".to_string()));
        }
        self.achievement_showcase = achievements;
        Ok(())
    }
    
    // 更新公开统计
    pub fn update_public_stats(&mut self, stats: PublicStats) {
        self.public_stats = stats;
    }
    
    // 获取可见信息（考虑隐私设置）
    pub fn get_visible_profile(&self, is_friend: bool) -> VisibleProfile {
        VisibleProfile {
            display_name: self.display_name.clone(),
            avatar_id: self.avatar_id,
            title: self.title.clone(),
            bio: if self.privacy_settings.show_stats || is_friend {
                self.bio.clone()
            } else {
                "私密信息".to_string()
            },
            trainer_level: if self.privacy_settings.show_stats || is_friend {
                Some(self.public_stats.trainer_level)
            } else {
                None
            },
            badge_showcase: self.badge_showcase.clone(),
            achievement_showcase: if self.privacy_settings.show_achievements || is_friend {
                self.achievement_showcase.clone()
            } else {
                Vec::new()
            },
            friend_code: if is_friend {
                Some(self.friend_code.clone())
            } else {
                None
            },
        }
    }
}

// 可见档案信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleProfile {
    pub display_name: String,
    pub avatar_id: u32,
    pub title: Option<String>,
    pub bio: String,
    pub trainer_level: Option<u32>,
    pub badge_showcase: Vec<u32>,
    pub achievement_showcase: Vec<u32>,
    pub friend_code: Option<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            show_location: false,
            show_last_online: true,
            show_stats: true,
            allow_friend_requests: true,
            show_pokemon_team: false,
            show_achievements: true,
        }
    }
}

impl Default for PublicStats {
    fn default() -> Self {
        Self {
            trainer_level: 1,
            pokemon_caught: 0,
            distance_walked: 0.0,
            battles_won: 0,
            favorite_type: None,
            start_date: std::time::SystemTime::now(),
        }
    }
}