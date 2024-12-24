use either::Either;

pub fn take_uint<'a>() -> impl Fn(&'a str) -> Option<(u64, &'a str)> {
    move |input: &str| {
        let mut chars = input.char_indices().peekable();
        let mut res = 0u64;
        let mut first = true;
        while let Some((_, c)) = chars.peek() {
            if first && !c.is_ascii_digit() {
                return None;
            }
            first = false;

            if c.is_ascii_digit() {
                res = res * 10 + (c.to_digit(10).unwrap() as u64);
            } else {
                break;
            }

            chars.next();
        }

        let rest = match chars.peek() {
            Some((idx, _)) => &input[*idx..],
            None => &input[input.len()..],
        };

        Some((res, rest))
    }
}

pub fn take_str<'a, 'e>(expected: &'e str) -> impl Fn(&'a str) -> Option<(&'e str, &'a str)> + 'e {
    move |input: &str| {
        if !input.starts_with(expected) {
            return None;
        }
        let rest = &input[expected.len()..];
        Some((expected, rest))
    }
}

pub fn take_char<'a>(expected: char) -> impl Fn(&'a str) -> Option<(char, &'a str)> {
    move |input: &str| match input.chars().next() {
        Some(c) if c == expected => Some((c, &input[c.len_utf8()..])),
        _ => None,
    }
}

pub fn take_any<'a, 'e>(expected: &'e str) -> impl Fn(&'a str) -> Option<(&'e str, &'a str)> + 'e {
    move |input: &str| match expected.chars().find(|c| input.starts_with(*c)) {
        Some(c) => {
            let rest = &input[c.len_utf8()..];
            Some((expected, rest))
        }
        None => None,
    }
}

pub fn take_any_func<'a, F>(expected: F) -> impl Fn(&'a str) -> Option<(char, &'a str)>
where
    F: Fn(&char) -> bool,
{
    move |input: &str| match input.chars().find(|c| expected(c)) {
        Some(c) => {
            let rest = &input[c.len_utf8()..];
            Some((c, rest))
        }
        None => None,
    }
}

pub fn take_newline<'a>() -> impl Fn(&'a str) -> Option<(&'a str, &'a str)> {
    take_any("\n\r")
}

pub fn take_whitespace<'a>() -> impl Fn(&'a str) -> Option<(&'a str, &'a str)> {
    take_any(" \t\r\n")
}

pub fn take_spacetab<'a>() -> impl Fn(&'a str) -> Option<(&'a str, &'a str)> {
    take_any(" \t")
}

pub fn take_eol<'a>() -> impl Fn(&'a str) -> Option<((), &'a str)> {
    move |input: &str| -> Option<((), &'a str)> {
        if input.is_empty() {
            return Some(((), input));
        }
        None
    }
}

pub fn take_int<'a>() -> impl Fn(&'a str) -> Option<(i64, &'a str)> {
    move |input: &str| {
        let (sign, rest) = if let Some((_, rest)) = take_str("-")(input) {
            (-1, rest)
        } else if let Some((_, rest)) = take_str("+")(input) {
            (1, rest)
        } else {
            (1, input)
        };

        let (res, rest) = take_uint()(rest)?;
        Some(((res as i64) * sign, rest))
    }
}

pub fn take_separator<'a, F, S>(
    item: impl Fn(&'a str) -> Option<(F, &'a str)>,
    separator: impl Fn(&'a str) -> Option<(S, &'a str)>,
) -> impl Fn(&'a str) -> Option<(Vec<F>, &'a str)> {
    move |input: &str| -> Option<(Vec<F>, &str)> {
        let mut res = vec![];
        let mut cur = input;
        while let Some((item, rest)) = item(cur) {
            res.push(item);
            cur = rest;
            if let Some((_, rest)) = separator(cur) {
                cur = rest;
            } else {
                break;
            }
        }

        Some((res, cur))
    }
}

pub fn catch<'a, P>(
    parser: impl Fn(&'a str) -> Option<(P, &'a str)>,
) -> impl Fn(&'a str) -> Option<((), &'a str)> {
    move |input: &str| -> Option<((), &str)> {
        if let Some((_, rest)) = parser(input) {
            return Some(((), rest));
        }
        Some(((), input))
    }
}

pub fn with_space<'a, P>(
    parser: impl Fn(&'a str) -> Option<(P, &'a str)> + 'a,
) -> impl Fn(&'a str) -> Option<(P, &'a str)> {
    move |input: &str| -> Option<(P, &str)> {
        let pos = input.find(|c| !" \t".contains(c))?;
        let rest = &input[pos..];
        let (res, rest) = parser(rest)?;
        match rest.find(|c| !" \t".contains(c)) {
            Some(pos) => Some((res, &rest[pos..])),
            None => Some((res, &rest[rest.len()..])),
        }
    }
}

pub fn take_first<'a, F, S>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(S, &'a str)>,
) -> impl Fn(&'a str) -> Option<(F, &'a str)> {
    move |input: &str| -> Option<(F, &str)> {
        let (l, rest) = first(input)?;
        let (_, rest) = second(rest)?;
        Some((l, rest))
    }
}

pub fn take_second<'a, F, S>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(S, &'a str)>,
) -> impl Fn(&'a str) -> Option<(S, &'a str)> {
    move |input: &str| -> Option<(S, &str)> {
        let (_, rest) = first(input)?;
        let (r, rest) = second(rest)?;
        Some((r, rest))
    }
}

pub fn take_tuple<'a, F, S>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(S, &'a str)>,
) -> impl Fn(&'a str) -> Option<((F, S), &'a str)> {
    move |input: &str| -> Option<((F, S), &str)> {
        let (l, rest) = first(input)?;
        let (r, rest) = second(rest)?;
        Some(((l, r), rest))
    }
}

pub fn take_tuple3<'a, A, B, C>(
    first: impl Fn(&'a str) -> Option<(A, &'a str)>,
    second: impl Fn(&'a str) -> Option<(B, &'a str)>,
    third: impl Fn(&'a str) -> Option<(C, &'a str)>,
) -> impl Fn(&'a str) -> Option<((A, B, C), &'a str)> {
    move |input: &str| -> Option<((A, B, C), &str)> {
        let (a, rest) = first(input)?;
        let (b, rest) = second(rest)?;
        let (c, rest) = third(rest)?;
        Some(((a, b, c), rest))
    }
}

#[allow(clippy::type_complexity)]
pub fn take_tuple4<'a, A, B, C, D>(
    first: impl Fn(&'a str) -> Option<(A, &'a str)>,
    second: impl Fn(&'a str) -> Option<(B, &'a str)>,
    third: impl Fn(&'a str) -> Option<(C, &'a str)>,
    fourth: impl Fn(&'a str) -> Option<(D, &'a str)>,
) -> impl Fn(&'a str) -> Option<((A, B, C, D), &'a str)> {
    move |input: &str| -> Option<((A, B, C, D), &str)> {
        let (a, rest) = first(input)?;
        let (b, rest) = second(rest)?;
        let (c, rest) = third(rest)?;
        let (d, rest) = fourth(rest)?;
        Some(((a, b, c, d), rest))
    }
}

pub fn take_either<'a, A, B>(
    first: impl Fn(&'a str) -> Option<(A, &'a str)>,
    second: impl Fn(&'a str) -> Option<(B, &'a str)>,
) -> impl Fn(&'a str) -> Option<(Either<A, B>, &'a str)> {
    move |input: &str| -> Option<(Either<A, B>, &str)> {
        if let Some((a, rest)) = first(input) {
            return Some((Either::Left(a), rest));
        }
        if let Some((b, rest)) = second(input) {
            return Some((Either::Right(b), rest));
        }
        None
    }
}

pub fn take_many0<'a, F>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
) -> impl Fn(&'a str) -> Option<(Vec<F>, &'a str)> {
    move |input: &str| -> Option<(Vec<F>, &str)> {
        let mut res = vec![];
        let mut cur = input;
        while let Some((item, rest)) = first(cur) {
            res.push(item);
            cur = rest;
        }

        Some((res, cur))
    }
}

pub fn take_many1<'a, F>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
) -> impl Fn(&'a str) -> Option<(Vec<F>, &'a str)> {
    move |input: &'a str| -> Option<(Vec<F>, &'a str)> {
        let mut res = vec![];
        let mut cur = input;
        while let Some((item, rest)) = first(cur) {
            res.push(item);
            cur = rest;
        }

        if res.is_empty() {
            return None;
        }

        Some((res, cur))
    }
}

pub fn take_or<'a, F>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(F, &'a str)>,
) -> impl Fn(&'a str) -> Option<(F, &'a str)> {
    move |input: &str| -> Option<(F, &str)> {
        if let Some((f, rest)) = first(input) {
            return Some((f, rest));
        }
        second(input)
    }
}

pub fn take_or3<'a, F>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(F, &'a str)>,
    third: impl Fn(&'a str) -> Option<(F, &'a str)>,
) -> impl Fn(&'a str) -> Option<(F, &'a str)> {
    move |input: &str| -> Option<(F, &str)> {
        if let Some((f, rest)) = first(input) {
            return Some((f, rest));
        }
        if let Some((f, rest)) = second(input) {
            return Some((f, rest));
        }
        third(input)
    }
}

pub fn take_or4<'a, F>(
    first: impl Fn(&'a str) -> Option<(F, &'a str)>,
    second: impl Fn(&'a str) -> Option<(F, &'a str)>,
    third: impl Fn(&'a str) -> Option<(F, &'a str)>,
    fourth: impl Fn(&'a str) -> Option<(F, &'a str)>,
) -> impl Fn(&'a str) -> Option<(F, &'a str)> {
    move |input: &str| -> Option<(F, &str)> {
        if let Some((f, rest)) = first(input) {
            return Some((f, rest));
        }
        if let Some((f, rest)) = second(input) {
            return Some((f, rest));
        }
        if let Some((f, rest)) = third(input) {
            return Some((f, rest));
        }
        fourth(input)
    }
}

pub fn map<'a, A, B>(
    first: impl Fn(&'a str) -> Option<(A, &'a str)>,
    f: impl Fn(A) -> B,
) -> impl Fn(&'a str) -> Option<(B, &'a str)> {
    move |input: &str| -> Option<(B, &str)> {
        if let Some((a, rest)) = first(input) {
            return Some((f(a), rest));
        }
        None
    }
}
