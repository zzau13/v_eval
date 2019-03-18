# v_eval

```rust
use v_eval::{Value, Eval};

fn main() -> Result<(), ()> {
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
```