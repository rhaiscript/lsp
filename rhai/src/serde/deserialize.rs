//! Implementations of [`serde::Deserialize`].

use crate::{Dynamic, ImmutableString, INT};
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use std::fmt;
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

#[cfg(not(feature = "no_index"))]
use crate::Array;

#[cfg(not(feature = "no_index"))]
use serde::de::SeqAccess;

#[cfg(not(feature = "no_object"))]
use crate::Map;

#[cfg(not(feature = "no_object"))]
use serde::de::MapAccess;

struct DynamicVisitor;

impl<'d> Visitor<'d> for DynamicVisitor {
    type Value = Dynamic;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any type that can be converted into a Dynamic")
    }
    fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_i8<E: Error>(self, v: i8) -> Result<Self::Value, E> {
        Ok(INT::from(v).into())
    }
    fn visit_i16<E: Error>(self, v: i16) -> Result<Self::Value, E> {
        Ok(INT::from(v).into())
    }
    fn visit_i32<E: Error>(self, v: i32) -> Result<Self::Value, E> {
        Ok(INT::from(v).into())
    }
    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        #[cfg(not(feature = "only_i32"))]
        {
            Ok(v.into())
        }
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as i64 {
            Ok(Dynamic::from(v))
        } else {
            self.visit_i32(v as i32)
        }
    }
    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        Ok(INT::from(v).into())
    }
    fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
        Ok(INT::from(v).into())
    }
    fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
        #[cfg(not(feature = "only_i32"))]
        {
            Ok(INT::from(v).into())
        }
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as u32 {
            Ok(Dynamic::from(v))
        } else {
            self.visit_i32(v as i32)
        }
    }
    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        #[cfg(not(feature = "only_i32"))]
        if v > i64::MAX as u64 {
            Ok(Dynamic::from(v))
        } else {
            self.visit_i64(v as i64)
        }
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as u64 {
            Ok(Dynamic::from(v))
        } else {
            self.visit_i32(v as i32)
        }
    }

    #[cfg(not(feature = "no_float"))]
    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        #[cfg(not(feature = "f32_float"))]
        return self.visit_f64(v as f64);
        #[cfg(feature = "f32_float")]
        return Ok(v.into());
    }
    #[cfg(not(feature = "no_float"))]
    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        #[cfg(not(feature = "f32_float"))]
        return Ok(v.into());
        #[cfg(feature = "f32_float")]
        return self.visit_f32(v as f32);
    }

    #[cfg(feature = "no_float")]
    #[cfg(feature = "decimal")]
    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        use rust_decimal::Decimal;
        use std::convert::TryFrom;

        Decimal::try_from(v)
            .map(|v| v.into())
            .map_err(Error::custom)
    }
    #[cfg(feature = "no_float")]
    #[cfg(feature = "decimal")]
    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        use rust_decimal::Decimal;
        use std::convert::TryFrom;

        Decimal::try_from(v)
            .map(|v| v.into())
            .map_err(Error::custom)
    }

    fn visit_char<E: Error>(self, v: char) -> Result<Self::Value, E> {
        self.visit_string(v.to_string())
    }
    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_borrowed_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_str(v)
    }
    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(v.into())
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(Dynamic::UNIT)
    }

    fn visit_newtype_struct<D: Deserializer<'d>>(self, de: D) -> Result<Self::Value, D::Error> {
        Deserialize::deserialize(de)
    }

    #[cfg(not(feature = "no_index"))]
    fn visit_seq<A: SeqAccess<'d>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut arr: Array = Default::default();

        while let Some(v) = seq.next_element()? {
            arr.push(v);
        }

        Ok(arr.into())
    }

    #[cfg(not(feature = "no_object"))]
    fn visit_map<M: MapAccess<'d>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut m: Map = Default::default();

        while let Some((k, v)) = map.next_entry::<&str, _>()? {
            m.insert(k.into(), v);
        }

        Ok(m.into())
    }
}

impl<'d> Deserialize<'d> for Dynamic {
    fn deserialize<D: Deserializer<'d>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_any(DynamicVisitor)
    }
}

impl<'d> Deserialize<'d> for ImmutableString {
    fn deserialize<D: Deserializer<'d>>(de: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        Ok(s.into())
    }
}
