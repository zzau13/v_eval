use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    IsNone,
    IsSome,
    UnWrap,
    UnWrapOr = 1 << F,
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
            "unwrap" => Ok(UnWrap),
            "unwrap_or" => Ok(UnWrapOr),
            "and" => Ok(And),
            "or" => Ok(Or),
            "xor" => Ok(Xor),
            _ => Err(()),
        }
    }
}

macro_rules! unpack {
    ($e:expr) => {
        match $e {
            Value::Option(x) => *x,
            _ => return Err(()),
        }
    };
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! bool {
            ($fun:ident) => {{
                let e = fun!($fun, unpack, stack);
                stack.push(Value::Bool(e));
                return Ok(());
            }};
        }
        let e = match self {
            And => fun_arg!(and, unpack, stack),
            IsNone => bool!(is_none),
            IsSome => bool!(is_some),
            Or => fun_arg!(or, unpack, stack),
            UnWrap => {
                let e = fun!(unwrap, unpack, stack);
                stack.push(e);
                return Ok(());
            }
            UnWrapOr => {
                let op2 = stack.pop().ok_or(())?;
                let op1 = unpack!(stack.pop().ok_or(())?);
                stack.push(op1.unwrap_or(op2));
                return Ok(());
            }
            Xor => fun_arg!(xor, unpack, stack),
        };
        stack.push(Value::Option(Box::new(e)));

        Ok(())
    }
}

impl HasArg for Fun {
    fn has_arg(self) -> bool {
        (self as u8).leading_zeros() as u8 == L
    }
}
