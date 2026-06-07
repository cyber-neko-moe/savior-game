# Savior Game

A Rust game scaffold built with **Bevy 0.18** + **bevy_egui 0.39**, targeting
both native desktop and WebAssembly (WebGL2).

## Project layout

```
savior-game/
├── src/
│   ├── main.rs               Entry point
│   └── game/
│       ├── mod.rs            GamePlugin – wires everything together
│       ├── states.rs         GameState enum + state transitions
│       ├── player.rs         Player component + movement
│       ├── ui.rs             Main-menu / HUD / pause overlays (egui)
│       └── debug_ui.rs       Always-on dev debug panel (FPS, state switcher)
├── assets/                   Game assets (copied to dist for web)
├── index.html                Trunk entry point for WASM builds
├── Trunk.toml                Trunk bundler configuration
├── .cargo/config.toml        Build flags (wasm target, fast linkers)
└── .vscode/
    ├── tasks.json            VS Code build/serve tasks
    └── launch.json           Native debugger config (requires CodeLLDB)
```

## Prerequisites

| Tool | Install |
|------|---------|
| Rust (stable ≥ 1.89) | `rustup update stable` |
| WASM target | `rustup target add wasm32-unknown-unknown` |
| Trunk bundler | `cargo install trunk` |
| wasm-opt (optional, faster) | bundled with Trunk |

On **Linux**, you also need a few XCB libraries for the native build:
```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Running

### Native desktop
```bash
cargo run
```
Or use **Tasks: Run Task → run (native)** in VS Code.

### Web (dev server with live-reload)
```bash
trunk serve
# → http://127.0.0.1:8080
```
Or use **Tasks: Run Task → serve (web – dev)** in VS Code.

### Web (optimised release build)
```bash
trunk build --release
# output in dist/
```

## Game states

| State | Description |
|-------|-------------|
| `MainMenu` | Title screen with "Play" button |
| `InGame` | Player can move with WASD / arrow keys |
| `Paused` | Pause overlay; resume or return to menu |

The **🔧 Debug** panel (top-left) is always visible and lets you jump between
states, monitor FPS, and frame time.

## Key crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | 0.18.1 | Game engine |
| `bevy_egui` | 0.39.1 | Immediate-mode UI via egui |
