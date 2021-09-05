use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, tag_no_case};
use nom::character::complete::{alphanumeric1, satisfy};
use nom::combinator::{eof, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

pub mod result;
#[cfg(test)]
mod tests;

use result::ParseCommand;

type NomResult<'a, T> = IResult<&'a str, T>;

/// Parse an SMTP command.
/// It automatically splits the commands into lines, so raw strings can be put in.
pub fn parse(input: &str) -> (Vec<(&str, Option<ParseCommand>)>, Option<&str>) {
    let mut result = Vec::new();
    let mut remaining = None;

    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        if lines.peek().is_none() && !input.ends_with('\n') {
            remaining = Some(line);
            break;
        }

        match parse_command(line) {
            Ok((rem, cmd)) => {
                if rem != "" {
                    result.push((line, None));
                    continue;
                }

                result.push((line, Some(cmd)));
            }
            Err(_) => result.push((line, None)),
        };
    }

    (result, remaining)
}

fn parse_command(input: &str) -> NomResult<ParseCommand> {
    alt((
        parse_ehlo, parse_helo, parse_mail, parse_rcpt, parse_data, parse_rset, parse_quit,
    ))(input)
}

fn parse_ehlo(input: &str) -> NomResult<ParseCommand> {
    let (rem, domain) = delimited(tag_no_case("EHLO "), parse_domain, eof)(input)?;

    Ok((rem, ParseCommand::EHLO(domain)))
}

fn parse_helo(input: &str) -> NomResult<ParseCommand> {
    let (rem, domain) = delimited(tag_no_case("HELO "), parse_domain, eof)(input)?;

    Ok((rem, ParseCommand::HELO(domain)))
}

fn parse_mail(input: &str) -> NomResult<ParseCommand> {
    let (rem, res) = tuple((tag_no_case("MAIL FROM:"), opt(tag(" ")), parse_path, eof))(input)?;
    let (_, _, mailbox, _) = res;

    Ok((rem, ParseCommand::FROM(mailbox)))
}

fn parse_rcpt(input: &str) -> NomResult<ParseCommand> {
    let (rem, res) = tuple((tag_no_case("RCPT TO:"), opt(tag(" ")), parse_path, eof))(input)?;
    let (_, _, mailbox, _) = res;

    Ok((rem, ParseCommand::RCPT(mailbox)))
}

fn parse_data(input: &str) -> NomResult<ParseCommand> {
    let (rem, _) = terminated(tag_no_case("DATA"), eof)(input)?;

    Ok((rem, ParseCommand::DATA))
}

fn parse_rset(input: &str) -> NomResult<ParseCommand> {
    let (rem, _) = terminated(tag_no_case("RSET"), eof)(input)?;

    Ok((rem, ParseCommand::RSET))
}

fn parse_quit(input: &str) -> NomResult<ParseCommand> {
    let (rem, _) = terminated(tag_no_case("QUIT"), eof)(input)?;

    Ok((rem, ParseCommand::QUIT))
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
        many0(pair(tag("."), parse_subdomain)),
    ))(input)?;

    Ok((rem, result::DomainParam(res)))
}

fn parse_subdomain(input: &str) -> NomResult<&str> {
    recognize(pair(alphanumeric1, many0(pair(tag("-"), alphanumeric1))))(input)
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
    recognize(satisfy(|c| {
        let val = c as u8;

        (val >= 32 && val <= 33) || (val >= 35 && val <= 91) || (val >= 93 && val <= 126)
    }))(input)
}

fn parse_quotedpair_smtp(input: &str) -> NomResult<&str> {
    preceded(
        tag("\\"),
        recognize(satisfy(|c| {
            let val = c as u8;

            val >= 32 && val <= 126
        })),
    )(input)
}

fn parse_atom(input: &str) -> NomResult<&str> {
    recognize(many1(parse_atext))(input)
}

fn parse_atext(input: &str) -> NomResult<&str> {
    alt((
        recognize(satisfy(|c| {
            let val = c as u8;

            (val >= 48 && val <= 57) || (val >= 65 && val <= 90) || (val >= 97 && val <= 122)
        })),
        is_a("!#$%&'*+-/=?^_`{|}~"),
    ))(input)
}
