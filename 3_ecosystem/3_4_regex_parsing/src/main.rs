use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    println!("Implement me!");
}

trait Parser {
    fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>);
}

struct RegexParser;

impl Parser for RegexParser {
    fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"^(?:.?[<^>])?(?P<sign>[+-])?#?0?(?P<width>[1-9]\d*)?(?:\.(?:(?P<precision_ast>\*)|(?P<precision_int>[1-9]\d*)|(?P<precision_arg>[1-9]\d*)\$))?(?:\?|x\?|X\?|(?:\p{XID_Start}|_)\p{XID_Continue}*)?$",
            )
            .unwrap()
        });
        let caps = RE.captures(input).unwrap();

        let sign = caps.name("sign").map(|sign| {
            if sign.as_str() == "+" {
                Sign::Plus
            } else {
                Sign::Minus
            }
        });

        let width = caps
            .name("width")
            .map(|width| width.as_str().parse().unwrap());

        #[allow(clippy::manual_map)]
        let precision = if caps.name("precision_ast").is_some() {
            Some(Precision::Asterisk)
        } else if let Some(precision_int) = caps.name("precision_int") {
            Some(Precision::Integer(precision_int.as_str().parse().unwrap()))
        } else if let Some(precision_arg) = caps.name("precision_arg") {
            Some(Precision::Argument(precision_arg.as_str().parse().unwrap()))
        } else {
            None
        };

        (sign, width, precision)
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
        for (input, expected) in [
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
        for (input, expected) in [
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
        for (input, expected) in [
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
