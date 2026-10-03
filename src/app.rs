use std::path::PathBuf;

use ratatui::crossterm::event::KeyCode;
use ratatui::text::{Line, Text};
use ratatui::widgets::ListState;

use crate::markdown;
use crate::tree::{self, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Logo,
    File { path: PathBuf, content: String },
}

#[derive(Debug)]
pub struct App {
    pub tree: Vec<Node>,
    pub selected: usize,
    /// スクロール位置をフレーム間で保持するためのサイドバーの描画状態
    pub tree_state: ListState,
    pub view: View,
    /// 表示中のファイルを整形した結果と、そのときの幅
    rendered: Option<(u16, Text<'static>)>,
    pub should_quit: bool,
}

impl App {
    pub fn new(tree: Vec<Node>) -> Self {
        Self {
            tree,
            selected: 0,
            tree_state: ListState::default(),
            view: View::Logo,
            rendered: None,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < tree::visible(&self.tree).len() {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char(' ') => {
                tree::toggle(&mut self.tree, self.selected);
                let len = tree::visible(&self.tree).len();
                self.selected = self.selected.min(len.saturating_sub(1));
            }
            KeyCode::Enter => self.open_selected(),
            _ => {}
        }
    }

    /// 表示中のファイルを幅 `width` で整形した行。
    /// 描画のたびに整形し直さないよう、ファイルか幅が変わったときだけ作り直す。
    pub fn file_lines(&mut self, width: u16) -> &[Line<'static>] {
        let View::File { content, .. } = &self.view else {
            return &[];
        };
        if self
            .rendered
            .as_ref()
            .is_none_or(|(cached, _)| *cached != width)
        {
            self.rendered = Some((width, markdown::render(content, width)));
        }
        self.rendered
            .as_ref()
            .map_or(&[], |(_, text)| text.lines.as_slice())
    }

    fn open_selected(&mut self) {
        let Some(item) = tree::visible(&self.tree).into_iter().nth(self.selected) else {
            return;
        };
        if item.is_dir {
            return;
        }
        let content = match std::fs::read(&item.path) {
            // Ratatui は制御文字を描画しないため、タブは空白に展開しておく
            Ok(bytes) => String::from_utf8_lossy(&bytes).replace('\t', "    "),
            Err(err) => format!("ファイルを読み込めませんでした: {err}"),
        };
        self.view = View::File {
            path: item.path,
            content,
        };
        self.rendered = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs/guide.md"), "# Guide").unwrap();
        fs::write(dir.path().join("README.md"), "# Readme").unwrap();
        let tree = tree::build_tree(dir.path()).unwrap();
        (dir, App::new(tree))
    }

    #[test]
    fn new_app_starts_with_logo_and_first_item_selected() {
        let (_dir, app) = setup();
        assert_eq!(app.view, View::Logo);
        assert_eq!(app.selected, 0);
        assert!(!app.should_quit);
    }

    #[test]
    fn j_and_down_move_selection_down_within_bounds() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.selected, 1);
        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn k_and_up_move_selection_up_within_bounds() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Down);
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.selected, 0);
        app.handle_key(KeyCode::Up);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn space_toggles_directory() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char(' '));
        assert_eq!(tree::visible(&app.tree).len(), 3);
        app.handle_key(KeyCode::Char(' '));
        assert_eq!(tree::visible(&app.tree).len(), 2);
    }

    #[test]
    fn collapsing_keeps_selection_in_range() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char(' '));
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.selected, 2);
        app.handle_key(KeyCode::Char('k'));
        app.handle_key(KeyCode::Char('k'));
        app.handle_key(KeyCode::Char(' '));
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn enter_on_file_shows_its_content() {
        let (dir, mut app) = setup();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        assert_eq!(
            app.view,
            View::File {
                path: dir.path().join("README.md"),
                content: "# Readme".to_string(),
            }
        );
    }

    #[test]
    fn enter_expands_tabs_in_file_content() {
        let (dir, mut app) = setup();
        fs::write(dir.path().join("README.md"), "a\tb").unwrap();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        let View::File { content, .. } = &app.view else {
            panic!("file view expected");
        };
        assert_eq!(content, "a    b");
    }

    #[test]
    fn enter_on_directory_does_nothing() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.view, View::Logo);
        assert_eq!(tree::visible(&app.tree).len(), 2);
    }

    #[test]
    fn q_quits() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn keys_on_empty_tree_do_not_panic() {
        let mut app = App::new(Vec::new());
        for code in [
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char(' '),
            KeyCode::Enter,
        ] {
            app.handle_key(code);
        }
        assert_eq!(app.selected, 0);
        assert_eq!(app.view, View::Logo);
    }

    fn line_strings(lines: &[Line]) -> Vec<String> {
        lines.iter().map(Line::to_string).collect()
    }

    #[test]
    fn file_lines_is_empty_for_logo() {
        let (_dir, mut app) = setup();
        assert!(app.file_lines(20).is_empty());
    }

    #[test]
    fn file_lines_renders_opened_file_as_markdown() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        assert_eq!(line_strings(app.file_lines(6)), ["Readme", "──────"]);
    }

    #[test]
    fn file_lines_follow_width_changes() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        app.file_lines(6);
        assert_eq!(line_strings(app.file_lines(8))[1], "────────");
    }

    #[test]
    fn file_lines_follow_opened_file() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        app.file_lines(10);
        app.handle_key(KeyCode::Char('k'));
        app.handle_key(KeyCode::Char(' '));
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        assert_eq!(line_strings(app.file_lines(10))[0], "Guide");
    }
}
