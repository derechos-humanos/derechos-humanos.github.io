use anyhow::{Context, Result};
use chrono::NaiveDate;
use pulldown_cmark::{Options, Parser, html};
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::{
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexContent {
    title: String,
    description: String,
    read_more: String,
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
    create_index_redirect()?;

    println!("✓ Site built successfully in _site/");
    Ok(())
}

fn build_language_site(
    tera: &Tera,
    lang: &str,
) -> Result<()> {
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
    let content_dir =
        PathBuf::from("content").join(lang).join("posts");

    WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str())
                == Some("md")
        })
        .map(|entry| parse_post(entry.path(), lang))
        .collect()
}

fn parse_post(path: &Path, _lang: &str) -> Result<Post> {
    let content =
        fs::read_to_string(path).with_context(|| {
            format!("Failed to read {:?}", path)
        })?;

    let (metadata, markdown) =
        extract_frontmatter(&content)?;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&markdown, options);
    let html_content = process_markdown_with_qr(parser)?;

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

fn extract_frontmatter(
    content: &str,
) -> Result<(PostMetadata, String)> {
    let parts: Vec<&str> =
        content.splitn(3, "---").collect();

    if parts.len() < 3 {
        anyhow::bail!("Invalid frontmatter format");
    }

    let metadata: PostMetadata =
        serde_yaml::from_str(parts[1].trim())
            .context("Failed to parse frontmatter")?;

    Ok((metadata, parts[2].trim().to_string()))
}

const QR_SIZE: usize = 160;

fn generate_qr_svg(url: &str) -> Result<String> {
    let code = QrCode::new(url.as_bytes())
        .context("Failed to generate QR code")?;

    let qr_matrix = code.to_colors();
    let width = code.width();
    let cell_size = QR_SIZE / width;
    let svg_size = cell_size * width;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        svg_size, svg_size, QR_SIZE, QR_SIZE
    );

    for (y, row) in qr_matrix.chunks(width).enumerate() {
        for (x, &color) in row.iter().enumerate() {
            if color == qrcode::Color::Dark {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}"/>"#,
                    x * cell_size,
                    y * cell_size,
                    cell_size,
                    cell_size
                ));
            }
        }
    }

    svg.push_str("</svg>");
    Ok(svg)
}

fn process_markdown_with_qr<'a>(
    parser: Parser<'a>,
) -> Result<String> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut html_output = String::new();
    let mut events = Vec::new();
    let mut current_link: Option<String> = None;

    for event in parser {
        match &event {
            Event::Start(Tag::Link {
                dest_url, ..
            }) => {
                let url = dest_url.to_string();
                if url.starts_with("http://")
                    || url.starts_with("https://")
                {
                    current_link = Some(url);
                }
                events.push(event);
            }
            Event::End(TagEnd::Link) => {
                events.push(event);
                if let Some(url) = current_link.take() {
                    if let Ok(qr_svg) =
                        generate_qr_svg(&url)
                    {
                        events.push(Event::Html(
                            format!(
                                r#"<br>{}<br>"#,
                                qr_svg
                            )
                            .into(),
                        ));
                    }
                }
            }
            _ => events.push(event),
        }
    }

    html::push_html(&mut html_output, events.into_iter());
    Ok(html_output)
}

fn load_index_content(lang: &str) -> Result<IndexContent> {
    let path = PathBuf::from("content")
        .join(lang)
        .join("index.md");
    let content =
        fs::read_to_string(&path).with_context(|| {
            format!("Failed to read {:?}", path)
        })?;

    let parts: Vec<&str> =
        content.splitn(3, "---").collect();

    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid frontmatter format in index.md"
        );
    }

    let index_content: IndexContent = serde_yaml::from_str(
        parts[1].trim(),
    )
    .context("Failed to parse index.md frontmatter")?;

    Ok(index_content)
}

fn render_post(
    tera: &Tera,
    lang: &str,
    post: &Post,
    output_dir: &Path,
) -> Result<()> {
    let mut context = TeraContext::new();
    context.insert("post", post);
    context.insert("lang", lang);
    context.insert("languages", &LANGUAGES);

    let footer_qr = generate_qr_svg(
        "https://derechos-humanos.github.io",
    )
    .unwrap_or_default();
    context.insert("footer_qr", &footer_qr);

    let html = tera
        .render("common/post.html", &context)
        .context("Failed to render post template")?;

    let output_path =
        output_dir.join(format!("{}.html", post.slug));
    fs::write(&output_path, html).with_context(|| {
        format!("Failed to write {:?}", output_path)
    })?;

    Ok(())
}

fn render_index(
    tera: &Tera,
    lang: &str,
    posts: &[Post],
    output_dir: &Path,
) -> Result<()> {
    let mut sorted_posts = posts.to_vec();
    sorted_posts.sort_by(|a, b| {
        b.metadata.date.cmp(&a.metadata.date)
    });

    let content = load_index_content(lang)?;

    let mut context = TeraContext::new();
    context.insert("posts", &sorted_posts);
    context.insert("lang", lang);
    context.insert("languages", &LANGUAGES);
    context.insert("content", &content);

    let footer_qr = generate_qr_svg(
        "https://derechos-humanos.github.io",
    )
    .unwrap_or_default();
    context.insert("footer_qr", &footer_qr);

    let html = tera
        .render("index.html", &context)
        .with_context(|| {
            format!("Failed to render index for {}", lang)
        })?;

    let output_path = output_dir.join("index.html");
    fs::write(&output_path, html).with_context(|| {
        format!("Failed to write {:?}", output_path)
    })?;

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
                let relative =
                    path.strip_prefix(static_dir)?;
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

fn create_index_redirect() -> Result<()> {
    let redirect_html = r#"<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url=es/index.html">
    <link rel="canonical" href="es/index.html">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="es/index.html">Spanish version</a>...</p>
</body>
</html>"#;

    fs::write("_site/index.html", redirect_html)?;
    Ok(())
}
