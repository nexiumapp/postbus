use std::fmt::Display;

// All command types.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Command {
    HELO(Domain),
    EHLO(Domain),
    RCPT(Mailbox),
    FROM(Mailbox),
    DATA,
    RSET,
    QUIT,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::EHLO(ident) => writeln!(f, "EHLO {:?}", ident),
            Command::HELO(ident) => writeln!(f, "HELO {:?}", ident),
            Command::FROM(Mailbox {
                local,
                domain: Domain(domain),
            }) => {
                writeln!(f, "MAIL FROM: {}@{}", local, domain)
            }
            Command::RCPT(Mailbox {
                local,
                domain: Domain(domain),
            }) => {
                writeln!(f, "RCPT TO: {}@{}", local, domain)
            }
            Command::DATA => writeln!(f, "DATA"),
            Command::RSET => writeln!(f, "RSET"),
            Command::QUIT => writeln!(f, "QUIT"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Domain(pub String);
#[derive(Debug, PartialEq, Clone)]
pub struct Mailbox {
    pub local: String,
    pub domain: Domain,
}

impl From<&str> for Domain {
    fn from(input: &str) -> Self {
        Domain(input.to_string())
    }
}
