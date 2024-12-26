use std::{cmp, collections::BTreeSet, io::Read};

use aoc24::parser;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn take_button(chr: char) -> impl Fn(&str) -> Option<((isize, isize), &str)> {
    move |input: &str| {
        let (_, rest) = parser::take_str("Button ")(input)?;
        let (_, rest) = parser::take_char(chr)(rest)?;
        let (_, rest) = parser::take_str(": X+")(rest)?;
        let (x, rest) = parser::take_int()(rest)?;
        let (_, rest) = parser::take_str(", Y+")(rest)?;
        let (y, rest) = parser::take_int()(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        Some(((x as isize, y as isize), rest))
    }
}

fn take_prize() -> impl Fn(&str) -> Option<((isize, isize), &str)> {
    move |input: &str| {
        let (_, rest) = parser::take_str("Prize: X=")(input)?;
        let (x, rest) = parser::take_int()(rest)?;
        let (_, rest) = parser::take_str(", Y=")(rest)?;
        let (y, rest) = parser::take_int()(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        Some(((x as isize, y as isize), rest))
    }
}

#[derive(Debug, Clone, Copy)]
struct Game {
    a: (isize, isize),
    b: (isize, isize),
    prize: (isize, isize),
}

fn cost((a, b): (isize, isize)) -> isize {
    3 * a + b
}

fn cost_f64((a, b): (f64, f64)) -> f64 {
    3.0 * a + b
}

fn matmul2x2v2(a: [[f64; 2]; 2], b: [f64; 2]) -> [f64; 2] {
    [
        a[0][0] * b[0] + a[0][1] * b[1],
        a[1][0] * b[0] + a[1][1] * b[1],
    ]
}

fn invert2x2(a: [[f64; 2]; 2]) -> Option<[[f64; 2]; 2]> {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if det == 0.0 {
        return None;
    }

    Some([
        [(a[1][1] / det), (-a[0][1] / det)],
        [(-a[1][0] / det), (a[0][0] / det)],
    ])
}

fn round(x: f64) -> Option<f64> {
    let res = x.round();
    if (x - res).abs() < 1e-4 {
        Some(res)
    } else {
        None
    }
}

fn solve2(game: &Game) -> Option<(f64, f64)> {
    let a = [
        [game.a.0 as f64, game.b.0 as f64],
        [game.a.1 as f64, game.b.1 as f64],
    ];
    let prize = [game.prize.0 as f64, game.prize.1 as f64];

    let Some(a_inverse) = invert2x2(a) else {
        println!("matrix: a_inverse not found");
        return None;
    };
    let s = matmul2x2v2(a_inverse, prize);

    let s0 = s[0];
    let s1 = s[1];

    let (x, y) = (round(s0)?, round(s1)?);
    if x < 0.0 || y < 0.0 {
        println!("solution is negative");
        return None;
    }

    Some((x, y))
}

fn solve(game: &Game) -> Option<(isize, isize)> {
    let col1_solutions = solutions(game.a.0, game.b.0, game.prize.0).collect::<BTreeSet<_>>();
    let col2_solutions = solutions(game.a.1, game.b.1, game.prize.1).collect::<BTreeSet<_>>();
    col1_solutions
        .intersection(&col2_solutions)
        .cloned()
        .min_by_key(|s| cost(*s))
}

fn take_game<'a>() -> impl Fn(&'a str) -> Option<(Game, &'a str)> {
    move |input: &str| {
        let (a, rest) = take_button('A')(input)?;
        let (b, rest) = take_button('B')(rest)?;
        let (prize, rest) = take_prize()(rest)?;
        Some((Game { a, b, prize }, rest))
    }
}

fn take_games<'a>() -> impl Fn(&'a str) -> Option<(Vec<Game>, &'a str)> {
    parser::take_first(
        parser::take_separator(take_game(), parser::take_newline()),
        parser::take_eol(),
    )
}

fn solutions(a: isize, b: isize, c: isize) -> impl Iterator<Item = (isize, isize)> {
    // a * x + b * y = c
    // c - a * x = b * y
    // y = (c - a * x) / b
    (0..=cmp::min(c / a, 100)).filter_map(move |x| {
        let c2 = c - a * x;
        let (b_div, b_rem) = (c2 / b, c2 % b);
        if b_rem == 0 {
            if b_div > 100 {
                return None;
            }
            Some((x, b_div))
        } else {
            None
        }
    })
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Game>> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let (games, _) = take_games()(&string).ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    Ok(games)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let total_cost = output
                .iter()
                .enumerate()
                .filter_map(|(i, game)| solve(game).map(|x| (i, game, x)))
                .inspect(|(id, game, x)| println!("id: {} x: {:?} game={:?}", id, x, game))
                .map(|(_, _, s)| cost(s))
                .sum::<isize>();

            println!("{}", total_cost);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let total_cost = output
                .iter()
                .enumerate()
                .map(|(i, game)| {
                    (
                        i,
                        Game {
                            a: game.a,
                            b: game.b,
                            prize: (game.prize.0 + 10000000000000, game.prize.1 + 10000000000000),
                        },
                    )
                })
                .filter_map(|(i, game)| solve2(&game).map(|x| (i, game, x)))
                .inspect(|(id, game, x)| println!("id: {} x: {:?} game={:?}", id, x, game))
                .map(|(_, _, s)| cost_f64(s))
                .sum::<f64>();

            println!("{}", total_cost);
            Ok(())
        }
    }
}
