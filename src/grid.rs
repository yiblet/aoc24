pub type Grid<V> = Vec<Vec<V>>;

pub fn get_at<V>(grid: &Vec<Vec<V>>, pos: (isize, isize)) -> Option<&V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get(row as usize)?.get(col as usize)
}

pub fn get_at_mut<V>(grid: &mut Vec<Vec<V>>, pos: (isize, isize)) -> Option<&mut V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get_mut(row as usize)?.get_mut(col as usize)
}

pub fn copy_with<V, V2: Default>(grid: &Vec<Vec<V>>) -> Vec<Vec<V2>> {
    grid.iter()
        .map(|v| v.iter().map(|_| V2::default()).collect())
        .collect()
}

pub fn vec_add(v: (isize, isize), v2: (isize, isize)) -> (isize, isize) {
    (v.0 + v2.0, v.1 + v2.1)
}

pub fn vec_sub(v: (isize, isize), v2: (isize, isize)) -> (isize, isize) {
    (v.0 - v2.0, v.1 - v2.1)
}

pub fn scale(v: (isize, isize), scale: isize) -> (isize, isize) {
    (v.0 * scale, v.1 * scale)
}

pub fn reduce_vec(v: (isize, isize)) -> (isize, isize) {
    let (a, b) = v;

    // Compute the GCD (greatest common divisor)
    fn gcd(mut x: isize, mut y: isize) -> isize {
        while y != 0 {
            let temp = y;
            y = x % y;
            x = temp;
        }
        x
    }

    let divisor = gcd(a.abs(), b.abs());
    (a / divisor, b / divisor) // Reduce both numerator and denominator
}

pub fn iter_pos<V>(grid: &Vec<Vec<V>>) -> impl Iterator<Item = (isize, isize, &V)> + '_ {
    grid.iter().enumerate().flat_map(|(row, v)| {
        v.iter()
            .enumerate()
            .map(move |(col, v)| (row as isize, col as isize, v))
    })
}
