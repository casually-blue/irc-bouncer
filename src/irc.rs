pub enum IRCCommand {
    Quit,
    Auth {nick: String, username: String, realname: String},
    Join {channel: String},
    Text(String),
    Message{channel: String, to: Option<String>, message: String},
    Pong{token: String}
}
