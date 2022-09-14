use inquire::Text;
use std::io::Write;
use tokio::{io::{split, AsyncWriteExt, self, BufReader, AsyncBufReadExt}, spawn};

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

    let mut output = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("output")?;

    let task = spawn(async move {
                let mut rdr = BufReader::new(&mut reader);
                loop {
                let mut data = String::new();
                rdr.read_line(&mut data).await.unwrap();
                if data.is_empty(){
                    break;
                }
                write!(output, "{}", data).unwrap();
            }
            });

    while let Ok(line) = Text::new("COMMAND> ").prompt() {
        if let Some('!') = line.chars().next() {
            match line.chars().skip(1).collect::<String>().as_ref() {
                "exit" => break,
                "read" => {
                },
                _ => {},
            }
        } else {
            writer.write_all(format!("{}\r\n", line).as_bytes()).await.unwrap();
            
        }
    }
    writer.shutdown().await?;

    task.await?;

    Ok(())
}
