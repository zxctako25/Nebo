#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(NbtTagType, Vec<NbtTag>),
    Compound(Vec<(String, NbtTag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NbtTagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl NbtTag {
    pub fn tag_type(&self) -> NbtTagType {
        match self {
            Self::Byte(_) => NbtTagType::Byte,
            Self::Short(_) => NbtTagType::Short,
            Self::Int(_) => NbtTagType::Int,
            Self::Long(_) => NbtTagType::Long,
            Self::Float(_) => NbtTagType::Float,
            Self::Double(_) => NbtTagType::Double,
            Self::ByteArray(_) => NbtTagType::ByteArray,
            Self::String(_) => NbtTagType::String,
            Self::List(_, _) => NbtTagType::List,
            Self::Compound(_) => NbtTagType::Compound,
            Self::IntArray(_) => NbtTagType::IntArray,
            Self::LongArray(_) => NbtTagType::LongArray,
        }
    }
}
