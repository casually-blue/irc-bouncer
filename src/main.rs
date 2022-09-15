#![feature(type_ascription)]
use clap::Parser;
use cursive::views::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::{
    io::{self, split, AsyncBufReadExt, BufReader},
    task,
};

mod tls_setup;
use tls_setup::*;

mod pipes;
mod ui;
mod irc;
use irc::IRCCommand;

#[derive(Parser)]
struct Options {
    host: String,

    #[clap(default_value_t = 6697)]
    port: u16,
}

pub struct App {}

#[tokio::main]
async fn main() -> io::Result<()> {
    let options = Options::parse();

    let stream = initialize_tls(options.host, options.port).await?;

    let (mut tls_read_side, tls_write_side) = split(stream);

    let (reader_handle, reader_output_channel): (Sender<String>, Receiver<String>) = channel();
    let reader_handle_client_messages = reader_handle.clone();
    let reader_handle_copy_tls = reader_handle.clone();

    let (wrtr_command_handle, wrtr_adapter): (Sender<IRCCommand>, Receiver<IRCCommand>) = channel();
    let wrtr_command_write_handle = wrtr_command_handle.clone();

    let content = TextContent::new("");
    let ui_content_handle = content.clone();

    tokio::select!(
    _ = task::spawn(pipes::update_chat_pane(reader_output_channel, content)) => {}
    _ = task::spawn(pipes::tls_responder(tls_write_side, wrtr_adapter, reader_handle_copy_tls)) => {}
    _ = task::spawn(ui::run_ui(ui_content_handle, wrtr_command_write_handle)) => {},
    _ = task::spawn(async move {
        let mut reader = BufReader::new(&mut tls_read_side);
        let wrtr_handle = wrtr_command_handle.clone();
        loop {
            let mut data = String::new();
            reader.read_line(&mut data).await.unwrap();
            if data.is_empty(){
                break;
            } else if data.starts_with("PING") {
                wrtr_handle.send(IRCCommand::Pong{token: data.chars().skip(5).collect::<String>().trim().into()}).unwrap();
            }

            reader_handle_client_messages.send(data).unwrap();
        }
    }) => {},

    );

    Ok(())
}
