use std::sync::mpsc::Sender;
use crate::irc::IRCCommand;

use cursive::{
    view::{Nameable, Resizable},
    views::*,
    Cursive, CursiveExt,
};

pub enum InputCommand {
    Exit,
    Join {channel: String},
    Message {channel: String, to: Option<String>, message: String},
    Command(String),
}

pub fn parse_command_input(command: &str, current_channel: &str) -> InputCommand {
    if command.starts_with('!'){
        match command.chars().skip(1).collect::<String>().as_ref() {
            "exit" | "quit" => {
                InputCommand::Exit
            }
            "lug" => {
                InputCommand::Join {channel: "utdlug".into()}
            }
            cmd => {
                InputCommand::Command(cmd.to_string())
            }
        }
    } else {
        InputCommand::Message {channel: current_channel.into(), to: None, message: command.into() }
    }
}

pub fn handle_chat(command_handle: Sender<IRCCommand>) -> Box<dyn FnMut(&mut Cursive, &str)> {
    let mut current_channel: String = "".into();
    let (username, nick, realname): (String, String, String) = ("casuallyblue".into(), "casuallyblue".into(), "Sierra".into());

    Box::new(move | cursive, text| {
        use InputCommand::*;
        match parse_command_input(text, current_channel.as_ref()) {
            Exit => {
                command_handle.send(IRCCommand::Quit).unwrap();
                cursive.quit();
            }
            Join{channel} => {
                current_channel = channel.clone();
                command_handle
                    .send(IRCCommand::Auth {nick: nick.clone(), username: username.clone(), realname: realname.clone()})
                    .unwrap();
                command_handle
                    .send(IRCCommand::Join{channel})
                    .unwrap();

                cursive.call_on_name("main_layout", |view: &mut Panel<LinearLayout>| {
                    view.set_title(current_channel.clone());
                });

            }
            Message{channel, to, message} => {
                match to {
                    Some(user) => {

                    }
                    None => {
                        command_handle.send(IRCCommand::Message{channel, to, message}).unwrap();
                    }
                }

            }
            Command(cmd) => {
                command_handle.send(IRCCommand::Text(cmd)).unwrap();
            }
        }

        cursive.call_on_name("editor_pane", |view: &mut EditView| {
            view.set_content("");
        });
        cursive.call_on_name("chat_view", |view: &mut ScrollView<TextView>| {
            view.set_scroll_strategy(cursive::view::ScrollStrategy::StickToBottom);
            view.scroll_to_bottom();
        });

    })
}

pub async fn run_ui(chat_content: TextContent, command_handle: Sender<IRCCommand>) {
    let view = TextView::new_with_content(chat_content);

    let mut cursive = Cursive::default();

    let chat_pane = ScrollView::new(view.full_width())
        .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom)
        .with_name("chat_view")
        .full_height();

    let input_pane = EditView::new()
        .on_submit_mut(handle_chat(command_handle))
        .with_name("editor_pane")
        .fixed_height(5);

    cursive.load_toml(include_str!("theme.toml")).unwrap();
    cursive.add_layer(
        Panel::new(LinearLayout::vertical().child(chat_pane).child(input_pane))
        .with_name("main_layout")
        .full_height(),
        );
    cursive.run();
}
