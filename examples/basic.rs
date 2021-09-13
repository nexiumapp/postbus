use std::sync::Arc;

use async_trait::async_trait;
use postbus::{Handler, SmtpService, SmtpState};

#[tokio::main]
pub async fn main() {
    env_logger::init();

    let service = SmtpService::create(
        "0.0.0.0:2525".parse().unwrap(),
        "Postbus Demo".into(),
        Arc::new(PrintingHandler {}),
    );

    service.listen().await
}

struct PrintingHandler {}

#[async_trait]
impl Handler for PrintingHandler {
    async fn recipient_local(&self, recipient: &postbus::command::Mailbox) -> bool {
        println!("Checking recipient {:#?}.", recipient);

        recipient.domain == "nexium.app".into()
    }

    async fn save(&self, state: &SmtpState) -> bool {
        println!("Saving mail state:\r\n{:#?}", state);

        let parsed = mailparse::parse_mail(state.data.as_bytes()).unwrap();
        println!("Body:\r\n{:#?}", parsed);

        true
    }
}
