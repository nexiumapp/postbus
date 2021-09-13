use std::{io::ErrorKind, net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;

use crate::{
    command::{Command, Domain, Mailbox},
    Handler, Response,
};

/// Struct holding data about the session.
pub struct SmtpSession {
    stream: TcpStream,
    server_name: String,
    remaining: String,
    addr: SocketAddr,
    handler: Arc<dyn Handler>,
    state: SmtpState,
}

/// Struct holding the current state of an transaction.
#[derive(Debug)]
pub struct SmtpState {
    pub receiving_data: bool,
    pub domain: Option<Domain>,
    pub from: Option<Mailbox>,
    pub recipients: Vec<Mailbox>,
    pub data: String,
}

impl Default for SmtpState {
    fn default() -> Self {
        Self {
            receiving_data: false,
            domain: None,
            from: None,
            recipients: Vec::new(),
            data: String::new(),
        }
    }
}

impl SmtpSession {
    /// Create a new session.
    pub(crate) fn new(
        stream: TcpStream,
        addr: SocketAddr,
        server_name: String,
        handler: Arc<dyn Handler>,
    ) -> Self {
        SmtpSession {
            stream,
            server_name,
            handler,
            addr,
            remaining: String::with_capacity(128),
            state: SmtpState::default(),
        }
    }

    /// Handle the session, reading and writing.
    /// Should only be called once, returns when the connection should be dropped.
    pub(crate) async fn handle(mut self) -> () {
        let mut buff = vec![0; 1024];

        debug!("Accepted new client {}.", self.addr);

        match self
            .send(&Response::Greeting(self.server_name.clone()))
            .await
        {
            Ok(_) => (),
            Err(_) => return (),
        };

        loop {
            match self.stream.readable().await {
                Ok(_) => (),
                Err(e) => {
                    error!(
                        "Encountered error while waiting for socket to get ready to read: {}.",
                        e
                    );
                    break;
                }
            }

            match self.stream.try_read(&mut buff) {
                Ok(0) => break,
                Ok(n) => {
                    let msg = match std::str::from_utf8(&buff[..n]) {
                        Ok(m) => m,
                        Err(_) => {
                            debug!("Received non-utf8 characters.");
                            break;
                        }
                    };

                    let should_quit = self.input(msg).await;
                    if should_quit {
                        debug!("Server indicated to quit.");
                        break;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    warn!("Received error while reading socket: {}.", e);
                    break;
                }
            }
        }
    }

    /// Handle new incoming input.
    async fn input(&mut self, input: &str) -> bool {
        let full_input = format!("{}{}", self.remaining.as_str(), input);

        let full_input = if self.state.receiving_data {
            let (has_ended, res, rem) = super::parser::parse_data_lines(full_input.as_str());

            self.state.data.push_str(res.as_str());

            if has_ended {
                let resp = match self.handler.save(&self.state).await {
                    true => Response::Ok,
                    false => Response::TransactionFailed,
                };

                match self.send(&resp).await {
                    Ok(_) => (),
                    Err(_) => return true,
                }

                self.state.receiving_data = false;
                self.state.data = String::new();
            }

            rem
        } else {
            full_input
        };

        let (cmds, rem) = super::parser::parse(full_input.as_str());
        self.remaining = rem.to_owned();

        for (_, command) in cmds {
            debug!("Processing command {:?}.", command);
            match command {
                Some(c) => match self.process_command(c).await {
                    Ok(cmd) => {
                        let resp = self.send(&cmd).await;

                        if cmd == Response::Goodbye || resp.is_err() {
                            return true;
                        }
                    }
                    Err(_) => return true,
                },
                None => match self.send(&Response::SyntaxError).await {
                    Ok(_) => (),
                    Err(_) => return true,
                },
            }
        }

        false
    }

    async fn process_command(&mut self, command: Command) -> Result<Response, std::io::Error> {
        Ok(match command {
            Command::HELO(domain) => self.process_helo(domain),
            Command::EHLO(domain) => self.process_ehlo(domain),
            Command::FROM(sender) => self.process_from(sender),
            Command::RCPT(recipient) => self.process_rcpt(recipient).await,
            Command::DATA => self.process_data(),
            Command::RSET => self.process_reset(),
            Command::QUIT => Response::Goodbye,
        })
    }

    fn process_helo(&mut self, domain: Domain) -> Response {
        debug!("Processing HELO for {:?}.", domain);

        self.state.domain = Some(domain.clone());
        Response::Helo(self.server_name.clone())
    }

    fn process_ehlo(&mut self, domain: Domain) -> Response {
        debug!("Processing EHLO for {:?}.", domain);

        self.state.domain = Some(domain.clone());
        Response::Ehlo(self.server_name.clone())
    }

    fn process_from(&mut self, sender: Mailbox) -> Response {
        debug!("Processing FROM for {:?}.", sender);

        if self.state.domain == None {
            debug!("MAIL command was out of sequence.");
            return Response::OutOfSequence;
        }

        debug!("Sender accepted.");
        self.state.from = Some(sender.clone());
        Response::Ok
    }

    async fn process_rcpt(&mut self, recipient: Mailbox) -> Response {
        debug!("Processing recipient for {:?}.", recipient);

        if self.state.domain == None {
            debug!("RCPT command was send out of sequence.");
            return Response::OutOfSequence;
        }

        if self.state.recipients.len() >= 100 {
            debug!("Received 100 or more recipients.");
            return Response::TooManyRecipients;
        }

        if !self.handler.recipient_local(&recipient).await {
            debug!("Handler indicated the recipient was not local.");
            return Response::RecipientNotLocal;
        }

        debug!("Recipient accepted.");
        self.state.recipients.push(recipient);
        Response::Ok
    }

    fn process_data(&mut self) -> Response {
        if self.state.domain == None {
            debug!("Received DATA without EHLO.");
            return Response::OutOfSequence;
        }

        if self.state.from == None {
            debug!("Received DATA without FROM.");
            return Response::OutOfSequence;
        }

        if self.state.recipients.len() <= 0 {
            debug!("Received DATA without RCPT.");
            return Response::OutOfSequence;
        }

        self.state.receiving_data = true;
        Response::StartData
    }

    fn process_reset(&mut self) -> Response {
        self.state.from = None;
        self.state.recipients = Vec::new();
        self.state.data = String::new();

        Response::Ok
    }

    /// Send a response to the client.
    async fn send(&self, res: &Response) -> Result<(), std::io::Error> {
        debug!("Sending `{:?}`.", res);

        match self.stream.writable().await {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "Encountered error while waiting for socket to get ready to write: {}.",
                    e
                );
            }
        }

        self.stream.try_write(res.to_response().as_bytes())?;

        Ok(())
    }
}
