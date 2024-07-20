use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<CameraAction>::default());
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    app.register_type::<(CameraAction, PlayerAction)>();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum CameraAction {
    Zoom,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Up,
    Right,
    Down,
    Left,
}
impl PlayerAction {
    pub const ALL: [PlayerAction; 4] = [
        PlayerAction::Up,
        PlayerAction::Right,
        PlayerAction::Down,
        PlayerAction::Left,
    ];
}

pub fn setup_camera_controls() -> InputManagerBundle<CameraAction> {
    let mut input_map = InputMap::default();
    input_map.insert(CameraAction::Zoom, SingleAxis::mouse_wheel_y());
    InputManagerBundle::with_map(input_map)
}
pub fn setup_movement_controls() -> InputManagerBundle<PlayerAction> {
    let mut input_map = InputMap::default();
    input_map.insert(PlayerAction::Up, KeyCode::KeyW);
    input_map.insert(PlayerAction::Right, KeyCode::KeyD);
    input_map.insert(PlayerAction::Down, KeyCode::KeyS);
    input_map.insert(PlayerAction::Left, KeyCode::KeyA);
    InputManagerBundle::with_map(input_map)
}
