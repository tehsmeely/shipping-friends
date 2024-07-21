use crate::game::movement::{ApplyTurnActions, Facing};
use crate::screen::Screen;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_egui::egui::{vec2, Color32, Frame, Id, WidgetText};
use bevy_egui::{egui, EguiContexts};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), setup);
    app.add_systems(Update, (do_ui).run_if(in_state(Screen::Playing)));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnAction {
    Forward,
    RotateClockwise,
    RotateAntiClockwise,
}

fn safe_u32_add(a: u32, b: i32) -> u32 {
    if b.is_negative() {
        a.saturating_sub(b.wrapping_abs() as u32)
    } else {
        a.saturating_add(b as u32)
    }
}
impl TurnAction {
    pub fn apply(&self, facing: &mut Facing, tilepos: &mut TilePos) {
        match self {
            TurnAction::Forward => {
                let offset = facing.to_offset();
                tilepos.x = safe_u32_add(tilepos.x, offset.x);
                tilepos.y = safe_u32_add(tilepos.y, offset.y);
            }
            TurnAction::RotateClockwise => {
                facing.rotate(true);
            }
            TurnAction::RotateAntiClockwise => {
                facing.rotate(false);
            }
        }
    }
}

impl std::fmt::Display for TurnAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TurnAction::Forward => write!(f, "Forward"),
            TurnAction::RotateClockwise => write!(f, "Rotate Clockwise"),
            TurnAction::RotateAntiClockwise => write!(f, "Rotate Anti-Clockwise"),
        }
    }
}

impl Into<WidgetText> for TurnAction {
    fn into(self) -> WidgetText {
        WidgetText::from(self.to_string())
    }
}

#[derive(Clone, Debug, Resource)]
pub struct CycleStore {
    action_column: Vec<TurnAction>,
    available_actions: Vec<TurnAction>,
}

impl CycleStore {
    pub fn new() -> Self {
        Self {
            action_column: vec![TurnAction::Forward],
            available_actions: vec![
                TurnAction::Forward,
                TurnAction::RotateClockwise,
                TurnAction::RotateAntiClockwise,
            ],
        }
    }

    fn columns(&self) -> Vec<Vec<TurnAction>> {
        vec![self.action_column.clone(), self.available_actions.clone()]
    }

    fn get_column_mut(&mut self, idx: usize) -> &mut Vec<TurnAction> {
        match idx {
            0 => &mut self.action_column,
            1 => &mut self.available_actions,
            _ => panic!("Invalid column index"),
        }
    }

    fn next(&mut self) {
        self.action_column.push(self.available_actions.remove(0));
        if self.available_actions.is_empty() {
            self.available_actions = vec![
                TurnAction::Forward,
                TurnAction::RotateClockwise,
                TurnAction::RotateAntiClockwise,
            ];
        }
    }

    fn clear(&mut self) {
        self.available_actions.extend(self.action_column.drain(..));
    }

    fn current(&self) -> TurnAction {
        self.action_column[0]
    }

    fn consume(&mut self) -> TurnAction {
        self.action_column.remove(0)
    }
}

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(CycleStore::new());
}

fn do_ui(mut commands: Commands, mut contexts: EguiContexts, mut cycle_store: ResMut<CycleStore>) {
    egui::Window::new("Game UI")
        .anchor(egui::Align2::RIGHT_BOTTOM, vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            // If there is a drop, store the location of the item being dragged, and the destination for the drop.
            let mut from = None;
            let mut to = None;

            ui.columns(2, |uis| {
                for (col_idx, column) in cycle_store.columns().into_iter().enumerate() {
                    let ui = &mut uis[col_idx];

                    let frame = Frame::default().inner_margin(4.0);

                    let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                        ui.set_min_size(vec2(64.0, 100.0));
                        for (row_idx, item) in column.iter().enumerate() {
                            let item_id = Id::new(("my_drag_and_drop_demo", col_idx, row_idx));
                            let item_location = Location {
                                col: col_idx,
                                row: row_idx,
                            };
                            let response = ui
                                .dnd_drag_source(item_id, item_location, |ui| {
                                    ui.label(*item);
                                })
                                .response;

                            // Detect drops onto this item:
                            if let (Some(pointer), Some(hovered_payload)) = (
                                ui.input(|i| i.pointer.interact_pos()),
                                response.dnd_hover_payload::<Location>(),
                            ) {
                                let rect = response.rect;

                                // Preview insertion:
                                let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                                let insert_row_idx = if *hovered_payload == item_location {
                                    // We are dragged onto ourselves
                                    ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                    row_idx
                                } else if pointer.y < rect.center().y {
                                    // Above us
                                    ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                    row_idx
                                } else {
                                    // Below us
                                    ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                    row_idx + 1
                                };

                                if let Some(dragged_payload) = response.dnd_release_payload() {
                                    // The user dropped onto this item.
                                    from = Some(dragged_payload);
                                    to = Some(Location {
                                        col: col_idx,
                                        row: insert_row_idx,
                                    });
                                }
                            }
                        }
                    });

                    if let Some(dragged_payload) = dropped_payload {
                        // The user dropped onto the column, but not on any one item.
                        from = Some(dragged_payload);
                        to = Some(Location {
                            col: col_idx,
                            row: usize::MAX, // Inset last
                        });
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    cycle_store.clear();
                }
                if ui.button("Go").clicked() {
                    commands.trigger(ApplyTurnActions(cycle_store.action_column.clone()));
                }
            });

            if let (Some(from), Some(mut to)) = (from, to) {
                if from.col == to.col {
                    // Dragging within the same column.
                    // Adjust row index if we are re-ordering:
                    to.row -= (from.row < to.row) as usize;
                }

                let item = cycle_store.get_column_mut(from.col).remove(from.row);

                let column = &mut cycle_store.get_column_mut(to.col);
                to.row = to.row.min(column.len());
                column.insert(to.row, item);
            }
        });
}
