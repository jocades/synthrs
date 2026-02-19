use std::fmt::Write;
use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{self, event::KeyCode},
};

fn main() -> io::Result<()> {
    ratatui::run(app)
}

fn app(term: &mut DefaultTerminal) -> io::Result<()> {
    let mut buf = String::new();
    loop {
        term.draw(draw)?;
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key_event) => {
                _ = write!(&mut buf, "{key_event:?}\n");
                if key_event.is_press() && key_event.code == KeyCode::Char('q') {
                    break;
                }
            }
            _ => {}
        }
    }
    std::fs::write("kv", buf).unwrap();
    Ok(())
}

fn draw(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
