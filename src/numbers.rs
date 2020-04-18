use super::MathNode;
use roxmltree::Node;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::num::{ParseFloatError, ParseIntError};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumType {
    Real(f64),
    Integer(i64),
    Rational(i64, i64),
    ComplexCartesian(f64, f64),
    ComplexPolar(f64, f64),
    Constant(String),
    ENotation(f64, i64),
}
impl Eq for NumType {}
impl PartialEq for NumType {
    fn eq(&self, other: &Self) -> bool {
        use NumType::*;
        match (self, other) {
            (Real(r), Real(r2)) => approx::abs_diff_eq!(r, r2),
            (Integer(r1), Integer(r2)) => r1 == r2,
            (Rational(a, b), Rational(c, d)) => (a == c) && (b == d),
            (ComplexPolar(a, b), ComplexPolar(c, d))
            | (ComplexCartesian(a, b), ComplexCartesian(c, d)) => {
                approx::abs_diff_eq!(a, c) && approx::abs_diff_eq!(b, d)
            }
            (Constant(a), Constant(b)) => a == b,
            (ENotation(a, b), ENotation(c, d)) => a == c && b == d,
            _ => false,
        }
    }
}

fn parse_and_trim_int(node: Node, base: u32) -> Result<i64, ParseIntError> {
    i64::from_str_radix(node.text().unwrap().trim(), base)
}
fn parse_and_trim_float(node: Node) -> Result<f64, ParseFloatError> {
    node.text().unwrap().trim().parse()
}
fn extract_enotation(node: Node) -> Result<(f64, i64), Box<dyn std::error::Error>> {
    // We can either have 1 child (SBML) or 3 children (MathML)
    let children_count = node.children().count();
    match children_count {
        3 => extract_float_int_children(node),
        1 => {
            let first_child: Vec<String> = node
                .first_child()
                .unwrap()
                .text()
                .unwrap()
                .to_lowercase()
                .split("e")
                .map(|e| e.trim().to_owned())
                .collect();

            let exponent: f64 = first_child[0].parse().unwrap();
            let mantissa: i64 = first_child[1].parse().unwrap();
            Ok((exponent, mantissa))
        }

        _ => panic!("We can only ever have 3 or 1 children"),
    }
}
pub(crate) fn node_to_cn(node: Node) -> MathNode {
    // TODO: make static
    let ignore_attrs: HashSet<&str> = vec!["type", "base", "encoding", "definitionUrl", "units"]
        .into_iter()
        .collect();
    let num_type_str = node.attribute("type").unwrap_or("real");
    let base: u32 = node.attribute("base").unwrap_or("10").parse().unwrap();

    let num_type = match num_type_str {
        "real" => NumType::Real(node.text().unwrap().trim().parse().unwrap()),
        "integer" => NumType::Integer(parse_and_trim_int(node, base).unwrap()),
        "rational" => {
            let child1 = parse_and_trim_int(node.first_child().unwrap(), base).unwrap();
            let child2 = parse_and_trim_int(node.children().nth_back(0).unwrap(), base).unwrap();
            NumType::Rational(child1, child2)
        }
        "complex-cartesian" => {
            let (a, b) = extract_float_children(node).unwrap();
            NumType::ComplexCartesian(a, b)
        }
        "complex-polar" => {
            let (a, b) = extract_float_children(node).unwrap();
            NumType::ComplexPolar(a, b)
        }
        "constant" => NumType::Constant(
            node.first_child()
                .unwrap()
                .text()
                .unwrap()
                .trim()
                .to_owned(),
        ),
        // This one can either be number <sep> number or just 2e-5 for SBML, we will support both
        "e-notation" => {
            let (a, b) = extract_enotation(node).unwrap();
            NumType::ENotation(a, b)
        }
        _ => panic!("Unhandled number type"),
    };

    let encoding = node.attribute("encoding").map(|p| p.parse().unwrap());
    let definition_url = node.attribute("definitionUrl").map(|p| p.parse().unwrap());

    let attributes: HashMap<String, String> = node
        .attributes()
        .iter()
        .filter(|n| {
            if ignore_attrs.contains(n.name()) & n.namespace().is_none() {
                false
            } else {
                true
            }
        })
        .map(|a| {
            (
                format!(
                    "{}:{}",
                    a.namespace().unwrap().to_owned(),
                    a.name().to_owned()
                ),
                a.value().to_owned(),
            )
        })
        .collect();
    MathNode::Cn {
        num_type,
        base,
        definition_url,
        encoding,
        attributes: if attributes.is_empty() {
            None
        } else {
            Some(attributes)
        },
    }
}

fn extract_float_children(node: Node) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let child1 = parse_and_trim_float(node.first_child().unwrap())?;
    let child2 = parse_and_trim_float(node.children().nth_back(0).unwrap())?;
    Ok((child1, child2))
}
fn extract_float_int_children(node: Node) -> Result<(f64, i64), Box<dyn std::error::Error>> {
    let child1 = parse_and_trim_float(node.first_child().unwrap())?;
    let child2 = parse_and_trim_int(node.children().nth_back(0).unwrap(), 10)?;
    Ok((child1, child2))
}
#[cfg(test)]
mod test {

    #[test]
    fn test_number_eq() {
        use super::NumType::*;
        assert_eq!(Constant("t".to_string()), Constant("t".to_string()));
        assert_eq!(Real(1212.212), Real(1212.212));
        assert_eq!(Integer(12), Integer(12))
    }
    #[test]
    fn test_number_e() {
        use super::node_to_cn;
        use super::MathNode::*;
        use super::NumType::*;
        let test = r#"<cn type="e-notation"> 2e-5 </cn>"#;
        let parsed = roxmltree::Document::parse(test).unwrap();
        let ret = node_to_cn(parsed.root().first_child().unwrap());
        let expected = Cn {
            num_type: ENotation(2.0, -5),
            base: 10,
            definition_url: None,
            encoding: None,
            attributes: None,
        };
        assert_eq!(ret, expected);
        let test = r#"<cn type="e-notation"> 2 <sep/> -5 </cn>"#;
        let parsed = roxmltree::Document::parse(test).unwrap();
        let ret = node_to_cn(parsed.root().first_child().unwrap());
        assert_eq!(ret, expected);
    }
}
