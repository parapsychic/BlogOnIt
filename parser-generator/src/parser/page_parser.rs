extern crate nom;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_until1, take_while, take_while1};
use nom::character::complete::{char, digit1, line_ending, multispace0, newline, space0, space1};
use nom::character::{is_alphanumeric, is_newline, is_space};
use nom::combinator::{map,  opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use super::markdown_elements::MarkdownElement;

fn is_part_of_url(c: char) -> bool {
    ['/', '?', '#', '.', '-', ':', '_', '@', '+', '&', '='].contains(&c) || is_alphanumeric(c as u8)
}

fn special(c: char) -> bool {
    ['\\', '_', '*', '['].contains(&c)
}

fn toc_marker_parser(input: &str) -> IResult<&str, MarkdownElement> {
   let (remaining, _)  = delimited(tag("%-%"), tag("TOC"), tag("%-%"))(input)?;

   Ok((remaining, MarkdownElement::TocMarker))
}


fn header_parser(input: &str) -> IResult<&str, MarkdownElement> {
    // recognise the '#' pattern
    let (remaining, level) = recognize(tuple((char('#'), many0(char('#')))))(input)?;

    // recognise spaces only, not newlines
    let (remaining, _) = many1(space1)(remaining)?;
    let (remaining, heading) = recognize(take_until("\n"))(remaining)?;

    Ok((
        remaining,
        MarkdownElement::Heading(level.len(), heading.to_string()),
    ))
}

fn unordered_list_item_parser(input: &str) -> IResult<&str, MarkdownElement> {
    // recognize the indentation
    let (remaining, level) = recognize(space0)(input)?;

    // recognize the '-' pattern
    let (remaining, _) = recognize(tuple((
        alt((char('-'), char('*'), char('+'))),
        many1(space1),
    )))(remaining)?;
    let (remaining, list_item) = paragraph_parser(remaining)?;
    let (remaining, _) = newline(remaining)?;
    Ok((
        remaining,
        MarkdownElement::UnorderedList(level.len() + 1, Box::new(list_item)),
    ))
}

fn ordered_list_item_parser(input: &str) -> IResult<&str, MarkdownElement> {
    // recognize the indentation
    let (remaining, level) = recognize(space0)(input)?;

    // recognize patterns like '1.'
    let (remaining, _) = recognize(tuple((digit1, char('.'), many1(space1))))(remaining)?;
    let (remaining, list_item) = paragraph_parser(remaining)?;
    let (remaining, _) = newline(remaining)?;
    Ok((
        remaining,
        MarkdownElement::OrderedList(level.len() + 1, Box::new(list_item)),
    ))
}

// uses the obsidian syntax, no alt text
fn obsidian_image_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, image_url) = delimited(
        tuple((space0, tag("![["), space0)),
        take_while1(|c: char| is_part_of_url(c)),
        tuple((space0, tag("]]"), multispace0)),
    )(input)?;

    Ok((
        remaining,
        MarkdownElement::Image(String::new(), image_url.to_string()), // no alt text
    ))
}

// uses standard markdown syntax
fn image_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, alt_text) = delimited(
        tuple((space0, tag("!["), space0)),
        map(
            many1(alt((
                escape_parser,
                take_while1(|c: char| c != ']' && c != '\\'),
            ))),
            |x| x.join(""),
        ),
        tuple((space0, char(']'))),
    )(input)?;

    let (remaining, image_url) = delimited(
        char('('),
        take_while1(|c: char| is_part_of_url(c)),
        tuple((space0, char(')'), multispace0)),
    )(remaining)?;

    Ok((
        remaining,
        MarkdownElement::Image(alt_text.to_string(), image_url.to_string()), // no alt text
    ))
}

fn wikilink_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, link) = delimited(
        tuple((space0, tag("[["), space0)),
        take_while1(|c: char| is_part_of_url(c) || is_space(c as u8) || c == '|'),
        tuple((space0, tag("]]"))),
    )(input)?;

    let (x, url) = take_while(|c: char| c != '|')(link)?;
    let (alt_text, res) = opt(tuple((char('|'), space0)))(x)?;
    // if res, means | was found and something exists after that. if whitespace, that's the
    // writer's problem.
    if let Some(_) = res {
        return Ok((
            remaining,
            MarkdownElement::WikiLink(alt_text.trim().to_string(), url.trim().to_string()), // no alt text
        ));
    }

    Ok((
        remaining,
        MarkdownElement::WikiLink(url.trim().to_string(), url.trim().to_string()), // no alt text
    ))
}

fn hyperlink_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, alt_text) = delimited(
        tuple((char('['), space0)),
        map(
            many1(alt((
                escape_parser,
                take_while1(|c: char| c != ']' && c != '\\'),
            ))),
            |x| x.join(""),
        ),
        tuple((space0, char(']'))),
    )(input)?;

    let (remaining, url) = delimited(
        char('('),
        take_while1(|c: char| is_part_of_url(c)),
        tuple((space0, char(')'), multispace0)),
    )(remaining)?;
    Ok((
        remaining,
        MarkdownElement::Hyperlink(alt_text.trim().to_string(), url.to_string()), // no alt text
    ))
}

fn codeblock_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, language) = delimited(
        tag("```"),
        take_while1(|c: char| !is_space(c as u8) && !is_newline(c as u8)),
        // ignore anything after a space till a new line
        tuple((take_until("\n"), multispace0)),
    )(input)?;

    let (remaining, text) = terminated(take_until1("\n```"), tag("\n```"))(remaining)?;

    Ok((
        remaining,
        MarkdownElement::Codeblock(language.to_string(), text.to_string()),
    ))
}

fn bold_italics_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = alt((
        delimited(tag("***"), take_until("***"), tag("***")),
        delimited(tag("___"), take_until("___"), tag("___")),
        delimited(tag("*__"), take_until("__*"), tag("__*")),
        delimited(tag("__*"), take_until("*__"), tag("*__")),
        delimited(tag("_**"), take_until("**_"), tag("**_")),
        delimited(tag("**_"), take_until("_**"), tag("_**")),
    ))(input)?;

    Ok((remaining, MarkdownElement::BoldItalics(text.to_string())))
}

fn inline_code_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = delimited(tag("`"), take_until("`"), tag("`"))(input)?;

    Ok((remaining, MarkdownElement::InlineCode(text.to_string())))
}

fn bold_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = alt((
        delimited(tag("**"), take_until("**"), tag("**")),
        delimited(tag("__"), take_until("__"), tag("__")),
    ))(input)?;

    Ok((remaining, MarkdownElement::Bold(text.to_string())))
}

fn new_line_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, _) = line_ending(input)?;

    Ok((remaining, MarkdownElement::Newline))
}

fn italics_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = alt((
        delimited(tag("*"), take_until("*"), tag("*")),
        delimited(tag("_"), take_until("_"), tag("_")),
    ))(input)?;

    Ok((remaining, MarkdownElement::Italics(text.to_string())))
}

fn escape_parser(input: &str) -> IResult<&str, &str> {
    let (remaining, escaped) = preceded(
        tag("\\"),
        alt((tag("*"), tag("_"), tag("-"), tag("```"), tag("`"))),
    )(input)?;

    Ok((remaining, escaped))
}

fn plain_text_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = many1(alt((
        escape_parser,
        take_while1(|c: char| !is_space(c as u8) && !is_newline(c as u8) && !special(c)),
    )))(input)?;
    Ok((remaining, MarkdownElement::PlainText(text.join(""))))
}

fn whitespace_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = space1(input)?;
    Ok((remaining, MarkdownElement::Whitespace(text.to_string())))
}

fn paragraph_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, text) = take_until1("\n")(input)?;
    let (_, text) = many1(alt((
        obsidian_image_parser,
        image_parser,
        wikilink_parser,
        hyperlink_parser,
        inline_code_parser,
        bold_italics_parser,
        bold_parser,
        italics_parser,
        whitespace_parser,
        plain_text_parser,
    )))(text)?;

    Ok((remaining, MarkdownElement::Paragraph(text)))
}

fn horizontal_line_parser(input: &str) -> IResult<&str, MarkdownElement> {
    let (remaining, _) = alt((
        terminated(tag("---"), pair(take_while(|c: char| c == '-'), char('\n'))),
        terminated(tag("___"), pair(take_while(|c: char| c == '_'), char('\n'))),
    ))(input)?;

    Ok((remaining, MarkdownElement::HorizontalLine))
}

pub fn parse_page(page: &str) -> Result<Vec<MarkdownElement>, String> {
    let markdown = format!("{}\n", page);
    let (remaining, parsed) = many1(alt((
        toc_marker_parser,
        header_parser,
        ordered_list_item_parser,
        unordered_list_item_parser,
        codeblock_parser,
        new_line_parser,
        horizontal_line_parser,
        paragraph_parser,
    )))(&markdown)
    .unwrap();

    if remaining != "" {
        return Err(format!("Could not parse string: {}", remaining));
    }


    Ok(parsed)
}
