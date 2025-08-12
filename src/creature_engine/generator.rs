/*
 * Pokemon Go - Creature Generator System
 * 开发心理过程:
 * 1. 实现高度可定制的生物生成器,支持多种生成策略
 * 2. 集成随机数生成、模板系统和统计学算法
 * 3. 实现程序化属性生成,确保每个生物都有独特的特征
 * 4. 支持基于种子的确定性生成,便于复现和平衡性测试
 * 5. 提供丰富的生成选项和约束条件
 */

use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};

use super::{CreatureEngineError, CreatureEngineResult, CreatureConfig, GeneratedCreature, CreatureStats};
use super::{CreatureTemplate, CreatureRarity, CreatureTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParameters {
    pub seed: Option<u64>,
    pub level_range: (u8, u8),
    pub force_rarity: Option<CreatureRarity>,
    pub stat_modifiers: HashMap<String, f64>,
    pub trait_filters: Vec<String>,
    pub generation_mode: GenerationMode,
    pub balance_mode: BalanceMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationMode {
    Standard,
    Competitive,
    Casual,
    Experimental,
    Custom(HashMap<String, f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalanceMode {
    Strict,
    Moderate,
    Lenient,
    Disabled,
}

impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            seed: None,
            level_range: (1, 50),
            force_rarity: None,
            stat_modifiers: HashMap::new(),
            trait_filters: Vec::new(),
            generation_mode: GenerationMode::Standard,
            balance_mode: BalanceMode::Moderate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatureGenerator {
    config: CreatureConfig,
    rng: ChaCha8Rng,
    generation_count: u64,
    stat_calculators: HashMap<String, Box<dyn StatCalculator + Send + Sync>>,
}

pub trait StatCalculator: std::fmt::Debug {
    fn calculate_stat(&self, base: u32, level: u8, iv: u8, ev: u8, nature_modifier: f64) -> u32;
    fn get_stat_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct StandardStatCalculator {
    stat_name: String,
}

impl StatCalculator for StandardStatCalculator {
    fn calculate_stat(&self, base: u32, level: u8, iv: u8, ev: u8, nature_modifier: f64) -> u32 {
        if self.stat_name == "hp" {
            (((((2 * base + iv as u32 + (ev as u32 / 4)) * level as u32) / 100) + level as u32 + 10) as f64 * nature_modifier) as u32
        } else {
            (((((2 * base + iv as u32 + (ev as u32 / 4)) * level as u32) / 100) + 5) as f64 * nature_modifier) as u32
        }
    }

    fn get_stat_name(&self) -> &str {
        &self.stat_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualValues {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub sp_attack: u8,
    pub sp_defense: u8,
    pub speed: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortValues {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub sp_attack: u8,
    pub sp_defense: u8,
    pub speed: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nature {
    pub name: String,
    pub increased_stat: Option<String>,
    pub decreased_stat: Option<String>,
    pub flavor_preferences: HashMap<String, i32>,
}

impl CreatureGenerator {
    pub fn new(config: &CreatureConfig) -> CreatureEngineResult<Self> {
        let rng = ChaCha8Rng::from_entropy();
        let mut stat_calculators: HashMap<String, Box<dyn StatCalculator + Send + Sync>> = HashMap::new();
        
        for stat_name in &["hp", "attack", "defense", "sp_attack", "sp_defense", "speed"] {
            stat_calculators.insert(
                stat_name.to_string(),
                Box::new(StandardStatCalculator { stat_name: stat_name.to_string() })
            );
        }

        Ok(Self {
            config: config.clone(),
            rng,
            generation_count: 0,
            stat_calculators,
        })
    }

    pub fn generate_from_template(
        &mut self,
        template: &CreatureTemplate,
        level: u8,
        rarity: CreatureRarity,
        traits: Vec<CreatureTrait>
    ) -> CreatureEngineResult<GeneratedCreature> {
        self.generation_count += 1;
        let seed = self.rng.gen::<u64>();
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(seed);

        let ivs = self.generate_ivs(&mut seeded_rng, &rarity)?;
        let evs = self.generate_evs(&mut seeded_rng, level)?;
        let nature = self.generate_nature(&mut seeded_rng)?;
        
        let base_stats = self.calculate_base_stats(template, &ivs, &evs, &nature, level)?;
        let modified_stats = self.apply_trait_modifiers(&base_stats, &traits)?;

        Ok(GeneratedCreature {
            id: format!("creature_{}", self.generation_count),
            template_id: template.id.clone(),
            level,
            rarity,
            base_stats: modified_stats,
            traits,
            mutations: Vec::new(),
            generation_seed: seed,
            created_at: chrono::Utc::now(),
        })
    }

    pub fn evolve_creature(
        &mut self,
        base_creature: &GeneratedCreature,
        evolution_template: &CreatureTemplate
    ) -> CreatureEngineResult<GeneratedCreature> {
        let mut evolved = base_creature.clone();
        evolved.template_id = evolution_template.id.clone();
        evolved.id = format!("evolved_{}", self.generation_count);
        self.generation_count += 1;

        let evolution_boost = self.calculate_evolution_boost(&base_creature.rarity)?;
        for (stat_name, base_value) in &base_creature.base_stats {
            if let Some(template_base) = evolution_template.base_stats.get(stat_name) {
                let boosted_value = (*base_value as f64 * evolution_boost) as u32;
                let new_value = boosted_value.max(*template_base);
                evolved.base_stats.insert(stat_name.clone(), new_value);
            }
        }

        evolved.traits.extend(self.generate_evolution_traits(&base_creature.rarity)?);
        Ok(evolved)
    }

    pub fn calculate_stats(&self, creature: &GeneratedCreature) -> CreatureEngineResult<CreatureStats> {
        let stats = CreatureStats {
            hp: *creature.base_stats.get("hp").unwrap_or(&0),
            attack: *creature.base_stats.get("attack").unwrap_or(&0),
            defense: *creature.base_stats.get("defense").unwrap_or(&0),
            sp_attack: *creature.base_stats.get("sp_attack").unwrap_or(&0),
            sp_defense: *creature.base_stats.get("sp_defense").unwrap_or(&0),
            speed: *creature.base_stats.get("speed").unwrap_or(&0),
        };
        
        let total = stats.hp + stats.attack + stats.defense + stats.sp_attack + stats.sp_defense + stats.speed;
        Ok(CreatureStats { total, ..stats })
    }

    fn generate_ivs(&mut self, rng: &mut ChaCha8Rng, rarity: &CreatureRarity) -> CreatureEngineResult<IndividualValues> {
        let min_iv = match rarity {
            CreatureRarity::Common => 0,
            CreatureRarity::Uncommon => 5,
            CreatureRarity::Rare => 10,
            CreatureRarity::Epic => 20,
            CreatureRarity::Legendary => 25,
            CreatureRarity::Mythical => 30,
        };

        Ok(IndividualValues {
            hp: rng.gen_range(min_iv..=31),
            attack: rng.gen_range(min_iv..=31),
            defense: rng.gen_range(min_iv..=31),
            sp_attack: rng.gen_range(min_iv..=31),
            sp_defense: rng.gen_range(min_iv..=31),
            speed: rng.gen_range(min_iv..=31),
        })
    }

    fn generate_evs(&mut self, rng: &mut ChaCha8Rng, level: u8) -> CreatureEngineResult<EffortValues> {
        let max_evs = ((level as f64 / 100.0) * 252.0) as u8;
        let total_evs = rng.gen_range(0..=510.min(max_evs as u16 * 6)) as u8;
        
        let mut remaining_evs = total_evs;
        let mut evs = EffortValues {
            hp: 0, attack: 0, defense: 0,
            sp_attack: 0, sp_defense: 0, speed: 0,
        };

        let stats = [&mut evs.hp, &mut evs.attack, &mut evs.defense, 
                    &mut evs.sp_attack, &mut evs.sp_defense, &mut evs.speed];

        for stat in stats {
            if remaining_evs > 0 {
                let allocation = rng.gen_range(0..=remaining_evs.min(252));
                *stat = allocation;
                remaining_evs = remaining_evs.saturating_sub(allocation);
            }
        }

        Ok(evs)
    }

    fn generate_nature(&mut self, rng: &mut ChaCha8Rng) -> CreatureEngineResult<Nature> {
        let natures = [
            ("Hardy", None, None),
            ("Lonely", Some("attack"), Some("defense")),
            ("Brave", Some("attack"), Some("speed")),
            ("Adamant", Some("attack"), Some("sp_attack")),
            ("Naughty", Some("attack"), Some("sp_defense")),
            ("Bold", Some("defense"), Some("attack")),
            ("Docile", None, None),
            ("Relaxed", Some("defense"), Some("speed")),
            ("Impish", Some("defense"), Some("sp_attack")),
            ("Lax", Some("defense"), Some("sp_defense")),
        ];

        let (name, inc, dec) = natures[rng.gen_range(0..natures.len())];
        
        Ok(Nature {
            name: name.to_string(),
            increased_stat: inc.map(|s| s.to_string()),
            decreased_stat: dec.map(|s| s.to_string()),
            flavor_preferences: HashMap::new(),
        })
    }

    fn calculate_base_stats(
        &self,
        template: &CreatureTemplate,
        ivs: &IndividualValues,
        evs: &EffortValues,
        nature: &Nature,
        level: u8
    ) -> CreatureEngineResult<HashMap<String, u32>> {
        let mut stats = HashMap::new();
        
        let iv_values = [
            ("hp", ivs.hp), ("attack", ivs.attack), ("defense", ivs.defense),
            ("sp_attack", ivs.sp_attack), ("sp_defense", ivs.sp_defense), ("speed", ivs.speed)
        ];
        
        let ev_values = [
            ("hp", evs.hp), ("attack", evs.attack), ("defense", evs.defense),
            ("sp_attack", evs.sp_attack), ("sp_defense", evs.sp_defense), ("speed", evs.speed)
        ];

        for ((stat_name, iv), (_, ev)) in iv_values.iter().zip(ev_values.iter()) {
            let base = *template.base_stats.get(*stat_name).unwrap_or(&50);
            let nature_modifier = self.get_nature_modifier(nature, stat_name);
            
            if let Some(calculator) = self.stat_calculators.get(*stat_name) {
                let final_stat = calculator.calculate_stat(base, level, *iv, *ev, nature_modifier);
                stats.insert(stat_name.to_string(), final_stat);
            }
        }

        Ok(stats)
    }

    fn get_nature_modifier(&self, nature: &Nature, stat_name: &str) -> f64 {
        if nature.increased_stat.as_deref() == Some(stat_name) {
            1.1
        } else if nature.decreased_stat.as_deref() == Some(stat_name) {
            0.9
        } else {
            1.0
        }
    }

    fn apply_trait_modifiers(
        &self,
        base_stats: &HashMap<String, u32>,
        traits: &[CreatureTrait]
    ) -> CreatureEngineResult<HashMap<String, u32>> {
        let mut modified_stats = base_stats.clone();

        for trait_obj in traits {
            for (stat_name, modifier) in &trait_obj.stat_modifiers {
                if let Some(current_value) = modified_stats.get_mut(stat_name) {
                    *current_value = (*current_value as f64 * modifier) as u32;
                }
            }
        }

        Ok(modified_stats)
    }

    fn calculate_evolution_boost(&self, rarity: &CreatureRarity) -> CreatureEngineResult<f64> {
        Ok(match rarity {
            CreatureRarity::Common => 1.2,
            CreatureRarity::Uncommon => 1.25,
            CreatureRarity::Rare => 1.3,
            CreatureRarity::Epic => 1.4,
            CreatureRarity::Legendary => 1.5,
            CreatureRarity::Mythical => 1.6,
        })
    }

    fn generate_evolution_traits(&mut self, rarity: &CreatureRarity) -> CreatureEngineResult<Vec<CreatureTrait>> {
        let trait_count = match rarity {
            CreatureRarity::Common => self.rng.gen_range(0..=1),
            CreatureRarity::Uncommon => self.rng.gen_range(0..=2),
            CreatureRarity::Rare => self.rng.gen_range(1..=2),
            CreatureRarity::Epic => self.rng.gen_range(1..=3),
            CreatureRarity::Legendary => self.rng.gen_range(2..=3),
            CreatureRarity::Mythical => self.rng.gen_range(2..=4),
        };

        let mut traits = Vec::new();
        for i in 0..trait_count {
            traits.push(CreatureTrait {
                id: format!("evolution_trait_{}", i),
                name: format!("Evolution Bonus {}", i + 1),
                description: "Bonus trait gained through evolution".to_string(),
                stat_modifiers: {
                    let mut modifiers = HashMap::new();
                    let boost_stat = ["attack", "defense", "speed"][self.rng.gen_range(0..3)];
                    modifiers.insert(boost_stat.to_string(), 1.1);
                    modifiers
                },
                special_effects: Vec::new(),
                rarity_requirement: *rarity,
            });
        }

        Ok(traits)
    }

    pub fn generate_with_parameters(
        &mut self,
        template: &CreatureTemplate,
        params: &GenerationParameters
    ) -> CreatureEngineResult<GeneratedCreature> {
        if let Some(seed) = params.seed {
            self.rng = ChaCha8Rng::seed_from_u64(seed);
        }

        let level = self.rng.gen_range(params.level_range.0..=params.level_range.1);
        let rarity = params.force_rarity.unwrap_or_else(|| {
            match params.generation_mode {
                GenerationMode::Competitive => CreatureRarity::Epic,
                GenerationMode::Casual => CreatureRarity::Common,
                _ => CreatureRarity::Uncommon,
            }
        });

        let traits = self.generate_traits_with_filters(&params.trait_filters, &rarity)?;
        let mut creature = self.generate_from_template(template, level, rarity, traits)?;

        for (stat_name, modifier) in &params.stat_modifiers {
            if let Some(stat_value) = creature.base_stats.get_mut(stat_name) {
                *stat_value = (*stat_value as f64 * modifier) as u32;
            }
        }

        Ok(creature)
    }

    fn generate_traits_with_filters(
        &mut self,
        filters: &[String],
        rarity: &CreatureRarity
    ) -> CreatureEngineResult<Vec<CreatureTrait>> {
        let mut traits = Vec::new();
        
        for filter in filters {
            if let Ok(trait_obj) = self.create_trait_by_name(filter, rarity) {
                traits.push(trait_obj);
            }
        }

        if traits.is_empty() {
            traits = self.generate_evolution_traits(rarity)?;
        }

        Ok(traits)
    }

    fn create_trait_by_name(&self, name: &str, rarity: &CreatureRarity) -> CreatureEngineResult<CreatureTrait> {
        let mut stat_modifiers = HashMap::new();
        
        match name.to_lowercase().as_str() {
            "strong" => { stat_modifiers.insert("attack".to_string(), 1.2); }
            "tough" => { stat_modifiers.insert("defense".to_string(), 1.2); }
            "fast" => { stat_modifiers.insert("speed".to_string(), 1.2); }
            "smart" => { stat_modifiers.insert("sp_attack".to_string(), 1.2); }
            "resilient" => { stat_modifiers.insert("sp_defense".to_string(), 1.2); }
            "hardy" => { stat_modifiers.insert("hp".to_string(), 1.2); }
            _ => return Err(CreatureEngineError::TraitError(format!("Unknown trait: {}", name))),
        }

        Ok(CreatureTrait {
            id: name.to_lowercase(),
            name: name.to_string(),
            description: format!("A creature with enhanced {} capabilities", name.to_lowercase()),
            stat_modifiers,
            special_effects: Vec::new(),
            rarity_requirement: *rarity,
        })
    }

    pub fn get_generation_stats(&self) -> GenerationStats {
        GenerationStats {
            total_generated: self.generation_count,
            current_seed: self.rng.get_seed()[0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub total_generated: u64,
    pub current_seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iv_generation() {
        let config = CreatureConfig::default();
        let mut generator = CreatureGenerator::new(&config).unwrap();
        let mut rng = ChaCha8Rng::from_entropy();
        
        let ivs = generator.generate_ivs(&mut rng, &CreatureRarity::Common).unwrap();
        assert!(ivs.hp <= 31);
        assert!(ivs.attack <= 31);
    }

    #[test]
    fn test_stat_calculation() {
        let calc = StandardStatCalculator { stat_name: "attack".to_string() };
        let stat = calc.calculate_stat(100, 50, 31, 252, 1.1);
        assert!(stat > 0);
    }

    #[test]
    fn test_nature_modifier() {
        let config = CreatureConfig::default();
        let generator = CreatureGenerator::new(&config).unwrap();
        
        let nature = Nature {
            name: "Adamant".to_string(),
            increased_stat: Some("attack".to_string()),
            decreased_stat: Some("sp_attack".to_string()),
            flavor_preferences: HashMap::new(),
        };
        
        assert_eq!(generator.get_nature_modifier(&nature, "attack"), 1.1);
        assert_eq!(generator.get_nature_modifier(&nature, "sp_attack"), 0.9);
        assert_eq!(generator.get_nature_modifier(&nature, "defense"), 1.0);
    }
}