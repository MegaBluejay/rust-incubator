fn main() {
    println!("Implement me!");
}

trait Parser {
    fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>);
}

struct RegexParser;

impl Parser for RegexParser {
    fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
enum Precision {
    Integer(usize),
    Argument(usize),
    Asterisk,
}

#[cfg(test)]
#[generic_tests::define]
mod spec {
    use super::*;

    #[test]
    fn parses_sign<P: Parser>() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", None),
            (">+8.*", Some(Sign::Plus)),
            ("-.1$x", Some(Sign::Minus)),
            ("a^#043.8?", None),
        ] {
            let (sign, ..) = <P>::parse(input);
            assert_eq!(sign, expected);
        }
    }

    #[test]
    fn parses_width<P: Parser>() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", Some(8)),
            (">+8.*", Some(8)),
            ("-.1$x", None),
            ("a^#043.8?", Some(43)),
        ] {
            let (_, width, _) = <P>::parse(input);
            assert_eq!(width, expected);
        }
    }

    #[test]
    fn parses_precision<P: Parser>() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", Some(Precision::Asterisk)),
            (">+8.*", Some(Precision::Asterisk)),
            ("-.1$x", Some(Precision::Argument(1))),
            ("a^#043.8?", Some(Precision::Integer(8))),
        ] {
            let (_, _, precision) = <P>::parse(input);
            assert_eq!(precision, expected);
        }
    }

    #[instantiate_tests(<RegexParser>)]
    mod regex {}
}
