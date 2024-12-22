use clap::Parser;
use std::io::BufRead;

fn parse_file(filename: &str) -> anyhow::Result<Vec<Vec<u32>>> {
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);

    let res = reader
        .lines()
        .map(|line| -> anyhow::Result<Vec<u32>> {
            let line = line?;
            let parts = line
                .split_whitespace()
                .map(|s| s.parse::<u32>())
                .collect::<Result<Vec<_>, _>>()?;

            Ok(parts)
        })
        .collect::<Result<Vec<_>, _>>()?;

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

            let ok_rows = res
                .iter()
                .filter(|row| {
                    let mut prev = 0;
                    let mut increasing = false;
                    let mut row_ok = true;
                    for (i, val) in row.iter().copied().enumerate() {
                        match (i, increasing) {
                            (0, _) => {}
                            (1, _) => {
                                if val > prev {
                                    increasing = true;
                                    if !(prev < val && val <= prev + 3) {
                                        row_ok = false;
                                        break;
                                    }
                                } else {
                                    increasing = false;
                                    if !(val < prev && prev <= val + 3) {
                                        row_ok = false;
                                        break;
                                    }
                                }
                            }
                            (_, true) => {
                                if !(prev < val && val <= prev + 3) {
                                    row_ok = false;
                                    break;
                                }
                            }
                            (_, false) => {
                                if !(val < prev && prev <= val + 3) {
                                    row_ok = false;
                                    break;
                                }
                            }
                        };

                        prev = val;
                    }
                    row_ok
                })
                .count();

            println!("{}", ok_rows);
            Ok(())
        }

        Args::Part2 { file } => {
            let res = parse_file(&file)?;

            let ok_rows = res
                .iter()
                .filter(|row| {
                    iter_row(row).any(|row_option| {
                        let mut prev = 0;
                        let mut increasing = false;
                        let mut row_ok = true;
                        for (i, val) in row_option.enumerate() {
                            match (i, increasing) {
                                (0, _) => {}
                                (1, _) => {
                                    if val > prev {
                                        increasing = true;
                                        if !(prev < val && val <= prev + 3) {
                                            row_ok = false;
                                            break;
                                        }
                                    } else {
                                        increasing = false;
                                        if !(val < prev && prev <= val + 3) {
                                            row_ok = false;
                                            break;
                                        }
                                    }
                                }
                                (_, true) => {
                                    if !(prev < val && val <= prev + 3) {
                                        row_ok = false;
                                        break;
                                    }
                                }
                                (_, false) => {
                                    if !(val < prev && prev <= val + 3) {
                                        row_ok = false;
                                        break;
                                    }
                                }
                            };

                            prev = val;
                        }
                        row_ok
                    })
                })
                .count();

            println!("{}", ok_rows);
            Ok(())
        }
    }
}

fn iter_row(row: &[u32]) -> impl Iterator<Item = impl Iterator<Item = u32> + use<'_>> + '_ {
    let mut skip_idx = None;

    std::iter::from_fn(move || {
        let cur_idx = skip_idx;
        if let Some(idx) = cur_idx {
            if idx >= row.len() {
                return None;
            }
        }

        skip_idx = match skip_idx {
            Some(idx) => Some(idx + 1),
            None => Some(0),
        };

        Some(
            row.iter()
                .copied()
                .enumerate()
                .filter(move |(x, _)| Some(*x) != cur_idx)
                .map(|(_, x)| x),
        )
    })
}
