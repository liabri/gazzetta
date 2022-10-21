use pulldown_cmark::{Parser, Options, html};
use anyhow::{ Context, Result };
use std::path::{ Path, PathBuf };
use std::io::{ Write, BufWriter };
use std::fs::{ create_dir_all, File, read_to_string };
use walkdir::WalkDir;
use chrono::naive::NaiveDate;
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::{ HashSet, HashMap };

/// All articles recursively found
#[derive(Serialize)]
pub(crate) struct Articles {
    /// Tag name, pointers to inner
    pub tags: HashMap<String, HashSet<usize>>,
    pub inner: Vec<Article>
}


impl Articles {
    pub fn read(path: &Path) -> Result<Self> {
        let mut inner: Vec<Article> = Vec::new();
        let mut tags: HashMap<String, HashSet<usize>> = HashMap::new();

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok())
        .filter(|x| x.path().extension().unwrap_or_default().to_str()==Some("md")) {
            let article = Article::read(&path, &entry.path())?;
            for tag in article.tags.clone() {
                let entry = tags.entry(tag);
                entry.or_default().insert(inner.len());
            }
            inner.push(article);
        }

        Ok(Self{ tags, inner })
    }

    pub fn write(&mut self, templates: &Handlebars, path: &Path) -> Result<()> {
        // tags
        for tag in &self.tags {
            let file = create_file(&path.join("t").join(tag.0).with_extension("html"), false, true)?;
            let mut writer = BufWriter::new(file);
            let mut articles = tag.1.iter().map(|i| self.inner[*i].clone()).collect::<Vec<_>>();
            articles.sort_by(|p1, p2| {
                p2.path.cmp(&p1.path)
            });

            let html = templates.render("tag", &(&tag.0, &articles)).with_context(|| format!("Failed to render tag '{}' HTML page", &tag.0))?;
            writer.write(html.as_bytes())?;
            writer.flush()?;
        }

        self.inner.sort_by(|p1, p2| {
                p2.path.cmp(&p1.path)
        });

        // index
        let file = create_file(&path.join("index").with_extension("html"), false, true)?;
        let mut writer = BufWriter::new(file);

        let html = templates.render("index", &self).with_context(|| "Failed to render index HTML page")?;
        writer.write(html.as_bytes())?;
        writer.flush()?;

        // articles
        for article in &self.inner {
            let file = create_file(&path.join(&article.path.with_extension("html")), false, true)?;
            let mut writer = BufWriter::new(file);
            
            let html = templates.render("article", &article).with_context(|| format!("Failed to render article '{:?}' HTML page", &article.path))?;
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
    pub secs: Vec<String>
}

use icu::calendar::{Date, Gregorian, Iso};
use icu::datetime::{options::length, TypedDateFormatter};
use icu::locid::Locale;
use std::str::FromStr;
use html_editor::operation::{ Htmlifiable, Editable, Selector, Queryable };
use html_editor::{ Element, Node, parse };


/// Extracts all plain text from all Text nodes of an Element.
pub fn extract_all_text(output: &mut String, el: Element) {
    for child in el.children {
        match child {
            Node::Text(s) => output.push_str(&s),
            Node::Element{name, attrs, children} => extract_all_text(output, Element{name, attrs, children}),
            _ => ()
        }
    }
}

impl Article {
    pub fn read(input: &Path, path: &Path) -> Result<Self> {
        let article = read_to_string(path)?;

        // separatation of yaml & markdown
        let (yaml, markdown) = article.split_once("\n\n").context("cannot split yaml & markdown")?;

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
        let (title, date, tags, lang): (String, String, Vec<String>, String) = zmerald::from_str(yaml).unwrap();

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
        ).expect("Failed to create TypedDateFormatter instance.");
        let date = Date::try_new_gregorian_date(year, month, day).expect("Failed to construct Date.");


        let mut doc: Vec<Node> = parse(&html).unwrap();

        // description
        let mut desc = String::new();
        extract_all_text(&mut desc, doc.query(&Selector::from("p")).unwrap());
        desc.truncate(300);
        let mut desc = desc.trim().to_string();
        desc.push_str("...");

        // add <section> around every header
        let mut secs: Vec<String> = Vec::new();

        //this does not respect the order in which they are. prioritises h1 above h2 etc.
        let mut headers = Vec::new();
        headers.append(&mut doc.query_all(&Selector::from("h1")));
        headers.append(&mut doc.query_all(&Selector::from("h2")));
        headers.append(&mut doc.query_all(&Selector::from("h3")));
        headers.append(&mut doc.query_all(&Selector::from("h4")));
        headers.append(&mut doc.query_all(&Selector::from("h5")));
        headers.append(&mut doc.query_all(&Selector::from("h6")));

        for header in headers  {
            // println!("header: {:?}", header);
            if let Some(Node::Text(v)) = header.children.first() {
                // println!(" V: {}", v);
                secs.push(v.to_owned());
                let sec = Node::new_element("section", vec![("id", &v)], vec![header.children.first().unwrap().to_owned()]);
                // html = doc.remove_by(&Selector::from("h2"))
                // .insert_to(&Selector::from("main"), sec).html();
            }
        }

        Ok(Self {
            title,
            desc,
            date: df.format(&date).to_string(),
            tags,
            lang,
            path: path.strip_prefix(input)?.to_path_buf().with_extension("html"),
            html,
            secs
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