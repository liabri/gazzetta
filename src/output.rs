use std::path::Path;
use anyhow::{ Context, Result };
use std::fs::{ write, create_dir_all };
use handlebars::Handlebars;

pub fn templates<'a>() -> Result<Handlebars<'a>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("articles", include_str!("../templates/articles.hbs"))?;
    handlebars.register_template_string("article", include_str!("../templates/article.hbs"))?;
    Ok(handlebars)
}

pub fn write_static(output: &Path) -> Result<()> {
    for (path, content) in [
        (output.join("index.css"), include_str!("../templates/index.css")),

    ] {
        create_dir_all(path.parent().with_context(|| format!("Could not determine parent directory of {:?}", path))?)?;
        write(&path, content).with_context(|| format!("Failed to write asset at {:?}", path))?;
    }

    Ok(())
}