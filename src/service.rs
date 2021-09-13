use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

use crate::{Handler, SmtpSession};

/// Smtp service.
pub struct SmtpService {
    address: SocketAddr,
    server_name: String,
    handler: Arc<dyn Handler>,
}

impl SmtpService {
    /// Create a new service.
    /// This does not listen on the port, call `.listen()` for that.
    pub fn create(
        address: SocketAddr,
        server_name: String,
        handler: Arc<dyn Handler>,
    ) -> SmtpService {
        SmtpService {
            address,
            server_name,
            handler,
        }
    }

    /// Listen the server.
    /// This is a normal Tokio server, and should be awaited.
    pub async fn listen(&self) -> ! {
        let listener = TcpListener::bind(self.address)
            .await
            .expect("Could not listen on the SMTP port.");

        debug!("Started listening on address {}.", self.address);

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to accept SMTP socket: {}", e);
                    continue;
                }
            };

            let session =
                SmtpSession::new(stream, addr, self.server_name.clone(), self.handler.clone());

            tokio::spawn(session.handle());
        }
    }
}
