use pulldown_cmark::{
    Alignment, BlockQuoteKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use ratatui::style::{Color, Style};
use ratatui::symbols::line::NORMAL as BORDER;
use ratatui::text::{Line, Span, StyledGrapheme, Text};
use unicode_width::UnicodeWidthStr;

const CODE_STYLE: Style = Style::new().fg(Color::White).bg(Color::DarkGray);
const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);
const BULLET: &str = "• ";

/// 折り返し後の 1 行分の span
type Row = Vec<Span<'static>>;

/// Markdown を GitHub 風に装飾した `Text` へ変換する。
/// 行頭の装飾を保ったまま `width` で折り返すため、表示側では折り返さないこと。
pub fn render(src: &str, width: u16) -> Text<'static> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM;
    let mut renderer = Renderer::new(width as usize);
    for event in Parser::new_ext(src, options) {
        renderer.handle(event);
    }
    renderer.flush();
    Text::from(renderer.lines)
}

/// 行頭の装飾を決めるブロック要素
enum Container {
    Quote {
        color: Color,
        /// 引用の開始時点の出力済み行数
        start: usize,
        /// 本文(アラートの見出しより後)の開始時点の出力済み行数
        body_start: usize,
    },
    Item {
        marker: String,
        /// マーカーを出力済みか
        shown: bool,
    },
}

impl Container {
    /// 継続行の行頭の装飾。リスト項目はマーカー幅の空白で揃える。
    fn indent(&self) -> Span<'static> {
        match self {
            Self::Quote { color, .. } => Span::styled("▌ ", *color),
            Self::Item { marker, .. } => Span::raw(" ".repeat(marker.width())),
        }
    }
}

struct List {
    next_number: Option<u64>,
    items: usize,
    /// 段落を含む(項目間に空行を入れる)リストか
    loose: bool,
}

struct Table {
    alignments: Vec<Alignment>,
    /// 先頭行はヘッダー
    rows: Vec<Vec<Row>>,
}

struct Renderer {
    width: usize,
    lines: Vec<Line<'static>>,
    /// 現在の行の本文
    spans: Row,
    styles: Vec<Style>,
    containers: Vec<Container>,
    lists: Vec<List>,
    code_block: Option<String>,
    table: Option<Table>,
}

impl Renderer {
    fn new(width: usize) -> Self {
        Self {
            width,
            lines: Vec::new(),
            spans: Vec::new(),
            styles: Vec::new(),
            containers: Vec::new(),
            lists: Vec::new(),
            code_block: None,
            table: None,
        }
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => match &mut self.code_block {
                Some(code) => code.push_str(&text),
                None => self.push_text(&text, self.style()),
            },
            Event::Code(code) => {
                let style = self.style().patch(CODE_STYLE);
                self.push_span(Span::styled(code.into_string(), style));
            }
            Event::Html(html) | Event::InlineHtml(html) => self.push_text(&html, self.style()),
            Event::SoftBreak => self.push_span(Span::styled(" ", self.style())),
            Event::HardBreak => self.flush(),
            Event::Rule => {
                self.start_block(false);
                self.push_rule();
            }
            Event::TaskListMarker(checked) => {
                let checkbox = if checked { "☑ " } else { "☐ " };
                let numbered = self
                    .lists
                    .last()
                    .is_some_and(|list| list.next_number.is_some());
                if let Some(Container::Item { marker, .. }) = self.containers.last_mut() {
                    // 番号付きリストでは番号の後ろにチェックボックスを置く
                    if numbered {
                        marker.push_str(checkbox);
                    } else {
                        *marker = checkbox.to_string();
                    }
                }
            }
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.start_block(true),
            Tag::Heading { level, .. } => {
                self.start_block(false);
                self.styles.push(heading_style(level));
            }
            Tag::BlockQuote(kind) => {
                self.start_block(false);
                let (color, label) = alert(kind);
                let start = self.lines.len();
                self.containers.push(Container::Quote {
                    color,
                    start,
                    body_start: start,
                });
                if let Some(label) = label {
                    self.push_span(Span::styled(label, Style::new().fg(color).bold()));
                    self.flush();
                    let lines = self.lines.len();
                    if let Some(Container::Quote { body_start, .. }) = self.containers.last_mut() {
                        *body_start = lines;
                    }
                }
            }
            Tag::CodeBlock(_) => {
                self.start_block(false);
                self.code_block = Some(String::new());
            }
            Tag::HtmlBlock => self.start_block(false),
            Tag::List(start) => {
                // 入れ子のリストは親の項目に続けて表示する
                if self.lists.is_empty() {
                    self.start_block(false);
                } else {
                    self.flush();
                }
                self.lists.push(List {
                    next_number: start,
                    items: 0,
                    loose: false,
                });
            }
            Tag::Item => {
                self.flush();
                let Some(list) = self.lists.last_mut() else {
                    return;
                };
                list.items += 1;
                let marker = match &mut list.next_number {
                    Some(number) => {
                        let marker = format!("{number}. ");
                        *number += 1;
                        marker
                    }
                    None => BULLET.to_string(),
                };
                self.containers.push(Container::Item {
                    marker,
                    shown: false,
                });
            }
            Tag::Table(alignments) => {
                self.start_block(false);
                self.table = Some(Table {
                    alignments,
                    rows: Vec::new(),
                });
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(table) = &mut self.table {
                    table.rows.push(Vec::new());
                }
                if tag == Tag::TableHead {
                    self.styles.push(Style::new().bold());
                }
            }
            Tag::TableCell => {
                if let Some(row) = self.table.as_mut().and_then(|table| table.rows.last_mut()) {
                    row.push(Vec::new());
                }
            }
            Tag::Emphasis => self.styles.push(Style::new().italic()),
            Tag::Strong => self.styles.push(Style::new().bold()),
            Tag::Strikethrough => self.styles.push(Style::new().crossed_out()),
            Tag::Link { .. } => self.styles.push(Style::new().blue().underlined()),
            Tag::Image { .. } => self.push_text("[image: ", self.style()),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::HtmlBlock => self.flush(),
            TagEnd::Heading(level) => {
                self.styles.pop();
                self.flush();
                if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
                    self.push_rule();
                }
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                // 中身のない引用でも縦線だけは表示する
                if let Some(Container::Quote { start, .. }) = self.containers.last()
                    && self.lines.len() == *start
                {
                    self.blank_line();
                }
                self.containers.pop();
            }
            TagEnd::CodeBlock => self.push_code_block(),
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
            }
            TagEnd::Item => {
                self.flush();
                // 中身のない項目でもマーカーだけは表示する
                if let Some(Container::Item { shown: false, .. }) = self.containers.last() {
                    self.emit_row(Vec::new());
                }
                self.containers.pop();
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.render_table(table);
                }
            }
            TagEnd::TableHead
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link => {
                self.styles.pop();
            }
            TagEnd::Image => self.push_text("]", self.style()),
            _ => {}
        }
    }

    fn style(&self) -> Style {
        self.styles
            .iter()
            .fold(Style::new(), |acc, style| acc.patch(*style))
    }

    /// ブロック要素の開始前に、必要なら空行を入れる。
    /// コンテナの最初のブロックの前には入れないが、
    /// ゆるいリスト(段落を含むリスト)では項目の間に入れる。
    fn start_block(&mut self, is_paragraph: bool) {
        self.flush();
        let blank = match self.containers.last() {
            Some(Container::Item { shown: false, .. }) => {
                self.lists.last_mut().is_some_and(|list| {
                    list.loose |= is_paragraph;
                    list.loose && list.items > 1
                })
            }
            Some(Container::Quote { body_start, .. }) => self.lines.len() != *body_start,
            _ => true,
        };
        if blank && !self.lines.is_empty() {
            self.blank_line();
        }
    }

    /// 引用の縦線だけを残した空行を出力する
    fn blank_line(&mut self) {
        let last_quote = self
            .containers
            .iter()
            .rposition(|container| matches!(container, Container::Quote { .. }));
        let spans = last_quote.map_or_else(Vec::new, |last| {
            self.containers[..=last]
                .iter()
                .map(Container::indent)
                .collect()
        });
        self.lines.push(Line::from(spans));
    }

    fn push_rule(&mut self) {
        self.emit(|width| {
            vec![vec![Span::styled(
                BORDER.horizontal.repeat(width),
                BORDER_STYLE,
            )]]
        });
    }

    fn push_code_block(&mut self) {
        let code = self.code_block.take().unwrap_or_default();
        // 空のコードブロックでも背景付きの 1 行を表示する
        for line in code.lines().chain(code.is_empty().then_some("")) {
            self.emit(|width| code_rows(line, width));
        }
    }

    fn push_text(&mut self, text: &str, style: Style) {
        for (i, part) in text.split('\n').enumerate() {
            if i > 0 {
                self.flush();
            }
            if !part.is_empty() {
                self.push_span(Span::styled(part.to_string(), style));
            }
        }
    }

    fn push_span(&mut self, span: Span<'static>) {
        match self
            .table
            .as_mut()
            .and_then(|table| table.rows.last_mut())
            .and_then(|row| row.last_mut())
        {
            Some(cell) => cell.push(span),
            None => self.spans.push(span),
        }
    }

    /// 現在の行を折り返して出力する
    fn flush(&mut self) {
        if self.spans.is_empty() {
            return;
        }
        let spans = std::mem::take(&mut self.spans);
        self.emit(|width| wrap(spans, width, true));
    }

    /// 折り返さずに 1 行を出力する(はみ出した分は表示側で切り詰められる)
    fn emit_row(&mut self, row: Row) {
        self.emit(|_| vec![row]);
    }

    /// 行頭の装飾を除いた幅を `layout` に渡し、返された行を出力する。
    /// 1 行目には未出力のマーカーを、2 行目以降には継続行の装飾を付ける。
    fn emit(&mut self, layout: impl FnOnce(usize) -> Vec<Row>) {
        let prefix: Row = self
            .containers
            .iter_mut()
            .map(|container| match container {
                Container::Item { marker, shown } if !*shown => {
                    *shown = true;
                    Span::raw(marker.clone())
                }
                container => container.indent(),
            })
            .collect();
        let mut rows = layout(self.width.saturating_sub(spans_width(&prefix))).into_iter();
        let Some(first) = rows.next() else {
            return;
        };
        self.lines.push(Line::from([prefix, first].concat()));
        let rest: Vec<Row> = rows.collect();
        if rest.is_empty() {
            return;
        }
        let continuation: Row = self.containers.iter().map(Container::indent).collect();
        for row in rest {
            self.lines
                .push(Line::from([continuation.clone(), row].concat()));
        }
    }

    fn render_table(&mut self, table: Table) {
        let Table { alignments, rows } = table;
        let widths: Vec<usize> = (0..alignments.len())
            .map(|col| {
                rows.iter()
                    .filter_map(|row| row.get(col))
                    .map(|cell| spans_width(cell))
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let border = |left: &str, mid: &str, right: &str| {
            let inner: Vec<String> = widths
                .iter()
                .map(|width| BORDER.horizontal.repeat(width + 2))
                .collect();
            vec![Span::styled(
                format!("{left}{}{right}", inner.join(mid)),
                BORDER_STYLE,
            )]
        };
        let top = border(BORDER.top_left, BORDER.horizontal_down, BORDER.top_right);
        let separator = border(BORDER.vertical_right, BORDER.cross, BORDER.vertical_left);
        let bottom = border(
            BORDER.bottom_left,
            BORDER.horizontal_up,
            BORDER.bottom_right,
        );

        self.emit_row(top);
        for (i, row) in rows.into_iter().enumerate() {
            let mut spans = vec![Span::styled(BORDER.vertical, BORDER_STYLE)];
            let mut cells = row.into_iter();
            for (width, alignment) in widths.iter().zip(&alignments) {
                let cell = cells.next().unwrap_or_default();
                let padding = width - spans_width(&cell);
                let (left, right) = match alignment {
                    Alignment::Right => (padding, 0),
                    Alignment::Center => (padding / 2, padding - padding / 2),
                    Alignment::Left | Alignment::None => (0, padding),
                };
                spans.push(Span::raw(" ".repeat(left + 1)));
                spans.extend(cell);
                spans.push(Span::raw(" ".repeat(right + 1)));
                spans.push(Span::styled(BORDER.vertical, BORDER_STYLE));
            }
            self.emit_row(spans);
            if i == 0 {
                self.emit_row(separator.clone());
            }
        }
        self.emit_row(bottom);
    }
}

/// コードの 1 行を背景付きで折り返す。折り返した行も含めて左に 1 桁の余白を付ける。
fn code_rows(line: &str, width: usize) -> Vec<Row> {
    let inner = width.saturating_sub(1);
    wrap(
        vec![Span::styled(line.to_string(), CODE_STYLE)],
        inner,
        false,
    )
    .into_iter()
    .map(|row| {
        let padding = inner.saturating_sub(spans_width(&row));
        let mut spans = vec![Span::styled(" ", CODE_STYLE)];
        spans.extend(row);
        spans.push(Span::styled(" ".repeat(padding), CODE_STYLE));
        spans
    })
    .collect()
}

fn spans_width(spans: &[Span]) -> usize {
    spans.iter().map(Span::width).sum()
}

/// `spans` を表示幅 `width` で折り返す。
/// `word_wrap` のときは空白の位置で区切り、区切り位置の前後の空白は捨てる。
/// 1 単語が収まらない場合や `word_wrap` でない場合は文字単位で区切る。
fn wrap(spans: Row, width: usize, word_wrap: bool) -> Vec<Row> {
    if width == 0 || spans_width(&spans) <= width {
        return vec![spans];
    }
    let is_space = |grapheme: &StyledGrapheme| word_wrap && grapheme.symbol == " ";
    let mut rows: Vec<Vec<StyledGrapheme>> = Vec::new();
    let mut row: Vec<StyledGrapheme> = Vec::new();
    let mut row_width = 0;
    let mut last_space = None;
    for grapheme in spans
        .iter()
        .flat_map(|span| span.styled_graphemes(Style::new()))
    {
        let grapheme_width = grapheme.symbol.width();
        // 空白は幅を超えても行末にぶら下げ、次の文字で改行する
        if !is_space(&grapheme) && row_width + grapheme_width > width && !row.is_empty() {
            let mut rest = last_space.map_or_else(Vec::new, |index| row.split_off(index));
            if word_wrap {
                trim_spaces_end(&mut row);
                rest.retain(|grapheme| !is_space(grapheme));
            }
            rows.push(std::mem::replace(&mut row, rest));
            row_width = row.iter().map(|grapheme| grapheme.symbol.width()).sum();
            last_space = None;
        }
        if is_space(&grapheme) {
            // 折り返した行の先頭の空白は捨てる
            if row.is_empty() && !rows.is_empty() {
                continue;
            }
            last_space = Some(row.len());
        }
        row.push(grapheme);
        row_width += grapheme_width;
    }
    rows.push(row);
    rows.into_iter().map(group_by_style).collect()
}

fn trim_spaces_end(row: &mut Vec<StyledGrapheme>) {
    while row.last().is_some_and(|grapheme| grapheme.symbol == " ") {
        row.pop();
    }
}

fn group_by_style(graphemes: Vec<StyledGrapheme>) -> Row {
    let mut spans: Row = Vec::new();
    for grapheme in graphemes {
        match spans.last_mut() {
            Some(span) if span.style == grapheme.style => {
                span.content.to_mut().push_str(grapheme.symbol);
            }
            _ => spans.push(Span::styled(grapheme.symbol.to_string(), grapheme.style)),
        }
    }
    spans
}

fn heading_style(level: HeadingLevel) -> Style {
    let style = Style::new().bold();
    match level {
        HeadingLevel::H1 | HeadingLevel::H2 => style.cyan(),
        HeadingLevel::H3 => style.green(),
        _ => style,
    }
}

/// 引用の縦線の色と、GitHub のアラート記法(`> [!NOTE]` など)の見出し
fn alert(kind: Option<BlockQuoteKind>) -> (Color, Option<&'static str>) {
    let (color, label) = match kind {
        None => return (Color::DarkGray, None),
        Some(BlockQuoteKind::Note) => (Color::Blue, "Note"),
        Some(BlockQuoteKind::Tip) => (Color::Green, "Tip"),
        Some(BlockQuoteKind::Important) => (Color::Magenta, "Important"),
        Some(BlockQuoteKind::Warning) => (Color::Yellow, "Warning"),
        Some(BlockQuoteKind::Caution) => (Color::Red, "Caution"),
    };
    (color, Some(label))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Modifier;

    fn plain(text: &Text) -> Vec<String> {
        text.lines.iter().map(Line::to_string).collect()
    }

    /// 指定した文字列を内容に持つ span のスタイルを返す
    fn style_of(text: &Text, content: &str) -> Style {
        text.lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == content)
            .unwrap_or_else(|| panic!("span not found: {content}"))
            .style
    }

    fn has_modifier(text: &Text, content: &str, modifier: Modifier) -> bool {
        style_of(text, content).add_modifier.contains(modifier)
    }

    #[test]
    fn empty_input_renders_nothing() {
        assert!(render("", 20).lines.is_empty());
    }

    #[test]
    fn paragraph_is_rendered_as_is() {
        assert_eq!(plain(&render("hello world", 20)), ["hello world"]);
    }

    #[test]
    fn soft_break_joins_lines_with_space() {
        assert_eq!(plain(&render("hello\nworld", 20)), ["hello world"]);
    }

    #[test]
    fn hard_break_starts_new_line() {
        assert_eq!(plain(&render("hello  \nworld", 20)), ["hello", "world"]);
    }

    #[test]
    fn paragraphs_are_separated_by_blank_line() {
        assert_eq!(plain(&render("a\n\nb", 20)), ["a", "", "b"]);
    }

    #[test]
    fn heading_hides_markers_and_is_bold() {
        let text = render("### Title", 20);
        assert_eq!(plain(&text), ["Title"]);
        assert!(has_modifier(&text, "Title", Modifier::BOLD));
    }

    #[test]
    fn h1_and_h2_are_underlined_with_rule() {
        assert_eq!(plain(&render("# A", 5)), ["A", "─────"]);
        assert_eq!(plain(&render("## B", 3)), ["B", "───"]);
    }

    #[test]
    fn heading_followed_by_paragraph_has_blank_line() {
        assert_eq!(plain(&render("### A\nb", 5)), ["A", "", "b"]);
    }

    #[test]
    fn inline_styles_are_applied() {
        let text = render("*i* **b** ~~s~~ `c`", 40);
        assert_eq!(plain(&text), ["i b s c"]);
        assert!(has_modifier(&text, "i", Modifier::ITALIC));
        assert!(has_modifier(&text, "b", Modifier::BOLD));
        assert!(has_modifier(&text, "s", Modifier::CROSSED_OUT));
        assert!(style_of(&text, "c").bg.is_some());
    }

    #[test]
    fn nested_inline_styles_are_combined() {
        let text = render("***x***", 20);
        assert!(has_modifier(&text, "x", Modifier::BOLD | Modifier::ITALIC));
    }

    #[test]
    fn unordered_list_uses_bullets() {
        assert_eq!(plain(&render("- a\n- b", 20)), ["• a", "• b"]);
    }

    #[test]
    fn ordered_list_keeps_start_number() {
        assert_eq!(plain(&render("3. a\n4. b", 20)), ["3. a", "4. b"]);
    }

    #[test]
    fn nested_list_is_indented() {
        assert_eq!(
            plain(&render("- a\n  - b\n- c", 20)),
            ["• a", "  • b", "• c"]
        );
    }

    #[test]
    fn task_list_uses_checkboxes() {
        assert_eq!(
            plain(&render("- [ ] todo\n- [x] done", 20)),
            ["☐ todo", "☑ done"]
        );
    }

    #[test]
    fn loose_list_item_continuation_is_indented() {
        assert_eq!(
            plain(&render("- a\n\n  more\n- b", 20)),
            ["• a", "", "  more", "", "• b"]
        );
    }

    #[test]
    fn list_is_separated_from_paragraphs() {
        assert_eq!(
            plain(&render("p\n\n- a\n\nq", 20)),
            ["p", "", "• a", "", "q"]
        );
    }

    #[test]
    fn blockquote_has_bar_prefix() {
        assert_eq!(plain(&render("> a\n>\n> b", 20)), ["▌ a", "▌ ", "▌ b"]);
    }

    #[test]
    fn list_inside_blockquote_keeps_both_prefixes() {
        assert_eq!(plain(&render("> - a", 20)), ["▌ • a"]);
    }

    #[test]
    fn alert_shows_label() {
        assert_eq!(
            plain(&render("> [!NOTE]\n> body", 20)),
            ["▌ Note", "▌ body"]
        );
    }

    #[test]
    fn code_block_hides_fences_and_fills_background() {
        let text = render("```rust\nfn x\n  y\n```", 8);
        assert_eq!(plain(&text), [" fn x   ", "   y    "]);
        for line in &text.lines {
            assert!(line.spans.iter().all(|span| span.style.bg.is_some()));
        }
    }

    #[test]
    fn code_block_keeps_blank_lines() {
        assert_eq!(plain(&render("```\na\n\nb\n```", 3)), [" a ", "   ", " b "]);
    }

    #[test]
    fn horizontal_rule_spans_width() {
        assert_eq!(
            plain(&render("a\n\n---\n\nb", 4)),
            ["a", "", "────", "", "b"]
        );
    }

    #[test]
    fn link_shows_text_with_link_style() {
        let text = render("[docs](https://example.com)", 20);
        assert_eq!(plain(&text), ["docs"]);
        let style = style_of(&text, "docs");
        assert_eq!(style.fg, Some(Color::Blue));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn image_shows_alt_text() {
        assert_eq!(plain(&render("![logo](a.png)", 20)), ["[image: logo]"]);
    }

    #[test]
    fn table_is_drawn_with_box_characters() {
        let src = "| Name | Age |\n|:--|--:|\n| foo | 1 |\n| barbaz | 10 |";
        assert_eq!(
            plain(&render(src, 40)),
            [
                "┌────────┬─────┐",
                "│ Name   │ Age │",
                "├────────┼─────┤",
                "│ foo    │   1 │",
                "│ barbaz │  10 │",
                "└────────┴─────┘",
            ]
        );
    }

    #[test]
    fn table_header_is_bold() {
        let text = render("| H |\n|---|\n| v |", 20);
        assert!(has_modifier(&text, "H", Modifier::BOLD));
        assert!(!has_modifier(&text, "v", Modifier::BOLD));
    }

    #[test]
    fn table_center_alignment() {
        let src = "| Head |\n|:-:|\n| a |";
        assert_eq!(plain(&render(src, 20))[3], "│  a   │");
    }

    #[test]
    fn table_measures_wide_characters() {
        let src = "| 名前 |\n|---|\n| a |";
        assert_eq!(
            plain(&render(src, 20)),
            ["┌──────┐", "│ 名前 │", "├──────┤", "│ a    │", "└──────┘"]
        );
    }

    #[test]
    fn item_starting_with_list_keeps_outer_marker() {
        assert_eq!(plain(&render("- - a\n  - b", 20)), ["• • a", "  • b"]);
        assert_eq!(plain(&render("1. - a", 20)), ["1. • a"]);
    }

    #[test]
    fn long_text_wraps_with_prefix() {
        assert_eq!(
            plain(&render("- long text that wraps", 11)),
            ["• long text", "  that", "  wraps"]
        );
        assert_eq!(plain(&render("> aaa bbb", 7)), ["▌ aaa", "▌ bbb"]);
    }

    #[test]
    fn long_word_is_broken_by_characters() {
        assert_eq!(plain(&render("abcdefg", 3)), ["abc", "def", "g"]);
    }

    #[test]
    fn wrapping_keeps_styles() {
        let text = render("aa **bb**", 3);
        assert_eq!(plain(&text), ["aa", "bb"]);
        assert!(has_modifier(&text, "bb", Modifier::BOLD));
    }

    #[test]
    fn wide_table_is_not_wrapped() {
        let lines = plain(&render("| aaaaaa | bbbbbb |\n|---|---|\n| x | y |", 12));
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], "┌────────┬────────┐");
    }

    #[test]
    fn loose_list_separates_items_starting_with_code_block() {
        assert_eq!(
            plain(&render("- a\n\n- ```\n  x\n  ```", 6)),
            ["• a", "", "•  x  "]
        );
    }

    #[test]
    fn tight_list_does_not_separate_items_starting_with_code_block() {
        assert_eq!(
            plain(&render("- a\n- ```\n  x\n  ```", 6)),
            ["• a", "•  x  "]
        );
    }

    #[test]
    fn ordered_task_list_keeps_number() {
        assert_eq!(
            plain(&render("1. [x] t\n\n   more", 20)),
            ["1. ☑ t", "", "     more"]
        );
    }

    #[test]
    fn empty_code_block_is_shown() {
        assert_eq!(plain(&render("```\n```", 3)), ["   "]);
    }

    #[test]
    fn empty_blockquote_is_shown() {
        assert_eq!(plain(&render("a\n\n>\n\nb", 20)), ["a", "", "▌ ", "", "b"]);
    }

    #[test]
    fn empty_list_item_shows_marker() {
        assert_eq!(plain(&render("- a\n-", 20)), ["• a", "• "]);
    }

    #[test]
    fn nested_blockquote_has_two_bars() {
        assert_eq!(plain(&render("> > a", 20)), ["▌ ▌ a"]);
    }

    #[test]
    fn blocks_inside_list_item_are_indented() {
        assert_eq!(
            plain(&render("- a\n\n  > q\n\n  ```\n  c\n  ```", 6)),
            ["• a", "", "  ▌ q", "", "   c  "]
        );
    }

    #[test]
    fn table_inside_blockquote_has_bar() {
        assert_eq!(
            plain(&render("> | h |\n> |---|\n> | v |", 20)),
            ["▌ ┌───┐", "▌ │ h │", "▌ ├───┤", "▌ │ v │", "▌ └───┘"]
        );
    }

    #[test]
    fn table_fills_missing_cells() {
        assert_eq!(
            plain(&render("| a | b |\n|---|---|\n| x |", 20))[3],
            "│ x │   │"
        );
    }

    #[test]
    fn alert_kinds_have_colors() {
        let text = render("> [!WARNING]\n> w", 20);
        assert_eq!(style_of(&text, "Warning").fg, Some(Color::Yellow));
        let text = render("> [!CAUTION]\n> c", 20);
        assert_eq!(style_of(&text, "Caution").fg, Some(Color::Red));
    }

    #[test]
    fn html_block_is_shown_raw() {
        assert_eq!(
            plain(&render("<details>\n<summary>s</summary>\n</details>", 30)),
            ["<details>", "<summary>s</summary>", "</details>"]
        );
    }

    #[test]
    fn zero_width_does_not_panic() {
        let src = "# h\n\n- a\n\n> q\n\n```\nc\n```\n\n---\n\n| t |\n|---|\n| v |";
        render(src, 0);
    }

    #[test]
    fn alert_without_body_shows_only_label() {
        assert_eq!(plain(&render("> [!NOTE]", 20)), ["▌ Note"]);
    }

    #[test]
    fn wrapped_line_does_not_start_with_spaces() {
        assert_eq!(plain(&render("abcde  fgh", 5)), ["abcde", "fgh"]);
    }

    #[test]
    fn wrapped_code_line_keeps_left_margin() {
        assert_eq!(
            plain(&render("```\nabcdefghijklmn\n```", 6)),
            [" abcde", " fghij", " klmn "]
        );
    }

    #[test]
    fn line_wrapped_at_spaces_does_not_end_with_spaces() {
        assert_eq!(plain(&render("ab  cdef", 5)), ["ab", "cdef"]);
        assert_eq!(plain(&render("ab  c", 3)), ["ab", "c"]);
        assert_eq!(plain(&render("abc   def", 4)), ["abc", "def"]);
    }
}
