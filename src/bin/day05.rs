use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

use aoc24::{parser, util};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn take_ordering<'a>() -> impl Fn(&'a str) -> Option<((i64, i64), &'a str)> {
    parser::map(
        parser::take_tuple3(
            parser::take_int(),
            parser::take_str("|"),
            parser::take_int(),
        ),
        |(l, _, r)| (l, r),
    )
}

fn take_input_line<'a>() -> impl Fn(&'a str) -> Option<(Vec<i64>, &'a str)> {
    parser::take_separator(parser::take_int(), parser::take_str(","))
}

#[derive(Debug)]
struct ParsedResult {
    orderings: Vec<(i64, i64)>,
    inputs: Vec<Vec<i64>>,
}

fn parse_input<S: AsRef<str>, I: Iterator<Item = S>>(lines: I) -> anyhow::Result<ParsedResult> {
    let mut orderings = vec![];
    let mut inputs = vec![];

    let mut lines = lines.peekable();
    while let Some(line) = lines.peek() {
        let line = line.as_ref();
        match take_ordering()(line) {
            Some((ordering, "")) => {
                orderings.push(ordering);
            }
            Some(_) => {
                Err(anyhow::anyhow!("could not finish parsing {:}", line))?;
            }
            None => {
                break;
            }
        }

        lines.next();
    }

    let Some(_) = lines.peek().and_then(|l| parser::take_eol()(l.as_ref())) else {
        Err(anyhow::anyhow!(
            "could not finish parsing {:}",
            lines.peek().unwrap().as_ref()
        ))?
    };
    lines.next();

    while let Some(line) = lines.peek() {
        let line = line.as_ref();
        match take_input_line()(line) {
            Some((input, "")) => {
                inputs.push(input);
            }
            Some(_) => {
                Err(anyhow::anyhow!("could not finish parsing {:}", line))?;
            }
            None => {
                break;
            }
        }

        lines.next();
    }

    Ok(ParsedResult { orderings, inputs })
}

fn parse_file(filename: &str) -> anyhow::Result<ParsedResult> {
    let mut lines = util::read_file_lines(filename)?;

    let input = parse_input(&mut lines);

    if let Some(err) = lines.error() {
        Err(err)?
    }

    input
}

fn toposort(
    orderings: &HashMap<i64, HashSet<i64>>,
    node_subset: &HashSet<i64>,
) -> anyhow::Result<Vec<i64>> {
    fn visit(
        orderings: &HashMap<i64, HashSet<i64>>,
        node_subset: &HashSet<i64>,
        visited: &mut HashMap<i64, bool>,
        res: &mut Vec<i64>,
        node: i64,
    ) -> bool {
        if let Some(v) = visited.get(&node) {
            return *v;
        };

        visited.insert(node, false);
        if let Some(nodes) = orderings.get(&node) {
            for node in nodes.intersection(node_subset) {
                if !visit(orderings, node_subset, visited, res, *node) {
                    return false;
                }
            }
        }

        visited.insert(node, true);
        res.push(node);
        true
    }

    let mut res = vec![];
    let mut visited = HashMap::new();
    let mut nodes = node_subset.iter().copied().collect::<Vec<_>>();

    while let Some(node) = nodes.pop() {
        if visited.contains_key(&node) {
            continue;
        }
        if !visit(orderings, node_subset, &mut visited, &mut res, node) {
            return Err(anyhow::anyhow!("cycle detected"));
        }
    }

    res.reverse();
    Ok(res)
}

impl ParsedResult {
    fn orderings(&self) -> HashMap<i64, HashSet<i64>> {
        self.orderings
            .iter()
            .fold(HashMap::<i64, HashSet<i64>>::new(), |mut acc, (l, r)| {
                acc.entry(*l).or_default().insert(*r);
                acc
            })
    }

    fn iter_input<'a>(
        &'a self,
        ords: &'a HashMap<i64, HashSet<i64>>,
    ) -> impl Iterator<Item = (bool, &'a [i64])> + 'a {
        let is_before = |l: i64, r: i64| ords.get(&l).map(|s| s.contains(&r));
        let check = move |a: i64, b: i64| match is_before(a, b) {
            Some(v) => v,
            None => !matches!(is_before(b, a), Some(true)),
        };

        self.inputs.iter().map(move |input| {
            let valid = input
                .iter()
                .cloned()
                .enumerate()
                .flat_map(|(i, a)| input[i + 1..].iter().cloned().map(move |b| check(a, b)))
                .all(|v| v);
            (valid, input.as_slice())
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let ords = output.orderings();

            let sum = output
                .iter_input(&ords)
                .filter_map(|(valid, input)| {
                    if valid {
                        Some(input[input.len() / 2])
                    } else {
                        None
                    }
                })
                .sum::<i64>();
            println!("{}", sum);
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let ords = output.orderings();

            let sum = output
                .iter_input(&ords)
                .filter_map(|(valid, input)| {
                    if valid {
                        return None;
                    }
                    let nodes = input.iter().copied().collect::<HashSet<_>>();
                    let Ok(order) = toposort(&ords, &nodes) else {
                        return None;
                    };
                    Some(order[order.len() / 2])
                })
                .sum::<i64>();

            println!("{}", sum);
        }
    };

    Ok(())
}
