use std::collections::HashMap;

use crate::html::{escape_html, layout};
use super::content::{PROGRAM_TIPS_BODY, TUTORIAL_INTRO, TUTORIAL_STEPS};
use super::{guide_by_href, HelpGuide, HELP_GUIDES};

pub(crate) fn render_help_index(
    username: &str,
    hud: Option<&str>,
    welcome_banner: &str,
) -> String {
    let mut body = String::from(r#"<div class="help-page">"#);
    body.push_str(r#"<div class="help-shell">"#);
    body.push_str(r#"<header class="help-header">"#);
    body.push_str(r#"<h1 class="help-page-title">Help center</h1>"#);
    body.push_str(
        r#"<p class="help-page-subtitle">Guides to get started, program smarter, and understand the mechanics.</p>"#,
    );
    body.push_str("</header>");
    if !welcome_banner.is_empty() {
        body.push_str(welcome_banner);
    }
    body.push_str(r#"<div class="help-card-grid">"#);
    for guide in HELP_GUIDES {
        body.push_str(&render_help_card(&guide));
    }
    body.push_str("</div></div></div>");

    layout("RoboMiner - Help", "help", username, hud, &body)
}

pub(crate) fn render_help_article(
    username: &str,
    hud: Option<&str>,
    guide_href: &str,
    step: Option<i64>,
) -> Option<String> {
    let guide = guide_by_href(guide_href)?;
    let content = if guide.href == "helpTutorial" {
        render_tutorial_content(tutorial_step_index(step))
    } else if guide.href == "helpProgramTips" {
        enrich_help_reference_content(&render_program_tips_content())
    } else {
        enrich_help_reference_content(guide.body)
    };
    let mut body = String::from(r#"<div class="help-page">"#);
    body.push_str(r#"<div class="help-shell help-reader">"#);
    body.push_str(&render_help_sidebar(Some(guide.href)));
    body.push_str(r#"<article class="help-article">"#);
    body.push_str(r#"<div class="help-article-content">"#);
    body.push_str(&content);
    body.push_str("</div></article></div></div>");

    Some(layout(
        &format!("RoboMiner - {}", guide.title),
        "help",
        username,
        hud,
        &body,
    ))
}

fn tutorial_step_index(step: Option<i64>) -> usize {
    match step {
        Some(value) if (1..=TUTORIAL_STEPS.len() as i64).contains(&value) => value as usize - 1,
        _ => 0,
    }
}

fn render_program_tips_content() -> String {
    format!(
        r#"<p class="help-spoiler-banner">Contains programming spoilers and example solutions.</p>{}"#,
        PROGRAM_TIPS_BODY
    )
}

fn enrich_help_reference_content(html: &str) -> String {
    let with_code = upgrade_help_code_blocks(html);
    let with_tables = wrap_help_tables(&with_code);
    let (with_anchors, toc) = add_help_heading_anchors(&with_tables);
    let mut content = String::new();
    if let Some(toc_markup) = render_help_article_toc(&toc) {
        content.push_str(&toc_markup);
    }
    content.push_str(&with_anchors);
    content
}

fn upgrade_help_code_blocks(html: &str) -> String {
    const OPEN: &str = "<p class=\"codeexample\">";
    let mut result = String::new();
    let mut rest = html;
    while let Some(pos) = rest.find(OPEN) {
        result.push_str(&rest[..pos]);
        let after_open = &rest[pos + OPEN.len()..];
        if let Some(end) = after_open.find("</p>") {
            result.push_str(r#"<pre class="help-code-block"><code>"#);
            result.push_str(after_open[..end].trim_start());
            result.push_str("</code></pre>");
            rest = &after_open[end + 4..];
        } else {
            result.push_str(&rest[pos..]);
            return result;
        }
    }
    result.push_str(rest);
    result
}

fn wrap_help_tables(html: &str) -> String {
    const OPEN: &str = "<table class=\"helptable\">";
    let mut result = String::new();
    let mut rest = html;
    while let Some(pos) = rest.find(OPEN) {
        result.push_str(&rest[..pos]);
        let after_open = &rest[pos + OPEN.len()..];
        if let Some(end) = after_open.find("</table>") {
            result.push_str(r#"<div class="help-table-wrap"><table class="helptable">"#);
            result.push_str(&after_open[..end]);
            result.push_str("</table></div>");
            rest = &after_open[end + 8..];
        } else {
            result.push_str(&rest[pos..]);
            return result;
        }
    }
    result.push_str(rest);
    result
}

fn add_help_heading_anchors(html: &str) -> (String, Vec<(String, String)>) {
    let mut output = String::with_capacity(html.len() + 128);
    let mut toc = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut rest = html;

    while let Some(start) = rest.find("<h2>") {
        output.push_str(&rest[..start]);
        let after_open = &rest[start + 4..];
        let Some(end) = after_open.find("</h2>") else {
            output.push_str(&rest[start..]);
            return (output, toc);
        };
        let title = after_open[..end].trim();
        let base_slug = help_heading_slug(title);
        let entry = slug_counts.entry(base_slug.clone()).or_insert(0);
        *entry += 1;
        let id = if *entry == 1 {
            base_slug
        } else {
            format!("{base_slug}-{}", entry)
        };
        toc.push((id.clone(), title.to_string()));
        output.push_str(&format!(r#"<h2 id="{id}">{}</h2>"#, escape_html(title)));
        rest = &after_open[end + 5..];
    }
    output.push_str(rest);
    (output, toc)
}

fn render_help_article_toc(entries: &[(String, String)]) -> Option<String> {
    if entries.len() < 3 {
        return None;
    }
    let mut toc = String::from(
        r#"<nav class="help-article-toc" aria-label="On this page"><h2 class="help-article-toc-title">On this page</h2><ul class="help-article-toc-list">"#,
    );
    for (id, title) in entries {
        toc.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>",
            escape_html(id),
            escape_html(title),
        ));
    }
    toc.push_str("</ul></nav>");
    Some(toc)
}

fn help_heading_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut previous_hyphen = false;
    for ch in title.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_hyphen = false;
        } else if !previous_hyphen && !slug.is_empty() {
            slug.push('-');
            previous_hyphen = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug.push_str("section");
    }
    slug
}

fn tutorial_action_links(step_index: usize) -> Vec<String> {
    match step_index {
        0 => vec![render_help_action_link("miningQueue", "Go to mining queue →")],
        1 => vec![render_help_action_link("achievements", "Go to achievements →")],
        2 => vec![render_help_action_link("miningResults", "Go to mining results →")],
        3 => vec![
            render_help_action_link("shop", "Go to shop →"),
            render_help_action_link("robot", "Go to robots →"),
        ],
        4 => vec![render_help_action_link("editCode", "Go to edit code →")],
        _ => Vec::new(),
    }
}

fn render_tutorial_content(step_index: usize) -> String {
    let step = &TUTORIAL_STEPS[step_index];
    let mut content = String::from(r#"<div class="help-tutorial">"#);
    content.push_str(r#"<header class="help-tutorial-header">"#);
    content.push_str(r#"<h1>Tutorial</h1>"#);
    content.push_str(&format!(
        r#"<p class="help-tutorial-progress">Step {} of {}</p>"#,
        step_index + 1,
        TUTORIAL_STEPS.len()
    ));
    content.push_str("</header>");
    if step_index == 0 {
        content.push_str(TUTORIAL_INTRO);
    }
    content.push_str(r#"<section class="help-tutorial-step">"#);
    content.push_str(&format!("<h2>Step {}: {}</h2>", step_index + 1, step.title));
    content.push_str(step.body);
    for link in tutorial_action_links(step_index) {
        content.push_str(&link);
    }
    content.push_str("</section>");
    content.push_str(&render_tutorial_nav(step_index));
    content.push_str("</div>");
    content
}

fn render_tutorial_nav(step_index: usize) -> String {
    let total = TUTORIAL_STEPS.len();
    let mut nav = String::from(r#"<nav class="help-tutorial-nav" aria-label="Tutorial steps">"#);
    if step_index > 0 {
        nav.push_str(&format!(
            r#"<a class="help-tutorial-nav-link" href="helpTutorial?step={}">Previous step</a>"#,
            step_index
        ));
    } else {
        nav.push_str(
            r#"<span class="help-tutorial-nav-link help-tutorial-nav-link-disabled">Previous step</span>"#,
        );
    }
    if step_index + 1 < total {
        nav.push_str(&format!(
            r#"<a class="help-tutorial-nav-link help-tutorial-nav-link-primary" href="helpTutorial?step={}">Next step</a>"#,
            step_index + 2
        ));
    } else {
        nav.push_str(
            r#"<a class="help-tutorial-nav-link help-tutorial-nav-link-primary" href="helpProgramTips">Continue to programming tips</a>"#,
        );
    }
    nav.push_str("</nav>");
    nav
}

fn render_help_action_link(href: &str, label: &str) -> String {
    format!(
        r#"<p class="help-action-link-wrap"><a class="help-action-link" href="{}">{}</a></p>"#,
        href,
        escape_html(label)
    )
}
fn render_help_card(guide: &HelpGuide) -> String {
    let href = if guide.href == "helpTutorial" {
        "helpTutorial?step=1"
    } else {
        guide.href
    };
    format!(
        r#"<a class="help-card" href="{}"><span class="help-card-tag">{}</span><h2 class="help-card-title">{}</h2><p class="help-card-summary">{}</p></a>"#,
        href,
        escape_html(guide.tag),
        escape_html(guide.title),
        escape_html(guide.summary),
    )
}

fn render_help_sidebar(active_href: Option<&str>) -> String {
    let mut sidebar = String::from(r#"<aside class="help-sidebar">"#);
    sidebar.push_str(r#"<nav class="help-nav" aria-label="Help guides">"#);
    sidebar.push_str(r#"<a class="help-nav-home" href="help">Help center</a>"#);
    sidebar.push_str(r#"<ul class="help-nav-list">"#);
    for guide in HELP_GUIDES {
        let item_class = if active_href == Some(guide.href) {
            "help-nav-item help-nav-item-active"
        } else {
            "help-nav-item"
        };
        let href = if guide.href == "helpTutorial" {
            "helpTutorial?step=1"
        } else {
            guide.href
        };
        sidebar.push_str(&format!(
            r#"<li class="{item_class}"><a href="{}">{}</a><span class="help-nav-tag">{}</span></li>"#,
            href,
            escape_html(guide.title),
            escape_html(guide.tag),
        ));
    }
    sidebar.push_str("</ul></nav></aside>");
    sidebar
}

pub(crate) fn welcome_banner_markup() -> &'static str {
    r#"<p class="help-welcome-banner">Account created. Start with the <a href="helpTutorial?step=1">Tutorial</a> to learn the basics.</p>"#
}

pub(crate) fn render_page_help_link(href: &str, label: &str) -> String {
    format!(
        r#"<a class="page-help-link" href="{}">{}</a>"#,
        href,
        escape_html(label)
    )
}

pub(crate) fn render_page_help_hint(intro: &str, href: &str, link_label: &str) -> String {
    format!(
        r#"<p class="page-help-hint">{} {}</p>"#,
        intro,
        render_page_help_link(href, link_label)
    )
}

pub(crate) fn render_page_help_hint_line(links: &[(&str, &str)]) -> String {
    let mut markup = String::from(r#"<p class="page-help-hint">Need help? "#);
    for (index, (href, label)) in links.iter().enumerate() {
        if index > 0 {
            markup.push_str(" · ");
        }
        markup.push_str(&render_page_help_link(href, label));
    }
    markup.push_str("</p>");
    markup
}

#[cfg(test)]
mod tests {
    use super::help_heading_slug;

    #[test]
    fn help_heading_slug_normalizes_titles() {
        assert_eq!(help_heading_slug("Ore Container"), "ore-container");
        assert_eq!(help_heading_slug("Repeated mining"), "repeated-mining");
    }
}
