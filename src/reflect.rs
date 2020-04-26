use std::{collections::BTreeMap, convert::TryFrom, ops};

use syn::{
    visit::Visit, BinOp, Expr, ExprArray, ExprBinary, ExprIndex, ExprMethodCall, ExprParen,
    ExprPath, ExprRange, ExprReference, ExprUnary, Lit,
};

use crate::{
    method::{HasArg, Method},
    operator::Operator,
    Value,
};

pub fn eval(ctx: &BTreeMap<String, syn::Expr>, expr: &Expr) -> Option<Value> {
    Reflect::new(ctx).eval(expr)
}

#[derive(Debug)]
enum Output {
    Op(Operator),
    V(Value),
    Fn(Method),
}

struct Reflect<'a> {
    ctx: &'a BTreeMap<String, syn::Expr>,
    operators: Vec<Operator>,
    output: Vec<Output>,
    on_err: bool,
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
    fn new(ctx: &BTreeMap<String, syn::Expr>) -> Reflect {
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
                    .map(Output::Op)
                    .collect::<Vec<Output>>(),
            );
            evaluate(self.output).ok()
        }
    }

    fn push_op(&mut self, op: Operator) {
        on_err!(self);
        if Operator::ParenLeft.eq_preference(op) {
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
                if op.gt_preference(*last) || *last == Operator::ParenLeft {
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
            Array(i) => self.visit_expr_array(i),
            Range(i) => self.visit_expr_range(i),
            Index(i) => self.visit_expr_index(i),
            Reference(i) => self.visit_expr_reference(i),
            MethodCall(i) => self.visit_expr_method_call(i),
            _ => self.on_err = true,
        }
    }

    fn visit_expr_array(&mut self, ExprArray { elems, .. }: &'a ExprArray) {
        let mut v = Vec::with_capacity(elems.len());
        for elem in elems {
            if let Some(val) = eval(self.ctx, elem) {
                v.push(val)
            } else {
                self.on_err = true;
                return;
            }
        }

        self.output.push(Output::V(Value::Vec(v)));
    }
    fn visit_expr_binary(
        &mut self,
        ExprBinary {
            left, op, right, ..
        }: &'a ExprBinary,
    ) {
        self.visit_expr(left);
        self.visit_bin_op(op);
        self.visit_expr(right);
    }

    fn visit_expr_index(&mut self, ExprIndex { expr, index, .. }: &'a ExprIndex) {
        use syn::Expr::*;
        match **expr {
            Paren(_) | Lit(_) | Array(_) | Path(_) => match eval(self.ctx, &*expr) {
                Some(Value::Vec(a)) => {
                    match eval(self.ctx, index).and_then(|v| match v {
                        Value::Int(i) => TryFrom::try_from(i)
                            .ok()
                            .and_then(|i: usize| a.get(i).cloned()),
                        Value::Range(i) => TryFrom::try_from(i.start)
                            .and_then(|start| {
                                TryFrom::try_from(i.end).map(|end| ops::Range { start, end })
                            })
                            .ok()
                            .and_then(|i: ops::Range<usize>| a.get(i))
                            .map(|x| Value::Vec(x.to_vec())),
                        _ => None,
                    }) {
                        Some(i) => self.output.push(Output::V(i)),
                        _ => self.on_err = true,
                    }
                }
                Some(Value::Str(a)) => {
                    match eval(self.ctx, index)
                        .and_then(|v| match v {
                            Value::Range(i) => TryFrom::try_from(i.start).ok().and_then(|start| {
                                TryFrom::try_from(i.end).ok().map(|end| start..end)
                            }),
                            _ => None,
                        })
                        .and_then(|i: ops::Range<usize>| a.get(i))
                    {
                        Some(i) => self.output.push(Output::V(Value::Str(i.to_owned()))),
                        _ => self.on_err = true,
                    }
                }
                _ => self.on_err = true,
            },
            _ => self.on_err = true,
        }
    }

    #[inline]
    fn visit_expr_method_call(
        &mut self,
        ExprMethodCall {
            receiver,
            method,
            args,
            ..
        }: &'a ExprMethodCall,
    ) {
        self.push_op(Operator::ParenLeft);
        self.visit_expr(receiver);
        self.push_op(Operator::ParenRight);
        let method: &str = &method.to_string();
        let method = match method.parse() {
            Ok(m) => Method::F64(m),
            Err(_) => return self.on_err = true,
        };
        if method.has_arg() {
            if args.len() != 1 {
                return self.on_err = true;
            }
            self.push_op(Operator::ParenLeft);
            self.visit_expr(&args[0]);
            self.push_op(Operator::ParenRight);
            self.output.push(Output::Fn(method));
        } else {
            if !args.is_empty() {
                return self.on_err = true;
            }
            self.output.push(Output::Fn(method));
        }
    }

    fn visit_expr_paren(&mut self, ExprParen { expr, .. }: &'a ExprParen) {
        self.push_op(Operator::ParenLeft);
        self.visit_expr(expr);
        self.push_op(Operator::ParenRight);
    }

    fn visit_expr_path(&mut self, ExprPath { qself, path, .. }: &'a ExprPath) {
        err_some!(self, qself);
        let path = match path.get_ident() {
            Some(i) => i.to_string(),
            _ => return self.on_err = true,
        };
        if let Some(src) = self.ctx.get(&path) {
            self.push_op(Operator::ParenLeft);
            self.visit_expr(src);
            self.push_op(Operator::ParenRight);
        } else {
            self.on_err = true;
        }
    }

    fn visit_expr_range(&mut self, ExprRange { from, to, .. }: &'a ExprRange) {
        if let Some(range) = from
            .as_ref()
            .and_then(|from| Reflect::new(self.ctx).eval(&*from))
            .and_then(|from| {
                if let Value::Int(from) = from {
                    Some(from)
                } else {
                    None
                }
            })
            .and_then(|from| {
                to.as_ref()
                    .and_then(|to| {
                        Reflect::new(self.ctx).eval(&*to).and_then(|to| {
                            if let Value::Int(to) = to {
                                Some(to)
                            } else {
                                None
                            }
                        })
                    })
                    .map(|to| from..to)
            })
        {
            self.output.push(Output::V(Value::Range(range)));
        } else {
            self.on_err = true;
        }
    }

    fn visit_expr_reference(&mut self, ExprReference { expr, .. }: &'a ExprReference) {
        self.visit_expr(expr);
    }

    fn visit_expr_unary(&mut self, ExprUnary { op, expr, .. }: &'a ExprUnary) {
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
            _ => self.on_err = true,
        }
    }

    fn visit_lit(&mut self, l: &'a Lit) {
        use syn::Lit::*;
        match l {
            Int(a) => match a.base10_parse() {
                Ok(n) => self.output.push(Output::V(Value::Int(n))),
                _ => self.on_err = true,
            },
            Float(a) => match a.base10_parse() {
                Ok(n) => self.output.push(Output::V(Value::Float(n))),
                _ => self.on_err = true,
            },
            Bool(a) => self.output.push(Output::V(Value::Bool(a.value))),
            Str(a) => self.output.push(Output::V(Value::Str(a.value()))),
            Char(a) => self
                .output
                .push(Output::V(Value::Str(a.value().to_string()))),
            _ => self.on_err = true,
        }
    }
}

pub(crate) trait Eval {
    fn eval(self, stack: &mut Vec<Value>) -> Result<(), ()>;
}

#[inline]
fn evaluate(output: Vec<Output>) -> Result<Value, ()> {
    let mut stack = Vec::new();
    for o in output {
        match o {
            Output::V(v) => stack.push(v),
            Output::Fn(m) => m.eval(&mut stack)?,
            Output::Op(op) => op.eval(&mut stack)?,
        }
    }

    if stack.len() == 1 {
        stack.pop().ok_or(())
    } else {
        Err(())
    }
}

#[cfg(test)]
mod test {
    use syn::parse_str;

    use super::{
        super::{operator::Operator::*, Value::*},
        Output::*,
        *,
    };

    #[test]
    fn test_evaluate_add() {
        let o_int = vec![V(Int(1)), V(Int(1)), Op(Add)];
        assert_eq!(evaluate(o_int).unwrap(), Int(2));

        let o_float = vec![V(Float(1.0)), V(Float(1.0)), Op(Add)];
        assert_eq!(evaluate(o_float).unwrap(), Float(2.0));

        let o_bool = vec![V(Bool(true)), V(Bool(false)), Op(Add)];
        assert!(evaluate(o_bool).is_err());

        let o = vec![V(Str("bar".into())), V(Str("foo".into())), Op(Add)];
        assert_eq!(evaluate(o).unwrap(), Str("barfoo".into()));
    }

    #[test]
    fn test_evaluate_sub() {
        let o = vec![V(Int(1)), V(Int(1)), Op(Sub)];
        assert_eq!(evaluate(o).unwrap(), Int(0));

        let o = vec![V(Float(1.0)), V(Float(1.0)), Op(Sub)];
        assert_eq!(evaluate(o).unwrap(), Float(0.0));

        let o = vec![V(Bool(true)), V(Bool(false)), Op(Sub)];
        assert!(evaluate(o).is_err());
    }

    #[test]
    fn test_evaluate_mul() {
        let o = vec![V(Int(1)), V(Int(1)), Op(Mul)];
        assert_eq!(evaluate(o).unwrap(), Int(1));

        let o = vec![V(Float(1.0)), V(Float(1.0)), Op(Mul)];
        assert_eq!(evaluate(o).unwrap(), Float(1.0));

        let o = vec![V(Bool(true)), V(Bool(false)), Op(Mul)];
        assert!(evaluate(o).is_err());

        let o = vec![V(Int(2)), V(Str("foo".into())), Op(Mul)];
        assert_eq!(evaluate(o).unwrap(), Str("foofoo".into()));

        let o = vec![V(Str("foo".into())), V(Int(2)), Op(Mul)];
        assert_eq!(evaluate(o).unwrap(), Str("foofoo".into()));
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
        let o = vec![V(Int(2)), V(Int(2)), Op(Eq)];
        assert_eq!(evaluate(o).unwrap(), Bool(true));

        let o = vec![V(Float(2.0)), V(Float(2.0)), Op(Eq)];
        assert_eq!(evaluate(o).unwrap(), Bool(true));

        let o = vec![V(Bool(true)), V(Bool(false)), Op(Eq)];
        assert_eq!(evaluate(o).unwrap(), Bool(false));

        let o = vec![V(Str("foo".into())), V(Str("foo".into())), Op(Eq)];
        assert_eq!(evaluate(o).unwrap(), Bool(true));

        let o = vec![V(Range(0..1)), V(Range(0..1)), Op(Eq)];
        assert_eq!(evaluate(o).unwrap(), Bool(true));
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
        ctx.insert("foo".to_string(), arg);

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

        ctx.insert("foo".to_string(), arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(false));

        let src = "!(foo == 1 * 1 - 10 / 5)";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1").unwrap();

        ctx.insert("foo".to_string(), arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Bool(false));

        let src = "(foo * 2) + 1";
        let e = parse_str::<syn::Expr>(src).unwrap();
        let mut ctx = BTreeMap::new();
        let arg = parse_str::<syn::Expr>("-1 + 1").unwrap();

        ctx.insert("foo".to_string(), arg);

        assert_eq!(eval(&ctx, &e).unwrap(), Int(1));
    }
}
