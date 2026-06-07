/// Always-on developer debug overlay (FPS, frame time, state switcher).
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::game::GameState;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(EguiPrimaryContextPass, debug_panel);
    }
}

fn debug_panel(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    egui::Window::new("🔧 Debug")
        .default_pos(egui::pos2(8.0, 8.0))
        .default_width(170.0)
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            egui::Grid::new("perf_grid")
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    ui.label("FPS");
                    ui.label(format!("{fps:.1}"));
                    ui.end_row();
                    ui.label("Frame");
                    ui.label(format!("{frame_ms:.2} ms"));
                    ui.end_row();
                });

            ui.separator();
            ui.label(format!("State: {:?}", state.get()));
            ui.separator();

            ui.label("Jump to state:");
            ui.horizontal(|ui| {
                if ui.small_button("Menu").clicked() {
                    next_state.set(GameState::MainMenu);
                }
                if ui.small_button("Play").clicked() {
                    next_state.set(GameState::InGame);
                }
                if ui.small_button("Pause").clicked() {
                    next_state.set(GameState::Paused);
                }
            });
        });

    Ok(())
}
