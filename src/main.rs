use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use polyp::{ClientMsg, ServerMsg};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const CTRL_C_EVENT: Event = Event::Key(KeyEvent {
    code: KeyCode::Char('c'),
    modifiers: KeyModifiers::CONTROL,
});

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut server = Command::new("polyp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut server_stdin = server.stdin.as_mut().unwrap();
    let mut server_stdout = BufReader::new(server.stdout.as_mut().unwrap());

    loop {
        match event::read()? {
            CTRL_C_EVENT => {
                eprintln!("polyp-cli-client: shutting down...\r");
                server.kill()?;
                terminal::disable_raw_mode()?;

                return Ok(());
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                serde_json::to_writer(&mut server_stdin, &ClientMsg::PressedKey(c))?;
                server_stdin.write_all(b"\n")?;
                server_stdin.flush()?;

                let server_msg = {
                    let mut server_output = String::new();
                    server_stdout.read_line(&mut server_output)?;
                    serde_json::from_slice(server_output.as_bytes())?
                };

                match server_msg {
                    ServerMsg::NewText(text) => {
                        let text = text.replace('\n', "\r\n");
                        println!("polyp-cli-client: got response from server:\r\n{}\r", text);
                    }
                }
            }
            _ => {}
        }
    }
}
