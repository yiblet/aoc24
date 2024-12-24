use std::io::BufRead;

use aoc24::parser;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn concat(lhs: u64, rhs: u64) -> u64 {
    // shift lhs to the right by the number of digits in rhs
    let mut shifted_lhs = lhs;
    let mut cur = rhs;
    while cur > 0 {
        cur /= 10;
        shifted_lhs *= 10;
    }

    shifted_lhs + rhs
}

fn solve(lhs: u64, cur: u64, rhs: &[u64]) -> bool {
    match rhs {
        [] => lhs == cur,
        [r, rest @ ..] => solve(lhs, cur + *r, rest) || solve(lhs, cur * r, rest),
    }
}

fn solve2(lhs: u64, cur: u64, rhs: &[u64]) -> bool {
    match rhs {
        [] => lhs == cur,
        [r, rest @ ..] => {
            solve2(lhs, cur + *r, rest)
                || solve2(lhs, cur * r, rest)
                || solve2(lhs, concat(cur, *r), rest)
        }
    }
}

struct Line {
    lhs: u64,
    rhs: Vec<u64>,
}

impl Line {
    fn solve(&self) -> bool {
        match self.rhs.as_slice() {
            [] => self.lhs == 0,
            [r, rest @ ..] => solve(self.lhs, *r, rest),
        }
    }

    fn solve2(&self) -> bool {
        match self.rhs.as_slice() {
            [] => self.lhs == 0,
            [r, rest @ ..] => solve2(self.lhs, *r, rest),
        }
    }
}

fn take_line<'a>() -> impl Fn(&'a str) -> Option<(Line, &'a str)> {
    move |input: &str| -> Option<(Line, &str)> {
        let (lhs, rest) = parser::take_uint()(input)?;
        let (_, rest) = parser::take_str(": ")(rest)?;
        let (rhs, rest) =
            parser::take_separator(parser::take_uint(), parser::take_spacetab())(rest)?;
        let (_, rest) = parser::take_eol()(rest)?;

        Some((Line { lhs, rhs }, rest))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Line>> {
    let file = std::fs::File::open(filename)?;
    let bufreader = std::io::BufReader::new(file);
    let lines = bufreader
        .lines()
        .map(|line| {
            let line = line.ok()?;
            let take_line = take_line();
            Some(take_line(&line)?.0)
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(anyhow::anyhow!("could not parse file"))?;

    Ok(lines)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let valids = output
                .iter()
                .filter(|l| l.solve())
                .map(|l| l.lhs)
                .sum::<u64>();
            println!("{}", valids);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;

            let valids = output
                .iter()
                .filter(|l| l.solve2())
                .map(|l| l.lhs)
                .sum::<u64>();
            println!("{}", valids);
            Ok(())
        }
    }
}
