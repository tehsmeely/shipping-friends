use crate::game::controls::CameraAction;
use crate::game::spawn::player::Player;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (zoom_camera, camera_follow));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraFollow {
    pub threshold: f32,
}

fn zoom_camera(
    mut camera_query: Query<
        (&mut OrthographicProjection, &ActionState<CameraAction>),
        With<Camera>,
    >,
) {
    for (mut camera, inputs) in &mut camera_query {
        let zoom_delta = inputs.value(&CameraAction::Zoom) * 0.05;
        camera.scale *= 1.0 - zoom_delta;
    }
}

fn camera_follow(
    mut camera_query: Query<(&mut Transform), (With<Camera>, Without<CameraFollow>)>,
    follower_query: Query<(&Transform, &CameraFollow)>,
) {
    for (follower_transform, camera_follow) in &follower_query {
        for mut camera_transform in &mut camera_query {
            let target = follower_transform.translation.truncate();
            let camera = camera_transform.translation.truncate();
            let distance = target.distance(camera);
            if distance > camera_follow.threshold {
                camera_transform.translation = target.extend(follower_transform.translation.z);
            }
        }
    }
}
