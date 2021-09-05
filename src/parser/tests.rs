use nom::error::ParseError;

use super::*;

#[test]
fn parse_single() {
    let (cmds, rem) = parse("EHLO nexium.app\r\n");

    assert_eq!(1, cmds.len());
    assert!(rem.is_none());

    let first = cmds.get(0).unwrap();
    let parsed = first.1.as_ref().unwrap();
    assert_eq!(
        result::ParseCommand::EHLO(result::DomainParam("nexium.app")),
        *parsed
    );
}

#[test]
fn parse_multiple() {
    let (cmds, rem) = parse("EHLO nexium.app\r\nMAIL FROM:<info@nexium.app>\r\n");

    assert_eq!(2, cmds.len());
    assert!(rem.is_none());

    let first = cmds.get(0).unwrap();
    let parsed = first.1.as_ref().unwrap();

    assert_eq!(
        result::ParseCommand::EHLO(result::DomainParam("nexium.app")),
        *parsed
    );

    let second = cmds.get(1).unwrap();
    let parsed = second.1.as_ref().unwrap();
    assert_eq!(
        result::ParseCommand::FROM(result::MailboxParam(
            "info",
            result::DomainParam("nexium.app")
        )),
        *parsed
    );
}

#[test]
fn parse_unfinished() {
    let (cmds, rem) = parse("EHLO nexium.app\r\nMAIL FR");

    assert_eq!(1, cmds.len());
    assert!(rem.is_some());
    assert_eq!("MAIL FR", rem.unwrap());

    let first = cmds.get(0).unwrap();
    let parsed = first.1.as_ref().unwrap();
    assert_eq!(
        result::ParseCommand::EHLO(result::DomainParam("nexium.app")),
        *parsed
    );
}

#[test]
fn parse_empty() {
    let (cmds, rem) = parse("");

    assert_eq!(0, cmds.len());
    assert!(rem.is_none());
}

#[test]
fn parse_invalid_valid() {
    let (cmds, rem) = parse("THIS IS AN ERROR\r\nMAIL FROM:<info@nexium.app>\r\nRC");

    assert_eq!(2, cmds.len());
    assert!(rem.is_some());
    assert_eq!("RC", rem.unwrap());

    let first = cmds.get(0).unwrap();
    assert_eq!("THIS IS AN ERROR", first.0);
    assert!(first.1.as_ref().is_none());

    let second = cmds.get(1).unwrap();
    let parsed = second.1.as_ref().unwrap();
    assert_eq!(
        result::ParseCommand::FROM(result::MailboxParam(
            "info",
            result::DomainParam("nexium.app")
        )),
        *parsed
    );
}

#[test]
fn parse_command_partial_simple() {
    let err = parse_command("MAIL FR").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "MAIL FR",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_command_partial_input() {
    let err = parse_command("MAIL FROM:<").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "MAIL FROM:<",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_command_ehlo_simple() {
    let (rem, cmd) = parse_command("EHLO nexium.app").unwrap();

    assert_eq!(
        result::ParseCommand::EHLO(result::DomainParam("nexium.app")),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_command_ehlo_empty_with_space() {
    let err = parse_command("EHLO ").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "EHLO ",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_command_helo_simple() {
    let (rem, cmd) = parse_command("HELO nexium.app").unwrap();

    assert_eq!(
        result::ParseCommand::HELO(result::DomainParam("nexium.app")),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_command_helo_empty_with_space() {
    let err = parse_command("HELO ").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "HELO ",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_command_from_simple() {
    let (rem, cmd) = parse_command("MAIL FROM:<hello@nexium.app>").unwrap();

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
fn parse_command_from_space() {
    let (rem, cmd) = parse_command("MAIL FROM: <hello@nexium.app>").unwrap();

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
fn parse_command_from_nobracket() {
    let err = parse_command("MAIL FROM:hello@nexium.app").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "MAIL FROM:hello@nexium.app",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_command_rcpt_simple() {
    let (rem, cmd) = parse_command("RCPT TO:<sendme@nexium.app>").unwrap();

    assert_eq!(
        result::ParseCommand::RCPT(result::MailboxParam(
            "sendme",
            result::DomainParam("nexium.app")
        )),
        cmd
    );
    assert_eq!("", rem);
}

#[test]
fn parse_command_data_simple() {
    let (rem, cmd) = parse_command("DATA").unwrap();

    assert_eq!(result::ParseCommand::DATA, cmd);
    assert_eq!("", rem);
}

#[test]
fn parse_command_reset_simple() {
    let (rem, cmd) = parse_command("RSET").unwrap();

    assert_eq!(result::ParseCommand::RSET, cmd);
    assert_eq!("", rem);
}

#[test]
fn parse_command_quit_simple() {
    let (rem, cmd) = parse_command("QUIT").unwrap();

    assert_eq!(result::ParseCommand::QUIT, cmd);
    assert_eq!("", rem);
}

#[test]
fn parse_mailbox_simple() {
    let (rem, res) = parse_mailbox("postbus@nexium.app ").unwrap();

    assert_eq!(
        result::MailboxParam("postbus", result::DomainParam("nexium.app")),
        res
    );
    assert_eq!(" ", rem);
}

#[test]
fn parse_mailbox_quoted() {
    let (rem, res) = parse_mailbox("\"john\"@nexium.app").unwrap();

    assert_eq!(
        result::MailboxParam("john", result::DomainParam("nexium.app")),
        res
    );
    assert_eq!("", rem);
}

#[test]
fn parse_mailbox_numbered() {
    let (rem, res) = parse_mailbox("1234567890@nexium.app").unwrap();

    assert_eq!(
        result::MailboxParam("1234567890", result::DomainParam("nexium.app")),
        res
    );
    assert_eq!("", rem);
}

#[test]
fn parse_mailbox_nodomain() {
    let err = parse_mailbox("apples@").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "",
            nom::error::ErrorKind::AlphaNumeric
        )),
        err
    );
}

#[test]
fn parse_mailbox_nouser() {
    let err = parse_mailbox("@nexium.app").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "@nexium.app",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_domain_normal() {
    let (rem, res) = parse_domain("nexium.app").unwrap();

    assert_eq!(result::DomainParam("nexium.app"), res);
    assert_eq!("", rem);
}

#[test]
fn parse_domain_nested() {
    let (rem, res) = parse_domain("very.deep.nesting.nexium.app").unwrap();

    assert_eq!(result::DomainParam("very.deep.nesting.nexium.app"), res);
    assert_eq!("", rem);
}

#[test]
fn parse_domain_firstdot() {
    let err = parse_domain(".nexium.app").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            ".nexium.app",
            nom::error::ErrorKind::AlphaNumeric
        )),
        err
    );
}

#[test]
fn parse_domain_lastdot() {
    let (rem, res) = parse_domain("nexium.app.").unwrap();

    assert_eq!(result::DomainParam("nexium.app"), res);
    assert_eq!(".", rem);
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
    let (rem, res) = parse_quoted_string("\"some.thing\"").unwrap();

    assert_eq!("some.thing", res);
    assert_eq!("", rem);
}

#[test]
fn parse_quoted_unquoted() {
    let err = parse_quoted_string("some.thing").unwrap_err();

    assert_eq!(
        nom::Err::Error(nom::error::Error::from_error_kind(
            "some.thing",
            nom::error::ErrorKind::Tag
        )),
        err
    );
}

#[test]
fn parse_quoted_first_dot() {
    let (rem, res) = parse_quoted_string("\".some.thing\"").unwrap();

    assert_eq!(".some.thing", res);
    assert_eq!("", rem);
}

#[test]
fn parse_quoted_last_dot() {
    let (rem, res) = parse_quoted_string("\"some.thing.\"").unwrap();

    assert_eq!("some.thing.", res);
    assert_eq!("", rem);
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

#[test]
fn parse_quotepair_escape() {
    let (rem, res) = parse_quotedpair_smtp("\\ ").unwrap();

    assert_eq!(" ", res);
    assert_eq!("", rem);
}
