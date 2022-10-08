use pulldown_cmark::{Parser, Options, html};
use anyhow::Result;
use std::path::{ Path, PathBuf };
use std::io::{ Write, BufWriter };
use std::fs::{ create_dir_all, File, read_to_string };
use walkdir::WalkDir;

/// All articles recursively found
pub(crate) struct Articles(Vec<Article>);

impl Articles {
    pub fn read(path: &Path) -> Result<Self> {
        let mut articles: Vec<Article> = Vec::new();

        for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok())
        .filter(|x| x.path().extension().unwrap_or_default().to_str()==Some("md")) {
            articles.push(Article::read(&path, &entry.path())?);
        }

        Ok(Self(articles))
    }

    pub fn write(&mut self, path: &Path) -> Result<()> {
        for article in &self.0 {
            let file = create_file(&path.join(&article.path.with_extension("html")), false, true)?;
            let mut writer = BufWriter::new(file);
            writer.write(article.html.as_bytes())?;
            writer.flush()?;
        }

        Ok(())
    }
}

pub fn create_file(path: &Path, read: bool, write: bool) -> std::io::Result<File> {
    let mut file = std::fs::OpenOptions::new()
        .read(read)
        .write(write)
        .open(path);

    if let Err(_) = file {
        //handle non-existence of parent()
        create_dir_all(&path.parent().unwrap())?;

        file = std::fs::OpenOptions::new()
            .read(read)
            .write(write)
            .create(true)
            .open(path);
    }

    file
}

pub(crate) struct Article {
    pub path: PathBuf,
    pub html: String
}

impl Article {
    pub fn read(input: &Path, path: &Path) -> Result<Self> {
        let markdown = read_to_string(path)?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&markdown, options);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        Ok(Self {
            path: path.strip_prefix(input)?.to_path_buf(),
            html
        })
    }
}