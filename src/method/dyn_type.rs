use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    Bool,
    Float,
    Int,
    Range,
    Str,
    Vec,
    Same = 1 << F,
}

/// Has arguments flags
const F: u8 = 6;
/// Has arguments number of leading zeros
const L: u8 = 1;

impl FromStr for Fun {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Fun::*;
        match s {
            "is_bool" => Ok(Bool),
            "is_float" => Ok(Float),
            "is_int" => Ok(Int),
            "is_range" => Ok(Range),
            "is_same" => Ok(Same),
            "is_str" => Ok(Str),
            "is_vec" => Ok(Vec),
            _ => Err(()),
        }
    }
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! check {
            ($pat:pat) => {{
                let op1 = stack.pop().ok_or(())?;
                if let $pat = op1 {
                    stack.push(Value::Bool(true));
                } else {
                    stack.push(Value::Bool(false));
                }
            }};
        }

        use Fun::*;
        match self {
            Bool => check!(Value::Bool(_)),
            Float => check!(Value::Float(_)),
            Int => check!(Value::Int(_)),
            Range => check!(Value::Range(_)),
            Same => {
                let op2 = stack.pop().ok_or(())?;
                let op1 = stack.pop().ok_or(())?;
                stack.push(Value::Bool(op1.is_same(&op2)))
            }
            Str => check!(Value::Str(_)),
            Vec => check!(Value::Vec(_)),
        }

        Ok(())
    }
}

impl HasArg for Fun {
    #[inline]
    fn has_arg(self) -> bool {
        (self as u8).leading_zeros() as u8 == L
    }
}
