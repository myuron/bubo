use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, List, ListItem, Paragraph};

use crate::app::{App, View};
use crate::logo::LOGO;
use crate::tree;

const SIDEBAR_WIDTH: u16 = 30;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [sidebar, main] =
        Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
            .areas(frame.area());
    draw_tree(frame, app, sidebar);
    match &app.view {
        View::Logo => draw_logo(frame, main),
        View::File { path, .. } => {
            let title = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let block = Block::bordered().title(title);
            let inner = block.inner(main);
            // 整形済みの行のうち、見えている範囲だけを複製して描画する
            let lines: Vec<_> = app
                .file_lines(inner.width)
                .iter()
                .take(inner.height as usize)
                .cloned()
                .collect();
            frame.render_widget(Paragraph::new(lines).block(block), main);
        }
    }
}

fn draw_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = tree::visible(&app.tree)
        .into_iter()
        .map(|item| {
            let indent = "  ".repeat(item.depth);
            let label = match (item.is_dir, item.expanded) {
                (true, true) => format!("{indent}▾ {}", item.name),
                (true, false) => format!("{indent}▸ {}", item.name),
                (false, _) => format!("{indent}  {}", item.name),
            };
            ListItem::new(label)
        })
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title("Files"))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    app.tree_state.select(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut app.tree_state);
}

/// 各行の左端を揃えたまま、ロゴ全体を領域の中央に配置する。
fn draw_logo(frame: &mut Frame, area: Rect) {
    let block = Block::bordered();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = LOGO
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let height = LOGO.lines().count() as u16;
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(inner);
    let [logo_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Paragraph::new(LOGO), logo_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logo::LOGO;
    use crate::tree;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::KeyCode;
    use std::fs;

    const TERM_WIDTH: u16 = 100;

    fn new_terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(TERM_WIDTH, 20)).unwrap()
    }

    fn render(terminal: &mut Terminal<TestBackend>, app: &mut App) -> String {
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn app_with_files() -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs/guide.md"), "guide body").unwrap();
        fs::write(dir.path().join("README.md"), "readme body").unwrap();
        let tree = tree::build_tree(dir.path()).unwrap();
        (dir, App::new(tree))
    }

    /// README.md を `content` に書き換えて開いた画面を返す
    fn render_readme(content: &str) -> String {
        let (dir, mut app) = app_with_files();
        fs::write(dir.path().join("README.md"), content).unwrap();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        render(&mut new_terminal(), &mut app)
    }

    #[test]
    fn renders_tree_and_logo_initially() {
        let (_dir, mut app) = app_with_files();
        let screen = render(&mut new_terminal(), &mut app);
        assert!(screen.contains("▸ docs"));
        assert!(screen.contains("README.md"));
        for line in LOGO.lines() {
            assert!(screen.contains(line), "logo line missing: {line}");
        }
    }

    #[test]
    fn renders_expanded_directory_with_indent() {
        let (_dir, mut app) = app_with_files();
        app.handle_key(KeyCode::Char(' '));
        let screen = render(&mut new_terminal(), &mut app);
        assert!(screen.contains("▾ docs"));
        assert!(screen.contains("  guide.md"));
    }

    #[test]
    fn sidebar_keeps_scroll_offset_when_moving_up() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..30 {
            fs::write(dir.path().join(format!("f{i:02}.md")), "").unwrap();
        }
        let mut app = App::new(tree::build_tree(dir.path()).unwrap());
        let mut terminal = new_terminal();
        for _ in 0..20 {
            app.handle_key(KeyCode::Char('j'));
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        }
        app.handle_key(KeyCode::Char('k'));
        let screen = render(&mut terminal, &mut app);
        let first_row = screen.lines().nth(1).unwrap();
        assert!(
            first_row.contains("f03.md"),
            "unexpected first row: {first_row}"
        );
    }

    #[test]
    fn renders_file_content_after_enter() {
        let screen = render_readme("readme body");
        assert!(screen.contains("readme body"));
        assert!(!screen.contains(LOGO.lines().nth(2).unwrap()));
    }

    #[test]
    fn renders_file_content_as_markdown_within_border() {
        let screen = render_readme("# Title\n\n- **item**");
        let rows: Vec<&str> = screen.lines().collect();
        assert!(
            rows[1].contains("│Title "),
            "unexpected title row: {}",
            rows[1]
        );
        let inner_width = (TERM_WIDTH - SIDEBAR_WIDTH - 2) as usize;
        assert!(
            rows[2].ends_with(&format!("│{}│", "─".repeat(inner_width))),
            "unexpected rule row: {}",
            rows[2]
        );
        assert!(
            rows[4].contains("│• item"),
            "unexpected item row: {}",
            rows[4]
        );
    }
}
