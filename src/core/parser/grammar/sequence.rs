// TODO:
// _trim_to_terminator (function)
fn position_metas(
    metas: &[Indent],              // Assuming Indent is a struct or type alias
    non_code: &[Box<dyn Segment>], // Assuming BaseSegment is a struct or type alias
) -> Vec<Box<dyn Segment>> {
    // Assuming BaseSegment can be cloned, or you have a way to handle ownership transfer

    // Check if all metas have a non-negative indent value
    if metas.iter().all(|m| m.indent_val >= 0) {
        let mut result: Vec<Box<dyn Segment>> = Vec::new();

        // Append metas first, then non-code elements
        for meta in metas {
            result.push(meta.clone().boxed()); // Assuming clone is possible or some equivalent
        }
        for segment in non_code {
            result.push(segment.clone()); // Assuming clone is possible or some equivalent
        }

        result
    } else {
        let mut result: Vec<Box<dyn Segment>> = Vec::new();

        // Append non-code elements first, then metas
        for segment in non_code {
            result.push(segment.clone()); // Assuming clone is possible or some equivalent
        }
        for meta in metas {
            result.push(meta.clone().boxed()); // Assuming clone is possible or some equivalent
        }

        result
    }
}

use std::collections::HashSet;

use itertools::enumerate;

use crate::{
    core::parser::{
        context::ParseContext,
        match_result::MatchResult,
        matchable::Matchable,
        segments::{base::Segment, meta::Indent},
        types::ParseMode,
    },
    traits::Boxed,
};

#[derive(Debug, Clone)]
pub struct Sequence {
    elements: Vec<Box<dyn Matchable>>,
    allow_gaps: bool,
    is_optional: bool,
}

impl Sequence {
    fn new(elements: Vec<Box<dyn Matchable>>) -> Self {
        Self {
            elements,
            allow_gaps: true,
            is_optional: false,
        }
    }

    fn with_allow_gaps(mut self, allow_gaps: bool) -> Self {
        self.allow_gaps = allow_gaps;
        self
    }
}

impl PartialEq for Sequence {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl Matchable for Sequence {
    fn is_optional(&self) -> bool {
        self.is_optional
    }

    fn simple(
        &self,
        parse_context: &ParseContext,
        crumbs: Option<Vec<&str>>,
    ) -> Option<(HashSet<String>, HashSet<String>)> {
        todo!()
    }

    fn match_segments(
        &self,
        segments: Vec<Box<dyn Segment>>,
        parse_context: &mut ParseContext,
    ) -> MatchResult {
        let mut matched_segments = Vec::new();
        let mut unmatched_segments = segments.clone();
        let tail = Vec::new();
        let first_match = true;

        // Buffers of segments, not yet added.
        let mut meta_buffer = Vec::new();
        let mut non_code_buffer = Vec::new();

        for (idx, elem) in enumerate(&self.elements) {
            // 1. Handle any metas or conditionals.
            // We do this first so that it's the same whether we've run
            // out of segments or not.
            // If it's a conditional, evaluate it.
            // In both cases, we don't actually add them as inserts yet
            // because their position will depend on what types we accrue.
            if let Some(indent) = elem.as_any().downcast_ref::<Indent>() {
                meta_buffer.push(indent.clone());
                continue;
            }

            // 2. Handle any gaps in the sequence.
            // At this point we know the next element isn't a meta or conditional
            // so if we're going to look for it we need to work up to the next
            // code element (if allowed)
            if self.allow_gaps && !matched_segments.is_empty() {
                // First, if we're allowing gaps, consume any non-code.
                // NOTE: This won't consume from the end of a sequence
                // because this happens only in the run up to matching
                // another element. This is as designed. It also won't
                // happen at the *start* of a sequence either.
                let mut found = false;
                for (idx, segment) in unmatched_segments.iter().enumerate() {
                    if segment.is_code() {
                        non_code_buffer.extend_from_slice(&unmatched_segments[..idx]);
                        unmatched_segments.drain(..idx);
                        found = true;
                        break;
                    }
                }

                if !found {
                    non_code_buffer.append(&mut unmatched_segments);
                }
            }

            // 4. Match the current element against the current position.
            let elem_match =
                parse_context.deeper_match(format!("Sequence-@{idx}"), false, &[], None, |this| {
                    elem.match_segments(unmatched_segments.clone(), this)
                });

            if !elem_match.has_match() {
                // If we can't match an element, we should ascertain whether it's
                // required. If so then fine, move on, but otherwise we should
                // crash out without a match. We have not matched the sequence.
                if elem.is_optional() {
                    // Pass this one and move onto the next element.
                    continue;
                }

                if ParseMode::Strict == ParseMode::Strict {
                    // In a strict mode, failing to match an element means that
                    // we don't match anything.
                    return MatchResult::from_unmatched(&segments);
                }
            }

            // 5. Successful match: Update the buffers.
            // First flush any metas along with the gap.
            let segments = position_metas(&meta_buffer, &non_code_buffer);
            matched_segments.extend(segments);
            non_code_buffer = Vec::new();
            meta_buffer = Vec::new();

            // Add on the match itself
            matched_segments.extend(elem_match.matched_segments);
            unmatched_segments = elem_match.unmatched_segments;
            // parse_context.update_progress(matched_segments)

            if first_match && ParseMode::Strict == ParseMode::GreedyOnceStarted {
                unimplemented!()
            }
        }

        // If we get to here, we've matched all of the elements (or skipped them).
        // Return successfully.
        unmatched_segments.extend(tail);
        MatchResult {
            matched_segments,
            unmatched_segments,
        }
    }

    fn cache_key(&self) -> String {
        todo!()
    }
}

// Bracketed (class)

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            dialects::init::{dialect_selector, get_default_dialect},
            parser::{
                context::ParseContext,
                markers::PositionMarker,
                matchable::Matchable,
                parsers::StringParser,
                segments::{keyword::KeywordSegment, meta::Indent, test_functions::test_segments},
            },
        },
        helpers::ToMatchable,
        traits::Boxed,
    };

    use super::Sequence;

    #[test]
    fn test__parser__grammar_sequence() {
        let bs = StringParser::new(
            "bar",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let fs = StringParser::new(
            "foo",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let mut ctx = ParseContext::new(dialect_selector(get_default_dialect()).unwrap());

        let g = Sequence::new(vec![bs.clone(), fs.clone()]);
        let gc = Sequence::new(vec![bs, fs]).with_allow_gaps(false);

        let match_result = g.match_segments(test_segments(), &mut ctx);

        assert_eq!(match_result.matched_segments[0].get_raw().unwrap(), "bar");
        assert_eq!(
            match_result.matched_segments[1].get_raw().unwrap(),
            test_segments()[1].get_raw().unwrap()
        );
        assert_eq!(match_result.matched_segments[2].get_raw().unwrap(), "foo");
        assert_eq!(match_result.len(), 3);

        assert!(!gc.match_segments(test_segments(), &mut ctx).has_match());

        assert!(!g
            .match_segments(test_segments()[1..].to_vec(), &mut ctx)
            .has_match());
    }

    #[test]
    fn test__parser__grammar_sequence_nested() {
        let bs = StringParser::new(
            "bar",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let fs = StringParser::new(
            "foo",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let bas = StringParser::new(
            "baar",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let g = Sequence::new(vec![Sequence::new(vec![bs, fs]).boxed(), bas]);

        let mut ctx = ParseContext::new(dialect_selector(get_default_dialect()).unwrap());

        assert!(
            !g.match_segments(test_segments()[..2].to_vec(), &mut ctx)
                .has_match(),
            "Expected no match, but a match was found."
        );

        let segments = g.match_segments(test_segments(), &mut ctx).matched_segments;
        assert_eq!(segments[0].get_raw().unwrap(), "bar");
        assert_eq!(
            segments[1].get_raw().unwrap(),
            test_segments()[1].get_raw().unwrap()
        );
        assert_eq!(segments[2].get_raw().unwrap(), "foo");
        assert_eq!(segments[3].get_raw().unwrap(), "baar");
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn test__parser__grammar_sequence_indent() {
        let bs = StringParser::new(
            "bar",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let fs = StringParser::new(
            "foo",
            |segment| {
                KeywordSegment::new(
                    segment.get_raw().unwrap(),
                    segment.get_position_marker().unwrap(),
                )
                .boxed()
            },
            None,
            false,
            None,
        )
        .boxed();

        let g = Sequence::new(vec![
            Indent::new(PositionMarker::default()).to_matchable(),
            bs,
            fs,
        ]);
        let mut ctx = ParseContext::new(dialect_selector(get_default_dialect()).unwrap());
        let segments = g.match_segments(test_segments(), &mut ctx).matched_segments;

        assert_eq!(segments[0].get_type(), "indent");
        assert_eq!(segments[1].get_type(), "kw");
    }
}