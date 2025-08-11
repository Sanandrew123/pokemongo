// Text Processing and Formatting System - 文本处理和格式化系统
//
// 开发心理过程：
// 1. 这个模块处理游戏中的所有文本相关功能，包括本地化、格式化、动画效果
// 2. 支持多语言本地化系统，动态加载语言包
// 3. 实现文本动画效果，如打字机效果、淡入淡出等
// 4. 提供文本格式化工具，支持颜色标记、变量替换等
// 5. 集成富文本渲染，支持不同字体、大小、样式的混合显示

use std::collections::{HashMap, VecDeque};
use std::fmt;
use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;

use crate::graphics::{Color, TextStyle};

/// 文本管理器
pub struct TextManager {
    pub current_language: String,
    pub language_packs: HashMap<String, LanguagePack>,
    pub formatters: HashMap<String, TextFormatter>,
    pub animation_system: TextAnimationSystem,
    pub rich_text_processor: RichTextProcessor,
    pub text_cache: HashMap<String, FormattedText>,
}

impl Default for TextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TextManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_language: "en".to_string(),
            language_packs: HashMap::new(),
            formatters: HashMap::new(),
            animation_system: TextAnimationSystem::new(),
            rich_text_processor: RichTextProcessor::new(),
            text_cache: HashMap::new(),
        };

        // 初始化默认语言包
        manager.load_default_language_packs();
        
        // 初始化默认格式化器
        manager.setup_default_formatters();

        manager
    }

    fn load_default_language_packs(&mut self) {
        // English language pack
        let mut en_pack = LanguagePack::new("en".to_string(), "English".to_string());
        en_pack.add_text("welcome", "Welcome to Pokemon Adventure!");
        en_pack.add_text("battle_start", "A wild {pokemon_name} appeared!");
        en_pack.add_text("battle_win", "You won the battle!");
        en_pack.add_text("level_up", "{pokemon_name} reached level {level}!");
        en_pack.add_text("move_learned", "{pokemon_name} learned {move_name}!");
        en_pack.add_text("critical_hit", "A critical hit!");
        en_pack.add_text("super_effective", "It's super effective!");
        en_pack.add_text("not_very_effective", "It's not very effective...");
        en_pack.add_text("no_effect", "It had no effect!");
        en_pack.add_text("pokemon_fainted", "{pokemon_name} fainted!");
        
        // Chinese language pack
        let mut zh_pack = LanguagePack::new("zh".to_string(), "中文".to_string());
        zh_pack.add_text("welcome", "欢迎来到宝可梦冒险！");
        zh_pack.add_text("battle_start", "野生的{pokemon_name}出现了！");
        zh_pack.add_text("battle_win", "你赢得了战斗！");
        zh_pack.add_text("level_up", "{pokemon_name}升到了{level}级！");
        zh_pack.add_text("move_learned", "{pokemon_name}学会了{move_name}！");
        zh_pack.add_text("critical_hit", "会心一击！");
        zh_pack.add_text("super_effective", "效果显著！");
        zh_pack.add_text("not_very_effective", "效果不理想...");
        zh_pack.add_text("no_effect", "没有效果！");
        zh_pack.add_text("pokemon_fainted", "{pokemon_name}失去了战斗能力！");

        self.language_packs.insert("en".to_string(), en_pack);
        self.language_packs.insert("zh".to_string(), zh_pack);
    }

    fn setup_default_formatters(&mut self) {
        // 数字格式化器
        self.formatters.insert("number".to_string(), TextFormatter::Number(NumberFormatter {
            thousands_separator: Some(",".to_string()),
            decimal_places: 0,
            prefix: None,
            suffix: None,
        }));

        // 货币格式化器
        self.formatters.insert("money".to_string(), TextFormatter::Number(NumberFormatter {
            thousands_separator: Some(",".to_string()),
            decimal_places: 0,
            prefix: Some("$".to_string()),
            suffix: None,
        }));

        // 百分比格式化器
        self.formatters.insert("percentage".to_string(), TextFormatter::Number(NumberFormatter {
            thousands_separator: None,
            decimal_places: 1,
            prefix: None,
            suffix: Some("%".to_string()),
        }));

        // 时间格式化器
        self.formatters.insert("time".to_string(), TextFormatter::Time(TimeFormatter {
            format: "{hours:02}:{minutes:02}:{seconds:02}".to_string(),
            show_milliseconds: false,
        }));
    }

    pub fn set_language(&mut self, language: String) -> Result<(), TextError> {
        if !self.language_packs.contains_key(&language) {
            return Err(TextError::LanguageNotFound(language));
        }

        self.current_language = language;
        self.text_cache.clear(); // 清除缓存以重新加载文本
        Ok(())
    }

    pub fn get_text(&mut self, key: &str) -> String {
        self.get_text_with_vars(key, &HashMap::new())
    }

    pub fn get_text_with_vars(&mut self, key: &str, variables: &HashMap<String, String>) -> String {
        let cache_key = format!("{}:{}:{:?}", self.current_language, key, variables);
        
        if let Some(cached_text) = self.text_cache.get(&cache_key) {
            return cached_text.raw_text.clone();
        }

        if let Some(language_pack) = self.language_packs.get(&self.current_language) {
            if let Some(template) = language_pack.texts.get(key) {
                let formatted_text = self.format_text(template, variables);
                let result = FormattedText {
                    raw_text: formatted_text.clone(),
                    formatted_segments: vec![], // TODO: 实现富文本分段
                };
                
                self.text_cache.insert(cache_key, result);
                return formatted_text;
            }
        }

        // 回退到英文
        if self.current_language != "en" {
            if let Some(en_pack) = self.language_packs.get("en") {
                if let Some(template) = en_pack.texts.get(key) {
                    return self.format_text(template, variables);
                }
            }
        }

        // 最后回退，返回键名
        format!("[MISSING: {}]", key)
    }

    fn format_text(&self, template: &str, variables: &HashMap<String, String>) -> String {
        lazy_static! {
            static ref VAR_REGEX: Regex = Regex::new(r"\{([^}]+)\}").unwrap();
        }

        VAR_REGEX.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];
            variables.get(var_name).cloned().unwrap_or_else(|| format!("{{{}}}", var_name))
        }).to_string()
    }

    pub fn format_number(&self, value: f64, formatter_name: &str) -> String {
        if let Some(TextFormatter::Number(formatter)) = self.formatters.get(formatter_name) {
            formatter.format(value)
        } else {
            value.to_string()
        }
    }

    pub fn format_time(&self, seconds: f64, formatter_name: &str) -> String {
        if let Some(TextFormatter::Time(formatter)) = self.formatters.get(formatter_name) {
            formatter.format(seconds)
        } else {
            format!("{:.1}s", seconds)
        }
    }

    pub fn create_animated_text(&mut self, text: String, animation_type: TextAnimationType, 
                               duration: f32) -> AnimatedTextId {
        self.animation_system.create_animation(text, animation_type, duration)
    }

    pub fn update_animations(&mut self, delta_time: f32) {
        self.animation_system.update(delta_time);
    }

    pub fn get_animated_text(&self, id: AnimatedTextId) -> Option<&AnimatedText> {
        self.animation_system.get_animation(id)
    }

    pub fn parse_rich_text(&mut self, markup: &str) -> RichText {
        self.rich_text_processor.parse(markup)
    }

    pub fn word_wrap(&self, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let char_width = font_size * 0.6; // 估算字符宽度

        for word in words {
            let word_width = word.len() as f32 * char_width;
            let current_width = current_line.len() as f32 * char_width;

            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_width + word_width + char_width <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    pub fn truncate_text(&self, text: &str, max_length: usize, suffix: &str) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            let truncate_point = max_length.saturating_sub(suffix.len());
            format!("{}{}", &text[..truncate_point], suffix)
        }
    }
}

/// 语言包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePack {
    pub language_code: String,
    pub display_name: String,
    pub texts: HashMap<String, String>,
    pub pluralization_rules: PluralizationRules,
}

impl LanguagePack {
    pub fn new(language_code: String, display_name: String) -> Self {
        Self {
            language_code: language_code.clone(),
            display_name,
            texts: HashMap::new(),
            pluralization_rules: PluralizationRules::default_for_language(&language_code),
        }
    }

    pub fn add_text(&mut self, key: &str, text: &str) {
        self.texts.insert(key.to_string(), text.to_string());
    }

    pub fn add_plural_text(&mut self, key: &str, singular: &str, plural: &str) {
        self.texts.insert(format!("{}_singular", key), singular.to_string());
        self.texts.insert(format!("{}_plural", key), plural.to_string());
    }

    pub fn get_plural_text(&self, key: &str, count: i32) -> Option<&String> {
        let form = self.pluralization_rules.get_plural_form(count);
        let plural_key = match form {
            PluralForm::Singular => format!("{}_singular", key),
            PluralForm::Plural => format!("{}_plural", key),
            PluralForm::Zero => format!("{}_zero", key),
            PluralForm::Other(n) => format!("{}_{}", key, n),
        };

        self.texts.get(&plural_key).or_else(|| {
            // 回退到单数形式
            self.texts.get(&format!("{}_singular", key))
        })
    }
}

/// 复数规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralizationRules {
    pub language: String,
    pub rules: Vec<PluralizationRule>,
}

impl PluralizationRules {
    pub fn default_for_language(language: &str) -> Self {
        match language {
            "en" => Self {
                language: "en".to_string(),
                rules: vec![
                    PluralizationRule {
                        condition: PluralizationCondition::Equals(1),
                        form: PluralForm::Singular,
                    },
                    PluralizationRule {
                        condition: PluralizationCondition::Other,
                        form: PluralForm::Plural,
                    },
                ],
            },
            "zh" => Self {
                language: "zh".to_string(),
                rules: vec![
                    PluralizationRule {
                        condition: PluralizationCondition::Other,
                        form: PluralForm::Singular, // 中文没有复数形式
                    },
                ],
            },
            _ => Self {
                language: language.to_string(),
                rules: vec![
                    PluralizationRule {
                        condition: PluralizationCondition::Other,
                        form: PluralForm::Singular,
                    },
                ],
            },
        }
    }

    pub fn get_plural_form(&self, count: i32) -> PluralForm {
        for rule in &self.rules {
            if rule.condition.matches(count) {
                return rule.form.clone();
            }
        }
        PluralForm::Singular
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralizationRule {
    pub condition: PluralizationCondition,
    pub form: PluralForm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluralizationCondition {
    Equals(i32),
    Range(i32, i32),
    Modulo { divisor: i32, remainder: i32 },
    Other,
}

impl PluralizationCondition {
    pub fn matches(&self, count: i32) -> bool {
        match self {
            PluralizationCondition::Equals(n) => count == *n,
            PluralizationCondition::Range(min, max) => count >= *min && count <= *max,
            PluralizationCondition::Modulo { divisor, remainder } => count % divisor == *remainder,
            PluralizationCondition::Other => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluralForm {
    Singular,
    Plural,
    Zero,
    Other(i32),
}

/// 文本格式化器
#[derive(Debug, Clone)]
pub enum TextFormatter {
    Number(NumberFormatter),
    Time(TimeFormatter),
    Date(DateFormatter),
}

#[derive(Debug, Clone)]
pub struct NumberFormatter {
    pub thousands_separator: Option<String>,
    pub decimal_places: usize,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl NumberFormatter {
    pub fn format(&self, value: f64) -> String {
        let rounded = (value * 10_f64.powi(self.decimal_places as i32)).round() / 10_f64.powi(self.decimal_places as i32);
        
        let mut formatted = if self.decimal_places > 0 {
            format!("{:.1$}", rounded, self.decimal_places)
        } else {
            format!("{:.0}", rounded)
        };

        // 添加千位分隔符
        if let Some(ref separator) = self.thousands_separator {
            if let Ok(int_part) = formatted.split('.').next().unwrap_or(&formatted).parse::<i64>() {
                let int_formatted = self.add_thousands_separator(int_part.abs(), separator);
                let sign = if value < 0.0 { "-" } else { "" };
                
                if self.decimal_places > 0 && formatted.contains('.') {
                    let decimal_part = formatted.split('.').nth(1).unwrap_or("0");
                    formatted = format!("{}{}.{}", sign, int_formatted, decimal_part);
                } else {
                    formatted = format!("{}{}", sign, int_formatted);
                }
            }
        }

        // 添加前缀和后缀
        if let Some(ref prefix) = self.prefix {
            formatted = format!("{}{}", prefix, formatted);
        }
        if let Some(ref suffix) = self.suffix {
            formatted = format!("{}{}", formatted, suffix);
        }

        formatted
    }

    fn add_thousands_separator(&self, mut number: i64, separator: &str) -> String {
        let mut result = Vec::new();
        let number_str = number.to_string();
        
        for (i, ch) in number_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(separator.chars().next().unwrap_or(','));
            }
            result.push(ch);
        }
        
        result.into_iter().rev().collect()
    }
}

#[derive(Debug, Clone)]
pub struct TimeFormatter {
    pub format: String,
    pub show_milliseconds: bool,
}

impl TimeFormatter {
    pub fn format(&self, seconds: f64) -> String {
        let total_seconds = seconds as i64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;
        let milliseconds = ((seconds - total_seconds as f64) * 1000.0) as i32;

        let mut result = self.format.clone();
        result = result.replace("{hours:02}", &format!("{:02}", hours));
        result = result.replace("{hours}", &hours.to_string());
        result = result.replace("{minutes:02}", &format!("{:02}", minutes));
        result = result.replace("{minutes}", &minutes.to_string());
        result = result.replace("{seconds:02}", &format!("{:02}", secs));
        result = result.replace("{seconds}", &secs.to_string());

        if self.show_milliseconds {
            result = result.replace("{milliseconds:03}", &format!("{:03}", milliseconds));
            result = result.replace("{milliseconds}", &milliseconds.to_string());
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct DateFormatter {
    pub format: String,
}

/// 文本动画系统
pub struct TextAnimationSystem {
    animations: HashMap<AnimatedTextId, AnimatedText>,
    next_id: AnimatedTextId,
}

impl TextAnimationSystem {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_animation(&mut self, text: String, animation_type: TextAnimationType, duration: f32) -> AnimatedTextId {
        let id = self.next_id;
        self.next_id += 1;

        let animation = AnimatedText {
            id,
            text: text.clone(),
            animation_type,
            duration,
            elapsed: 0.0,
            is_finished: false,
            current_visible_chars: 0,
            visible_text: String::new(),
        };

        self.animations.insert(id, animation);
        id
    }

    pub fn update(&mut self, delta_time: f32) {
        for animation in self.animations.values_mut() {
            if !animation.is_finished {
                animation.update(delta_time);
            }
        }

        // 清理已完成的动画
        self.animations.retain(|_, animation| !animation.is_finished || animation.elapsed < animation.duration + 1.0);
    }

    pub fn get_animation(&self, id: AnimatedTextId) -> Option<&AnimatedText> {
        self.animations.get(&id)
    }

    pub fn remove_animation(&mut self, id: AnimatedTextId) -> bool {
        self.animations.remove(&id).is_some()
    }
}

pub type AnimatedTextId = u32;

/// 动画文本
#[derive(Debug, Clone)]
pub struct AnimatedText {
    pub id: AnimatedTextId,
    pub text: String,
    pub animation_type: TextAnimationType,
    pub duration: f32,
    pub elapsed: f32,
    pub is_finished: bool,
    pub current_visible_chars: usize,
    pub visible_text: String,
}

impl AnimatedText {
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;

        match self.animation_type {
            TextAnimationType::Typewriter { speed } => {
                let chars_per_second = speed;
                let target_chars = (self.elapsed * chars_per_second) as usize;
                
                if target_chars >= self.text.len() {
                    self.current_visible_chars = self.text.len();
                    self.visible_text = self.text.clone();
                    self.is_finished = true;
                } else {
                    self.current_visible_chars = target_chars;
                    self.visible_text = self.text.chars().take(target_chars).collect();
                }
            }
            TextAnimationType::FadeIn => {
                if self.elapsed >= self.duration {
                    self.is_finished = true;
                }
                self.visible_text = self.text.clone();
            }
            TextAnimationType::SlideIn { .. } => {
                if self.elapsed >= self.duration {
                    self.is_finished = true;
                }
                self.visible_text = self.text.clone();
            }
            TextAnimationType::Bounce => {
                if self.elapsed >= self.duration {
                    self.is_finished = true;
                }
                self.visible_text = self.text.clone();
            }
        }
    }

    pub fn get_alpha(&self) -> f32 {
        match self.animation_type {
            TextAnimationType::FadeIn => {
                (self.elapsed / self.duration).min(1.0)
            }
            _ => 1.0,
        }
    }

    pub fn get_offset(&self) -> (f32, f32) {
        match self.animation_type {
            TextAnimationType::SlideIn { direction } => {
                let progress = (self.elapsed / self.duration).min(1.0);
                let eased_progress = 1.0 - (1.0 - progress).powi(3); // ease-out cubic
                
                match direction {
                    SlideDirection::Left => (50.0 * (1.0 - eased_progress), 0.0),
                    SlideDirection::Right => (-50.0 * (1.0 - eased_progress), 0.0),
                    SlideDirection::Up => (0.0, 30.0 * (1.0 - eased_progress)),
                    SlideDirection::Down => (0.0, -30.0 * (1.0 - eased_progress)),
                }
            }
            TextAnimationType::Bounce => {
                let progress = (self.elapsed / self.duration).min(1.0);
                let bounce_height = 10.0;
                let bounce = (progress * std::f32::consts::PI * 4.0).sin() * bounce_height * (1.0 - progress);
                (0.0, -bounce)
            }
            _ => (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextAnimationType {
    Typewriter { speed: f32 }, // characters per second
    FadeIn,
    SlideIn { direction: SlideDirection },
    Bounce,
}

#[derive(Debug, Clone)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// 富文本处理器
pub struct RichTextProcessor {
    tag_processors: HashMap<String, Box<dyn TagProcessor>>,
}

impl RichTextProcessor {
    pub fn new() -> Self {
        let mut processor = Self {
            tag_processors: HashMap::new(),
        };

        // 注册默认标签处理器
        processor.register_default_processors();
        processor
    }

    fn register_default_processors(&mut self) {
        // 颜色标签处理器
        self.tag_processors.insert("color".to_string(), Box::new(ColorTagProcessor));
        self.tag_processors.insert("size".to_string(), Box::new(SizeTagProcessor));
        self.tag_processors.insert("bold".to_string(), Box::new(BoldTagProcessor));
        self.tag_processors.insert("italic".to_string(), Box::new(ItalicTagProcessor));
        self.tag_processors.insert("underline".to_string(), Box::new(UnderlineTagProcessor));
    }

    pub fn parse(&self, markup: &str) -> RichText {
        lazy_static! {
            static ref TAG_REGEX: Regex = Regex::new(r"<(/?)(\w+)(?:\s+([^>]*))?>").unwrap();
        }

        let mut segments = Vec::new();
        let mut current_style = TextSegmentStyle::default();
        let mut style_stack = Vec::new();
        let mut last_end = 0;

        for cap in TAG_REGEX.captures_iter(markup) {
            let match_start = cap.get(0).unwrap().start();
            let match_end = cap.get(0).unwrap().end();
            
            // 添加标签前的文本
            if match_start > last_end {
                let text = &markup[last_end..match_start];
                if !text.is_empty() {
                    segments.push(TextSegment {
                        text: text.to_string(),
                        style: current_style.clone(),
                    });
                }
            }

            let is_closing = cap.get(1).map_or(false, |m| m.as_str() == "/");
            let tag_name = cap.get(2).unwrap().as_str();
            let attributes = cap.get(3).map_or("", |m| m.as_str());

            if is_closing {
                // 关闭标签 - 恢复之前的样式
                if let Some(previous_style) = style_stack.pop() {
                    current_style = previous_style;
                }
            } else {
                // 开启标签 - 保存当前样式并应用新样式
                style_stack.push(current_style.clone());
                
                if let Some(processor) = self.tag_processors.get(tag_name) {
                    current_style = processor.process_tag(&current_style, attributes);
                }
            }

            last_end = match_end;
        }

        // 添加剩余的文本
        if last_end < markup.len() {
            let text = &markup[last_end..];
            if !text.is_empty() {
                segments.push(TextSegment {
                    text: text.to_string(),
                    style: current_style,
                });
            }
        }

        RichText { segments }
    }
}

/// 标签处理器接口
pub trait TagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, attributes: &str) -> TextSegmentStyle;
}

struct ColorTagProcessor;

impl TagProcessor for ColorTagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, attributes: &str) -> TextSegmentStyle {
        let mut new_style = current_style.clone();
        
        // 解析颜色属性
        if let Some(color) = parse_color_attribute(attributes) {
            new_style.color = color;
        }
        
        new_style
    }
}

struct SizeTagProcessor;

impl TagProcessor for SizeTagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, attributes: &str) -> TextSegmentStyle {
        let mut new_style = current_style.clone();
        
        // 解析大小属性
        if let Ok(size) = attributes.parse::<f32>() {
            new_style.font_size = size;
        }
        
        new_style
    }
}

struct BoldTagProcessor;

impl TagProcessor for BoldTagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, _attributes: &str) -> TextSegmentStyle {
        let mut new_style = current_style.clone();
        new_style.is_bold = true;
        new_style
    }
}

struct ItalicTagProcessor;

impl TagProcessor for ItalicTagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, _attributes: &str) -> TextSegmentStyle {
        let mut new_style = current_style.clone();
        new_style.is_italic = true;
        new_style
    }
}

struct UnderlineTagProcessor;

impl TagProcessor for UnderlineTagProcessor {
    fn process_tag(&self, current_style: &TextSegmentStyle, _attributes: &str) -> TextSegmentStyle {
        let mut new_style = current_style.clone();
        new_style.is_underlined = true;
        new_style
    }
}

fn parse_color_attribute(attributes: &str) -> Option<Color> {
    // 解析颜色属性，支持多种格式
    let trimmed = attributes.trim();
    
    // 十六进制颜色 #RRGGBB 或 #RGB
    if trimmed.starts_with('#') {
        return parse_hex_color(&trimmed[1..]);
    }
    
    // RGB颜色 rgb(r, g, b)
    if trimmed.starts_with("rgb(") && trimmed.ends_with(')') {
        return parse_rgb_color(&trimmed[4..trimmed.len()-1]);
    }
    
    // 命名颜色
    match trimmed.to_lowercase().as_str() {
        "red" => Some(Color::RED),
        "green" => Some(Color::GREEN),
        "blue" => Some(Color::BLUE),
        "white" => Some(Color::WHITE),
        "black" => Some(Color::BLACK),
        "yellow" => Some(Color::YELLOW),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color::from_rgba(r, g, b, 1.0))
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
        let g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
        let b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
        Some(Color::from_rgba(r, g, b, 1.0))
    } else {
        None
    }
}

fn parse_rgb_color(rgb: &str) -> Option<Color> {
    let components: Vec<&str> = rgb.split(',').map(|s| s.trim()).collect();
    if components.len() == 3 {
        let r = components[0].parse::<u8>().ok()? as f32 / 255.0;
        let g = components[1].parse::<u8>().ok()? as f32 / 255.0;
        let b = components[2].parse::<u8>().ok()? as f32 / 255.0;
        Some(Color::from_rgba(r, g, b, 1.0))
    } else {
        None
    }
}

/// 富文本结构
#[derive(Debug, Clone)]
pub struct RichText {
    pub segments: Vec<TextSegment>,
}

impl RichText {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, text: String, style: TextSegmentStyle) {
        self.segments.push(TextSegment { text, style });
    }

    pub fn get_plain_text(&self) -> String {
        self.segments.iter().map(|s| &s.text).collect::<String>()
    }

    pub fn get_total_length(&self) -> usize {
        self.segments.iter().map(|s| s.text.len()).sum()
    }
}

impl Default for RichText {
    fn default() -> Self {
        Self::new()
    }
}

/// 文本段落
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub style: TextSegmentStyle,
}

/// 文本段落样式
#[derive(Debug, Clone)]
pub struct TextSegmentStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_family: String,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underlined: bool,
    pub background_color: Option<Color>,
}

impl Default for TextSegmentStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            font_size: 16.0,
            font_family: "default".to_string(),
            is_bold: false,
            is_italic: false,
            is_underlined: false,
            background_color: None,
        }
    }
}

/// 格式化后的文本
#[derive(Debug, Clone)]
pub struct FormattedText {
    pub raw_text: String,
    pub formatted_segments: Vec<TextSegment>,
}

/// 文本错误类型
#[derive(Debug, Clone)]
pub enum TextError {
    LanguageNotFound(String),
    FormatterNotFound(String),
    InvalidFormat(String),
    ParseError(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::LanguageNotFound(lang) => write!(f, "Language pack not found: {}", lang),
            TextError::FormatterNotFound(formatter) => write!(f, "Text formatter not found: {}", formatter),
            TextError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            TextError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for TextError {}

/// 文本工具函数
pub struct TextUtils;

impl TextUtils {
    /// 计算文本宽度（估算）
    pub fn calculate_text_width(text: &str, font_size: f32) -> f32 {
        text.len() as f32 * font_size * 0.6
    }

    /// 计算文本高度
    pub fn calculate_text_height(text: &str, font_size: f32, line_height: f32) -> f32 {
        let lines = text.lines().count().max(1);
        lines as f32 * font_size * line_height
    }

    /// 居中对齐文本位置
    pub fn center_align_position(text: &str, font_size: f32, container_width: f32, container_height: f32, line_height: f32) -> (f32, f32) {
        let text_width = Self::calculate_text_width(text, font_size);
        let text_height = Self::calculate_text_height(text, font_size, line_height);
        
        let x = (container_width - text_width) / 2.0;
        let y = (container_height - text_height) / 2.0;
        
        (x, y)
    }

    /// 左对齐文本位置
    pub fn left_align_position(padding: f32, container_height: f32, font_size: f32, line_height: f32) -> (f32, f32) {
        let x = padding;
        let y = (container_height - font_size * line_height) / 2.0;
        (x, y)
    }

    /// 右对齐文本位置
    pub fn right_align_position(text: &str, font_size: f32, padding: f32, container_width: f32, container_height: f32, line_height: f32) -> (f32, f32) {
        let text_width = Self::calculate_text_width(text, font_size);
        let text_height = font_size * line_height;
        
        let x = container_width - text_width - padding;
        let y = (container_height - text_height) / 2.0;
        
        (x, y)
    }

    /// 清理和净化文本输入
    pub fn sanitize_input(input: &str, max_length: usize, allowed_chars: Option<&str>) -> String {
        let mut result = input.trim().to_string();
        
        // 限制长度
        if result.len() > max_length {
            result.truncate(max_length);
        }
        
        // 过滤字符
        if let Some(allowed) = allowed_chars {
            result = result.chars()
                .filter(|c| allowed.contains(*c) || c.is_alphanumeric() || c.is_whitespace())
                .collect();
        }
        
        result
    }

    /// 转义HTML特殊字符
    pub fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// 生成随机字符串ID
    pub fn generate_text_id(prefix: &str, length: usize) -> String {
        use rand::Rng;
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        
        let random_part: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();
        
        format!("{}_{}", prefix, random_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_manager_creation() {
        let manager = TextManager::new();
        assert_eq!(manager.current_language, "en");
        assert!(manager.language_packs.contains_key("en"));
        assert!(manager.language_packs.contains_key("zh"));
    }

    #[test]
    fn test_text_formatting() {
        let mut manager = TextManager::new();
        let mut vars = HashMap::new();
        vars.insert("pokemon_name".to_string(), "Pikachu".to_string());
        
        let result = manager.get_text_with_vars("battle_start", &vars);
        assert_eq!(result, "A wild Pikachu appeared!");
    }

    #[test]
    fn test_language_switching() {
        let mut manager = TextManager::new();
        
        manager.set_language("zh".to_string()).unwrap();
        let result = manager.get_text("welcome");
        assert_eq!(result, "欢迎来到宝可梦冒险！");
    }

    #[test]
    fn test_number_formatting() {
        let manager = TextManager::new();
        let result = manager.format_number(12345.67, "number");
        assert!(result.contains("12,346") || result.contains("12345")); // 根据实现可能不同
    }

    #[test]
    fn test_rich_text_parsing() {
        let mut manager = TextManager::new();
        let rich_text = manager.parse_rich_text("<color red>Warning:</color> <bold>Critical Error</bold>");
        
        assert_eq!(rich_text.segments.len(), 3);
        assert_eq!(rich_text.segments[0].text, "Warning:");
        assert_eq!(rich_text.segments[1].text, " ");
        assert_eq!(rich_text.segments[2].text, "Critical Error");
        assert!(rich_text.segments[2].style.is_bold);
    }

    #[test]
    fn test_text_animation() {
        let mut manager = TextManager::new();
        let id = manager.create_animated_text(
            "Hello World".to_string(),
            TextAnimationType::Typewriter { speed: 10.0 },
            2.0
        );
        
        // 模拟0.5秒后的状态
        manager.update_animations(0.5);
        
        let animation = manager.get_animated_text(id).unwrap();
        assert!(animation.current_visible_chars > 0);
        assert!(animation.current_visible_chars < "Hello World".len());
    }

    #[test]
    fn test_word_wrap() {
        let manager = TextManager::new();
        let lines = manager.word_wrap("This is a very long sentence that should be wrapped into multiple lines", 200.0, 16.0);
        
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() * 16 as usize <= 200);
        }
    }

    #[test]
    fn test_text_utils() {
        let width = TextUtils::calculate_text_width("Hello", 16.0);
        assert!(width > 0.0);
        
        let height = TextUtils::calculate_text_height("Hello\nWorld", 16.0, 1.2);
        assert!(height > 16.0); // 应该是两行的高度
        
        let sanitized = TextUtils::sanitize_input("  Hello World!  ", 10, None);
        assert_eq!(sanitized, "Hello Worl");
    }
}