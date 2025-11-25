use anyhow::{Context, Result};
use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostMetadata {
    title: String,
    date: NaiveDate,
    #[serde(default)]
    author: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct Post {
    metadata: PostMetadata,
    content: String,
    url: String,
    slug: String,
}

const LANGUAGES: [&str; 3] = ["es", "en", "ru"];

fn main() -> Result<()> {
    println!("Building multi-language blog...");

    let tera = Tera::new("templates/**/*.html")
        .context("Failed to initialize template engine")?;

    for lang in LANGUAGES {
        build_language_site(&tera, lang)?;
    }

    copy_static_files()?;
    copy_static_files()?;
    create_language_selector(&tera)?;

    println!("✓ Site built successfully in _site/");
    Ok(())
}

fn build_language_site(tera: &Tera, lang: &str) -> Result<()> {
    println!("Building {} version...", lang);

    let posts = collect_posts(lang)?;
    let output_dir = PathBuf::from("_site").join(lang);
    fs::create_dir_all(&output_dir)?;

    for post in &posts {
        render_post(tera, lang, post, &output_dir)?;
    }

    render_index(tera, lang, &posts, &output_dir)?;

    Ok(())
}

fn collect_posts(lang: &str) -> Result<Vec<Post>> {
    let content_dir = PathBuf::from("content").join(lang).join("posts");
    
    WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .map(|entry| parse_post(entry.path(), lang))
        .collect()
}

fn parse_post(path: &Path, _lang: &str) -> Result<Post> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;

    let (metadata, markdown) = extract_frontmatter(&content)?;
    
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    
    let parser = Parser::new_ext(&markdown, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    let url = format!("{}.html", slug);

    Ok(Post {
        metadata,
        content: html_content,
        url,
        slug,
    })
}

fn extract_frontmatter(content: &str) -> Result<(PostMetadata, String)> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    
    if parts.len() < 3 {
        anyhow::bail!("Invalid frontmatter format");
    }

    let metadata: PostMetadata = serde_json::from_str(parts[1].trim())
        .context("Failed to parse frontmatter")?;
    
    Ok((metadata, parts[2].trim().to_string()))
}

fn render_post(tera: &Tera, lang: &str, post: &Post, output_dir: &Path) -> Result<()> {
    let mut context = TeraContext::new();
    context.insert("post", post);
    context.insert("lang", lang);
    context.insert("languages", &LANGUAGES);

    let html = tera
        .render("common/post.html", &context)
        .context("Failed to render post template")?;

    let output_path = output_dir.join(format!("{}.html", post.slug));
    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {:?}", output_path))?;

    Ok(())
}

fn render_index(tera: &Tera, lang: &str, posts: &[Post], output_dir: &Path) -> Result<()> {
    let mut sorted_posts = posts.to_vec();
    sorted_posts.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));

    let mut context = TeraContext::new();
    context.insert("posts", &sorted_posts);
    context.insert("lang", lang);
    context.insert("languages", &LANGUAGES);

    let template_name = format!("{}/index.html", lang);
    let html = tera
        .render(&template_name, &context)
        .with_context(|| format!("Failed to render {}", template_name))?;

    let output_path = output_dir.join("index.html");
    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {:?}", output_path))?;

    Ok(())
}

fn copy_static_files() -> Result<()> {
    let static_dir = Path::new("static");
    let output_dir = Path::new("_site");

    if static_dir.exists() {
        for entry in WalkDir::new(static_dir) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let relative = path.strip_prefix(static_dir)?;
                let dest = output_dir.join(relative);
                
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::copy(path, &dest)?;
            }
        }
    }

    Ok(())
}

fn create_language_selector(tera: &Tera) -> Result<()> {
    let lang_names: HashMap<&str, &str> = [
        ("es", "Español"),
        ("en", "English"),
        ("ru", "Русский"),
    ]
    .iter()
    .copied()
    .collect();

    let mut context = TeraContext::new();
    context.insert("languages", &LANGUAGES);
    context.insert("lang_names", &lang_names);

    let html = tera
        .render("index.html", &context)
        .context("Failed to render root index template")?;

    fs::write("_site/index.html", html)?;
    Ok(())
}
