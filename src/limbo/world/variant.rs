use crate::limbo::nbt::{NbtTag, NbtTagType};

use super::helpers::{compound, int, string};
use super::registry::{registry, simple_variant_registry, variant_registry_entry};

pub fn painting_variant_registry() -> NbtTag {
    simple_variant_registry(
        "minecraft:painting_variant",
        "minecraft:alban",
        compound(vec![
            int("width", 1),
            int("height", 1),
            string("asset_id", "minecraft:alban"),
        ]),
    )
}

pub fn entity_variant_registries() -> Vec<(String, NbtTag)> {
    let empty_conditions = || NbtTag::List(NbtTagType::Compound, vec![]);
    let variant = |fields: Vec<(String, NbtTag)>| compound(fields);
    let sound_variant = compound(vec![
        string("ambient_sound", "minecraft:entity.wolf_angry.ambient"),
        string("death_sound", "minecraft:entity.wolf_angry.death"),
        string("growl_sound", "minecraft:entity.wolf_angry.growl"),
        string("hurt_sound", "minecraft:entity.wolf_angry.hurt"),
        string("pant_sound", "minecraft:entity.wolf_angry.pant"),
        string("whine_sound", "minecraft:entity.wolf_angry.whine"),
    ]);
    let wolf_assets = || {
        compound(vec![
            string("wild", "minecraft:entity/wolf/wolf_ashen"),
            string("tame", "minecraft:entity/wolf/wolf_ashen_tame"),
            string("angry", "minecraft:entity/wolf/wolf_ashen_angry"),
        ])
    };

    vec![
        (
            "minecraft:cat_variant".to_owned(),
            simple_variant_registry(
                "minecraft:cat_variant",
                "minecraft:all_black",
                variant(vec![
                    string("asset_id", "minecraft:entity/cat/all_black"),
                    ("spawn_conditions".to_owned(), empty_conditions()),
                ]),
            ),
        ),
        (
            "minecraft:chicken_variant".to_owned(),
            registry(
                "minecraft:chicken_variant",
                ["cold", "temperate", "warm"]
                    .into_iter()
                    .enumerate()
                    .map(|(id, name)| {
                        variant_registry_entry(
                            name,
                            id as i32,
                            variant(vec![
                                string("asset_id", "minecraft:entity/chicken/cold_chicken"),
                                string("model", "cold"),
                                ("spawn_conditions".to_owned(), empty_conditions()),
                            ]),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "minecraft:cow_variant".to_owned(),
            simple_variant_registry(
                "minecraft:cow_variant",
                "minecraft:cold",
                variant(vec![
                    string("asset_id", "minecraft:entity/cow/cold_cow"),
                    string("model", "cold"),
                    ("spawn_conditions".to_owned(), empty_conditions()),
                ]),
            ),
        ),
        (
            "minecraft:frog_variant".to_owned(),
            simple_variant_registry(
                "minecraft:frog_variant",
                "minecraft:cold",
                variant(vec![
                    string("asset_id", "minecraft:entity/frog/cold_frog"),
                    ("spawn_conditions".to_owned(), empty_conditions()),
                ]),
            ),
        ),
        (
            "minecraft:pig_variant".to_owned(),
            simple_variant_registry(
                "minecraft:pig_variant",
                "minecraft:cold",
                variant(vec![
                    string("asset_id", "minecraft:entity/pig/cold_pig"),
                    string("model", "cold"),
                    ("spawn_conditions".to_owned(), empty_conditions()),
                ]),
            ),
        ),
        (
            "minecraft:wolf_sound_variant".to_owned(),
            simple_variant_registry(
                "minecraft:wolf_sound_variant",
                "minecraft:angry",
                sound_variant,
            ),
        ),
        (
            "minecraft:wolf_variant".to_owned(),
            simple_variant_registry(
                "minecraft:wolf_variant",
                "minecraft:ashen",
                variant(vec![
                    ("assets".to_owned(), wolf_assets()),
                    ("baby_assets".to_owned(), wolf_assets()),
                    ("spawn_conditions".to_owned(), empty_conditions()),
                ]),
            ),
        ),
    ]
}
