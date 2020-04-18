use super::MathNode;
use roxmltree::Node;
use serde_derive::{Deserialize, Serialize};
use std::num::{ParseFloatError, ParseIntError};
use std::string::ParseError;
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NumType {
    Real(f64),
    Integer(i64),
    Rational(i64, i64),
    ComplexCartesian(f64, f64),
    ComplexPolar(f64, f64),
    Constant(String),
}
impl Eq for NumType {}
impl PartialEq for NumType {
    fn eq(&self, other: &Self) -> bool {
        use NumType::*;
        match (self, other) {
            (Real(r), Real(r2)) => approx::abs_diff_eq!(r, r2),
            (Integer(r1), Integer(r2)) => r1 == r2,
            (Rational(a, b), Rational(c, d)) => a == c && b == d,
            (ComplexPolar(a, b), ComplexPolar(c, d))
            | (ComplexCartesian(a, b), ComplexCartesian(c, d)) => {
                approx::abs_diff_eq!(a, c) && approx::abs_diff_eq!(b, c)
            }
            (Constant(a), Constant(b)) => a == b,
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
pub(crate) fn node_to_cn(node: Node) -> MathNode {
    let num_type_str = node.attribute("type").unwrap_or("real");
    let base: u32 = node.attribute("base").unwrap_or("10").parse().unwrap();
    dbg!(node);
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
        _ => panic!("Unhandled number type"),
    };

    let encoding = node.attribute("encoding").map(|p| p.parse().unwrap());
    let definition_url = node.attribute("definitionUrl").map(|p| p.parse().unwrap());
    MathNode::Cn {
        num_type,
        base,
        definition_url,
        encoding,
    }
}

fn extract_float_children(node: Node) -> Result<(f64, f64), ParseFloatError> {
    let child1 = parse_and_trim_float(node.first_child().unwrap())?;
    let child2 = parse_and_trim_float(node.children().nth_back(0).unwrap())?;
    Ok((child1, child2))
}
