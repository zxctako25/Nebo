use crate::limbo::nbt::{NbtTag, NbtTagType};

pub fn compound(fields: Vec<(String, NbtTag)>) -> NbtTag {
    NbtTag::Compound(fields)
}

pub fn string(name: &str, value: &str) -> (String, NbtTag) {
    (name.to_owned(), NbtTag::String(value.to_owned()))
}

pub fn string_list(values: &[&str]) -> NbtTag {
    NbtTag::List(
        NbtTagType::String,
        values
            .iter()
            .map(|value| NbtTag::String((*value).to_owned()))
            .collect(),
    )
}

pub fn boolean(name: &str, value: bool) -> (String, NbtTag) {
    (name.to_owned(), NbtTag::Byte(i8::from(value)))
}

pub fn int(name: &str, value: i32) -> (String, NbtTag) {
    (name.to_owned(), NbtTag::Int(value))
}

pub fn float(name: &str, value: f32) -> (String, NbtTag) {
    (name.to_owned(), NbtTag::Float(value))
}

pub fn double(name: &str, value: f64) -> (String, NbtTag) {
    (name.to_owned(), NbtTag::Double(value))
}
