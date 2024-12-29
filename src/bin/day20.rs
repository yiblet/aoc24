use std::collections::{BTreeMap, BTreeSet};

use aoc24::{graph, grid};
use clap::Parser;
use either::Either;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long)]
    debug: bool,

    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, clap::Subcommand)]
enum Cmd {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

// solution idea:
// 1. dijkstras from start to 1
// 2. dijkstras from 1 to end

// opposing solution idea:
// for the graph, index i create: (i, 0), (i, 1), (i, 2) and (i, 3)
// (i, 0) -> (n, 0)
// (i, 0) -> (n, 1)
// (n, 1) -> (n, 2)
// (n, 1) -> (n, 3)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Shortcut {
    Pre,
    SecondFirst,
    SecondLast,
    Post,
}

impl Shortcut {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Pre, Self::SecondFirst, Self::SecondLast, Self::Post].into_iter()
    }
}

type Index = (grid::Index, Shortcut);

fn create_graph2(grid: &grid::Grid<char>, skip_max_distance: usize) -> graph::Graph<Index> {
    if skip_max_distance < 2 {
        panic!("skip_max_distance must be at least 2");
    }

    let mut graph: graph::Graph<Index> = graph::Graph::<Index>::new();
    let mut new_edge = |pos1: Index, pos2: Index, weight: usize| {
        graph.entry(pos1).or_default().insert((pos2, weight));
    };
    for (pos, c) in grid::iter_pos(grid) {
        if *c == 'E' {
            new_edge((pos, Shortcut::SecondLast), (pos, Shortcut::Post), 0);
        }
        if *c == 'S' {
            new_edge((pos, Shortcut::Pre), (pos, Shortcut::SecondFirst), 0);
        }
        for dir in grid::Direction::all_directions() {
            let pos2 = dir.apply(pos);
            let Some(c2) = grid::get_at(grid, pos2).cloned() else {
                continue;
            };
            for shortcut in Shortcut::all() {
                match shortcut {
                    Shortcut::Pre => {
                        if *c == '#' {
                            continue;
                        }
                        if c2 != '#' {
                            new_edge((pos, shortcut), (pos2, Shortcut::Pre), 1);
                            new_edge((pos, shortcut), (pos2, Shortcut::SecondFirst), 1);
                        }
                    }
                    Shortcut::SecondFirst => {
                        // skip - this case will be handled in the second inner for loop
                    }
                    Shortcut::SecondLast => {
                        if *c == '#' {
                            continue;
                        }
                        if c2 != '#' {
                            new_edge((pos, shortcut), (pos2, Shortcut::Post), 1);
                        }
                    }
                    Shortcut::Post => {
                        if *c == '#' {
                            continue;
                        }
                        if c2 != '#' {
                            new_edge((pos, shortcut), (pos2, Shortcut::Post), 1);
                        }
                    }
                }
            }
        }

        // handle the case for second first to second last
        for delta in bounded_distance(skip_max_distance) {
            if delta == (0, 0) {
                continue;
            }
            let pos2 = vec2_add(pos, delta);
            let Some(c2) = grid::get_at(grid, pos2).cloned() else {
                continue;
            };
            if c2 == '#' {
                continue;
            }
            let distance = magnitude(delta);
            new_edge(
                (pos, Shortcut::SecondFirst),
                (pos2, Shortcut::SecondLast),
                distance,
            );
        }
    }
    graph
}

// returns a set of all pairs of locations that are within the given distance
fn bounded_distance(distance: usize) -> impl Iterator<Item = (isize, isize)> {
    (0..=distance as isize).flat_map(move |n| {
        (-n..=n).flat_map(move |x| {
            let y = n - x.abs();
            let res = if y != 0 {
                Either::Left([(x, y), (x, -y)])
            } else {
                Either::Right([(x, y)])
            };
            res.into_iter()
        })
    })
}

fn vec2_add(v1: grid::Index, v2: grid::Index) -> grid::Index {
    (v1.0 + v2.0, v1.1 + v2.1)
}

fn magnitude(v: grid::Index) -> usize {
    v.0.unsigned_abs() + v.1.unsigned_abs()
}

fn find_start(graph: &grid::Grid<char>) -> anyhow::Result<Index> {
    let res = grid::iter_pos(graph)
        .find(|(_, c)| **c == 'S')
        .ok_or_else(|| anyhow::anyhow!("could not find start"))?
        .0;

    Ok((res, Shortcut::Pre))
}

fn find_end(graph: &grid::Grid<char>) -> anyhow::Result<Index> {
    let res = grid::iter_pos(graph)
        .find(|(_, c)| **c == 'E')
        .ok_or_else(|| anyhow::anyhow!("could not find end"))?
        .0;

    Ok((res, Shortcut::Post))
}

fn parse_file(filename: &str) -> anyhow::Result<grid::Grid<char>> {
    let mut file = std::fs::File::open(filename)?;
    let grid = grid::read_grid(&mut file)?;
    Ok(grid)
}

#[allow(unused)]
struct RunResult {
    start: Index,
    end: Index,
    counts: BTreeMap<usize, BTreeSet<(grid::Index, grid::Index)>>,
    default_distance: usize,
}

fn run_problem(output: &grid::Grid<char>, max_seconds: usize) -> anyhow::Result<RunResult> {
    let graph = create_graph2(output, max_seconds);
    let rev_graph = graph::reverse_graph(&graph);
    let start = find_start(output)?;
    let end = find_end(output)?;

    let distance_to_end: BTreeMap<&Index, usize> = graph::dijkstras(&rev_graph, &end);
    let distance_from_start: BTreeMap<&Index, usize> = graph::dijkstras(&graph, &start);

    let default_distance = distance_to_end
        .get(&(start.0, Shortcut::Post))
        .copied()
        .ok_or_else(|| anyhow::anyhow!("could not find distance"))?;

    let mut counts = BTreeMap::<usize, BTreeSet<(grid::Index, grid::Index)>>::new();
    for (s1, _) in grid::iter_pos(output) {
        let Some(d1) = distance_from_start
            .get(&(s1, Shortcut::SecondFirst))
            .copied()
        else {
            continue;
        };
        for delta in bounded_distance(max_seconds) {
            let s2 = vec2_add(s1, delta);
            if grid::get_at(output, s2).is_none() {
                continue;
            }
            let Some(d2) = distance_to_end.get(&(s2, Shortcut::SecondLast)).copied() else {
                continue;
            };

            let dist = d1 + d2 + magnitude(delta);
            if dist < default_distance {
                counts
                    .entry(default_distance - dist)
                    .or_default()
                    .insert((s1, s2));
            }
        }
    }

    Ok(RunResult {
        start,
        end,
        counts,
        default_distance,
    })
}

fn main() -> anyhow::Result<()> {
    let Args { debug, cmd } = Args::parse();
    match cmd {
        Cmd::Part1 { file } => {
            let output = parse_file(&file)?;
            let res = run_problem(&output, 2)?;

            if debug {
                println!("default: {}", res.default_distance);
                println!("end loc: {:?}", res.end);
                for (dist, pairs) in res.counts.iter() {
                    println!("{dist}: {:} - {:?}", pairs.len(), pairs);
                }
            }

            let cheats = res.counts.range(100..).map(|d| d.1.len()).sum::<usize>();
            println!("{cheats}");
            Ok(())
        }

        Cmd::Part2 { file } => {
            let output = parse_file(&file)?;
            let res = run_problem(&output, 20)?;

            if debug {
                println!("default: {}", res.default_distance);
                println!("end loc: {:?}", res.end);
                for (dist, pairs) in res.counts.iter() {
                    println!("{dist}: {:} - {:?}", pairs.len(), pairs);
                }
            }

            let cheats = res.counts.range(100..).map(|d| d.1.len()).sum::<usize>();
            println!("{cheats}");
            Ok(())
        }
    }
}
