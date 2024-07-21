use crate::game::audio::soundtrack::MusicPlayer;
use crate::game::settings::Settings;
use bevy::audio::Volume;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (adjust_volume, enable_or_disable));
}

fn adjust_volume(settings: Res<Settings>, mut playback_settings: Query<&mut AudioSink>) {
    for mut sink in playback_settings.iter_mut() {
        sink.set_volume(settings.volume);
    }
}

fn enable_or_disable(
    settings: Res<Settings>,
    mut playback_settings: Query<&mut AudioSink, With<MusicPlayer>>,
) {
    for mut sink in playback_settings.iter_mut() {
        if sink.is_paused() && settings.music {
            sink.play();
        } else if !sink.is_paused() && !settings.music {
            sink.pause();
        }
    }
}
