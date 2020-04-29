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

use Fun::*;

/// Has arguments flags
const F: u8 = 6;
/// Has arguments number of leading zeros
const L: u8 = 1;

impl FromStr for Fun {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! fun_arg {
            ($fun:ident) => {{
                let op2 = stack.pop().ok_or(())?;
                let op1 = stack.pop().ok_or(())?;
                stack.push(
                    match op1 {
                        Value::Vec(x) => x.$fun(&TryInto::<Vec<Value>>::try_into(op2)?),
                        Value::Str(x) => x.$fun(&TryInto::<String>::try_into(op2)?),
                        _ => return Err(()),
                    }
                    .into(),
                )
            }};
        }

        macro_rules! fun {
            ($fun:ident) => {{
                let op1 = stack.pop().ok_or(())?;
                stack.push(
                    match op1 {
                        Value::Vec(x) => x.$fun(),
                        Value::Str(x) => x.$fun(),
                        _ => return Err(()),
                    }
                    .into(),
                )
            }};
        }

        match self {
            Len => fun!(len),
            IsEmpty => fun!(is_empty),
            Contains => {
                let op2 = stack.pop().ok_or(())?;
                let op1 = stack.pop().ok_or(())?;
                stack.push(
                    match op1 {
                        Value::Vec(op1) => op1.contains(&op2),
                        Value::Str(op1) => op1.contains(&TryInto::<String>::try_into(op2)?),
                        _ => return Err(()),
                    }
                    .into(),
                )
            }
            StartsWith => fun_arg!(starts_with),
            EndsWith => fun_arg!(ends_with),
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
