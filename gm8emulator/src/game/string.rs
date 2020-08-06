use encoding_rs::Encoding;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Cow, fmt, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RCStr(Rc<[u8]>);

impl RCStr {
    pub fn decode(&self, encoding: &'static Encoding) -> Cow<str> {
        encoding.decode_without_bom_handling(&self.0).0
    }

    pub fn decode_utf8(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.0)
    }

    pub fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.0.len() == other.len()
            && self.0.iter().copied().zip(other.iter().copied()).all(|(x, y)| {
                x == y || {
                    let x = if x >= b'A' && x <= b'Z' { x + b'a' - b'A' } else { x };
                    let y = if y >= b'A' && y <= b'Z' { y + b'a' - b'A' } else { y };
                    x == y
                }
            })
    }
}

impl AsRef<[u8]> for RCStr {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Display for RCStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        String::from_utf8_lossy(&self.0).fmt(f)
    }
}

impl From<String> for RCStr {
    fn from(value: String) -> Self {
        Self(value.into_bytes().into())
    }
}

impl From<&str> for RCStr {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec().into())
    }
}

impl From<Vec<u8>> for RCStr {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for RCStr {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec().into())
    }
}

struct SerdeVisitor;

impl<'de> Visitor<'de> for SerdeVisitor {
    type Value = RCStr;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value.into())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value.into())
    }
}

impl Serialize for RCStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for RCStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(SerdeVisitor)
    }
}
