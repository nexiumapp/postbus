use nom::error::ParseError;

use super::*;

#[test]
fn parse_ehlo_simple() {
    let (rem, cmd) = parse("EHLO nexium.app\r\n").unwrap();

    assert_eq!(
        result::ParseCommand::EHLO(result::DomainParam("nexium.app")),
        cmd
    );
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
fn parse_helo_simple() {
    let (rem, cmd) = parse("HELO nexium.app\r\n").unwrap();

    assert_eq!(
        result::ParseCommand::HELO(result::DomainParam("nexium.app")),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_helo_empty_with_space() {
    let err = parse("HELO \r\n").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "HELO \r\n",
            nom::error::ErrorKind::Tag
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
fn parse_from_simple() {
    let (rem, cmd) = parse("MAIL FROM:<hello@nexium.app>\r\n").unwrap();

    assert_eq!(
        result::ParseCommand::FROM(result::MailboxParam(
            "hello",
            result::DomainParam("nexium.app")
        )),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_from_space() {
    let (rem, cmd) = parse("MAIL FROM: <hello@nexium.app>\r\n").unwrap();

    assert_eq!(
        result::ParseCommand::FROM(result::MailboxParam(
            "hello",
            result::DomainParam("nexium.app")
        )),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_from_nobracket() {
    let err = parse("MAIL FROM:hello@nexium.app\r\n").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "MAIL FROM:hello@nexium.app\r\n",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_data_simple() {
    let (rem, cmd) = parse("DATA\r\n").unwrap();

    assert_eq!(result::ParseCommand::DATA, cmd);
    assert_eq!("", rem);
}

#[test]
fn parse_mailbox_simple() {
    let (rem, res) = parse_mailbox("postbus@nexium.app\n").unwrap();

    assert_eq!(
        result::MailboxParam("postbus", result::DomainParam("nexium.app")),
        res
    );
    assert_eq!("\n", rem);
}

#[test]
fn parse_mailbox_quoted() {
    let (rem, res) = parse_mailbox("\"john\"@nexium.app\n").unwrap();

    assert_eq!(
        result::MailboxParam("john", result::DomainParam("nexium.app")),
        res
    );
    assert_eq!("\n", rem);
}

#[test]
fn parse_mailbox_numbered() {
    let (rem, res) = parse_mailbox("1234567890@nexium.app\n").unwrap();

    assert_eq!(
        result::MailboxParam("1234567890", result::DomainParam("nexium.app")),
        res
    );
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

    assert_eq!(result::DomainParam("nexium.app"), res);
    assert_eq!("\n", rem);
}

#[test]
fn parse_domain_nested() {
    let (rem, res) = parse_domain("very.deep.nesting.nexium.app\n").unwrap();

    assert_eq!(result::DomainParam("very.deep.nesting.nexium.app"), res);
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

    assert_eq!(result::DomainParam("nexium.app"), res);
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
