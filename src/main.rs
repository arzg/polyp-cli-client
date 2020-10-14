use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use polyp::{ServerMsg, UserInput};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::process::Command;
use tungstenite::{Message, WebSocket};

const CTRL_C_EVENT: Event = Event::Key(KeyEvent {
    code: KeyCode::Char('c'),
    modifiers: KeyModifiers::CONTROL,
});

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1618);

    Command::new("polyp-server")
        .arg(socket_addr.to_string())
        .spawn()?;

    let mut server_websocket = {
        let listener = TcpListener::bind(socket_addr)?;
        println!("polyp-cli-client: listening at socket {:?}\r", socket_addr);

        let stream = listener.incoming().next().unwrap()?;

        tungstenite::server::accept(stream)?
    };

    println!("polyp-cli-client: connected to server\r");

    loop {
        match event::read()? {
            CTRL_C_EVENT => {
                eprintln!("polyp-cli-client: shutting down...\r");

                server_websocket.close(None)?;
                terminal::disable_raw_mode()?;

                return Ok(());
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => handle_key(c, server_websocket)?,
            _ => {}
        }
    }
}

fn handle_key(c: char, server_websocket: WebSocket<TcpStream>) -> anyhow::Result<()> {
    println!("polyp-cli-client: received keystroke ‘{}’\r", c);

    let serialized = serde_json::to_vec(&UserInput::PressedKey(c))?;
    server_websocket.write_message(Message::Binary(serialized))?;

    println!("polyp-cli-client: wrote user input to server\r");

    let server_msg = {
        let message = server_websocket.read_message()?;
        println!("polyp-cli-client: read message from server\r");

        if let Message::Binary(json) = message {
            serde_json::from_slice(&json)?
        } else {
            unreachable!();
        }
    };

    match server_msg {
        ServerMsg::NewText(text) => {
            let text = text.replace('\n', "\r\n");
            println!("polyp-cli-client: new text from server:\r\n{}\r", text);
        }
    }

    Ok(())
}
