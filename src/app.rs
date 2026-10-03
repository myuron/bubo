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

/// キー入力を受け付けるペイン
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Content,
}

#[derive(Debug)]
pub struct App {
    pub tree: Vec<Node>,
    pub selected: usize,
    /// スクロール位置をフレーム間で保持するためのサイドバーの描画状態
    pub tree_state: ListState,
    pub view: View,
    pub focus: Focus,
    /// 表示中のファイルを整形した結果と、そのときの幅
    rendered: Option<(u16, Text<'static>)>,
    /// ファイル表示の先頭に来る行の位置
    scroll: usize,
    /// 直近に描画したファイル表示の高さ。ページ単位のスクロール量と末尾の判定に使う
    viewport_height: u16,
    pub should_quit: bool,
}

impl App {
    pub fn new(tree: Vec<Node>) -> Self {
        Self {
            tree,
            selected: 0,
            tree_state: ListState::default(),
            view: View::Logo,
            focus: Focus::Tree,
            rendered: None,
            scroll: 0,
            viewport_height: 0,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab | KeyCode::BackTab => self.toggle_focus(),
            _ => match self.focus {
                Focus::Tree => self.handle_tree_key(code),
                Focus::Content => self.handle_content_key(code),
            },
        }
    }

    /// ファイルを表示していないときは、右ペインにフォーカスを移さない
    fn toggle_focus(&mut self) {
        self.focus = match (self.focus, &self.view) {
            (Focus::Tree, View::File { .. }) => Focus::Content,
            _ => Focus::Tree,
        };
    }

    fn handle_tree_key(&mut self, code: KeyCode) {
        match code {
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

    fn handle_content_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
            KeyCode::PageDown => self.scroll_down(self.viewport_height.into()),
            KeyCode::PageUp => self.scroll_up(self.viewport_height.into()),
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

    /// 幅 `width`・高さ `height` の領域に見えている整形済みの行。
    /// 幅の変化で行数が減った場合も末尾を越えないよう、ここでスクロール位置を補正する。
    pub fn file_view(&mut self, width: u16, height: u16) -> &[Line<'static>] {
        self.viewport_height = height;
        self.file_lines(width);
        self.scroll = self.scroll.min(self.max_scroll());
        let start = self.scroll;
        let lines = self.file_lines(width);
        &lines[start..lines.len().min(start + usize::from(height))]
    }

    /// 最終行が表示の下端に来るときのスクロール位置
    fn max_scroll(&self) -> usize {
        self.rendered.as_ref().map_or(0, |(_, text)| {
            text.lines.len().saturating_sub(self.viewport_height.into())
        })
    }

    fn scroll_down(&mut self, amount: usize) {
        self.scroll = (self.scroll + amount).min(self.max_scroll());
    }

    fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
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
        self.scroll = 0;
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

    /// 10 段落(各 1 行 + 空行)の README を開いた App
    fn app_with_long_file() -> (TempDir, App) {
        let (dir, mut app) = setup();
        let body: Vec<String> = (0..10).map(|i| format!("line{i}")).collect();
        fs::write(dir.path().join("README.md"), body.join("\n\n")).unwrap();
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Enter);
        (dir, app)
    }

    #[test]
    fn file_view_starts_at_top() {
        let (_dir, mut app) = app_with_long_file();
        assert_eq!(line_strings(app.file_view(20, 3)), ["line0", "", "line1"]);
    }

    #[test]
    fn focus_starts_on_tree() {
        let (_dir, app) = setup();
        assert_eq!(app.focus, Focus::Tree);
    }

    #[test]
    fn tab_toggles_focus_when_file_is_open() {
        let (_dir, mut app) = app_with_long_file();
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Content);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Tree);
        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.focus, Focus::Content);
    }

    #[test]
    fn tab_keeps_focus_on_tree_while_logo_is_shown() {
        let (_dir, mut app) = setup();
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Tree);
    }

    #[test]
    fn j_and_k_scroll_file_view_when_content_is_focused() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Down);
        assert_eq!(line_strings(app.file_view(20, 3)), ["line1", "", "line2"]);
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(line_strings(app.file_view(20, 3)), ["", "line1", ""]);
        app.handle_key(KeyCode::Up);
        assert_eq!(line_strings(app.file_view(20, 3))[0], "line0");
        assert_eq!(app.selected, 1, "scrolling must not move the tree cursor");
    }

    #[test]
    fn j_does_not_scroll_file_view_when_tree_is_focused() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::PageDown);
        assert_eq!(line_strings(app.file_view(20, 3))[0], "line0");
    }

    #[test]
    fn space_and_k_do_not_affect_tree_when_content_is_focused() {
        let (_dir, mut app) = app_with_long_file();
        // ディレクトリを選んでおき、Space が効けば開いてしまう状態にする
        app.handle_key(KeyCode::Char('k'));
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Char(' '));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.selected, 0);
        assert_eq!(tree::visible(&app.tree).len(), 2);
    }

    #[test]
    fn enter_does_not_reopen_file_when_content_is_focused() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Tab);
        for _ in 0..4 {
            app.handle_key(KeyCode::Char('j'));
        }
        app.handle_key(KeyCode::Enter);
        assert_eq!(line_strings(app.file_view(20, 3)), ["line2", "", "line3"]);
    }

    #[test]
    fn page_keys_scroll_file_view_by_viewport_height() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::PageDown);
        assert_eq!(line_strings(app.file_view(20, 3)), ["", "line2", ""]);
        app.handle_key(KeyCode::PageDown);
        assert_eq!(line_strings(app.file_view(20, 3)), ["line3", "", "line4"]);
        app.handle_key(KeyCode::PageUp);
        assert_eq!(line_strings(app.file_view(20, 3)), ["", "line2", ""]);
    }

    #[test]
    fn scroll_stops_at_top_and_bottom() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(line_strings(app.file_view(20, 3))[0], "line0");
        for _ in 0..50 {
            app.handle_key(KeyCode::Char('j'));
        }
        // 描画を挟まなくても末尾で止まっているので、1 回戻せばすぐに表示が変わる
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(line_strings(app.file_view(20, 3)), ["", "line8", ""]);
    }

    #[test]
    fn reopening_file_resets_scroll() {
        let (_dir, mut app) = app_with_long_file();
        app.file_view(20, 3);
        app.handle_key(KeyCode::Tab);
        for _ in 0..4 {
            app.handle_key(KeyCode::Char('j'));
        }
        app.handle_key(KeyCode::Tab);
        // 開き直す先も表示より長いので、末尾へのクランプでは先頭に戻らない
        app.handle_key(KeyCode::Enter);
        assert_eq!(line_strings(app.file_view(20, 3))[0], "line0");
    }

    #[test]
    fn page_keys_on_logo_do_not_panic() {
        let (_dir, mut app) = setup();
        for code in [KeyCode::Tab, KeyCode::PageDown, KeyCode::PageUp] {
            app.handle_key(code);
        }
        assert!(app.file_view(20, 3).is_empty());
    }
}
