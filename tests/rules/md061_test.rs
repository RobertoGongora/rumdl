use rumdl_lib::lint_context::LintContext;
use rumdl_lib::rule::Rule;
use rumdl_lib::rules::MD061FootnoteIndentation;

#[test]
fn test_rule_name() {
    let rule = MD061FootnoteIndentation::default();
    assert_eq!(rule.name(), "MD061");
}

#[test]
fn test_valid_multi_paragraph_footnote() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"Reference.[^1]

[^1]: First paragraph of the footnote.
    Second paragraph remains part of the footnote.

    Third paragraph is also indented.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_missing_indent_is_flagged() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: First paragraph of the footnote.
  Second paragraph is missing indentation.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert_eq!(result.len(), 1);
    assert!(
        result[0]
            .message
            .contains("Footnote continuation lines should be indented")
    );
}

#[test]
fn test_nested_list_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Example footnote.
    - Bullet inside footnote.
        - Nested bullet remains inside footnote.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_blockquote_content_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with blockquotes.
    > Quote line one.
    > Quote line two.

    > Nested blockquote.
    > > Second level.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_fenced_code_block_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with fenced code.
    ```rust
    fn main() {
        println!("Hello");
    }
    ```
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_indented_code_block_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with indented code.
        let value = 42;
        println!("{}", value);
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_ordered_list_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with ordered list.
    1. First item.
    2. Second item.
        1. Nested ordered item.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_table_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with GFM table.
    | Column | Value |
    | ------ | ----- |
    | left   | right |
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_thematic_break_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with thematic break.
    ---
    Continued explanation after rule.
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_html_block_is_allowed() {
    let rule = MD061FootnoteIndentation::default();
    let content = r#"[^1]: Footnote with HTML block.
    <details>
        <summary>Summary</summary>
        <p>Inner paragraph</p>
    </details>
"#;

    let ctx = LintContext::new(content, rumdl_lib::config::MarkdownFlavor::Standard);
    let result = rule.check(&ctx).unwrap();
    assert!(result.is_empty());
}
