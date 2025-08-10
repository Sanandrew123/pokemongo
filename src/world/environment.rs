// 环境系统
// 开发心理：环境系统控制游戏世界的光照、音效、粒子特效、动态效果
// 设计原则：沉浸体验、性能优化、动态变化、氛围营造

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::core::error::GameError;
use glam::{Vec3, Vec4};

// 环境系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    // 光照系统
    pub lighting: LightingSystem,
    
    // 音效系统
    pub audio: AudioSystem,
    
    // 粒子系统
    pub particles: ParticleSystem,
    
    // 环境效果
    pub effects: Vec<EnvironmentEffect>,
    
    // 环境变量
    pub ambient_temperature: f32,
    pub humidity: f32,
    pub wind_speed: f32,
    pub wind_direction: Vec3,
    
    // 时间相关
    pub time_of_day: f32,        // 0.0-24.0
    pub season: Season,
    
    // 区域设置
    pub biome: BiomeType,
    pub elevation: f32,
    
    // 动态效果
    pub dynamic_objects: HashMap<u32, DynamicEnvironmentObject>,
    pub next_object_id: u32,
}

// 光照系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingSystem {
    pub sun_light: DirectionalLight,
    pub moon_light: DirectionalLight,
    pub point_lights: Vec<PointLight>,
    pub spot_lights: Vec<SpotLight>,
    pub ambient_light: Vec4,
    pub fog: FogSettings,
    pub shadows: ShadowSettings,
}

// 方向光
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec4,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub enabled: bool,
}

// 点光源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec4,
    pub intensity: f32,
    pub range: f32,
    pub attenuation: Vec3,       // 衰减系数 (constant, linear, quadratic)
    pub cast_shadows: bool,
    pub enabled: bool,
}

// 聚光灯
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec4,
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
    pub enabled: bool,
}

// 雾效设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogSettings {
    pub enabled: bool,
    pub color: Vec4,
    pub near_distance: f32,
    pub far_distance: f32,
    pub density: f32,
    pub fog_type: FogType,
}

// 雾类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FogType {
    Linear,         // 线性雾
    Exponential,    // 指数雾
    ExponentialSquared, // 指数平方雾
}

// 阴影设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSettings {
    pub enabled: bool,
    pub cascade_count: u32,
    pub shadow_distance: f32,
    pub shadow_resolution: u32,
    pub shadow_bias: f32,
    pub soft_shadows: bool,
}

// 音效系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSystem {
    pub background_music: Option<String>,
    pub ambient_sounds: Vec<AmbientSound>,
    pub sound_zones: Vec<SoundZone>,
    pub reverb_settings: ReverbSettings,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

// 环境音效
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientSound {
    pub sound_id: String,
    pub position: Option<Vec3>,  // None表示全局音效
    pub volume: f32,
    pub pitch: f32,
    pub loop_sound: bool,
    pub fade_distance: f32,      // 音效衰减距离
    pub is_playing: bool,
}

// 声音区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundZone {
    pub name: String,
    pub center: Vec3,
    pub radius: f32,
    pub sounds: Vec<String>,
    pub volume_multiplier: f32,
    pub reverb_preset: Option<String>,
}

// 混响设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbSettings {
    pub enabled: bool,
    pub preset: String,          // "cave", "forest", "hall", etc.
    pub room_size: f32,
    pub damping: f32,
    pub wet_level: f32,
    pub dry_level: f32,
}

// 粒子系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub emitters: Vec<ParticleEmitter>,
    pub global_wind: Vec3,
    pub gravity: Vec3,
    pub max_particles: u32,
    pub current_particles: u32,
}

// 粒子发射器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    pub id: u32,
    pub name: String,
    pub position: Vec3,
    pub emission_rate: f32,
    pub particle_lifetime: f32,
    pub start_velocity: Vec3,
    pub velocity_variation: Vec3,
    pub start_size: f32,
    pub end_size: f32,
    pub start_color: Vec4,
    pub end_color: Vec4,
    pub texture_id: Option<u32>,
    pub blend_mode: ParticleBlendMode,
    pub enabled: bool,
    pub looping: bool,
    pub duration: f32,
    pub remaining_time: f32,
}

// 粒子混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleBlendMode {
    Alpha,          // 透明混合
    Additive,       // 加法混合
    Multiply,       // 乘法混合
    Screen,         // 屏幕混合
}

// 季节
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,         // 春天
    Summer,         // 夏天
    Autumn,         // 秋天
    Winter,         // 冬天
}

// 生物群系
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Plains,         // 平原
    Forest,         // 森林
    Mountain,       // 山地
    Desert,         // 沙漠
    Ocean,          // 海洋
    Swamp,          // 沼泽
    Tundra,         // 苔原
    Jungle,         // 丛林
    Cave,           // 洞穴
    Urban,          // 城市
}

// 环境效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEffect {
    pub id: u32,
    pub effect_type: EffectType,
    pub position: Vec3,
    pub scale: Vec3,
    pub intensity: f32,
    pub duration: f32,
    pub remaining_time: f32,
    pub properties: HashMap<String, f32>,
    pub active: bool,
}

// 效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    Rain,           // 下雨
    Snow,           // 下雪
    Leaves,         // 落叶
    Dust,           // 灰尘
    Sparkle,        // 闪光
    Smoke,          // 烟雾
    Fire,           // 火焰
    Lightning,      // 闪电
    Wind,           // 风
    Magic,          // 魔法效果
}

// 动态环境对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicEnvironmentObject {
    pub id: u32,
    pub object_type: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub animation: Option<String>,
    pub physics_enabled: bool,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub properties: HashMap<String, f32>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            lighting: LightingSystem::new(),
            audio: AudioSystem::new(),
            particles: ParticleSystem::new(),
            effects: Vec::new(),
            ambient_temperature: 20.0,
            humidity: 0.5,
            wind_speed: 2.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.0),
            time_of_day: 12.0,
            season: Season::Spring,
            biome: BiomeType::Plains,
            elevation: 0.0,
            dynamic_objects: HashMap::new(),
            next_object_id: 1,
        }
    }
    
    // 更新环境系统
    pub fn update(&mut self, delta_time: f32) -> Result<(), GameError> {
        // 更新时间
        self.time_of_day += delta_time / 3600.0; // 1秒 = 1小时
        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }
        
        // 更新光照
        self.update_lighting(delta_time)?;
        
        // 更新粒子系统
        self.update_particles(delta_time)?;
        
        // 更新环境效果
        self.update_effects(delta_time)?;
        
        // 更新动态对象
        self.update_dynamic_objects(delta_time)?;
        
        Ok(())
    }
    
    // 设置天气效果
    pub fn set_weather(&mut self, weather: crate::world::Weather, intensity: f32) -> Result<(), GameError> {
        // 移除现有天气效果
        self.effects.retain(|effect| {
            !matches!(effect.effect_type, EffectType::Rain | EffectType::Snow)
        });
        
        // 添加新的天气效果
        let effect_type = match weather {
            crate::world::Weather::Rain => EffectType::Rain,
            crate::world::Weather::Snow => EffectType::Snow,
            _ => return Ok(()), // 其他天气不需要粒子效果
        };
        
        let effect = EnvironmentEffect {
            id: self.effects.len() as u32 + 1,
            effect_type,
            position: Vec3::ZERO,
            scale: Vec3::new(1000.0, 100.0, 1000.0), // 大范围效果
            intensity,
            duration: -1.0, // 持续效果
            remaining_time: -1.0,
            properties: HashMap::new(),
            active: true,
        };
        
        self.effects.push(effect);
        
        // 更新音效
        match weather {
            crate::world::Weather::Rain => {
                self.add_ambient_sound("rain".to_string(), None, 0.3 * intensity, true);
            },
            crate::world::Weather::Storm => {
                self.add_ambient_sound("thunder".to_string(), None, 0.5 * intensity, true);
                self.add_ambient_sound("heavy_rain".to_string(), None, 0.4 * intensity, true);
            },
            _ => {}
        }
        
        debug!("设置天气效果: {:?} 强度: {}", weather, intensity);
        Ok(())
    }
    
    // 设置生物群系
    pub fn set_biome(&mut self, biome: BiomeType) -> Result<(), GameError> {
        self.biome = biome;
        
        // 根据生物群系调整环境设置
        match biome {
            BiomeType::Desert => {
                self.ambient_temperature = 35.0;
                self.humidity = 0.1;
                self.lighting.ambient_light = Vec4::new(1.2, 1.1, 0.9, 1.0);
                self.add_particle_emitter("sand_particles", Vec3::ZERO, EffectType::Dust);
            },
            BiomeType::Forest => {
                self.ambient_temperature = 18.0;
                self.humidity = 0.7;
                self.lighting.ambient_light = Vec4::new(0.8, 1.0, 0.8, 1.0);
                self.add_ambient_sound("forest_birds".to_string(), None, 0.2, true);
                self.add_ambient_sound("wind_trees".to_string(), None, 0.3, true);
            },
            BiomeType::Ocean => {
                self.ambient_temperature = 22.0;
                self.humidity = 0.9;
                self.lighting.ambient_light = Vec4::new(0.9, 0.95, 1.1, 1.0);
                self.add_ambient_sound("ocean_waves".to_string(), None, 0.4, true);
            },
            BiomeType::Mountain => {
                self.ambient_temperature = 10.0;
                self.humidity = 0.4;
                self.wind_speed = 8.0;
                self.add_ambient_sound("mountain_wind".to_string(), None, 0.5, true);
            },
            BiomeType::Cave => {
                self.ambient_temperature = 15.0;
                self.humidity = 0.8;
                self.lighting.ambient_light = Vec4::new(0.3, 0.3, 0.4, 1.0);
                self.add_ambient_sound("cave_drip".to_string(), None, 0.1, true);
                self.audio.reverb_settings.preset = "cave".to_string();
                self.audio.reverb_settings.enabled = true;
            },
            _ => {}
        }
        
        debug!("设置生物群系: {:?}", biome);
        Ok(())
    }
    
    // 添加环境音效
    pub fn add_ambient_sound(&mut self, sound_id: String, position: Option<Vec3>, volume: f32, looping: bool) {
        let ambient_sound = AmbientSound {
            sound_id,
            position,
            volume,
            pitch: 1.0,
            loop_sound: looping,
            fade_distance: 100.0,
            is_playing: true,
        };
        
        self.audio.ambient_sounds.push(ambient_sound);
    }
    
    // 添加粒子发射器
    pub fn add_particle_emitter(&mut self, name: String, position: Vec3, effect_type: EffectType) -> u32 {
        let emitter_id = self.particles.emitters.len() as u32 + 1;
        
        let (emission_rate, lifetime, start_color, end_color) = match effect_type {
            EffectType::Rain => (50.0, 2.0, Vec4::new(0.8, 0.9, 1.0, 0.8), Vec4::new(0.8, 0.9, 1.0, 0.2)),
            EffectType::Snow => (20.0, 5.0, Vec4::new(1.0, 1.0, 1.0, 1.0), Vec4::new(1.0, 1.0, 1.0, 0.5)),
            EffectType::Dust => (15.0, 3.0, Vec4::new(0.8, 0.7, 0.5, 0.3), Vec4::new(0.8, 0.7, 0.5, 0.0)),
            EffectType::Leaves => (5.0, 8.0, Vec4::new(0.2, 0.8, 0.2, 1.0), Vec4::new(0.8, 0.6, 0.2, 0.8)),
            EffectType::Sparkle => (30.0, 1.0, Vec4::new(1.0, 1.0, 0.8, 1.0), Vec4::new(1.0, 1.0, 0.8, 0.0)),
            _ => (10.0, 2.0, Vec4::new(1.0, 1.0, 1.0, 0.8), Vec4::new(1.0, 1.0, 1.0, 0.2)),
        };
        
        let emitter = ParticleEmitter {
            id: emitter_id,
            name,
            position,
            emission_rate,
            particle_lifetime: lifetime,
            start_velocity: Vec3::new(0.0, -5.0, 0.0),
            velocity_variation: Vec3::new(2.0, 1.0, 2.0),
            start_size: 1.0,
            end_size: 0.5,
            start_color,
            end_color,
            texture_id: None,
            blend_mode: ParticleBlendMode::Alpha,
            enabled: true,
            looping: true,
            duration: -1.0,
            remaining_time: -1.0,
        };
        
        self.particles.emitters.push(emitter);
        emitter_id
    }
    
    // 私有方法
    fn update_lighting(&mut self, delta_time: f32) -> Result<(), GameError> {
        // 根据时间更新日光和月光
        let sun_angle = (self.time_of_day - 6.0) * std::f32::consts::PI / 12.0; // 6点为日出
        let sun_height = sun_angle.sin();
        
        // 太阳光
        self.lighting.sun_light.intensity = (sun_height * 1.2).max(0.0);
        self.lighting.sun_light.enabled = sun_height > 0.0;
        self.lighting.sun_light.direction = Vec3::new(
            sun_angle.cos(),
            -sun_height.abs(),
            0.3,
        ).normalize();
        
        // 月光
        self.lighting.moon_light.intensity = ((-sun_height) * 0.3).max(0.0);
        self.lighting.moon_light.enabled = sun_height < 0.0;
        
        // 环境光
        let ambient_intensity = if sun_height > 0.0 {
            0.3 + sun_height * 0.4
        } else {
            0.1 + (-sun_height) * 0.1
        };
        
        self.lighting.ambient_light.w = ambient_intensity;
        
        Ok(())
    }
    
    fn update_particles(&mut self, delta_time: f32) -> Result<(), GameError> {
        for emitter in &mut self.particles.emitters {
            if !emitter.enabled {
                continue;
            }
            
            if emitter.duration > 0.0 {
                emitter.remaining_time -= delta_time;
                if emitter.remaining_time <= 0.0 {
                    emitter.enabled = false;
                    continue;
                }
            }
        }
        
        Ok(())
    }
    
    fn update_effects(&mut self, delta_time: f32) -> Result<(), GameError> {
        let mut effects_to_remove = Vec::new();
        
        for (i, effect) in self.effects.iter_mut().enumerate() {
            if !effect.active {
                continue;
            }
            
            if effect.duration > 0.0 {
                effect.remaining_time -= delta_time;
                if effect.remaining_time <= 0.0 {
                    effects_to_remove.push(i);
                }
            }
        }
        
        // 移除过期效果
        for &i in effects_to_remove.iter().rev() {
            self.effects.remove(i);
        }
        
        Ok(())
    }
    
    fn update_dynamic_objects(&mut self, delta_time: f32) -> Result<(), GameError> {
        for object in self.dynamic_objects.values_mut() {
            if object.physics_enabled {
                // 简单的物理更新
                object.position += object.velocity * delta_time;
                object.rotation += object.angular_velocity * delta_time;
                
                // 应用重力和风力
                if object.properties.get("affected_by_gravity").unwrap_or(&0.0) > &0.0 {
                    object.velocity.y -= 9.8 * delta_time;
                }
                
                if object.properties.get("affected_by_wind").unwrap_or(&0.0) > &0.0 {
                    let wind_force = self.wind_direction * self.wind_speed * 0.1;
                    object.velocity += wind_force * delta_time;
                }
            }
        }
        
        Ok(())
    }
}

impl LightingSystem {
    pub fn new() -> Self {
        Self {
            sun_light: DirectionalLight {
                direction: Vec3::new(0.3, -0.7, 0.3).normalize(),
                color: Vec4::new(1.0, 0.95, 0.8, 1.0),
                intensity: 1.0,
                cast_shadows: true,
                enabled: true,
            },
            moon_light: DirectionalLight {
                direction: Vec3::new(-0.3, -0.7, -0.3).normalize(),
                color: Vec4::new(0.8, 0.8, 1.2, 1.0),
                intensity: 0.2,
                cast_shadows: false,
                enabled: false,
            },
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_light: Vec4::new(0.4, 0.4, 0.5, 0.3),
            fog: FogSettings {
                enabled: false,
                color: Vec4::new(0.5, 0.6, 0.7, 1.0),
                near_distance: 10.0,
                far_distance: 100.0,
                density: 0.1,
                fog_type: FogType::Linear,
            },
            shadows: ShadowSettings {
                enabled: true,
                cascade_count: 3,
                shadow_distance: 50.0,
                shadow_resolution: 1024,
                shadow_bias: 0.001,
                soft_shadows: true,
            },
        }
    }
}

impl AudioSystem {
    pub fn new() -> Self {
        Self {
            background_music: None,
            ambient_sounds: Vec::new(),
            sound_zones: Vec::new(),
            reverb_settings: ReverbSettings {
                enabled: false,
                preset: "none".to_string(),
                room_size: 0.5,
                damping: 0.5,
                wet_level: 0.3,
                dry_level: 0.7,
            },
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
        }
    }
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            emitters: Vec::new(),
            global_wind: Vec3::new(1.0, 0.0, 0.0),
            gravity: Vec3::new(0.0, -9.8, 0.0),
            max_particles: 10000,
            current_particles: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert_eq!(env.biome, BiomeType::Plains);
        assert_eq!(env.season, Season::Spring);
        assert!(env.lighting.sun_light.enabled);
    }
    
    #[test]
    fn test_biome_setting() {
        let mut env = Environment::new();
        env.set_biome(BiomeType::Desert).unwrap();
        
        assert_eq!(env.biome, BiomeType::Desert);
        assert_eq!(env.ambient_temperature, 35.0);
        assert_eq!(env.humidity, 0.1);
    }
    
    #[test]
    fn test_particle_emitter() {
        let mut env = Environment::new();
        let emitter_id = env.add_particle_emitter("test".to_string(), Vec3::ZERO, EffectType::Rain);
        
        assert!(emitter_id > 0);
        assert_eq!(env.particles.emitters.len(), 1);
        assert_eq!(env.particles.emitters[0].name, "test");
    }
    
    #[test]
    fn test_weather_effects() {
        let mut env = Environment::new();
        env.set_weather(crate::world::Weather::Rain, 0.8).unwrap();
        
        assert!(!env.effects.is_empty());
        assert_eq!(env.effects[0].effect_type, EffectType::Rain);
        assert!(!env.audio.ambient_sounds.is_empty());
    }
}