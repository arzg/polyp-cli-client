use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use polyp::{Key, Ui, UserInput};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const CTRL_C_EVENT: Event = Event::Key(KeyEvent {
    code: KeyCode::Char('c'),
    modifiers: KeyModifiers::CONTROL,
});

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let server = Command::new("polyp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut server_stdin = server.stdin.unwrap();
    let mut server_stdout = BufReader::new(server.stdout.unwrap());

    loop {
        let code = match event::read()? {
            CTRL_C_EVENT => {
                eprintln!("polyp-cli-client: shutting down...\r");
                terminal::disable_raw_mode()?;

                return Ok(());
            }

            Event::Key(KeyEvent { code, .. }) => code,

            _ => continue,
        };

        match code {
            KeyCode::Char(c) => handle_key(Key::Char(c), &mut server_stdin, &mut server_stdout)?,
            KeyCode::Backspace => {
                handle_key(Key::Backspace, &mut server_stdin, &mut server_stdout)?
            }
            KeyCode::Up => handle_key(Key::Up, &mut server_stdin, &mut server_stdout)?,
            KeyCode::Down => handle_key(Key::Down, &mut server_stdin, &mut server_stdout)?,
            KeyCode::Left => handle_key(Key::Left, &mut server_stdin, &mut server_stdout)?,
            KeyCode::Right => handle_key(Key::Right, &mut server_stdin, &mut server_stdout)?,
            _ => {}
        }
    }
}

fn handle_key(
    key: Key,
    mut server_stdin: impl Write,
    mut server_stdout: impl BufRead,
) -> anyhow::Result<()> {
    eprintln!("polyp-cli-client: received keystroke ‘{:?}’\r", key);

    let serialized = serde_json::to_vec(&UserInput::PressedKey(key))?;
    server_stdin.write_all(&serialized)?;
    server_stdin.write_all(b"\n")?;

    eprintln!("polyp-cli-client: wrote user input to server\r");

    let ui = {
        let mut message = String::new();
        server_stdout.read_line(&mut message)?;
        eprintln!("polyp-cli-client: read message from server\r");

        serde_json::from_str(&message)?
    };

    eprintln!("polyp-cli-client: UI from server:\r\n{}\r", format_ui(ui));

    Ok(())
}

fn format_ui(ui: Ui) -> String {
    match ui {
        Ui::Value(_) => todo!(),
        Ui::TextField {
            mut current_text,
            cursor_idx,
        } => {
            current_text.insert(cursor_idx, '|');
            current_text
        }
    }
}
