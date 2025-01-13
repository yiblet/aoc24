use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    iter,
};

pub type Graph<Node> = BTreeMap<Node, BTreeSet<(Node, usize)>>;
pub type AllPairShortestPaths<Node> = BTreeMap<Node, BTreeMap<Node, usize>>;
pub type ShortestPaths<Node> = BTreeMap<Node, usize>;

pub fn dijkstras<'a, T>(weighted_graph: &'a Graph<T>, start: &'a T) -> ShortestPaths<&'a T>
where
    T: PartialEq + Eq + Ord + PartialOrd,
{
    let mut visited = BTreeSet::new();
    let mut queue = BTreeSet::new();
    queue.insert((0usize, start));

    let mut distances = BTreeMap::new();
    distances.insert(start, 0usize);

    while let Some((dist, cur)) = queue.pop_first() {
        if visited.contains(&cur) {
            continue;
        }
        visited.insert(cur);
        for (next, weight) in weighted_graph.get(cur).into_iter().flatten() {
            let new_dist = dist + *weight;
            match distances.entry(next) {
                Entry::Occupied(mut dist) if dist.get() > &new_dist => {
                    dist.insert(new_dist);
                    queue.insert((new_dist, next));
                }
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(new_dist);
                    queue.insert((new_dist, next));
                }
            };
        }
    }

    distances
}

pub fn reachable<'a, Node: Ord>(graph: &'a Graph<Node>, start: &'a Node) -> BTreeSet<&'a Node> {
    let mut visited = BTreeSet::new();
    let mut stac = vec![start];

    while let Some(cur) = stac.pop() {
        if visited.contains(cur) {
            continue;
        }
        visited.insert(cur);
        stac.extend(graph.get(cur).iter().flat_map(|n| n.iter().map(|(n, _)| n)));
    }

    visited
}

pub fn is_fully_connected<Node: Ord>(graph: &Graph<Node>) -> bool {
    let nodes = nodes(graph);
    nodes.iter().all(|n| reachable(graph, n) == nodes)
}

pub fn nodes<Node: Ord>(graph: &Graph<Node>) -> BTreeSet<&Node> {
    graph
        .iter()
        .flat_map(|v| iter::once(v.0).chain(v.1.iter().map(|(n, _)| n)))
        .collect()
}

pub fn all_pairs_shortest_paths<Node: Ord + Copy>(
    graph: &Graph<Node>,
) -> AllPairShortestPaths<&Node> {
    let nodes = nodes(graph);

    nodes
        .into_iter()
        .map(|n| (n, dijkstras(graph, n)))
        .collect()
}

pub fn add_edge<Node: std::cmp::Ord + Copy>(
    graph: &mut Graph<Node>,
    n1: Node,
    n2: Node,
    weight: usize,
) -> bool {
    graph.entry(n1).or_default().insert((n2, weight))
}

pub fn remove_edge<Node: std::cmp::Ord + Copy>(
    graph: &mut Graph<Node>,
    n1: Node,
    n2: Node,
    weight: usize,
) -> bool {
    graph.entry(n1).or_default().remove(&(n2, weight))
}

pub fn reverse_graph<Node: std::cmp::Ord + Copy>(graph: &Graph<Node>) -> Graph<Node> {
    let mut res: Graph<Node> = Graph::new();
    for (n, v) in graph.iter() {
        for (n2, w) in v {
            add_edge(&mut res, *n2, *n, *w);
        }
    }
    res
}

pub fn rev_all_paths<Node: std::cmp::Ord + Copy>(
    graph: &Graph<Node>,
    distances: &BTreeMap<&Node, usize>,
    start: Node,
    end: Node,
) -> BTreeMap<Node, Vec<Node>> {
    let graph = reverse_graph(graph);
    let mut res = BTreeMap::<Node, Vec<Node>>::new();
    let mut stack: Vec<Node> = vec![end];

    while let Some(cur) = stack.pop() {
        if cur == start {
            res.entry(cur).or_default();
            continue;
        }

        let Some(min_dist) = neighbors(&graph, &cur)
            .filter_map(|n| distances.get(&n.0).copied().map(|x| x + n.1))
            .min()
        else {
            continue;
        };

        let min_dist_neighbors = neighbors(&graph, &cur)
            .filter(|n| distances.get(&n.0).copied().map(|x| x + n.1) == Some(min_dist))
            .collect::<Vec<_>>();

        for (n, _) in min_dist_neighbors {
            res.entry(cur).or_default().push(*n);
            stack.push(*n);
        }
    }
    res
}

pub fn all_paths<Node: std::cmp::Ord + Copy>(
    graph: &Graph<Node>,
    distances: &BTreeMap<&Node, usize>,
    start: Node,
    end: Node,
) -> BTreeMap<Node, Vec<Node>> {
    let paths = rev_all_paths(graph, distances, start, end);
    paths
        .into_iter()
        .flat_map(|(n, v)| v.into_iter().map(move |v| (n, v)))
        .fold(BTreeMap::<Node, Vec<Node>>::new(), |mut acc, (n, v)| {
            acc.entry(v).or_default().push(n);
            acc
        })
}

pub fn paths_to_vecs<N: Ord + Copy>(paths: &BTreeMap<N, Vec<N>>, start: N, end: N) -> Vec<Vec<N>> {
    let mut stack = vec![(start, vec![start])];
    let mut result = Vec::new();

    while let Some((current, path)) = stack.pop() {
        if current == end {
            result.push(path);
            continue;
        }

        if let Some(neighbors) = paths.get(&current) {
            for &neighbor in neighbors.iter() {
                let mut new_path = path.clone();
                new_path.push(neighbor);
                stack.push((neighbor, new_path));
            }
        }
    }

    result
}

pub fn neighbors<'a, Node: std::cmp::Ord + Copy>(
    graph: &'a Graph<Node>,
    n: &'a Node,
) -> impl Iterator<Item = (&'a Node, usize)> {
    graph
        .get(n)
        .into_iter()
        .flatten()
        .map(move |(n, w)| (n, *w))
}

pub fn toposort<Node: std::cmp::Ord + Copy>(graph: &Graph<Node>) -> Option<Vec<Node>> {
    let mut res: Vec<Node> = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();

    fn visit<Node: std::cmp::Ord + Copy>(
        node: &Node,
        graph: &Graph<Node>,
        visiting: &mut BTreeSet<Node>,
        visited: &mut BTreeSet<Node>,
        res: &mut Vec<Node>,
    ) -> Option<()> {
        if visiting.contains(node) {
            // Cycle detected
            return None;
        }

        if !visited.contains(node) {
            visiting.insert(*node);

            for (neighbor, _) in neighbors(graph, node) {
                visit(neighbor, graph, visiting, visited, res)?;
            }

            visiting.remove(node);
            visited.insert(*node);
            res.push(*node);
        }

        Some(())
    }

    for node in nodes(graph) {
        if !visited.contains(node) {
            visit(node, graph, &mut visiting, &mut visited, &mut res)?;
        }
    }

    res.reverse(); // Reverse to get the correct topological order
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toposort_test() {
        let mut graph = Graph::new();
        for i in 1..=20usize {
            add_edge(&mut graph, i, i - 1, 1);
            add_edge(&mut graph, i, i / 2, 1);
        }

        let soln: Vec<_> = (0..=20usize).rev().collect();
        assert_eq!(soln, toposort(&graph).unwrap());
    }
}
