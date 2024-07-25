//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use crate::game::controls::{CameraAction, PlayerAction};
use crate::game::game_ui::{CycleNum, GlobalTurnLock, TurnAction, TurnActions};
use crate::game::spawn::level::{LevelWalls, TilemapOffset, GRID_SIZE_V};
use crate::AppSet;
use bevy::reflect::{ApplyError, ReflectMut, ReflectOwned, ReflectRef, TypeInfo};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::SquareDirection;
use bevy_ecs_tilemap::map::TilemapType;
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::action_state::ActionState;
use std::any::Any;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(AutoGridPlacement, AutoFacingTurn, Facing)>();
    app.add_systems(
        Update,
        (auto_tile_pos, handle_player_movement, apply_facing),
    );
    app.observe(apply_turn_actions);
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Automatically place entities with this property in the transform with their TilePos
pub struct AutoGridPlacement;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Automatically face entities with this property to match their Facing property
pub struct AutoFacingTurn;

#[derive(Component, Default, Debug, Reflect, Clone, Copy)]
#[reflect(Component)]
/// Cardinal directions
pub enum Facing {
    North,
    #[default]
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
    mut query: Query<(&GridCoords, &mut Transform), (With<AutoGridPlacement>, Changed<GridCoords>)>,
) {
    for (grid_coords, mut transform) in &mut query {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, GRID_SIZE_V)
                .extend(transform.translation.z);
    }
}

fn apply_facing(
    mut query: Query<(&Facing, &mut Transform), (Changed<Facing>, With<AutoFacingTurn>)>,
) {
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

fn handle_player_movement(
    mut camera_query: Query<(&mut GridCoords, &ActionState<PlayerAction>)>,

    level_walls: Res<LevelWalls>,
) {
    for (mut orig_grid_coords, inputs) in &mut camera_query {
        if let Some(action) = PlayerAction::ALL
            .iter()
            .find(|action| inputs.just_pressed(action))
        {
            let mut grid_coords = orig_grid_coords.clone();
            match action {
                PlayerAction::Up => {
                    grid_coords.y += 1;
                }
                PlayerAction::Down => {
                    grid_coords.y -= 1;
                }
                PlayerAction::Left => {
                    grid_coords.x -= 1;
                }
                PlayerAction::Right => {
                    grid_coords.x += 1;
                }
            }
            if !level_walls.in_wall(&grid_coords) {
                *orig_grid_coords = grid_coords;
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct ApplyTurnActions(pub TurnActions);

fn apply_turn_actions(
    trigger: Trigger<ApplyTurnActions>,
    mut player_query: Query<(&mut Facing, &mut GridCoords)>,
    mut global_turn_lock: ResMut<GlobalTurnLock>,
    mut cycle_num: ResMut<CycleNum>,
    level_walls: Res<LevelWalls>,
) {
    for (mut facing, mut coords) in player_query.iter_mut() {
        let turn_actions = trigger.event();
        for action in turn_actions.0.clone().0.iter() {
            if let Some(action) = action {
                let (new_facing, new_coords) = action.apply(&facing, &coords);
                if !level_walls.in_wall(&new_coords) {
                    *facing = new_facing;
                    *coords = new_coords;
                }
                // TODO: Should we stop taking actions if you hit a wall, or continue?
            }
        }
        global_turn_lock.unlock();
        cycle_num.increment();
    }
}
