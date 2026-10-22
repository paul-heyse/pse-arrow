// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One bounded typed presentation for local Markdown and static HTML.
use super::*;

pub(crate) fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn markdown_escape(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if "\\[]()!*_#`<>|".contains(c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
fn target(member: &str, pointer: &str, sources: &BTreeMap<String, String>) -> String {
    format!(
        "{}#{}",
        sources.get(member).map_or(member, String::as_str),
        pointer
    )
    .bytes()
    .map(|byte| {
        if byte.is_ascii_alphanumeric() || b"/-._~#".contains(&byte) {
            char::from(byte).to_string()
        } else {
            format!("%{byte:02X}")
        }
    })
    .collect()
}
/// Source-backed descriptive orientation, shared by both report encodings.
#[derive(Default)]
pub(crate) struct ReportOrientation {
    pub sections: Vec<ReportSection>,
}
/// Finite already-selected text or table; values retain source-native integer text.
pub(crate) struct ReportSection {
    pub title: String,
    pub source: Option<(String, String)>,
    pub paragraphs: Vec<String>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
impl ReportSection {
    pub(crate) fn text(
        title: impl Into<String>,
        source: Option<(String, String)>,
        paragraphs: Vec<String>,
    ) -> Self {
        Self {
            title: title.into(),
            source,
            paragraphs,
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}
/// Render the shared typed document with its ordinary finite default report allowance.
pub fn render_markdown(document: &AnalysisDocument) -> String {
    render_markdown_with_sources(document, &BTreeMap::new())
}
/// Render source links using only explicitly supplied local member paths.
pub fn render_markdown_with_sources(
    document: &AnalysisDocument,
    sources: &BTreeMap<String, String>,
) -> String {
    render_reports(document, sources, &ReportOrientation::default(), 16 << 20).0
}
/// Bound both encodings at section boundaries, retaining an explicit omission marker.
pub(crate) fn render_reports(
    document: &AnalysisDocument,
    sources: &BTreeMap<String, String>,
    orientation: &ReportOrientation,
    maximum: usize,
) -> (String, String) {
    let mut markdown = String::new();
    let mut html = String::new();
    let html_start = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'\"><title>Offline review</title><style>body{max-width:80em;margin:2em auto;font-family:system-ui}td,th{border:1px solid #ccc;padding:.3em;overflow-wrap:anywhere}table{border-collapse:collapse}p{overflow-wrap:anywhere}</style><body>";
    let ending = "</body></html>";
    let omission = "Report budget reached; remaining presentation omitted. Original selected members retain their indexed bytes.";
    // Reserve wrappers and explicit omissions before accepting a presentation section.
    let allowance = maximum.saturating_sub(html_start.len() + ending.len() + 256);
    let mut omitted = false;
    let mut add = |md: String, ht: String| {
        if !omitted && markdown.len() + md.len() <= allowance && html.len() + ht.len() <= allowance
        {
            markdown.push_str(&md);
            html.push_str(&ht);
        } else {
            omitted = true;
        }
    };
    let introduction = format!(
        "Operation {}; baseline {:?}; recorded window {:?}. Inspected {} unique source items; omitted {} source items and {} statements. Original scientific permission and capture completeness remain independent.",
        document.request.operation,
        document.request.baseline,
        document.request.window,
        document.inspected_source_items,
        document.omitted_source_items,
        document.omitted_statements
    );
    add(
        format!(
            "# Immutable evidence analysis\n\n{}\n",
            markdown_escape(&introduction)
        ),
        format!(
            "<h1>Immutable evidence analysis</h1><p>{}</p>",
            html_escape(&introduction)
        ),
    );
    for question in &document.questions {
        add(
            format!("\n## {}\n", question.question.title()),
            format!("<h2>{}</h2>", html_escape(question.question.title())),
        );
        for statement in &question.statements {
            let message = format!(
                "{:?} {}: {}",
                statement.kind, statement.code, statement.message
            );
            add(
                format!("\n{}\n", markdown_escape(&message)),
                format!("<p>{}</p>", html_escape(&message)),
            );
            for reference in &statement.evidence {
                let label = format!("{}#{}", reference.member, reference.pointer);
                let value = format!(
                    "{:?}{}",
                    reference.value,
                    if reference.partial { " (partial)" } else { "" }
                );
                let link = target(&reference.member, &reference.pointer, sources);
                add(
                    format!(
                        "\n- [{}]({}): {}\n",
                        markdown_escape(&label),
                        link,
                        markdown_escape(&value)
                    ),
                    format!(
                        "<p><a href=\"{}\">{}</a>: <code>{}</code></p>",
                        html_escape(&link),
                        html_escape(&label),
                        html_escape(&value)
                    ),
                );
            }
            if !statement.next.is_empty() {
                let next = format!("Optional next observations: {:?}", statement.next);
                add(
                    format!("\n{}\n", markdown_escape(&next)),
                    format!("<p>{}</p>", html_escape(&next)),
                );
            }
        }
    }
    for section in &orientation.sections {
        let mut md = format!("\n## {}\n", markdown_escape(&section.title));
        let mut ht = format!("<section><h2>{}</h2>", html_escape(&section.title));
        if let Some((member, pointer)) = &section.source {
            let link = target(member, pointer, sources);
            let label = format!("{member}#{pointer}");
            md.push_str(&format!("\n[{}]({link})\n", markdown_escape(&label)));
            ht.push_str(&format!(
                "<p><a href=\"{}\">{}</a></p>",
                html_escape(&link),
                html_escape(&label)
            ));
        }
        for paragraph in &section.paragraphs {
            md.push_str(&format!("\n{}\n", markdown_escape(paragraph)));
            ht.push_str(&format!("<p>{}</p>", html_escape(paragraph)));
        }
        if !section.columns.is_empty() {
            md.push_str(&format!(
                "\n| {} |\n| {} |\n",
                section
                    .columns
                    .iter()
                    .map(|s| markdown_escape(s))
                    .collect::<Vec<_>>()
                    .join(" | "),
                vec!["---"; section.columns.len()].join(" | ")
            ));
            ht.push_str("<table><thead><tr>");
            for column in &section.columns {
                ht.push_str(&format!("<th>{}</th>", html_escape(column)));
            }
            ht.push_str("</tr></thead><tbody>");
            for row in section.rows.iter().take(20) {
                md.push_str(&format!(
                    "| {} |\n",
                    row.iter()
                        .map(|s| markdown_escape(s))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
                ht.push_str("<tr>");
                for value in row {
                    ht.push_str(&format!("<td>{}</td>", html_escape(value)));
                }
                ht.push_str("</tr>");
            }
            ht.push_str("</tbody></table>");
        }
        ht.push_str("</section>");
        add(md, ht);
    }
    if omitted {
        markdown.push_str(&format!("\n{omission}\n"));
        html.push_str(&format!("<p>{omission}</p>"));
    }
    (markdown, format!("{html_start}{html}{ending}"))
}
