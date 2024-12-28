use std::io::Read;

use aoc24::parser;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

type Pattern = Vec<u8>;
fn take_pattern<'a>() -> impl Fn(&'a str) -> Option<(Pattern, &'a str)> {
    move |input: &str| {
        let (indices, rest_idx) = input
            .char_indices()
            .take_while(|c| matches!(c.1, 'r' | 'w' | 'b' | 'g' | 'u'))
            .fold((Vec::new(), (0)), |mut acc, c| {
                acc.0.push(c.1 as u8);
                acc.1 = c.0 + c.1.len_utf8();
                acc
            });

        if rest_idx == 0 {
            return None;
        }

        Some((indices, &input[rest_idx..]))
    }
}

fn take_options<'a>() -> impl Fn(&'a str) -> Option<(Vec<Pattern>, &'a str)> {
    parser::take_separator(take_pattern(), parser::take_str(", "))
}

#[derive(Debug)]
struct ParsedResult {
    options: Vec<Pattern>,
    checks: Vec<Pattern>,
}

fn take_file<'a>() -> impl Fn(&'a str) -> Option<(ParsedResult, &'a str)> {
    move |input: &str| {
        let (mut options, rest) =
            parser::take_first(take_options(), parser::take_newline())(input)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (checks, rest) = parser::take_separator(take_pattern(), parser::take_newline())(rest)?;
        let (_, rest) = parser::take_eol()(rest)?;
        options.sort();
        options.reverse();
        Some((ParsedResult { options, checks }, rest))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<ParsedResult> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let (result, rest) = take_file()(&string).ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    if !rest.is_empty() {
        Err(anyhow::anyhow!("could not parse: {rest}"))?;
    }
    Ok(result)
}

fn starts_with<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    needle.iter().zip(haystack.iter()).all(|(a, b)| a == b)
}

fn check(patterns: &[Pattern], haystack: &[u8]) -> bool {
    let mut dp = vec![None; haystack.len() + 1];
    dp_check(patterns, haystack, &mut dp)
}

fn dp_check(patterns: &[Pattern], haystack: &[u8], dp: &mut Vec<Option<bool>>) -> bool {
    if let Some(v) = dp[haystack.len()] {
        return v;
    }

    if haystack.is_empty() {
        dp[haystack.len()] = Some(true);
        return true;
    }

    for pattern in patterns {
        if !starts_with(haystack, pattern) {
            continue;
        }

        if dp_check(patterns, &haystack[pattern.len()..], dp) {
            dp[haystack.len()] = Some(true);
            return true;
        }
    }

    dp[haystack.len()] = Some(false);
    false
}

fn check2(patterns: &[Pattern], haystack: &[u8]) -> usize {
    let mut dp = vec![None; haystack.len() + 1];
    dp_check2(patterns, haystack, &mut dp)
}

fn dp_check2(patterns: &[Pattern], haystack: &[u8], dp: &mut Vec<Option<usize>>) -> usize {
    if let Some(v) = dp[haystack.len()] {
        return v;
    }

    if haystack.is_empty() {
        dp[haystack.len()] = Some(1);
        return 1;
    }

    let mut sum = 0;
    for pattern in patterns {
        if !starts_with(haystack, pattern) {
            continue;
        }

        sum += dp_check2(patterns, &haystack[pattern.len()..], dp);
    }

    dp[haystack.len()] = Some(sum);
    sum
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let count = output
                .checks
                .iter()
                .filter(|p| check(&output.options, p))
                .count();

            println!("{}", count);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let sum = output
                .checks
                .iter()
                .map(|p| check2(&output.options, p))
                .sum::<usize>();

            println!("{}", sum);
            Ok(())
        }
    }
}
