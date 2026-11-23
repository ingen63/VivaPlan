//! #221: Finds NaN and ∞ in a result before it becomes JSON. serde_json writes them as `null`, so after the conversion
//! an invalid number can no longer be told apart from a missing value.

use std::fmt;

use serde::ser::{self, Serialize};

/// Place of a value in the result, as a dotted path (`summary.endTotal`, `records.3.cash`).
#[derive(Clone, Copy)]
enum At<'a> {
    Root,
    Field(&'a At<'a>, &'a str),
    Index(&'a At<'a>, usize),
}

impl fmt::Display for At<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (parent, own) = match self {
            At::Root => return Ok(()),
            At::Field(p, k) => (p, k.to_string()),
            At::Index(p, i) => (p, i.to_string()),
        };
        if matches!(parent, At::Root) {
            write!(f, "{own}")
        } else {
            write!(f, "{parent}.{own}")
        }
    }
}

/// Path of the first number that is NaN or ∞; `None` if all numbers are finite.
pub fn first_non_finite<T: Serialize + ?Sized>(value: &T) -> Option<String> {
    match value.serialize(Check { at: &At::Root }) {
        Err(Stop(path)) => path,
        Ok(()) => None,
    }
}

/// End of the search: `Some(path)` for an invalid number, `None` for another error of the serialization.
#[derive(Debug)]
struct Stop(Option<String>);

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for Stop {}

impl ser::Error for Stop {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Stop(None)
    }
}

struct Check<'a> {
    at: &'a At<'a>,
}

impl Check<'_> {
    fn number(self, x: f64) -> Result<(), Stop> {
        if x.is_finite() {
            Ok(())
        } else {
            Err(Stop(Some(self.at.to_string())))
        }
    }
}

/// Lists, tuples, maps and structs: `next` counts the elements, `key` is the key of the next map value.
struct Compound<'a> {
    at: &'a At<'a>,
    next: usize,
    key: String,
}

macro_rules! ignore {
    ($($f:ident: $t:ty),*) => {
        $(fn $f(self, _: $t) -> Result<(), Stop> {
            Ok(())
        })*
    };
}

impl<'a> ser::Serializer for Check<'a> {
    type Ok = ();
    type Error = Stop;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    ignore!(serialize_bool: bool, serialize_i8: i8, serialize_i16: i16, serialize_i32: i32, serialize_i64: i64,
        serialize_i128: i128, serialize_u8: u8, serialize_u16: u16, serialize_u32: u32, serialize_u64: u64,
        serialize_u128: u128, serialize_char: char, serialize_str: &str, serialize_bytes: &[u8],
        serialize_unit_struct: &'static str);

    fn serialize_f32(self, v: f32) -> Result<(), Stop> {
        self.number(f64::from(v))
    }
    fn serialize_f64(self, v: f64) -> Result<(), Stop> {
        self.number(v)
    }
    fn serialize_none(self) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_unit(self) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Stop> {
        v.serialize(self)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, v: &T) -> Result<(), Stop> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<(), Stop> {
        v.serialize(Check {
            at: &At::Field(self.at, variant),
        })
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_tuple(self, _: usize) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, Stop> {
        Ok(self.compound())
    }
}

impl<'a> Check<'a> {
    fn compound(self) -> Compound<'a> {
        Compound {
            at: self.at,
            next: 0,
            key: String::new(),
        }
    }
}

impl Compound<'_> {
    fn element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Stop> {
        let at = At::Index(self.at, self.next);
        self.next += 1;
        v.serialize(Check { at: &at })
    }
    fn field<T: Serialize + ?Sized>(&mut self, key: &str, v: &T) -> Result<(), Stop> {
        v.serialize(Check {
            at: &At::Field(self.at, key),
        })
    }
}

macro_rules! sequence {
    ($($trait:ident :: $f:ident),*) => {
        $(impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = Stop;
            fn $f<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Stop> {
                self.element(v)
            }
            fn end(self) -> Result<(), Stop> {
                Ok(())
            }
        })*
    };
}

sequence!(
    SerializeSeq::serialize_element,
    SerializeTuple::serialize_element,
    SerializeTupleStruct::serialize_field,
    SerializeTupleVariant::serialize_field
);

macro_rules! structure {
    ($($trait:ident),*) => {
        $(impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = Stop;
            fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, v: &T) -> Result<(), Stop> {
                self.field(key, v)
            }
            fn end(self) -> Result<(), Stop> {
                Ok(())
            }
        })*
    };
}

structure!(SerializeStruct, SerializeStructVariant);

impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Stop;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Stop> {
        self.key = match serde_json::to_value(key) {
            Ok(serde_json::Value::String(s)) => s,
            Ok(other) => other.to_string(),
            Err(_) => "?".into(),
        };
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Stop> {
        let key = std::mem::take(&mut self.key);
        self.field(&key, v)
    }
    fn end(self) -> Result<(), Stop> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Row {
        end_total: f64,
        share: f32,
        tag: Option<f64>,
    }

    #[derive(Serialize)]
    enum Shape {
        Point(f64),
        Pair(f64, f64),
        Named { x: f64 },
    }

    /// #221/AK5: NaN and ∞ are found with their place in every kind of container; finite numbers, `None` and other
    /// values pass.
    #[test]
    fn issue_221_ak5_finds_nan_and_infinity_with_their_path() {
        let row = |end_total, share, tag| Row { end_total, share, tag };
        assert_eq!(first_non_finite(&row(1.0, 0.5, None)), None);
        assert_eq!(first_non_finite(&row(f64::NAN, 0.5, None)).as_deref(), Some("endTotal"));
        assert_eq!(
            first_non_finite(&row(1.0, f32::INFINITY, None)).as_deref(),
            Some("share")
        );
        assert_eq!(
            first_non_finite(&row(1.0, 0.5, Some(f64::NEG_INFINITY))).as_deref(),
            Some("tag")
        );
        let rows = vec![row(1.0, 0.5, Some(2.0)), row(1.0, 0.5, Some(f64::NAN))];
        assert_eq!(first_non_finite(&rows).as_deref(), Some("1.tag"));
        let map = BTreeMap::from([("a", vec![1.0]), ("b", vec![2.0, f64::NAN])]);
        assert_eq!(first_non_finite(&map).as_deref(), Some("b.1"));
        assert_eq!(first_non_finite(&(1, "x", f64::NAN)).as_deref(), Some("2"));
        assert_eq!(first_non_finite(&Shape::Point(f64::NAN)).as_deref(), Some("Point"));
        assert_eq!(first_non_finite(&Shape::Pair(1.0, f64::NAN)).as_deref(), Some("1"));
        assert_eq!(first_non_finite(&Shape::Named { x: f64::NAN }).as_deref(), Some("x"));
        assert_eq!(
            first_non_finite(&serde_json::json!({ "a": [1, 2.5, "x", null, true] })),
            None
        );
        assert_eq!(first_non_finite(&Shape::Named { x: 1.0 }), None);
    }
}
