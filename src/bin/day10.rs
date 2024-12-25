use std::{collections::HashSet, rc::Rc};

use aoc24::grid;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn walk2(
    grid: &grid::Grid<u32>,
    visited: &mut grid::Grid<Option<usize>>,
    pos: (isize, isize),
    value: u32,
) -> usize {
    match grid::get_at(visited, pos) {
        Some(Some(v)) => {
            return *v;
        }
        None => return 0,
        _ => {}
    };

    if value == 9 {
        return 1;
    }

    let mut res = 0;
    if value == 9 {
        res = 1;
    } else {
        for dir in grid::Direction::all_directions() {
            let pos2 = dir.apply(pos);

            let Some(v) = grid::get_at(grid, pos2) else {
                continue;
            };
            if *v != value + 1 {
                continue;
            }

            res += walk2(grid, visited, pos2, value + 1);
        }
    }

    if let Some(v) = grid::get_at_mut(visited, pos) {
        *v = Some(res);
    };

    res
}

type TrailEnds = Rc<HashSet<(isize, isize)>>;


fn walk(
    grid: &grid::Grid<u32>,
    visited: &mut grid::Grid<Option<TrailEnds>>,
    pos: (isize, isize),
    value: u32,
) -> TrailEnds {
    match grid::get_at(visited, pos) {
        Some(Some(v)) => {
            return v.clone();
        }
        None => return Rc::new(HashSet::new()),
        _ => {}
    };

    let mut res;
    if value == 9 {
        res = HashSet::from([(pos)]);
    } else {
        res = HashSet::new();
        for dir in grid::Direction::all_directions() {
            let pos2 = dir.apply(pos);

            let Some(v) = grid::get_at(grid, pos2) else {
                continue;
            };
            if *v != value + 1 {
                continue;
            }

            res.extend(walk(grid, visited, pos2, value + 1).iter().cloned());
        }
    }

    let res = Rc::new(res);
    if let Some(v) = grid::get_at_mut(visited, pos) {
        *v = Some(res.clone());
    };

    res
}


fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = grid::parse_grid(&file)?;
            let grid = grid::map_result(&output, |_, _, chr| {
                chr.to_digit(10)
                    .ok_or_else(|| anyhow::anyhow!("not a digit"))
            })?;

            let mut visited = grid::copy_with(&grid);
            let zeros = grid::iter_pos(&grid).filter(|(_, _, c)| **c == 0);

            let mut res = 0;
            for (row, col, c) in zeros {
                // print the row 
                let score = walk(&grid, &mut visited, (row, col), *c).len();
                println!("row {}, col {}, score {}", row, col, score);
                res += score;
            }

            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = grid::parse_grid(&file)?;
            let grid = grid::map_result(&output, |_, _, chr| {
                chr.to_digit(10)
                    .ok_or_else(|| anyhow::anyhow!("not a digit"))
            })?;

            let mut visited = grid::copy_with(&grid);
            let zeros = grid::iter_pos(&grid).filter(|(_, _, c)| **c == 0);

            let mut res = 0;
            for (row, col, c) in zeros {
                // print the row 
                let score = walk2(&grid, &mut visited, (row, col), *c);
                println!("row {}, col {}, score {}", row, col, score);
                res += score;
            }

            println!("{}", res);
            Ok(())
        }
    }
}
