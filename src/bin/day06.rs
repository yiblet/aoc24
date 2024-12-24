use aoc24::{parser, util};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Up = 1 << 0,
    Down = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
}

impl State {
    fn rotate_90_right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }

    fn apply(self, cur: (isize, isize)) -> (isize, isize) {
        let delta = match self {
            Self::Up => (-1, 0),
            Self::Right => (0, 1),
            Self::Left => (0, -1),
            Self::Down => (1, 0),
        };

        (cur.0 + delta.0, cur.1 + delta.1)
    }

    fn apply_inverse(self, cur: (isize, isize)) -> (isize, isize) {
        let delta = match self {
            Self::Up => (-1, 0),
            Self::Right => (0, 1),
            Self::Left => (0, -1),
            Self::Down => (1, 0),
        };

        (cur.0 - delta.0, cur.1 - delta.1)
    }
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

    if let Some(err) = lines.error() {
        Err(err)?
    }

    input
}

fn get_at<V>(grid: &Vec<Vec<V>>, pos: (isize, isize)) -> Option<&V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get(row as usize)?.get(col as usize)
}

fn get_at_mut<V>(grid: &mut Vec<Vec<V>>, pos: (isize, isize)) -> Option<&mut V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get_mut(row as usize)?.get_mut(col as usize)
}

fn fill_visited(output: &Grid) -> (Vec<Vec<usize>>, bool) {
    let mut visited: Vec<Vec<usize>> = output
        .grid
        .iter()
        .map(|v| v.iter().map(|_| 0).collect())
        .collect();

    let mut pos = (output.start.0 as isize, output.start.1 as isize);
    let mut state = State::Up;
    let mut cycle = false;

    while let Some((loc, visited)) = get_at(&output.grid, pos).zip(get_at_mut(&mut visited, pos)) {
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
                let Some(loc) = get_at_mut(&mut output.grid, (row, col)) else {
                    Err(anyhow::anyhow!("could not get location"))?
                };
                let prev = *loc;
                *loc = Loc::Hash;

                let (_, cycle) = fill_visited(&output);
                cycles += cycle as usize;

                let Some(loc) = get_at_mut(&mut output.grid, (row, col)) else {
                    Err(anyhow::anyhow!("could not get location"))?
                };
                *loc = prev;
            }

            println!("{}", cycles);
            Ok(())
        }
    }
}
