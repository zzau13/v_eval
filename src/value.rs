use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    ops::{Add, Div, Mul, Range, Rem, Sub},
};

#[derive(Clone, Debug)]
/// Wrapper for value
/// implements simple operations and check types
///
/// #### Panic
/// Panics if operate different value types
pub enum Value {
    Bool(bool),
    Float(f64),
    Int(i64),
    Str(String),
    Range(Range<i64>),
    Vec(Vec<Value>),
    None,
}

impl Value {
    /// Is same type
    pub fn is_same(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Float(_), Float(_))
            | (Int(_), Int(_))
            | (Float(_), Int(_))
            | (Int(_), Float(_))
            | (Bool(_), Bool(_))
            | (Str(_), Str(_))
            | (Range(_), Range(_))
            | (Vec(_), Vec(_))
            | (None, None) => true,
            _ => false,
        }
    }

    pub fn not(&self) -> Value {
        use self::Value::*;
        match self {
            Bool(a) => Bool(!*a),
            _ => panic!("Not valid operation"),
        }
    }

    pub fn neg(&self) -> Value {
        use self::Value::*;
        match self {
            Int(a) => Int(-*a),
            Float(a) => Float(-*a),
            _ => panic!("Not valid operation"),
        }
    }

    pub fn and(&self, other: &Value) -> Value {
        use self::Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Bool(*a && *b),
            _ => panic!("Not valid operation"),
        }
    }

    pub fn or(&self, other: &Value) -> Value {
        use self::Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Bool(*a || *b),
            _ => panic!("Not valid operation"),
        }
    }

    pub fn unwrap(self) -> Result<Self, ()> {
        if self.is_some() {
            Ok(self)
        } else {
            Err(())
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Value::None => false,
            Value::Vec(a) => a.iter().all(|v| v.is_some()),
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Value::*;
        match self {
            Int(a) => a.fmt(f),
            Float(a) => a.fmt(f),
            Bool(a) => a.fmt(f),
            Str(a) => fmt::Debug::fmt(a, f),
            Range(a) => fmt::Debug::fmt(a, f),
            Vec(a) => {
                f.write_str("[")?;
                for i in a {
                    i.fmt(f)?;
                    f.write_str(",")?;
                }
                f.write_str("]")
            }
            None => f.write_str("None"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use self::Value::*;
        match (self, other) {
            (Float(a), Float(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Float(a), Int(b)) => *a == (*b as f64),
            (Int(a), Float(b)) => (*a as f64) == *b,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Vec(a), Vec(b)) => a == b,
            (Range(a), Range(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, v: Value) -> Value {
        use self::Value::*;
        match (self, v) {
            (Float(a), Float(b)) => Float(a + b),
            (Int(a), Int(b)) => Int(a + b),
            (Float(a), Int(b)) => Float(a + (b as f64)),
            (Int(a), Float(b)) => Float((a as f64) + b),
            (Str(a), Str(b)) => Str(a + &b),
            _ => panic!("Not valid operation"),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, v: Value) -> Value {
        use self::Value::*;
        match (self, v) {
            (Float(a), Float(b)) => Float(a - b),
            (Int(a), Int(b)) => Int(a - b),
            (Float(a), Int(b)) => Float(a - (b as f64)),
            (Int(a), Float(b)) => Float((a as f64) - b),
            _ => panic!("Not valid operation"),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, v: Value) -> Value {
        use self::Value::*;
        match (self, v) {
            (Float(a), Float(b)) => Float(a * b),
            (Int(a), Int(b)) => Int(a * b),
            (Float(a), Int(b)) => Float(a * (b as f64)),
            (Int(a), Float(b)) => Float((a as f64) * b),
            (Int(a), Str(b)) | (Str(b), Int(a)) => {
                if 0 < a {
                    Str(b.repeat(a as usize))
                } else {
                    Str("".into())
                }
            }
            _ => panic!("Not valid operation"),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, v: Value) -> Value {
        use self::Value::*;
        match (self, v) {
            (Float(a), Float(b)) => Float(a / b),
            (Int(a), Int(b)) => Int(a / b),
            (Float(a), Int(b)) => Float(a / (b as f64)),
            (Int(a), Float(b)) => Float((a as f64) / b),
            _ => panic!("Not valid operation"),
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, v: Value) -> Value {
        use self::Value::*;
        match (self, v) {
            (Float(a), Float(b)) => Float(a % b),
            (Int(a), Int(b)) => Int(a % b),
            (Float(a), Int(b)) => Float(a % (b as f64)),
            (Int(a), Float(b)) => Float((a as f64) % b),
            _ => panic!("Not valid operation"),
        }
    }
}
