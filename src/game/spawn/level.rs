//! Spawn the main level by triggering other observers.

use super::player::SpawnPlayer;
use crate::game::assets::{HandleMap, ImageKey, LdtkKey};
use crate::screen::Screen;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.init_resource::<LevelWalls>();
    app.register_type::<LevelWalls>();

    app.insert_resource(LevelSelection::index(0))
        .register_ldtk_int_cell::<WallBundle>(1);

    app.add_systems(Update, cache_wall_locations);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Wall;

#[derive(Bundle, LdtkIntCell, Debug, Clone, Default, Reflect)]
pub struct WallBundle {
    wall: Wall,
}

pub const GRID_SIZE: i32 = 50;
pub const GRID_SIZE_V: IVec2 = IVec2::new(GRID_SIZE, GRID_SIZE);

#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct TilemapOffset(pub Transform);

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut ldtk_handles: Res<HandleMap<LdtkKey>>,
) {
    commands.trigger(SpawnPlayer);

    commands
        .spawn(LdtkWorldBundle {
            ldtk_handle: ldtk_handles[&LdtkKey::Main].clone(),
            ..default()
        })
        .insert((Name::new("LdtkLevel"), StateScoped(Screen::Playing)));
}

fn _spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    image_handles: Res<HandleMap<ImageKey>>,
) {
    commands.trigger(SpawnPlayer);

    let map_size = TilemapSize { x: 30, y: 17 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    let tile_parent = commands.spawn(Name::new("Tiles")).id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let (idx, tile_name) = (1, "whomp"); //_get_tile_idx(x, y);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(idx),
                    ..Default::default()
                })
                .insert(Name::new(format!("Tile-{}", tile_name)))
                .set_parent(tile_parent)
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 100.0, y: 100.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let tilemap_texture = image_handles.get(&ImageKey::BasicTileSet).unwrap().clone();

    let tilemap_offset = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0);

    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tilemap_texture),
            tile_size,
            transform: tilemap_offset,
            ..Default::default()
        })
        .insert(StateScoped(Screen::Playing));

    commands.insert_resource(TilemapOffset(tilemap_offset));
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single())
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");

            let wall_locations = walls.iter().copied().collect();

            let new_level_walls = LevelWalls {
                wall_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_walls = new_level_walls;
        }
    }
}
