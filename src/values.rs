use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
};

#[derive(Debug)]
/// Wrapper for value
/// implements simple operations and check types
///
/// #### Panic
/// Panics if operate different value types
pub enum Value {
    Bool(bool),
    Float(f64),
    Int(i64),
}

impl Value {
    /// Is same type
    pub fn is_same(&self, other: &Value) -> bool {
        use self::Value::*;
        match (self, other) {
            (Float(_), Float(_)) | (Int(_), Int(_)) | (Bool(_), Bool(_)) => true,
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
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use self::Value::*;
        match (self, other) {
            (Float(a), Float(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        use self::Value::*;
        match (self, other) {
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Int(a), Int(b)) => a.partial_cmp(b),
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
            _ => panic!("Not valid operation"),
        }
    }
}
