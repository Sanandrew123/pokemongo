/*
* 开发心理过程：
* 1. 设计真实感的天气系统，影响游戏的视觉和玩法体验
* 2. 实现动态天气变化，基于时间、季节、地理位置
* 3. 支持天气对Pokemon类型、出现率、战斗的影响
* 4. 集成粒子效果和环境音效系统
* 5. 提供可预测和随机天气模式
* 6. 优化性能，支持大世界实时天气模拟
* 7. 实现天气预报系统，增加策略深度
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    core::error::{GameError, GameResult},
    pokemon::types::PokemonType,
    world::{
        tile::TerrainType,
        environment::{Season, TimeOfDay},
    },
    utils::random::RandomGenerator,
};

#[derive(Debug, Clone, Component)]
pub struct WeatherSystem {
    /// 当前天气状态
    pub current_weather: WeatherState,
    /// 天气历史
    pub weather_history: Vec<WeatherHistoryEntry>,
    /// 天气预报
    pub forecast: WeatherForecast,
    /// 全局天气配置
    pub config: WeatherConfig,
    /// 区域天气
    pub regional_weather: HashMap<Uuid, RegionalWeather>,
    /// 天气效果
    pub active_effects: HashMap<WeatherCondition, WeatherEffectInstance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,          // 晴朗
    PartlyCloudy,   // 少云
    Cloudy,         // 多云
    Overcast,       // 阴天
    LightRain,      // 小雨
    Rain,           // 中雨
    HeavyRain,      // 大雨
    Thunderstorm,   // 雷雨
    LightSnow,      // 小雪
    Snow,           // 中雪
    HeavySnow,      // 大雪
    Blizzard,       // 暴雪
    Fog,            // 雾
    Sandstorm,      // 沙尘暴
    Hail,           // 冰雹
    Tornado,        // 龙卷风
    Aurora,         // 极光（特殊天气）
    Eclipse,        // 日食（特殊天气）
    Meteor,         // 流星雨（特殊天气）
}

#[derive(Debug, Clone)]
pub struct WeatherState {
    pub condition: WeatherCondition,
    pub intensity: f32, // 0.0 - 1.0
    pub temperature: f32, // 摄氏度
    pub humidity: f32, // 0.0 - 1.0
    pub wind_speed: f32, // m/s
    pub wind_direction: f32, // 角度 0-360
    pub visibility: f32, // 0.0 - 1.0
    pub pressure: f32, // hPa
    pub start_time: f64,
    pub duration: f32, // 秒
}

#[derive(Debug, Clone)]
pub struct WeatherHistoryEntry {
    pub timestamp: f64,
    pub weather: WeatherState,
    pub location: Option<Vec2>,
    pub trigger_reason: WeatherTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherTrigger {
    Natural,        // 自然天气变化
    TimeProgression, // 时间推进
    SeasonChange,   // 季节变化
    PlayerAction,   // 玩家行动
    EventTriggered, // 事件触发
    AdminForced,    // 管理员强制
}

#[derive(Debug, Clone)]
pub struct WeatherForecast {
    /// 预报条目列表
    pub forecast_entries: Vec<ForecastEntry>,
    /// 预报准确度 (0.0 - 1.0)
    pub accuracy: f32,
    /// 预报范围（小时）
    pub forecast_range_hours: u32,
    /// 更新间隔（秒）
    pub update_interval: f32,
    /// 上次更新时间
    pub last_update: f64,
}

#[derive(Debug, Clone)]
pub struct ForecastEntry {
    pub time_offset_hours: u32,
    pub condition: WeatherCondition,
    pub probability: f32, // 0.0 - 1.0
    pub temperature_range: (f32, f32),
    pub precipitation_chance: f32,
}

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    /// 天气变化频率
    pub change_frequency: f32,
    /// 季节影响强度
    pub seasonal_influence: f32,
    /// 地形影响强度
    pub terrain_influence: f32,
    /// 随机性程度
    pub randomness_factor: f32,
    /// 极端天气概率
    pub extreme_weather_chance: f32,
    /// 天气持续时间范围
    pub duration_range: (f32, f32),
    /// 是否启用动态天气
    pub dynamic_weather: bool,
    /// 性能模式
    pub performance_mode: WeatherPerformanceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherPerformanceMode {
    High,    // 高质量：完整效果
    Medium,  // 中等质量：平衡效果和性能
    Low,     // 低质量：最小效果
}

#[derive(Debug, Clone)]
pub struct RegionalWeather {
    pub region_id: Uuid,
    pub bounds: WeatherRegionBounds,
    pub local_weather: WeatherState,
    pub biome_modifiers: HashMap<TerrainType, WeatherModifier>,
    pub altitude: f32,
    pub coastal_influence: f32,
}

#[derive(Debug, Clone)]
pub enum WeatherRegionBounds {
    Circle { center: Vec2, radius: f32 },
    Rectangle { min: Vec2, max: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

#[derive(Debug, Clone)]
pub struct WeatherModifier {
    pub condition_probability: HashMap<WeatherCondition, f32>,
    pub temperature_offset: f32,
    pub humidity_modifier: f32,
    pub wind_modifier: f32,
}

#[derive(Debug, Clone)]
pub struct WeatherEffectInstance {
    pub condition: WeatherCondition,
    pub visual_effects: Vec<VisualEffect>,
    pub audio_effects: Vec<AudioEffect>,
    pub gameplay_effects: GameplayEffects,
    pub intensity: f32,
    pub start_time: f64,
}

#[derive(Debug, Clone)]
pub struct VisualEffect {
    pub effect_type: VisualEffectType,
    pub intensity: f32,
    pub color: Color,
    pub blend_mode: VisualBlendMode,
    pub layer: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualEffectType {
    Particles,      // 粒子效果
    Overlay,        // 覆盖层
    Tint,          // 色调
    Blur,          // 模糊
    Lightning,     // 闪电
    Rainbow,       // 彩虹
    Fog,           // 雾效
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
}

#[derive(Debug, Clone)]
pub struct AudioEffect {
    pub sound_id: String,
    pub volume: f32,
    pub pitch: f32,
    pub loop_sound: bool,
    pub spatial: bool,
}

#[derive(Debug, Clone)]
pub struct GameplayEffects {
    /// Pokemon类型加成修正
    pub type_bonuses: HashMap<PokemonType, f32>,
    /// 遭遇率修正
    pub encounter_rate_modifier: f32,
    /// 视野距离修正
    pub visibility_modifier: f32,
    /// 移动速度修正
    pub movement_speed_modifier: f32,
    /// 特殊能力可用性
    pub ability_availability: HashMap<String, bool>,
}

impl WeatherSystem {
    pub fn new(config: WeatherConfig) -> Self {
        let initial_weather = WeatherState {
            condition: WeatherCondition::Clear,
            intensity: 0.5,
            temperature: 20.0,
            humidity: 0.5,
            wind_speed: 5.0,
            wind_direction: 0.0,
            visibility: 1.0,
            pressure: 1013.25,
            start_time: 0.0,
            duration: 3600.0, // 1小时
        };

        Self {
            current_weather: initial_weather,
            weather_history: Vec::new(),
            forecast: WeatherForecast::new(),
            config,
            regional_weather: HashMap::new(),
            active_effects: HashMap::new(),
        }
    }

    /// 更新天气系统
    pub fn update(
        &mut self,
        current_time: f64,
        season: Season,
        time_of_day: TimeOfDay,
        player_position: Vec2,
        rng: &mut RandomGenerator,
    ) -> GameResult<Vec<WeatherEvent>> {
        let mut events = Vec::new();

        // 检查是否需要天气变化
        if self.should_change_weather(current_time) {
            let new_weather = self.generate_next_weather(season, time_of_day, player_position, rng)?;
            events.push(self.change_weather(new_weather, current_time, WeatherTrigger::Natural)?);
        }

        // 更新区域天气
        self.update_regional_weather(current_time, season, time_of_day, rng)?;

        // 更新预报
        if self.should_update_forecast(current_time) {
            self.update_forecast(current_time, season, rng)?;
        }

        // 更新天气效果
        self.update_weather_effects(current_time)?;

        Ok(events)
    }

    fn should_change_weather(&self, current_time: f64) -> bool {
        let elapsed = current_time - self.current_weather.start_time;
        elapsed >= self.current_weather.duration as f64
    }

    fn generate_next_weather(
        &self,
        season: Season,
        time_of_day: TimeOfDay,
        position: Vec2,
        rng: &mut RandomGenerator,
    ) -> GameResult<WeatherState> {
        // 获取基础天气概率
        let mut weather_probabilities = self.get_base_weather_probabilities();

        // 应用季节修正
        self.apply_seasonal_modifiers(&mut weather_probabilities, season);

        // 应用时间修正
        self.apply_time_modifiers(&mut weather_probabilities, time_of_day);

        // 应用地形修正
        self.apply_terrain_modifiers(&mut weather_probabilities, position);

        // 应用当前天气的连续性影响
        self.apply_continuity_modifiers(&mut weather_probabilities);

        // 选择新天气
        let selected_condition = self.select_weather_condition(&weather_probabilities, rng);

        // 生成天气状态
        let weather_state = self.generate_weather_state(selected_condition, season, time_of_day, rng);

        Ok(weather_state)
    }

    fn get_base_weather_probabilities(&self) -> HashMap<WeatherCondition, f32> {
        let mut probabilities = HashMap::new();
        
        probabilities.insert(WeatherCondition::Clear, 0.30);
        probabilities.insert(WeatherCondition::PartlyCloudy, 0.25);
        probabilities.insert(WeatherCondition::Cloudy, 0.20);
        probabilities.insert(WeatherCondition::Overcast, 0.10);
        probabilities.insert(WeatherCondition::LightRain, 0.08);
        probabilities.insert(WeatherCondition::Rain, 0.05);
        probabilities.insert(WeatherCondition::HeavyRain, 0.01);
        probabilities.insert(WeatherCondition::Thunderstorm, 0.005);
        probabilities.insert(WeatherCondition::Fog, 0.003);
        probabilities.insert(WeatherCondition::Sandstorm, 0.002);
        
        probabilities
    }

    fn apply_seasonal_modifiers(&self, probabilities: &mut HashMap<WeatherCondition, f32>, season: Season) {
        match season {
            Season::Spring => {
                *probabilities.get_mut(&WeatherCondition::LightRain).unwrap() *= 1.5;
                *probabilities.get_mut(&WeatherCondition::Clear).unwrap() *= 1.2;
            },
            Season::Summer => {
                *probabilities.get_mut(&WeatherCondition::Clear).unwrap() *= 1.8;
                *probabilities.get_mut(&WeatherCondition::Thunderstorm).unwrap() *= 2.0;
                *probabilities.get_mut(&WeatherCondition::LightSnow).unwrap() *= 0.1;
                *probabilities.get_mut(&WeatherCondition::Snow).unwrap() *= 0.1;
            },
            Season::Autumn => {
                *probabilities.get_mut(&WeatherCondition::Cloudy).unwrap() *= 1.3;
                *probabilities.get_mut(&WeatherCondition::Fog).unwrap() *= 1.5;
                *probabilities.get_mut(&WeatherCondition::Rain).unwrap() *= 1.2;
            },
            Season::Winter => {
                *probabilities.get_mut(&WeatherCondition::LightSnow).unwrap() *= 3.0;
                *probabilities.get_mut(&WeatherCondition::Snow).unwrap() *= 2.0;
                *probabilities.get_mut(&WeatherCondition::Overcast).unwrap() *= 1.4;
                *probabilities.get_mut(&WeatherCondition::Clear).unwrap() *= 0.7;
            },
        }
    }

    fn apply_time_modifiers(&self, probabilities: &mut HashMap<WeatherCondition, f32>, time_of_day: TimeOfDay) {
        match time_of_day {
            TimeOfDay::Dawn | TimeOfDay::Dusk => {
                *probabilities.get_mut(&WeatherCondition::Fog).unwrap() *= 2.0;
            },
            TimeOfDay::Night => {
                *probabilities.get_mut(&WeatherCondition::Clear).unwrap() *= 1.1; // 夜晚更容易晴朗
            },
            _ => {},
        }
    }

    fn apply_terrain_modifiers(&self, _probabilities: &mut HashMap<WeatherCondition, f32>, _position: Vec2) {
        // 这里应该基于位置的地形类型来修正概率
        // 简化实现，实际需要查询地形系统
    }

    fn apply_continuity_modifiers(&self, probabilities: &mut HashMap<WeatherCondition, f32>) {
        // 天气的连续性：当前天气有更高概率持续
        let current = self.current_weather.condition;
        if let Some(current_prob) = probabilities.get_mut(&current) {
            *current_prob *= 1.5; // 50% 更高概率保持当前天气
        }

        // 相似天气的转换概率更高
        match current {
            WeatherCondition::Clear => {
                *probabilities.get_mut(&WeatherCondition::PartlyCloudy).unwrap() *= 1.3;
            },
            WeatherCondition::PartlyCloudy => {
                *probabilities.get_mut(&WeatherCondition::Clear).unwrap() *= 1.2;
                *probabilities.get_mut(&WeatherCondition::Cloudy).unwrap() *= 1.2;
            },
            WeatherCondition::LightRain => {
                *probabilities.get_mut(&WeatherCondition::Rain).unwrap() *= 1.4;
                *probabilities.get_mut(&WeatherCondition::Cloudy).unwrap() *= 1.3;
            },
            _ => {},
        }
    }

    fn select_weather_condition(
        &self,
        probabilities: &HashMap<WeatherCondition, f32>,
        rng: &mut RandomGenerator,
    ) -> WeatherCondition {
        let total_weight: f32 = probabilities.values().sum();
        let mut random_value = rng.range_f32(0.0, total_weight);

        for (&condition, &weight) in probabilities.iter() {
            random_value -= weight;
            if random_value <= 0.0 {
                return condition;
            }
        }

        // fallback
        WeatherCondition::Clear
    }

    fn generate_weather_state(
        &self,
        condition: WeatherCondition,
        season: Season,
        time_of_day: TimeOfDay,
        rng: &mut RandomGenerator,
    ) -> WeatherState {
        let base_temp = self.get_base_temperature(season, time_of_day);
        let temp_variation = self.get_weather_temperature_modifier(condition);
        
        WeatherState {
            condition,
            intensity: rng.range_f32(0.3, 1.0),
            temperature: base_temp + temp_variation + rng.range_f32(-3.0, 3.0),
            humidity: self.get_weather_humidity(condition, rng),
            wind_speed: self.get_weather_wind_speed(condition, rng),
            wind_direction: rng.range_f32(0.0, 360.0),
            visibility: self.get_weather_visibility(condition, rng),
            pressure: rng.range_f32(980.0, 1040.0),
            start_time: 0.0, // 将在change_weather中设置
            duration: self.get_weather_duration(condition, rng),
        }
    }

    fn get_base_temperature(&self, season: Season, time_of_day: TimeOfDay) -> f32 {
        let seasonal_base = match season {
            Season::Spring => 15.0,
            Season::Summer => 25.0,
            Season::Autumn => 12.0,
            Season::Winter => 2.0,
        };

        let time_modifier = match time_of_day {
            TimeOfDay::Dawn => -2.0,
            TimeOfDay::Morning => 0.0,
            TimeOfDay::Noon => 5.0,
            TimeOfDay::Afternoon => 3.0,
            TimeOfDay::Evening => -1.0,
            TimeOfDay::Dusk => -3.0,
            TimeOfDay::Night => -5.0,
        };

        seasonal_base + time_modifier
    }

    fn get_weather_temperature_modifier(&self, condition: WeatherCondition) -> f32 {
        match condition {
            WeatherCondition::Clear => 2.0,
            WeatherCondition::PartlyCloudy => 1.0,
            WeatherCondition::Cloudy => 0.0,
            WeatherCondition::Overcast => -1.0,
            WeatherCondition::LightRain | WeatherCondition::Rain => -3.0,
            WeatherCondition::HeavyRain | WeatherCondition::Thunderstorm => -5.0,
            WeatherCondition::LightSnow => -8.0,
            WeatherCondition::Snow => -12.0,
            WeatherCondition::HeavySnow | WeatherCondition::Blizzard => -18.0,
            WeatherCondition::Fog => -2.0,
            WeatherCondition::Sandstorm => 5.0,
            WeatherCondition::Hail => -10.0,
            _ => 0.0,
        }
    }

    fn get_weather_humidity(&self, condition: WeatherCondition, rng: &mut RandomGenerator) -> f32 {
        let base_humidity = match condition {
            WeatherCondition::Clear => 0.4,
            WeatherCondition::PartlyCloudy => 0.5,
            WeatherCondition::Cloudy => 0.6,
            WeatherCondition::Overcast => 0.7,
            WeatherCondition::LightRain => 0.8,
            WeatherCondition::Rain => 0.9,
            WeatherCondition::HeavyRain | WeatherCondition::Thunderstorm => 0.95,
            WeatherCondition::Fog => 1.0,
            WeatherCondition::Sandstorm => 0.1,
            _ => 0.5,
        };

        (base_humidity + rng.range_f32(-0.1, 0.1)).clamp(0.0, 1.0)
    }

    fn get_weather_wind_speed(&self, condition: WeatherCondition, rng: &mut RandomGenerator) -> f32 {
        let base_speed = match condition {
            WeatherCondition::Clear => 3.0,
            WeatherCondition::PartlyCloudy => 5.0,
            WeatherCondition::Cloudy => 8.0,
            WeatherCondition::Thunderstorm => 15.0,
            WeatherCondition::Sandstorm => 25.0,
            WeatherCondition::Blizzard => 20.0,
            WeatherCondition::Tornado => 50.0,
            _ => 5.0,
        };

        base_speed + rng.range_f32(-2.0, 5.0).max(0.0)
    }

    fn get_weather_visibility(&self, condition: WeatherCondition, rng: &mut RandomGenerator) -> f32 {
        let base_visibility = match condition {
            WeatherCondition::Clear => 1.0,
            WeatherCondition::PartlyCloudy => 0.95,
            WeatherCondition::Cloudy => 0.9,
            WeatherCondition::LightRain => 0.8,
            WeatherCondition::Rain => 0.6,
            WeatherCondition::HeavyRain => 0.4,
            WeatherCondition::Thunderstorm => 0.3,
            WeatherCondition::Fog => 0.2,
            WeatherCondition::Sandstorm => 0.15,
            WeatherCondition::Blizzard => 0.1,
            WeatherCondition::HeavySnow => 0.3,
            _ => 0.8,
        };

        (base_visibility + rng.range_f32(-0.05, 0.05)).clamp(0.05, 1.0)
    }

    fn get_weather_duration(&self, condition: WeatherCondition, rng: &mut RandomGenerator) -> f32 {
        let base_duration = match condition {
            WeatherCondition::Clear => 7200.0, // 2小时
            WeatherCondition::PartlyCloudy => 5400.0, // 1.5小时
            WeatherCondition::Cloudy => 3600.0, // 1小时
            WeatherCondition::LightRain => 2700.0, // 45分钟
            WeatherCondition::Rain => 1800.0, // 30分钟
            WeatherCondition::HeavyRain => 900.0, // 15分钟
            WeatherCondition::Thunderstorm => 600.0, // 10分钟
            WeatherCondition::Fog => 3600.0, // 1小时
            WeatherCondition::Sandstorm => 1200.0, // 20分钟
            WeatherCondition::Tornado => 300.0, // 5分钟
            _ => 1800.0, // 30分钟默认
        };

        let (min_duration, max_duration) = self.config.duration_range;
        let duration = base_duration * rng.range_f32(0.5, 1.5);
        duration.clamp(min_duration, max_duration)
    }

    fn change_weather(
        &mut self,
        mut new_weather: WeatherState,
        current_time: f64,
        trigger: WeatherTrigger,
    ) -> GameResult<WeatherEvent> {
        let old_weather = self.current_weather.condition;
        new_weather.start_time = current_time;

        // 记录历史
        self.weather_history.push(WeatherHistoryEntry {
            timestamp: current_time,
            weather: self.current_weather.clone(),
            location: None, // 可以添加位置信息
            trigger_reason: trigger,
        });

        // 限制历史记录长度
        if self.weather_history.len() > 100 {
            self.weather_history.drain(0..50);
        }

        // 更新当前天气
        self.current_weather = new_weather.clone();

        // 更新天气效果
        self.update_weather_effect_instance(new_weather.condition, new_weather.intensity, current_time);

        Ok(WeatherEvent {
            event_type: WeatherEventType::Changed,
            old_condition: Some(old_weather),
            new_condition: new_weather.condition,
            intensity: new_weather.intensity,
            timestamp: current_time,
            location: None,
        })
    }

    fn update_regional_weather(
        &mut self,
        current_time: f64,
        season: Season,
        time_of_day: TimeOfDay,
        rng: &mut RandomGenerator,
    ) -> GameResult<()> {
        // 更新每个区域的天气
        for (_, regional_weather) in self.regional_weather.iter_mut() {
            // 简化实现：区域天气受全球天气影响
            if rng.probability() < 0.1 { // 10% 概率区域天气发生变化
                // 可以在这里实现更复杂的区域天气逻辑
            }
        }
        Ok(())
    }

    fn should_update_forecast(&self, current_time: f64) -> bool {
        current_time - self.forecast.last_update >= self.forecast.update_interval as f64
    }

    fn update_forecast(
        &mut self,
        current_time: f64,
        season: Season,
        rng: &mut RandomGenerator,
    ) -> GameResult<()> {
        self.forecast.forecast_entries.clear();
        self.forecast.last_update = current_time;

        // 生成未来几小时的天气预报
        let mut current_condition = self.current_weather.condition;
        
        for hour in 1..=self.forecast.forecast_range_hours {
            let probabilities = self.get_base_weather_probabilities();
            // 简化：基于当前天气和时间偏移预测
            
            let forecast_entry = ForecastEntry {
                time_offset_hours: hour,
                condition: current_condition, // 简化实现
                probability: 0.7 + rng.range_f32(-0.2, 0.2),
                temperature_range: (
                    self.current_weather.temperature - 3.0,
                    self.current_weather.temperature + 3.0,
                ),
                precipitation_chance: self.get_precipitation_chance(current_condition),
            };

            self.forecast.forecast_entries.push(forecast_entry);

            // 有概率改变下一小时的天气
            if rng.probability() < 0.3 {
                current_condition = self.select_weather_condition(&probabilities, rng);
            }
        }

        Ok(())
    }

    fn get_precipitation_chance(&self, condition: WeatherCondition) -> f32 {
        match condition {
            WeatherCondition::Clear => 0.0,
            WeatherCondition::PartlyCloudy => 0.1,
            WeatherCondition::Cloudy => 0.3,
            WeatherCondition::Overcast => 0.5,
            WeatherCondition::LightRain => 0.8,
            WeatherCondition::Rain => 0.9,
            WeatherCondition::HeavyRain => 1.0,
            WeatherCondition::Thunderstorm => 1.0,
            _ => 0.0,
        }
    }

    fn update_weather_effects(&mut self, current_time: f64) -> GameResult<()> {
        // 更新活跃的天气效果
        let condition = self.current_weather.condition;
        let intensity = self.current_weather.intensity;
        
        self.update_weather_effect_instance(condition, intensity, current_time);
        
        Ok(())
    }

    fn update_weather_effect_instance(&mut self, condition: WeatherCondition, intensity: f32, current_time: f64) {
        let effect_instance = WeatherEffectInstance {
            condition,
            visual_effects: self.create_visual_effects(condition, intensity),
            audio_effects: self.create_audio_effects(condition, intensity),
            gameplay_effects: self.create_gameplay_effects(condition, intensity),
            intensity,
            start_time: current_time,
        };

        self.active_effects.insert(condition, effect_instance);
    }

    fn create_visual_effects(&self, condition: WeatherCondition, intensity: f32) -> Vec<VisualEffect> {
        let mut effects = Vec::new();

        match condition {
            WeatherCondition::Rain | WeatherCondition::LightRain | WeatherCondition::HeavyRain => {
                effects.push(VisualEffect {
                    effect_type: VisualEffectType::Particles,
                    intensity: intensity * 0.8,
                    color: Color::rgba(0.7, 0.8, 1.0, 0.6),
                    blend_mode: VisualBlendMode::Normal,
                    layer: 100,
                });
                effects.push(VisualEffect {
                    effect_type: VisualEffectType::Tint,
                    intensity: intensity * 0.3,
                    color: Color::rgba(0.6, 0.7, 0.9, 0.2),
                    blend_mode: VisualBlendMode::Multiply,
                    layer: -1,
                });
            },
            WeatherCondition::Snow | WeatherCondition::LightSnow | WeatherCondition::HeavySnow => {
                effects.push(VisualEffect {
                    effect_type: VisualEffectType::Particles,
                    intensity: intensity * 0.9,
                    color: Color::WHITE,
                    blend_mode: VisualBlendMode::Normal,
                    layer: 100,
                });
            },
            WeatherCondition::Fog => {
                effects.push(VisualEffect {
                    effect_type: VisualEffectType::Fog,
                    intensity: intensity * 0.7,
                    color: Color::rgba(0.9, 0.9, 0.9, 0.8),
                    blend_mode: VisualBlendMode::Normal,
                    layer: 50,
                });
            },
            WeatherCondition::Thunderstorm => {
                effects.push(VisualEffect {
                    effect_type: VisualEffectType::Lightning,
                    intensity: intensity,
                    color: Color::WHITE,
                    blend_mode: VisualBlendMode::Screen,
                    layer: 200,
                });
            },
            _ => {},
        }

        effects
    }

    fn create_audio_effects(&self, condition: WeatherCondition, intensity: f32) -> Vec<AudioEffect> {
        let mut effects = Vec::new();

        match condition {
            WeatherCondition::Rain => {
                effects.push(AudioEffect {
                    sound_id: "rain_light".to_string(),
                    volume: intensity * 0.6,
                    pitch: 1.0,
                    loop_sound: true,
                    spatial: false,
                });
            },
            WeatherCondition::HeavyRain => {
                effects.push(AudioEffect {
                    sound_id: "rain_heavy".to_string(),
                    volume: intensity * 0.8,
                    pitch: 1.0,
                    loop_sound: true,
                    spatial: false,
                });
            },
            WeatherCondition::Thunderstorm => {
                effects.push(AudioEffect {
                    sound_id: "thunder".to_string(),
                    volume: intensity,
                    pitch: 1.0,
                    loop_sound: false,
                    spatial: false,
                });
            },
            _ => {},
        }

        effects
    }

    fn create_gameplay_effects(&self, condition: WeatherCondition, intensity: f32) -> GameplayEffects {
        let mut type_bonuses = HashMap::new();
        let mut ability_availability = HashMap::new();

        match condition {
            WeatherCondition::Rain | WeatherCondition::HeavyRain => {
                type_bonuses.insert(PokemonType::Water, 1.5);
                type_bonuses.insert(PokemonType::Fire, 0.5);
                ability_availability.insert("Swift Swim".to_string(), true);
            },
            WeatherCondition::Clear => {
                type_bonuses.insert(PokemonType::Fire, 1.5);
                type_bonuses.insert(PokemonType::Grass, 1.2);
                ability_availability.insert("Solar Power".to_string(), true);
            },
            WeatherCondition::Sandstorm => {
                type_bonuses.insert(PokemonType::Rock, 1.3);
                type_bonuses.insert(PokemonType::Ground, 1.3);
                type_bonuses.insert(PokemonType::Steel, 1.3);
                ability_availability.insert("Sand Rush".to_string(), true);
            },
            WeatherCondition::Hail => {
                type_bonuses.insert(PokemonType::Ice, 1.5);
                ability_availability.insert("Snow Cloak".to_string(), true);
            },
            _ => {},
        }

        GameplayEffects {
            type_bonuses,
            encounter_rate_modifier: self.get_encounter_rate_modifier(condition, intensity),
            visibility_modifier: self.current_weather.visibility,
            movement_speed_modifier: self.get_movement_speed_modifier(condition, intensity),
            ability_availability,
        }
    }

    fn get_encounter_rate_modifier(&self, condition: WeatherCondition, intensity: f32) -> f32 {
        match condition {
            WeatherCondition::Rain => 1.2 + intensity * 0.3,
            WeatherCondition::Fog => 1.5 + intensity * 0.5,
            WeatherCondition::Sandstorm => 0.7 - intensity * 0.2,
            WeatherCondition::Clear => 1.0,
            _ => 1.0,
        }
    }

    fn get_movement_speed_modifier(&self, condition: WeatherCondition, intensity: f32) -> f32 {
        match condition {
            WeatherCondition::Blizzard => 0.5 - intensity * 0.3,
            WeatherCondition::HeavyRain => 0.8 - intensity * 0.2,
            WeatherCondition::Sandstorm => 0.7 - intensity * 0.2,
            WeatherCondition::Fog => 0.9 - intensity * 0.1,
            _ => 1.0,
        }
    }

    /// 强制设置天气
    pub fn force_weather(&mut self, condition: WeatherCondition, duration: f32, current_time: f64) -> GameResult<WeatherEvent> {
        let mut weather_state = self.generate_weather_state(condition, Season::Spring, TimeOfDay::Noon, &mut RandomGenerator::new());
        weather_state.duration = duration;
        
        self.change_weather(weather_state, current_time, WeatherTrigger::AdminForced)
    }

    /// 获取天气在指定位置的影响
    pub fn get_weather_at_position(&self, position: Vec2) -> WeatherState {
        // 检查区域天气
        for regional_weather in self.regional_weather.values() {
            if regional_weather.bounds.contains(position) {
                return regional_weather.local_weather.clone();
            }
        }

        // 返回全局天气
        self.current_weather.clone()
    }

    /// 获取当前天气预报
    pub fn get_forecast(&self) -> &WeatherForecast {
        &self.forecast
    }

    /// 获取天气历史
    pub fn get_weather_history(&self, max_entries: Option<usize>) -> &[WeatherHistoryEntry] {
        match max_entries {
            Some(max) => {
                let start = self.weather_history.len().saturating_sub(max);
                &self.weather_history[start..]
            },
            None => &self.weather_history,
        }
    }
}

impl WeatherForecast {
    pub fn new() -> Self {
        Self {
            forecast_entries: Vec::new(),
            accuracy: 0.8,
            forecast_range_hours: 6,
            update_interval: 1800.0, // 30分钟
            last_update: 0.0,
        }
    }
}

impl WeatherRegionBounds {
    pub fn contains(&self, position: Vec2) -> bool {
        match self {
            WeatherRegionBounds::Circle { center, radius } => {
                center.distance(position) <= *radius
            },
            WeatherRegionBounds::Rectangle { min, max } => {
                position.x >= min.x && position.x <= max.x &&
                position.y >= min.y && position.y <= max.y
            },
            WeatherRegionBounds::Polygon { vertices } => {
                // 简化的点在多边形内检测
                false // 需要实现射线投射算法
            },
        }
    }
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            change_frequency: 0.3,
            seasonal_influence: 0.7,
            terrain_influence: 0.5,
            randomness_factor: 0.4,
            extreme_weather_chance: 0.05,
            duration_range: (600.0, 7200.0), // 10分钟到2小时
            dynamic_weather: true,
            performance_mode: WeatherPerformanceMode::Medium,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeatherEvent {
    pub event_type: WeatherEventType,
    pub old_condition: Option<WeatherCondition>,
    pub new_condition: WeatherCondition,
    pub intensity: f32,
    pub timestamp: f64,
    pub location: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherEventType {
    Changed,        // 天气改变
    Intensified,    // 强度增加
    Weakened,       // 强度减少
    Started,        // 天气开始
    Ended,          // 天气结束
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_system_creation() {
        let config = WeatherConfig::default();
        let weather_system = WeatherSystem::new(config);
        
        assert_eq!(weather_system.current_weather.condition, WeatherCondition::Clear);
        assert_eq!(weather_system.weather_history.len(), 0);
    }

    #[test]
    fn test_weather_probabilities() {
        let config = WeatherConfig::default();
        let weather_system = WeatherSystem::new(config);
        let probabilities = weather_system.get_base_weather_probabilities();
        
        assert!(probabilities.get(&WeatherCondition::Clear).unwrap() > &0.0);
        assert!(probabilities.get(&WeatherCondition::Thunderstorm).unwrap() < probabilities.get(&WeatherCondition::Clear).unwrap());
    }

    #[test]
    fn test_forecast_creation() {
        let forecast = WeatherForecast::new();
        
        assert_eq!(forecast.forecast_range_hours, 6);
        assert_eq!(forecast.accuracy, 0.8);
        assert_eq!(forecast.forecast_entries.len(), 0);
    }

    #[test]
    fn test_weather_bounds() {
        let circle_bounds = WeatherRegionBounds::Circle {
            center: Vec2::ZERO,
            radius: 100.0,
        };
        
        assert!(circle_bounds.contains(Vec2::new(50.0, 50.0)));
        assert!(!circle_bounds.contains(Vec2::new(150.0, 150.0)));
        
        let rect_bounds = WeatherRegionBounds::Rectangle {
            min: Vec2::new(-50.0, -50.0),
            max: Vec2::new(50.0, 50.0),
        };
        
        assert!(rect_bounds.contains(Vec2::ZERO));
        assert!(!rect_bounds.contains(Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn test_weather_effects() {
        let config = WeatherConfig::default();
        let mut weather_system = WeatherSystem::new(config);
        
        let force_result = weather_system.force_weather(WeatherCondition::Rain, 1800.0, 0.0);
        assert!(force_result.is_ok());
        assert_eq!(weather_system.current_weather.condition, WeatherCondition::Rain);
        
        let effects = weather_system.active_effects.get(&WeatherCondition::Rain);
        assert!(effects.is_some());
    }
}