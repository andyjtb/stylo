use cssparser::Parser;
use style::parser::ParserContext;
use style::properties::longhands::color;
use style::stylesheets::{Origin, UrlExtraData, Namespaces};
use style::context::QuirksMode;
use std::borrow::Cow;
use style_traits::ParsingMode;
use url::Url;

fn main() {
    // Create a dummy UrlExtraData (you might need to implement this or use a real one)
    let dummy_url = Url::parse("http://example.com").unwrap();
    let url_data = UrlExtraData::from(dummy_url);

    // Create a parser context
    let context = ParserContext::new(
        Origin::Author,
        &url_data,
        None, // rule_type is now optional
        ParsingMode::DEFAULT, // Use the public ParsingMode
        QuirksMode::NoQuirks,
        Cow::Owned(Namespaces::default()),
        None, // error_reporter
        None, // use_counters
    );

    // The color string we want to parse
    let color_str = "hsla(-300, 100%, 37.5%, -3)";

    // Create a parser
    let mut input = cssparser::ParserInput::new(color_str);
    let mut parser = Parser::new(&mut input);

    // Parse the color
    match color::parse(&context, &mut parser) {
        Ok(color) => {
            println!("Successfully parsed color: {:?}", color);
        },
        Err(e) => {
            println!("Failed to parse color: {:?}", e);
        }
    }
}