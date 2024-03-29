use std::io::{stdout, Stdout, Write};
use termimad::crossterm::{cursor, ExecutableCommand};
use termimad::crossterm::terminal::Clear;
use termimad::crossterm::terminal::ClearType::FromCursorDown;
use termimad::{FmtLine, FmtText, MadSkin};

#[derive(Debug, Clone)]
struct RenderedMarkdown {
    text: String,
    line_width: Vec<usize>,
}

impl From<FmtText<'_, '_>> for RenderedMarkdown {
    fn from(fmt_text: FmtText<'_, '_>) -> Self {
        let text = format!("{}", fmt_text);
        let line_width = fmt_text.lines.iter().map(FmtLine::visible_length).collect();
        Self {
            text,
            line_width,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WrapWidth {
    None,
    Width(usize),
    AutoFitTerminalWidth,
}

impl Into<Option<usize>> for WrapWidth {
    fn into(self) -> Option<usize> {
        match self {
            WrapWidth::None => None,
            WrapWidth::Width(width) => Some(width),
            WrapWidth::AutoFitTerminalWidth => {
                let (width, _) = termimad::terminal_size();
                Some(width as usize)
            }
        }
    }
}

impl From<Option<usize>> for WrapWidth {
    fn from(width: Option<usize>) -> Self {
        match width {
            None => WrapWidth::None,
            Some(width) => WrapWidth::Width(width),
        }
    }
}

impl From<usize> for WrapWidth {
    fn from(width: usize) -> Self {
        WrapWidth::Width(width)
    }
}

#[derive(Debug)]
pub struct AnchoredMarkdownPrinter {
    pub skin: MadSkin,
    pub wrap_width: WrapWidth,
    stdout: Stdout,
    cursor_anchor: Option<(u16, u16)>,
    activated: bool,
    hide_cursor: bool,
}

impl Default for AnchoredMarkdownPrinter {
    fn default() -> Self {
        Self {
            skin: MadSkin::default(),
            wrap_width: WrapWidth::None,
            stdout: stdout(),
            cursor_anchor: None,
            activated: false,
            hide_cursor: false,
        }
    }
}

impl AnchoredMarkdownPrinter {
    pub fn hide_cursor(&mut self, hide_cursor: bool) {
        if self.hide_cursor && !hide_cursor {
            self.stdout.execute(cursor::Show).unwrap();
        } else if !self.hide_cursor && hide_cursor {
            self.stdout.execute(cursor::Hide).unwrap();
        }
        self.hide_cursor = hide_cursor;
    }

    pub fn activated(&self) -> bool {
        self.activated
    }

    pub fn activate(&mut self, hide_cursor: bool) {
        if self.activated {
            eprintln!("AnchoredMarkdownPrinter is already activated");
            return;
        }
        self.activated = true;
        self.set_anchor();
        if hide_cursor {
            self.stdout.execute(cursor::Hide).unwrap();
        }
        self.hide_cursor = hide_cursor;
    }

    pub fn set_anchor_with(&mut self, anchor_position: (u16, u16)) {
        assert!(self.activated, "AnchoredMarkdownPrinter must be activated before anchoring cursor");
        self.cursor_anchor = Some(anchor_position);
    }

    pub fn set_anchor(&mut self) {
        self.set_anchor_with(cursor::position().unwrap());
    }

    pub fn deactivate(&mut self) {
        if !self.activated {
            eprintln!("AnchoredMarkdownPrinter is already deactivated");
            return;
        }
        self.activated = false;
        self.cursor_anchor = None;
        if self.hide_cursor {
            stdout().execute(cursor::Show).unwrap();
        }
    }


    pub fn print(&mut self, partial_markdown: &str) {
        let rendered_markdown = FmtText::from(&self.skin,
                                              partial_markdown,
                                              self.wrap_width.into()).into();
        self.print_rendered(&rendered_markdown);
    }

    fn print_rendered(&mut self, rendered_markdown: &RenderedMarkdown) {
        assert!(self.activated, "AnchoredMarkdownPrinter must be activated before printing");
        let cursor_anchor = self.cursor_anchor.unwrap();
        // restore cursor position to anchor
        self.stdout
            .execute(cursor::MoveTo(cursor_anchor.0, cursor_anchor.1)).unwrap()
            .execute(Clear(FromCursorDown)).unwrap(); // clear previous output
        let rows = rendered_markdown.line_width.len() as u16;
        let columns = rendered_markdown.line_width.last().copied().unwrap_or(0) as u16;
        write!(self.stdout, "{}", rendered_markdown.text).unwrap();
        self.stdout.flush().unwrap();
        // update cursor anchor
        // the cursor position is relative to the terminal not the screen/history, so the anchor "floats/drifts" when a scrollbar appears.
        let (mut new_col, mut new_row) = cursor::position().unwrap();
        if new_col > columns {
            new_col -= columns;
        } else {
            new_col = 0;
        }
        if new_row > rows {
            new_row -= rows;
        } else {
            new_row = 0;
        }
        self.set_anchor_with((new_col, new_row));
    }
}

impl Drop for AnchoredMarkdownPrinter {
    fn drop(&mut self) {
        if self.activated {
            self.deactivate();
        }
    }
}

#[derive(Debug)]
pub struct IncrementalMarkdownPrinter {
    anchored_printer: AnchoredMarkdownPrinter,
    markdown_string_buffer: String,
    buffer_changed: bool,
    rendered_string_cache: Option<RenderedMarkdown>,
}

impl Default for IncrementalMarkdownPrinter {
    fn default() -> Self {
        Self {
            anchored_printer: AnchoredMarkdownPrinter::default(),
            markdown_string_buffer: String::new(),
            buffer_changed: false,
            rendered_string_cache: None,
        }
    }
}

impl IncrementalMarkdownPrinter {
    pub fn hide_cursor(&mut self, hide_cursor: bool) {
        self.anchored_printer.hide_cursor(hide_cursor);
    }

    pub fn with_skin(mut self, skin: MadSkin) -> Self {
        assert!(!self.activated(), "IncrementalMarkdownPrinter must be deactivated before changing skin");
        self.anchored_printer.skin = skin;
        self
    }

    pub fn with_wrap_width(mut self, wrap_width: WrapWidth) -> Self {
        assert!(!self.activated(), "IncrementalMarkdownPrinter must be deactivated before changing wrap width");
        self.anchored_printer.wrap_width = wrap_width;
        self
    }

    pub fn set_skin(&mut self, skin: MadSkin) {
        self.anchored_printer.skin = skin;
        if self.activated() {
            self.buffer_changed = true;
        }
    }

    pub fn set_wrap_width(&mut self, wrap_width: WrapWidth) {
        self.anchored_printer.wrap_width = wrap_width;
        if self.activated() {
            self.buffer_changed = true;
        }
    }

    pub fn activate(&mut self, hide_cursor: bool) {
        self.anchored_printer.activate(hide_cursor);
    }

    pub fn activated(&self) -> bool {
        self.anchored_printer.activated()
    }

    pub fn deactivate(&mut self) {
        self.anchored_printer.deactivate();
    }

    pub fn push_str(&mut self, chunk: &str) {
        assert!(self.activated(), "IncrementalMarkdownPrinter must be activated before push_str");
        self.markdown_string_buffer.push_str(chunk);
        self.buffer_changed = true
    }

    pub fn push_and_print(&mut self, chunk: &str) {
        self.push_str(chunk);
        self.print()
    }

    pub fn print(&mut self) {
        assert!(self.activated(), "IncrementalMarkdownPrinter must be activated before printing");
        if self.buffer_changed {
            let rendered = FmtText::from(&self.anchored_printer.skin,
                                         &self.markdown_string_buffer,
                                         self.anchored_printer.wrap_width.into()).into();
            self.rendered_string_cache = Some(rendered);
            self.buffer_changed = false;
        }
        if let Some(renderer_cache) = &self.rendered_string_cache {
            self.anchored_printer.print_rendered(renderer_cache);
        }
    }
}
