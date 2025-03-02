use anyhow::{Context, Result};
use chrono::naive::NaiveDate;
use handlebars::Handlebars;
use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// All articles recursively found
#[derive(Serialize)]
pub(crate) struct Articles {
    /// Tag name, pointers to inner
    pub tags: HashMap<String, HashSet<usize>>,
    pub inner: Vec<Article>,
}

impl Articles {
    pub fn read(path: &Path) -> Result<Self> {
        let mut inner: Vec<Article> = Vec::new();
        let mut tags: HashMap<String, HashSet<usize>> = HashMap::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|x| x.path().extension().unwrap_or_default().to_str() == Some("md"))
        {
            let article = Article::read(&path, &entry.path())?;
            for tag in article.tags.clone() {
                let entry = tags.entry(tag);
                entry.or_default().insert(inner.len());
            }
            inner.push(article);
        }

        Ok(Self { tags, inner })
    }

    pub fn write(&mut self, templates: &Handlebars, path: &Path) -> Result<()> {
        // tags
        for tag in &self.tags {
            let file = create_file(
                &path.join("tag").join(tag.0).with_extension("html"),
                false,
                true,
            )?;
            let mut writer = BufWriter::new(file);
            let mut articles = tag
                .1
                .iter()
                .map(|i| self.inner[*i].clone())
                .collect::<Vec<_>>();
            articles.sort_by(|p1, p2| p2.path.cmp(&p1.path));

            let html = templates
                .render("tag", &(&tag.0, &articles))
                .with_context(|| format!("Failed to render tag '{}' HTML page", &tag.0))?;
            writer.write(html.as_bytes())?;
            writer.flush()?;
        }

        self.inner.sort_by(|p1, p2| p2.path.cmp(&p1.path));

        // index
        let file = create_file(&path.join("index").with_extension("html"), false, true)?;
        let mut writer = BufWriter::new(file);

        let html = templates
            .render("index", &self)
            .with_context(|| "Failed to render index HTML page")?;
        writer.write(html.as_bytes())?;
        writer.flush()?;

        // articles
        for article in &self.inner {
            let file = create_file(
                &path.join(&article.path.with_extension("html")),
                false,
                true,
            )?;
            let mut writer = BufWriter::new(file);

            let html = templates.render("article", &article).with_context(|| {
                format!("Failed to render article '{:?}' HTML page", &article.path)
            })?;
            writer.write(html.as_bytes())?;
            writer.flush()?;
        }

        Ok(())
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct Article {
    pub title: String,
    pub desc: String,
    pub date: String,
    pub tags: Vec<String>,
    pub lang: String,
    pub path: PathBuf,
    pub html: String,
    pub secs: Vec<Section>,
}

#[derive(Serialize, Clone)]
pub struct Section {
    value: String,
    href: String,
    level: char,
}

use html_editor::operation::{Editable, Htmlifiable, Queryable, Selector};
use html_editor::{parse, Element, Node};
use icu::calendar::{Date, Gregorian, Iso};
use icu::datetime::{options::length, TypedDateFormatter};
use icu::locid::Locale;
use std::str::FromStr;

/// Extracts all plain text from all Text nodes of an Element.
pub fn extract_all_text(output: &mut String, el: Element) {
    for child in el.children {
        match child {
            Node::Text(s) => output.push_str(&s),
            Node::Element(e) => extract_all_text(
                output,
                Element {
                    name: e.name,
                    attrs: e.attrs,
                    children: e.children,
                },
            ),
            _ => (),
        }
    }
}

impl Article {
    pub fn read(input: &Path, path: &Path) -> Result<Self> {
        let article = read_to_string(path)?;

        // separatation of yaml & markdown
        let (yaml, markdown) = article
            .split_once("\n\n")
            .context("cannot split yaml & markdown")?;

        // parsing of markdown
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&markdown, options);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        // html = String::from_utf8_lossy(&minify_html_onepass::copy(html.as_bytes(), &minify_html_onepass::Cfg::new()).unwrap()).to_string();
        // https://crates.io/crates/scraper

        // parsing of yaml
        let (title, date, tags, lang): (String, String, Vec<String>, String) =
            zmerald::from_str(yaml).unwrap();

        // processing of yaml data
        let locale = Locale::from_str(&lang).unwrap();

        // Localised Date, disgusting -- will need to fix
        let mut d = date.split("-");
        let year = d.next().unwrap().parse::<i32>().unwrap();
        let month = d.next().unwrap().parse::<u8>().unwrap();
        let day = d.next().unwrap().parse::<u8>().unwrap();
        let df = TypedDateFormatter::<Gregorian>::try_new_with_length_unstable(
            &icu_testdata::unstable(),
            &locale.into(),
            length::Date::Long,
        )
        .expect("Failed to create TypedDateFormatter instance.");
        let date =
            Date::try_new_gregorian_date(year, month, day).expect("Failed to construct Date.");

        let doc: Vec<Node> = parse(&html).unwrap();

        // description
        let mut desc = String::new();
        if let Some(element) = doc.query(&Selector::from("p")) {
            extract_all_text(&mut desc, element.clone());
        }
        desc.truncate(300);
        let mut desc = desc.trim().to_string();
        desc.push_str("...");

        // wrap every header and its content (until next header) in a <section>
        let secs: Vec<Section> = Vec::new();

        // temp
        let mut headers = Vec::new();
        headers.append(&mut doc.query_all(&Selector::from("h2, h3, h4, h5, h6")));

        // maybe nest sections inside sections, should make <ul> for every new level
        // for header in headers  {
        //     if let Some(Node::Text(v)) = &header.children.first() {
        //         let value = v.to_owned();
        //         let href = v.to_lowercase().replace(" ", "_");
        //         let level = header.name.clone().remove(1);

        //         // for now just add a section above the header
        //         let sec = Node::new_element("section", vec![("id", &href)], vec![/*Node::Element{name: header.name.clone(), attrs: header.attrs.clone(), children: header.children.clone() }*/]);
        //         for (i, node) in doc.clone().into_iter().enumerate() {
        //             if i>0 {
        //                 if node.html().contains(&value) {
        //                     doc.insert(i-1, sec.clone());
        //                 }
        //             }
        //         }

        //         let section = Section{ value, href, level };
        //         secs.push(section);
        //     }
        // }

        Ok(Self {
            title,
            desc,
            date: df.format(&date).to_string(),
            tags,
            lang,
            path: path
                .strip_prefix(input)?
                .to_path_buf()
                .with_extension("html"),
            html: doc.html(),
            secs,
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
