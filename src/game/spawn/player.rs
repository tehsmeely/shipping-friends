//! Spawn the player.

use crate::game::assets::AtlasLayoutKey;
use crate::game::camera::CameraFollow;
use crate::game::controls::setup_movement_controls;
use crate::game::movement::{AutoFacingTurn, AutoGridPlacement, Facing};
use crate::game::spawn::level::GRID_SIZE_V;
use crate::{
    game::assets::{HandleMap, ImageKey},
    screen::Screen,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::SquareDirection;
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
    let coords = GridCoords::new(3, 3);
    let translation = bevy_ecs_ldtk::utils::grid_coords_to_translation(coords, GRID_SIZE_V);
    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            texture: image_handles[&ImageKey::BulkLoadVessel].clone(),
            transform: Transform::from_translation(translation.extend(2.0))
                .with_scale(Vec3::splat(0.25)),
            ..Default::default()
        },
        TextureAtlas { layout, index: 0 },
        StateScoped(Screen::Playing),
        coords,
        AutoGridPlacement,
        AutoFacingTurn,
        Facing::East,
        setup_movement_controls(),
        CameraFollow { threshold: 120.0 },
    ));
}
