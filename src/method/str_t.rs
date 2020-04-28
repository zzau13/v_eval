use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    IsMatch = 1 << F,
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
            "is_match" => Ok(IsMatch),
            _ => Err(()),
        }
    }
}

macro_rules! unpack {
    ($e:expr) => {
        match $e {
            Value::Str(x) => x,
            _ => return Err(()),
        }
    };
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        use Fun::*;
        match self {
            IsMatch => {
                let op2 = unpack!(stack.pop().ok_or(())?);
                let op1 = unpack!(stack.pop().ok_or(())?);
                let re = Regex::new(&op2).map_err(|_| ())?;
                stack.push(Value::Bool(re.is_match(&op1)))
            }
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
