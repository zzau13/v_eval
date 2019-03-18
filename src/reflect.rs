use syn::{visit::Visit, BinOp, Expr, ExprBinary, ExprParen, ExprPath, ExprUnary, Lit};

use std::{cmp::Ordering, collections::BTreeMap};

use super::{operator::Operator, Value};

pub fn eval(ctx: &BTreeMap<&str, &syn::Expr>, expr: &Expr) -> Option<Value> {
    Reflect::new(ctx).eval(expr)
}

#[derive(Debug)]
enum Output {
    Op(Operator),
    V(Value),
}

struct Reflect<'a> {
    ctx: &'a BTreeMap<&'a str, &'a syn::Expr>,
    operators: Vec<Operator>,
    output: Vec<Output>,
    on_err: bool,
}

macro_rules! err_attrs {
    ($_self:ident, $attrs:expr) => {
        if !$attrs.is_empty() {
            return $_self.on_err = true;
        }
    };
}

macro_rules! err_some {
    ($_self:ident, $some:expr) => {
        if $some.is_some() {
            return $_self.on_err = true;
        }
    };
}

macro_rules! on_err {
    ($_self:ident) => {
        if $_self.on_err {
            return;
        }
    };
}

impl<'a> Reflect<'a> {
    fn new<'n>(ctx: &'n BTreeMap<&'n str, &'n syn::Expr>) -> Reflect<'n> {
        Reflect {
            ctx,
            operators: vec![],
            output: vec![],
            on_err: false,
        }
    }

    fn eval(mut self, e: &'a Expr) -> Option<Value> {
        self.visit_expr(e);

        if self.on_err {
            None
        } else {
            self.output.extend(
                self.operators
                    .drain(..)
                    .rev()
                    .map(|o| Output::Op(o))
                    .collect::<Vec<Output>>(),
            );
            evaluate(self.output).ok()
        }
    }

    fn push_op(&mut self, op: Operator) {
        on_err!(self);
        if Operator::ParenLeft.eq_preference(&op) {
            if op == Operator::ParenRight {
                loop {
                    if let Some(last) = self.operators.last() {
                        if *last == Operator::ParenLeft {
                            self.operators.pop();
                            break;
                        }
                        self.output.push(Output::Op(self.operators.pop().unwrap()));
                    } else {
                        break self.on_err = true;
                    }
                }
            } else {
                self.operators.push(op);
            }
        } else {
            while let Some(last) = self.operators.last() {
                if op.gt_preference(&last) || *last == Operator::ParenLeft {
                    break;
                } else {
                    self.output.push(Output::Op(self.operators.pop().unwrap()));
                }
            }
            self.operators.push(op);
        }
    }
}

impl<'a> Visit<'a> for Reflect<'a> {
    fn visit_attribute(&mut self, _a: &'a syn::Attribute) {
        self.on_err = true;
    }

    fn visit_bin_op(&mut self, b: &'a BinOp) {
        on_err!(self);
        use syn::BinOp::*;
        match *b {
            Add(_) => self.push_op(Operator::Add),
            Sub(_) => self.push_op(Operator::Sub),
            Mul(_) => self.push_op(Operator::Mul),
            Div(_) => self.push_op(Operator::Div),
            Rem(_) => self.push_op(Operator::Rem),
            And(_) => self.push_op(Operator::And),
            Or(_) => self.push_op(Operator::Or),
            Eq(_) => self.push_op(Operator::Eq),
            Ne(_) => self.push_op(Operator::Ne),
            Lt(_) => self.push_op(Operator::Lt),
            Le(_) => self.push_op(Operator::Le),
            Gt(_) => self.push_op(Operator::Gt),
            Ge(_) => self.push_op(Operator::Ge),
            _ => self.on_err = true,
        }
    }

    fn visit_expr(&mut self, e: &'a Expr) {
        on_err!(self);
        use syn::Expr::*;
        match e {
            Binary(i) => self.visit_expr_binary(i),
            Lit(i) => self.visit_expr_lit(i),
            Paren(i) => self.visit_expr_paren(i),
            Path(i) => self.visit_expr_path(i),
            Unary(i) => self.visit_expr_unary(i),
            _ => self.on_err = true,
        }
    }

    fn visit_expr_binary(
        &mut self,
        ExprBinary {
            attrs,
            left,
            op,
            right,
        }: &'a ExprBinary,
    ) {
        err_attrs!(self, attrs);
        self.visit_expr(left);
        on_err!(self);
        self.visit_bin_op(op);
        on_err!(self);
        self.visit_expr(right);
    }

    fn visit_expr_paren(&mut self, ExprParen { attrs, expr, .. }: &'a ExprParen) {
        err_attrs!(self, attrs);
        self.push_op(Operator::ParenLeft);
        on_err!(self);
        self.visit_expr(expr);
        on_err!(self);
        self.push_op(Operator::ParenRight);
    }

    fn visit_expr_path(&mut self, ExprPath { attrs, qself, path }: &'a ExprPath) {
        err_attrs!(self, attrs);
        err_some!(self, qself);
        let path: &str = &quote!(#path).to_string();
        if let Some(src) = self.ctx.get(&path) {
            self.push_op(Operator::ParenLeft);
            self.visit_expr(src);
            self.push_op(Operator::ParenRight);
        } else {
            self.on_err = true;
        }
    }

    fn visit_expr_unary(&mut self, ExprUnary { attrs, op, expr }: &'a ExprUnary) {
        err_attrs!(self, attrs);
        self.visit_expr(expr);
        use syn::UnOp::*;
        match op {
            Not(_) => {
                self.push_op(Operator::Not);
                self.output.push(Output::V(Value::Bool(false)));
            }
            Neg(_) => {
                self.push_op(Operator::Neg);
                self.output.push(Output::V(Value::Int(0)));
            }
            _ => return self.on_err = true,
        }
    }

    fn visit_lit(&mut self, l: &'a Lit) {
        use syn::Lit::*;
        match l {
            Int(a) => self.output.push(Output::V(Value::Int(a.value() as i64))),
            Float(a) => self.output.push(Output::V(Value::Float(a.value()))),
            Bool(a) => self.output.push(Output::V(Value::Bool(a.value))),
            _ => self.on_err = true,
        }
    }
}

#[inline]
fn evaluate(output: Vec<Output>) -> Result<Value, ()> {
    let mut stack = Vec::new();
    for o in output {
        match o {
            Output::V(v) => stack.push(v),
            Output::Op(ref op) => {
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

                if check_op(op, &op1, &op2) {
                    use Operator::*;
                    stack.push(match op {
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
                } else {
                    return Err(());
                }
            }
        }
    }

    if stack.len() == 1 {
        stack.pop().ok_or(())
    } else {
        Err(())
    }
}

#[inline]
fn check_op(op: &Operator, op1: &Value, op2: &Value) -> bool {
    use Operator::*;
    match op1 {
        Value::Int(_) | Value::Float(_) => match op {
            Add | Sub | Mul | Div | Rem | Eq | Ne | Gt | Ge | Lt | Le => op1.is_same(op2),
            Neg => *op2 == Value::Int(0),
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
    use syn::parse_str;

    use super::super::operator::Operator::*;
    use super::super::Value::*;
    use super::Output::*;
    use super::*;

    #[test]
    fn test_evaluate_add() {
        let o_int = vec![V(Int(1)), V(Int(1)), Op(Add)];
        assert_eq!(evaluate(o_int).unwrap(), Int(2));

        let o_float = vec![V(Float(1.0)), V(Float(1.0)), Op(Add)];
        assert_eq!(evaluate(o_float).unwrap(), Float(2.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Add)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_sub() {
        let o_int = vec![V(Int(1)), V(Int(1)), Op(Sub)];
        assert_eq!(evaluate(o_int).unwrap(), Int(0));

        let o_float = vec![V(Float(1.0)), V(Float(1.0)), Op(Sub)];
        assert_eq!(evaluate(o_float).unwrap(), Float(0.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Sub)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_mul() {
        let o_int = vec![V(Int(1)), V(Int(1)), Op(Mul)];
        assert_eq!(evaluate(o_int).unwrap(), Int(1));

        let o_float = vec![V(Float(1.0)), V(Float(1.0)), Op(Mul)];
        assert_eq!(evaluate(o_float).unwrap(), Float(1.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Mul)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_div() {
        let o_int = vec![V(Int(1)), V(Int(1)), Op(Div)];
        assert_eq!(evaluate(o_int).unwrap(), Int(1));

        let o_float = vec![V(Float(1.0)), V(Float(1.0)), Op(Div)];
        assert_eq!(evaluate(o_float).unwrap(), Float(1.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Div)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_rem() {
        let o_int = vec![V(Int(4)), V(Int(2)), Op(Rem)];
        assert_eq!(evaluate(o_int).unwrap(), Int(0));

        let o_float = vec![V(Float(4.0)), V(Float(2.0)), Op(Rem)];
        assert_eq!(evaluate(o_float).unwrap(), Float(0.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Rem)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_eq() {
        let o_int = vec![V(Int(2)), V(Int(2)), Op(Eq)];
        assert_eq!(evaluate(o_int).unwrap(), Bool(true));

        let o_float = vec![V(Float(2.0)), V(Float(2.0)), Op(Eq)];
        assert_eq!(evaluate(o_float).unwrap(), Bool(true));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Eq)];
        assert_eq!(evaluate(o_bool).unwrap(), Bool(false));
    }

    #[test]
    fn test_evaluate_ge() {
        let o_int = vec![V(Int(2)), V(Int(2)), Op(Ge)];
        assert_eq!(evaluate(o_int).unwrap(), Bool(true));

        let o_float = vec![V(Float(2.0)), V(Float(2.0)), Op(Ge)];
        assert_eq!(evaluate(o_float).unwrap(), Bool(true));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Ge)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_le() {
        let o_int = vec![V(Int(2)), V(Int(2)), Op(Le)];
        assert_eq!(evaluate(o_int).unwrap(), Bool(true));

        let o_float = vec![V(Float(2.0)), V(Float(2.0)), Op(Le)];
        assert_eq!(evaluate(o_float).unwrap(), Bool(true));

        let o_bool = vec![V(Bool(true)), V(Bool(true)), Op(Le)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_gt() {
        let o_int = vec![V(Int(2)), V(Int(1)), Op(Gt)];
        assert_eq!(evaluate(o_int).unwrap(), Bool(true));

        let o_float = vec![V(Float(2.0)), V(Float(1.9)), Op(Gt)];
        assert_eq!(evaluate(o_float).unwrap(), Bool(true));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Gt)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_evaluate_lt() {
        let o_int = vec![V(Int(1)), V(Int(2)), Op(Lt)];
        assert_eq!(evaluate(o_int).unwrap(), Bool(true));

        let o_float = vec![V(Float(1.0)), V(Float(1.1)), Op(Lt)];
        assert_eq!(evaluate(o_float).unwrap(), Bool(true));

        let o_bool = vec![V(Bool(true)), V(Bool(true)), Op(Lt)];
        assert!(evaluate(o_bool).is_err());
    }

    #[test]
    fn test_eval_literal() {
        let src = "true";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(true));

        let src = "-1";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Int(-1));

        let src = "-1.0";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Float(-1.0));

        let src = "foo";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1").unwrap();
        ctx.insert("foo", &arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Int(-1));
    }

    #[test]
    fn test_eval_one() {
        let src = "true + true";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e), None);

        let src = "1 + 1";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Int(2));
    }

    #[test]
    fn test_eval() {
        let src = "true && true || false == true";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(true));

        let src = "1 + 1 - 6 % 5";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Int(1));

        let src = "1 + 1 - 10 / 5";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Int(0));

        let src = "1 * 1 - 10 / 5 == -1";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(true));

        let src = "-1 == 1 * 1 - 10 / 5";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(true));

        let src = "!(1 * 1 - 10 / 5 == -1)";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e), Some(Bool(false)));

        let src = "!(-1 == 1 * 1 - 10 / 5)";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let ctx = BTreeMap::new();

        assert_eq!(eval(&ctx, &e), Some(Bool(false)));

        let src = "!(1 * 1 - 10 / 5 == foo)";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1").unwrap();

        ctx.insert("foo", &arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(false));

        let src = "!(foo == 1 * 1 - 10 / 5)";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1").unwrap();

        ctx.insert("foo", &arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(false));

        let src = "(foo * 2) + 1";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1 + 1").unwrap();

        ctx.insert("foo", &arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Int(1));
    }
}
