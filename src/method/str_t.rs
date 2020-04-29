use std::{convert::TryInto, str::FromStr};

use crate::{reflect::Eval, Value};

use super::*;
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    IsMatch = 1 << F,
    Find = (1 << F) + 1,
    RFind = (1 << F) + 2,
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
            "find" => Ok(Find),
            "rfind" => Ok(RFind),
            _ => Err(()),
        }
    }
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! some_arg {
            ($fun:ident) => {{
                let op2: String = pop!(stack);
                let op1: String = stack.pop().ok_or(()).and_then(TryInto::try_into)?;
                op1.$fun(&op2).map_or(Value::None, Into::into)
            }};
        }
        use Fun::*;
        let e = match self {
            IsMatch => {
                let op2: String = stack.pop().ok_or(()).and_then(TryInto::try_into)?;
                let op1: String = stack.pop().ok_or(()).and_then(TryInto::try_into)?;
                let re = Regex::new(&op2).map_err(|_| ())?;
                re.is_match(&op1).into()
            }
            Find => some_arg!(find),
            RFind => some_arg!(rfind),
        };
        stack.push(e);

        Ok(())
    }
}

impl HasArg for Fun {
    #[inline]
    fn has_arg(self) -> bool {
        (self as u8).leading_zeros() as u8 == L
    }
}
