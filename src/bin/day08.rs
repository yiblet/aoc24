use std::collections::{HashMap, HashSet};

use aoc24::{
    grid::{self, Grid},
    parser, util,
};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Loc {
    Space,
    Antenna(char),
}

fn take_line<'a>() -> impl Fn(&'a str) -> Option<(Vec<Loc>, &'a str)> {
    let take_space = parser::map(parser::take_any("."), |_| Loc::Space);
    let take_hash = parser::map(
        parser::take_any_func(char::is_ascii_alphanumeric),
        Loc::Antenna,
    );

    parser::take_first(
        parser::take_many1(parser::take_or(take_space, take_hash)),
        parser::take_eol(),
    )
}

fn parse_input(iter: impl Iterator<Item = String>) -> anyhow::Result<Grid<Loc>> {
    let mut grid = vec![];

    for line in iter {
        let (res, _) = take_line()(&line).ok_or(anyhow::anyhow!("could not parse line"))?;
        grid.push(res);
    }

    Ok(grid)
}

fn parse_file(filename: &str) -> anyhow::Result<Grid<Loc>> {
    let mut lines = util::read_file_lines(filename)?;

    let input = parse_input(&mut lines);

    lines.error()?;

    input
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let mut antinodes = grid::copy_default(&output);
            let positions = grid::iter_pos(&output)
                .map(|((row, col), loc)| (loc, (row, col)))
                .filter(|(loc, _)| matches!(loc, Loc::Antenna(_)))
                .fold(
                    HashMap::new(),
                    |mut acc: HashMap<Loc, HashSet<(isize, isize)>>, (loc, pos)| {
                        acc.entry(*loc).or_default().insert(pos);
                        acc
                    },
                );

            for (_, points) in positions.iter() {
                for p1 in points.iter() {
                    for p2 in points.iter() {
                        if p1 == p2 {
                            continue;
                        }

                        let delta = grid::vec_sub(*p2, *p1);
                        let p3 = grid::vec_add(*p1, grid::scale(delta, 2));
                        if let Some(loc) = grid::get_at_mut(&mut antinodes, p3) {
                            *loc = true;
                        }
                    }
                }
            }

            let count = antinodes
                .iter()
                .flat_map(|v| v.iter())
                .filter(|v| **v)
                .count();
            println!("{}", count);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let mut antinodes = grid::copy_default(&output);
            let positions = grid::iter_pos(&output)
                .map(|((row, col), loc)| (loc, (row, col)))
                .filter(|(loc, _)| matches!(loc, Loc::Antenna(_)))
                .fold(
                    HashMap::new(),
                    |mut acc: HashMap<Loc, HashSet<(isize, isize)>>, (loc, pos)| {
                        acc.entry(*loc).or_default().insert(pos);
                        acc
                    },
                );

            for (_, points) in positions.iter() {
                for p1 in points.iter() {
                    for p2 in points.iter() {
                        if p1 == p2 {
                            continue;
                        }

                        let delta = grid::vec_sub(*p2, *p1);
                        let delta = grid::reduce_vec(delta);

                        let mut p3 = *p1;
                        while let Some(loc) = grid::get_at_mut(&mut antinodes, p3) {
                            *loc = true;
                            p3 = grid::vec_add(p3, delta);
                        }
                    }
                }
            }

            let count = antinodes
                .iter()
                .flat_map(|v| v.iter())
                .filter(|v| **v)
                .count();
            println!("{}", count);
            Ok(())
        }
    }
}
