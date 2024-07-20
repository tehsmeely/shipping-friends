//! Spawn the player.

use crate::game::assets::AtlasLayoutKey;
use crate::game::camera::CameraFollow;
use crate::game::controls::setup_movement_controls;
use crate::game::movement::AutoTilePosPlacement;
use crate::game::spawn::level::get_start;
use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{HandleMap, ImageKey},
        movement::{Movement, MovementController, WrapWithinWindow},
    },
    screen::Screen,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    image_handles: Res<HandleMap<ImageKey>>,
    texture_atlas_layouts: Res<HandleMap<AtlasLayoutKey>>,
) {
    let layout = texture_atlas_layouts[&AtlasLayoutKey::BulkLoadVessel].clone();
    let start = get_start();
    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            texture: image_handles[&ImageKey::BulkLoadVessel].clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0))
                .with_scale(Vec3::splat(0.5)),
            ..Default::default()
        },
        TextureAtlas { layout, index: 0 },
        StateScoped(Screen::Playing),
        TilePos {
            x: start.0,
            y: start.1,
        },
        AutoTilePosPlacement,
        setup_movement_controls(),
        CameraFollow { threshold: 300.0 },
    ));
}
