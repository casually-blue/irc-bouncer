use inquire::Text;
use std::io::Write;
use tokio::{io::{split, AsyncWriteExt, self, BufReader, AsyncBufReadExt}, spawn, sync::mpsc::channel};

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

    let (rdr_handle,mut rdr_adapter) = channel(256);
    let rdr_handle_messages_handle = rdr_handle.clone();
    let rdr_handle_write_handle = rdr_handle.clone();
    let (wrtr_command_handle, mut wrtr_adapter) = channel(256);
    let wrtr_command_write_handle = wrtr_command_handle.clone();

    let mut output = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("output")?;

    let reader_output_task = spawn(async move {
        while let Some(message) = rdr_adapter.recv().await {
            write!(output, "{}", message).unwrap();
        }
    });

    let reader_handle_messages_task = spawn(async move {
        let mut rdr = BufReader::new(&mut reader);
        let wrtr_handle = wrtr_command_handle.clone();
        loop {
            let mut data = String::new();
            rdr.read_line(&mut data).await.unwrap();
            if data.is_empty(){
                break;
            } else if data.starts_with("PING") {
                wrtr_handle.send(format!("PONG {}", data.chars().skip(5).collect::<String>())).await.unwrap();
            } 

            rdr_handle_messages_handle.send(data).await.unwrap();
        }
    });

    let writer_handle_messages_task = spawn(async move {
        while let Some(cmd) = wrtr_adapter.recv().await {
                    writer.write_all(format!("{}\r\n", cmd).as_bytes()).await.unwrap();
                    rdr_handle_write_handle.send(format!("{}\r\n", cmd)).await.unwrap();
                
        }
    });

    while let Ok(line) = Text::new("COMMAND> ").prompt() {
        if let Some('!') = line.chars().next() {
            match line.chars().skip(1).collect::<String>().as_ref() {
                "exit" => break,
                "lug" => {
                    wrtr_command_write_handle.send("USER casuallyblue test test :Sierra".into()).await.unwrap();
                    wrtr_command_write_handle.send("NICK casuallyb_".into()).await.unwrap();
                    wrtr_command_write_handle.send("JOIN #utdlug".into()).await.unwrap();
                }
                cmd => {
                    rdr_handle.clone().send(format!("{}\r\n", line)).await.unwrap();
                },
            }
        } else {
            wrtr_command_write_handle.send(format!("PRIVMSG #utdlug :{}", line)).await.unwrap();
        }

        
    }

    reader_handle_messages_task.await?;
    reader_output_task.await?;
    writer_handle_messages_task.await?;

    Ok(())
}
