/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A simple example that parses a CSS stylesheet and reports the number of
//! top-level rules it contains.
//!
//! Usage:
//!   cargo run --example count_rules -- path/to/style.css
//!   cargo run --example count_rules          # uses the built-in example stylesheet

use std::{env, fs, process};

use servo_arc::Arc;
use style::context::QuirksMode;
use style::media_queries::MediaList;
use style::shared_lock::SharedRwLock;
use style::stylesheets::CssRule;
use style::stylesheets::{AllowImportRules, Origin, Stylesheet, StylesheetInDocument, UrlExtraData};
use url::Url;

const EXAMPLE_CSS: &str = r#"
/* Example stylesheet */

body {
    margin: 0;
    font-family: sans-serif;
    background-color: #fff;
}

h1, h2, h3 {
    color: #333;
    font-weight: bold;
}

a {
    color: blue;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

@media (max-width: 768px) {
    body {
        font-size: 14px;
    }
}

@keyframes fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
}

@supports (display: grid) {
    .container {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
    }
}

@font-face {
    font-family: "MyFont";
    src: url("myfont.woff2") format("woff2");
}

@layer base {
    p { line-height: 1.5; }
}
"#;

fn rule_type_name(rule: &CssRule) -> &'static str {
    match rule {
        CssRule::Style(..) => "style",
        CssRule::Import(..) => "@import",
        CssRule::Media(..) => "@media",
        CssRule::Supports(..) => "@supports",
        CssRule::Keyframes(..) => "@keyframes",
        CssRule::FontFace(..) => "@font-face",
        CssRule::FontFeatureValues(..) => "@font-feature-values",
        CssRule::FontPaletteValues(..) => "@font-palette-values",
        CssRule::CounterStyle(..) => "@counter-style",
        CssRule::Namespace(..) => "@namespace",
        CssRule::Page(..) => "@page",
        CssRule::Document(..) => "@document",
        CssRule::LayerBlock(..) => "@layer (block)",
        CssRule::LayerStatement(..) => "@layer (statement)",
        CssRule::Container(..) => "@container",
        CssRule::CustomMedia(..) => "@custom-media",
        CssRule::Scope(..) => "@scope",
        CssRule::StartingStyle(..) => "@starting-style",
        CssRule::AppearanceBase(..) => "@appearance-base",
        CssRule::PositionTry(..) => "@position-try",
        CssRule::Property(..) => "@property",
        CssRule::Margin(..) => "margin rule",
        CssRule::NestedDeclarations(..) => "nested declarations",
        CssRule::ViewTransition(..) => "@view-transition",
    }
}

fn main() {
    let (source, label) = match env::args().nth(1) {
        Some(path) => {
            let css = fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error reading '{}': {}", path, e);
                process::exit(1);
            });
            (css, path)
        },
        None => (EXAMPLE_CSS.to_string(), "<built-in example>".to_string()),
    };

    let url = Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData(Arc::new(url));

    let shared_lock = SharedRwLock::new();
    let media = Arc::new(shared_lock.wrap(MediaList::empty()));

    let stylesheet = Stylesheet::from_str(
        &source,
        url_data,
        Origin::Author,
        media,
        shared_lock.clone(),
        /* stylesheet_loader = */ None,
        /* error_reporter = */ None,
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    );

    let guard = shared_lock.read();
    let contents = stylesheet.contents(&guard);
    let rules = contents.rules(&guard);

    println!("{}: {} top-level rule(s)", label, rules.len());
    for (i, rule) in rules.iter().enumerate() {
        println!("  [{:>2}] {}", i, rule_type_name(rule));
    }
}
