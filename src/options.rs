/// Configuration options for the CEL formatter
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Maximum line width before wrapping
    pub max_width: usize,

    /// Number of spaces per indentation level
    pub indent_width: usize,

    /// Use spaces for indentation (vs tabs)
    pub use_spaces: bool,

    /// Add trailing commas in multi-line lists/maps
    pub trailing_comma: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            max_width: 80,
            indent_width: 2,
            use_spaces: true,
            trailing_comma: true,
        }
    }
}

impl FormatOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    pub fn with_indent_width(mut self, width: usize) -> Self {
        self.indent_width = width;
        self
    }

    pub fn with_tabs(mut self) -> Self {
        self.use_spaces = false;
        self
    }

    pub fn with_trailing_comma(mut self, enabled: bool) -> Self {
        self.trailing_comma = enabled;
        self
    }
}
