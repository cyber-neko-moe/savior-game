use bevy::prelude::*;

use crate::game::GameState;

// ── Component ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { speed: 200.0 }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(
                Update,
                player_movement.run_if(in_state(GameState::InGame)),
            );
    }
}

// ── Systems ──────────────────────────────────────────────────────────────────

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        // Auto-despawn when leaving InGame (Bevy 0.18 API)
        DespawnOnExit(GameState::InGame),
        Name::new("Player"),
        Player::default(),
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.3),
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn player_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    for (player, mut transform) in &mut query {
        let mut dir = Vec2::ZERO;

        if input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }

        if dir != Vec2::ZERO {
            transform.translation +=
                (dir.normalize() * player.speed * time.delta_secs()).extend(0.0);
        }
    }
}
