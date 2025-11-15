/// A pretty-printer document representation
/// Inspired by Wadler's "A prettier printer" and Prettier.js
#[derive(Debug, Clone)]
pub enum Doc {
    /// Empty document
    Nil,

    /// A string literal (must not contain newlines)
    Text(String),

    /// Concatenation of documents
    Concat(Vec<Doc>),

    /// A line break (space in flat mode, newline in break mode)
    Line,

    /// A line break that becomes nothing in flat mode
    SoftLine,

    /// Increase indentation level for the inner doc
    Indent(Box<Doc>),

    /// A group - tries to fit on one line, breaks if it doesn't fit
    Group(Box<Doc>),

    /// If-break: first doc if breaking, second if flat
    IfBreak {
        break_doc: Box<Doc>,
        flat_doc: Box<Doc>,
    },
}

impl Doc {
    /// Create an empty document
    pub fn nil() -> Self {
        Doc::Nil
    }

    /// Create a text document
    pub fn text<S: Into<String>>(s: S) -> Self {
        Doc::Text(s.into())
    }

    /// Create a line break
    pub fn line() -> Self {
        Doc::Line
    }

    /// Create a soft line break (disappears when group fits)
    pub fn soft_line() -> Self {
        Doc::SoftLine
    }

    /// Concatenate documents
    pub fn concat<I: IntoIterator<Item = Doc>>(docs: I) -> Self {
        Doc::Concat(docs.into_iter().collect())
    }

    /// Indent a document
    pub fn indent(doc: Doc) -> Self {
        Doc::Indent(Box::new(doc))
    }

    /// Create a group
    pub fn group(doc: Doc) -> Self {
        Doc::Group(Box::new(doc))
    }

    /// If-break combinator
    pub fn if_break(break_doc: Doc, flat_doc: Doc) -> Self {
        Doc::IfBreak {
            break_doc: Box::new(break_doc),
            flat_doc: Box::new(flat_doc),
        }
    }

    /// Join documents with a separator
    pub fn join(docs: Vec<Doc>, sep: Doc) -> Self {
        if docs.is_empty() {
            return Doc::nil();
        }

        let mut result = Vec::new();
        for (i, doc) in docs.into_iter().enumerate() {
            if i > 0 {
                result.push(sep.clone());
            }
            result.push(doc);
        }
        Doc::concat(result)
    }

    /// Join with commas and optional trailing comma
    pub fn join_comma(docs: Vec<Doc>, trailing: bool) -> Self {
        if docs.is_empty() {
            return Doc::nil();
        }

        let len = docs.len();
        let mut result = Vec::new();
        for (i, doc) in docs.into_iter().enumerate() {
            result.push(doc);
            if i < len - 1 {
                result.push(Doc::text(","));
                result.push(Doc::line());
            } else if trailing {
                result.push(Doc::if_break(Doc::text(","), Doc::nil()));
            }
        }
        Doc::concat(result)
    }

    /// Render the document to a string
    pub fn render(&self, max_width: usize, indent_str: &str) -> String {
        let mut buffer = String::new();
        self.render_impl(&mut buffer, max_width, indent_str, 0, Mode::Flat);
        buffer
    }

    fn render_impl(
        &self,
        buffer: &mut String,
        max_width: usize,
        indent_str: &str,
        indent_level: usize,
        mode: Mode,
    ) {
        match self {
            Doc::Nil => {}

            Doc::Text(s) => buffer.push_str(s),

            Doc::Concat(docs) => {
                for doc in docs {
                    doc.render_impl(buffer, max_width, indent_str, indent_level, mode);
                }
            }

            Doc::Line => match mode {
                Mode::Flat => buffer.push(' '),
                Mode::Break => {
                    buffer.push('\n');
                    for _ in 0..indent_level {
                        buffer.push_str(indent_str);
                    }
                }
            },

            Doc::SoftLine => match mode {
                Mode::Flat => {}
                Mode::Break => {
                    buffer.push('\n');
                    for _ in 0..indent_level {
                        buffer.push_str(indent_str);
                    }
                }
            },

            Doc::Indent(doc) => {
                doc.render_impl(buffer, max_width, indent_str, indent_level + 1, mode);
            }

            Doc::Group(doc) => {
                // Try flat mode first
                let mut flat_buffer = String::new();
                doc.render_impl(
                    &mut flat_buffer,
                    max_width,
                    indent_str,
                    indent_level,
                    Mode::Flat,
                );

                // Check if it fits on current line
                let current_line_len = buffer.lines().last().map(|l| l.len()).unwrap_or(0);
                let fits = current_line_len + flat_buffer.len() <= max_width
                    && !flat_buffer.contains('\n');

                if fits {
                    buffer.push_str(&flat_buffer);
                } else {
                    doc.render_impl(buffer, max_width, indent_str, indent_level, Mode::Break);
                }
            }

            Doc::IfBreak {
                break_doc,
                flat_doc,
            } => match mode {
                Mode::Break => {
                    break_doc.render_impl(buffer, max_width, indent_str, indent_level, mode)
                }
                Mode::Flat => {
                    flat_doc.render_impl(buffer, max_width, indent_str, indent_level, mode)
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

// Helper functions for common patterns
impl Doc {
    /// Wrap in parentheses
    pub fn parens(doc: Doc) -> Self {
        Doc::concat(vec![Doc::text("("), doc, Doc::text(")")])
    }

    /// Wrap in brackets
    pub fn brackets(doc: Doc) -> Self {
        Doc::concat(vec![Doc::text("["), doc, Doc::text("]")])
    }

    /// Wrap in braces
    pub fn braces(doc: Doc) -> Self {
        Doc::concat(vec![Doc::text("{"), doc, Doc::text("}")])
    }

    /// Wrap with possible line breaks inside
    pub fn wrap_parens(doc: Doc) -> Self {
        Doc::group(Doc::concat(vec![
            Doc::text("("),
            Doc::indent(Doc::concat(vec![Doc::soft_line(), doc])),
            Doc::soft_line(),
            Doc::text(")"),
        ]))
    }

    /// Wrap list with brackets
    pub fn wrap_brackets(doc: Doc) -> Self {
        Doc::group(Doc::concat(vec![
            Doc::text("["),
            Doc::indent(Doc::concat(vec![Doc::soft_line(), doc])),
            Doc::soft_line(),
            Doc::text("]"),
        ]))
    }

    /// Wrap map with braces
    pub fn wrap_braces(doc: Doc) -> Self {
        Doc::group(Doc::concat(vec![
            Doc::text("{"),
            Doc::indent(Doc::concat(vec![Doc::soft_line(), doc])),
            Doc::soft_line(),
            Doc::text("}"),
        ]))
    }
}
