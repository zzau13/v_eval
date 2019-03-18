# v_eval [![Documentation](https://docs.rs/v_eval/badge.svg)](https://docs.rs/v_eval/) [![Latest version](https://img.shields.io/crates/v/v_eval.svg)](https://crates.io/crates/v_eval)

Expression evaluator with context

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

## Contributing
Please, contribute to v_eval! The more the better! Feel free to to open an issue and/or contacting directly with the 
owner for any request or suggestion.


## Code of conduct
This Code of Conduct is adapted from the [Contributor Covenant][homepage], version 1.4, available at [http://contributor-covenant.org/version/1/4][version]

[homepage]: http://contributor-covenant.org
[version]: http://contributor-covenant.org/version/1/4/

## License
This project is distributed under the terms of both the Apache License (Version 2.0) and the MIT license, specified in 
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) respectively.

## Support
[Patreon][patreon]

[patreon]: https://www.patreon.com/r_iendo 
