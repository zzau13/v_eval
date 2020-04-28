use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    Len,
    IsEmpty,
}

impl FromStr for Fun {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Fun::*;
        match s {
            "len" => Ok(Len),
            "is_empty" => Ok(IsEmpty),
            _ => Err(()),
        }
    }
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
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
        };

        Ok(())
    }
}

impl HasArg for Fun {
    #[inline]
    fn has_arg(self) -> bool {
        false
    }
}
