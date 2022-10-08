use pulldown_cmark::{Parser, Options, html};
// use serde::de;
// use serde::Deserialize;
use anyhow::Result;
use std::path::{ Path, PathBuf };
use std::io::{ Write, BufWriter };
use std::fs::{ read_to_string, read_dir };
use walkdir::WalkDir;

/// All articles recursively found
// #[derive(Debug, Deserialize)]
pub(crate) struct Articles(Vec<Article>);

impl Articles {
    pub fn read(path: &Path) -> Result<Self> {
        let mut articles: Vec<Article> = Vec::new();

        for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok())
        .filter(|x| x.path().extension().unwrap_or_default().to_str()==Some("md")) {
            articles.push(Article::read(&entry.path())?);
        }

        Ok(Self(articles))
    }

    pub fn write(&mut self, path: &Path) -> Result<()> {
        for article in &self.0 {
            println!("DD");
            let file = std::fs::OpenOptions::new().write(true).truncate(true).open(&article.path)?;
            let mut writer = BufWriter::new(file);
            writer.write(article.html.as_bytes())?;
            writer.flush()?;
        }

        Ok(())
    }
}

// #[derive(Debug, Deserialize)]
pub(crate) struct Article {
    // #[serde(skip)] 
    pub path: PathBuf,
    // #[serde(deserialize_with = "md_to_html")]
    pub html: String
}

impl Article {
    pub fn read(path: &Path) -> Result<Self> {
        let markdown = read_to_string(path)?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&markdown, options);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        Ok(Self {
            path: path.to_path_buf(),
            html
        })
    }
}

// fn md_to_html<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
//     struct Markdown;

//     impl<'de> de::Visitor<'de> for Markdown {
//         type Value = String;

//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("string")
//         }

//         fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
//             let mut options = Options::empty();
//             options.insert(Options::ENABLE_STRIKETHROUGH);
            
//             let parser = Parser::new_ext(value, options);
//             let mut html_output = String::new();
//             html::push_html(&mut html_output, parser);
//             Ok(html_output)
//         }
//     }

//     deserializer.deserialize_any(Markdown)
// }