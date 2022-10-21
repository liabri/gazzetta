use std::path::Path;
use anyhow::{ Context, Result };
use std::fs::{ write, create_dir_all };
use handlebars::Handlebars;

pub fn templates<'a>() -> Result<Handlebars<'a>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("index", include_str!("../templates/index.hbs"))?;
    handlebars.register_template_string("article", include_str!("../templates/article.hbs"))?;
    handlebars.register_template_string("tag", include_str!("../templates/tag.hbs"))?;
    Ok(handlebars)
}

pub fn write_static(input: &Path, output: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(&input.join("data")).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            let path = output.join(entry.path().strip_prefix(input)?.to_path_buf());
            create_dir_all(path.parent().with_context(|| format!("Could not determine parent directory of {:?}", entry.path()))?)?;
            std::fs::copy(entry.path(), path).unwrap();
        }
    }

    for (path, content) in [
        (output.join("index.css"), include_str!("../templates/index.css")),
        (output.join("articles.css"), include_str!("../templates/articles.css")),
        (output.join("article.css"), include_str!("../templates/article.css")),
    ] {
        create_dir_all(path.parent().with_context(|| format!("Could not determine parent directory of {:?}", path))?)?;
        write(&path, content).with_context(|| format!("Failed to write asset at {:?}", path))?;
    }

    Ok(())
}