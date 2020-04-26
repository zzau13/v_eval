use crate::{reflect::Eval, Value};

macro_rules! fun_arg {
    ($m:ident, $cb: ident, $stack:ident) => {{
        let op2 = $cb!($stack.pop().ok_or(())?);
        let op1 = $cb!($stack.pop().ok_or(())?);
        op1.$m(op2)
    }};
}

macro_rules! fun {
    ($m:ident, $cb: ident, $stack:ident) => {{
        let op1 = $cb!($stack.pop().ok_or(())?);
        op1.$m()
    }};
}

pub mod dyn_type;
pub mod f64_t;
pub mod option_t;

pub(crate) trait HasArg: Copy {
    fn has_arg(self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Method {
    DynType(dyn_type::Fun),
    F64(f64_t::Fun),
    Option(option_t::Fun),
}

use Method::*;

impl Eval for Method {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        match self {
            DynType(f) => f.eval(stack),
            F64(f) => f.eval(stack),
            Option(f) => f.eval(stack),
        }
    }
}

impl HasArg for Method {
    fn has_arg(self) -> bool {
        match self {
            DynType(f) => f.has_arg(),
            F64(f) => f.has_arg(),
            Option(f) => f.has_arg(),
        }
    }
}
