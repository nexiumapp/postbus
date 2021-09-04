use nom::branch::alt;
use nom::bytes::streaming::{is_a, tag, tag_no_case, take_while_m_n};
use nom::character::is_alphanumeric;
use nom::character::streaming::line_ending;
use nom::combinator::{opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

pub mod result;
#[cfg(test)]
mod tests;

use result::ParseCommand;

type NomResult<'a, T> = IResult<&'a str, T>;

/// Parse an SMTP command.
/// It automatically splits the commands into lines, so raw strings can be put in.
pub fn parse(input: &str) -> NomResult<ParseCommand> {
    alt((parse_ehlo, parse_helo, parse_mail, parse_data))(input)
}
fn parse_ehlo(input: &str) -> NomResult<ParseCommand> {
    let (remaining, domain) = delimited(tag_no_case("EHLO "), parse_domain, line_ending)(input)?;

    Ok((remaining, ParseCommand::EHLO(domain)))
}

fn parse_helo(input: &str) -> NomResult<ParseCommand> {
    let (remaining, domain) = delimited(tag_no_case("HELO "), parse_domain, line_ending)(input)?;

    Ok((remaining, ParseCommand::HELO(domain)))
}

fn parse_mail(input: &str) -> NomResult<ParseCommand> {
    let (rem, res) = tuple((
        tag_no_case("MAIL FROM:"),
        opt(tag(" ")),
        parse_path,
        line_ending,
    ))(input)?;
    let (_, _, mailbox, _) = res;

    Ok((rem, ParseCommand::FROM(mailbox)))
}

fn parse_data(input: &str) -> NomResult<ParseCommand> {
    let (rem, _) = tuple((tag_no_case("DATA"), line_ending))(input)?;

    Ok((rem, ParseCommand::DATA))
}

fn parse_path(input: &str) -> NomResult<result::MailboxParam> {
    delimited(tag("<"), parse_mailbox, tag(">"))(input)
}

fn parse_mailbox(input: &str) -> NomResult<result::MailboxParam> {
    let (rem, res) = tuple((parse_localpart, tag("@"), parse_domain))(input)?;
    let (user, _, domain) = res;

    Ok((rem, result::MailboxParam(user, domain)))
}

fn parse_domain(input: &str) -> NomResult<result::DomainParam> {
    let (rem, res) = recognize(pair(
        parse_subdomain,
        many0(pair(opt(tag(".")), parse_subdomain)),
    ))(input)?;

    Ok((rem, result::DomainParam(res)))
}

fn parse_subdomain(input: &str) -> NomResult<&str> {
    recognize(pair(parse_letdig, opt(parse_ldhstr)))(input)
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
    recognize(pair(parse_atom, many0(pair(opt(tag(".")), parse_atom))))(input)
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
