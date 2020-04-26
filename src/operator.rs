use std::{cmp::Ordering, convert::TryFrom};

use crate::{reflect::Eval, Value};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub(super) enum Operator {
    ParenLeft = 1 << 1,
    ParenRight = (1 << 1) + 1,

    Not = 1 << 2,
    Neg = (1 << 2) + 1,

    Mul = 1 << 3,
    Div = (1 << 3) + 1,
    Rem = (1 << 3) + 2,

    Add = 1 << 4,
    Sub = (1 << 4) + 1,

    Eq = 1 << 5,
    Ne = (1 << 5) + 1,
    Gt = (1 << 5) + 2,
    Lt = (1 << 5) + 3,
    Ge = (1 << 5) + 4,
    Le = (1 << 5) + 5,

    And = 1 << 6,
    Or = (1 << 6) + 1,
}

impl Operator {
    fn preference(self, o: Operator) -> Ordering {
        (self as u8).leading_zeros().cmp(&(o as u8).leading_zeros())
    }

    pub(super) fn gt_preference(self, o: Operator) -> bool {
        match self.preference(o) {
            Ordering::Greater => true,
            _ => false,
        }
    }

    pub(super) fn eq_preference(self, o: Operator) -> bool {
        match self.preference(o) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl TryFrom<syn::BinOp> for Operator {
    type Error = ();

    fn try_from(value: syn::BinOp) -> Result<Self, Self::Error> {
        use syn::BinOp::*;
        match value {
            Add(_) => Ok(Operator::Add),
            Sub(_) => Ok(Operator::Sub),
            Mul(_) => Ok(Operator::Mul),
            Div(_) => Ok(Operator::Div),
            Rem(_) => Ok(Operator::Rem),
            And(_) => Ok(Operator::And),
            Or(_) => Ok(Operator::Or),
            Eq(_) => Ok(Operator::Eq),
            Ne(_) => Ok(Operator::Ne),
            Lt(_) => Ok(Operator::Lt),
            Le(_) => Ok(Operator::Le),
            Gt(_) => Ok(Operator::Gt),
            Ge(_) => Ok(Operator::Ge),
            _ => Err(()),
        }
    }
}

impl Eval for Operator {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()> {
        let op2 = stack.pop().ok_or(())?;
        let op1 = stack.pop().ok_or(())?;

        macro_rules! _i {
                    ($a:ident for $e:path) => {
                        $a == $e
                    };
                    ($a:ident for $e:path | $($t:tt)+) => {
                        $a == $e || _i!($a for $($t)+)
                    };
                }

        macro_rules! order {
                    ($($t:tt)+) => {
                        if let Some(a) = op1.partial_cmp(&op2) {
                            Value::Bool(_i!(a for $($t)+))
                        } else {
                            return Err(());
                        }
                    };
                }

        if check_op(self, &op1, &op2) {
            use Operator::*;
            stack.push(match self {
                Add => op1 + op2,
                Sub => op1 - op2,
                Mul => op1 * op2,
                Div => op1 / op2,
                Rem => op1 % op2,
                Eq => Value::Bool(op1 == op2),
                Ne => Value::Bool(op1 != op2),
                Gt => order!(Ordering::Greater),
                Ge => order!(Ordering::Greater | Ordering::Equal),
                Lt => order!(Ordering::Less),
                Le => order!(Ordering::Less | Ordering::Equal),
                And => op1.and(&op2),
                Or => op1.or(&op2),
                Not => op1.not(),
                Neg => op1.neg(),
                ParenLeft | ParenRight => unreachable!(),
            });
            Ok(())
        } else {
            Err(())
        }
    }
}

#[inline]
fn check_op(op: Operator, op1: &Value, op2: &Value) -> bool {
    use Operator::*;
    match op1 {
        Value::Int(_) => match op {
            Mul => match op2 {
                Value::Str(_) => true,
                _ => op1.is_same(op2),
            },
            Add | Sub | Div | Rem | Eq | Ne | Gt | Ge | Lt | Le => op1.is_same(op2),
            Neg => *op2 == Value::Int(0),
            _ => false,
        },
        Value::Float(_) => match op {
            Add | Mul | Sub | Div | Rem | Eq | Ne | Gt | Ge | Lt | Le => op1.is_same(op2),
            Neg => *op2 == Value::Int(0),
            _ => false,
        },
        Value::Str(_) => match op {
            Mul => match op2 {
                Value::Int(_) => true,
                _ => false,
            },
            Add | Eq | Ne => op1.is_same(op2),
            _ => false,
        },
        Value::Range(_) | Value::Vec(_) => match op {
            Eq | Ne => op1.is_same(op2),
            _ => false,
        },
        Value::Bool(_) => match op {
            Eq | Ne | And | Or => op1.is_same(op2),
            Not => *op2 == Value::Bool(false),
            _ => false,
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_op_preference() {
        assert!(!Operator::Add.gt_preference(Operator::Sub) && Operator::Sub != Operator::Add);
        assert!(!Operator::Not.gt_preference(Operator::Not));
        assert!(!Operator::Add.gt_preference(Operator::Not));
        assert!(Operator::Not.gt_preference(Operator::Add));
    }
}
