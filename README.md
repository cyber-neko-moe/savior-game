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
│       ├── protocol.rs       Game data protocol (Map / Zone / Slot / Item)
│       ├── svg_assets.rs     SVG parser (ID refs -> interactive shapes)
│       ├── states.rs         GameState enum + state transitions
│       ├── ui.rs             3-panel game UI (Map / Inspection / Actions)
│       └── debug_ui.rs       Always-on dev debug panel (FPS, state switcher)
├── assets/
│   └── svg/                  Map + zone SVG definitions (ID-driven)
├── index.html                Trunk entry point for WASM builds
├── Trunk.toml                Trunk bundler configuration
├── .cargo/config.toml        Build flags (wasm target, fast linkers)
└── .vscode/
    ├── tasks.json            VS Code build/serve tasks
    └── launch.json           Native debugger config (requires CodeLLDB)
```

## Prerequisites

| Tool                        | Install                                    |
| --------------------------- | ------------------------------------------ |
| Rust (stable ≥ 1.89)        | `rustup update stable`                     |
| WASM target                 | `rustup target add wasm32-unknown-unknown` |
| Trunk bundler               | `cargo install trunk`                      |
| wasm-opt (optional, faster) | bundled with Trunk                         |

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

| State      | Description                             |
| ---------- | --------------------------------------- |
| `MainMenu` | Title screen with "Play" button         |
| `InGame`   | 3-panel inspection game loop            |
| `Paused`   | Pause overlay; resume or return to menu |

The **🔧 Debug** panel (top-left) is always visible and lets you jump between
states, monitor FPS, and frame time.

## 3-panel loop

1. **Left panel (`Map`)**
   - Renders `assets/svg/house_4b2b1k.svg`.
   - Every `room_*` SVG element ID is clickable and maps to a `RoomNode`.

2. **Middle panel (`Inspection`)**
   - Shows current `ZoneDef` telemetry: temperature, humidity, room HP.
   - Renders zone SVG (`zone_bedroom.svg`, `zone_bathroom.svg`, `zone_kitchen.svg`).
   - Every `slot_*` SVG ID is clickable and maps to a `SlotDef`.

3. **Right panel (`Actions`)**
   - Shows selected slot and items currently in it.
   - Sends gameplay actions (`InspectSlot`, `LootFirstItem`, `RepairCurrentZone`).

## SVG + data protocol contract

The game uses an ID contract between assets and logic:

- `room_*` IDs in map SVG -> `MapDef.rooms[*].id`
- `slot_*` IDs in zone SVG -> `ZoneDef.slots[*].id`

This lets UI interactions stay asset-driven: artists change layout SVG, while
logic remains in protocol structs and action systems.

## Bevy integration flow

1. `ProtocolPlugin` initializes resources:
   - `GameProtocol` (Map/Zone/Slot/Item definitions)
   - `SvgCatalog` (parsed SVG scenes)
   - `UiSelection` (selected room + slot)
   - `GameLog`

2. UI systems (`ui.rs`) only read protocol data and emit `GameAction` messages.

3. `apply_game_actions` system consumes messages and mutates game state in one place.

This keeps rendering, input, and simulation clearly separated in ECS style.

## Key crates

| Crate       | Version | Purpose                         |
| ----------- | ------- | ------------------------------- |
| `bevy`      | 0.18.1  | Game engine                     |
| `bevy_egui` | 0.39.1  | Immediate-mode UI via egui      |
| `roxmltree` | 0.20    | SVG XML parsing for element IDs |
