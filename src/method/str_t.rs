use std::{convert::TryInto, str::FromStr};

use regex::Regex;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    IsAscii,
    ToLowercase,
    ToUppercase,
    ToAsciiLowercase,
    ToAsciiUppercase,
    Trim,
    TrimEnd,
    TrimStart,
    EqIgnoreAsciiCase = 1 << F,
    Find = (1 << F) + 1,
    IsMatch = (1 << F) + 2,
    RFind = (1 << F) + 3,
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
            "eq_ignore_ascii_case" => Ok(EqIgnoreAsciiCase),
            "find" => Ok(Find),
            "is_ascii" => Ok(IsAscii),
            "is_match" => Ok(IsMatch),
            "to_lowercase" => Ok(ToLowercase),
            "to_uppercase" => Ok(ToUppercase),
            "to_ascii_lowercase" => Ok(ToAsciiLowercase),
            "to_ascii_uppercase" => Ok(ToAsciiUppercase),
            "trim" => Ok(Trim),
            "trim_end" => Ok(TrimEnd),
            "trim_start" => Ok(TrimStart),
            "rfind" => Ok(RFind),
            _ => Err(()),
        }
    }
}

impl Eval for Fun {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! fun_ref {
            ($fun:ident) => {{
                let op2: String = pop!(stack);
                let op1: String = pop!(stack);
                op1.$fun(&op2).into()
            }};
        }

        let e = match self {
            EqIgnoreAsciiCase => fun_ref!(eq_ignore_ascii_case),
            Find => fun_ref!(find),
            IsAscii => fun!(is_ascii, String, stack),
            IsMatch => {
                let op2: String = pop!(stack);
                let op1: String = pop!(stack);
                let re = Regex::new(&op2).map_err(|_| ())?;
                re.is_match(&op1).into()
            }
            ToLowercase => fun!(to_lowercase, String, stack),
            ToUppercase => fun!(to_uppercase, String, stack),
            ToAsciiLowercase => fun!(to_ascii_lowercase, String, stack),
            ToAsciiUppercase => fun!(to_ascii_uppercase, String, stack),
            Trim => fun!(trim, String, stack),
            TrimEnd => fun!(trim_end, String, stack),
            TrimStart => fun!(trim_start, String, stack),
            RFind => fun_ref!(rfind),
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
