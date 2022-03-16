use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn is_bool(&self) -> bool {
        if let Value::Bool(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int(&self) -> bool {
        if let Value::Int(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_float(&self) -> bool {
        if let Value::Float(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_string(&self) -> bool {
        if let Value::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_array(&self) -> bool {
        if let Value::Array(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_object(&self) -> bool {
        if let Value::Object(_) = self {
            true
        } else {
            false
        }
    }
}

impl TryInto<bool> for Value {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Value::Bool(b) = self {
            Ok(b)
        } else {
            Err(format!("{:?} is not a bool", self))
        }
    }
}

impl TryInto<i64> for Value {
    type Error = String;

    fn try_into(self) -> Result<i64, Self::Error> {
        if let Value::Int(i) = self {
            Ok(i)
        } else {
            Err(format!("{:?} is not an int", self))
        }
    }
}

impl TryInto<f64> for Value {
    type Error = String;

    fn try_into(self) -> Result<f64, Self::Error> {
        if let Value::Float(f) = self {
            Ok(f)
        } else {
            Err(format!("{:?} is not a float", self))
        }
    }
}

impl TryInto<String> for Value {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(format!("{:?} is not a string", self))
        }
    }
}

impl TryInto<Vec<Value>> for Value {
    type Error = String;

    fn try_into(self) -> Result<Vec<Value>, Self::Error> {
        if let Value::Array(a) = self {
            Ok(a)
        } else {
            Err(format!("{:?} is not an array", self))
        }
    }
}

impl TryInto<HashMap<String, Value>> for Value {
    type Error = String;

    fn try_into(self) -> Result<HashMap<String, Value>, Self::Error> {
        if let Value::Object(o) = self {
            Ok(o)
        } else {
            Err(format!("{:?} is not an object", self))
        }
    }
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
            Value::Int(i) => serializer.serialize_i64(*i),
            Value::Float(n) => serializer.serialize_f64(*n),
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
                Ok(Value::Int(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Int(value as i64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Value::Float(value))
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
