use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    First,
    Last,
    Get = 1 << F,
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
            "first" => Ok(First),
            "get" => Ok(Get),
            "last" => Ok(Last),
            _ => Err(()),
        }
    }
}

impl Eval for Fun {
    #[inline]
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        use Fun::*;
        let e = match self {
            First => fun!(first, Vec<Value>, stack),
            Get => fun_arg!(get, usize, Vec<Value>, stack),
            Last => fun!(last, Vec<Value>, stack),
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
