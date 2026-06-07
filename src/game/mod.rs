use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod debug_ui;
pub mod player;
pub mod states;
pub mod ui;

pub use states::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Savior Game".into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(EguiPlugin::default())
        // Sub-plugins (order matters: states first)
        .add_plugins(states::StatesPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(debug_ui::DebugUiPlugin)
        // Persistent camera – lives for the entire session
        .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
