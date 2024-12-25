use aoc24::{grid::{self, Direction}, parser, util};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
    Space,
    Hash,
}


#[derive(Debug)]
struct Grid {
    grid: Vec<Vec<Loc>>,
    start: (usize, usize),
}

fn take_line<'a>() -> impl Fn(&'a str) -> Option<(Vec<Loc>, &'a str)> {
    let take_space = parser::map(parser::take_any(".^<>v"), |_| Loc::Space);
    let take_hash = parser::map(parser::take_any("#"), |_| Loc::Hash);

    parser::take_first(
        parser::take_many1(parser::take_or(take_space, take_hash)),
        parser::take_eol(),
    )
}

fn parse_input(iter: impl Iterator<Item = String>) -> anyhow::Result<Grid> {
    let mut grid = vec![];
    let mut pos = None;

    for (row, line) in iter.enumerate() {
        let (res, _) = take_line()(&line).ok_or(anyhow::anyhow!("could not parse line"))?;
        if let Some(col) = line.find('^') {
            pos = Some((row, col));
        }
        grid.push(res);
    }

    let start = pos.ok_or(anyhow::anyhow!("could not find start"))?;

    Ok(Grid { grid, start })
}

fn parse_file(filename: &str) -> anyhow::Result<Grid> {
    let mut lines = util::read_file_lines(filename)?;

    let input = parse_input(&mut lines);

    lines.error()?;
    input
}

fn fill_visited(output: &Grid) -> (Vec<Vec<usize>>, bool) {
    let mut visited: Vec<Vec<usize>> = output
        .grid
        .iter()
        .map(|v| v.iter().map(|_| 0).collect())
        .collect();

    let mut pos = (output.start.0 as isize, output.start.1 as isize);
    let mut state = Direction::Up;
    let mut cycle = false;

    while let Some((loc, visited)) =
        grid::get_at(&output.grid, pos).zip(grid::get_at_mut(&mut visited, pos))
    {
        // cycle detection
        if (*visited & state as usize) != 0 {
            cycle = true;
            break;
        }
        *visited |= state as usize;
        match loc {
            Loc::Space => pos = state.apply(pos),
            Loc::Hash => {
                pos = state.apply_inverse(pos); // back out
                state = state.rotate_90_right()
            }
        };
    }

    (visited, cycle)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let (visited, _) = fill_visited(&output);

            let positions = visited
                .iter()
                .zip(output.grid.iter())
                .flat_map(|v| v.0.iter().zip(v.1.iter()))
                .filter(|(v, x)| *x == &Loc::Space && **v != 0)
                .count();

            println!("{}", positions);
            Ok(())
        }

        Args::Part2 { file } => {
            let mut output = parse_file(&file)?;
            let (visited, _) = fill_visited(&output);

            let positions = visited
                .iter()
                .zip(output.grid.clone())
                .enumerate()
                .flat_map(|(row, v)| {
                    v.0.iter()
                        .zip(v.1)
                        .enumerate()
                        .map(move |(col, v)| (row, col, v))
                })
                .filter(|(_, _, (bits, x))| *x == Loc::Space && **bits != 0)
                .map(|(row, col, _)| (row as isize, col as isize));

            let mut cycles = 0;
            for (row, col) in positions {
                let Some(loc) = grid::get_at_mut(&mut output.grid, (row, col)) else {
                    Err(anyhow::anyhow!("could not get location"))?
                };
                let prev = *loc;
                *loc = Loc::Hash;

                let (_, cycle) = fill_visited(&output);
                cycles += cycle as usize;

                let Some(loc) = grid::get_at_mut(&mut output.grid, (row, col)) else {
                    Err(anyhow::anyhow!("could not get location"))?
                };
                *loc = prev;
            }

            println!("{}", cycles);
            Ok(())
        }
    }
}
