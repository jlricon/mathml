[![Crates.io](https://img.shields.io/crates/v/mathml.svg)](https://crates.io/crates/mathml)
[![Documentation](https://docs.rs/mathml/badge.svg)](https://docs.rs/mathml/)
# Mathml parser
This implements a parser for the MathML spec https://www.w3.org/TR/2003/REC-MathML2-20031021/chapter4.html
At the moment only content markup is implemented 
## Usage example

```rust
use mathml::parse_document;
let test = "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">
                    <apply>
                  <plus/>
              <ci> x </ci>
              <ci> y </ci>
            </apply></math>";
let res = parse_document(test).unwrap();
let exp = Root(vec![Apply(vec![
    Op(BuiltinOp::plus),
    Ci(vec![Text("x".to_owned())]),
    Ci(vec![Text("y".to_owned())]),
])]);
assert_eq!(res, exp);
```