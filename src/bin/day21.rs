use aoc24::{
    graph::{self, Graph},
    grid::{self, Direction},
    parser,
};
use clap::Parser;
use std::{collections::BTreeMap, io::Read};

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
enum NumberPad {
    Number(u8),
    #[default]
    A,
}

fn take_numberpad<'a>() -> impl Fn(&'a str) -> Option<(NumberPad, &'a str)> {
    move |input: &str| {
        let chr = input.chars().next()?;
        match chr {
            '0'..='9' => Some((
                NumberPad::Number(chr.to_digit(10).unwrap() as u8),
                &input[chr.len_utf8()..],
            )),
            'A' => Some((NumberPad::A, &input[chr.len_utf8()..])),
            _ => None,
        }
    }
}

impl NumberPad {
    fn grid() -> &'static [[Option<Self>; 3]; 4] {
        &[
            [
                Some(NumberPad::Number(7)),
                Some(NumberPad::Number(8)),
                Some(NumberPad::Number(9)),
            ],
            [
                Some(NumberPad::Number(4)),
                Some(NumberPad::Number(5)),
                Some(NumberPad::Number(6)),
            ],
            [
                Some(NumberPad::Number(1)),
                Some(NumberPad::Number(2)),
                Some(NumberPad::Number(3)),
            ],
            [None, Some(NumberPad::Number(0)), Some(NumberPad::A)],
        ]
    }

    fn all() -> &'static [Self] {
        &[
            NumberPad::Number(9),
            NumberPad::Number(8),
            NumberPad::Number(7),
            NumberPad::Number(6),
            NumberPad::Number(5),
            NumberPad::Number(4),
            NumberPad::Number(3),
            NumberPad::Number(2),
            NumberPad::Number(1),
            NumberPad::Number(0),
            NumberPad::A,
        ]
    }

    fn neighbors(&self) -> impl Iterator<Item = Self> {
        neighbors(Self::grid(), *self)
    }

    fn neighbor(&self, dir: Direction) -> Option<Self> {
        neighbor(Self::grid(), *self, dir)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
enum ArrowPad {
    Direction(grid::Direction),
    #[default]
    A,
}
impl ArrowPad {
    fn grid() -> &'static [[Option<Self>; 3]; 2] {
        &[
            [
                None,
                Some(ArrowPad::Direction(Direction::Up)),
                Some(ArrowPad::A),
            ],
            [
                Some(ArrowPad::Direction(Direction::Left)),
                Some(ArrowPad::Direction(Direction::Down)),
                Some(ArrowPad::Direction(Direction::Right)),
            ],
        ]
    }

    fn all() -> &'static [Self] {
        &[
            ArrowPad::Direction(Direction::Up),
            ArrowPad::Direction(Direction::Left),
            ArrowPad::Direction(Direction::Right),
            ArrowPad::Direction(Direction::Down),
            ArrowPad::A,
        ]
    }

    fn neighbors(&self) -> impl Iterator<Item = Self> {
        neighbors(Self::grid(), *self)
    }

    fn neighbor(&self, dir: Direction) -> Option<Self> {
        neighbor(Self::grid(), *self, dir)
    }
}

fn find<const N: usize, const M: usize, T: Eq, F>(
    arr: &[[T; M]; N],
    mut pred: F,
) -> Option<(isize, isize)>
where
    F: FnMut(&T) -> bool,
{
    arr.iter().enumerate().find_map(|(i, r)| {
        r.iter().enumerate().find_map(|(j, v)| {
            if pred(v) {
                Some((i as isize, j as isize))
            } else {
                None
            }
        })
    })
}

fn neighbors<const N: usize, const M: usize, T: Eq + Clone>(
    arr: &[[Option<T>; M]; N],
    value: T,
) -> impl Iterator<Item = T> + '_ {
    find(arr, |v| *v == Some(value.clone()))
        .into_iter()
        .flat_map(move |v| {
            grid::Direction::all_directions()
                .into_iter()
                .filter_map(move |d| -> Option<T> {
                    let pos = d.apply(v);
                    if pos.0 < 0 || pos.1 < 0 {
                        return None;
                    }
                    arr.get(pos.0 as usize)?.get(pos.1 as usize).cloned()?
                })
        })
}

fn neighbor<const N: usize, const M: usize, T: Eq + Clone>(
    arr: &[[Option<T>; M]; N],
    value: T,
    dir: Direction,
) -> Option<T> {
    let v = find(arr, |v| *v == Some(value.clone()))?;
    let pos = dir.apply(v);
    if pos.0 < 0 || pos.1 < 0 {
        return None;
    }
    arr.get(pos.0 as usize)?.get(pos.1 as usize).cloned()?
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// enum Button<A> {
//     Press(A),
//     Hover(A),
// }

// impl<A> Button<A> {
//     fn iter<I: Clone + IntoIterator<Item = A>>(into_iter: I) -> impl Iterator<Item = Self> {
//         into_iter
//             .clone()
//             .into_iter()
//             .map(Button::Press)
//             .chain(into_iter.into_iter().map(Button::Hover))
//     }
// }

fn create_graph_np_ap() -> Graph<(NumberPad, ArrowPad)> {
    let mut graph = Graph::new();
    for n1 in NumberPad::all().iter().cloned() {
        for a1 in ArrowPad::all().iter().cloned() {
            let v = (n1, a1);

            for dir in Direction::all_directions() {
                let a1n = ArrowPad::Direction(dir);
                let Some(n1n) = n1.neighbor(dir) else {
                    continue;
                };
                let u = (n1n, a1n);
                graph::add_edge(&mut graph, v, u, 1);
            }

            graph::add_edge(&mut graph, v, (n1, ArrowPad::A), 1);
        }
    }

    graph
}

fn create_graph_ap_ap() -> Graph<(ArrowPad, ArrowPad)> {
    let mut graph = Graph::new();
    for a1 in ArrowPad::all().iter().cloned() {
        for a2 in ArrowPad::all().iter().cloned() {
            let v = (a1, a2);
            for dir in grid::Direction::all_directions() {
                let a2n = ArrowPad::Direction(dir);
                let Some(a1n) = a1.neighbor(dir) else {
                    continue;
                };
                let u = (a1n, a2n);
                graph::add_edge(&mut graph, v, u, 1);
            }

            graph::add_edge(&mut graph, v, (a1, ArrowPad::A), 1);
        }
    }

    graph
}

type FullNode = (NumberPad, ArrowPad, ArrowPad, ArrowPad);

fn create_graph_full() -> Graph<FullNode> {
    let mut graph = Graph::new();
    for n1 in NumberPad::all().iter().cloned() {
        for a1 in ArrowPad::all().iter().cloned() {
            for a2 in ArrowPad::all().iter().cloned() {
                for a3 in ArrowPad::all().iter().cloned() {
                    let v = (n1, a1, a2, a3);

                    // for the last arrow pad, we add an edge from the current state, to the state
                    // the neighbor's arrow value.
                    for dir in Direction::all_directions() {
                        let a3n = ArrowPad::Direction(dir);
                        let Some(a2n) = a2.neighbor(dir) else {
                            continue;
                        };
                        let u = (n1, a1, a2n, a3n);
                        graph::add_edge(&mut graph, v, u, 1);
                    }

                    // You can press all buttons.
                    graph::add_edge(&mut graph, v, (n1, a1, a2, ArrowPad::A), 1);
                }

                // if the child node is pressed, then the parent node can move as well.
                // (0, ^, *) -> 
                let v = (n1, a1, a2, ArrowPad::A);
                for dir in Direction::all_directions() {
                    let a2n = ArrowPad::Direction(dir);
                    let Some(a1n) = a1.neighbor(dir) else {
                        continue;
                    };
                    let u = (n1, a1n, a2n, ArrowPad::A);
                    graph::add_edge(&mut graph, v, u, 1);
                }

                graph::add_edge(&mut graph, v, (n1, a1, ArrowPad::A, ArrowPad::A), 1);
            }

            let v = (n1, a1, ArrowPad::A, ArrowPad::A);
            for dir in Direction::all_directions() {
                let a1n = ArrowPad::Direction(dir);
                let Some(n1n) = n1.neighbor(dir) else {
                    continue;
                };
                let u = (n1n, a1n, ArrowPad::A, ArrowPad::A);
                graph::add_edge(&mut graph, v, u, 1);
            }

            graph::add_edge(
                &mut graph,
                v,
                (n1, ArrowPad::A, ArrowPad::A, ArrowPad::A),
                1,
            );
        }
    }

    graph
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Vec<NumberPad>>> {
    let mut file = std::fs::File::open(filename)?;
    let mut str = String::new();
    file.read_to_string(&mut str)?;

    let take_line = parser::take_many1(take_numberpad());
    let take_lines = parser::take_separator(take_line, parser::take_newline());
    let take_input = parser::take_first(take_lines, parser::take_eol());

    let (lines, _) = take_input(str.as_str()).ok_or_else(|| anyhow::anyhow!("could not parse"))?;

    Ok(lines)
}

type PathCache<N> = BTreeMap<(N, N), BTreeMap<N, Vec<N>>>;

fn solve_fn(
    graph: &Graph<FullNode>,
    apsp: &graph::AllPairShortestPaths<&FullNode>,
    expected: &[NumberPad],
    all_paths: &mut PathCache<FullNode>,
) -> anyhow::Result<[Vec<ArrowPad>; 3]> {
    let start: FullNode = (NumberPad::default(), ArrowPad::A, ArrowPad::A, ArrowPad::A);
    let mut cur = start;
    let mut res = [vec![], vec![], vec![]];
    let mut push = |v: FullNode| {
        res[2].push(v.3);
        if v.3 == ArrowPad::A {
            res[1].push(v.2);
            if v.2 == ArrowPad::A {
                res[0].push(v.1);
            }
        }
    };

    for next in expected.iter().cloned() {
        let n = (next, ArrowPad::A, ArrowPad::A, ArrowPad::A);
        let Some(distances) = apsp.get(&cur) else {
            return Err(anyhow::anyhow!("no path found due to no distances"));
        };

        let paths = all_paths
            .entry((cur, n))
            .or_insert_with(|| graph::all_paths(graph, distances, cur, n));

        if cur == n {
            // he path is a self loop
            push(n);
            continue;
        }

        while cur != n {
            let v = *paths
                .get(&cur)
                .ok_or(anyhow::anyhow!("no path found for {:?} to {:?}", cur, next))?
                .first()
                .ok_or(anyhow::anyhow!("no path found: node has no edges"))?;
            cur = v;
            print!("{:?} ", v);
            push(v);
        }
    }

    Ok(res)
}

fn solve<N: Ord + Copy + Default + std::fmt::Debug>(
    graph: &Graph<(N, ArrowPad)>,
    apsp: &graph::AllPairShortestPaths<&(N, ArrowPad)>,
    expected: &[N],
    all_paths: &mut PathCache<(N, ArrowPad)>,
) -> anyhow::Result<Vec<ArrowPad>> {
    let start = (N::default(), ArrowPad::A);
    let mut cur = start;
    let mut res = vec![];

    for next in expected.iter().cloned() {
        let Some(distances) = apsp.get(&cur) else {
            return Err(anyhow::anyhow!("no path found due to no distances"));
        };

        let paths = all_paths
            .entry((cur, (next, ArrowPad::A)))
            .or_insert_with(|| graph::all_paths(graph, distances, cur, (next, ArrowPad::A)));

        println!("trying {:?} to {:?}", cur, next);

        if cur == (next, ArrowPad::A) {
            // the path is a self loop
            res.push(ArrowPad::A);
            continue;
        }

        let mut idx = 0;
        while idx == 0 || cur != (next, ArrowPad::A) {
            idx += 1;
            let v = *paths
                .get(&cur)
                .ok_or(anyhow::anyhow!("no path found for {:?} to {:?}", cur, next))?
                .first()
                .ok_or(anyhow::anyhow!("no path found: node has no edges"))?;
            cur = v;
            res.push(v.1);
        }
    }

    Ok(res)
}

fn solve_np_ap_ap_ap(input: &[Vec<NumberPad>]) -> anyhow::Result<Vec<Vec<ArrowPad>>> {
    let np_ap_graph = create_graph_np_ap();
    let np_ap_paths = graph::all_pairs_shortest_paths(&np_ap_graph);
    let mut np_ap_cache = PathCache::<(NumberPad, ArrowPad)>::new();

    let ap_ap_graph = create_graph_ap_ap();
    let ap_ap_paths = graph::all_pairs_shortest_paths(&ap_ap_graph);
    let mut ap_ap_cache = PathCache::<(ArrowPad, ArrowPad)>::new();

    let mut res = vec![];
    for line in input.iter() {
        let moves_np_ap = solve(&np_ap_graph, &np_ap_paths, line, &mut np_ap_cache)?;
        let moves_np_ap_ap = solve(&ap_ap_graph, &ap_ap_paths, &moves_np_ap, &mut ap_ap_cache)?;
        let moves_ap_ap_ap_ap = solve(
            &ap_ap_graph,
            &ap_ap_paths,
            &moves_np_ap_ap,
            &mut ap_ap_cache,
        )?;
        res.extend([moves_np_ap, moves_np_ap_ap, moves_ap_ap_ap_ap]);
    }

    Ok(res)
}

fn solve_fn_full(input: &[Vec<NumberPad>]) -> anyhow::Result<Vec<Vec<ArrowPad>>> {
    let full_graph = create_graph_full();
    let full_paths = graph::all_pairs_shortest_paths(&full_graph);
    let mut path_cache = PathCache::<FullNode>::new();

    let mut res = vec![];
    for line in input.iter() {
        let moves = solve_fn(&full_graph, &full_paths, line, &mut path_cache)?;
        res.extend(moves);
    }

    Ok(res)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let res = parse_file(&file)?;
            let arrowpads = solve_fn_full(&res)?;
            for arrowpads in arrowpads.iter() {
                let mut s = String::new();
                for arrowpad in arrowpads.iter() {
                    match arrowpad {
                        ArrowPad::Direction(Direction::Up) => s.push('^'),
                        ArrowPad::Direction(Direction::Down) => s.push('v'),
                        ArrowPad::Direction(Direction::Left) => s.push('<'),
                        ArrowPad::Direction(Direction::Right) => s.push('>'),
                        ArrowPad::A => s.push('A'),
                    }
                }
                println!("{}", s);
            }
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;

            todo!();
            Ok(())
        }
    }
}
