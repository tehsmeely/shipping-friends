//! A credits screen that can be accessed from the title screen.

use super::Screen;
use crate::{
    game::{assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack},
    ui::prelude::*,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), enter_credits);
    app.add_systems(OnExit(Screen::Settings), exit_credits);

    app.add_systems(Update, update_settings.run_if(in_state(Screen::Settings)));
    app.register_type::<SettingsAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum SettingsAction {
    Back,
}

fn enter_credits(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Menus));
}

fn exit_credits(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn update_settings(
    mut next_screen: ResMut<NextState<Screen>>,
    mut contexts: EguiContexts,
    mut settings: ResMut<crate::game::settings::Settings>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.vertical(|ui| {
                    ui.heading("Settings");

                    ui.horizontal(|ui| {
                        ui.label("Volume");
                        ui.add(egui::Slider::new(&mut settings.volume, 0.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Music");
                        ui.add(egui::Checkbox::new(&mut settings.music, ""));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Fullscreen");
                        ui.checkbox(&mut true, "");
                    });

                    if ui.button("Back").clicked() {
                        next_screen.set(Screen::Title);
                    }
                });
            },
        );
    });
}
