//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use crate::game::controls::{CameraAction, PlayerAction};
use crate::game::spawn::level::TilemapOffset;
use crate::AppSet;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::map::TilemapType;
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::action_state::ActionState;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        record_movement_controller.in_set(AppSet::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<(Movement, WrapWithinWindow)>();
    app.add_systems(
        Update,
        (apply_movement, wrap_within_window)
            .chain()
            .in_set(AppSet::Update),
    );

    app.register_type::<AutoTilePosPlacement>();
    app.add_systems(Update, (auto_tile_pos, handle_player_movement));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
/// Automatically place entities with this property in the transform with their TilePos
pub struct AutoTilePosPlacement;

fn auto_tile_pos(
    mut query: Query<(&TilePos, &mut Transform), (With<AutoTilePosPlacement>, Changed<TilePos>)>,
    map_query: Query<(&TilemapGridSize, &TilemapType)>,
    tilemap_offset: Res<TilemapOffset>,
) {
    if let Ok((tm_grid_size, tm_type)) = map_query.get_single() {
        for (tile_pos, mut transform) in &mut query {
            let world_pos = tile_pos.center_in_world(tm_grid_size, tm_type);
            let world_pos = world_pos + tilemap_offset.0.translation.truncate();
            transform.translation = world_pos.extend(transform.translation.z);
        }
    }
}

fn handle_player_movement(mut camera_query: Query<(&mut TilePos, &ActionState<PlayerAction>)>) {
    for (mut tilepos, inputs) in &mut camera_query {
        if let Some(action) = PlayerAction::ALL
            .iter()
            .find(|action| inputs.just_pressed(action))
        {
            match action {
                PlayerAction::Up => {
                    tilepos.y += 1;
                }
                PlayerAction::Down => {
                    tilepos.y -= 1;
                }
                PlayerAction::Left => {
                    tilepos.x -= 1;
                }
                PlayerAction::Right => {
                    tilepos.x += 1;
                }
            }
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController(pub Vec2);

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 = intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    /// Note that physics engines may use different unit/pixel ratios.
    pub speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &Movement, &mut Transform)>,
) {
    for (controller, movement, mut transform) in &mut movement_query {
        let velocity = movement.speed * controller.0;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WrapWithinWindow;

fn wrap_within_window(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<WrapWithinWindow>>,
) {
    let size = window_query.single().size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
