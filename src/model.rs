use pulldown_cmark::{Parser, Options, html};
use serde::de;
use serde::Deserialize;
use anyhow::Result;
use std::path::{ Path, PathBuf };

/// All articles recursively found
#[derive(Debug, Deserialize)]
pub(crate) struct Articles(Vec<Article>);

impl Articles {
    pub fn get(path: &Path) -> Result<Self> {
        todo!()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Article {
    #[serde(skip)] 
    pub path: PathBuf,
    #[serde(deserialize_with = "md_to_html")]
    pub html: String
}

fn md_to_html<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
    struct Markdown;

    impl<'de> de::Visitor<'de> for Markdown {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            
            let parser = Parser::new_ext(value, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            Ok(html_output)
        }
    }

    deserializer.deserialize_any(Markdown)
}