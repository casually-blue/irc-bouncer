use std::{
    io::Result,
    sync::mpsc::{Receiver, Sender},
};

use cursive::views::TextContent;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream;

use crate::irc::IRCCommand;

pub async fn update_chat_pane(
    content_channel: Receiver<String>,
    content_pane: TextContent,
    ) -> Result<()> {
    while let Ok(message) = content_channel.recv() {
        content_pane.append(message);
    }

    Ok(())
}

pub async fn tls_responder(
    mut tls_respond_handle: WriteHalf<TlsStream<TcpStream>>,
    data_handle: Receiver<IRCCommand>,
    forwarder_handle: Sender<String>,
    ) -> Result<()> {
    loop {
        let maybe_command = data_handle.recv();
        if let Ok(cmd) = maybe_command {
            use IRCCommand::*;
            let mut commands: Vec<String> = vec![];

            match cmd {
                Quit => {
                    commands.push("QUIT".into());
                },
                Pong { token } => {
                    commands.push(format!("PONG {token}"));
                }
                Auth { nick, username, realname } => {
                    commands.push(format!("NICK {nick}"));
                    commands.push(format!("USER {username} test test :{realname}"));
                }
                Join {channel} => {
                    commands.push(format!("JOIN #{channel}"));
                }
                Text(command) => {
                    commands.push(command);
                }
                Message { channel, to, message } => {
                    match to {
                        Some(user) => {
                            commands.push(format!("PRIVMSG #{channel} :{user}: {message}"));
                        }
                        None => {
                            commands.push(format!("PRIVMSG #{channel} :{message}"));
                        }
                    }
                }
            };

            for command in commands.iter() {
                let actual_command = format!("{command}\r\n");
                tls_respond_handle.write_all(actual_command.as_bytes()).await.unwrap();
                forwarder_handle.send(actual_command).unwrap();
            }
        } else {
            break
        }
    }

    Ok(())
}
