use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::{EguiTextureHandle, EguiUserTextures, egui};

use crate::game::GameState;
use crate::game::protocol::{
    GameAction, GameLog, GameProtocol, SvgAssetKey, SvgCatalog, UiSelection,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SvgTextureEntry {
    pub image_handle: Handle<Image>,
    pub texture_id: egui::TextureId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgEntityKind {
    Room,
    Slot,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SvgInteractiveNode {
    pub raw_id: String,
    pub base_id: String,
    pub kind: Option<SvgEntityKind>,
    pub clickable: bool,
    pub metadata: HashMap<String, String>,
    pub unknown_flags: Vec<String>,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Resource, Default)]
pub struct SvgRenderCache {
    pub trees: HashMap<SvgAssetKey, usvg::Tree>,
    pub textures: HashMap<SvgAssetKey, SvgTextureEntry>,
    pub dirty_assets: Vec<SvgAssetKey>,
    pub interaction_nodes: HashMap<SvgAssetKey, Vec<SvgInteractiveNode>>,
}

impl SvgRenderCache {
    #[allow(dead_code)]
    pub fn texture_id(&self, asset_key: SvgAssetKey) -> Option<egui::TextureId> {
        self.textures.get(&asset_key).map(|entry| entry.texture_id)
    }

    pub fn mark_dirty(&mut self, asset_key: SvgAssetKey) {
        if !self.dirty_assets.contains(&asset_key) {
            self.dirty_assets.push(asset_key);
        }
    }

    #[allow(dead_code)]
    pub fn find_node(
        &self,
        asset_key: SvgAssetKey,
        kind: SvgEntityKind,
        base_id: &str,
    ) -> Option<&SvgInteractiveNode> {
        self.interaction_nodes
            .get(&asset_key)?
            .iter()
            .find(|node| node.kind == Some(kind) && node.base_id == base_id)
    }
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug, Clone)]
pub struct SvgInteractionState {
    pub hovered_id: Option<String>,
    pub selected_id: Option<String>,
    pub hovered_metadata: Option<Vec<(String, String)>>,
    pub selected_metadata: Option<Vec<(String, String)>>,
}

pub fn build_svg_pipeline(app: &mut App) {
    app.init_resource::<SvgRenderCache>()
        .init_resource::<SvgInteractionState>()
        .add_systems(
            PostStartup,
            (
                seed_svg_render_cache_system,
                validate_svg_definitions_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            mark_dirty_from_game_actions.run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            PostUpdate,
            svg_texture_baking_system.run_if(in_state(GameState::InGame)),
        );
}

fn mark_dirty_from_game_actions(
    mut actions: MessageReader<GameAction>,
    selection: Res<UiSelection>,
    protocol: Res<GameProtocol>,
    mut cache: ResMut<SvgRenderCache>,
) {
    let mut any_action = false;
    for _ in actions.read() {
        any_action = true;
    }

    if !any_action {
        return;
    }

    cache.mark_dirty(protocol.map.map_svg);

    if let Some(zone) = selection.current_zone(&protocol) {
        cache.mark_dirty(zone.zone_svg);
    }
}

fn seed_svg_render_cache_system(mut cache: ResMut<SvgRenderCache>) {
    let options = usvg::Options::default();

    for (asset_key, svg_source) in embedded_svg_sources() {
        match usvg::Tree::from_str(svg_source, &options) {
            Ok(tree) => {
                let nodes = collect_interaction_nodes(&tree);
                cache.trees.insert(asset_key, tree);
                cache.interaction_nodes.insert(asset_key, nodes);
                cache.mark_dirty(asset_key);
            }
            Err(error) => {
                panic!("failed to parse embedded SVG '{asset_key}': {error}");
            }
        }
    }
}

pub fn validate_svg_definitions_system(
    mut protocol: ResMut<GameProtocol>,
    svg_catalog: Res<SvgCatalog>,
    cache: Res<SvgRenderCache>,
    mut game_log: ResMut<GameLog>,
) {
    let map_svg_ids: HashSet<&str> = cache
        .interaction_nodes
        .get(&protocol.map.map_svg)
        .into_iter()
        .flatten()
        .filter(|node| node.kind == Some(SvgEntityKind::Room) && node.clickable)
        .map(|node| node.base_id.as_str())
        .collect();

    let expected_room_ids: Vec<&str> = protocol
        .map
        .rooms
        .iter()
        .map(|room| room.id.as_str())
        .collect();

    let missing_rooms: Vec<&str> = expected_room_ids
        .iter()
        .copied()
        .filter(|room_id| !map_svg_ids.contains(*room_id))
        .collect();

    assert!(
        missing_rooms.is_empty(),
        "missing room_* ids in map SVG: {}",
        missing_rooms.join(", ")
    );

    if let Some(map_nodes) = cache.interaction_nodes.get(&protocol.map.map_svg) {
        for node in map_nodes {
            for flag in &node.unknown_flags {
                game_log.push(format!(
                    "WARN: unknown map flag '{}' in id '{}' (ignored).",
                    flag, node.raw_id
                ));
            }
        }
    }

    for zone in protocol.zones.values_mut() {
        let Some(zone_scene) = svg_catalog.zone_scenes.get(&zone.zone_svg) else {
            game_log.push(format!(
                "WARN: zone scene '{}' is missing for zone {}. Disabling all slots.",
                zone.zone_svg, zone.id
            ));
            zone.slots.clear();
            continue;
        };

        let scene_slot_ids: HashSet<&str> = cache
            .interaction_nodes
            .get(&zone.zone_svg)
            .into_iter()
            .flatten()
            .filter(|node| node.kind == Some(SvgEntityKind::Slot) && node.clickable)
            .map(|node| node.base_id.as_str())
            .collect();

        let mut removed_ids = Vec::new();
        zone.slots.retain(|slot| {
            let exists = scene_slot_ids.contains(slot.id.as_str());
            if !exists {
                removed_ids.push(slot.id.clone());
            }
            exists
        });

        for slot_id in removed_ids {
            game_log.push(format!(
                "WARN: slot id '{}' missing in '{}'; interaction disabled.",
                slot_id, zone.zone_svg
            ));
        }

        if let Some(zone_nodes) = cache.interaction_nodes.get(&zone.zone_svg) {
            for node in zone_nodes {
                for flag in &node.unknown_flags {
                    game_log.push(format!(
                        "WARN: unknown zone flag '{}' in id '{}' (ignored).",
                        flag, node.raw_id
                    ));
                }
            }
        }

        if zone_scene.slots.is_empty() {
            game_log.push(format!(
                "WARN: zone '{}' has no slot markers in parsed SVG scene.",
                zone.zone_svg
            ));
        }
    }
}

pub fn svg_texture_baking_system(
    mut cache: ResMut<SvgRenderCache>,
    mut images: ResMut<Assets<Image>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    if cache.dirty_assets.is_empty() {
        return;
    }

    let dirty_keys: Vec<SvgAssetKey> = cache.dirty_assets.drain(..).collect();

    for key in dirty_keys {
        let Some(tree) = cache.trees.get(&key) else {
            continue;
        };

        let svg_size = tree.size().to_int_size();
        let width = svg_size.width();
        let height = svg_size.height();

        let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) else {
            continue;
        };

        let mut pixmap_mut = pixmap.as_mut();
        resvg::render(tree, tiny_skia::Transform::identity(), &mut pixmap_mut);

        let image_data = pixmap.data().to_vec();
        let image = Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &image_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        if let Some(old_entry) = cache.textures.remove(&key) {
            user_textures.remove_image(old_entry.image_handle.id());
            images.remove(old_entry.image_handle.id());
        }

        let image_handle = images.add(image);
        let texture_id = user_textures.add_image(EguiTextureHandle::Strong(image_handle.clone()));

        cache.textures.insert(
            key,
            SvgTextureEntry {
                image_handle,
                texture_id,
                width,
                height,
            },
        );
    }
}

fn embedded_svg_sources() -> [(SvgAssetKey, &'static str); 4] {
    [
        (
            SvgAssetKey::MapHouse4b2b1k,
            include_str!("../../../assets/svg/house_4b2b1k.svg"),
        ),
        (
            SvgAssetKey::ZoneBedroom,
            include_str!("../../../assets/svg/zone_bedroom.svg"),
        ),
        (
            SvgAssetKey::ZoneBathroom,
            include_str!("../../../assets/svg/zone_bathroom.svg"),
        ),
        (
            SvgAssetKey::ZoneKitchen,
            include_str!("../../../assets/svg/zone_kitchen.svg"),
        ),
    ]
}

fn collect_interaction_nodes(tree: &usvg::Tree) -> Vec<SvgInteractiveNode> {
    let mut out = Vec::new();
    collect_group_nodes(tree.root(), &mut out);
    out
}

fn collect_group_nodes(group: &usvg::Group, out: &mut Vec<SvgInteractiveNode>) {
    for node in group.children() {
        if let Some(parsed) = parse_svg_node(node) {
            out.push(parsed);
        }

        if let usvg::Node::Group(child_group) = node {
            collect_group_nodes(child_group, out);
        }
    }
}

fn parse_svg_node(node: &usvg::Node) -> Option<SvgInteractiveNode> {
    let raw_id = node.id();
    if raw_id.is_empty() {
        return None;
    }

    let parsed = parse_svg_id(raw_id);
    if !parsed.clickable {
        return None;
    }

    let bbox = node.abs_layer_bounding_box()?;
    Some(SvgInteractiveNode {
        raw_id: raw_id.to_string(),
        base_id: parsed.base_id,
        kind: parsed.kind,
        clickable: parsed.clickable,
        metadata: parsed.metadata,
        unknown_flags: parsed.unknown_flags,
        left: bbox.left(),
        top: bbox.top(),
        width: bbox.width(),
        height: bbox.height(),
    })
}

struct ParsedSvgId {
    base_id: String,
    kind: Option<SvgEntityKind>,
    clickable: bool,
    metadata: HashMap<String, String>,
    unknown_flags: Vec<String>,
}

fn parse_svg_id(raw_id: &str) -> ParsedSvgId {
    let mut parts = raw_id.split("--");
    let base_id = parts.next().unwrap_or_default().to_string();

    let kind = if base_id.starts_with("room_") {
        Some(SvgEntityKind::Room)
    } else if base_id.starts_with("slot_") {
        Some(SvgEntityKind::Slot)
    } else {
        None
    };

    let mut clickable = kind.is_some();
    let mut metadata = HashMap::new();
    let mut unknown_flags = Vec::new();

    for token in parts {
        if token.eq_ignore_ascii_case("clickable") {
            clickable = true;
            continue;
        }

        if let Some(rest) = token.strip_prefix("data-") {
            let mut kv = rest.splitn(2, '-');
            let key = kv.next().unwrap_or_default();
            let value = kv.next().unwrap_or("true");
            if !key.is_empty() {
                metadata.insert(key.to_string(), value.to_string());
            }
            continue;
        }

        unknown_flags.push(token.to_string());
    }

    ParsedSvgId {
        base_id,
        kind,
        clickable,
        metadata,
        unknown_flags,
    }
}
