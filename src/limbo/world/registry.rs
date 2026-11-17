use bytes::{BufMut, BytesMut};

use crate::limbo::nbt::{write_network_tag, NbtTag, NbtTagType};
use crate::protocol::io::{ProtocolUtils, VarInt};

use super::biome::{biome_data, biome_entry, BiomeKind};
use super::damage::damage_type_registry;
use super::helpers::{compound, float, int, string, string_list};
use super::variant::{entity_variant_registries, painting_variant_registry};

#[derive(Clone, Copy)]
pub enum DimensionKind {
    Overworld,
    Nether,
    End,
}
pub fn builtin_registry(protocol: i32) -> NbtTag {
    let dimension = registry(
        "minecraft:dimension_type",
        [
            ("minecraft:overworld", 0, DimensionKind::Overworld),
            ("minecraft:the_nether", 1, DimensionKind::Nether),
            ("minecraft:the_end", 2, DimensionKind::End),
        ]
            .into_iter()
            .map(|(name, id, kind)| {
                compound(vec![
                    string("name", name),
                    int("id", id),
                    ("element".to_owned(), dimension_data(protocol, kind)),
                ])
            })
            .collect(),
    );
    let biome = registry(
        "minecraft:worldgen/biome",
        [
            biome_entry(protocol, "minecraft:plains", 1, BiomeKind::Plains),
            biome_entry(protocol, "minecraft:swamp", 6, BiomeKind::Swamp),
            biome_entry(
                protocol,
                "minecraft:swamp_hills",
                134,
                BiomeKind::SwampHills,
            ),
            biome_entry(
                protocol,
                "minecraft:nether_wastes",
                8,
                BiomeKind::NetherWastes,
            ),
            biome_entry(protocol, "minecraft:the_end", 9, BiomeKind::TheEnd),
        ]
            .into_iter()
            .collect(),
    );
    let mut registries = vec![
        ("minecraft:dimension_type".to_owned(), dimension),
        ("minecraft:worldgen/biome".to_owned(), biome),
    ];
    if protocol >= 759 {
        registries.push((
            "minecraft:chat_type".to_owned(),
            chat_type_registry(protocol),
        ));
    }
    if protocol >= 762 {
        registries.push((
            "minecraft:damage_type".to_owned(),
            damage_type_registry(protocol),
        ));
    }
    if protocol >= 767 {
        registries.push((
            "minecraft:painting_variant".to_owned(),
            painting_variant_registry(),
        ));
    }
    if protocol >= 770 {
        registries.extend(entity_variant_registries());
    }
    if protocol >= 774 {
        registries.push((
            "minecraft:zombie_nautilus_variant".to_owned(),
            simple_variant_registry(
                "minecraft:zombie_nautilus_variant",
                "minecraft:temperate",
                compound(vec![
                    string("asset_id", "minecraft:entity/nautilus/zombie_nautilus"),
                    (
                        "spawn_conditions".to_owned(),
                        NbtTag::List(NbtTagType::Compound, vec![]),
                    ),
                ]),
            ),
        ));
    }
    compound(registries)
}

fn chat_type_registry(protocol: i32) -> NbtTag {
    let values = if protocol == 759 {
        vec![
            compound(vec![
                string("name", "minecraft:chat"),
                int("id", 0),
                (
                    "element".to_owned(),
                    compound(vec![
                        (
                            "chat".to_owned(),
                            compound(vec![(
                                "decoration".to_owned(),
                                compound(vec![
                                    string("translation_key", "chat.type.text"),
                                    ("style".to_owned(), compound(vec![])),
                                    ("parameters".to_owned(), string_list(&["sender", "content"])),
                                ]),
                            )]),
                        ),
                        (
                            "narration".to_owned(),
                            compound(vec![
                                string("priority", "chat"),
                                (
                                    "decoration".to_owned(),
                                    compound(vec![
                                        string("translation_key", "chat.type.text.narrate"),
                                        ("style".to_owned(), compound(vec![])),
                                        (
                                            "parameters".to_owned(),
                                            string_list(&["sender", "content"]),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                    ]),
                ),
            ]),
            compound(vec![
                string("name", "minecraft:system"),
                int("id", 1),
                (
                    "element".to_owned(),
                    compound(vec![
                        ("chat".to_owned(), compound(vec![])),
                        (
                            "narration".to_owned(),
                            compound(vec![string("priority", "system")]),
                        ),
                    ]),
                ),
            ]),
            compound(vec![
                string("name", "minecraft:game_info"),
                int("id", 2),
                (
                    "element".to_owned(),
                    compound(vec![("overlay".to_owned(), compound(vec![]))]),
                ),
            ]),
        ]
    } else {
        vec![compound(vec![
            string("name", "minecraft:chat"),
            int("id", 1),
            (
                "element".to_owned(),
                compound(vec![
                    (
                        "chat".to_owned(),
                        compound(vec![
                            ("parameters".to_owned(), string_list(&["sender", "content"])),
                            string("translation_key", "chat.type.text"),
                        ]),
                    ),
                    (
                        "narration".to_owned(),
                        compound(vec![
                            ("parameters".to_owned(), string_list(&["sender", "content"])),
                            string("translation_key", "chat.type.text.narrate"),
                        ]),
                    ),
                ]),
            ),
        ])]
    };
    registry("minecraft:chat_type", values)
}

pub fn dimension_data(protocol: i32, kind: DimensionKind) -> NbtTag {
    let (key, skylight, ceiling, ultrawarm, piglin_safe, respawn_anchor, logical_height, scale) =
        match kind {
            DimensionKind::Overworld => (
                "minecraft:overworld",
                true,
                false,
                false,
                false,
                false,
                if protocol >= 757 { 384 } else { 256 },
                1.0,
            ),
            DimensionKind::Nether => (
                "minecraft:the_nether",
                false,
                true,
                true,
                true,
                true,
                if protocol >= 757 { 384 } else { 256 },
                8.0,
            ),
            DimensionKind::End => (
                "minecraft:the_end",
                false,
                false,
                false,
                false,
                false,
                if protocol >= 757 { 384 } else { 256 },
                1.0,
            ),
        };
    let infiniburn = match kind {
        DimensionKind::Nether => "minecraft:infiniburn_nether",
        DimensionKind::End => "minecraft:infiniburn_end",
        DimensionKind::Overworld => "minecraft:infiniburn_overworld",
    };
    let mut fields = vec![
        super::helpers::boolean("natural", false),
        float("ambient_light", 0.0),
        super::helpers::boolean("shrunk", false),
        super::helpers::boolean("ultrawarm", ultrawarm),
        super::helpers::boolean("has_ceiling", ceiling),
        super::helpers::boolean("has_skylight", skylight),
        super::helpers::boolean("piglin_safe", piglin_safe),
        super::helpers::boolean("bed_works", false),
        super::helpers::boolean("respawn_anchor_works", respawn_anchor),
        super::helpers::boolean("has_raids", false),
        int("logical_height", logical_height),
        string(
            "infiniburn",
            if protocol >= 758 {
                match kind {
                    DimensionKind::Nether => "#minecraft:infiniburn_nether",
                    DimensionKind::End => "#minecraft:infiniburn_end",
                    DimensionKind::Overworld => "#minecraft:infiniburn_overworld",
                }
            } else {
                infiniburn
            },
        ),
        super::helpers::double("coordinate_scale", scale),
        string("effects", key),
        int("min_y", if protocol >= 757 { -64 } else { 0 }),
        int("height", if protocol >= 757 { 384 } else { 256 }),
        int("monster_spawn_block_light_limit", 0),
        int("monster_spawn_light_level", 0),
    ];

    if protocol >= 774 {
        fields.extend(dimension_attributes(kind));
    }
    compound(fields)
}

fn dimension_attributes(kind: DimensionKind) -> Vec<(String, NbtTag)> {
    let mut dimension_fields = Vec::new();
    let attributes = match kind {
        DimensionKind::Overworld => vec![
            string("minecraft:visual/cloud_color", "#ccffffff"),
            float("minecraft:visual/cloud_height", 192.33),
            string("minecraft:visual/fog_color", "#c0d8ff"),
            string("minecraft:visual/sky_color", "#78a7ff"),
        ],
        DimensionKind::Nether => {
            dimension_fields.push(string("cardinal_light", "nether"));
            dimension_fields.push(string("skybox", "none"));
            vec![
                float("minecraft:gameplay/sky_light_level", 4.0),
                float("minecraft:visual/fog_end_distance", 96.0),
                float("minecraft:visual/fog_start_distance", 10.0),
                string("minecraft:visual/sky_light_color", "#7a7aff"),
            ]
        }
        DimensionKind::End => {
            dimension_fields.push(string("skybox", "end"));
            vec![
                string("minecraft:visual/fog_color", "#181318"),
                string("minecraft:visual/sky_color", "#000000"),
                string("minecraft:visual/sky_light_color", "#e580ff"),
            ]
        }
    };
    dimension_fields.push(("attributes".to_owned(), compound(attributes)));
    dimension_fields
}

pub fn registry(registry_type: &str, values: Vec<NbtTag>) -> NbtTag {
    compound(vec![
        string("type", registry_type),
        (
            "value".to_owned(),
            NbtTag::List(NbtTagType::Compound, values),
        ),
    ])
}

pub fn simple_variant_registry(registry_type: &str, name: &str, element: NbtTag) -> NbtTag {
    registry(
        registry_type,
        vec![variant_registry_entry(name, 0, element)],
    )
}

pub fn variant_registry_entry(name: &str, id: i32, element: NbtTag) -> NbtTag {
    let resource_name = if name.contains(':') {
        name.to_owned()
    } else {
        format!("minecraft:{name}")
    };
    compound(vec![
        string("name", &resource_name),
        int("id", id),
        ("element".to_owned(), element),
    ])
}

pub fn configuration_packets(protocol: i32) -> anyhow::Result<Vec<BytesMut>> {
    anyhow::ensure!(
        (764..=774).contains(&protocol),
        "configuration is not supported for {protocol}"
    );
    let mut packets = Vec::new();
    let registry = builtin_registry(protocol);

    if protocol <= 765 {
        let mut packet = BytesMut::new();
        VarInt::write(&mut packet, 0x05); 
        packet.extend_from_slice(&write_network_tag(protocol, &registry)?);
        packets.push(packet);
    } else {
        let registries = compound_entries(&registry)?;
        for (key, value) in registries {
            let registry_type = compound_string(value, "type")?;
            let entries = compound_list(value, "value")?;
            let indexed = index_registry_entries(entries)?;

            let mut packet = BytesMut::new();
            VarInt::write(&mut packet, 0x07); 
            ProtocolUtils::write_string(&mut packet, registry_type);
            VarInt::write(&mut packet, i32::try_from(indexed.len())?);
            for (name, element) in indexed {
                ProtocolUtils::write_string(&mut packet, &name);
                if let Some(element) = element {
                    packet.put_u8(1);
                    packet.extend_from_slice(&write_network_tag(protocol, &element)?);
                } else {
                    packet.put_u8(0);
                }
            }
            debug_assert!(key.starts_with("minecraft:"));
            packets.push(packet);
        }
    }

    let mut tags = BytesMut::new();
    VarInt::write(
        &mut tags,
        if protocol <= 764 {
            0x08
        } else if protocol == 765 {
            0x09
        } else {
            0x0D
        },
    ); 
    VarInt::write(&mut tags, 0); 
    packets.push(tags);

    let mut finish = BytesMut::new();
    VarInt::write(&mut finish, if protocol <= 765 { 0x02 } else { 0x03 });
    packets.push(finish);
    Ok(packets)
}

fn compound_entries(tag: &NbtTag) -> anyhow::Result<&[(String, NbtTag)]> {
    match tag {
        NbtTag::Compound(entries) => Ok(entries),
        _ => anyhow::bail!("expected compound NBT tag"),
    }
}

fn compound_string<'a>(tag: &'a NbtTag, name: &str) -> anyhow::Result<&'a str> {
    match compound_entries(tag)?
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
    {
        Some(NbtTag::String(value)) => Ok(value),
        _ => anyhow::bail!("missing NBT string field {name}"),
    }
}

fn compound_int(tag: &NbtTag, name: &str) -> anyhow::Result<i32> {
    match compound_entries(tag)?
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
    {
        Some(NbtTag::Int(value)) => Ok(*value),
        _ => anyhow::bail!("missing NBT integer field {name}"),
    }
}

fn compound_list<'a>(tag: &'a NbtTag, name: &str) -> anyhow::Result<&'a [NbtTag]> {
    match compound_entries(tag)?
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
    {
        Some(NbtTag::List(_, values)) => Ok(values),
        _ => anyhow::bail!("missing NBT list field {name}"),
    }
}

fn index_registry_entries(entries: &[NbtTag]) -> anyhow::Result<Vec<(String, Option<NbtTag>)>> {
    let mut indexed = Vec::<Option<(String, NbtTag)>>::new();
    for entry in entries {
        let id = usize::try_from(compound_int(entry, "id")?)?;
        anyhow::ensure!(id < 4096, "registry entry id exceeds safe bound");
        if indexed.len() <= id {
            indexed.resize_with(id + 1, || None);
        }
        let name = compound_string(entry, "name")?.to_owned();
        let element = compound_entries(entry)?
            .iter()
            .find(|(key, _)| key == "element")
            .map(|(_, value)| value.clone())
            .ok_or_else(|| anyhow::anyhow!("registry entry has no element"))?;
        indexed[id] = Some((name, element));
    }
    let first_element = indexed
        .iter()
        .find_map(|entry| entry.as_ref().map(|(_, tag)| tag.clone()))
        .ok_or_else(|| anyhow::anyhow!("registry is empty"))?;
    Ok(indexed
        .into_iter()
        .enumerate()
        .map(|(id, entry)| match entry {
            Some((name, element)) => (name, Some(element)),
            None => (
                format!("limboapi_padding_{id}"),
                Some(first_element.clone()),
            ),
        })
        .collect())
}
