use std::str::FromStr;

use crate::{reflect::Eval, Value};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Fun {
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atanh,
    Cbrt,
    Ceil,
    Cos,
    Cosh,
    Exp,
    Exp2,
    ExpM1,
    Floor,
    Fract,
    Ln,
    Ln1p,
    Log10,
    Log2,
    Recip,
    Round,
    Signum,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
    ToDegrees,
    ToRadians,
    Trunc,
    PowI = 1 << F,
    PowF = (1 << F) + 1,
    Atan2 = (1 << F) + 2,
    Hypot = (1 << F) + 3,
    Log = (1 << F) + 4,
    Max = (1 << F) + 5,
    Min = (1 << F) + 6,
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
            "abs" => Ok(Abs),
            "acos" => Ok(Acos),
            "acosh" => Ok(Acosh),
            "asin" => Ok(Asin),
            "asinh" => Ok(Asinh),
            "atan" => Ok(Atan),
            "atan2" => Ok(Atan2),
            "atanh" => Ok(Atanh),
            "cbrt" => Ok(Cbrt),
            "ceil" => Ok(Ceil),
            "cos" => Ok(Cos),
            "cosh" => Ok(Cosh),
            "exp" => Ok(Exp),
            "exp2" => Ok(Exp2),
            "exp_m1" => Ok(ExpM1),
            "floor" => Ok(Floor),
            "fract" => Ok(Fract),
            "hypot" => Ok(Hypot),
            "ln" => Ok(Ln),
            "ln_1p" => Ok(Ln1p),
            "log" => Ok(Log),
            "log10" => Ok(Log10),
            "log2" => Ok(Log2),
            "max" => Ok(Max),
            "min" => Ok(Min),
            "powf" => Ok(PowF),
            "powi" => Ok(PowI),
            "recip" => Ok(Recip),
            "round" => Ok(Round),
            "signum" => Ok(Signum),
            "sin" => Ok(Sin),
            "sinh" => Ok(Sinh),
            "sqrt" => Ok(Sqrt),
            "tan" => Ok(Tan),
            "tanh" => Ok(Tanh),
            "to_degrees" => Ok(ToDegrees),
            "to_radians" => Ok(ToRadians),
            "trunc" => Ok(Trunc),
            _ => Err(()),
        }
    }
}

macro_rules! unpack {
    ($e:expr) => {
        match $e {
            Value::Float(x) => x,
            Value::Int(x) => x as f64,
            _ => return Err(()),
        }
    };
}

impl Eval for Fun {
    #[allow(clippy::cognitive_complexity)]
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! to_int {
            ($fun:ident) => {{
                let v = fun!($fun, unpack, stack) as i64;
                stack.push(Value::Int(v));
                return Ok(());
            }};
        }
        let e = match self {
            Atan2 => fun_arg!(atan2, unpack, stack),
            Hypot => fun_arg!(hypot, unpack, stack),
            Log => fun_arg!(log, unpack, stack),
            Max => fun_arg!(max, unpack, stack),
            Min => fun_arg!(min, unpack, stack),
            PowF => fun_arg!(powf, unpack, stack),
            PowI => {
                let op2 = unpack!(stack.pop().ok_or(())?);
                let op1 = unpack!(stack.pop().ok_or(())?);
                op1.powi(op2 as i32)
            }
            Abs => fun!(abs, unpack, stack),
            Acos => fun!(acos, unpack, stack),
            Acosh => fun!(acosh, unpack, stack),
            Asin => fun!(asin, unpack, stack),
            Asinh => fun!(asinh, unpack, stack),
            Atan => fun!(atan, unpack, stack),
            Atanh => fun!(atanh, unpack, stack),
            Cbrt => fun!(cbrt, unpack, stack),
            Cos => fun!(cos, unpack, stack),
            Cosh => fun!(cosh, unpack, stack),
            Exp => fun!(exp, unpack, stack),
            Exp2 => fun!(exp2, unpack, stack),
            ExpM1 => fun!(exp_m1, unpack, stack),
            Fract => fun!(fract, unpack, stack),
            Ln => fun!(ln, unpack, stack),
            Ln1p => fun!(ln_1p, unpack, stack),
            Log10 => fun!(log10, unpack, stack),
            Log2 => fun!(log2, unpack, stack),
            Recip => fun!(recip, unpack, stack),
            Signum => fun!(signum, unpack, stack),
            Sin => fun!(sin, unpack, stack),
            Sinh => fun!(sinh, unpack, stack),
            Sqrt => fun!(sqrt, unpack, stack),
            Tan => fun!(tan, unpack, stack),
            Tanh => fun!(tanh, unpack, stack),
            ToDegrees => fun!(to_degrees, unpack, stack),
            ToRadians => fun!(to_radians, unpack, stack),
            Ceil => to_int!(ceil),
            Floor => to_int!(floor),
            Round => to_int!(round),
            Trunc => to_int!(trunc),
        };
        stack.push(Value::Float(e));
        Ok(())
    }
}

impl HasArg for Fun {
    #[inline]
    fn has_arg(self) -> bool {
        (self as u8).leading_zeros() as u8 == L
    }
}
