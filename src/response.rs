/// All responses possible from the server.
#[derive(Debug, PartialEq)]
pub enum Response {
    Goodbye,
    Ok,
    StartData,
    TooManyRecipients,
    SyntaxError,
    OutOfSequence,
    NotImplemented,
    RecipientNotLocal,
    InvalidRecipient,
    TransactionFailed,
    Greeting(String),
    Helo(String),
    Ehlo(String),
}

impl Response {
    pub fn to_response(&self) -> String {
        match self {
            Response::Goodbye => "221 Goodbye!\r\n".into(),
            Response::Ok => "250 Ok\r\n".into(),
            Response::StartData => "354 Go ahead\r\n".into(),
            Response::TooManyRecipients => "452 Too many recipients\r\n".into(),
            Response::SyntaxError => "500 Syntax error\r\n".into(),
            Response::OutOfSequence => "503 Command out of sequence\r\n".into(),
            Response::NotImplemented => "504 Command not implemented\r\n".into(),
            Response::RecipientNotLocal => "550 User not local\r\n".into(),
            Response::InvalidRecipient => "554 No valid recipient\r\n".into(),
            Response::TransactionFailed => "554 Transaction failed\r\n".into(),

            Response::Greeting(name) => format!("220 {} ESMTP\r\n", name),
            Response::Helo(name) => format!("250 {} ESMTP\r\n", name),
            Response::Ehlo(name) => format!("250 {} ESMTP\r\n", name),
        }
    }
}
