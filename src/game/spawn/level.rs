//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use super::player::SpawnPlayer;
use crate::game::assets::{HandleMap, ImageKey};
use crate::screen::Screen;
use bevy_ecs_tilemap::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.init_resource::<TilemapOffset>();
    app.register_type::<TilemapOffset>();
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct TilemapOffset(pub Transform);

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
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
            let (idx, tile_name) = get_tile_idx(x, y);
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
    //let tilemap_texture = asset_server.load("images/basic_tiles.png");

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

const MAP: [&str; 17] = [
    "##############################",
    "######...............#########",
    "######...............#########",
    "######...............#########",
    "######...............#########",
    "######...########....#########",
    "######...########....#########",
    "#################....#########",
    "#################....#########",
    "#########......S.....#########",
    "#########.....################",
    "#########.....################",
    "#########...................##",
    "#########...................##",
    "##############################",
    "##############################",
    "##############################",
];

// This takes the place of actual map loading
fn get_tile_idx(x: u32, y: u32) -> (u32, &'static str) {
    let row = MAP[y as usize];
    let c = row.chars().nth(x as usize).unwrap();
    match c {
        '#' => (0, "land"),
        '.' => (1, "sea"),
        'S' => (1, "sea+spawn"),
        _ => (0, "unknown"),
    }
}

pub fn get_start() -> (u32, u32) {
    for (y, row) in MAP.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == 'S' {
                return (x as u32, y as u32);
            }
        }
    }
    panic!("No start position found in map");
}
