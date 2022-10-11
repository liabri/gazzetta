use pulldown_cmark::{Parser, Options, html};
use anyhow::{ Context, Result };
use std::path::{ Path, PathBuf };
use std::io::{ Write, BufWriter };
use std::fs::{ create_dir_all, File, read_to_string };
use walkdir::WalkDir;
use chrono::naive::NaiveDate;
use handlebars::Handlebars;
use serde::Serialize;

/// All articles recursively found
#[derive(Serialize)]
pub(crate) struct Articles {
    pub inner: Vec<Article>
}


impl Articles {
    pub fn read(path: &Path) -> Result<Self> {
        let mut inner: Vec<Article> = Vec::new();

        for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok())
        .filter(|x| x.path().extension().unwrap_or_default().to_str()==Some("md")) {
            inner.push(Article::read(&path, &entry.path())?);
        }

        Ok(Self{ inner })
    }

    pub fn write(&mut self, templates: &Handlebars, path: &Path) -> Result<()> {
        let file = create_file(&path.join("articles").with_extension("html"), false, true)?;
        let mut writer = BufWriter::new(file);
        
        let html = templates.render("articles", &self).with_context(|| "Failed to render articles HTML page")?;
        writer.write(html.as_bytes())?;
        writer.flush()?;


        for article in &self.inner {
            let file = create_file(&path.join(&article.path.with_extension("html")), false, true)?;
            let mut writer = BufWriter::new(file);
            
            let html = templates.render("article", &article).with_context(|| "Failed to render article HTML page")?;
            writer.write(html.as_bytes())?;
            writer.flush()?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub(crate) struct Article {
    pub title: String,
    pub desc: String,
    pub date: String,
    pub tags: Vec<String>,
    pub lang: String,
    pub path: PathBuf,
    pub html: String
}

use icu::calendar::{Date, Gregorian};
use icu::datetime::{options::length, TypedDateFormatter};
use icu::locid::Locale;
use std::str::FromStr;

impl Article {
    pub fn read(input: &Path, path: &Path) -> Result<Self> {
        let article = read_to_string(path)?;

        //separate yaml & markdown
        let (yaml, markdown) = article.split_once("\n\n").context("cannot split yaml & markdown")?;
        let (title, date, tags, lang): (String, String, Vec<String>, String) = zmerald::from_str(yaml).unwrap();

        let desc = markdown.split_once("\n\n").context("cannot find description")?.0.to_string();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&markdown, options);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        // Localised Date
        let df = TypedDateFormatter::<Gregorian>::try_new_with_length_unstable(
            &icu_testdata::unstable(),
            &Locale::from_str(&lang).unwrap().into(),
            length::Date::Long,
        ).expect("Failed to create TypedDateFormatter instance.");
        let date = Date::try_new_gregorian_date(2020, 2, 1).expect("Failed to construct Date.");

        Ok(Self {
            title,
            desc,
            date: df.format(&date).to_string(),
            tags,
            lang,
            path: path.strip_prefix(input)?.to_path_buf().with_extension("html"),
            html
        })
    }
}

pub fn create_file(path: &Path, read: bool, write: bool) -> std::io::Result<File> {
    let mut file = std::fs::OpenOptions::new()
        .read(read)
        .write(write)
        .truncate(true)
        .open(path);

    if let Err(_) = file {
        //handle non-existence of parent()
        create_dir_all(&path.parent().unwrap())?;

        file = std::fs::OpenOptions::new()
            .read(read)
            .write(write)
            .create(true)
            .truncate(true)
            .open(path);
    }

    file
}