use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::streaming::{is_a, tag, tag_no_case, take_while_m_n};
use nom::character::is_alphanumeric;
use nom::character::streaming::line_ending;
use nom::combinator::{opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

type NomResult<'a, T> = IResult<&'a str, T>;

/// Result of a successful call to the `.parse()` function.
#[derive(Debug, PartialEq)]
pub enum ParseResult<'a> {
    EHLO(&'a str),
    HELO(&'a str),
    Mailbox(&'a str, &'a str),
}

impl<'a> Display for ParseResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseResult::EHLO(ident) => writeln!(f, "EHLO {:?}", ident),
            ParseResult::HELO(ident) => writeln!(f, "HELO {:?}", ident),
            ParseResult::Mailbox(user, domain) => writeln!(f, "{}@{}", user, domain),
        }
    }
}

/// Parse an SMTP command.
/// It automatically splits the commands into lines, so raw strings can be put in.
pub fn parse(input: &str) -> NomResult<ParseResult> {
    alt((parse_ehlo, parse_helo))(input)
}
fn parse_ehlo(input: &str) -> NomResult<ParseResult> {
    let (remaining, domain) = delimited(tag_no_case("EHLO "), parse_domain, line_ending)(input)?;

    Ok((remaining, ParseResult::EHLO(domain)))
}

fn parse_helo(input: &str) -> NomResult<ParseResult> {
    let (remaining, domain) = delimited(tag_no_case("HELO "), parse_domain, line_ending)(input)?;

    Ok((remaining, ParseResult::HELO(domain)))
}

fn parse_mailbox(input: &str) -> NomResult<ParseResult> {
    let (rem, res) = tuple((parse_localpart, tag("@"), parse_domain))(input)?;
    let (user, _, domain) = res;

    Ok((rem, ParseResult::Mailbox(user, domain)))
}

fn parse_domain(input: &str) -> NomResult<&str> {
    recognize(pair(
        parse_subdomain,
        many0(pair(opt(tag(".")), parse_subdomain)),
    ))(input)
}

fn parse_subdomain(input: &str) -> NomResult<&str> {
    recognize(pair(parse_letdig, many0(parse_ldhstr)))(input)
}

fn parse_letdig(input: &str) -> NomResult<&str> {
    take_while_m_n(1, 1, |c| is_alphanumeric(c as u8))(input)
}

fn parse_ldhstr(input: &str) -> NomResult<&str> {
    recognize(pair(
        many0(take_while_m_n(1, 1, |c| {
            is_alphanumeric(c as u8) || c == '-'
        })),
        parse_letdig,
    ))(input)
}

fn parse_localpart(input: &str) -> NomResult<&str> {
    alt((parse_dot_string, parse_quoted_string))(input)
}

fn parse_dot_string(input: &str) -> NomResult<&str> {
    recognize(pair(parse_atext, many0(pair(opt(tag(".")), parse_atext))))(input)
}

fn parse_quoted_string(input: &str) -> NomResult<&str> {
    delimited(tag("\""), recognize(many1(parse_qcontent_smtp)), tag("\""))(input)
}

fn parse_qcontent_smtp(input: &str) -> NomResult<&str> {
    alt((parse_qtext_smtp, parse_quotedpair_smtp))(input)
}

fn parse_qtext_smtp(input: &str) -> NomResult<&str> {
    take_while_m_n(1, 1, |c| {
        let val = c as u8;

        (val >= 32 && val <= 33) || (val >= 35 && val <= 91) || (val >= 93 && val <= 126)
    })(input)
}

fn parse_quotedpair_smtp(input: &str) -> NomResult<&str> {
    preceded(
        tag("\\"),
        take_while_m_n(1, 1, |c| {
            let val = c as u8;

            val >= 32 && val <= 126
        }),
    )(input)
}

fn parse_atom(input: &str) -> NomResult<&str> {
    recognize(many1(parse_atext))(input)
}

fn parse_atext(input: &str) -> NomResult<&str> {
    alt((
        take_while_m_n(1, 1, |c| is_alphanumeric(c as u8)),
        is_a("!#$%&'*+-/=?^_`{|}~"),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::error::ParseError;

    use super::*;

    #[test]
    fn parse_ehlo() {
        let (rem, cmd) = parse("EHLO nexium.app\r\n").unwrap();

        assert_eq!(ParseResult::EHLO("nexium.app"), cmd);
        assert_eq!("", rem);
    }

    #[test]
    fn parse_ehlo_empty_with_space() {
        let err = parse("EHLO \r\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "EHLO \r\n",
                nom::error::ErrorKind::Tag
            )),
            err
        );
    }

    #[test]
    fn parse_ehlo_no_domain() {
        let err = parse("EHLO\r\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "EHLO\r\n",
                nom::error::ErrorKind::Tag
            )),
            err
        );
    }

    #[test]
    fn parse_helo() {
        let (rem, cmd) = parse("HELO nexium.app\r\n").unwrap();

        assert_eq!(ParseResult::HELO("nexium.app"), cmd);
        assert_eq!("", rem);
    }

    #[test]
    fn parse_helo_empty_with_space() {
        let err = parse("HELO \r\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "\r\n",
                nom::error::ErrorKind::TakeWhileMN
            )),
            err
        );
    }

    #[test]
    fn parse_helo_no_domain() {
        let err = parse("HELO\r\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "HELO\r\n",
                nom::error::ErrorKind::Tag
            )),
            err
        );
    }

    #[test]
    fn parse_mailbox_simple() {
        let (rem, res) = parse_mailbox("postbus@nexium.app\n").unwrap();

        assert_eq!(ParseResult::Mailbox("postbus", "nexium.app"), res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_mailbox_quoted() {
        let (rem, res) = parse_mailbox("\"john\"@nexium.app\n").unwrap();

        assert_eq!(ParseResult::Mailbox("john", "nexium.app"), res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_mailbox_numbered() {
        let (rem, res) = parse_mailbox("1234567890@nexium.app\n").unwrap();

        assert_eq!(ParseResult::Mailbox("1234567890", "nexium.app"), res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_mailbox_nodomain() {
        let err = parse_mailbox("apples@\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "\n",
                nom::error::ErrorKind::TakeWhileMN
            )),
            err
        );
    }

    #[test]
    fn parse_mailbox_nouser() {
        let err = parse_mailbox("@nexium.app\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "@nexium.app\n",
                nom::error::ErrorKind::Tag
            )),
            err
        );
    }

    #[test]
    fn parse_domain_normal() {
        let (rem, res) = parse_domain("nexium.app\n").unwrap();

        assert_eq!("nexium.app", res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_domain_nested() {
        let (rem, res) = parse_domain("very.deep.nesting.nexium.app\n").unwrap();

        assert_eq!("very.deep.nesting.nexium.app", res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_domain_firstdot() {
        let err = parse_domain(".nexium.app\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                ".nexium.app\n",
                nom::error::ErrorKind::TakeWhileMN
            )),
            err
        );
    }

    #[test]
    fn parse_domain_lastdot() {
        let (rem, res) = parse_domain("nexium.app.\n").unwrap();

        assert_eq!("nexium.app", res);
        assert_eq!(".\n", rem);
    }

    #[test]
    fn parse_dotstring_alpha() {
        let (rem, res) = parse_dot_string("hello ").unwrap();

        assert_eq!("hello", res);
        assert_eq!(" ", rem);
    }

    #[test]
    fn parse_dotstring_spaced() {
        let (rem, res) = parse_dot_string("hello world").unwrap();

        assert_eq!("hello", res);
        assert_eq!(" world", rem);
    }

    #[test]
    fn parse_dotstring_dotted() {
        let (rem, res) = parse_dot_string("h.e.l.l.o w.o.r.l.d").unwrap();

        assert_eq!("h.e.l.l.o", res);
        assert_eq!(" w.o.r.l.d", rem);
    }

    #[test]
    fn parse_dotstring_first_dot() {
        let err = parse_dot_string(".hello").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                ".hello",
                nom::error::ErrorKind::IsA
            )),
            err
        );
    }

    #[test]
    fn parse_dotstring_specials() {
        let (rem, res) = parse_dot_string("!#$%&'*+-/=?^_`{|}~.1234 ").unwrap();

        assert_eq!("!#$%&'*+-/=?^_`{|}~.1234", res);
        assert_eq!(" ", rem);
    }

    #[test]
    fn parse_atom_alpha() {
        let (rem, res) = parse_atom("hello ").unwrap();

        assert_eq!("hello", res);
        assert_eq!(" ", rem);
    }

    #[test]
    fn parse_atom_spaced() {
        let (rem, res) = parse_atom("hello world").unwrap();

        assert_eq!("hello", res);
        assert_eq!(" world", rem);
    }

    #[test]
    fn parse_atom_dotted() {
        let (rem, res) = parse_atom("h.e.l.l.o w.o.r.l.d").unwrap();

        assert_eq!("h", res);
        assert_eq!(".e.l.l.o w.o.r.l.d", rem);
    }

    #[test]
    fn parse_atom_first_dot() {
        let err = parse_atom(".hello").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                ".hello",
                nom::error::ErrorKind::IsA
            )),
            err
        );
    }

    #[test]
    fn parse_atom_specials() {
        let (rem, res) = parse_atom("!#$%&'*+-/=?^_`{|}~. ").unwrap();

        assert_eq!("!#$%&'*+-/=?^_`{|}~", res);
        assert_eq!(". ", rem);
    }

    #[test]
    fn parse_quoted_normal() {
        let (rem, res) = parse_quoted_string("\"some.thing\"\n").unwrap();

        assert_eq!("some.thing", res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_quoted_unquoted() {
        let err = parse_quoted_string("some.thing\n").unwrap_err();

        assert_eq!(
            nom::Err::Error(nom::error::Error::from_error_kind(
                "some.thing\n",
                nom::error::ErrorKind::Tag
            )),
            err
        );
    }

    #[test]
    fn parse_quoted_first_dot() {
        let (rem, res) = parse_quoted_string("\".some.thing\"\n").unwrap();

        assert_eq!(".some.thing", res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_quoted_last_dot() {
        let (rem, res) = parse_quoted_string("\"some.thing.\"\n").unwrap();

        assert_eq!("some.thing.", res);
        assert_eq!("\n", rem);
    }

    #[test]
    fn parse_localpart_normal() {
        let (rem, res) = parse_localpart("this.matches but this does not").unwrap();

        assert_eq!("this.matches", res);
        assert_eq!(" but this does not", rem);
    }

    #[test]
    fn parse_localpart_quoted() {
        let (rem, res) = parse_localpart("\"this.matches\" but this does not").unwrap();

        assert_eq!("this.matches", res);
        assert_eq!(" but this does not", rem);
    }
}
