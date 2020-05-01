use std::{convert::TryInto, str::FromStr};

use crate::{reflect::Eval, Value};

macro_rules! pop {
    ($stack:ident) => {
        $stack.pop().ok_or(()).and_then(TryInto::try_into)?
    };
}

macro_rules! fun_arg {
    ($m:ident, $t1:ty, $t2:ty, $stack:ident) => {{
        let op2: $t1 = pop!($stack);
        let op1: $t2 = pop!($stack);
        op1.$m(op2).into()
    }};
}

macro_rules! fun_arg_s {
    ($m:ident, $t:ty, $stack:ident) => {
        fun_arg!($m, $t, $t, $stack)
    };
}

macro_rules! fun {
    ($m:ident, $t:ty, $stack:ident) => {{
        let op1: $t = pop!($stack);
        op1.$m().into()
    }};
}

macro_rules! fun_un {
    ($m:ident, $cb: ident, $stack:ident) => {{
        let op1 = $cb!($stack.pop().ok_or(())?);
        op1.$m().into()
    }};
}

macro_rules! fun_arg_un {
    ($m:ident, $cb: ident, $stack:ident) => {{
        let op2 = $cb!($stack.pop().ok_or(())?);
        let op1 = $cb!($stack.pop().ok_or(())?);
        op1.$m(op2).into()
    }};
}

pub mod dyn_type;
pub mod f64_t;
pub mod option_t;
pub mod slice_t;
pub mod str_t;
pub mod vec_t;

pub(crate) trait HasArg: Copy {
    fn has_arg(self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Method {
    DynType(dyn_type::Fun),
    F64(f64_t::Fun),
    Option(option_t::Fun),
    Slice(slice_t::Fun),
    Str(str_t::Fun),
    VecT(vec_t::Fun),
}

use Method::*;

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! parse {
            ($p:path) => {
                match s.parse().map($p) {
                    Ok(m) => m,
                    Err(_) => return Err(()),
                }
            };
            ($p:path, $($t:tt)+) => {
                match s.parse().map($p) {
                    Ok(m) => m,
                    Err(_) => parse!($($t)+)
                }
            };
        }
        Ok(parse!(DynType, Option, VecT, Slice, Str, F64))
    }
}

impl Eval for Method {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        match self {
            DynType(f) => f.eval(stack),
            F64(f) => f.eval(stack),
            Option(f) => f.eval(stack),
            Slice(f) => f.eval(stack),
            Str(f) => f.eval(stack),
            VecT(f) => f.eval(stack),
        }
    }
}

impl HasArg for Method {
    fn has_arg(self) -> bool {
        match self {
            DynType(f) => f.has_arg(),
            F64(f) => f.has_arg(),
            Option(f) => f.has_arg(),
            Slice(f) => f.has_arg(),
            Str(f) => f.has_arg(),
            VecT(f) => f.has_arg(),
        }
    }
}
