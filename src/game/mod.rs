//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod assets;
pub mod audio;
mod camera;
pub mod controls;
mod game_ui;
mod movement;
pub mod settings;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        assets::plugin,
        movement::plugin,
        spawn::plugin,
        controls::plugin,
        camera::plugin,
        game_ui::plugin,
    ));
}
