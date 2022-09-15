use std::{cell::RefCell, rc::Rc, time::Duration};

use clap::Parser;
use crossterm::event::{KeyEvent, KeyCode};
use inquire::Text;
use tokio::{io::{split, AsyncWriteExt, self, BufReader, AsyncBufReadExt}, sync::mpsc::{channel, Sender, Receiver}, task};

mod tls_setup;
use tls_setup::*;
use tui::{backend::{CrosstermBackend, Backend}, Terminal, layout::{Layout, Direction, Constraint, Alignment}, Frame, widgets::{Paragraph, Borders, Block, BorderType}, style::{Style, Color}};

#[derive(Parser)]
struct Options {
    host: String,

    #[clap(default_value_t=6697)]
    port: u16,
}

pub struct App {

}

#[tokio::main]
async fn main() -> io::Result<()> {
    let options = Options::parse();


    let stream = initialize_tls(options.host, options.port).await?;

    let (mut tls_read_side, mut tls_write_side) = split(stream);

    let (reader_handle, mut reader_output_channel): (Sender<String>, Receiver<String>) = channel(256);
    let reader_handle_client_messages = reader_handle.clone();
    let reader_handle_copy_tls = reader_handle.clone();

    let (wrtr_command_handle, mut wrtr_adapter): (Sender<String>, Receiver<String>) = channel(256);
    let wrtr_command_write_handle = wrtr_command_handle.clone();

    tokio::select!(
        _ = task::spawn(async move {
            let app = Rc::new(RefCell::new(App {}));
            start_ui(app).unwrap();
        }) => {},
        _ = task::spawn(async move {
            let mut output = tokio::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("output").await.unwrap();

            while let Some(message) = reader_output_channel.recv().await {
                output.write_all(message.as_bytes()).await.unwrap();
            }
        }) => {},

        _ = task::spawn(async move {
            let mut reader = BufReader::new(&mut tls_read_side);
            let wrtr_handle = wrtr_command_handle.clone();
            loop {
                let mut data = String::new();
                reader.read_line(&mut data).await.unwrap();
                if data.is_empty(){
                    break;
                } else if data.starts_with("PING") {
                    wrtr_handle.send(format!("PONG {}", data.chars().skip(5).collect::<String>()).trim().into()).await.unwrap();
                } 

                reader_handle_client_messages.send(data).await.unwrap();
            }
        }) => {},

        _ = task::spawn(async move {
            while let Some(cmd) = wrtr_adapter.recv().await {
                tls_write_side.write_all(format!("{}\r\n", cmd).as_bytes()).await.unwrap();
                reader_handle_copy_tls.send(format!("{}\r\n", cmd)).await.unwrap();

            }
        }) => {},
        /*
        _ = async {
            while let Ok(line) = Text::new("COMMAND> ").prompt() {
                if let Some('!') = line.chars().next() {
                    match line.chars().skip(1).collect::<String>().as_ref() {
                        "exit" => break,
                        "lug" => {
                            wrtr_command_write_handle.send("USER casuallyblue test test :Sierra".into()).await.unwrap();
                            wrtr_command_write_handle.send("NICK casuallyblue".into()).await.unwrap();
                            wrtr_command_write_handle.send("JOIN #utdlug".into()).await.unwrap();
                        }
                        cmd => {
                            wrtr_command_write_handle.send(cmd.to_string()).await.unwrap();
                        },
                    }
                } else {
                    wrtr_command_write_handle.send(format!("PRIVMSG #utdlug :{}", line)).await.unwrap();
                }


            }
        } => {}*/);

    Ok(())
}

pub fn start_ui(app: Rc<RefCell<App>>) -> io::Result<()> {
    let stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    loop {
        let app = app.borrow();
        terminal.draw(|rect| draw(rect, &app))?;

        if crossterm::event::poll(Duration::from_millis(200))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(k) => {
                    if k.code == KeyCode::Esc {
                        break
                    }
                }
                _ => {}
            }
        }
    }

    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub fn draw<B>(rect: &mut Frame<B>, _app: &App)
where
    B: Backend,
{
    let size = rect.size();
    // TODO check size

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)].as_ref())
        .split(size);

    // Title block
    let title = draw_title();
    rect.render_widget(title, chunks[0]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("IRC Client")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}
