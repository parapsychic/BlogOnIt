use generator::Html;
use parser::page_parser::parse_page;

use crate::generator::page_generator::generate_page;

pub mod generator;
pub mod parser;
pub mod utils;


pub fn parse_and_generator(page: &str) -> (String, Html) {
    let parsed_tokens = parse_page(&page);
    //todo : PARSE Options
    generate_page(parsed_tokens.unwrap()).unwrap()
}
