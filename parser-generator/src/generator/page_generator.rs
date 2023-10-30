use crate::{parser::markdown_elements::MarkdownElement, utils::remove_all_special};

use super::{Html,  toc_generator::generate_toc};

pub fn generate_page(tokens: Vec<MarkdownElement> ) -> Result<(String, Html), String> {

    let mut title = String::new();
    tokens.clone().iter().find(|&x| match x {
            MarkdownElement::Heading(1, heading) => {
                title = heading.to_string();
                true
            },
            _ => false,
        });
    let toc = generate_toc(tokens.clone());

    let mut token_iter = tokens.into_iter().peekable();
    let mut page_elements = Vec::new();

    while let Some(item) = token_iter.next() {
        let new_item = match item {
            MarkdownElement::TocMarker => {
                match &toc{
                    Ok(x) => x.to_string(),
                    Err(_) => return Err("A table of contents was requested, but could not be generated".to_string()),
                }
            }
            MarkdownElement::Heading(level, text) => {
                heading_generator(level, &text)
            }
            MarkdownElement::OrderedList(level, text) => {
                let initial_level = level;
                let mut accumulator = vec![MarkdownElement::OrderedList(level, text)];

                // we take all list-like items above the initial level and send it to a generator
                while let Some(x) = token_iter.peek() {
                    match x {
                        MarkdownElement::OrderedList(l, _) => {
                            if *l >= initial_level {
                                accumulator.push(x.clone());
                            } else {
                                break;
                            }
                            token_iter.next();
                        }
                        MarkdownElement::UnorderedList(l, _) => {
                            if *l > initial_level {
                                accumulator.push(x.clone());
                            } else {
                                break;
                            }
                            token_iter.next();
                        }
                        _ => break,
                    }
                }

                complete_list_generator(accumulator)
            }
            MarkdownElement::UnorderedList(level, text) => {
                let initial_level = level;
                let mut accumulator = vec![MarkdownElement::UnorderedList(level, text)];

                // we take all list-like items above the initial level and send it to a generator
                while let Some(x) = token_iter.peek() {
                    match x {
                        MarkdownElement::OrderedList(l, _) => {
                            if *l > initial_level {
                                accumulator.push(x.clone());
                            } else {
                                break;
                            }
                            token_iter.next();
                        }
                        MarkdownElement::UnorderedList(l, _) => {
                            if *l >= initial_level {
                                accumulator.push(x.clone());
                            } else {
                                break;
                            }
                            token_iter.next();
                        }
                        _ => break,
                    }
                }

                complete_list_generator(accumulator)
            }
            MarkdownElement::Codeblock(language, code) => code_generator(language, code),
            MarkdownElement::BlockQuote(_) => String::new(),
            MarkdownElement::InlineCode(_) => String::new(),
            MarkdownElement::HorizontalLine => String::from("<hr>"),
            MarkdownElement::Newline => String::from("\n"),
            MarkdownElement::Whitespace(space) => space,
            MarkdownElement::Paragraph(paragraph) => {
                format!("<p>{}</p>", paragraph_generator(paragraph))
            }
            _ => String::new(),
        };
        // join new item to html
        page_elements.push(new_item);
    }

    let html = page_elements.join("\n");

    Ok((title.clone(), format!(
        "<!DOCTYPE html>
<html lang=\"en\">

<head>
  <title>{}</title>
  <meta charset=\"UTF-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <link href=\"css/style.css\" rel=\"stylesheet\">
</head>
{}
<body>
</body>
</html>",
        title, html
    )))
}

fn heading_generator(level: usize, text: &str) -> Html {
    format!(
        "<h{} id={} class=main-h{}>{}</h{}>",
        level,
        remove_all_special(text),
        level,
        text,
        level,
    )
}

fn li_generator(level: usize, text: MarkdownElement, add_ordered_classname: bool) -> Html {
    if let MarkdownElement::Paragraph(tokens) = text {
        format!(
            "<li class={}-list-{}>{}</li>",
            if add_ordered_classname {
                "ordered"
            } else {
                "unordered"
            },
            level,
            paragraph_generator(tokens),
        )
    } else {
        panic!("Cannot parse list since list text could not be parsed");
    }
}

fn complete_list_generator(tokens: Vec<MarkdownElement>) -> Html {
    let mut stack = Vec::new();

    let mut level = 0;
    let mut lists = tokens
        .iter()
        .map(|x| match x {
            MarkdownElement::OrderedList(l, t) => match level.cmp(l) {
                std::cmp::Ordering::Less => {
                    stack.push("</ol>".to_string());
                    level = *l;
                    format!("<ol>{}", li_generator(*l, ((**t).clone()).clone(), true))
                }
                std::cmp::Ordering::Equal => {
                    println!("An equal ordering was found");
                    if let Some(list_closer) = stack.last() {
                        if list_closer == "</ul>" {
                            stack.pop().unwrap();
                            stack.push("</ol>".to_string());
                            level = *l;
                            return format!(
                                "</ul><ol>{}",
                                li_generator(*l, ((**t).clone()).clone(), true)
                            );
                        }
                    }
                    li_generator(*l, ((**t).clone()).clone(), true)
                }
                std::cmp::Ordering::Greater => {
                    let last_list = stack.pop().unwrap();
                    stack.push("</ol>".to_string());
                    level = *l;
                    format!("{}<ol>{}", last_list, li_generator(*l, (**t).clone(), true))
                }
            },
            MarkdownElement::UnorderedList(l, t) => match level.cmp(l) {
                std::cmp::Ordering::Less => {
                    stack.push("</ul>".to_string());
                    level = *l;
                    format!("<ul>{}", li_generator(*l, (**t).clone(), false))
                }
                std::cmp::Ordering::Equal => {
                    println!("An equal ordering was found");
                    if let Some(list_closer) = stack.last() {
                        if list_closer == "</ol>" {
                            stack.pop().unwrap();
                            stack.push("</ul>".to_string());
                            level = *l;
                            return format!("</ol><ul>{}", li_generator(*l, (**t).clone(), false));
                        }
                    }
                    li_generator(*l, (**t).clone(), false)
                }
                std::cmp::Ordering::Greater => {
                    let last_list = stack.pop().unwrap();
                    stack.push("</ul>".to_string());
                    level = *l;
                    format!(
                        "{}<ul>{}",
                        last_list,
                        li_generator(*l, (**t).clone(), false)
                    )
                }
            },
            _ => "".to_string(),
        })
        .collect::<Vec<Html>>();

    stack.reverse();
    lists.extend(stack);

    lists.join("\n")
}

fn paragraph_generator(tokens: Vec<MarkdownElement>) -> Html {
    tokens
        .iter()
        .map(|x| match x {
            MarkdownElement::Image(text, link) => format!(
                "<figure><img class='image' src=\"{}\" alt=\"{}\"><figcaption>{}</figcaption></figure>",
                link, text, text
            ),
            MarkdownElement::WikiLink(text, link) => format!("<a href={}>{}</a>", link, text),
            MarkdownElement::Hyperlink(text, link) => format!("<a href={}>{}</a>", link, text),
            MarkdownElement::InlineCode(code) => format!("<code>{}</code>", code),
            MarkdownElement::BoldItalics(text) => format!("<b><em>{}</em></b>", text),
            MarkdownElement::Italics(text) => format!("<em>{}</em>", text),
            MarkdownElement::Bold(text) => format!("<b>{}</b>", text),
            MarkdownElement::Whitespace(space) => space.to_string(),
            MarkdownElement::PlainText(x) => x.to_string(),
            _ => String::new(),
        })
        .collect::<Vec<String>>()
        .join("")
}

// TODO create http request to hilite.me to get a highlighted html with language
fn code_generator(language: String, code: String) -> Html {
    format!("<pre>\n<code>{}</code>\n</pre>", code)
}
