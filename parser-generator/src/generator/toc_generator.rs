use itertools::Itertools;

use crate::{parser::markdown_elements::MarkdownElement, utils::remove_all_special};

use super::Html;

pub fn generate_toc(parsed_tokens: Vec<MarkdownElement>) -> Result<Html, String> {
    let mut current_level = 0;
    let mut headings = parsed_tokens
        .iter()
        .filter_map(|token| match token {
            MarkdownElement::Heading(level, text) => {
                if *level == 1 as usize {
                    return None;
                }
                let level_changer_prefix = match current_level.cmp(&level) {
                    std::cmp::Ordering::Less => {
                        current_level = *level;
                        "<ol>"
                    }
                    std::cmp::Ordering::Equal => "",
                    std::cmp::Ordering::Greater => {
                        current_level = *level;
                        "</ol>"
                    }
                };

                Some(format!(
                    "{}<li class=toc-h{}><a href='#{}'>{}</a></li>\n",
                    level_changer_prefix,
                    level,
                    remove_all_special(text),
                    text
                ))
            }
            _ => None,
        })
        .join("\n");

    headings.push_str("</ol>");

    Ok(headings)
}
