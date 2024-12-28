use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    fmt,
};

pub fn dijkstras<'a, T>(
    weighted_graph: &'a BTreeMap<T, Vec<(T, usize)>>,
    start: &'a T,
) -> BTreeMap<&'a T, usize>
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

pub fn reverse_graph<Node: std::cmp::Ord + Copy>(
    graph: &BTreeMap<Node, Vec<(Node, usize)>>,
) -> BTreeMap<Node, Vec<(Node, usize)>> {
    let mut res: BTreeMap<Node, Vec<(Node, usize)>> = BTreeMap::new();
    for (n, v) in graph.iter() {
        for (n2, w) in v {
            res.entry(*n2).or_default().push((*n, *w));
        }
    }
    res
}

pub fn all_paths<Node: std::cmp::Ord + Copy + fmt::Debug>(
    graph: &BTreeMap<Node, Vec<(Node, usize)>>,
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

        let Some(min_dist) = neighbors(&graph, cur)
            .filter_map(|n| distances.get(&n.0).copied().map(|x| x + n.1))
            .min()
        else {
            continue;
        };

        let min_dist_neighbors = neighbors(&graph, cur)
            .filter(|n| distances.get(&n.0).copied().map(|x| x + n.1) == Some(min_dist))
            .collect::<Vec<_>>();

        for (n, _) in min_dist_neighbors {
            res.entry(cur).or_default().push(n);
            stack.push(n);
        }
    }

    res
}

fn neighbors<Node: std::cmp::Ord + Copy>(
    graph: &BTreeMap<Node, Vec<(Node, usize)>>,
    end: Node,
) -> impl Iterator<Item = (Node, usize)> + '_ {
    graph.get(&end).into_iter().flatten().cloned()
}
