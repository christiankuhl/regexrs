use std::fmt::Debug;

pub(super) type ParseResult<'a, Output> = Result<(Output, &'a str), ()>;

pub(super) trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
    fn map<F, MappedOutput>(self, map_fn: F) -> BoxedParser<'a, MappedOutput>
    where
        Self: Sized + 'a,
        Output: 'a,
        MappedOutput: 'a,
        F: Fn(Output) -> MappedOutput + 'a,
    {
        BoxedParser::new(map(self, map_fn))
    }
    fn pred<F>(self, predicate: F) -> BoxedParser<'a, Output>
    where
        Self: Sized + 'a,
        Output: 'a,
        F: Fn(&Output) -> bool + 'a,
    {
        BoxedParser::new(pred(self, predicate))
    }
    fn or(self, parser: impl Parser<'a, Output> + 'a) -> BoxedParser<'a, Output>
    where
        Self: Sized + 'a,
        Output: 'a,
    {
        let alternative = move |input| self.parse(input).or_else(|_| parser.parse(input));
        BoxedParser::new(alternative)
    }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub(super) struct BoxedParser<'a, Output> {
    parser: Box<dyn Parser<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    fn new(parser: impl Parser<'a, Output> + 'a) -> Self {
        Self {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

pub(super) fn pair<'a, R1, R2>(
    parser1: impl Parser<'a, R1>,
    parser2: impl Parser<'a, R2>,
) -> impl Parser<'a, (R1, R2)> {
    move |input| {
        parser1.parse(input).and_then(|(result1, next_input)| {
            parser2
                .parse(next_input)
                .map(|(result2, rest)| ((result1, result2), rest))
        })
    }
}

pub(super) fn map<'a, F, A, B>(
    parser: impl Parser<'a, A>,
    map_fn: F,
) -> impl Fn(&'a str) -> ParseResult<'a, B>
where
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(result, rest)| (map_fn(result), rest))
    }
}

pub(super) fn left<'a, A, B>(
    left_parser: impl Parser<'a, A>,
    right_parser: impl Parser<'a, B>,
) -> impl Parser<'a, A> {
    map(pair(left_parser, right_parser), |(left, _right)| left)
}

pub(super) fn right<'a, A, B>(
    left_parser: impl Parser<'a, A>,
    right_parser: impl Parser<'a, B>,
) -> impl Parser<'a, B> {
    map(pair(left_parser, right_parser), |(_left, right)| right)
}

pub(super) fn pred<'a, A, F>(parser: impl Parser<'a, A>, predicate: F) -> impl Parser<'a, A>
where
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((result, rest)) = parser.parse(input) {
            if predicate(&result) {
                return Ok((result, rest));
            }
        }
        Err(())
    }
}

pub(super) fn one_or_more<'a, R>(parser: impl Parser<'a, R>) -> impl Parser<'a, Vec<R>> {
    move |input| {
        let mut result = Vec::new();
        let mut tmp_input;
        if let Ok((first, rest)) = parser.parse(input) {
            tmp_input = rest;
            result.push(first);
        } else {
            return Err(());
        }
        while let Ok((next, rest)) = parser.parse(tmp_input) {
            tmp_input = rest;
            result.push(next);
        }
        return Ok((result, tmp_input));
    }
}

pub(super) fn zero_or_more<'a, R>(parser: impl Parser<'a, R>) -> impl Parser<'a, Vec<R>> {
    move |input| {
        let mut result = Vec::new();
        let mut tmp_input = input;
        while let Ok((next, rest)) = parser.parse(tmp_input) {
            tmp_input = rest;
            result.push(next);
        }
        return Ok((result, tmp_input));
    }
}

pub(super) fn maybe<'a, R>(parser: impl Parser<'a, R>) -> impl Parser<'a, Option<R>> {
    move |input| match parser.parse(input) {
        Ok((value, rest)) => Ok((Some(value), rest)),
        Err(()) => Ok((None, input)),
    }
}

pub(super) fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((next, &input[next.len_utf8()..])),
        _ => Err(()),
    }
}

pub(super) fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<((), &str), ()> {
    move |input| match input.split_once(expected) {
        Some((before, rest)) => {
            if before == "" {
                Ok(((), rest))
            } else {
                Err(())
            }
        }
        None => Err(()),
    }
}

pub(super) fn whitespace(input: &str) -> ParseResult<()> {
    zero_or_more(any_char.pred(|c| c.is_whitespace()))
        .map(|_| ())
        .parse(input)
}

pub(super) fn whitespace_surrounded_sep<'a>(sep: &'static str) -> impl Parser<'a, ()> {
    pair(pair(whitespace, match_literal(sep)), whitespace).map(|_| ())
}

pub(super) fn sep_by<'a, R: Debug>(
    parser: impl Parser<'a, R>,
    sep: &'static str,
) -> impl Parser<'a, Vec<R>> {
    move |input| {
        if let Ok((first, rest)) = parser.parse(input) {
            let mut result = Vec::new();
            let mut tmp_input = rest;
            result.push(first);
            while let Ok((next, rest)) = whitespace_surrounded_sep(sep)
                .parse(tmp_input)
                .and_then(|(_, s)| parser.parse(s))
            {
                tmp_input = rest;
                result.push(next)
            }
            return Ok((result, tmp_input));
        } else {
            return Err(());
        }
    }
}
