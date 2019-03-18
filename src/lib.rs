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

use std::collections::{BTreeMap, HashMap};

mod operator;
mod reflect;
mod values;

pub use self::reflect::eval;
pub use self::values::Value;

/// Evaluator with context
pub struct Eval(HashMap<String, syn::Expr>);

impl Default for Eval {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl Eval {
    pub fn new(c: HashMap<String, syn::Expr>) -> Self {
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
            .map_or(None, |src| eval(&ctx_as_ref(&self.0), &src))
    }
}

/// Cast context elements to references
pub fn ctx_as_ref(ctx: &HashMap<String, syn::Expr>) -> BTreeMap<&str, &syn::Expr> {
    let mut b: BTreeMap<&str, &syn::Expr> = BTreeMap::new();
    for (k, v) in ctx {
        b.insert(k, v);
    }

    b
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> Result<(), ()> {
        let e = Eval::default()
            .insert("foo", "true")?
            .insert("bar", "false")?;

        assert_eq!(e.eval("foo != bar").unwrap(), Value::Bool(true));
        assert_eq!(
            e.eval("true && foo != bar && true").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(e.eval("1 == 1 != bar").unwrap(), Value::Bool(true));
        assert_eq!(e.eval("1 == 1 + 1 == bar").unwrap(), Value::Bool(true));

        Ok(())
    }
}
