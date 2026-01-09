//! Command-line interface for whatwg-html-rs.

use clap::{Parser, ValueEnum};
use std::io::{self, Read, Write};
use whatwg_html_rs::{parse, selector::query_all, sanitize::DEFAULT_POLICY};

#[derive(Parser)]
#[command(name = "whatwg-html")]
#[command(author, version, about = "WHATWG HTML5 parser and sanitizer", long_about = None)]
struct Cli {
    /// Input file (use - for stdin)
    #[arg(default_value = "-")]
    input: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "html")]
    format: OutputFormat,

    /// Sanitize output (remove scripts, dangerous attributes, etc.)
    #[arg(short, long)]
    safe: bool,

    /// Pretty print output (HTML only)
    #[arg(short, long)]
    pretty: bool,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Show parse errors
    #[arg(long)]
    errors: bool,

    /// CSS selector to query (outputs matching elements)
    #[arg(short, long)]
    query: Option<String>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    /// HTML output
    Html,
    /// Markdown output
    Markdown,
    /// Plain text output
    Text,
    /// DOM tree structure (debug)
    Tree,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Read input
    let html = if cli.input == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        std::fs::read_to_string(&cli.input)?
    };

    // Parse
    let result = if cli.errors {
        whatwg_html_rs::parse_with_errors(&html)
    } else {
        parse(&html)
    };

    // Show errors if requested
    if cli.errors && !result.errors.is_empty() {
        eprintln!("Parse errors:");
        for error in &result.errors {
            eprintln!("  - {}", error);
        }
        eprintln!();
    }

    // Apply query if provided
    let output = if let Some(selector) = &cli.query {
        let node_ids = query_all(&result.dom, result.document, selector)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        let mut output = String::new();
        for node_id in node_ids {
            match cli.format {
                OutputFormat::Html => {
                    if cli.safe {
                        let mut dom_copy = result.dom.clone();
                        let _ = whatwg_html_rs::sanitize::sanitize_dom(
                            &mut dom_copy,
                            node_id,
                            &DEFAULT_POLICY,
                        );
                        output.push_str(&whatwg_html_rs::serialize::serialize_to_html(
                            &dom_copy,
                            node_id,
                        ));
                    } else {
                        output.push_str(&whatwg_html_rs::serialize::serialize_to_html(
                            &result.dom,
                            node_id,
                        ));
                    }
                }
                OutputFormat::Text => {
                    output.push_str(&extract_text(&result.dom, node_id));
                }
                OutputFormat::Markdown => {
                    output.push_str(&whatwg_html_rs::serialize::serialize_to_markdown(
                        &result.dom,
                        node_id,
                    ));
                }
                OutputFormat::Tree => {
                    output.push_str(&format!("{:#?}\n", result.dom.get(node_id)));
                }
            }
            output.push('\n');
        }
        output
    } else {
        // Full document output
        match cli.format {
            OutputFormat::Html => {
                if cli.safe {
                    result.to_html_safe()
                } else if cli.pretty {
                    whatwg_html_rs::serialize::serialize_to_html_pretty(&result.dom, result.document)
                } else {
                    result.to_html()
                }
            }
            OutputFormat::Markdown => {
                if cli.safe {
                    result.to_markdown_safe()
                } else {
                    result.to_markdown()
                }
            }
            OutputFormat::Text => {
                if cli.safe {
                    result.to_text_safe()
                } else {
                    result.to_text()
                }
            }
            OutputFormat::Tree => {
                format!("{:#?}", result.dom)
            }
        }
    };

    // Write output
    if let Some(output_path) = cli.output {
        std::fs::write(&output_path, &output)?;
    } else {
        io::stdout().write_all(output.as_bytes())?;
        if !output.ends_with('\n') {
            println!();
        }
    }

    Ok(())
}

fn extract_text(dom: &whatwg_html_rs::Dom, node_id: whatwg_html_rs::NodeId) -> String {
    use whatwg_html_rs::NodeKind;

    let mut result = String::new();
    collect_text(dom, node_id, &mut result);
    result
}

fn collect_text(dom: &whatwg_html_rs::Dom, node_id: whatwg_html_rs::NodeId, result: &mut String) {
    use whatwg_html_rs::NodeKind;

    let node = dom.get(node_id);
    match &node.kind {
        NodeKind::Text(text) => result.push_str(text),
        NodeKind::Element(el) => {
            if matches!(el.name.as_str(), "script" | "style") {
                return;
            }
            let mut child = node.first_child;
            while let Some(child_id) = child {
                collect_text(dom, child_id, result);
                child = dom.get(child_id).next_sibling;
            }
        }
        _ => {
            let mut child = node.first_child;
            while let Some(child_id) = child {
                collect_text(dom, child_id, result);
                child = dom.get(child_id).next_sibling;
            }
        }
    }
}
