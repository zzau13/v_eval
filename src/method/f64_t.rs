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

impl Eval for Fun {
    #[allow(clippy::cognitive_complexity)]
    #[inline]
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        macro_rules! to_int {
            ($fun:ident) => {{
                let v: f64 = fun!($fun, f64, stack);

                (v as i64).into()
            }};
        }
        let e = match self {
            Atan2 => fun_arg_s!(atan2, f64, stack),
            Hypot => fun_arg_s!(hypot, f64, stack),
            Log => fun_arg_s!(log, f64, stack),
            Max => fun_arg_s!(max, f64, stack),
            Min => fun_arg_s!(min, f64, stack),
            PowF => fun_arg_s!(powf, f64, stack),
            PowI => {
                let op2: f64 = pop!(stack);
                let op1: f64 = pop!(stack);
                op1.powi(op2 as i32).into()
            }
            Abs => fun!(abs, f64, stack),
            Acos => fun!(acos, f64, stack),
            Acosh => fun!(acosh, f64, stack),
            Asin => fun!(asin, f64, stack),
            Asinh => fun!(asinh, f64, stack),
            Atan => fun!(atan, f64, stack),
            Atanh => fun!(atanh, f64, stack),
            Cbrt => fun!(cbrt, f64, stack),
            Cos => fun!(cos, f64, stack),
            Cosh => fun!(cosh, f64, stack),
            Exp => fun!(exp, f64, stack),
            Exp2 => fun!(exp2, f64, stack),
            ExpM1 => fun!(exp_m1, f64, stack),
            Fract => fun!(fract, f64, stack),
            Ln => fun!(ln, f64, stack),
            Ln1p => fun!(ln_1p, f64, stack),
            Log10 => fun!(log10, f64, stack),
            Log2 => fun!(log2, f64, stack),
            Recip => fun!(recip, f64, stack),
            Signum => fun!(signum, f64, stack),
            Sin => fun!(sin, f64, stack),
            Sinh => fun!(sinh, f64, stack),
            Sqrt => fun!(sqrt, f64, stack),
            Tan => fun!(tan, f64, stack),
            Tanh => fun!(tanh, f64, stack),
            ToDegrees => fun!(to_degrees, f64, stack),
            ToRadians => fun!(to_radians, f64, stack),
            Ceil => to_int!(ceil),
            Floor => to_int!(floor),
            Round => to_int!(round),
            Trunc => to_int!(trunc),
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
