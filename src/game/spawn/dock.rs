use crate::game::movement::Facing;
use crate::game::spawn::level::GRID_SIZE_V;
use crate::game::spawn::player::Player;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<DockBundle>("Crane_N");
    app.register_ldtk_entity::<DockBundle>("Crane_E");
    app.register_ldtk_entity::<DockBundle>("Crane_S");
    app.register_ldtk_entity::<DockBundle>("Crane_W");
    app.add_systems(Update, fix_dock_grid_coord_positions);
}
#[derive(Default, Bundle, LdtkEntity)]
struct DockBundle {
    dock: Dock,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    #[from_entity_instance]
    facing: Facing,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Dock;
fn fix_dock_grid_coord_positions(
    mut docks: Query<(&mut Transform, &Facing, &GridCoords), With<Dock>>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(_) = level_event {
            for (mut transform, facing, grid_coords) in &mut docks {
                // TODO: Offset by 10.0 away form facing
                let offset = facing.to_offset().as_vec2() * -10.0;
                transform.translation =
                    (bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, GRID_SIZE_V)
                        + offset)
                        // Z should be above player boat
                        .extend(3.0);
            }
        }
    }
}

impl From<&EntityInstance> for Facing {
    fn from(entity_instance: &EntityInstance) -> Self {
        // I *should* be using the enum in EntityInstance.field_instances to
        // determine the facing, but using the name is just as good
        match entity_instance.identifier.as_str() {
            "Crane_N" => Self::North,
            "Crane_E" => Self::East,
            "Crane_S" => Self::South,
            "Crane_W" => Self::West,
            _ => panic!("Unexpected entity instance: {:?}", entity_instance),
        }
    }
}
