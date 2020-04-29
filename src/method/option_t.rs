use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    IsNone,
    IsSome,
    And = (1 << F) + 1,
    Or = (1 << F) + 2,
    Xor = (1 << F) + 3,
}

use Fun::*;

/// Has arguments flags
const F: u8 = 6;
/// Has arguments number of leading zeros
const L: u8 = 1;

impl FromStr for Fun {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Fun::*;
        match s {
            "is_some" => Ok(IsSome),
            "is_none" => Ok(IsNone),
            "and" => Ok(And),
            "or" => Ok(Or),
            "xor" => Ok(Xor),
            _ => Err(()),
        }
    }
}

// Into<Option> for Value is implemented
macro_rules! unpack {
    ($e:expr) => {
        match $e {
            Value::None => None,
            v => Some(v),
        }
    };
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! bool {
            ($fun:ident) => {{
                let e: bool = fun_un!($fun, unpack, stack);
                e.into()
            }};
        }
        let e = match self {
            And => fun_arg_un!(and, unpack, stack),
            IsNone => bool!(is_none),
            IsSome => bool!(is_some),
            Or => fun_arg_un!(or, unpack, stack),
            Xor => fun_arg_un!(xor, unpack, stack),
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
