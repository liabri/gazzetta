use crate::error::{path_error, PathErrorContext};

use anyhow::Result;
use std::path::{ Path, PathBuf };
use time::Date;
// use chrono::DateTime;

/// All articles recursively found
#[derive(Debug)]
pub(crate) struct Articles(Vec<Article>)

impl Articles {
    pub fn get(path: &Path) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Article {
    #[serde(skip)] 
    pub path: PathBuf,
    #[serde(deserialize_with = "md_to_html")]
    pub html: String
}

fn md_to_html<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
    struct String;

    impl<'de> Visitor<'de> for String {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E> where E: de::Error {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            
            let parser = Parser::new_ext(markdown_input, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            Ok(html_output)
        }
    }

    deserializer.deserialize_any(String)
}