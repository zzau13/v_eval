use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    Len,
    IsEmpty,
    Contains = 1 << F,
    StartsWith = (1 << F) + 1,
    EndsWith = (1 << F) + 2,
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
            "len" => Ok(Len),
            "is_empty" => Ok(IsEmpty),
            "contains" => Ok(Contains),
            "starts_with" => Ok(StartsWith),
            "ends_with" => Ok(EndsWith),
            _ => Err(()),
        }
    }
}

macro_rules! unpack_str {
    ($e:expr) => {
        match $e {
            Value::Str(x) => x,
            _ => return Err(()),
        }
    };
}

macro_rules! unpack_vec {
    ($e:expr) => {
        match $e {
            Value::Vec(x) => x,
            _ => return Err(()),
        }
    };
}

macro_rules! unpack_value {
    ($e:expr) => {
        match $e {
            Value::Vec(_) => return Err(()),
            a => a,
        }
    };
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! bool_arg {
            ($fun:ident) => {{
                let op2 = stack.pop().ok_or(())?;
                let op1 = stack.pop().ok_or(())?;
                stack.push(Value::Bool(match op1 {
                    Value::Vec(x) => {
                        let e = unpack_vec!(op2);
                        x.$fun(&e)
                    }
                    Value::Str(x) => x.$fun(&unpack_str!(op2)),
                    _ => return Err(()),
                }))
            }};
        }

        use Fun::*;
        match self {
            Len => {
                let op1 = stack.pop().ok_or(())?;
                stack.push(Value::Int(match op1 {
                    Value::Vec(x) => x.len(),
                    Value::Str(x) => x.len(),
                    _ => return Err(()),
                } as i64))
            }
            IsEmpty => {
                let op1 = stack.pop().ok_or(())?;
                stack.push(Value::Bool(match op1 {
                    Value::Vec(x) => x.is_empty(),
                    Value::Str(x) => x.is_empty(),
                    _ => return Err(()),
                }))
            }
            Contains => {
                let op2 = stack.pop().ok_or(())?;
                let op1 = stack.pop().ok_or(())?;
                stack.push(Value::Bool(match op1 {
                    Value::Vec(x) => x.contains(&unpack_value!(op2)),
                    Value::Str(x) => x.contains(&unpack_str!(op2)),
                    _ => return Err(()),
                }))
            }
            StartsWith => bool_arg!(starts_with),
            EndsWith => bool_arg!(ends_with),
        };

        Ok(())
    }
}

impl HasArg for Fun {
    #[inline]
    fn has_arg(self) -> bool {
        (self as u8).leading_zeros() as u8 == L
    }
}
