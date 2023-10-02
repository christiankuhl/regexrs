mod combinators;

use combinators::*;

const SPECIAL_CHARS: [char; 14] = [
    '.', '^', '$', '*', '+', '?', '{', '}', '[', ']', '\\', '|', '(', ')',
];

const ESCAPES: [char; 7] = ['a', 'b', 'f', 'n', 'r', 't', 'v'];

const SEQ_CHARS: [char; 10] = ['A', 'b', 'B', 'd', 'D', 's', 'S', 'w', 'W', 'Z'];

#[derive(PartialEq, Debug)]
pub(crate) enum Quantifier {
    ZeroOrMore,
    OneOrMore,
    Maybe,
    LazyZeroOrMore,
    LazyOneOrMore,
    LazyMaybe,
    Once,
    AtLeast(usize),
    Between(usize, usize),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Sign {
    Inclusive,
    Exclusive,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Token {
    Range(char, char),
    Literal(char),
}

#[derive(Debug, PartialEq)]
pub(crate) struct CharacterClass {
    sign: Sign,
    items: Vec<Token>,
    quantifier: Quantifier,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Element {
    Class(CharacterClass),
    Sequence(SpecialSequence, Quantifier),
    CaptureGroup(Term, Quantifier),
}

#[derive(Debug, PartialEq)]
pub(crate) enum SpecialSequence {
    // .
    // Matches any character
    AnyCharacter,
    // \A
    // Matches only at the start of the string.
    Start,
    // \b
    // Matches the empty string, but only at the beginning or end of a word. A word is defined as a sequence of word characters. Note that formally, \b is defined as the
    //  boundary between a \w and a \W character (or vice versa), or between \w and the beginning/end of the string. This means that r'\bfoo\b' matches 'foo', 'foo.', '(foo)',
    // 'bar foo baz' but not 'foobar' or 'foo3'.
    WordBoundary,
    // \B
    // Matches the empty string, but only when it is not at the beginning or end of a word. This means that r'py\B' matches 'python', 'py3', 'py2', but not 'py', 'py.', or
    // 'py!'. \B is just the opposite of \b, so word characters in Unicode patterns are Unicode alphanumerics or the underscore, although this can be changed by using the ASCII
    // flag. Word boundaries are determined by the current locale if the LOCALE flag is used.
    WithinWord,
    // \d
    // Matches any Unicode decimal digit (that is, any character in Unicode character category [Nd]). This includes [0-9], and also many other digit characters.
    Digit,
    // \D
    // Matches any character which is not a decimal digit. This is the opposite of \d. If the ASCII flag is used this becomes the equivalent of [^0-9].
    NotDigit,
    // \s
    // Matches Unicode whitespace characters (which includes [ \t\n\r\f\v], and also many other characters,
    Whitespace,
    // \S
    // Matches any character which is not a whitespace character. This is the opposite of \s..
    NotWhitespace,
    // \w
    // Matches Unicode word characters; this includes alphanumeric characters (as defined by str.isalnum()) as well as the underscore (_).
    WordCharacter,
    // \W
    // Matches any character which is not a word character. This is the opposite of \w.
    NotWordCharacter,
    // \Z
    // Matches only at the end of the string
    End,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Term {
    left_anchored: bool,
    right_anchored: bool,
    elements: Vec<Element>,
}

fn character_class(input: &str) -> ParseResult<Element> {
    pair(
        right(
            match_literal("["),
            left(inside_character_class, match_literal("]")),
        ),
        maybe(parse_quantifier),
    )
    .map(|((sign, items), quantifier)| CharacterClass {
        sign,
        items,
        quantifier: quantifier.unwrap_or(Quantifier::Once),
    })
    .map(|c| Element::Class(c))
    .parse(input)
}

fn parse_quantifier(input: &str) -> ParseResult<Quantifier> {
    match_literal("+?").map(|_| Quantifier::LazyOneOrMore)
        .or(match_literal("*?").map(|_| Quantifier::LazyZeroOrMore))
        .or(match_literal("??").map(|_| Quantifier::LazyMaybe))
        .or(match_literal("+").map(|_| Quantifier::OneOrMore))
        .or(match_literal("*").map(|_| Quantifier::ZeroOrMore))
        .or(match_literal("?").map(|_| Quantifier::Maybe))
        .or(left(
            right(match_literal("{"), sep_by(parse_int, ",")),
            match_literal("}"),
        )
        .map(|values| {
            if values.len() == 1 {
                Quantifier::AtLeast(values[0])
            } else {
                Quantifier::Between(values[0], values[1])
            }
        }))
        .parse(input)
}

fn parse_int(input: &str) -> ParseResult<usize> {
    one_or_more(any_char.pred(|&c| c.is_digit(10)))
        .map(|value| {
            let value: String = value.iter().collect();
            usize::from_str_radix(&value, 10).unwrap()
        })
        .parse(input)
}

fn inside_character_class(input: &str) -> ParseResult<(Sign, Vec<Token>)> {
    pair(parse_sign, one_or_more(character_range.or(single_item))).parse(input)
}

fn parse_sign(input: &str) -> ParseResult<Sign> {
    maybe(match_literal("^"))
        .map(|s| match s {
            Some(_) => Sign::Exclusive,
            None => Sign::Inclusive,
        })
        .parse(input)
}

fn single_item(input: &str) -> ParseResult<Token> {
    not_backslash
        .pred(|&c| c != ']')
        .map(|c| Token::Literal(c))
        .parse(input)
}

fn character_range(input: &str) -> ParseResult<Token> {
    if let Ok((values, rest)) = sep_by(not_backslash.pred(|&c| c != '-'), "-").parse(input) {
        if values.len() == 2 {
            Ok((Token::Range(values[0], values[1]), rest))
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

fn regular_character(input: &str) -> ParseResult<char> {
    any_char.pred(|c| !SPECIAL_CHARS.contains(c)).parse(input)
}

fn not_backslash(input: &str) -> ParseResult<char> {
    any_char.pred(|&c| c != '\\').parse(input)
}

fn regex_term(input: &str) -> ParseResult<Term> {
    pair(
        maybe(match_literal("^")),
        pair(
            one_or_more(
                special_sequence
                    .or(character_class)
                    .or(quantified_ordinary_character)
                    .or(match_group),
            ),
            maybe(match_literal("$")),
        ),
    )
    .map(|(start, (elements, end))| Term {
        left_anchored: start.is_some(),
        right_anchored: end.is_some(),
        elements,
    })
    .parse(input)
}

pub(crate) fn parse_regex(input: &str) -> ParseResult<Vec<Term>> {
    if let Ok((value, rest)) = sep_by(regex_term, "|").parse(input) {
        if rest == "" {
            return Ok((value, rest))
        } else {
            return Err(())
        }
    }
    Err(())
}

fn special_sequence(input: &str) -> ParseResult<Element> {
    pair(
        match_literal(".").map(|_| '.').or(right(
            match_literal("\\"),
            any_char.pred(|c| SEQ_CHARS.contains(c)),
        )),
        maybe(parse_quantifier),
    )
    .map(|(c, q)| {
        let seq = match c {
            '.' => SpecialSequence::AnyCharacter,
            'A' => SpecialSequence::Start,
            'b' => SpecialSequence::WordBoundary,
            'B' => SpecialSequence::WithinWord,
            'd' => SpecialSequence::Digit,
            'D' => SpecialSequence::NotDigit,
            's' => SpecialSequence::Whitespace,
            'S' => SpecialSequence::NotWhitespace,
            'w' => SpecialSequence::WordCharacter,
            'W' => SpecialSequence::NotWordCharacter,
            'Z' => SpecialSequence::End,
            _ => unreachable!(),
        };
        Element::Sequence(seq, q.unwrap_or(Quantifier::Once))
    })
    .parse(input)
}

fn match_group(input: &str) -> ParseResult<Element> {
    pair(left(right(match_literal("("), regex_term), match_literal(")")), maybe(parse_quantifier))
        .map(|(t, q)| Element::CaptureGroup(t, q.unwrap_or(Quantifier::Once)))
        .parse(input)
}

fn escaped_character(input: &str) -> ParseResult<char> {
    right(
        match_literal("\\"),
        any_char.pred(|c| SPECIAL_CHARS.contains(c)),
    )
    .parse(input)
}

fn quantified_ordinary_character(input: &str) -> ParseResult<Element> {
    pair(
        regular_character.or(escaped_character),
        maybe(parse_quantifier),
    )
    .map(|(c, q)| {
        Element::Class(CharacterClass {
            sign: Sign::Inclusive,
            quantifier: q.unwrap_or(Quantifier::Once),
            items: vec![Token::Literal(c)],
        })
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn class_parser() {
        assert_eq!(
            character_class("[^a-z]{4, 5}xyz"),
            Ok((
                Element::Class(CharacterClass {
                    sign: Sign::Exclusive,
                    items: vec![Token::Range('a', 'z')],
                    quantifier: Quantifier::Between(4, 5),
                }),
                "xyz"
            ))
        );
    }

    #[test]
    fn sep_by_works() {
        assert_eq!(
            parse_regex("(ab)+c"),
            Ok((vec![
                Term { 
                    left_anchored: false, 
                    right_anchored: false, 
                    elements: vec![
                        Element::CaptureGroup(
                            Term { 
                                left_anchored: false, 
                                right_anchored: false, 
                                elements: vec![
                                    Element::Class(CharacterClass { sign: Sign::Inclusive, items: vec![Token::Literal('a')], quantifier: Quantifier::Once }),
                                    Element::Class(CharacterClass { sign: Sign::Inclusive, items: vec![Token::Literal('b')], quantifier: Quantifier::Once })
                                ]
                            }, Quantifier::OneOrMore
                        ),
                        Element::Class(CharacterClass { sign: Sign::Inclusive, items: vec![Token::Literal('c')], quantifier: Quantifier::Once })
                    ] 
                }
            ],""))
        );
    }
}
