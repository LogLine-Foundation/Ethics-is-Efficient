# Building Papers

This directory contains the build script for converting markdown papers to HTML for the website.

## Overview

The website displays the full content of papers from `docs/papers/LogLine_Papers_v1.0.1/` as HTML pages. To maintain simplicity, we generate static HTML files at build time rather than using a complex build system or loading markdown dynamically.

## When to Rebuild

Rebuild papers when:
- Paper content in `docs/papers/LogLine_Papers_v1.0.1/*.md` is updated
- Paper metadata (title, description, etc.) changes
- The HTML template needs updates

## How to Rebuild

From the repository root, run:

```bash
node scripts/build-papers.js
```

This will:
1. Read all markdown papers from `docs/papers/LogLine_Papers_v1.0.1/`
2. Parse the markdown content (removing YAML frontmatter)
3. Generate HTML with proper styling and structure
4. Write HTML files to `website/papers/`

## Output

The script generates 9 HTML files:
- `00_Prologue_Ethics_is_Efficient.html`
- `01_From_Silicon_to_User.html`
- `02_I_The_LogLine_Protocol.html`
- `03_II_JSON_Atomic.html`
- `04_III_LLLV.html`
- `05_IV_TDLN.html`
- `06_V_SIRP.html`
- `07_Hardware_as_Text_and_Power.html`
- `08_Chip_as_Code.html`

## Features

The generated HTML pages include:
- **Full markdown content** from the repository papers
- **Improved margins** (800px max-width with 40px padding)
- **Better typography** (1.8 line-height, improved spacing)
- **Proper markdown rendering**:
  - Headers (h1-h4)
  - Blockquotes with styled borders
  - Code blocks with syntax highlighting styles
  - Tables with borders
  - Lists (bulleted and numbered)
  - Bold, italic, and inline code
  - Links
  - Horizontal rules

## No Runtime Dependencies

The build script uses **no external dependencies** - just Node.js built-in modules. The generated HTML is self-contained with no external JavaScript libraries or CSS frameworks.

## Simplicity Maintained

- **No build tools needed** for the website itself (no webpack, vite, etc.)
- **No CDN dependencies** at runtime
- **Single static HTML files** that work anywhere
- **Fast page loads** with inline CSS

## Paper Metadata

Each paper's metadata is defined in `build-papers.js`:

```javascript
{
    file: '00_Prologue_Ethics_is_Efficient.md',
    number: 'PAPER 00 • PROLOGUE',
    title: 'Ethics is Efficient',
    description: '...'
}
```

To add a new paper:
1. Add the markdown file to `docs/papers/LogLine_Papers_v1.0.1/`
2. Add metadata to the `papers` array in `build-papers.js`
3. Run the build script
4. Update the main `index.html` to link to the new paper

## Development

To test changes locally:

```bash
# Rebuild papers
node scripts/build-papers.js

# Start a local server
cd website
python3 -m http.server 8080

# Open http://localhost:8080 in your browser
```

## Deployment

The website is deployed to Cloudflare Pages, which automatically serves files from the `website/` directory. After rebuilding papers, commit the changes:

```bash
git add website/papers/*.html
git commit -m "Rebuild papers with updated content"
git push
```

Cloudflare Pages will automatically deploy the updated HTML files.
