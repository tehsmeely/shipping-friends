use crate::game::movement::{ApplyTurnActions, Facing};
use crate::screen::Screen;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_egui::egui::{vec2, Color32, Frame, Id, Stroke, WidgetText};
use bevy_egui::{egui, EguiContexts};

pub fn plugin(app: &mut App) {
    app.init_resource::<GlobalTurnLock>();
    app.init_resource::<CycleNum>();
    app.register_type::<(GlobalTurnLock, CycleNum)>();
    app.add_systems(OnEnter(Screen::Playing), setup);
    app.add_systems(Update, (do_ui).run_if(in_state(Screen::Playing)));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnAction {
    Forward,
    RotateClockwise,
    RotateAntiClockwise,
}

#[derive(Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct GlobalTurnLock {
    locked: bool,
}
impl Default for GlobalTurnLock {
    fn default() -> Self {
        Self { locked: false }
    }
}

impl GlobalTurnLock {
    pub fn unlock(&mut self) {
        self.locked = false;
    }
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

#[derive(Clone, Debug)]
pub struct TurnActions(pub [Option<TurnAction>; 6]);

#[derive(Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct CycleNum {
    cycle_num: usize,
    turn_num: usize,
    turns_per_cycle: usize,
}

impl CycleNum {
    pub fn increment(&mut self) {
        self.turn_num += 1;
        if self.turn_num >= self.turns_per_cycle {
            self.turn_num = 0;
            self.cycle_num += 1;
        }
    }

    fn display_cycle_num(&self) -> String {
        format!("Cycle: {}", self.cycle_num + 1)
    }
    fn display_turn_num(&self) -> String {
        format!("Turn: {}/{}", self.turn_num + 1, self.turns_per_cycle)
    }
}

impl Default for CycleNum {
    fn default() -> Self {
        Self {
            cycle_num: 0,
            turn_num: 0,
            turns_per_cycle: 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ColumnName {
    Active,
    Store,
}

#[derive(Clone, Debug, Resource)]
pub struct CycleStore {
    turn_actions: TurnActions,
    store: Vec<TurnAction>,
}

impl CycleStore {
    pub fn new() -> Self {
        Self {
            turn_actions: TurnActions([None; 6]),
            store: vec![
                TurnAction::Forward,
                TurnAction::RotateClockwise,
                TurnAction::RotateAntiClockwise,
            ],
        }
    }

    fn clear(&mut self) {
        for elt in self.turn_actions.0.iter_mut() {
            if let Some(elt) = elt {
                self.store.push(*elt);
            }
            *elt = None;
        }
    }

    fn take(&mut self, col: ColumnName, row: usize) -> Option<TurnAction> {
        match col {
            ColumnName::Active => {
                if row < self.turn_actions.0.len() {
                    self.turn_actions.0[row].take()
                } else {
                    None
                }
            }
            ColumnName::Store => Some(self.store.remove(row)),
        }
    }

    fn take_turn_actions(&mut self) -> TurnActions {
        let populated = self.turn_actions.clone();
        self.turn_actions = TurnActions([None; 6]);
        populated
    }

    fn add(&mut self, col: ColumnName, row: usize, action: TurnAction) {
        match col {
            ColumnName::Active => {
                if row < self.turn_actions.0.len() {
                    self.turn_actions.0[row] = Some(action)
                }
            }
            ColumnName::Store => self.store.insert(row, action),
        }
    }
}

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: ColumnName,
    row: usize,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(CycleStore::new());
}

fn do_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut cycle_store: ResMut<CycleStore>,
    cycle_num: Res<CycleNum>,
    mut global_turn_lock: ResMut<GlobalTurnLock>,
) {
    egui::Window::new("Game UI")
        .anchor(egui::Align2::RIGHT_BOTTOM, vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            // If there is a drop, store the location of the item being dragged, and the destination for the drop.
            let mut from = None;
            let mut to = None;

            ui.label(cycle_num.display_cycle_num());
            ui.label(cycle_num.display_turn_num());

            ui.columns(2, |uis| {
                if global_turn_lock.locked {
                    for ui in uis.iter_mut() {
                        ui.disable();
                    }
                }

                // Active Column
                {
                    let ui = &mut uis[0];
                    let frame = Frame::default().inner_margin(4.0);

                    for (idx, elt) in cycle_store.turn_actions.0.iter().enumerate() {
                        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                            ui.set_min_size(vec2(64.0, 30.0));

                            let item_id = Id::new(("drag_and_drop_cycle", ColumnName::Active, idx));
                            let item_location = Location {
                                col: ColumnName::Active,
                                row: idx,
                            };
                            let _label: WidgetText = match elt {
                                Some(action) => (*action).into(),
                                None => "Wait".into(),
                            };
                            match elt {
                                Some(action) => {
                                    ui.dnd_drag_source(item_id, item_location, |ui| {
                                        ui.label(*action);
                                    });
                                }
                                None => {
                                    ui.label("Wait");
                                }
                            }
                        });
                        if let Some(dragged_payload) = dropped_payload {
                            // The user dropped onto the whole area, which is what we want for this
                            from = Some(dragged_payload);
                            to = Some(Location {
                                col: ColumnName::Active,
                                row: idx,
                            });
                        }
                    }
                }

                // Store Column
                {
                    let ui = &mut uis[1];

                    let frame = Frame::default().inner_margin(4.0);

                    let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                        ui.set_min_size(vec2(64.0, 100.0));
                        for (row_idx, item) in cycle_store.store.iter().enumerate() {
                            let item_id =
                                Id::new(("drag_and_drop_cycle", ColumnName::Store, row_idx));
                            let item_location = Location {
                                col: ColumnName::Store,
                                row: row_idx,
                            };
                            let response = ui
                                .dnd_drag_source(item_id, item_location, |ui| {
                                    // I'd like to have a frame and whatnot here, but adding
                                    // frames results in a debug assertion failure.
                                    // See: https://github.com/emilk/egui/issues/4604

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
                                        col: ColumnName::Store,
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
                            col: ColumnName::Store,
                            row: cycle_store.store.len(),
                        });
                    }
                }
            });

            if let (Some(from), Some(mut to)) = (from, to) {
                if from.col == to.col {
                    // Dragging within the same column.
                    // Adjust row index if we are re-ordering:
                    to.row -= (from.row < to.row) as usize;
                }

                //let item = cycle_store.get_column_mut(from.col).remove(from.row);
                if let Some(item) = cycle_store.take(from.col, from.row) {
                    cycle_store.add(to.col, to.row, item);
                }
            }

            ui.horizontal(|ui| {
                if global_turn_lock.locked {
                    ui.disable();
                }
                if ui.button("Clear").clicked() {
                    cycle_store.clear();
                }
                if ui.button("Go").clicked() {
                    global_turn_lock.locked = true;
                    commands.trigger(ApplyTurnActions(cycle_store.take_turn_actions()));
                }
            });
        });
}
