//! Inline emphasis for translated lines.
//!
//! A translated line sometimes needs one word picked out in the accent colour —
//! and *which* word that is belongs to the language, not to the render: PT-BR
//! stresses "e {{talvez}} eu responda", another language may stress a different
//! word in a different position. So the emphasis travels **inside the string**,
//! marked with `{{…}}`, and the render turns it into styled spans.
//!
//! [`parse`] answers *which segments are emphasised*; [`styled_line`] answers
//! *what emphasis looks like*. The split is deliberate: the accent colour is a
//! `Palette` value that changes with the theme, so only the render — which
//! already holds the palette — gets to know it.
//!
//! Malformed markup is never an error: an unclosed `{{` simply stays on screen
//! as literal text. Our content is baked into the binary and only we edit it, so
//! a typo shows up the first time the screen is run — loud enough, and it keeps
//! this module total.

use ratatui::style::{Color, Style};
use ratatui::text::Line;

/// Split `text` into `(fragment, emphasised)` runs at the `{{…}}` markers.
///
/// Fragments borrow from `text` — nothing is copied. Empty fragments are never
/// produced, so a marker at either end yields two segments, not three.
pub fn parse(text: &str) -> Vec<(&str, bool)> {
    let mut rest = text;

    let mut acc: Vec<(&str, bool)> = vec![];

    while let Some((before, after)) = rest.split_once("{{") {
        if let Some((styled, tail)) = after.split_once("}}") {
            if !before.is_empty() {
                acc.push((before, false));
            }

            if !styled.is_empty() {
                acc.push((styled, true));
            }

            rest = tail;
        } else {
            break;
        }
    }

    if !rest.is_empty() {
        acc.push((rest, false));
    }

    acc
}

/// Build a styled [`Line`] from a marked-up `text`: plain runs wear `base`,
/// emphasised runs wear `accent` (the accent replaces `base` rather than adding
/// to it — an emphasised word is not a dim word in a different colour).
pub fn styled_line(text: &str, base: Style, accent: Color) -> Line<'_> {
    todo!("Danilo: parse, then map each segment to a Span and collect into a Line")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse: where the emphasis falls ──────────────────────────────────────
    // The contract is a split, not a parser: walk the string, cut at `{{` and
    // `}}`, keep every fragment tied to its flag. Two invariants carry all the
    // edge cases — no empty fragments, and anything that isn't a complete
    // marker is ordinary text.

    #[test]
    fn text_without_markers_is_one_plain_segment() {
        assert_eq!(
            parse("Você está prestes a abrir uma porta para o desconhecido."),
            vec![(
                "Você está prestes a abrir uma porta para o desconhecido.",
                false
            )]
        );
    }

    #[test]
    fn a_marker_in_the_middle_splits_into_three_segments() {
        assert_eq!(
            parse("Para que {{SUED}} responda, elogie-o."),
            vec![
                ("Para que ", false),
                ("SUED", true),
                (" responda, elogie-o.", false),
            ]
        );
    }

    #[test]
    fn a_marker_at_the_start_yields_no_leading_empty_segment() {
        // The naive split produces an empty fragment before the marker. An empty
        // span renders nothing but still costs a Span — and it would make
        // segment counts lie about the line.
        assert_eq!(
            parse("{{Elogie}} o Sued antes de qualquer coisa."),
            vec![
                ("Elogie", true),
                (" o Sued antes de qualquer coisa.", false)
            ]
        );
    }

    #[test]
    fn a_marker_at_the_end_yields_no_trailing_empty_segment() {
        assert_eq!(parse("Faça {{uma}}"), vec![("Faça ", false), ("uma", true)]);
    }

    #[test]
    fn a_fully_marked_line_is_a_single_emphasised_segment() {
        assert_eq!(parse("{{ATENÇÃO}}"), vec![("ATENÇÃO", true)]);
    }

    #[test]
    fn two_markers_alternate_across_five_segments() {
        assert_eq!(
            parse("Elogie {{uma}} vez e pergunte com {{clareza}} depois."),
            vec![
                ("Elogie ", false),
                ("uma", true),
                (" vez e pergunte com ", false),
                ("clareza", true),
                (" depois.", false),
            ]
        );
    }

    #[test]
    fn an_unclosed_marker_stays_literal_text() {
        // Settled ruling: malformed markup is not an error. The braces render as
        // themselves, the mistake is visible on the first run, and this function
        // stays total — no Result, no panic, no silent swallowing of the line.
        assert_eq!(
            parse("Tenha cuidado {{com o que irá perguntar"),
            vec![("Tenha cuidado {{com o que irá perguntar", false)]
        );
    }

    #[test]
    fn a_closing_marker_without_an_opening_one_stays_literal_text() {
        assert_eq!(
            parse("A resposta virá do além}} sempre"),
            vec![("A resposta virá do além}} sempre", false)]
        );
    }

    #[test]
    fn no_segment_is_ever_empty() {
        // The invariant behind the two end-marker tests, stated once over the
        // shapes most likely to breed a stray empty fragment.
        let lines = [
            "{{tudo}}",
            "{{início}} do texto",
            "texto do {{fim}}",
            "{{a}}{{b}}",
            "sem marcador algum",
        ];

        for line in lines {
            for (fragment, _) in parse(line) {
                assert!(
                    !fragment.is_empty(),
                    "parse({line:?}) produced an empty fragment"
                );
            }
        }
    }

    #[test]
    fn the_fragments_rebuild_the_line_without_the_markers() {
        // Nothing is dropped and nothing is duplicated: concatenating the
        // fragments must give back exactly the text minus its markers.
        let marked = "Para que {{SUED}} responda, pergunte com {{clareza}}.";
        let expected = "Para que SUED responda, pergunte com clareza.";

        let rebuilt: String = parse(marked).into_iter().map(|(text, _)| text).collect();

        assert_eq!(rebuilt, expected);
    }

    // ── styled_line: what emphasis looks like ────────────────────────────────
    // Pure enough to pin: it builds spans, it doesn't touch a terminal. The one
    // rule worth nailing down is that the accent REPLACES the base style rather
    // than layering on top of it — the dim hint line's emphasised word must come
    // out bright, not dim-and-red.

    #[test]
    fn plain_text_becomes_a_single_span_wearing_the_base_style() {
        let base = Style::new().fg(Color::Gray);
        let line = styled_line("conexão com o além estabelecida.", base, Color::Red);

        assert_eq!(line.spans.len(), 1);
        assert_eq!(
            line.spans[0].content.as_ref(),
            "conexão com o além estabelecida."
        );
        assert_eq!(line.spans[0].style.fg, Some(Color::Gray));
    }

    #[test]
    fn an_emphasised_word_wears_the_accent_instead_of_the_base() {
        let base = Style::new().fg(Color::Gray);
        let line = styled_line("e {{talvez}} eu responda.", base, Color::Red);

        assert_eq!(line.spans.len(), 3);
        assert_eq!(
            line.spans[0].style.fg,
            Some(Color::Gray),
            "plain run keeps base"
        );
        assert_eq!(
            line.spans[1].content.as_ref(),
            "talvez",
            "the marked word is its own span"
        );
        assert_eq!(
            line.spans[1].style.fg,
            Some(Color::Red),
            "the emphasised run wears the accent"
        );
        assert_eq!(
            line.spans[2].style.fg,
            Some(Color::Gray),
            "and back to base"
        );
    }

    #[test]
    fn the_accent_does_not_inherit_the_bases_modifiers() {
        // The concrete case from `ask.rs`: a dim hint line with one bright word.
        // If the accent were layered onto the base, "talvez" would render dim.
        let dim = Style::new().add_modifier(ratatui::style::Modifier::DIM);
        let line = styled_line("e {{talvez}} eu responda.", dim, Color::Red);

        assert!(
            !line.spans[1]
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::DIM),
            "the emphasised word must not inherit the base's DIM"
        );
    }
}
