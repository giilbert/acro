use std::fmt::Debug;

use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, Copy, Default)]
pub enum Dim {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PositioningOptions {
    #[serde(default)]
    pub width: Dim,
    #[serde(default)]
    pub height: Dim,

    #[serde(default)]
    pub min_width: Option<Dim>,
    #[serde(default)]
    pub min_height: Option<Dim>,

    #[serde(default)]
    pub max_width: Option<Dim>,
    #[serde(default)]
    pub max_height: Option<Dim>,

    #[serde(default)]
    pub padding: DirDim,
    #[serde(default)]
    pub margin: DirDim,
    #[serde(default)]
    pub flex: FlexOptions,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct FlexOptions {
    #[serde(default)]
    pub direction: FlexDirection,
    #[serde(default)]
    pub gap: Dim,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    // RowReverse,
    #[default]
    Column,
    // ColumnReverse,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DirDim {
    pub left: Dim,
    pub right: Dim,
    pub top: Dim,
    pub bottom: Dim,
}

impl DirDim {
    pub fn all(val: Dim) -> DirDim {
        DirDim {
            left: val,
            right: val,
            top: val,
            bottom: val,
        }
    }

    pub fn xy(x: Dim, y: Dim) -> DirDim {
        Self {
            left: x,
            right: x,
            top: y,
            bottom: y,
        }
    }
}

impl Serialize for Dim {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = match self {
            Dim::Auto => "auto".to_string(),
            Dim::Px(val) => format!("{}px", val),
            Dim::Percent(val) => format!("{}%", val * 100.0),
        };
        serializer.serialize_str(&data)
    }
}

impl<'de> Deserialize<'de> for Dim {
    fn deserialize<D>(deserializer: D) -> Result<Dim, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DimVisitor;

        impl<'de> Visitor<'de> for DimVisitor {
            type Value = Dim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a dimensional value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "auto" {
                    Ok(Dim::Auto)
                } else {
                    let first_alpha = v
                        .chars()
                        .take_while(|c| !c.is_ascii_alphabetic() && *c != '%')
                        .count();
                    let (val, unit) = v.split_at(first_alpha);
                    let val = val.parse().map_err(de::Error::custom)?;

                    match unit {
                        "px" => Ok(Dim::Px(val)),
                        "%" => Ok(Dim::Percent(val / 100.0)),
                        _ => Err(de::Error::custom("invalid unit")),
                    }
                }
            }
        }

        deserializer.deserialize_string(DimVisitor)
    }
}

impl Serialize for DirDim {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.top)?;
        tup.serialize_element(&self.right)?;
        tup.serialize_element(&self.bottom)?;
        tup.serialize_element(&self.left)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for DirDim {
    fn deserialize<D>(deserializer: D) -> Result<DirDim, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DirDimVisitor;

        impl<'de> Visitor<'de> for DirDimVisitor {
            type Value = DirDim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple of 4 elements")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let left = seq.next_element()?.unwrap();
                let right = seq.next_element()?.unwrap();
                let top = seq.next_element()?.unwrap();
                let bottom = seq.next_element()?.unwrap();

                Ok(DirDim {
                    left,
                    right,
                    top,
                    bottom,
                })
            }
        }

        deserializer.deserialize_tuple(4, DirDimVisitor)
    }
}
