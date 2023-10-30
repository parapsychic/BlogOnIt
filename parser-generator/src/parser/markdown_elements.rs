pub type Level = usize;
pub type Link = String;
pub type Text = String;
pub type LanguageHint = String;

#[derive(Clone )]
pub enum MarkdownElement {
    TocMarker,
    Heading(Level, Text), // level, text
    OrderedList(Level, Box<MarkdownElement>),
    UnorderedList(Level, Box<MarkdownElement>),
    Image(Text, Link),
    Hyperlink(Text, Link),           // text, link
    WikiLink(Text, Link),            // text, link
    Codeblock(LanguageHint, String), // language, text
    BlockQuote(String),
    InlineCode(String),
    HorizontalLine,
    Newline,
    Whitespace(String), // whatever whitespace it is, we take it
    Italics(Text),
    Bold(Text),
    BoldItalics(Text),
    PlainText(Text),
    Paragraph(Vec<MarkdownElement>), // this might be tricky, with bold and italics
                                     // stuff that could be added
}
