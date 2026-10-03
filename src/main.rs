mod app;
mod logo;
mod markdown;
mod tree;
mod ui;

use std::io;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;

fn main() -> io::Result<()> {
    let tree = tree::build_tree(&std::env::current_dir()?)?;
    let mut app = App::new(tree);
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code);
        }
    }
    Ok(())
}
