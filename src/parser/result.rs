use std::fmt::Display;

/// Result of a successful call to the `.parse()` function.
#[derive(Debug, PartialEq)]
pub enum ParseCommand<'a> {
    EHLO(DomainParam<'a>),
    HELO(DomainParam<'a>),
    FROM(MailboxParam<'a>),
    RCPT(MailboxParam<'a>),
    DATA,
    QUIT,
}

impl<'a> Display for ParseCommand<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseCommand::EHLO(ident) => writeln!(f, "EHLO {:?}", ident),
            ParseCommand::HELO(ident) => writeln!(f, "HELO {:?}", ident),
            ParseCommand::FROM(MailboxParam(user, DomainParam(domain))) => {
                writeln!(f, "MAIL FROM: {}@{}", user, domain)
            }
            ParseCommand::RCPT(MailboxParam(user, DomainParam(domain))) => {
                writeln!(f, "RCPT TO: {}@{}", user, domain)
            }
            ParseCommand::DATA => writeln!(f, "DATA"),
            ParseCommand::QUIT => writeln!(f, "QUIT"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DomainParam<'a>(pub &'a str);
#[derive(Debug, PartialEq)]
pub struct MailboxParam<'a>(pub &'a str, pub DomainParam<'a>);

/// Parameters used in ParseCommand.
#[derive(Debug, PartialEq)]
pub enum ParseParam<'a> {
    Domain(DomainParam<'a>),
    Mailbox(MailboxParam<'a>),
}

impl<'a> Display for ParseParam<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseParam::Domain(DomainParam(domain)) => writeln!(f, "{}", domain),
            ParseParam::Mailbox(MailboxParam(user, DomainParam(domain))) => {
                writeln!(f, "{}@{}", user, domain)
            }
        }
    }
}
