use itertools::Itertools;

use crate::core::rules::base::{LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, RootOnlyCrawler};
use crate::helpers::Boxed;
use crate::utils::functional::segments::Segments;

#[derive(Debug, Default)]
pub struct RuleLT13 {}

impl Rule for RuleLT13 {
    fn crawl_behaviour(&self) -> Box<dyn Crawler> {
        RootOnlyCrawler::default().boxed()
    }

    fn eval(&self, context: RuleContext) -> Vec<LintResult> {
        let mut raw_segments = Vec::new();

        for seg in context.segment.recursive_crawl_all(false) {
            if !seg.get_segments().is_empty() {
                continue;
            }

            if matches!(seg.get_type(), "newline" | "whitespace" | "indent" | "dedent") {
                raw_segments.push(seg);
                continue;
            }

            let raw_stack =
                Segments::from_vec(raw_segments.clone(), context.templated_file.clone());
            // Non-whitespace segment.
            if !raw_stack.all(Some(|seg| seg.is_meta())) {
                return vec![LintResult::new(
                    context.segment.into(),
                    raw_stack.into_iter().map(|seg| LintFix::delete(seg)).collect_vec(),
                    None,
                    None,
                    None,
                )];
            } else {
                break;
            }
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::RuleLT13;
    use crate::api::simple::{fix, lint};
    use crate::core::rules::base::{Erased, ErasedRule};

    fn rules() -> Vec<ErasedRule> {
        vec![RuleLT13::default().erased()]
    }

    #[test]
    fn test_pass_leading_whitespace_statement() {
        let lints =
            lint("SELECT foo FROM bar\n".into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    #[ignore]
    fn test_pass_leading_whitespace_comment() {
        let lints = lint(
            "/*I am a comment*/\nSELECT foo FROM bar\n".into(),
            "ansi".into(),
            rules(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    fn test_pass_leading_whitespace_inline_comment() {
        let lints = lint(
            "--I am a comment\nSELECT foo FROM bar\n".into(),
            "ansi".into(),
            rules(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    #[ignore = "dialect: bigquery"]
    fn test_pass_leading_whitespace_inline_comment_hash() {}

    #[test]
    #[ignore = "parser needs further development"]
    fn test_pass_leading_whitespace_jinja_comment() {
        let lints = lint(
            "{# I am a comment #}\nSELECT foo FROM bar\n".into(),
            "ansi".into(),
            rules(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    #[ignore = "parser needs further development"]
    fn test_pass_leading_whitespace_jinja_if() {
        let lints = lint(
            "{% if True %}\nSELECT foo\nFROM bar;\n{% endif %}\n".into(),
            "ansi".into(),
            rules(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    #[ignore = "parser needs further development"]
    fn test_pass_leading_whitespace_jinja_for() {
        let lints = lint(
            "{% for item in range(10) %}\nSELECT foo_{{ item }}\nFROM bar;\n{% endfor %}\n".into(),
            "ansi".into(),
            rules(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(lints, []);
    }

    #[test]
    fn test_fail_leading_whitespace_statement() {
        let fixed = fix("\n  SELECT foo FROM bar\n".into(), rules());
        println!("{fixed:?}");
    }
}