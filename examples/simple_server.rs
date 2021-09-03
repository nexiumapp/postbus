use std::net::SocketAddr;

use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
pub async fn main() -> io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 2525));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind the socket!");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepting new connection from {}.", addr);

        tokio::spawn(async move { handle_socket(socket).await });
    }
}

async fn handle_socket(socket: TcpStream) {
    loop {
        socket.readable().await.unwrap();
        let mut buf = vec![0; 32 * 1024];

        match socket.try_read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                buf.truncate(n);

                let input = match std::str::from_utf8(&buf) {
                    Ok(inp) => inp,
                    Err(_) => panic!("Received non-utf8 characters."),
                };

                let res = postbus::parser::parse(input);
                println!("Received {:?}", buf);

                if let Ok((rem, cmd)) = res {
                    println!("Parsed {}", cmd);
                    println!("Remaining {:?}", rem);
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                panic!("Error occured: {}.", e);
            }
        }
    }
}
