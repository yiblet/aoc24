use std::io::Read;

use aoc24::parser::take_uint;
use clap::Parser;

fn take_mul() -> impl Fn(&str) -> Option<(u64, &str)> {
    move |input: &str| {
        if !input.starts_with("mul(") {
            return None;
        }
        let (mul1, rest) = take_uint()(&input[4..])?;
        if !rest.starts_with(',') {
            return None;
        }
        let (mul2, rest) = take_uint()(&rest[1..])?;
        if !rest.starts_with(')') {
            return None;
        }
        Some((mul1 * mul2, &rest[1..]))
    }
}

fn take_do() -> impl Fn(&str) -> Option<(bool, &str)> {
    move |input: &str| {
        if !input.starts_with("do()") {
            return None;
        }
        Some((true, &input[4..]))
    }
}

fn take_dont() -> impl Fn(&str) -> Option<(bool, &str)> {
    move |input: &str| {
        if !input.starts_with("don't()") {
            return None;
        }
        Some((false, &input["don't()".len()..]))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<usize> {
    let mut file = std::fs::File::open(filename)?;

    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let mut res = 0usize;
    let mut cur = &input[..];
    while let Some(pos) = cur.find("mul(") {
        match take_mul()(&cur[pos..]) {
            Some((mul, rest)) => {
                res += mul as usize;
                cur = rest;
            }
            None => {
                // increment by one character to begin with the next potential mul
                cur = &cur[1..];
            }
        }
    }

    Ok(res)
}

fn parse_file2(filename: &str) -> anyhow::Result<usize> {
    let mut file = std::fs::File::open(filename)?;

    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let mut cur = &input[..];
    let mut do_status = true;
    let mut res = 0usize;
    while !cur.is_empty() {
        if let Some((_, rest)) = take_do()(cur) {
            do_status = true;
            cur = rest;
        } else if let Some((_, rest)) = take_dont()(cur) {
            do_status = false;
            cur = rest;
        } else if let Some((mul, rest)) = take_mul()(cur) {
            if do_status {
                res += mul as usize;
            }
            cur = rest;
        } else {
            cur = &cur[1..];
        }
    }

    Ok(res)
}

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let res = parse_file(&file)?;
            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file } => {
            let res = parse_file2(&file)?;
            println!("{}", res);
            Ok(())
        }
    }
}
