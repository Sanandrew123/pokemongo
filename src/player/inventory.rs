// 背包系统
// 开发心理：背包管理需要分类存储、容量限制、物品效果、使用记录
// 设计原则：分类管理、容量控制、效果系统、持久化存储

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;

// 物品类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Pokeball,       // 精灵球
    Medicine,       // 药品
    Berry,          // 树果
    TM,            // 技能机器
    KeyItem,       // 重要道具
    Battle,        // 战斗道具
    Evolution,     // 进化道具
    Misc,          // 其他
}

// 物品稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,        // 普通
    Uncommon,      // 不常见
    Rare,          // 稀有
    Epic,          // 史诗
    Legendary,     // 传说
}

// 物品数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub max_stack: u32,        // 最大堆叠数量
    pub buy_price: u32,
    pub sell_price: u32,
    pub effects: Vec<ItemEffect>,
    pub usable_in_battle: bool,
    pub consumable: bool,
}

// 物品效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemEffect {
    pub effect_type: String,
    pub value: i32,
    pub target: String,        // "self", "pokemon", "all", etc.
}

// 背包物品实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: u32,
    pub quantity: u32,
    pub obtained_date: std::time::SystemTime,
}

// 背包系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<u32, InventoryItem>,
    pub capacity: HashMap<ItemType, u32>,     // 各类型容量限制
    pub sort_order: Vec<u32>,                 // 排序顺序
    
    // 统计
    pub total_items_obtained: u32,
    pub total_items_used: u32,
    pub coins: u32,                           // 游戏币
}

impl Inventory {
    pub fn new() -> Self {
        let mut capacity = HashMap::new();
        capacity.insert(ItemType::Pokeball, 50);
        capacity.insert(ItemType::Medicine, 30);
        capacity.insert(ItemType::Berry, 20);
        capacity.insert(ItemType::TM, 20);
        capacity.insert(ItemType::KeyItem, 50);
        capacity.insert(ItemType::Battle, 20);
        capacity.insert(ItemType::Evolution, 10);
        capacity.insert(ItemType::Misc, 30);
        
        Self {
            items: HashMap::new(),
            capacity,
            sort_order: Vec::new(),
            total_items_obtained: 0,
            total_items_used: 0,
            coins: 1000, // 初始金币
        }
    }
    
    // 添加物品
    pub fn add_item(&mut self, item_id: u32, quantity: u32, item_data: &Item) -> Result<u32, GameError> {
        // 检查容量限制
        let current_count = self.get_item_type_count(item_data.item_type);
        let type_capacity = self.capacity.get(&item_data.item_type).unwrap_or(&50);
        
        if current_count >= *type_capacity && !self.items.contains_key(&item_id) {
            return Err(GameError::Inventory("背包空间不足".to_string()));
        }
        
        let actual_quantity = if let Some(existing) = self.items.get_mut(&item_id) {
            // 物品已存在，增加数量
            let max_can_add = item_data.max_stack - existing.quantity;
            let added = quantity.min(max_can_add);
            existing.quantity += added;
            added
        } else {
            // 新物品
            let added = quantity.min(item_data.max_stack);
            self.items.insert(item_id, InventoryItem {
                item_id,
                quantity: added,
                obtained_date: std::time::SystemTime::now(),
            });
            
            // 更新排序顺序
            if !self.sort_order.contains(&item_id) {
                self.sort_order.push(item_id);
            }
            
            added
        };
        
        self.total_items_obtained += actual_quantity;
        debug!("添加物品: ID={} 数量={}", item_id, actual_quantity);
        
        Ok(actual_quantity)
    }
    
    // 移除物品
    pub fn remove_item(&mut self, item_id: u32, quantity: u32) -> Result<u32, GameError> {
        if let Some(item) = self.items.get_mut(&item_id) {
            let removed = quantity.min(item.quantity);
            item.quantity -= removed;
            
            // 如果数量为0，移除物品
            if item.quantity == 0 {
                self.items.remove(&item_id);
                self.sort_order.retain(|&id| id != item_id);
            }
            
            debug!("移除物品: ID={} 数量={}", item_id, removed);
            Ok(removed)
        } else {
            Err(GameError::Inventory(format!("物品不存在: {}", item_id)))
        }
    }
    
    // 使用物品
    pub fn use_item(&mut self, item_id: u32, quantity: u32) -> Result<u32, GameError> {
        let used = self.remove_item(item_id, quantity)?;
        self.total_items_used += used;
        debug!("使用物品: ID={} 数量={}", item_id, used);
        Ok(used)
    }
    
    // 获取物品数量
    pub fn get_item_quantity(&self, item_id: u32) -> u32 {
        self.items.get(&item_id).map_or(0, |item| item.quantity)
    }
    
    // 检查是否拥有物品
    pub fn has_item(&self, item_id: u32, quantity: u32) -> bool {
        self.get_item_quantity(item_id) >= quantity
    }
    
    // 获取按类型分组的物品
    pub fn get_items_by_type(&self, item_type: ItemType, item_database: &ItemDatabase) -> Vec<u32> {
        self.items
            .keys()
            .filter(|&&item_id| {
                item_database.get_item(item_id)
                    .map_or(false, |item| item.item_type == item_type)
            })
            .copied()
            .collect()
    }
    
    // 获取某类型物品的数量
    pub fn get_item_type_count(&self, item_type: ItemType) -> u32 {
        // 简化实现：返回该类型的唯一物品种类数量
        // 实际可能需要考虑堆叠等复杂情况
        self.items.len() as u32 // 简化版本
    }
    
    // 排序背包
    pub fn sort_by_type(&mut self, item_database: &ItemDatabase) {
        self.sort_order.sort_by(|&a, &b| {
            let type_a = item_database.get_item(a).map_or(ItemType::Misc, |item| item.item_type);
            let type_b = item_database.get_item(b).map_or(ItemType::Misc, |item| item.item_type);
            
            type_a.cmp(&type_b).then_with(|| a.cmp(&b))
        });
        
        debug!("背包按类型排序完成");
    }
    
    // 按稀有度排序
    pub fn sort_by_rarity(&mut self, item_database: &ItemDatabase) {
        self.sort_order.sort_by(|&a, &b| {
            let rarity_a = item_database.get_item(a).map_or(ItemRarity::Common, |item| item.rarity);
            let rarity_b = item_database.get_item(b).map_or(ItemRarity::Common, |item| item.rarity);
            
            rarity_b.cmp(&rarity_a).then_with(|| a.cmp(&b)) // 稀有度降序
        });
        
        debug!("背包按稀有度排序完成");
    }
    
    // 按获得时间排序
    pub fn sort_by_obtained_date(&mut self) {
        self.sort_order.sort_by(|&a, &b| {
            let date_a = self.items.get(&a).map_or(std::time::SystemTime::UNIX_EPOCH, |item| item.obtained_date);
            let date_b = self.items.get(&b).map_or(std::time::SystemTime::UNIX_EPOCH, |item| item.obtained_date);
            
            date_b.cmp(&date_a) // 最新获得的在前
        });
        
        debug!("背包按获得时间排序完成");
    }
    
    // 搜索物品
    pub fn search_items(&self, query: &str, item_database: &ItemDatabase) -> Vec<u32> {
        self.items
            .keys()
            .filter(|&&item_id| {
                item_database.get_item(item_id)
                    .map_or(false, |item| {
                        item.name.to_lowercase().contains(&query.to_lowercase()) ||
                        item.description.to_lowercase().contains(&query.to_lowercase())
                    })
            })
            .copied()
            .collect()
    }
    
    // 获取背包统计信息
    pub fn get_stats(&self) -> InventoryStats {
        InventoryStats {
            total_unique_items: self.items.len(),
            total_item_count: self.items.values().map(|item| item.quantity).sum(),
            coins: self.coins,
            items_obtained: self.total_items_obtained,
            items_used: self.total_items_used,
        }
    }
    
    // 清空背包 (调试用)
    pub fn clear(&mut self) {
        self.items.clear();
        self.sort_order.clear();
        debug!("清空背包");
    }
}

// 物品数据库
pub struct ItemDatabase {
    items: HashMap<u32, Item>,
}

impl ItemDatabase {
    pub fn new() -> Self {
        let mut database = Self {
            items: HashMap::new(),
        };
        database.initialize_default_items();
        database
    }
    
    // 初始化默认物品
    fn initialize_default_items(&mut self) {
        // 精灵球
        self.add_item(Item {
            id: 1,
            name: "精灵球".to_string(),
            description: "最基础的捕获道具".to_string(),
            item_type: ItemType::Pokeball,
            rarity: ItemRarity::Common,
            max_stack: 99,
            buy_price: 200,
            sell_price: 100,
            effects: vec![ItemEffect {
                effect_type: "catch_rate".to_string(),
                value: 100,
                target: "wild_pokemon".to_string(),
            }],
            usable_in_battle: true,
            consumable: true,
        });
        
        // 超级球
        self.add_item(Item {
            id: 2,
            name: "超级球".to_string(),
            description: "比精灵球更容易捕获Pokemon".to_string(),
            item_type: ItemType::Pokeball,
            rarity: ItemRarity::Uncommon,
            max_stack: 99,
            buy_price: 600,
            sell_price: 300,
            effects: vec![ItemEffect {
                effect_type: "catch_rate".to_string(),
                value: 150,
                target: "wild_pokemon".to_string(),
            }],
            usable_in_battle: true,
            consumable: true,
        });
        
        // 伤药
        self.add_item(Item {
            id: 101,
            name: "伤药".to_string(),
            description: "恢复Pokemon 20 HP".to_string(),
            item_type: ItemType::Medicine,
            rarity: ItemRarity::Common,
            max_stack: 50,
            buy_price: 300,
            sell_price: 150,
            effects: vec![ItemEffect {
                effect_type: "heal_hp".to_string(),
                value: 20,
                target: "pokemon".to_string(),
            }],
            usable_in_battle: true,
            consumable: true,
        });
        
        // 好伤药
        self.add_item(Item {
            id: 102,
            name: "好伤药".to_string(),
            description: "恢复Pokemon 50 HP".to_string(),
            item_type: ItemType::Medicine,
            rarity: ItemRarity::Uncommon,
            max_stack: 50,
            buy_price: 700,
            sell_price: 350,
            effects: vec![ItemEffect {
                effect_type: "heal_hp".to_string(),
                value: 50,
                target: "pokemon".to_string(),
            }],
            usable_in_battle: true,
            consumable: true,
        });
        
        debug!("初始化物品数据库: {} 个物品", self.items.len());
    }
    
    // 添加物品到数据库
    pub fn add_item(&mut self, item: Item) {
        self.items.insert(item.id, item);
    }
    
    // 获取物品数据
    pub fn get_item(&self, item_id: u32) -> Option<&Item> {
        self.items.get(&item_id)
    }
    
    // 获取所有物品
    pub fn get_all_items(&self) -> Vec<&Item> {
        self.items.values().collect()
    }
    
    // 按类型获取物品
    pub fn get_items_by_type(&self, item_type: ItemType) -> Vec<&Item> {
        self.items
            .values()
            .filter(|item| item.item_type == item_type)
            .collect()
    }
}

// 背包统计
#[derive(Debug, Clone)]
pub struct InventoryStats {
    pub total_unique_items: usize,
    pub total_item_count: u32,
    pub coins: u32,
    pub items_obtained: u32,
    pub items_used: u32,
}

// 实现ItemType的比较顺序
impl PartialOrd for ItemType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let order = |t: &ItemType| match t {
            ItemType::Pokeball => 0,
            ItemType::Medicine => 1,
            ItemType::Berry => 2,
            ItemType::Battle => 3,
            ItemType::TM => 4,
            ItemType::Evolution => 5,
            ItemType::KeyItem => 6,
            ItemType::Misc => 7,
        };
        
        order(self).cmp(&order(other))
    }
}

// 实现ItemRarity的比较顺序
impl PartialOrd for ItemRarity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemRarity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let value = |r: &ItemRarity| match r {
            ItemRarity::Common => 0,
            ItemRarity::Uncommon => 1,
            ItemRarity::Rare => 2,
            ItemRarity::Epic => 3,
            ItemRarity::Legendary => 4,
        };
        
        value(self).cmp(&value(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inventory_creation() {
        let inventory = Inventory::new();
        assert_eq!(inventory.items.len(), 0);
        assert_eq!(inventory.coins, 1000);
    }
    
    #[test]
    fn test_item_add_remove() {
        let mut inventory = Inventory::new();
        let database = ItemDatabase::new();
        
        let pokeball = database.get_item(1).unwrap();
        
        // 添加物品
        let added = inventory.add_item(1, 5, pokeball).unwrap();
        assert_eq!(added, 5);
        assert_eq!(inventory.get_item_quantity(1), 5);
        
        // 移除物品
        let removed = inventory.remove_item(1, 3).unwrap();
        assert_eq!(removed, 3);
        assert_eq!(inventory.get_item_quantity(1), 2);
        
        // 使用物品
        let used = inventory.use_item(1, 2).unwrap();
        assert_eq!(used, 2);
        assert_eq!(inventory.get_item_quantity(1), 0);
        assert_eq!(inventory.total_items_used, 2);
    }
    
    #[test]
    fn test_item_stacking() {
        let mut inventory = Inventory::new();
        let database = ItemDatabase::new();
        
        let pokeball = database.get_item(1).unwrap();
        
        // 添加到堆叠上限
        let added1 = inventory.add_item(1, 50, pokeball).unwrap();
        assert_eq!(added1, 50);
        
        let added2 = inventory.add_item(1, 60, pokeball).unwrap();
        assert_eq!(added2, 49); // 只能再添加49个，因为max_stack是99
        assert_eq!(inventory.get_item_quantity(1), 99);
    }
    
    #[test]
    fn test_item_database() {
        let database = ItemDatabase::new();
        
        assert!(database.get_item(1).is_some()); // 精灵球
        assert!(database.get_item(101).is_some()); // 伤药
        assert!(database.get_item(999).is_none()); // 不存在的物品
        
        let pokeball = database.get_item(1).unwrap();
        assert_eq!(pokeball.name, "精灵球");
        assert_eq!(pokeball.item_type, ItemType::Pokeball);
    }
}