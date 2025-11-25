# Multi-Language Blog with Polysite

A static blog generator built with Rust using the Polysite library, featuring built-in internationalization support for Spanish (es), English (en), and Russian (ru). The blog is completely JavaScript-free and styled with Picnic CSS.

## Features

- **Multi-language support**: Content available in Spanish, English, and Russian
- **Static site generation**: Fast, secure, and easy to deploy
- **JavaScript-free**: Pure HTML and CSS for maximum compatibility and performance
- **Markdown-based**: Write posts in Markdown with JSON frontmatter
- **Responsive design**: Mobile-friendly layout using Picnic CSS
- **SEO-friendly**: Semantic HTML with proper meta tags

## Project Structure

```
site/
├── content/           # Markdown content files
│   ├── es/posts/     # Spanish blog posts
│   ├── en/posts/     # English blog posts
│   └── ru/posts/     # Russian blog posts
├── templates/         # HTML templates
│   ├── common/       # Shared templates (post layout)
│   ├── es/           # Spanish index page
│   ├── en/           # English index page
│   └── ru/           # Russian index page
├── static/           # Static assets
│   └── css/          # Custom CSS
├── _site/            # Generated output (git-ignored)
└── src/              # Rust source code
    └── main.rs       # Site generator
```

## Building the Site

### Prerequisites

- Rust (2024 edition or later)
- Cargo

### Build Command

```bash
cargo run
```

This will:
1. Parse all Markdown files in `content/{lang}/posts/`
2. Render them using templates from `templates/`
3. Generate static HTML files in `_site/`
4. Copy static assets to `_site/static/`
5. Create a language selector at `_site/index.html`

## Adding New Posts

### 1. Create a Markdown File

Create a new `.md` file in the appropriate language directory:

```bash
content/es/posts/mi-nuevo-post.md
content/en/posts/my-new-post.md
content/ru/posts/moy-novyy-post.md
```

### 2. Add Frontmatter

Start your post with JSON frontmatter between `---` delimiters:

```markdown
---
{
  "title": "My Post Title",
  "date": "2025-11-25",
  "author": "Author Name",
  "description": "Brief description of the post"
}
---

# Your Post Content

Write your content here in Markdown...
```

### 3. Rebuild the Site

```bash
cargo run
```

## Frontmatter Fields

- **title** (required): Post title
- **date** (required): Publication date in YYYY-MM-DD format
- **author** (optional): Author name
- **description** (optional): Brief description for SEO and post previews

## Markdown Support

The generator supports:
- Headers (H1-H6)
- Paragraphs
- Lists (ordered and unordered)
- Links
- Images
- Blockquotes
- Code blocks
- Tables
- Strikethrough

## Deployment

The generated site in `_site/` is completely static and can be deployed to:

- GitHub Pages
- Netlify
- Vercel
- Any static hosting service

Simply upload the contents of `_site/` to your hosting provider.

## Customization

### Styling

Edit `static/css/custom.css` to customize the appearance. The site uses:
- Picnic CSS (via CDN) for base styling
- Custom CSS for blog-specific styles

### Templates

Modify templates in `templates/` to change the layout:
- `common/post.html` - Individual post layout
- `{lang}/index.html` - Language-specific index pages

### Languages

To add a new language:
1. Add the language code to `LANGUAGES` array in `src/main.rs`
2. Create `content/{lang}/posts/` directory
3. Create `templates/{lang}/index.html` template
4. Add language name to navigation in templates

## License

MIT
