use crate::rule::{Fix, LintError, LintResult, LintWarning, Rule, Severity};
use crate::utils::mkdocs_common::{MKDOCS_CONTENT_INDENT, get_line_indent};
use crate::utils::mkdocs_footnotes;

/// Rule MD061: Footnote indentation
///
/// See [docs/md061.md](../../docs/md061.md) for configuration and examples.
///
/// Ensures continuation lines in multi-line footnote definitions are indented by
/// four spaces relative to the definition marker so that Markdown renderers treat
/// them as part of the same footnote instead of a top-level code block.
#[derive(Debug, Default, Clone)]
pub struct MD061FootnoteIndentation;

impl MD061FootnoteIndentation {
    fn create_warning(
        &self,
        line_idx: usize,
        line: &str,
        base_indent: usize,
        ctx: &crate::lint_context::LintContext,
    ) -> LintWarning {
        let required_indent = base_indent + MKDOCS_CONTENT_INDENT;
        let current_indent = get_line_indent(line);
        let trimmed = line.trim_start();
        let line_index = &ctx.line_index;

        let replacement = format!("{}{}", " ".repeat(required_indent), trimmed);
        let range = line_index.line_col_to_byte_range_with_length(line_idx + 1, 1, current_indent);

        LintWarning {
            rule_name: Some(self.name().to_string()),
            message: format!(
                "Footnote continuation lines should be indented by {} spaces",
                MKDOCS_CONTENT_INDENT
            ),
            line: line_idx + 1,
            column: current_indent + 1,
            end_line: line_idx + 1,
            end_column: current_indent + trimmed.len() + 1,
            severity: Severity::Warning,
            fix: Some(Fix { range, replacement }),
        }
    }
}

impl Rule for MD061FootnoteIndentation {
    fn name(&self) -> &'static str {
        "MD061"
    }

    fn description(&self) -> &'static str {
        "Continuation lines in footnotes should be indented by four spaces"
    }

    fn should_skip(&self, ctx: &crate::lint_context::LintContext) -> bool {
        !ctx.content.contains("[^")
    }

    fn check(&self, ctx: &crate::lint_context::LintContext) -> LintResult {
        let mut warnings = Vec::new();
        let lines: Vec<&str> = ctx.content.lines().collect();
        let mut current_footnote: Option<(usize, usize)> = None; // (base_indent, definition_line)

        for (i, line) in lines.iter().enumerate() {
            if mkdocs_footnotes::is_footnote_definition(line) {
                current_footnote = Some((mkdocs_footnotes::get_footnote_indent(line).unwrap_or(0), i));
                continue;
            }

            if let Some((base_indent, _definition_line)) = current_footnote {
                if line.trim().is_empty() {
                    continue;
                }

                let indent = get_line_indent(line);

                if indent == 0 {
                    current_footnote = None;
                    continue;
                }

                if indent >= base_indent + MKDOCS_CONTENT_INDENT {
                    if mkdocs_footnotes::is_footnote_definition(line) {
                        current_footnote = Some((mkdocs_footnotes::get_footnote_indent(line).unwrap_or(0), i));
                    }
                    continue;
                }

                // Non-empty line with partial indentation should be fixed.
                warnings.push(self.create_warning(i, line, base_indent, ctx));
            }
        }

        Ok(warnings)
    }

    fn fix(&self, ctx: &crate::lint_context::LintContext) -> Result<String, LintError> {
        let lines: Vec<&str> = ctx.content.lines().collect();
        let mut fixed_lines = Vec::with_capacity(lines.len());
        let mut current_base_indent: Option<usize> = None;

        for line in &lines {
            if mkdocs_footnotes::is_footnote_definition(line) {
                current_base_indent = Some(mkdocs_footnotes::get_footnote_indent(line).unwrap_or(0));
                fixed_lines.push((*line).to_string());
                continue;
            }

            if let Some(base_indent) = current_base_indent {
                if line.trim().is_empty() {
                    fixed_lines.push((*line).to_string());
                    continue;
                }

                let indent = get_line_indent(line);

                if indent == 0 {
                    current_base_indent = None;
                    fixed_lines.push((*line).to_string());
                    continue;
                }

                if indent >= base_indent + MKDOCS_CONTENT_INDENT {
                    fixed_lines.push((*line).to_string());
                    continue;
                }

                let mut new_line = " ".repeat(base_indent + MKDOCS_CONTENT_INDENT);
                new_line.push_str(line.trim_start());
                fixed_lines.push(new_line);
                continue;
            }

            fixed_lines.push((*line).to_string());
        }

        let mut result = fixed_lines.join("\n");
        if ctx.content.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn from_config(_config: &crate::config::Config) -> Box<dyn Rule>
    where
        Self: Sized,
    {
        Box::new(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_context::LintContext;

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

        let ctx = LintContext::new(content, crate::config::MarkdownFlavor::Standard);
        let result = rule.check(&ctx).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_missing_indent_is_flagged() {
        let rule = MD061FootnoteIndentation::default();
        let content = r#"Reference.[^1]

[^1]: First paragraph of the footnote.
  Second paragraph is missing indentation.
"#;

        let ctx = LintContext::new(content, crate::config::MarkdownFlavor::Standard);
        let result = rule.check(&ctx).unwrap();
        assert_eq!(result.len(), 1);
        assert!(
            result[0]
                .message
                .contains("Footnote continuation lines should be indented")
        );
    }

    #[test]
    fn test_fix_adds_required_indent() {
        let rule = MD061FootnoteIndentation::default();
        let content = r#"[^1]: Example footnote.
  Second line should be indented.
"#;

        let ctx = LintContext::new(content, crate::config::MarkdownFlavor::Standard);
        let result = rule.check(&ctx).unwrap();
        assert_eq!(result.len(), 1);

        let fix = result[0].fix.clone().expect("expected fix");
        assert!(fix.replacement.starts_with("    "));
    }

    #[test]
    fn test_nested_list_in_footnote_is_allowed() {
        let rule = MD061FootnoteIndentation::default();
        let content = r#"[^1]: Example footnote.
    - Bullet inside footnote.
        - Nested bullet remains inside footnote.
"#;

        let ctx = LintContext::new(content, crate::config::MarkdownFlavor::Standard);
        let result = rule.check(&ctx).unwrap();
        assert!(result.is_empty());
    }
}
