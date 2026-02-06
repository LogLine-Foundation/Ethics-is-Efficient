#!/usr/bin/env node

/**
 * Build script to convert markdown papers to HTML
 * This maintains simplicity - no build tools needed for the website itself
 * Run this script when papers are updated: node scripts/build-papers.js
 */

const fs = require('fs');
const path = require('path');

// Paper metadata mapping
const papers = [
    {
        file: '00_Prologue_Ethics_is_Efficient.md',
        number: 'PAPER 00 • PROLOGUE',
        title: 'Ethics is Efficient',
        description: 'One-page thesis and system invariants. The foundation of LogLine\'s architectural philosophy.'
    },
    {
        file: '01_From_Silicon_to_User.md',
        number: 'PAPER 0',
        title: 'From Silicon to User',
        description: 'Economic rationale: accountability reduces total cost through structural constraints.'
    },
    {
        file: '02_I_The_LogLine_Protocol.md',
        number: 'PAPER I',
        title: 'The LogLine Protocol',
        description: 'The 9-field tuple. Ghost records. Threat model. The core mechanism for accountability.'
    },
    {
        file: '03_II_JSON_Atomic.md',
        number: 'PAPER II',
        title: 'JSON✯Atomic',
        description: 'Deterministic canonicalization. Same meaning = same bytes. Cryptographic stability.'
    },
    {
        file: '04_III_LLLV.md',
        number: 'PAPER III',
        title: 'LLLV',
        description: 'Ledger and Proof Vectors. Proof-carrying retrieval. Evidence capsules.'
    },
    {
        file: '05_IV_TDLN.md',
        number: 'PAPER IV',
        title: 'TDLN',
        description: 'Deterministic Translation of Natural Language. Policy compilation. Consent protocol.'
    },
    {
        file: '06_V_SIRP.md',
        number: 'PAPER V',
        title: 'SIRP',
        description: 'Secure Intent Routing Protocol. Network transport. Capsules. Cryptographic receipts.'
    },
    {
        file: '07_Hardware_as_Text_and_Power.md',
        number: 'SYNTHESIS',
        title: 'Hardware as Text and Power',
        description: 'Substrate theory: signed text becomes structural power at the silicon level.'
    },
    {
        file: '08_Chip_as_Code.md',
        number: 'PAPER VI',
        title: 'Chip as Code',
        description: 'Computational realization. Hardware as backend. Policy execution in silicon.'
    }
];

// Simple markdown parser
function parseMarkdown(markdown) {
    let html = markdown;
    
    // Remove YAML frontmatter
    html = html.replace(/^---[\s\S]*?---\n\n?/m, '');
    
    // Escape HTML entities in code blocks first to preserve them
    const codeBlocks = [];
    html = html.replace(/```([^\n]*)\n([\s\S]*?)```/g, (match, lang, code) => {
        const escaped = code
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
        const placeholder = `__CODEBLOCK_${codeBlocks.length}__`;
        codeBlocks.push(`<pre><code>${escaped.trim()}</code></pre>`);
        return placeholder;
    });
    
    // Convert headers
    html = html.replace(/^#### (.*$)/gim, '<h4>$1</h4>');
    html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');
    
    // Convert horizontal rules
    html = html.replace(/^---$/gim, '<hr>');
    
    // Convert blockquotes (multi-line support)
    const lines = html.split('\n');
    let inBlockquote = false;
    let blockquoteContent = [];
    const processedLines = [];
    
    for (let line of lines) {
        if (line.startsWith('> ')) {
            if (!inBlockquote) {
                inBlockquote = true;
                blockquoteContent = [];
            }
            blockquoteContent.push(line.substring(2));
        } else {
            if (inBlockquote) {
                processedLines.push('<blockquote>' + blockquoteContent.join(' ') + '</blockquote>');
                inBlockquote = false;
                blockquoteContent = [];
            }
            processedLines.push(line);
        }
    }
    if (inBlockquote) {
        processedLines.push('<blockquote>' + blockquoteContent.join(' ') + '</blockquote>');
    }
    html = processedLines.join('\n');
    
    // Convert tables
    html = html.replace(/^\|(.+)\|\s*\n\|[-:\s|]+\|\s*\n((?:\|.+\|\s*\n?)*)/gim, (match, header, rows) => {
        const headers = header.split('|').map(h => h.trim()).filter(h => h);
        const tableRows = rows.trim().split('\n').map(row => {
            const cells = row.split('|').map(c => c.trim()).filter(c => c);
            return '<tr>' + cells.map(c => `<td>${c}</td>`).join('') + '</tr>';
        }).join('\n');
        const headerRow = '<tr>' + headers.map(h => `<th>${h}</th>`).join('') + '</tr>';
        return `<table>\n<thead>\n${headerRow}\n</thead>\n<tbody>\n${tableRows}\n</tbody>\n</table>`;
    });
    
    // Convert unordered lists
    html = html.replace(/^[-*] (.+)$/gim, '<li>$1</li>');
    html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, '<ul>\n$1</ul>\n');
    
    // Convert bold and italic (do this after lists to avoid conflicts)
    html = html.replace(/\*\*\*(.*?)\*\*\*/g, '<strong><em>$1</em></strong>');
    html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
    
    // Convert inline code
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
    
    // Convert links
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');
    
    // Restore code blocks
    codeBlocks.forEach((block, i) => {
        html = html.replace(`__CODEBLOCK_${i}__`, block);
    });
    
    // Convert paragraphs (split by double newlines)
    const blocks = html.split('\n\n');
    html = blocks.map(block => {
        block = block.trim();
        if (!block) return '';
        // Don't wrap block-level elements
        if (block.match(/^<(h[1-6]|blockquote|ul|ol|pre|hr|table|div)/)) {
            return block;
        }
        // Replace single newlines with <br> within paragraphs
        return '<p>' + block.replace(/\n/g, '<br>') + '</p>';
    }).join('\n\n');
    
    return html;
}

// HTML template
function generateHTML(paper, content, metadata) {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="${paper.description}">
    <meta name="keywords" content="LogLine, accountability, security, protocol, ${paper.title.toLowerCase()}">
    <title>${paper.title} - LogLine Foundation</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        :root {
            --bg-dark: #0a0a0a;
            --text-light: #e8e8e8;
            --accent: #4a9eff;
            --border: #2a2a2a;
            --code-bg: #151515;
        }

        body {
            background-color: var(--bg-dark);
            color: var(--text-light);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.8;
            font-size: 16px;
        }

        .container {
            max-width: 800px;
            margin: 0 auto;
            padding: 0 40px;
        }

        /* Header */
        header {
            padding: 60px 0 40px;
            border-bottom: 1px solid var(--border);
        }

        .back-link {
            display: inline-block;
            color: var(--accent);
            text-decoration: none;
            margin-bottom: 20px;
            font-weight: 500;
        }

        .back-link:hover {
            text-decoration: underline;
        }

        .paper-number {
            font-family: 'Courier New', monospace;
            color: var(--accent);
            font-weight: 700;
            font-size: 0.9rem;
            margin-bottom: 10px;
        }

        h1 {
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 10px;
            color: var(--text-light);
        }

        .paper-meta {
            color: #999;
            font-size: 0.95rem;
            margin-top: 20px;
            padding: 20px;
            background-color: var(--code-bg);
            border-left: 3px solid var(--accent);
        }

        .paper-meta p {
            margin-bottom: 8px;
        }

        .paper-meta p:last-child {
            margin-bottom: 0;
        }

        /* Content */
        article {
            padding: 80px 0;
        }

        #markdown-content h1 {
            font-size: 2.5rem;
            font-weight: 700;
            margin-top: 60px;
            margin-bottom: 25px;
            color: var(--text-light);
            line-height: 1.3;
        }

        #markdown-content h1:first-child {
            margin-top: 0;
        }

        #markdown-content h2 {
            font-size: 2rem;
            margin-top: 50px;
            margin-bottom: 20px;
            color: var(--text-light);
            line-height: 1.4;
        }

        #markdown-content h3 {
            font-size: 1.5rem;
            margin-top: 40px;
            margin-bottom: 15px;
            color: var(--text-light);
            line-height: 1.4;
        }

        #markdown-content h4 {
            font-size: 1.25rem;
            margin-top: 30px;
            margin-bottom: 12px;
            color: var(--text-light);
        }

        #markdown-content p {
            margin-bottom: 20px;
            color: #ccc;
            line-height: 1.8;
        }

        #markdown-content blockquote {
            font-style: italic;
            color: #aaa;
            padding: 25px 30px;
            border-left: 4px solid var(--accent);
            margin: 30px 0;
            background-color: var(--code-bg);
        }

        #markdown-content blockquote p {
            margin-bottom: 10px;
        }

        #markdown-content blockquote p:last-child {
            margin-bottom: 0;
        }

        #markdown-content hr {
            border: none;
            border-top: 1px solid var(--border);
            margin: 50px 0;
        }

        /* Code Blocks */
        #markdown-content pre {
            background-color: var(--code-bg);
            padding: 25px;
            margin: 25px 0;
            border: 1px solid var(--border);
            overflow-x: auto;
            border-radius: 4px;
        }

        #markdown-content code {
            font-family: 'Courier New', Monaco, monospace;
            font-size: 0.9rem;
        }

        #markdown-content pre code {
            color: #4a9eff;
        }

        #markdown-content p code,
        #markdown-content li code {
            background-color: var(--code-bg);
            color: var(--accent);
            padding: 3px 6px;
            border-radius: 3px;
            font-size: 0.9em;
        }

        /* Lists */
        #markdown-content ul, 
        #markdown-content ol {
            margin-left: 30px;
            margin-bottom: 20px;
            margin-top: 10px;
        }

        #markdown-content li {
            margin-bottom: 12px;
            color: #ccc;
            line-height: 1.7;
        }

        #markdown-content li p {
            margin-bottom: 10px;
        }

        /* Tables */
        #markdown-content table {
            width: 100%;
            border-collapse: collapse;
            margin: 25px 0;
            background-color: var(--code-bg);
        }

        #markdown-content th,
        #markdown-content td {
            padding: 12px 15px;
            text-align: left;
            border: 1px solid var(--border);
        }

        #markdown-content th {
            background-color: var(--border);
            color: var(--text-light);
            font-weight: 600;
        }

        #markdown-content td {
            color: #ccc;
        }

        #markdown-content strong {
            color: var(--text-light);
            font-weight: 600;
        }

        #markdown-content em {
            color: #bbb;
        }

        /* Links */
        a {
            color: var(--accent);
            text-decoration: none;
        }

        a:hover {
            text-decoration: underline;
        }

        /* Footer */
        footer {
            padding: 40px 0;
            text-align: center;
            color: #666;
            border-top: 1px solid var(--border);
        }

        /* Responsive */
        @media (max-width: 768px) {
            .container {
                padding: 0 20px;
            }

            h1 {
                font-size: 2rem;
            }

            #markdown-content h1 {
                font-size: 2rem;
            }

            #markdown-content h2 {
                font-size: 1.5rem;
            }

            #markdown-content h3 {
                font-size: 1.25rem;
            }
        }
    </style>
</head>
<body>
    <header>
        <div class="container">
            <a href="../index.html" class="back-link">← Back to All Papers</a>
            <div class="paper-number">${paper.number}</div>
            <h1>${paper.title}</h1>
            <div class="paper-meta">
                <p><strong>Author:</strong> ${metadata.author || 'Dan Voulez'}</p>
                <p><strong>Institution:</strong> ${metadata.institution || 'The LogLine Foundation'}</p>
                <p><strong>Version:</strong> ${metadata.version || '1.0.1'}</p>
                <p><strong>Date:</strong> ${metadata.date || 'February 05, 2026'}</p>
                ${metadata.thesis ? `<p><strong>Thesis:</strong> ${metadata.thesis}</p>` : ''}
            </div>
        </div>
    </header>

    <article>
        <div class="container">
            <div id="markdown-content">
${content}
            </div>
        </div>
    </article>

    <footer>
        <div class="container">
            <p>The LogLine Foundation</p>
            <p><a href="https://github.com/LogLine-Foundation/Ethics-is-Efficient">github.com/LogLine-Foundation</a></p>
        </div>
    </footer>
</body>
</html>`;
}

// Extract metadata from markdown frontmatter
function extractMetadata(markdown) {
    const match = markdown.match(/^---\n([\s\S]*?)\n---/);
    if (!match) return {};
    
    const metadata = {};
    const lines = match[1].split('\n');
    for (const line of lines) {
        const [key, ...valueParts] = line.split(':');
        if (key && valueParts.length) {
            const value = valueParts.join(':').trim().replace(/^["']|["']$/g, '');
            metadata[key.trim()] = value;
        }
    }
    return metadata;
}

// Main build function
function buildPapers() {
    const docsDir = path.join(__dirname, '../docs/papers/LogLine_Papers_v1.0.1');
    const websiteDir = path.join(__dirname, '../website/papers');
    
    console.log('Building papers from markdown...\n');
    
    let builtCount = 0;
    let errorCount = 0;
    
    for (const paper of papers) {
        try {
            const mdPath = path.join(docsDir, paper.file);
            const htmlPath = path.join(websiteDir, paper.file.replace('.md', '.html'));
            
            // Read markdown file
            const markdown = fs.readFileSync(mdPath, 'utf8');
            
            // Extract metadata
            const metadata = extractMetadata(markdown);
            
            // Convert markdown to HTML
            const content = parseMarkdown(markdown);
            
            // Generate full HTML page
            const html = generateHTML(paper, content, metadata);
            
            // Write HTML file
            fs.writeFileSync(htmlPath, html, 'utf8');
            
            console.log(`✓ Built ${paper.file} -> ${path.basename(htmlPath)}`);
            builtCount++;
        } catch (error) {
            console.error(`✗ Error building ${paper.file}:`, error.message);
            errorCount++;
        }
    }
    
    console.log(`\nBuild complete: ${builtCount} papers built, ${errorCount} errors`);
}

// Run the build
buildPapers();
