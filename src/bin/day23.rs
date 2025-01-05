use core::str;
use std::{collections::BTreeSet, fmt, io::Read};

use aoc24::{
    graph::{self, Graph},
    parser,
};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Node([u8; 2]);

impl Node {
    fn new(name: [u8; 2]) -> Self {
        Self(name)
    }

    fn as_str(&self) -> Result<&str, str::Utf8Error> {
        let res = str::from_utf8(&self.0)?;
        Ok(res)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.as_str()
            .map(|s| s.starts_with(prefix))
            .unwrap_or(false)
    }
}

fn take_node<'a>() -> impl Fn(&'a str) -> Option<(Node, &'a str)> {
    move |input: &str| {
        if input.len() < 2 {
            return None;
        }
        let mut res: [u8; 2] = [0; 2];
        let node = &input[0..2];
        res.copy_from_slice(node.as_bytes());
        if !res.iter().all(|c| (*c as char).is_ascii_alphabetic()) {
            return None;
        }

        Some((Node::new(res), &input[2..]))
    }
}

fn take_edge<'a>() -> impl Fn(&'a str) -> Option<((Node, Node), &'a str)> {
    move |input: &str| {
        let (node1, rest) = take_node()(input)?;
        let (_, rest) = parser::take_str("-")(rest)?;
        let (node2, rest) = take_node()(rest)?;
        Some(((node1, node2), rest))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<(Node, Node)>> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let (nodes, rest) = parser::take_separator(take_edge(), parser::take_newline())(&string)
        .ok_or_else(|| anyhow::anyhow!("could not parse file"))?;

    if !rest.is_empty() {
        Err(anyhow::anyhow!("could not parse file, remaining: {rest}"))?;
    }

    Ok(nodes)
}

fn write_nodes<'a, W: fmt::Write>(
    w: &'a mut W,
    nodes: impl Iterator<Item = &'a Node>,
) -> fmt::Result {
    fn conv(n: &Node) -> &str {
        n.as_str().unwrap_or("??")
    }

    for (i, n) in nodes.enumerate() {
        if i != 0 {
            write!(w, ",")?;
        }
        write!(w, "{}", conv(n))?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let mut graph = Graph::new();
            for (n1, n2) in output.iter() {
                graph::add_edge(&mut graph, *n1, *n2, 1);
                graph::add_edge(&mut graph, *n2, *n1, 1);
            }

            let mut triplets = BTreeSet::new();
            for v in graph.keys().filter(|n| n.starts_with("t")) {
                for (u, _) in graph::neighbors(&graph, v) {
                    let common_neighbors = graph.get(u).zip(graph.get(v)).into_iter().flat_map(
                        |(u_neighbors, v_neighbors)| u_neighbors.intersection(v_neighbors),
                    );
                    for (w, _) in common_neighbors {
                        if w == u || w == v {
                            continue;
                        }
                        let mut triplet = [*u, *v, *w];
                        triplet.sort();
                        triplets.insert(triplet);
                    }
                }
            }

            let mut res = String::new();
            for triplet in triplets.iter() {
                write_nodes(&mut res, triplet.iter())?;
                res.push('\n');
            }

            println!("{}", res.trim_end());
            println!("{}", triplets.len());
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let mut graph = Graph::new();
            for (n1, n2) in output.iter() {
                graph::add_edge(&mut graph, *n1, *n2, 1);
                graph::add_edge(&mut graph, *n2, *n1, 1);
            }

            // get all cliques
            let mut cliques = BTreeSet::new();
            for v in graph.keys() {
                for (u, _) in graph::neighbors(&graph, v) {
                    let common_neighbors = graph.get(u).zip(graph.get(v)).into_iter().flat_map(
                        |(u_neighbors, v_neighbors)| u_neighbors.intersection(v_neighbors),
                    );
                    for (w, _) in common_neighbors {
                        if w == u || w == v {
                            continue;
                        }
                        let triplet = BTreeSet::from([(*u, 1usize), (*v, 1), (*w, 1)]);
                        cliques.insert(triplet);
                    }
                }
            }

            loop {
                let mut new_cliques = BTreeSet::new();

                for clique in cliques.iter() {
                    let candidates = clique
                        .iter()
                        .map(|(n, _)| *n)
                        .flat_map(|n| graph.get(&n))
                        .flat_map(|n| n.iter())
                        .map(|(n, _)| *n)
                        .collect::<BTreeSet<_>>();

                    for (c, cs) in candidates
                        .iter()
                        .filter_map(|n| graph.get(n).map(|nebs| (n, nebs)))
                    {
                        if cs.is_superset(clique) {
                            let mut new_clique = clique.clone();
                            new_clique.insert((*c, 1));
                            new_cliques.insert(new_clique);
                        }
                    }
                }

                if new_cliques.is_empty() {
                    break;
                }

                cliques = new_cliques;
            }

            let mut res = String::new();
            for clique in cliques.iter() {
                write_nodes(&mut res, clique.iter().map(|n| &n.0))?;
                res.push('\n');
            }

            println!("{}", res.trim_end());
            println!("{}", cliques.first().map_or(0, |c| c.len()));
            Ok(())
        }
    }
}
