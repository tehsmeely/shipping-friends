use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};
use bevy_ecs_ldtk::assets::LdtkProject;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HandleMap<ImageKey>>();
    app.init_resource::<HandleMap<ImageKey>>();

    app.register_type::<HandleMap<AtlasLayoutKey>>();
    app.init_resource::<HandleMap<AtlasLayoutKey>>();

    app.register_type::<HandleMap<SfxKey>>();
    app.init_resource::<HandleMap<SfxKey>>();

    app.register_type::<HandleMap<SoundtrackKey>>();
    app.init_resource::<HandleMap<SoundtrackKey>>();

    app.register_type::<HandleMap<LdtkKey>>();
    app.init_resource::<HandleMap<LdtkKey>>();
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ImageKey {
    Ducky,
    BasicTileSet,
    BulkLoadVessel,
    LoadingCrane,
    OverlayMarker,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AtlasLayoutKey {
    BasicTileSet,
    BulkLoadVessel,
    LoadingCrane,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum LdtkKey {
    Main,
}

impl AssetKey for ImageKey {
    type Asset = Image;
}
impl AssetKey for AtlasLayoutKey {
    type Asset = TextureAtlasLayout;
}

impl AssetKey for LdtkKey {
    type Asset = LdtkProject;
}

impl FromWorld for HandleMap<ImageKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                ImageKey::Ducky,
                asset_server.load_with_settings(
                    "images/ducky.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::BasicTileSet,
                asset_server.load_with_settings(
                    "images/basic_tiles.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::BulkLoadVessel,
                asset_server.load_with_settings(
                    "images/bulk_load_vessel.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::LoadingCrane,
                asset_server.load_with_settings(
                    "images/ship_loading_crane_smol.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::OverlayMarker,
                asset_server.load_with_settings(
                    "images/overlay_marker.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
        ]
        .into()
    }
}

impl FromWorld for HandleMap<AtlasLayoutKey> {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        [
            (
                AtlasLayoutKey::BasicTileSet,
                texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(100),
                    2,
                    1,
                    None,
                    None,
                )),
            ),
            (
                AtlasLayoutKey::BulkLoadVessel,
                texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::new(200, 80),
                    9,
                    1,
                    None,
                    None,
                )),
            ),
            (
                AtlasLayoutKey::LoadingCrane,
                texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(100),
                    6,
                    1,
                    None,
                    None,
                )),
            ),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SfxKey {
    ButtonHover,
    ButtonPress,
    Step1,
    Step2,
    Step3,
    Step4,
}

impl AssetKey for SfxKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SfxKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SfxKey::ButtonHover,
                asset_server.load("audio/sfx/button_hover.ogg"),
            ),
            (
                SfxKey::ButtonPress,
                asset_server.load("audio/sfx/button_press.ogg"),
            ),
            (SfxKey::Step1, asset_server.load("audio/sfx/step1.ogg")),
            (SfxKey::Step2, asset_server.load("audio/sfx/step2.ogg")),
            (SfxKey::Step3, asset_server.load("audio/sfx/step3.ogg")),
            (SfxKey::Step4, asset_server.load("audio/sfx/step4.ogg")),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SoundtrackKey {
    Menus,
    Gameplay,
}

impl AssetKey for SoundtrackKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SoundtrackKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SoundtrackKey::Menus,
                asset_server.load("audio/soundtracks/Monkeys Spinning Monkeys.ogg"),
            ),
            (
                SoundtrackKey::Gameplay,
                asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
            ),
        ]
        .into()
    }
}

impl FromWorld for HandleMap<LdtkKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [(LdtkKey::Main, asset_server.load("levels/maps.ldtk"))].into()
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);

impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<K: AssetKey> HandleMap<K> {
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.values()
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }
}
