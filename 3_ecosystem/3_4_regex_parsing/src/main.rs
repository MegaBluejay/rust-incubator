use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, anychar, char, digit0, one_of},
    combinator::{all_consuming, map, map_res, opt, recognize, value},
    multi::many0_count,
    sequence::{pair, preceded, terminated},
    IResult,
};
use once_cell::sync::Lazy;
use regex::Regex;

fn main() {}

type Parsed = (Option<Sign>, Option<usize>, Option<Precision>);

trait Parser {
    fn parse(input: &str) -> Parsed;
}

struct RegexParser;

impl Parser for RegexParser {
    fn parse(input: &str) -> Parsed {
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

struct NomParser;

fn sign(input: &str) -> IResult<&str, Sign> {
    alt((value(Sign::Plus, char('+')), value(Sign::Minus, char('-'))))(input)
}

fn integer(input: &str) -> IResult<&str, usize> {
    map_res(
        recognize(preceded(one_of("123456789"), digit0)),
        |i: &str| i.parse(),
    )(input)
}

fn precision(input: &str) -> IResult<&str, Precision> {
    alt((
        value(Precision::Asterisk, char('*')),
        map(terminated(integer, char('$')), Precision::Argument),
        map(integer, Precision::Integer),
    ))(input)
}

fn ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn align(input: &str) -> IResult<&str, char> {
    one_of("<^>")(input)
}

fn format_spec(input: &str) -> IResult<&str, Parsed> {
    let (input, _) = opt(alt((preceded(anychar, align), align)))(input)?;
    dbg!(input);
    let (input, sign) = opt(sign)(input)?;
    dbg!(input);
    let (input, _) = opt(char('#'))(input)?;
    dbg!(input);
    let (input, _) = opt(char('0'))(input)?;
    dbg!(input);
    let (input, width) = opt(integer)(input)?;
    dbg!(input);
    let (input, precision) = opt(preceded(char('.'), precision))(input)?;
    dbg!(input);
    let (input, _) = opt(alt((tag("?"), tag("x?"), tag("X?"), ident)))(input)?;
    dbg!(input);
    Ok((input, (sign, width, precision)))
}

impl Parser for NomParser {
    fn parse(input: &str) -> Parsed {
        all_consuming(format_spec)(input).unwrap().1
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Precision {
    Integer(usize),
    Argument(usize),
    Asterisk,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        let mut p = all_consuming(sign);
        assert_eq!(p("+").unwrap().1, Sign::Plus);
        assert_eq!(p("-").unwrap().1, Sign::Minus);
    }

    #[test]
    fn test_integer() {
        assert_eq!(all_consuming(integer)("8").unwrap().1, 8);
    }

    #[test]
    fn test_precision() {
        let mut p = all_consuming(precision);
        assert_eq!(p("*").unwrap().1, Precision::Asterisk);
        assert_eq!(p("4$").unwrap().1, Precision::Argument(4));
        assert_eq!(p("5").unwrap().1, Precision::Integer(5));
    }

    #[test]
    fn test_align() {
        let mut p = all_consuming(align);
        assert_eq!(p(">").unwrap().1, '>');
    }

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
                println!("{}", input);
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
                println!("{}", input);
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
                println!("{}", input);
                let (_, _, precision) = <P>::parse(input);
                assert_eq!(precision, expected);
            }
        }

        #[instantiate_tests(<RegexParser>)]
        mod regex {}

        #[instantiate_tests(<NomParser>)]
        mod nom {}
    }
}
