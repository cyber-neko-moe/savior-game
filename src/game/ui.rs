use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::game::GameState;
use crate::game::protocol::{
    GameAction, GameLog, GameProtocol, PlayerStatus, SvgCatalog, UiSelection,
};
use crate::game::svg::{SvgEntityKind, SvgInteractionState, SvgRenderCache};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (
                main_menu_ui.run_if(in_state(GameState::MainMenu)),
                in_game_ui.run_if(in_state(GameState::InGame)),
                pause_menu_ui.run_if(in_state(GameState::Paused)),
            )
                .chain(),
        )
        .add_systems(
            Update,
            keyboard_pause_toggle.run_if(in_state(GameState::InGame)),
        );
    }
}

fn keyboard_pause_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

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
                ui.heading("3-Panel House Inspection");
                ui.label("Left: Map | Middle: Zone Inspection | Right: Slot Actions");
                ui.add_space(12.0);
                if ui.button("Play").clicked() {
                    next_state.set(GameState::InGame);
                }
            });
        });
    Ok(())
}

fn in_game_ui(
    mut contexts: EguiContexts,
    protocol: Res<GameProtocol>,
    svg_catalog: Res<SvgCatalog>,
    svg_cache: Res<SvgRenderCache>,
    mut interaction_state: ResMut<SvgInteractionState>,
    mut selection: ResMut<UiSelection>,
    mut actions: MessageWriter<GameAction>,
    log: Res<GameLog>,
    player: Res<PlayerStatus>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interaction_state.hovered_id = None;
    interaction_state.hovered_metadata = None;

    egui::TopBottomPanel::top("top_hud").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.strong("Savior Game");
            ui.separator();
            ui.label(format!("Map: {} [{}]", protocol.map.label, protocol.map.id));
            if ui.button("Pause").clicked() {
                next_state.set(GameState::Paused);
            }
        });
    });

    egui::TopBottomPanel::bottom("panel_player_row")
        .resizable(false)
        .exact_height(188.0)
        .show(ctx, |ui| {
            draw_player_row(ui, &player);
        });

    egui::SidePanel::left("panel_map")
        .resizable(true)
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.heading("Map");
            ui.label(format!(
                "Asset: {} (room IDs = SVG element refs)",
                protocol.map.map_svg.file_name()
            ));
            ui.add_space(6.0);
            draw_map_panel(
                ui,
                &protocol,
                &svg_catalog,
                &svg_cache,
                &mut interaction_state,
                &mut selection,
            );
        });

    egui::SidePanel::right("panel_actions")
        .resizable(true)
        .default_width(390.0)
        .show(ctx, |ui| {
            ui.heading("Actions");
            draw_action_panel(ui, &protocol, &selection, &interaction_state, &mut actions);
            ui.separator();
            ui.heading("Game Log");
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(280.0)
                .show(ui, |ui| {
                    for line in log.entries.iter().rev().take(20) {
                        ui.label(line);
                    }
                });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Inspection");
        draw_inspection_panel(
            ui,
            &protocol,
            &svg_catalog,
            &svg_cache,
            &mut interaction_state,
            &mut selection,
        );
    });

    Ok(())
}

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
            if ui.button("Resume").clicked() {
                next_state.set(GameState::InGame);
            }
            if ui.button("Main Menu").clicked() {
                next_state.set(GameState::MainMenu);
            }
        });
    Ok(())
}

fn draw_map_panel(
    ui: &mut egui::Ui,
    protocol: &GameProtocol,
    svg_catalog: &SvgCatalog,
    svg_cache: &SvgRenderCache,
    interaction_state: &mut SvgInteractionState,
    selection: &mut UiSelection,
) {
    let scene = &svg_catalog.map_scene;
    let map_texture = svg_cache.textures.get(&protocol.map.map_svg);
    let width = ui.available_width();
    let aspect = map_texture
        .map(|entry| entry.height as f32 / entry.width.max(1) as f32)
        .unwrap_or(scene.height / scene.width.max(1.0));
    let desired = egui::vec2(width, width * aspect);
    let (canvas_rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(canvas_rect);

    if let Some(texture_entry) = map_texture {
        painter.image(
            texture_entry.texture_id,
            canvas_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        for block in &scene.blocks {
            let r = map_rect(
                block.x,
                block.y,
                block.w,
                block.h,
                scene.width,
                scene.height,
                canvas_rect,
            );
            painter.rect_filled(r, 4.0, egui::Color32::from_gray(28));
            painter.rect_stroke(
                r,
                4.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                egui::StrokeKind::Outside,
            );
        }
    }

    let map_nodes: Vec<_> = svg_cache
        .interaction_nodes
        .get(&protocol.map.map_svg)
        .into_iter()
        .flatten()
        .filter(|node| node.kind == Some(SvgEntityKind::Room) && node.clickable)
        .collect();

    if map_nodes.is_empty() {
        for room_shape in &scene.rooms {
            let room_rect = map_rect(
                room_shape.x,
                room_shape.y,
                room_shape.w,
                room_shape.h,
                scene.width,
                scene.height,
                canvas_rect,
            );
            paint_room_region(
                ui,
                &painter,
                room_rect,
                &room_shape.id,
                None,
                protocol,
                selection,
                interaction_state,
                map_texture.is_none(),
            );
        }
    } else {
        for node in map_nodes {
            let room_rect = map_rect(
                node.left,
                node.top,
                node.width,
                node.height,
                scene.width,
                scene.height,
                canvas_rect,
            );
            paint_room_region(
                ui,
                &painter,
                room_rect,
                &node.base_id,
                Some(&node.metadata),
                protocol,
                selection,
                interaction_state,
                map_texture.is_none(),
            );
        }
    }

    ui.separator();
    ui.label("Rooms");
    egui::Grid::new("room_table").num_columns(2).show(ui, |ui| {
        for room in &protocol.map.rooms {
            let selected = selection.room_id == room.id;
            if ui.selectable_label(selected, &room.label).clicked() {
                selection.room_id = room.id.clone();
                selection.slot_id = None;
            }
            ui.label(format!("{} -> {}", room.id, room.zone_id));
            ui.end_row();
        }
    });
}

fn paint_room_region(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    room_rect: egui::Rect,
    room_id: &str,
    metadata: Option<&std::collections::HashMap<String, String>>,
    protocol: &GameProtocol,
    selection: &mut UiSelection,
    interaction_state: &mut SvgInteractionState,
    draw_fill: bool,
) {
    let effective_room_id = metadata
        .and_then(|meta| meta.get("room"))
        .map(|s| s.as_str())
        .unwrap_or(room_id);

    let room = protocol
        .map
        .rooms
        .iter()
        .find(|r| r.id == effective_room_id);
    let selected = selection.room_id == effective_room_id;
    let fill = if selected {
        egui::Color32::from_rgb(76, 120, 180)
    } else {
        egui::Color32::from_rgb(53, 58, 64)
    };

    let response = ui.interact(
        room_rect,
        ui.id().with(("room", effective_room_id)),
        egui::Sense::click(),
    );
    if response.hovered() {
        interaction_state.hovered_id = Some(effective_room_id.to_string());
        interaction_state.hovered_metadata = metadata.map(|meta| {
            let mut entries: Vec<(String, String)> =
                meta.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        });
    }
    if response.clicked() {
        selection.room_id = effective_room_id.to_string();
        selection.slot_id = None;
        interaction_state.selected_id = Some(effective_room_id.to_string());
        interaction_state.selected_metadata = metadata.map(|meta| {
            let mut entries: Vec<(String, String)> =
                meta.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        });
    }

    if draw_fill {
        painter.rect_filled(room_rect, 6.0, fill);
    }

    let outline_color = if response.hovered() {
        egui::Color32::from_rgb(251, 191, 36)
    } else {
        egui::Color32::from_gray(210)
    };
    painter.rect_stroke(
        room_rect,
        6.0,
        egui::Stroke::new(1.0, outline_color),
        egui::StrokeKind::Outside,
    );

    if let Some(room) = room {
        painter.text(
            room_rect.center(),
            egui::Align2::CENTER_CENTER,
            &room.label,
            egui::FontId::proportional(13.0),
            egui::Color32::WHITE,
        );
    }
}

fn draw_inspection_panel(
    ui: &mut egui::Ui,
    protocol: &GameProtocol,
    svg_catalog: &SvgCatalog,
    svg_cache: &SvgRenderCache,
    interaction_state: &mut SvgInteractionState,
    selection: &mut UiSelection,
) {
    let Some(zone) = selection.current_zone(protocol) else {
        ui.colored_label(egui::Color32::LIGHT_RED, "No zone selected.");
        return;
    };

    ui.horizontal(|ui| {
        ui.heading(&zone.room_label);
        ui.label(format!("Zone ID: {}", zone.id));
    });

    egui::Grid::new("zone_metrics")
        .num_columns(3)
        .show(ui, |ui| {
            ui.label(format!("Temp: {:.1} C", zone.temp_c));
            ui.label(format!("Humidity: {:.0}%", zone.humidity_pct));
            ui.label(format!("Room HP: {}", zone.hp));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(format!(
        "Zone SVG: {} (slot IDs as interactive refs)",
        zone.zone_svg.file_name()
    ));

    let Some(scene) = svg_catalog.zone_scenes.get(&zone.zone_svg) else {
        ui.colored_label(egui::Color32::LIGHT_RED, "Zone SVG scene not found.");
        return;
    };

    let zone_texture = svg_cache.textures.get(&zone.zone_svg);
    let width = ui.available_width();
    let aspect = zone_texture
        .map(|entry| entry.height as f32 / entry.width.max(1) as f32)
        .unwrap_or(scene.height / scene.width.max(1.0));
    let desired = egui::vec2(width, width * aspect);
    let (canvas_rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(canvas_rect);

    if let Some(texture_entry) = zone_texture {
        painter.image(
            texture_entry.texture_id,
            canvas_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        for block in &scene.blocks {
            let r = map_rect(
                block.x,
                block.y,
                block.w,
                block.h,
                scene.width,
                scene.height,
                canvas_rect,
            );
            painter.rect_filled(r, 3.0, egui::Color32::from_gray(22));
            painter.rect_stroke(
                r,
                3.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(95)),
                egui::StrokeKind::Outside,
            );
        }
    }

    let slot_nodes: Vec<_> = svg_cache
        .interaction_nodes
        .get(&zone.zone_svg)
        .into_iter()
        .flatten()
        .filter(|node| node.kind == Some(SvgEntityKind::Slot) && node.clickable)
        .collect();

    if slot_nodes.is_empty() {
        for slot_shape in &scene.slots {
            let center = map_point(
                slot_shape.cx,
                slot_shape.cy,
                scene.width,
                scene.height,
                canvas_rect,
            );
            let radius = slot_shape.r * canvas_rect.width() / scene.width.max(1.0);
            let hit = egui::Rect::from_center_size(center, egui::vec2(radius * 2.2, radius * 2.2));
            paint_slot_region(
                ui,
                &painter,
                hit,
                center,
                radius,
                &slot_shape.id,
                None,
                zone,
                selection,
                interaction_state,
            );
        }
    } else {
        for node in slot_nodes {
            let hit = map_rect(
                node.left,
                node.top,
                node.width,
                node.height,
                scene.width,
                scene.height,
                canvas_rect,
            );
            let center = hit.center();
            let radius = (hit.width().min(hit.height()) * 0.4).max(6.0);
            paint_slot_region(
                ui,
                &painter,
                hit,
                center,
                radius,
                &node.base_id,
                Some(&node.metadata),
                zone,
                selection,
                interaction_state,
            );
        }
    }

    ui.separator();
    ui.label("Slots in zone");
    for slot in &zone.slots {
        let selected = selection.slot_id.as_deref() == Some(slot.id.as_str());
        if ui
            .selectable_label(selected, format!("{} ({})", slot.name, slot.id))
            .clicked()
        {
            selection.slot_id = Some(slot.id.clone());
        }
    }
}

fn paint_slot_region(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    hit: egui::Rect,
    center: egui::Pos2,
    radius: f32,
    slot_id: &str,
    metadata: Option<&std::collections::HashMap<String, String>>,
    zone: &crate::game::protocol::ZoneDef,
    selection: &mut UiSelection,
    interaction_state: &mut SvgInteractionState,
) {
    let effective_slot_id = metadata
        .and_then(|meta| meta.get("slot"))
        .map(|s| s.as_str())
        .unwrap_or(slot_id);

    let response = ui.interact(
        hit,
        ui.id().with(("slot", effective_slot_id)),
        egui::Sense::click(),
    );
    if response.hovered() {
        interaction_state.hovered_id = Some(effective_slot_id.to_string());
        interaction_state.hovered_metadata = metadata.map(|meta| {
            let mut entries: Vec<(String, String)> =
                meta.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        });
    }
    if response.clicked() {
        selection.slot_id = Some(effective_slot_id.to_string());
        interaction_state.selected_id = Some(effective_slot_id.to_string());
        interaction_state.selected_metadata = metadata.map(|meta| {
            let mut entries: Vec<(String, String)> =
                meta.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        });
    }

    let selected = selection.slot_id.as_deref() == Some(effective_slot_id);
    let mut color = egui::Color32::from_rgb(150, 150, 150);
    if selected {
        color = egui::Color32::from_rgb(250, 130, 80);
    }
    if zone
        .slots
        .iter()
        .any(|slot| slot.id == effective_slot_id && !slot.items.is_empty())
    {
        color = egui::Color32::from_rgb(95, 188, 95);
    }
    if selected {
        color = egui::Color32::from_rgb(250, 130, 80);
    }

    painter.circle_filled(center, radius, color);
    painter.circle_stroke(center, radius, egui::Stroke::new(1.0, egui::Color32::WHITE));
    painter.text(
        center + egui::vec2(0.0, radius + 10.0),
        egui::Align2::CENTER_TOP,
        effective_slot_id,
        egui::FontId::proportional(11.0),
        egui::Color32::LIGHT_GRAY,
    );
}

fn draw_action_panel(
    ui: &mut egui::Ui,
    protocol: &GameProtocol,
    selection: &UiSelection,
    interaction_state: &SvgInteractionState,
    actions: &mut MessageWriter<GameAction>,
) {
    let Some(zone) = selection.current_zone(protocol) else {
        ui.label("No zone selected");
        return;
    };

    ui.label(format!("Current Zone: {}", zone.room_label));
    ui.label(format!("Zone Asset Ref: {}", zone.zone_svg.file_name()));
    if let Some(hovered) = &interaction_state.hovered_id {
        ui.label(format!("Hovered ID: {}", hovered));
    }
    if let Some(selected) = &interaction_state.selected_id {
        ui.label(format!("Selected SVG ID: {}", selected));
    }
    ui.separator();

    if let Some(meta) = &interaction_state.hovered_metadata {
        ui.label("Hovered metadata:");
        for (k, v) in meta {
            ui.small(format!("{} = {}", k, v));
        }
        ui.separator();
    }

    if let Some(meta) = &interaction_state.selected_metadata {
        ui.label("Selected metadata:");
        for (k, v) in meta {
            ui.small(format!("{} = {}", k, v));
        }
        ui.separator();
    }

    if let Some(slot_id) = selection.slot_id.as_deref() {
        if let Some(slot) = zone.slots.iter().find(|s| s.id == slot_id) {
            ui.heading(format!("Slot: {}", slot.name));
            ui.label(format!("Slot ID ref: {}", slot.id));
            ui.label(format!("Hint: {}", slot.action_hint));
            ui.add_space(8.0);

            if ui.button("Inspect Slot").clicked() {
                actions.write(GameAction::InspectSlot {
                    slot_id: slot.id.clone(),
                });
            }

            if ui.button("Take First Item").clicked() {
                actions.write(GameAction::LootFirstItem {
                    slot_id: slot.id.clone(),
                });
            }

            if ui.button("Repair Current Zone (+10 HP)").clicked() {
                actions.write(GameAction::RepairCurrentZone);
            }

            ui.separator();
            ui.label("Items in slot:");
            if slot.items.is_empty() {
                ui.label("(empty)");
            } else {
                egui::Grid::new("item_grid").num_columns(3).show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Name");
                    ui.strong("Q");
                    ui.end_row();
                    for item in &slot.items {
                        ui.label(&item.id);
                        ui.label(format!("{} ({})", item.name, item.kind));
                        ui.label(item.quality.to_string());
                        ui.end_row();
                    }
                });
            }
        } else {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                "Selected slot missing in zone data",
            );
        }
    } else {
        ui.label("Select a slot from the middle panel to enable actions.");
    }
}

fn draw_player_row(ui: &mut egui::Ui, player: &PlayerStatus) {
    const INV_ROWS: usize = 3;
    const INV_COLS: usize = 7;

    ui.horizontal(|ui| {
        ui.group(|ui| {
            ui.set_min_width(120.0);
            ui.heading(&player.avatar);
            ui.label(&player.name);
            ui.small("Avatar");
        });

        ui.group(|ui| {
            ui.set_min_width(220.0);
            ui.label(format!("HP: {}", player.hp));
            ui.add(
                egui::ProgressBar::new(player.hp as f32 / 100.0)
                    .desired_width(170.0)
                    .text("Health"),
            );
            ui.label(format!("Stamina: {}", player.stamina));
            ui.add(
                egui::ProgressBar::new(player.stamina as f32 / 100.0)
                    .desired_width(170.0)
                    .text("Stamina"),
            );
            ui.small(format!("Condition: {}", player.condition));
        });

        ui.group(|ui| {
            ui.set_min_width(660.0);
            ui.label("Inventory (3x7)");
            egui::Grid::new("inventory_grid")
                .num_columns(INV_COLS)
                .spacing(egui::vec2(6.0, 6.0))
                .show(ui, |ui| {
                    for row in 0..INV_ROWS {
                        for col in 0..INV_COLS {
                            let idx = row * INV_COLS + col;
                            let item = player.inventory.get(idx);
                            let (label, color) = if let Some(item) = item {
                                (
                                    format!("{}\nQ{}", item.name, item.quality),
                                    egui::Color32::from_rgb(46, 56, 40),
                                )
                            } else {
                                (
                                    format!("[{}]", idx + 1),
                                    egui::Color32::from_rgb(32, 32, 32),
                                )
                            };

                            egui::Frame::new()
                                .fill(color)
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100)))
                                .corner_radius(4.0)
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        egui::vec2(78.0, 42.0),
                                        egui::Label::new(
                                            egui::RichText::new(label)
                                                .size(11.0)
                                                .color(egui::Color32::WHITE),
                                        ),
                                    );
                                });
                        }
                        ui.end_row();
                    }
                });
        });
    });
}

fn map_rect(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_w: f32,
    src_h: f32,
    rect: egui::Rect,
) -> egui::Rect {
    let min = map_point(x, y, src_w, src_h, rect);
    let max = map_point(x + w, y + h, src_w, src_h, rect);
    egui::Rect::from_min_max(min, max)
}

fn map_point(x: f32, y: f32, src_w: f32, src_h: f32, rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(
        rect.left() + (x / src_w.max(1.0)) * rect.width(),
        rect.top() + (y / src_h.max(1.0)) * rect.height(),
    )
}
