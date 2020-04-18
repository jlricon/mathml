/// We need to apply some replacements to account for things like
/// https://www.tutorialspoint.com/mathml/mathml_greek_letters.htm
/// Or else the xml parser fails :(
/// Replaces &$STRING; with $STRING

macro_rules! replace {
    ($($e:ident),*) => {{
    let mut temp_vec = Vec::new();
            $(
               temp_vec.push(replace_one(format!("&{};", stringify!($e)), stringify!($e)));
            )*
            move |x| temp_vec.iter().fold(x,|acc,next|next(acc))
           }
    }

}
fn replace_one(from: String, to: &'static str) -> impl Fn(String) -> String + 'static {
    move |x: String| x.replace(&from, to)
}

pub fn sanitize_xml(x: &str) -> String {
    let replacer = replace! {tau};
    replacer(x.to_owned())
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_replace() {
        let test = r#"<cn type="constant">  &tau;&bla; </cn>"#;
        let replacer = replace! {tau,bla};
        let output = replacer(test.to_owned());
        assert_eq!(output, test.replace("&tau;", "tau").replace("&bla;", "bla"))
    }
}
