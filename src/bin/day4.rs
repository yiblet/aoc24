use clap::Parser;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Char {
    X,
    M,
    A,
    S,
    Other(char),
}

type Grid = Vec<Vec<Char>>;

fn parse_file(filename: &str) -> anyhow::Result<Grid> {
    let mut file = std::fs::File::open(filename)?;
    let bufreader = std::io::BufReader::new(&mut file);

    let res = bufreader
        .lines()
        .map(|line| {
            let line = line?;
            let res = line
                .chars()
                .map(|c| match c {
                    'X' => Char::X,
                    'M' => Char::M,
                    'A' => Char::A,
                    'S' => Char::S,
                    c => Char::Other(c),
                })
                .collect::<Vec<_>>();

            Ok(res)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(res)
}

fn lookup(grid: &Grid, x: isize, y: isize) -> Option<Char> {
    if x < 0 || y < 0 {
        return None;
    }
    Some(*grid.get(y as usize)?.get(x as usize)?)
}

fn check(grid: &Grid, x: usize, y: usize, delta: (isize, isize), expected: &[Char]) -> bool {
    let mut cur = (x as isize, y as isize);

    for c in expected {
        let (x, y) = cur;
        let l = lookup(grid, x, y);
        if Some(*c) != l {
            return false;
        }
        cur = (cur.0 + delta.0, cur.1 + delta.1);
    }

    true
}

fn check_all_part1(grid: &Grid, x: usize, y: usize) -> usize {
    let check_part1 = |grid: &Grid, x: usize, y: usize, delta: (isize, isize)| {
        check(grid, x, y, delta, &[Char::X, Char::M, Char::A, Char::S])
    };

    let deltas = [
        (-1, 0),
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ];

    deltas
        .iter()
        .filter(|delta| check_part1(grid, x, y, **delta))
        .count()
}
fn check_all_part2(grid: &Grid, x: usize, y: usize) -> bool {
    let check_part2 = |grid: &Grid, x: usize, y: usize, delta: (isize, isize)| {
        check(grid, x, y, delta, &[Char::M, Char::A, Char::S])
    };
    let deltas = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let Some(Char::A) = lookup(grid, x as isize, y as isize) else {
        return false;
    };

    deltas
        .iter()
        .filter(|delta| {
            let x = x as isize - delta.0;
            if x < 0 {
                return false;
            }
            let x = x as usize;

            let y = y as isize - delta.1;
            if y < 0 {
                return false;
            }
            let y = y as usize;

            check_part2(grid, x, y, **delta)
        })
        .count()
        == 2
}

fn find_all(grid: &Grid, chr: Char) -> impl Iterator<Item = (usize, usize)> + '_ {
    grid.iter().enumerate().flat_map(move |(y, row)| {
        row.iter()
            .enumerate()
            .filter_map(move |(x, c)| if *c == chr { Some((x, y)) } else { None })
    })
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
            let grid = parse_file(&file)?;

            let sum = find_all(&grid, Char::X)
                .map(|(x, y)| check_all_part1(&grid, x, y))
                .sum::<usize>();

            println!("{}", sum);
            Ok(())
        }

        Args::Part2 { file } => {
            let grid = parse_file(&file)?;

            let count = find_all(&grid, Char::A)
                .filter(|(x, y)| check_all_part2(&grid, *x, *y))
                .count();

            println!("{}", count);
            Ok(())
        }
    }
}
