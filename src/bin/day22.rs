use itertools::Itertools;
use std::{collections::BTreeMap, io::BufRead, iter};

use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<u64>> {
    let file = std::fs::File::open(filename)?;
    let bufreader = std::io::BufReader::new(file);
    bufreader
        .lines()
        .map(|line| Ok(line?.parse::<u64>()?))
        .collect()
}

fn mix(secret: u64, input: u64) -> u64 {
    secret ^ input
}

fn prune(input: u64) -> u64 {
    input % 16777216
}

// Calculate the result of multiplying the secret number by 64. Then, mix this result into the secret number. Finally, prune the secret number.
// Calculate the result of dividing the secret number by 32. Round the result down to the nearest integer. Then, mix this result into the secret number. Finally, prune the secret number.
// Calculate the result of multiplying the secret number by 2048. Then, mix this result into the secret number. Finally, prune the secret number.
fn next(secret: u64) -> u64 {
    let mut res = secret;
    res = prune(mix(res, res * 64));
    res = prune(mix(res, res / 32));
    res = prune(mix(res, res * 2048));
    res
}

fn next_iter(secret: u64) -> impl Iterator<Item = u64> {
    iter::successors(Some(secret), |secret| Some(next(*secret)))
}

fn price(secret: u64) -> i8 {
    (secret % 10) as i8
}

fn push_shift<const N: usize, T: Copy>(seq: &mut [T; N], end: T) {
    seq.copy_within(1.., 0);
    seq[N - 1] = end;
}

fn changes(iter: impl Iterator<Item = i8>) -> impl Iterator<Item = (i8, [i8; 4])> {
    let mut iter = iter
        .tuple_windows()
        .map(move |(prev, next)| (next, next - prev));

    let mut seq = [0i8; 4];
    for _ in 0..3 {
        let Some((_, diff)) = iter.next() else {
            return either::Either::Left(iter::empty());
        };
        push_shift(&mut seq, diff);
    }

    let res = iter.map(move |(cur, diff)| {
        push_shift(&mut seq, diff);
        (cur, seq)
    });

    either::Either::Right(res)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let input = parse_file(&file)?;

            let mut sum = 0u64;
            for start in input {
                let mut secret = start;
                secret = next_iter(secret).take(2000).last().unwrap();
                println!("{}: {}", start, secret);
                sum += secret;
            }

            println!("{}", sum);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;

            let mut res = BTreeMap::<[i8; 4], Vec<i8>>::new();
            for secret in output {
                let mut sequences = BTreeMap::new();
                let iter = changes(next_iter(secret).map(price).take(2000));
                for (price, seq) in iter {
                    sequences.entry(seq).or_insert(price);
                }

                for (seq, price) in sequences.into_iter() {
                    res.entry(seq).or_default().push(price);
                }
            }

            let (max_seq, soln) = res
                .into_iter()
                .max_by_key(|(_, prices)| prices.iter().map(|x| *x as i32).sum::<i32>())
                .ok_or_else(|| anyhow::anyhow!("could not find max"))?;

            println!("{:?}", max_seq);
            println!("{:?}", soln);
            println!("{}", soln.iter().cloned().map(|x| x as i32).sum::<i32>());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_test() {
        assert_eq!(mix(42, 15), 37);
    }

    #[test]
    fn next_test() {
        let test = [
            123, 15887950, 16495136, 527345, 704524, 1553684, 12683156, 11100544, 12249484,
            7753432, 5908254,
        ];

        for (first, second) in test.iter().cloned().tuple_windows() {
            assert_eq!(second, next(first));
        }
    }

    #[test]
    fn changes_test() {
        let tests = next_iter(123).take(10).map(price).collect::<Vec<_>>();
        assert_eq!(tests, vec![3, 0, 6, 5, 4, 4, 6, 4, 4, 2]);

        let changes = changes(tests.iter().copied()).collect::<Vec<_>>();
        assert_eq!(
            changes,
            vec![
                (4, [-3, 6, -1, -1]),
                (4, [6, -1, -1, 0]),
                (6, [-1, -1, 0, 2]),
                (4, [-1, 0, 2, -2]),
                (4, [0, 2, -2, 0]),
                (2, [2, -2, 0, -2])
            ]
        );
    }
}
