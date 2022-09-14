use std::io;
use inquire::Text;
use tokio::io::{split, stdin as tokio_stdin, stdout as tokio_stdout, AsyncWriteExt, AsyncReadExt};

mod tls_setup;
use tls_setup::*;

struct Options {
    host: String,
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let options = Options {
        host: "irc.oftc.net".into(),
        port: 6697,
    };

    let stream = initialize_tls(options.host, options.port).await?;
    let (mut reader, mut writer) = split(stream);

    let (mut stdin, mut stdout) = (tokio_stdin(), tokio_stdout());

    while let Ok(line) = Text::new("COMMAND> ").prompt() {
        if let Some('!') = line.chars().next() {
            match line.chars().skip(1).collect::<String>().as_ref() {
                "exit" => break,
                _ => {}
            }
        } else {
            writer.write_all(format!("{}\r\n", line).as_bytes()).await?;
        }

        let mut text = Vec::new();
        let _ = reader.read(&mut text).await?;
        println!("{:?}", text);

        let _ = stdout.write(&text).await?;
    }

    writer.shutdown().await?;

    Ok(())
}
