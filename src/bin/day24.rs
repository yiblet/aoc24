use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write},
    io::Read,
    ops::RangeBounds,
};

use aoc24::{graph, parser};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Node([u8; 3]);

impl Node {
    fn position(&self) -> Option<u8> {
        if self.0[1..].iter().cloned().all(|c: u8| c.is_ascii_digit()) {
            Some(self.0[1..].iter().fold(0u8, |mut acc, c| {
                acc *= 10;
                acc += *c - b'0';
                acc
            }))
        } else {
            None
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node(\"{}\")", self.as_str().unwrap_or("???"))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or("???"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Op {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Operand {
    lhs: Node,
    op: Op,
    rhs: Node,
    result: Node,
}

impl Node {
    fn new(name: [u8; 3]) -> Self {
        Self(name)
    }

    fn from_prefix(prefix: char, value: u8) -> Self {
        Self([prefix as u8, b'0' + value / 10, b'0' + value % 10])
    }

    fn starts_with(&self, prefix: u8) -> bool {
        self.0[0] == prefix
    }

    fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        let res = std::str::from_utf8(&self.0)?;
        Ok(res)
    }
}

impl Operand {
    fn pins(&self) -> impl Iterator<Item = Node> {
        [self.lhs, self.rhs].into_iter()
    }
}

fn take_node<'a>() -> impl Fn(&'a str) -> Option<(Node, &'a str)> {
    move |input: &str| {
        if input.len() < 3 {
            return None;
        }
        let mut res: [u8; 3] = [0; 3];
        let node = &input[0..3];
        res.copy_from_slice(node.as_bytes());
        if !res.iter().all(|c| (*c as char).is_ascii_alphanumeric()) {
            return None;
        }

        Some((Node::new(res), &input[3..]))
    }
}

fn take_initial_node<'a>() -> impl Fn(&'a str) -> Option<((Node, bool), &'a str)> {
    move |input: &str| {
        let (node, rest) = take_node()(input)?;
        let (_, rest) = parser::take_str(": ")(rest)?;
        let (value, rest) = parser::take_int()(rest)?;
        Some(((node, value != 0), rest))
    }
}

fn take_op<'a>() -> impl Fn(&'a str) -> Option<(Op, &'a str)> {
    move |input: &str| {
        let (op, rest) = parser::take_or3(
            parser::map(parser::take_str("AND"), |_| Op::And),
            parser::map(parser::take_str("OR"), |_| Op::Or),
            parser::map(parser::take_str("XOR"), |_| Op::Xor),
        )(input)?;

        Some((op, rest))
    }
}

fn take_final_node<'a>() -> impl Fn(&'a str) -> Option<(Operand, &'a str)> {
    move |input: &str| {
        let (node1, rest) = take_node()(input)?;
        let (_, rest) = parser::take_str(" ")(rest)?;
        let (op, rest) = take_op()(rest)?;
        let (_, rest) = parser::take_str(" ")(rest)?;
        let (node2, rest) = take_node()(rest)?;
        let (_, rest) = parser::take_str(" -> ")(rest)?;
        let (node3, rest) = take_node()(rest)?;
        Some((
            Operand {
                lhs: node1,
                op,
                rhs: node2,
                result: node3,
            },
            rest,
        ))
    }
}

#[derive(Debug)]
struct Circuit {
    nodes: BTreeMap<Node, usize>,
    starts: BTreeMap<Node, bool>,
    ops: BTreeMap<Node, Operand>,
}

impl Circuit {
    fn new(starts: Vec<(Node, bool)>, ops: Vec<Operand>) -> Self {
        let starts = starts.into_iter().collect::<BTreeMap<_, _>>();
        let ops = ops
            .into_iter()
            .map(|op| (op.result, op))
            .collect::<BTreeMap<_, _>>();

        let nodes: BTreeSet<Node> = {
            let keys = starts.keys().cloned();
            let ops = ops.values().flat_map(|op| [op.lhs, op.rhs, op.result]);
            keys.chain(ops).collect()
        };
        let nodes = nodes.into_iter().enumerate().map(|v| (v.1, v.0)).collect();
        Self { starts, ops, nodes }
    }

    fn idx(&self, node: &Node) -> Option<usize> {
        self.nodes.get(node).cloned()
    }

    fn gates(&self) -> BTreeSet<Node> {
        self.ops.keys().cloned().collect()
    }

    fn ancestors(&self, node: Node, swaps: &[(Node, Node)]) -> BTreeSet<Node> {
        self.ancestors_of(&[node], swaps)
    }

    fn get_ops(&self, node: Node, swaps: &[(Node, Node)]) -> Option<Operand> {
        let swap_node = use_swap(&node, swaps);
        self.ops.get(&swap_node).cloned().map(|mut op| {
            op.result = node;
            op
        })
    }

    fn ancestors_of(&self, nodes: &[Node], swaps: &[(Node, Node)]) -> BTreeSet<Node> {
        let mut res = BTreeSet::new();
        let mut stack = nodes.to_vec();
        while let Some(cur) = stack.pop() {
            if res.contains(&cur) {
                continue;
            }
            res.insert(cur);

            if let Some(x) = self.get_ops(cur, swaps) {
                stack.push(x.lhs);
                stack.push(x.rhs);
            }
        }

        res
    }

    fn ancestors_range<R: RangeBounds<Node>>(
        &self,
        range: R,
        swaps: &[(Node, Node)],
    ) -> BTreeSet<Node> {
        let nodes_in_range = self
            .nodes
            .range(range)
            .map(|v| v.0)
            .cloned()
            .collect::<Vec<_>>();
        self.ancestors_of(&nodes_in_range, swaps)
    }

    fn max_z_position(&self) -> Option<u8> {
        self.nodes
            .range(Node::new([b'z', b'0', b'0'])..=Node::new([b'z', b'9', b'9']))
            .map(|v| v.0)
            .last()
            .and_then(|node| node.position())
    }

    fn evaluate(&self) -> Option<Evaluate<'_>> {
        Evaluate::new(self, &[])
    }

    fn evaluate_swapped<'a>(&'a self, swapped: &'a [(Node, Node)]) -> Option<EvaluateV2<'a>> {
        EvaluateV2::new(self, swapped)
    }
}

#[derive(Debug)]
struct Evaluate<'a> {
    parsed: &'a Circuit,
    swaps: &'a [(Node, Node)],
    sorted: Vec<Node>,
}

fn use_swap(result: &Node, swaps: &[(Node, Node)]) -> Node {
    swaps
        .iter()
        .find_map(|x| {
            if x.0 == *result {
                Some(x.1)
            } else if x.1 == *result {
                Some(x.0)
            } else {
                None
            }
        })
        .unwrap_or(*result)
}

impl<'a> Evaluate<'a> {
    fn new(parsed: &'a Circuit, swapped: &'a [(Node, Node)]) -> Option<Self> {
        let mut graph = graph::Graph::new();

        for op in parsed.ops.values() {
            let result = use_swap(&op.result, swapped);
            graph::add_edge(&mut graph, op.lhs, result, 1);
            graph::add_edge(&mut graph, op.rhs, result, 1);
        }

        let sorted = graph::toposort(&graph)?;
        Some(Self {
            parsed,
            swaps: swapped,
            sorted,
        })
    }

    fn exec(&self, x: u64, y: u64) -> anyhow::Result<u64> {
        let mut values = self.parsed.starts.clone();
        set_input(&mut values, b'x', x);
        set_input(&mut values, b'y', y);

        self.run_operations(&mut values)?;

        Ok(solution(&values))
    }

    fn exec_default(&self) -> anyhow::Result<u64> {
        let mut values = self.parsed.starts.clone();
        self.run_operations(&mut values)?;
        Ok(solution(&values))
    }

    fn get_op(&self, node: &Node) -> Option<Operand> {
        self.parsed.get_ops(*node, self.swaps)
    }

    fn run_operations(&self, values: &mut BTreeMap<Node, bool>) -> anyhow::Result<()> {
        for node in self.sorted.iter() {
            let Some(op) = self.get_op(node) else {
                continue;
            };

            let Some(lhs) = values.get(&op.lhs) else {
                return Err(anyhow::anyhow!(
                    "could not find lhs: {:?} - toposort is likely wrong",
                    op
                ));
            };

            let Some(rhs) = values.get(&op.rhs) else {
                return Err(anyhow::anyhow!(
                    "could not find rhs: {:?} - toposort is likely wrong",
                    op
                ));
            };

            let res = match op.op {
                Op::And => *lhs && *rhs,
                Op::Or => *lhs || *rhs,
                Op::Xor => *lhs ^ *rhs,
            };

            values.insert(op.result, res);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct EvaluateV2<'a> {
    parsed: &'a Circuit,
    values: Vec<bool>,
    ops: Vec<(usize, Op, usize)>,
}

fn create_ops(parsed: &Circuit, swapped: &[(Node, Node)]) -> Option<Vec<(usize, Op, usize)>> {
    fn create_ops_recur(
        parsed: &Circuit,
        swapped: &[(Node, Node)],
        visited: &mut [bool],
        visiting: &mut [bool],
        ops: &mut Vec<(usize, Op, usize)>,
        cur: Node,
    ) -> Option<()> {
        let idx = parsed.idx(&cur)?;
        if visited[idx] {
            return Some(());
        }
        if visiting[idx] {
            return None;
        }
        visiting[idx] = true;

        if let Some(op) = parsed.get_ops(cur, swapped) {
            create_ops_recur(parsed, swapped, visited, visiting, ops, op.lhs)?;
            create_ops_recur(parsed, swapped, visited, visiting, ops, op.rhs)?;
            ops.push((parsed.idx(&op.lhs)?, op.op, idx));
            ops.push((parsed.idx(&op.rhs)?, op.op, idx));
        }

        visiting[idx] = false;
        visited[idx] = true;
        Some(())
    }

    let mut ops = vec![];
    let mut visited = vec![false; parsed.nodes.len()];
    let mut visiting = vec![false; parsed.nodes.len()];

    for node in parsed.nodes.keys().cloned() {
        create_ops_recur(parsed, swapped, &mut visited, &mut visiting, &mut ops, node)?;
    }
    Some(ops)
}

impl<'a> EvaluateV2<'a> {
    // FIXME: ops needs to be in topological order.
    fn new(parsed: &'a Circuit, swapped: &'a [(Node, Node)]) -> Option<Self> {
        let ops = create_ops(parsed, swapped)?;
        let mut start = vec![false; parsed.nodes.len()];
        for (n, v) in parsed.starts.iter() {
            let idx = parsed.idx(n)?;
            start[idx] = *v;
        }
        for n in parsed.ops.keys().cloned() {
            let op = parsed.get_ops(n, swapped)?;
            start[parsed.idx(&n)?] = matches!(op.op, Op::And);
        }

        Some(Self {
            parsed,
            ops,
            values: start,
        })
    }

    fn set_input(&self, values: &mut [bool], prefix: u8, input: u64) {
        for (z, idx) in self
            .parsed
            .nodes
            .range(Node::new([prefix, b'0', b'0'])..=Node::new([prefix, b'9', b'9']))
        {
            // SAFETY: we know that the node is a valid position since the range above guarantees
            // validity
            let position = z.position().unwrap();
            values[*idx] = input & (1u64 << position) != 0;
        }
    }

    fn solution(&self, values: &[bool]) -> u64 {
        let mut res = 0u64;
        for (z, idx) in self
            .parsed
            .nodes
            .range(Node::new([b'z', b'0', b'0'])..=Node::new([b'z', b'9', b'9']))
        {
            let val = values[*idx];
            if !val {
                continue;
            }
            // SAFETY: we know that the node is a valid position since the range above
            // guarantees that
            let position = z.position().unwrap();
            res |= 1u64 << position;
        }

        res
    }

    fn exec(&self, x: u64, y: u64) -> anyhow::Result<u64> {
        let mut values = self.values.clone();
        self.set_input(&mut values, b'x', x);
        self.set_input(&mut values, b'y', y);
        self.run_operations(&mut values);
        Ok(self.solution(&values))
    }

    fn exec_default(&self) -> anyhow::Result<u64> {
        let mut values = self.values.clone();
        self.run_operations(&mut values);
        Ok(self.solution(&values))
    }

    fn run_operations(&self, values: &mut [bool]) {
        for (src, op, dest) in self.ops.iter() {
            let src = values[*src];
            let dest = &mut values[*dest];
            match op {
                Op::And => *dest &= src,
                Op::Or => *dest |= src,
                Op::Xor => *dest ^= src,
            }
        }
    }
}

fn solution(values: &BTreeMap<Node, bool>) -> u64 {
    let mut res = 0u64;
    for (z, val) in values.range(Node::new([b'z', b'0', b'0'])..=Node::new([b'z', b'9', b'9'])) {
        if !*val {
            continue;
        }
        // SAFETY: we know that the node is a valid position since the range above
        // guarantees that
        let position = z.position().unwrap();
        res |= 1u64 << position;
    }

    res
}

fn set_input(values: &mut BTreeMap<Node, bool>, prefix: u8, input: u64) {
    for (z, val) in
        values.range_mut(Node::new([prefix, b'0', b'0'])..=Node::new([prefix, b'9', b'9']))
    {
        // SAFETY: we know that the node is a valid position since the range above guarantees
        // validity
        let position = z.position().unwrap();
        *val = input & (1u64 << position) != 0;
    }
}

fn create_tests(z: u8) -> impl Iterator<Item = [u64; 2]> {
    let tests = [
        [1 << z, 1 << z],             // test that summing sets the bit to zero
        [1 << z, 0],                  // test that summing with 0 is zero
        [0, 0],                       // test that 0+0 is zero
        [1 << (z - 1), 1 << (z - 1)], // test that the carry works
    ];

    tests.into_iter().chain(tests.into_iter().map(|mut x| {
        x.reverse();
        x
    }))
}

fn validate<F>(max_bit: u8, eval: &F) -> bool
where
    F: Fn(u64, u64) -> anyhow::Result<u64>,
{
    (0..=max_bit).all(|bit| validate_bit(bit, eval))
}

fn validate_bit<F>(bit: u8, eval: &F) -> bool
where
    F: Fn(u64, u64) -> anyhow::Result<u64>,
{
    create_tests(bit).all(|test| {
        let Ok(partial_res) = eval(test[0], test[1]) else {
            return false;
        };
        let mask: u64 = (1u64 << (bit + 1)).wrapping_sub(1);
        let expected = (test[1] + test[0]) & mask;
        let res = partial_res & mask;
        expected == res
    })
}

fn solve(circuit: &Circuit) -> anyhow::Result<Vec<(Node, Node)>> {
    let candidates = circuit.gates();
    println!("gates = {:?}", candidates.len());

    let mut swaps = Vec::new();
    let max_z = circuit
        .max_z_position()
        .ok_or_else(|| anyhow::anyhow!("could not find max z position: circuit is not valid"))?;

    for z in 0..max_z {
        if swaps.len() > 4 {
            anyhow::bail!("reached max swaps")
        }

        println!(
            "trying z{:02} with swaps = {:?} remaining candidates {}",
            z,
            swaps,
            candidates.len()
        );

        let Some(eval) = circuit.evaluate_swapped(&swaps) else {
            continue;
        };
        if validate(z, &|x, y| eval.exec(x, y)) {
            continue;
        }

        // we will have to try finding a swap that fixes this bit
        match find_swaps(circuit, &swaps, &candidates, z) {
            None => anyhow::bail!("could not find a solution for z{:02}", z),
            Some(new_swaps) => swaps = new_swaps,
        };
    }

    Ok(swaps)
}

fn swaps_iter(swaps: &[(Node, Node)]) -> impl Iterator<Item = Node> + '_ {
    swaps.iter().flat_map(|nodes| [nodes.0, nodes.1])
}

fn valid_swap_candidate(swaps: &[(Node, Node)], swap: (Node, Node)) -> bool {
    if swap.0 >= swap.1 {
        return false;
    }

    swaps_iter(swaps).all(|existing_swap| existing_swap != swap.0)
        && swaps_iter(swaps).all(|existing_swap| existing_swap != swap.1)
}

fn find_swaps(
    circuit: &Circuit,
    swaps: &[(Node, Node)],
    candidates: &BTreeSet<Node>,
    z: u8,
) -> Option<Vec<(Node, Node)>> {
    let mut swaps = swaps.to_vec();

    let z_node = Node::from_prefix('z', z);
    let ancestors = circuit.ancestors_range(..=z_node, &swaps);
    for n1 in ancestors.iter().cloned() {
        for n2 in candidates.iter().cloned() {
            let swap = (n1, n2);
            if !valid_swap_candidate(&swaps, swap) {
                continue;
            }
            swaps.push(swap);
            if let Some(eval) = circuit.evaluate_swapped(&swaps) {
                if validate(z, &|x, y| eval.exec(x, y)) {
                    return Some(swaps);
                }
            };
            swaps.pop();
        }
    }
    None
}

fn parse_file(filename: &str) -> anyhow::Result<Circuit> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let (starts, rest) =
        parser::take_separator(take_initial_node(), parser::take_newline())(&string)
            .ok_or_else(|| anyhow::anyhow!("could not parse file"))?;

    let (_, rest) = parser::take_newline()(rest)
        .ok_or_else(|| anyhow::anyhow!("could not parse file, remaining: {rest}"))?;
    let (ops, rest) = parser::take_separator(take_final_node(), parser::take_newline())(rest)
        .ok_or_else(|| anyhow::anyhow!("could not parse file"))?;

    if !rest.is_empty() {
        Err(anyhow::anyhow!("could not parse file, remaining: {rest}"))?;
    }

    Ok(Circuit::new(starts, ops))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let eval = output
                .evaluate()
                .ok_or_else(|| anyhow::anyhow!("cycle detected"))?
                .exec_default()?;
            println!("{}", eval);
            Ok(())
        }

        Args::Part2 { file } => {
            let circuit = parse_file(&file)?;
            let nodes = solve(&circuit)?
                .into_iter()
                .flat_map(|x| [x.0, x.1])
                .collect::<BTreeSet<_>>();

            let mut s = String::new();
            let mut first = true;
            for node in nodes.iter() {
                if !first {
                    s.push(',');
                }
                let _ = write!(s, "{}", node);
                first = false;
            }

            println!("{}", s);
            Ok(())
        }
    }
}
