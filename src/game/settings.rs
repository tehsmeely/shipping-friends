use bevy::prelude::Resource;

#[derive(Debug, Clone, Resource)]
pub struct Settings {
    pub volume: f32,
    pub music: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 0.5,
            music: false,
        }
    }
}
