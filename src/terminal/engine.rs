pub struct TerminalEngine {
    parser: vt100::Parser,
}

impl TerminalEngine {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, cols, 0),
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    pub fn render_text(&self) -> String {
        self.parser.screen().contents().to_owned()
    }
}