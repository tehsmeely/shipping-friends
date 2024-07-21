//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use crate::game::controls::{CameraAction, PlayerAction};
use crate::game::game_ui::{CycleNum, GlobalTurnLock, TurnAction, TurnActions};
use crate::game::spawn::level::TilemapOffset;
use crate::AppSet;
use bevy::reflect::{ApplyError, ReflectMut, ReflectOwned, ReflectRef, TypeInfo};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::helpers::square_grid::neighbors::SquareDirection;
use bevy_ecs_tilemap::map::TilemapType;
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::action_state::ActionState;
use std::any::Any;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(AutoTilePosPlacement, Facing)>();
    app.add_systems(
        Update,
        (auto_tile_pos, handle_player_movement, apply_facing),
    );
    app.observe(apply_turn_actions);
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Automatically place entities with this property in the transform with their TilePos
pub struct AutoTilePosPlacement;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
/// Cardinal directions
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn to_offset(&self) -> IVec2 {
        match self {
            Self::East => IVec2::new(1, 0),
            Self::North => IVec2::new(0, 1),
            Self::West => IVec2::new(-1, 0),
            Self::South => IVec2::new(0, -1),
        }
    }

    fn rotate_cw(&self) -> Self {
        match self {
            Self::East => Self::South,
            Self::North => Self::East,
            Self::West => Self::North,
            Self::South => Self::West,
        }
    }
    fn rotate_acw(&self) -> Self {
        match self {
            Self::East => Self::North,
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
        }
    }

    pub fn rotate(&mut self, clockwise: bool) {
        *self = match clockwise {
            true => self.rotate_cw(),
            false => self.rotate_acw(),
        }
    }
}

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

fn apply_facing(mut query: Query<(&Facing, &mut Transform), Changed<Facing>>) {
    for (facing, mut transform) in &mut query {
        match facing {
            Facing::East => {
                transform.rotation = Quat::from_rotation_z(0.0);
            }
            Facing::West => {
                transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
            }
            Facing::South => {
                transform.rotation = Quat::from_rotation_z(-std::f32::consts::PI / 2.0);
            }
            Facing::North => {
                transform.rotation = Quat::from_rotation_z(std::f32::consts::PI / 2.0);
            }
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

#[derive(Event, Debug)]
pub struct ApplyTurnActions(pub TurnActions);

fn apply_turn_actions(
    trigger: Trigger<ApplyTurnActions>,
    mut player_query: Query<(&mut Facing, &mut TilePos)>,
    mut global_turn_lock: ResMut<GlobalTurnLock>,
    mut cycle_num: ResMut<CycleNum>,
) {
    for (mut facing, mut tilepos) in player_query.iter_mut() {
        let turn_actions = trigger.event();
        for action in turn_actions.0.clone().0.iter() {
            if let Some(action) = action {
                action.apply(&mut facing, &mut tilepos);
            }
        }
        global_turn_lock.unlock();
        cycle_num.increment();
    }
}
