use std::collections::HashMap;
use std::fmt;

use bevy::prelude::*;

use crate::game::states::GameState;
use crate::game::svg_assets::SvgScene;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameAction>()
            .insert_resource(GameLog::default())
            .insert_resource(PlayerStatus::default())
            .add_systems(Startup, setup_game_protocol)
            .add_systems(
                Update,
                apply_game_actions.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub quality: u8,
}

#[derive(Debug, Clone)]
pub struct SlotDef {
    pub id: String,
    pub name: String,
    pub action_hint: String,
    pub items: Vec<ItemDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvgAssetKey {
    MapHouse4b2b1k,
    ZoneBedroom,
    ZoneBathroom,
    ZoneKitchen,
}

impl SvgAssetKey {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::MapHouse4b2b1k => "house_4b2b1k.svg",
            Self::ZoneBedroom => "zone_bedroom.svg",
            Self::ZoneBathroom => "zone_bathroom.svg",
            Self::ZoneKitchen => "zone_kitchen.svg",
        }
    }
}

impl fmt::Display for SvgAssetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.file_name())
    }
}

#[derive(Debug, Clone)]
pub struct ZoneDef {
    pub id: String,
    pub room_label: String,
    pub zone_svg: SvgAssetKey,
    pub temp_c: f32,
    pub humidity_pct: f32,
    pub hp: u8,
    pub slots: Vec<SlotDef>,
}

#[derive(Debug, Clone)]
pub struct RoomNode {
    pub id: String,
    pub label: String,
    pub zone_id: String,
}

#[derive(Debug, Clone)]
pub struct MapDef {
    pub id: String,
    pub label: String,
    pub map_svg: SvgAssetKey,
    pub rooms: Vec<RoomNode>,
}

#[derive(Resource, Debug, Clone)]
pub struct GameProtocol {
    pub map: MapDef,
    pub zones: HashMap<String, ZoneDef>,
}

#[derive(Resource, Debug, Clone)]
pub struct SvgCatalog {
    pub map_scene: SvgScene,
    pub zone_scenes: HashMap<SvgAssetKey, SvgScene>,
}

#[derive(Resource, Debug, Clone)]
pub struct UiSelection {
    pub room_id: String,
    pub slot_id: Option<String>,
}

impl UiSelection {
    pub fn current_zone<'a>(&self, protocol: &'a GameProtocol) -> Option<&'a ZoneDef> {
        let room = protocol.map.rooms.iter().find(|r| r.id == self.room_id)?;
        protocol.zones.get(&room.zone_id)
    }

    pub fn current_zone_mut<'a>(&self, protocol: &'a mut GameProtocol) -> Option<&'a mut ZoneDef> {
        let zone_id = protocol
            .map
            .rooms
            .iter()
            .find(|r| r.id == self.room_id)
            .map(|r| r.zone_id.clone())?;
        protocol.zones.get_mut(&zone_id)
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct GameLog {
    pub entries: Vec<String>,
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerStatus {
    pub name: String,
    pub avatar: String,
    pub hp: u8,
    pub stamina: u8,
    pub condition: String,
    pub inventory: Vec<ItemDef>,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            name: "Operator".to_string(),
            avatar: "[@]".to_string(),
            hp: 94,
            stamina: 81,
            condition: "Focused".to_string(),
            inventory: Vec::new(),
        }
    }
}

impl GameLog {
    pub fn push(&mut self, line: impl Into<String>) {
        self.entries.push(line.into());
        if self.entries.len() > 100 {
            let drop_count = self.entries.len() - 100;
            self.entries.drain(0..drop_count);
        }
    }
}

#[derive(Message, Debug, Clone)]
pub enum GameAction {
    InspectSlot { slot_id: String },
    LootFirstItem { slot_id: String },
    RepairCurrentZone,
}

fn setup_game_protocol(mut commands: Commands) {
    let map_svg = include_str!("../../assets/svg/house_4b2b1k.svg");
    let bedroom_svg = include_str!("../../assets/svg/zone_bedroom.svg");
    let bathroom_svg = include_str!("../../assets/svg/zone_bathroom.svg");
    let kitchen_svg = include_str!("../../assets/svg/zone_kitchen.svg");

    let map_scene = SvgScene::parse(map_svg).expect("map SVG should parse");
    let mut zone_scenes = HashMap::new();
    zone_scenes.insert(
        SvgAssetKey::ZoneBedroom,
        SvgScene::parse(bedroom_svg).expect("bedroom SVG should parse"),
    );
    zone_scenes.insert(
        SvgAssetKey::ZoneBathroom,
        SvgScene::parse(bathroom_svg).expect("bathroom SVG should parse"),
    );
    zone_scenes.insert(
        SvgAssetKey::ZoneKitchen,
        SvgScene::parse(kitchen_svg).expect("kitchen SVG should parse"),
    );

    let map = MapDef {
        id: "map_house_4b2b1k".to_string(),
        label: "4B2B1K House".to_string(),
        map_svg: SvgAssetKey::MapHouse4b2b1k,
        rooms: vec![
            RoomNode {
                id: "room_bed_1".to_string(),
                label: "Bedroom 1".to_string(),
                zone_id: "zone_bed_1".to_string(),
            },
            RoomNode {
                id: "room_bed_2".to_string(),
                label: "Bedroom 2".to_string(),
                zone_id: "zone_bed_2".to_string(),
            },
            RoomNode {
                id: "room_bed_3".to_string(),
                label: "Bedroom 3".to_string(),
                zone_id: "zone_bed_3".to_string(),
            },
            RoomNode {
                id: "room_bed_4".to_string(),
                label: "Bedroom 4".to_string(),
                zone_id: "zone_bed_4".to_string(),
            },
            RoomNode {
                id: "room_bath_1".to_string(),
                label: "Bathroom 1".to_string(),
                zone_id: "zone_bath_1".to_string(),
            },
            RoomNode {
                id: "room_bath_2".to_string(),
                label: "Bathroom 2".to_string(),
                zone_id: "zone_bath_2".to_string(),
            },
            RoomNode {
                id: "room_kitchen_1".to_string(),
                label: "Kitchen".to_string(),
                zone_id: "zone_kitchen_1".to_string(),
            },
        ],
    };

    let mut zones = HashMap::new();
    zones.insert(
        "zone_bed_1".to_string(),
        demo_bedroom_zone("zone_bed_1", "Bedroom 1", 22.4, 44.0, 92),
    );
    zones.insert(
        "zone_bed_2".to_string(),
        demo_bedroom_zone("zone_bed_2", "Bedroom 2", 24.1, 42.0, 89),
    );
    zones.insert(
        "zone_bed_3".to_string(),
        demo_bedroom_zone("zone_bed_3", "Bedroom 3", 23.0, 49.0, 84),
    );
    zones.insert(
        "zone_bed_4".to_string(),
        demo_bedroom_zone("zone_bed_4", "Bedroom 4", 21.8, 46.0, 97),
    );
    zones.insert(
        "zone_bath_1".to_string(),
        demo_bathroom_zone("zone_bath_1", "Bathroom 1", 26.5, 68.0, 78),
    );
    zones.insert(
        "zone_bath_2".to_string(),
        demo_bathroom_zone("zone_bath_2", "Bathroom 2", 25.8, 65.0, 83),
    );
    zones.insert(
        "zone_kitchen_1".to_string(),
        demo_kitchen_zone("zone_kitchen_1", "Kitchen", 27.2, 55.0, 86),
    );

    commands.insert_resource(GameProtocol { map, zones });
    commands.insert_resource(SvgCatalog {
        map_scene,
        zone_scenes,
    });
    commands.insert_resource(UiSelection {
        room_id: "room_bed_1".to_string(),
        slot_id: None,
    });
}

fn demo_bedroom_zone(id: &str, label: &str, temp_c: f32, humidity_pct: f32, hp: u8) -> ZoneDef {
    ZoneDef {
        id: id.to_string(),
        room_label: label.to_string(),
        zone_svg: SvgAssetKey::ZoneBedroom,
        temp_c,
        humidity_pct,
        hp,
        slots: vec![
            SlotDef {
                id: "slot_bed".to_string(),
                name: "Bed Side".to_string(),
                action_hint: "Search around bed".to_string(),
                items: vec![ItemDef {
                    id: "item_keycard".to_string(),
                    name: "Blue Keycard".to_string(),
                    kind: "Access".to_string(),
                    quality: 77,
                }],
            },
            SlotDef {
                id: "slot_desk".to_string(),
                name: "Desk".to_string(),
                action_hint: "Inspect drawers".to_string(),
                items: vec![ItemDef {
                    id: "item_note".to_string(),
                    name: "Maintenance Note".to_string(),
                    kind: "Document".to_string(),
                    quality: 100,
                }],
            },
            SlotDef {
                id: "slot_window".to_string(),
                name: "Window".to_string(),
                action_hint: "Check frame integrity".to_string(),
                items: vec![],
            },
        ],
    }
}

fn demo_bathroom_zone(id: &str, label: &str, temp_c: f32, humidity_pct: f32, hp: u8) -> ZoneDef {
    ZoneDef {
        id: id.to_string(),
        room_label: label.to_string(),
        zone_svg: SvgAssetKey::ZoneBathroom,
        temp_c,
        humidity_pct,
        hp,
        slots: vec![
            SlotDef {
                id: "slot_sink".to_string(),
                name: "Sink".to_string(),
                action_hint: "Check leakage".to_string(),
                items: vec![ItemDef {
                    id: "item_wrench".to_string(),
                    name: "Mini Wrench".to_string(),
                    kind: "Tool".to_string(),
                    quality: 81,
                }],
            },
            SlotDef {
                id: "slot_shower".to_string(),
                name: "Shower".to_string(),
                action_hint: "Inspect pressure valve".to_string(),
                items: vec![],
            },
            SlotDef {
                id: "slot_cabinet".to_string(),
                name: "Cabinet".to_string(),
                action_hint: "Open for supplies".to_string(),
                items: vec![ItemDef {
                    id: "item_filter".to_string(),
                    name: "Water Filter".to_string(),
                    kind: "Component".to_string(),
                    quality: 68,
                }],
            },
        ],
    }
}

fn demo_kitchen_zone(id: &str, label: &str, temp_c: f32, humidity_pct: f32, hp: u8) -> ZoneDef {
    ZoneDef {
        id: id.to_string(),
        room_label: label.to_string(),
        zone_svg: SvgAssetKey::ZoneKitchen,
        temp_c,
        humidity_pct,
        hp,
        slots: vec![
            SlotDef {
                id: "slot_stove".to_string(),
                name: "Stove".to_string(),
                action_hint: "Inspect burners".to_string(),
                items: vec![],
            },
            SlotDef {
                id: "slot_sink_k".to_string(),
                name: "Kitchen Sink".to_string(),
                action_hint: "Inspect drainage".to_string(),
                items: vec![ItemDef {
                    id: "item_valve".to_string(),
                    name: "Spare Valve".to_string(),
                    kind: "Component".to_string(),
                    quality: 73,
                }],
            },
            SlotDef {
                id: "slot_fridge".to_string(),
                name: "Fridge".to_string(),
                action_hint: "Check coolant".to_string(),
                items: vec![ItemDef {
                    id: "item_battery".to_string(),
                    name: "Backup Battery".to_string(),
                    kind: "Power".to_string(),
                    quality: 60,
                }],
            },
        ],
    }
}

fn apply_game_actions(
    mut actions: MessageReader<GameAction>,
    mut protocol: ResMut<GameProtocol>,
    mut player: ResMut<PlayerStatus>,
    selection: Res<UiSelection>,
    mut log: ResMut<GameLog>,
) {
    for action in actions.read() {
        match action {
            GameAction::InspectSlot { slot_id } => {
                if let Some(zone) = selection.current_zone(&protocol) {
                    if let Some(slot) = zone.slots.iter().find(|s| s.id == *slot_id) {
                        log.push(format!(
                            "Inspect {} / {}: {} item(s)",
                            zone.room_label,
                            slot.name,
                            slot.items.len()
                        ));
                    }
                }
            }
            GameAction::LootFirstItem { slot_id } => {
                if let Some(zone) = selection.current_zone_mut(&mut protocol)
                    && let Some(slot) = zone.slots.iter_mut().find(|s| s.id == *slot_id)
                {
                    if let Some(item) = slot.items.first().cloned() {
                        slot.items.remove(0);
                        player.inventory.push(item.clone());
                        log.push(format!(
                            "Looted [{}] {} from {}",
                            item.kind, item.name, slot.name
                        ));
                    } else {
                        log.push(format!("No item left in {}", slot.name));
                    }
                }
            }
            GameAction::RepairCurrentZone => {
                if let Some(zone) = selection.current_zone_mut(&mut protocol) {
                    let old = zone.hp;
                    zone.hp = zone.hp.saturating_add(10).min(100);
                    log.push(format!(
                        "Repair {}: HP {} -> {}",
                        zone.room_label, old, zone.hp
                    ));
                }
            }
        }
    }
}
