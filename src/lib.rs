//! Evaluate some [syn::Expr](https://docs.rs/syn/0.15.29/syn/enum.Expr.html) with context
//!
//! ```rust
//! use v_eval::{Value, Eval};
//!
//!# fn main() -> Result<(), ()> {
//! let e = Eval::default()
//!     .insert("foo", "true")?
//!     .insert("bar", "false")?;
//!
//! assert_eq!(e.eval("foo != bar").unwrap(), Value::Bool(true));
//! assert_eq!(
//!     e.eval("true && foo != bar && true").unwrap(),
//!     Value::Bool(true)
//! );
//! assert_eq!(e.eval("1 == 1 != bar").unwrap(), Value::Bool(true));
//! assert_eq!(e.eval("1 == 1 + 1 == bar").unwrap(), Value::Bool(true));
//!# Ok(())
//!# }
//! ```
//!
#[macro_use]
extern crate quote;

use syn::parse_str;

use std::collections::BTreeMap;

mod operator;
mod reflect;
mod value;

pub use self::reflect::eval;
pub use self::value::Value;

/// Evaluator with context
pub struct Eval(BTreeMap<String, syn::Expr>);

impl Default for Eval {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl Eval {
    pub fn new(c: BTreeMap<String, syn::Expr>) -> Self {
        Self(c)
    }

    /// Parse and insert in context name - syn::Expr
    pub fn insert(mut self, k: &str, v: &str) -> Result<Self, ()> {
        let e = parse_str::<syn::Expr>(v).map_err(|_| ())?;
        self.0.insert(k.to_owned(), e);

        Ok(self)
    }

    /// Remove key in context
    pub fn remove(mut self, k: &str) -> Self {
        self.0.remove(k);

        self
    }

    /// Evaluate expression with current context
    pub fn eval(&self, src: &str) -> Option<Value> {
        parse_str::<syn::Expr>(src)
            .ok()
            .and_then(|src| eval(&self.0, &src))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> Result<(), ()> {
        let e = Eval::default()
            .insert("foo", "true")?
            .insert("fon", "1")?
            .insert("s", r#""foo""#)?
            .insert("bar", "false")?;

        assert_eq!(e.eval("foo != bar").unwrap(), Value::Bool(true));
        assert_eq!(
            e.eval("true && foo != bar && true").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(e.eval("1 == 1 != bar").unwrap(), Value::Bool(true));
        assert_eq!(e.eval("1 == 1 + 1 == bar").unwrap(), Value::Bool(true));
        assert_eq!(e.eval("0..1").unwrap(), Value::Range(0..1));
        assert_eq!(e.eval("(0..1) == (0..1)").unwrap(), Value::Bool(true));
        assert_eq!(e.eval("0..2 * (1 + 1)").unwrap(), Value::Range(0..4));
        assert_eq!(
            e.eval("fon + 1..fon + 2 * (1 + 1)").unwrap(),
            Value::Range(2..5)
        );
        assert_eq!(
            e.eval("fon + 1..fon + 2 * (1 + 1)").unwrap(),
            Value::Range(2..5)
        );
        assert_eq!(e.eval(r#""foo" == s"#).unwrap(), Value::Bool(true));
        assert_eq!(e.eval(r#""bar" != s"#).unwrap(), Value::Bool(true));
        assert_eq!(e.eval("s").unwrap(), Value::Str("foo".into()));
        assert_eq!(
            e.eval("[foo, true]").unwrap(),
            Value::Vec(vec![Value::Bool(true), Value::Bool(true)])
        );
        assert_eq!(
            e.eval("[foo, 1]").unwrap(),
            Value::Vec(vec![Value::Bool(true), Value::Int(1)])
        );
        assert_eq!(
            e.eval("[foo, [1, 2]]").unwrap(),
            Value::Vec(vec![
                Value::Bool(true),
                Value::Vec(vec![Value::Int(1), Value::Int(2)])
            ])
        );
        assert_eq!(
            e.eval(r#""foo" * 2 * 2"#).unwrap(),
            Value::Str("foofoofoofoo".into())
        );
        assert_eq!(
            e.eval(r#""foo" * (2 * 2 - 1 + 1)"#).unwrap(),
            Value::Str("foofoofoofoo".into())
        );
        assert_eq!(
            e.eval(r#"("foo" + "bar") * 2"#).unwrap(),
            Value::Str("foobarfoobar".into())
        );
        assert_eq!(
            e.eval(r#"("bar" + s * 2) * 2"#).unwrap(),
            Value::Str("barfoofoobarfoofoo".into())
        );

        Ok(())
    }
}
