use bevy::prelude::*;

/// Top-level game states.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    /// Title / main-menu screen.
    #[default]
    MainMenu,
    /// Active gameplay.
    InGame,
    /// Paused – game world frozen, pause overlay visible.
    Paused,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::MainMenu), on_enter_main_menu)
            .add_systems(OnEnter(GameState::InGame), on_enter_in_game)
            .add_systems(OnEnter(GameState::Paused), on_enter_paused)
            .add_systems(OnExit(GameState::InGame), on_exit_in_game);
    }
}

fn on_enter_main_menu() {
    info!("[state] → MainMenu");
}

fn on_enter_in_game() {
    info!("[state] → InGame");
}

fn on_enter_paused() {
    info!("[state] → Paused");
}

fn on_exit_in_game() {
    info!("[state] ← leaving InGame");
}
