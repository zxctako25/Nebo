use crate::limbo::nbt::NbtTag;

use super::helpers::{boolean, compound, double, float, int, string};

#[derive(Clone, Copy)]
pub enum BiomeKind {
    Plains,
    Swamp,
    SwampHills,
    NetherWastes,
    TheEnd,
}

pub fn biome_entry(protocol: i32, name: &str, id: i32, kind: BiomeKind) -> NbtTag {
    compound(vec![
        string("name", name),
        int("id", id),
        ("element".to_owned(), biome_data(protocol, kind)),
    ])
}

pub fn biome_data(protocol: i32, kind: BiomeKind) -> NbtTag {
    let (has_precipitation, depth, temperature, scale, downfall, category, colors) = match kind {
        BiomeKind::Plains => (
            true,
            0.125,
            0.8,
            0.05,
            0.4,
            "plains",
            (7_907_327, 329_011, 12_638_463, 415_920),
        ),
        BiomeKind::Swamp => (
            true,
            -0.2,
            0.8,
            0.1,
            0.9,
            "swamp",
            (7_907_327, 329_011, 12_638_463, 415_920),
        ),
        BiomeKind::SwampHills => (
            true,
            -0.1,
            0.8,
            0.3,
            0.9,
            "swamp",
            (7_907_327, 329_011, 12_638_463, 415_920),
        ),
        BiomeKind::NetherWastes => (
            false,
            0.1,
            2.0,
            0.2,
            0.0,
            "nether",
            (7_254_527, 329_011, 3_344_392, 4_159_204),
        ),
        BiomeKind::TheEnd => (
            false,
            0.1,
            0.5,
            0.2,
            0.5,
            "the_end",
            (0, 10_518_688, 12_638_463, 4_159_204),
        ),
    };
    let mut effects = vec![
        int("sky_color", colors.0),
        int("water_fog_color", colors.3),
        int("fog_color", colors.2),
        int("water_color", colors.3),
        (
            "mood_sound".to_owned(),
            compound(vec![
                int("tick_delay", 6_000),
                double("offset", 2.0),
                int("block_search_extent", 8),
                string(
                    "sound",
                    if matches!(kind, BiomeKind::NetherWastes) {
                        "minecraft:ambient.nether_wastes.mood"
                    } else {
                        "minecraft:ambient.cave"
                    },
                ),
            ]),
        ),
    ];
    if matches!(kind, BiomeKind::Swamp | BiomeKind::SwampHills) {
        effects.push(int("foliage_color", 6_975_545));
        effects.push(string("grass_color_modifier", "swamp"));
    }

    let mut fields = vec![
        float("depth", depth),
        float("temperature", temperature),
        float("scale", scale),
        float("downfall", downfall),
        string("category", category),
        ("effects".to_owned(), compound(effects)),
    ];
    if protocol < 762 {
        fields.push(string(
            "precipitation",
            if has_precipitation { "rain" } else { "none" },
        ));
    } else {
        fields.push(boolean("has_precipitation", has_precipitation));
    }
    compound(fields)
}
