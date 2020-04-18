#![allow(dead_code)]
use roxmltree;
use roxmltree::Node;
use roxmltree::NodeType;
use serde_derive::{Deserialize, Serialize};
use serde_plain;

mod numbers;
mod regexes;
#[macro_use]
extern crate approx;
type Op = String;
#[derive(Deserialize, Debug, Serialize, Eq, PartialEq)]
#[allow(non_camel_case_types)]
enum BuiltinOp {
    factorial,
    minus,
    abs,
    conjugate,
    arg,
    real,
    imaginary,
    floor,
    ceiling,
    not,
    inverse,
    ident,
    domain,
    codomain,
    image,
    sin,
    cos,
    tan,
    sec,
    csc,
    cot,
    sinh,
    cosh,
    tanh,
    sech,
    csch,
    coth,
    arcsin,
    arccos,
    arctan,
    arccosh,
    arccot,
    arccoth,
    arccsc,
    arccsch,
    arcsec,
    arcsech,
    arcsinh,
    arctanh,
    exp,
    ln,
    log,
    determinant,
    transpose,
    divergence,
    grad,
    curl,
    laplacian,
    card,
    quotient,
    divide,
    power,
    rem,
    implies,
    equivalent,
    approx,
    setdiff,
    vectorproduct,
    scalarproduct,
    outerproduct,
    plus,
    times,
    max,
    min,
    gcd,
    lcm,
    mean,
    sdev,
    variance,
    median,
    mode,
    and,
    or,
    xor,
    selector,
    union,
    intersect,
    cartesianproduct,
    compose,
    r#fn,
    int,
    sum,
    product,
    diff,
    partialdiff,
    forall,
    exists,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
enum MathNode {
    Apply(Vec<MathNode>),
    Op(BuiltinOp),
    Text(String),
    Root(Vec<MathNode>),
    Ci(Vec<MathNode>),
    Csymbol {
        definition_url: String,
        encoding: Option<String>,
        children: Vec<MathNode>,
    },
    Cn {
        num_type: numbers::NumType,
        base: u32,
        definition_url: Option<String>,
        encoding: Option<String>,
    },
}

fn has_text(math_node: &MathNode) -> bool {
    match math_node {
        MathNode::Text(e) if e.is_empty() => false,
        _ => true,
    }
}
fn map_children(node: Node) -> Vec<MathNode> {
    node.children().map(parse_node).filter(has_text).collect()
}
fn parse_element_type(node: Node) -> MathNode {
    let tag_name = node.tag_name().name();
    // Is this a defined op?
    let maybe_op: Result<BuiltinOp, serde_plain::Error> = serde_plain::from_str(tag_name);
    if let Ok(op) = maybe_op {
        return MathNode::Op(op);
    }
    return match tag_name {
        "apply" => MathNode::Apply(map_children(node)),
        "ci" => MathNode::Ci(map_children(node)),
        "cn" => numbers::node_to_cn(node),
        "csymbol" => MathNode::Csymbol {
            definition_url: node.attribute("definitionUrl").unwrap().to_owned(),
            encoding: node.attribute("encoding").map(|e| e.to_owned()),
            children: map_children(node),
        },
        _ => {
            dbg!(node);
            panic!()
        }
    };
}
fn parse_node(node: Node) -> MathNode {
    match node.node_type() {
        NodeType::Text => MathNode::Text(node.text().unwrap().trim().to_owned()),
        NodeType::Element if node.tag_name().name() == "math" => MathNode::Root(map_children(node)),
        NodeType::Root => parse_node(node.first_child().unwrap()),
        NodeType::Element => parse_element_type(node),
        NodeType::PI => panic!(),
        NodeType::Comment => {
            dbg!(node.tag_name());
            panic!()
        }
    }
}
fn parse_document(text: &str) -> Result<MathNode, roxmltree::Error> {
    let sanitized = regexes::sanitize_xml(text);
    let xml = roxmltree::Document::parse(&sanitized)?;

    let parsed: MathNode = parse_node(xml.root());
    Ok(parsed)
}

#[cfg(test)]
mod test {
    use super::MathNode::*;
    use super::*;
    #[test]
    fn test_simple_parsing() {
        let test = r#"<math xmlns="http://www.w3.org/1998/Math/MathML">
                            <apply>
                          <plus/>
                      <ci> x </ci>
                      <ci> y </ci>
                    </apply></math>"#;
        let res = parse_document(test).unwrap();
        let exp = Root(vec![Apply(vec![
            Op(BuiltinOp::plus),
            Ci(vec![Text("x".to_owned())]),
            Ci(vec![Text("y".to_owned())]),
        ])]);
        assert_eq!(res, exp);
    }
    #[test]
    fn test_recursion() {
        let test = r#"<apply>
                      <plus/>
                      <apply>
                        <times/>
                        <ci> a </ci>
                        <ci> x </ci>
                      </apply>
                      <ci> b </ci>
                    </apply>"#;
        let res = parse_document(test).unwrap();
        let expect = Apply(vec![
            Op(BuiltinOp::plus),
            Apply(vec![
                Op(BuiltinOp::times),
                Ci(vec![Text("a".to_owned())]),
                Ci(vec![Text("x".to_owned())]),
            ]),
            Ci(vec![Text("b".to_owned())]),
        ]);
        assert_eq!(res, expect)
    }
    #[test]
    fn test_numbers() {
        let test = r#"
        <math xmlns="http://www.w3.org/1998/Math/MathML">
        <cn type="real"> 12345.7 </cn>
                    <cn type="integer"> 12345 </cn>
                    <cn type="integer" base="16"> AB3 </cn>
                    <cn type="rational"> 12342 <sep/> 2342342 </cn>
                    <cn type="complex-cartesian"> 12.3 <sep/> 5 </cn>
                    <cn type="complex-polar"> 2 <sep/> 3.1415 </cn>
                    <cn type="constant">  &tau; </cn>
                    </math>
                    "#;
        let parsed = parse_document(test).unwrap();
    }
}
