use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[cfg(feature = "value-serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "value-serde")]
use serde::de::{MapAccess, SeqAccess};

#[cfg(feature = "value-serde")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match self {
            Value::Nil => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => serializer.serialize_f64(*n),
            Value::String(s) => serializer.serialize_str(&s),
            Value::Array(a) => a.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
        }
    }
}

#[cfg(feature = "value-serde")]
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        use serde::de::Error;
        use serde::de::Visitor;

        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Number(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Number(value as f64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Number(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::String(value))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Nil)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>
            {
                let mut vec = Vec::new();
                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }
                Ok(Value::Array(vec))
            }

            fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
                where A: MapAccess<'de>
            {
                let mut map = HashMap::new();
                while let Some(value) = map_access.next_entry()? {
                    map.insert(value.0, value.1);
                }
                Ok(Value::Object(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
