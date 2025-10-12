use dialogue::TypingSpeed;
use isolang::Language;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub(crate) language: Language,
    pub(crate) typing: Typing,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::from_639_1("en").unwrap(),
            typing: Typing::default(),
        }
    }
}

impl Config {
    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn set_language(&mut self, language: impl Into<isolang::Language>) {
        self.language = language.into();
    }

    pub fn typing(&self) -> &Typing {
        &self.typing
    }

    pub fn typing_mut(&mut self) -> &mut Typing {
        &mut self.typing
    }
}

#[derive(Debug)]
pub struct Typing {
    pub(crate) speed: TypingSpeed,
    pub(crate) start_delay: Duration,
    pub(crate) fast_forward_speed: f32,
}

impl Default for Typing {
    fn default() -> Self {
        Self {
            speed: TypingSpeed::from(20.0),
            start_delay: std::time::Duration::from_millis(300),
            fast_forward_speed: 1.0,
        }
    }
}

impl Typing {
    pub fn speed(&self) -> &TypingSpeed {
        &self.speed
    }

    pub fn speed_mut(&mut self) -> &mut TypingSpeed {
        &mut self.speed
    }

    pub fn start_delay(&self) -> &Duration {
        &self.start_delay
    }

    pub fn start_delay_mut(&mut self) -> &mut Duration {
        &mut self.start_delay
    }

    pub fn fast_forward_speed(&self) -> f32 {
        self.fast_forward_speed
    }

    pub fn fast_forward_speed_mut(&mut self) -> &mut f32 {
        &mut self.fast_forward_speed
    }
}

