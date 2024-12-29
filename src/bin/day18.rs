use std::{cmp, collections::BTreeSet, io::Read};

use aoc24::{graph, grid, parser};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 {
        file: String,
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
    },
}

fn take_line<'a>() -> impl Fn(&'a str) -> Option<(grid::Index, &'a str)> {
    move |input: &str| {
        let (i, rest) = parser::take_int()(input)?;
        let (_, rest) = parser::take_str(",")(rest)?;
        let (j, rest) = parser::take_int()(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        Some(((i as isize, j as isize), rest))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<grid::Index>> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let (grid, rest) = parser::take_many1(take_line())(string.as_str())
        .ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    if !rest.is_empty() {
        Err(anyhow::anyhow!("could not parse: {rest}"))?;
    }

    Ok(grid)
}

fn run_with_drops(output: &[grid::Index], value: usize) -> anyhow::Result<Option<usize>> {
    let drops = &output[0..=cmp::min(value, output.len())]
        .iter()
        .collect::<BTreeSet<_>>();
    let mut graph = graph::Graph::<grid::Index>::new();
    let shape = 70;

    for i in 0..=shape {
        for j in 0..=shape {
            let pos = (i, j);
            if drops.contains(&pos) {
                continue;
            }
            for dir in grid::Direction::all_directions() {
                let pos2 = dir.apply(pos);
                if pos2.0 < 0 || pos2.0 > shape || pos2.1 < 0 || pos2.1 > shape {
                    continue;
                }
                if drops.contains(&pos2) {
                    continue;
                }
                graph.entry(pos).or_default().insert((pos2, 1));
            }
        }
    }

    let distance = graph::dijkstras(&graph, &(0, 0))
        .get(&(shape, shape))
        .cloned();

    Ok(distance)
}

// binsearch to the first value that is false.
// low may or may not be the first value that is false.
// high must be false.
fn binsearch(drops: &[grid::Index], low: usize, high: usize) -> anyhow::Result<Option<usize>> {
    if low >= high {
        if run_with_drops(drops, low)?.is_none() {
            return Ok(Some(low));
        };
        return Ok(None);
    }

    let mid = low + (high - low) / 2;
    let res = run_with_drops(drops, mid)?.is_some();

    if res {
        binsearch(drops, mid + 1, high)
    } else {
        binsearch(drops, low, mid)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let distance =
                run_with_drops(&output, 1024)?.ok_or_else(|| anyhow::anyhow!("could not find"))?;
            println!("{:?}", distance);
            Ok(())
        }

        Args::Part2 { file, debug } => {
            let drops = parse_file(&file)?;
            let search = binsearch(&drops, 0, drops.len() - 1)?
                .ok_or_else(|| anyhow::anyhow!("could not find"))?;

            if debug {
                for i in 0..drops.len() {
                    let res = run_with_drops(&drops, i)?;
                    println! {"i = {}, {:?}: {:?}", i, drops[i], res};
                    if search == i {
                        println!("found at i = {}", i);
                    }
                }
            }

            println!("{},{}", drops[search].0, drops[search].1);
            Ok(())
        }
    }
}
