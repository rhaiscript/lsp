//! Implementations of [`serde::Serialize`].

use crate::dynamic::Union;
use crate::{Dynamic, ImmutableString};
use serde::ser::{Serialize, Serializer};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

#[cfg(not(feature = "no_object"))]
use serde::ser::SerializeMap;

#[cfg(not(feature = "no_std"))]
use crate::dynamic::Variant;

impl Serialize for Dynamic {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Union::Unit(_, _, _) => ser.serialize_unit(),
            Union::Bool(x, _, _) => ser.serialize_bool(x),
            Union::Str(ref s, _, _) => ser.serialize_str(s.as_str()),
            Union::Char(c, _, _) => ser.serialize_str(&c.to_string()),

            #[cfg(not(feature = "only_i32"))]
            Union::Int(x, _, _) => ser.serialize_i64(x),
            #[cfg(feature = "only_i32")]
            Union::Int(x, _, _) => ser.serialize_i32(x),

            #[cfg(not(feature = "no_float"))]
            #[cfg(not(feature = "f32_float"))]
            Union::Float(x, _, _) => ser.serialize_f64(*x),
            #[cfg(not(feature = "no_float"))]
            #[cfg(feature = "f32_float")]
            Union::Float(x, _, _) => ser.serialize_f32(*x),

            #[cfg(feature = "decimal")]
            #[cfg(not(feature = "f32_float"))]
            Union::Decimal(ref x, _, _) => {
                use rust_decimal::prelude::ToPrimitive;

                if let Some(v) = x.to_f64() {
                    ser.serialize_f64(v)
                } else {
                    ser.serialize_str(&x.to_string())
                }
            }
            #[cfg(feature = "decimal")]
            #[cfg(feature = "f32_float")]
            Union::Decimal(ref x, _, _) => {
                use rust_decimal::prelude::ToPrimitive;

                if let Some(v) = x.to_f32() {
                    ser.serialize_f32(v)
                } else {
                    ser.serialize_str(&x.to_string())
                }
            }

            #[cfg(not(feature = "no_index"))]
            Union::Array(ref a, _, _) => (**a).serialize(ser),
            #[cfg(not(feature = "no_object"))]
            Union::Map(ref m, _, _) => {
                let mut map = ser.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter() {
                    map.serialize_entry(k.as_str(), v)?;
                }
                map.end()
            }
            Union::FnPtr(ref f, _, _) => ser.serialize_str(f.fn_name()),
            #[cfg(not(feature = "no_std"))]
            Union::TimeStamp(ref x, _, _) => ser.serialize_str(x.as_ref().type_name()),

            Union::Variant(ref v, _, _) => ser.serialize_str((***v).type_name()),

            #[cfg(not(feature = "no_closure"))]
            #[cfg(not(feature = "sync"))]
            Union::Shared(ref cell, _, _) => cell.borrow().serialize(ser),
            #[cfg(not(feature = "no_closure"))]
            #[cfg(feature = "sync")]
            Union::Shared(ref cell, _, _) => cell.read().unwrap().serialize(ser),
        }
    }
}

impl Serialize for ImmutableString {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}
