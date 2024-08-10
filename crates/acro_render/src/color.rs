use serde::{ser::SerializeTuple, Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Color {
    Srgba(Srgba),
}

impl Color {
    pub fn to_srgba(&self) -> Srgba {
        match self {
            Color::Srgba(srgba) => *srgba,
        }
    }
}

// All of the fields are [0.0, 1.0]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Srgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Srgba {
    pub const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Srgba = Srgba::new(0.0, 0.0, 0.0, 1.0);

    pub const RED: Srgba = Srgba::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Srgba = Srgba::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Srgba = Srgba::new(0.0, 0.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl Serialize for Srgba {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_tuple(4)?;
        seq.serialize_element(&self.r)?;
        seq.serialize_element(&self.g)?;
        seq.serialize_element(&self.b)?;
        seq.serialize_element(&self.a)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Srgba {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SrgbaVisitor;

        impl<'de> serde::de::Visitor<'de> for SrgbaVisitor {
            type Value = Srgba;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple of 4 elements")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let r = seq.next_element()?.unwrap();
                let g = seq.next_element()?.unwrap();
                let b = seq.next_element()?.unwrap();
                let a = seq.next_element()?.unwrap();

                Ok(Srgba::new(r, g, b, a))
            }
        }

        deserializer.deserialize_tuple(4, SrgbaVisitor)
    }
}
