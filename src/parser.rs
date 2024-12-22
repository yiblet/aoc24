pub fn take_uint() -> impl Fn(&str) -> Option<(u64, &str)> {
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

pub fn take_str<'a, 'b>(expected: &'a str) -> impl Fn(&'b str) -> Option<(&'a str, &'b str)> + 'a {
    move |input: &str| {
        if !input.starts_with(expected) {
            return None;
        }
        let rest = &input[expected.len()..];
        Some((expected, rest))
    }
}

pub fn take_int() -> impl Fn(&str) -> Option<(i32, &str)> {
    move |input: &str| {
        let (sign, rest) = if let Some((_, rest)) = take_str("-")(input) {
            (-1, rest)
        } else if let Some((_, rest)) = take_str("+")(input) {
            (1, rest)
        } else {
            (1, input)
        };

        let (res, rest) = take_uint()(rest)?;
        Some(((res as i32) * sign, rest))
    }
}
