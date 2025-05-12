use chrono::Duration;
use serde::{de, ser, ser::SerializeTuple, Deserialize, Deserializer, Serializer};
use shuuro::{Color, SubVariant, Variant};
use std::{
    fmt::{self, Display},
    time::Duration as StdD,
};

pub fn duration_to_u64<S>(x: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration = x.num_milliseconds() as u64;
    s.serialize_u64(duration)
}

pub fn str_to_duration<'de, D>(data: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u64 = Deserialize::deserialize(data)?;
    let d2 = StdD::from_millis(s);
    if let Ok(d2) = Duration::from_std(d2) {
        return Ok(d2);
    }
    Ok(Duration::minutes(1))
}

pub fn duration_to_array<S>(x: &[Duration; 2], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tup = s.serialize_tuple(2)?;
    for duration in x.iter() {
        let value = duration.num_milliseconds() as u64;
        tup.serialize_element(&value).ok();
    }
    Ok(tup.end().ok().unwrap())
}

pub fn array_to_duration<'de, D>(data: D) -> Result<[Duration; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let s: [u64; 2] = Deserialize::deserialize(data)?;
    let mut durations = [Duration::seconds(1); 2];
    for (i, u) in s.iter().enumerate() {
        let d2 = StdD::from_millis(*u);
        if let Ok(d) = Duration::from_std(d2) {
            durations[i] = d;
        }
    }
    Ok(durations)
}

pub fn serialize_subvariant<S>(
    x: &Option<SubVariant>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(x) = x {
        return s.serialize_u8(x.index() as u8);
    }
    s.serialize_u8(100_u8)
}

pub fn deserialize_subvariant<'de, D>(
    data: D,
) -> Result<Option<SubVariant>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Result<u8, <D as Deserializer<'_>>::Error> =
        Deserialize::deserialize(data);
    if let Ok(s) = s {
        if let Ok(sv) = SubVariant::try_from(s as u8) {
            return Ok(Some(sv));
        }
    }
    Ok(None)
}

pub fn serialize_variant<S>(x: &Variant, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u8((*x as usize) as u8)
}

pub fn deserialize_color<'de, D>(data: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let color: usize = Deserialize::deserialize(data)?;
    Ok(Color::from(color))
}

pub fn deserialize_variant<'de, D>(data: D) -> Result<Variant, D::Error>
where
    D: Deserializer<'de>,
{
    let variant: u8 = Deserialize::deserialize(data)?;
    let variant = match variant {
        0 => Variant::Shuuro,
        1 => Variant::ShuuroFairy,
        2 => Variant::Standard,
        3 => Variant::StandardFairy,
        4 => Variant::ShuuroMini,
        5 => Variant::ShuuroMiniFairy,
        _ => Variant::Shuuro,
    };
    Ok(variant)
}

#[derive(Debug)]
pub struct VariantError {}

impl de::Error for VariantError {
    fn custom<T>(_: T) -> Self
    where
        T: std::fmt::Display,
    {
        VariantError {}
    }
}

impl ser::Error for VariantError {
    fn custom<T>(_: T) -> Self
    where
        T: std::fmt::Display,
    {
        VariantError {}
    }
}
impl std::error::Error for VariantError {}

impl Display for VariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("")
    }
}
