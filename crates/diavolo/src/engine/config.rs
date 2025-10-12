use dialogue::{Text, TypingSpeedFactor};
use language_tags::LanguageTag;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub language: LanguageTag,
    pub typing: TypingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: LanguageTag::parse("en").unwrap(),
            typing: TypingConfig::default(),
        }
    }
}

impl Config {
    pub fn effective_typing_speed(&self, text: &Text) -> TypingSpeed {
        self.typing.effective_speed(text, &self.language)
    }
}

#[derive(Debug, Clone)]
pub struct TypingConfig {
    pub speed_factor: TypingSpeedFactor,
    pub language_speeds: HashMap<LanguageTag, TypingSpeed>,
    pub auto_adjust: bool,
    pub start_delay: Duration,
    pub fast_forward_factor: TypingSpeedFactor,
}

impl Default for TypingConfig {
    fn default() -> Self {
        let language_speeds: HashMap<LanguageTag, TypingSpeed> = [
            ("en", 30.0),
            ("es", 28.0),
            ("fr", 28.0),
            ("de", 28.0),
            ("it", 28.0),
            ("ja", 20.0),
            ("ko", 18.0),
            ("zh-CN", 15.0),
            ("zh-TW", 15.0),
            ("ru", 25.0),
        ]
        .iter()
        .map(|(lang, speed)| (LanguageTag::parse(lang).unwrap(), TypingSpeed::from(*speed)))
        .collect();

        Self {
            speed_factor: TypingSpeedFactor::from(1.0),
            fast_forward_factor: TypingSpeedFactor::from(1.0),
            language_speeds,
            auto_adjust: true,
            start_delay: std::time::Duration::from_millis(300),
        }
    }
}

impl TypingConfig {
    pub fn effective_speed(&self, text: &Text, language: &LanguageTag) -> TypingSpeed {
        let base_speed = self.speed_for_language(language);

        if !self.auto_adjust {
            return base_speed;
        }

        let complexity_factor = self.calculate_complexity(text);
        (*base_speed * complexity_factor).into()
    }

    fn speed_for_language(&self, language: &LanguageTag) -> TypingSpeed {
        self.language_speeds
            .get(language)
            .cloned()
            .unwrap_or_default()
    }

    fn calculate_complexity(&self, text: &str) -> f32 {
        let total_chars = text.chars().count() as f32;
        if total_chars == 0.0 {
            return 1.0;
        }

        let mut ascii_count = 0.0;
        let mut hiragana_count = 0.0;
        let mut katakana_count = 0.0;
        let mut kanji_count = 0.0;
        let mut other_count = 0.0;

        for ch in text.chars() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' => ascii_count += 1.0,
                '\u{3040}'..='\u{309F}' => hiragana_count += 1.0,
                '\u{30A0}'..='\u{30FF}' => katakana_count += 1.0,
                '\u{4E00}'..='\u{9FFF}' => kanji_count += 1.0,
                _ => other_count += 1.0,
            }
        }

        let weighted_complexity = (ascii_count * 1.0
            + hiragana_count * 0.8
            + katakana_count * 0.8
            + kanji_count * 0.6
            + other_count * 0.9)
            / total_chars;

        weighted_complexity
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TypingSpeed(f32);

impl std::ops::Deref for TypingSpeed {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for TypingSpeed {
    fn default() -> Self {
        Self(25.0)
    }
}

impl From<f32> for TypingSpeed {
    fn from(value: f32) -> Self {
        TypingSpeed(value)
    }
}
