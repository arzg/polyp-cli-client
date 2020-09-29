use polyp::ServerMsg;
use std::io::Read;
use std::process::{Command, Stdio};

fn main() -> anyhow::Result<()> {
    let server = Command::new("polyp-server")
        .stdout(Stdio::piped())
        .spawn()?;

    let mut server_stdout = server.stdout.unwrap();
    let mut server_output = Vec::new();
    server_stdout.read_to_end(&mut server_output)?;

    let server_msg: ServerMsg = serde_json::from_slice(&server_output)?;

    match server_msg {
        ServerMsg::NewText(text) => println!("{}", text),
    }

    Ok(())
}
