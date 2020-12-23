use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use polyp::protocol::Connection;
use polyp::{Key, Ui, UserInput};
use std::io::{self, Write};
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

    let mut server_connection = Connection::new_from_child(server).unwrap();

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

        let key = match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            _ => continue,
        };

        eprintln!("polyp-cli-client: received keystroke ‘{:?}’\r", key);

        server_connection.send_message(&UserInput::PressedKey(key))?;
        eprintln!("polyp-cli-client: wrote user input to server\r");

        let ui = server_connection.recv_message()?;
        eprintln!("polyp-cli-client: read UI from server\r");

        let mut stdout = io::stdout();
        crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        print!("\r{}", format_ui(ui));
        stdout.flush()?;
    }
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
