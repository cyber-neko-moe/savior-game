/// In-game UI: main-menu overlay, pause menu, HUD.
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::game::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (
                main_menu_ui.run_if(in_state(GameState::MainMenu)),
                hud_ui.run_if(in_state(GameState::InGame)),
                pause_menu_ui.run_if(in_state(GameState::Paused)),
            )
                .chain(), // run sequentially – each borrows EguiContexts mutably
        );
    }
}

// ── Main Menu ─────────────────────────────────────────────────────────────────

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    egui::Window::new("Savior Game")
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label("A Bevy + egui game scaffold");
                ui.add_space(16.0);
                if ui.button("  ▶  Play  ").clicked() {
                    next_state.set(GameState::InGame);
                }
                ui.add_space(10.0);
            });
        });
    Ok(())
}

// ── HUD ───────────────────────────────────────────────────────────────────────

fn hud_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    egui::Window::new("##hud")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .show(contexts.ctx_mut()?, |ui| {
            if ui.small_button("⏸ Pause").clicked() {
                next_state.set(GameState::Paused);
            }
        });
    Ok(())
}

// ── Pause Menu ────────────────────────────────────────────────────────────────

fn pause_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    egui::Window::new("Paused")
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                if ui.button("  ▶  Resume  ").clicked() {
                    next_state.set(GameState::InGame);
                }
                ui.add_space(6.0);
                if ui.button("  ⏏  Main Menu  ").clicked() {
                    next_state.set(GameState::MainMenu);
                }
                ui.add_space(10.0);
            });
        });
    Ok(())
}
