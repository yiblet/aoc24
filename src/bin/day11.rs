use clap::Parser;
use either::Either;
use std::{collections::HashMap, io::Read};

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<u64>> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string
        .split_whitespace()
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?)
}

fn num_digits(input: u64) -> u32 {
    let mut res = 0;
    let mut cur = input;
    while cur > 0 {
        cur /= 10;
        res += 1;
    }
    res
}

fn process(input: u64) -> Either<u64, [u64; 2]> {
    if input == 0 {
        return Either::Left(1);
    }
    let digits = num_digits(input);

    if digits % 2 == 0 {
        let splitter = 10u64.pow(digits / 2);
        return Either::Right([input / splitter, input % splitter]);
    }

    Either::Left(input * 2024)
}

fn process_hashed(input: u64, iter: u32, cache: &mut HashMap<(u64, u32), u64>) -> u64 {
    if let Some(res) = cache.get(&(input, iter)) {
        return *res;
    }
    let res = {
        if iter == 0 {
            1
        } else {
            match process(input) {
                Either::Left(res) => process_hashed(res, iter - 1, cache),
                Either::Right([a, b]) => {
                    process_hashed(a, iter - 1, cache) + process_hashed(b, iter - 1, cache)
                }
            }
        }
    };

    cache.insert((input, iter), res);
    res
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let mut cache = HashMap::new();
            let mut res = 0u64;
            for i in output.iter().cloned() {
                res += process_hashed(i, 25, &mut cache);
            }
            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let mut cache = HashMap::new();
            let mut res = 0u64;
            for i in output.iter().cloned() {
                res += process_hashed(i, 75, &mut cache);
            }
            println!("{}", res);
            Ok(())
        }
    }
}
