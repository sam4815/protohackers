use core::fmt;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;

use super::models::{LengthPrefixedString, LengthPrefixedVector};

impl<'de> Deserialize<'de> for LengthPrefixedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LengthPrefixedStringVisitor;

        impl<'de> Visitor<'de> for LengthPrefixedStringVisitor {
            type Value = LengthPrefixedString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("length-prefixed string")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let len: u8 = match seq.next_element()? {
                    Some(v) => v,
                    None => return Err(de::Error::invalid_length(0, &self)),
                };

                let mut data = Vec::<u8>::with_capacity(len as usize);

                for _ in 0..len {
                    let byte = match seq.next_element()? {
                        Some(v) => v,
                        None => return Err(de::Error::invalid_length(len as usize, &self)),
                    };
                    data.push(byte);
                }

                match String::from_utf8(data) {
                    Ok(s) => Ok(LengthPrefixedString(s)),
                    Err(_) => Err(de::Error::custom("Invalid UTF-8 sequence")),
                }
            }
        }

        deserializer.deserialize_tuple(u8::MAX.into(), LengthPrefixedStringVisitor)
    }
}

impl<'de> Deserialize<'de> for LengthPrefixedVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LengthPrefixedVectorVisitor;

        impl<'de> Visitor<'de> for LengthPrefixedVectorVisitor {
            type Value = LengthPrefixedVector;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("length-prefixed u16 vector")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let len: u8 = match seq.next_element()? {
                    Some(v) => v,
                    None => return Err(de::Error::invalid_length(0, &self)),
                };

                let mut data = Vec::<u16>::with_capacity(len as usize);

                for _ in 0..len {
                    let value: u16 = match seq.next_element()? {
                        Some(v) => v,
                        None => return Err(de::Error::invalid_length(len as usize, &self)),
                    };
                    data.push(value);
                }

                Ok(LengthPrefixedVector(data))
            }
        }

        deserializer.deserialize_tuple(u8::MAX.into(), LengthPrefixedVectorVisitor)
    }
}
