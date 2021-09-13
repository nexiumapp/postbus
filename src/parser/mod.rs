use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, tag_no_case};
use nom::character::complete::{alphanumeric1, satisfy};
use nom::combinator::{eof, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::command::{Command, Domain, Mailbox};

#[cfg(test)]
mod tests;

type NomResult<'a, T> = IResult<&'a str, T>;

/// Parse an SMTP command.
/// It automatically splits the commands into lines, so raw strings can be put in.
pub fn parse(input: &str) -> (Vec<(&str, Option<Command>)>, &str) {
    let mut result = Vec::new();
    let mut remaining = "";

    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        if lines.peek().is_none() && !input.ends_with('\n') {
            remaining = line;
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

/// Parse a data line.
/// This is not done with Nom.
/// The returning tuple contains:
/// - Boolean indicating if an end of data state was reached.
/// - String with the data, this is only complete if the boolean is true.
/// - Remaining string with input after the data end. This is only non-empty if the boolean is true.
pub fn parse_data_lines(input: &str) -> (bool, String, String) {
    let mut has_ended = false;
    let mut result = Vec::new();
    let mut remaining = Vec::new();

    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        if has_ended {
            remaining.push(line);
        }

        if line == "." {
            has_ended = true;
        }

        if line.starts_with(".") {
            result.push(&line[1..]);
        } else {
            result.push(line);
        }
    }

    (has_ended, result.join("\r\n"), remaining.join("\r\n"))
}

fn parse_command(input: &str) -> NomResult<Command> {
    alt((
        parse_ehlo, parse_helo, parse_mail, parse_rcpt, parse_data, parse_rset, parse_quit,
    ))(input)
}

fn parse_ehlo(input: &str) -> NomResult<Command> {
    let (rem, domain) = delimited(tag_no_case("EHLO "), parse_domain, eof)(input)?;

    Ok((rem, Command::EHLO(domain)))
}

fn parse_helo(input: &str) -> NomResult<Command> {
    let (rem, domain) = delimited(tag_no_case("HELO "), parse_domain, eof)(input)?;

    Ok((rem, Command::HELO(domain)))
}

fn parse_mail(input: &str) -> NomResult<Command> {
    let (rem, res) = tuple((tag_no_case("MAIL FROM:"), opt(tag(" ")), parse_path, eof))(input)?;
    let (_, _, mailbox, _) = res;

    Ok((rem, Command::FROM(mailbox)))
}

fn parse_rcpt(input: &str) -> NomResult<Command> {
    let (rem, res) = tuple((tag_no_case("RCPT TO:"), opt(tag(" ")), parse_path, eof))(input)?;
    let (_, _, mailbox, _) = res;

    Ok((rem, Command::RCPT(mailbox)))
}

fn parse_data(input: &str) -> NomResult<Command> {
    let (rem, _) = terminated(tag_no_case("DATA"), eof)(input)?;

    Ok((rem, Command::DATA))
}

fn parse_rset(input: &str) -> NomResult<Command> {
    let (rem, _) = terminated(tag_no_case("RSET"), eof)(input)?;

    Ok((rem, Command::RSET))
}

fn parse_quit(input: &str) -> NomResult<Command> {
    let (rem, _) = terminated(tag_no_case("QUIT"), eof)(input)?;

    Ok((rem, Command::QUIT))
}

fn parse_path(input: &str) -> NomResult<Mailbox> {
    delimited(tag("<"), parse_mailbox, tag(">"))(input)
}

fn parse_mailbox(input: &str) -> NomResult<Mailbox> {
    let (rem, res) = tuple((parse_localpart, tag("@"), parse_domain))(input)?;
    let (user, _, domain) = res;

    Ok((
        rem,
        Mailbox {
            domain,
            local: user.to_string(),
        },
    ))
}

fn parse_domain(input: &str) -> NomResult<Domain> {
    let (rem, res) = recognize(pair(
        parse_subdomain,
        many0(pair(tag("."), parse_subdomain)),
    ))(input)?;

    Ok((rem, res.into()))
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
