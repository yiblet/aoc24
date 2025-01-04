use std::collections::{BTreeMap, BTreeSet};

use aoc24::{
    graph::{self, rev_all_paths, dijkstras},
    grid::{self},
};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Start,
    End,
    Space,
    Wall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedResult {
    grid: grid::Grid<Item>,
    start: grid::Index,
    end: grid::Index,
}

type Node = (grid::Index, grid::Direction);

// sentinel nodes to represent the start and end of the graph
const START_NODE: (grid::Index, grid::Direction) = ((isize::MIN, isize::MIN), grid::Direction::Up);
const END_NODE: (grid::Index, grid::Direction) = ((isize::MAX, isize::MAX), grid::Direction::Up);

impl ParsedResult {
    fn create_graph(&self) -> graph::Graph<Node> {
        let mut res = graph::Graph::<Node>::new();
        for (pos, item) in grid::iter_pos(&self.grid) {
            for dir in grid::Direction::all_directions() {
                match item {
                    Item::Start | Item::End | Item::Space => {
                        let pos2 = dir.apply(pos);
                        if !matches!(grid::get_at(&self.grid, pos2), Some(Item::Wall) | None) {
                            let v = res.entry((pos, dir)).or_default();
                            v.insert(((pos2, dir), 1));
                        }

                        for dir2 in grid::Direction::all_directions() {
                            if dir2 == dir {
                                continue;
                            }
                            res.entry((pos, dir))
                                .or_default()
                                .insert(((pos, dir2), 1000));
                        }
                    }
                    Item::Wall => {
                        res.entry((pos, dir)).or_default();
                    }
                }
            }
        }

        res.entry(START_NODE)
            .or_default()
            .insert(((self.start, grid::Direction::Right), 0));

        // connect all possible end positions
        for dir in grid::Direction::all_directions() {
            res.entry((self.end, dir))
                .or_default()
                .insert((END_NODE, 0));
        }

        res
    }

    fn shortest_path(&self) -> Option<usize> {
        let graph = self.create_graph();
        let res = dijkstras(&graph, &START_NODE);
        println!("{:?}", res);
        res.get(&END_NODE).cloned()
    }

    fn all_shortest_paths(&self) -> BTreeMap<Node, Vec<Node>> {
        let graph = self.create_graph();
        let distances = dijkstras(&graph, &START_NODE);
        rev_all_paths(&graph, &distances, START_NODE, END_NODE)
    }
}

fn all_nodes_in_paths(paths: &BTreeMap<Node, Vec<Node>>) -> BTreeSet<grid::Index> {
    let mut res = BTreeSet::new();
    for path in paths.iter() {
        res.extend(path.1.iter().map(|n| n.0));
        res.insert(path.0 .0);
    }
    res.remove(&END_NODE.0);
    res.remove(&START_NODE.0);
    res
}

#[allow(unused)]
fn convert_paths<Node: std::cmp::Ord + Copy>(
    paths: &BTreeMap<Node, Vec<Node>>,
    start: Node,
    end: Node,
) -> Vec<Vec<Node>> {
    let mut res = Vec::new();
    let mut stack: Vec<(Node, Vec<Node>)> = vec![(end, vec![end])];

    while let Some((cur, mut path)) = stack.pop() {
        if cur == start {
            path.push(cur);
            path.reverse();
            res.push(path);
            continue;
        }

        paths.get(&cur).into_iter().flatten().for_each(|n| {
            stack.push((*n, path.clone()));
        })
    }

    res
}

fn parse_file(filename: &str) -> anyhow::Result<ParsedResult> {
    let mut file = std::fs::File::open(filename)?;
    let grid = grid::read_grid(&mut file)?;
    let grid = grid::map_result(&grid, |_pos, chr| match chr {
        '.' => Ok(Item::Space),
        '#' => Ok(Item::Wall),
        'S' => Ok(Item::Start),
        'E' => Ok(Item::End),
        c => Err(anyhow::anyhow!("invalid character: {c}")),
    })?;

    let start_pos = grid::iter_pos(&grid)
        .find(|(_, chr)| **chr == Item::Start)
        .ok_or_else(|| anyhow::anyhow!("could not find start position"))?;

    let end_pos = grid::iter_pos(&grid)
        .find(|(_, chr)| **chr == Item::End)
        .ok_or_else(|| anyhow::anyhow!("could not find end position"))?;

    Ok(ParsedResult {
        start: start_pos.0,
        end: end_pos.0,
        grid,
    })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let res = output
                .shortest_path()
                .ok_or_else(|| anyhow::anyhow!("could not find shortest path"))?;

            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let paths = output.all_shortest_paths();
            let nodes = all_nodes_in_paths(&paths);

            let s = print_paths(&output, &nodes);
            println!("{}", s);

            let res = nodes.len();
            println!("{}", res);
            Ok(())
        }
    }
}

fn print_paths(graph: &ParsedResult, nodes: &BTreeSet<grid::Index>) -> String {
    let mut res = String::new();
    for (row, line) in graph.grid.iter().enumerate() {
        for (col, chr) in line.iter().enumerate() {
            if nodes.contains(&(row as isize, col as isize)) {
                res.push('O');
            } else {
                match chr {
                    Item::Start => res.push('S'),
                    Item::End => res.push('E'),
                    Item::Space => res.push('.'),
                    Item::Wall => res.push('#'),
                }
            }
        }

        res.push('\n');
    }
    res
}
