use clap::Parser;
use std::{collections::HashMap, io::BufRead};

fn parse_file(filename: &str) -> anyhow::Result<(Vec<i32>, Vec<i32>)> {
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);

    let (mut firsts, mut seconds) = (vec![], vec![]);
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        if let Some(first) = parts.next() {
            if let Some(second) = parts.next() {
                let first = first.parse::<i32>()?;
                let second = second.parse::<i32>()?;

                firsts.push(first);
                seconds.push(second);
            }
        }
    }

    Ok((firsts, seconds))
}

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn main() {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let (mut firsts, mut seconds) = parse_file(&file).unwrap();
            firsts.sort();
            seconds.sort();

            let sum: u32 = firsts
                .iter()
                .zip(seconds.iter())
                .map(|(f, s)| f.abs_diff(*s))
                .sum();
            println!("{}", sum);
        }
        Args::Part2 { file } => {
            let (firsts, seconds) = parse_file(&file).unwrap();
            let map = seconds.iter().cloned().fold(HashMap::new(), |mut acc, s| {
                *acc.entry(s).or_insert(0) += 1;
                acc
            });

            let sum: i32 = firsts
                .iter()
                .cloned()
                .map(|f| f * map.get(&f).cloned().unwrap_or(0))
                .sum();

            println!("{}", sum);
        }
    }
}
