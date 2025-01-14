use std::io::Read;

use aoc24::parser;
use clap::Parser;
use either::Either;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Puzzle {
    is_key: bool,
    pins: [u8; 5],
}

impl Puzzle {
    fn matches(&self, other: &Self) -> bool {
        if self.is_key == other.is_key {
            return false;
        }

        let (key, lock) = if self.is_key {
            (self, other)
        } else {
            (other, self)
        };

        key.pins
            .iter()
            .cloned()
            .zip(lock.pins.iter().cloned())
            .all(|(k, l)| pin_match(k, l))
    }
}

type Grid = [[bool; 5]; 7];

fn pin_match(key: u8, lock: u8) -> bool {
    let lock_space = 6u8.wrapping_sub(lock);
    key < lock_space
}

fn take_grid<'a>() -> impl Fn(&'a str) -> Option<([[bool; 5]; 7], &'a str)> {
    move |input| {
        let mut rest = input;
        let mut res = [[false; 5]; 7];
        for row in res.iter_mut() {
            let (prefix, remainder) = rest.split_once('\n')?;
            rest = remainder;
            if prefix.len() != row.len() {
                return None;
            }
            for (c, byte) in row.iter_mut().zip(prefix.bytes()) {
                *c = byte == b'#'
            }
        }

        Some((res, rest))
    }
}

fn take_puzzle<'a>() -> impl Fn(&'a str) -> Option<(Puzzle, &'a str)> {
    move |input| {
        let (grid, rest) = take_grid()(input)?;
        let puzzle = parse_puzzle(&grid, true).or_else(|| parse_puzzle(&grid, false))?;
        Some((puzzle, rest))
    }
}

fn parse_puzzle(grid: &Grid, is_key: bool) -> Option<Puzzle> {
    let mut res = [0u8; 5];
    let mut done = [false; 5];

    let iter_rows = if is_key {
        Either::Left(grid.iter().rev())
    } else {
        Either::Right(grid.iter())
    };

    for row in iter_rows {
        for (c, (res, done)) in row.iter().zip(res.iter_mut().zip(done.iter_mut())) {
            if *done {
                if *c {
                    return None;
                }
                continue;
            }
            if *c {
                *res += 1
            } else {
                *done = true;
            }
        }
    }

    for c in res.iter_mut() {
        if *c == 0 {
            return None;
        }
        *c -= 1;
    }

    Some(Puzzle { pins: res, is_key })
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Puzzle>> {
    let mut file = std::fs::File::open(filename)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    let (res, rest) = parser::take_separator(take_puzzle(), parser::take_newline())(&input)
        .ok_or_else(|| anyhow::anyhow!("could not read file"))?;

    if !rest.is_empty() {
        anyhow::bail!("not empty: {:?}", rest)
    }

    Ok(res)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let keys = output.iter().filter(|p| p.is_key).collect::<Vec<_>>();
            let locks = output.iter().filter(|p| !p.is_key).collect::<Vec<_>>();
            let matches = itertools::iproduct!(keys, locks)
                .filter(|(k, l)| k.matches(l))
                .count();

            println!("{:?}", matches);
            Ok(())
        }

        Args::Part2 { file: _ } => {
            // There's no part 2 for this one
            println!("42");
            Ok(())
        }
    }
}
