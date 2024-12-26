use crate::util;

pub type Grid<V> = Vec<Vec<V>>;

pub fn parse_grid(filename: &str) -> anyhow::Result<Grid<char>> {
    let mut lines = util::read_file_lines(filename)?;

    let grid: Grid<char> = lines.by_ref().map(|line| line.chars().collect()).collect();
    lines.error()?;

    Ok(grid)
}

pub fn neighbors<V>(
    grid: &Grid<V>,
    pos: (isize, isize),
) -> impl Iterator<Item = ((isize, isize), &V)> + '_ {
    Direction::all_directions()
        .into_iter()
        .map(move |dir| dir.apply(pos))
        .filter_map(|(row, col)| get_at(grid, (row, col)).map(|v| ((row, col), v)))
}

pub fn map<V, V2, F>(grid: &Grid<V>, mut f: F) -> Grid<V2>
where
    F: FnMut((isize, isize), &V) -> V2,
{
    grid.iter()
        .enumerate()
        .map(|(row, v)| {
            v.iter()
                .enumerate()
                .map(|(col, v)| f((row as isize, col as isize), v))
                .collect()
        })
        .collect()
}

pub fn map_result<V, V2, E, F>(grid: &Grid<V>, mut f: F) -> Result<Grid<V2>, E>
where
    F: FnMut((isize, isize), &V) -> Result<V2, E>,
{
    grid.iter()
        .enumerate()
        .map(|(row, v)| {
            v.iter()
                .enumerate()
                .map(|(col, v)| f((row as isize, col as isize), v))
                .collect()
        })
        .collect()
}

pub fn get_at<V>(grid: &Grid<V>, pos: (isize, isize)) -> Option<&V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get(row as usize)?.get(col as usize)
}

pub fn get_at_mut<V>(grid: &mut Grid<V>, pos: (isize, isize)) -> Option<&mut V> {
    let (row, col) = pos;
    if row < 0 || col < 0 {
        return None;
    }
    grid.get_mut(row as usize)?.get_mut(col as usize)
}

pub fn copy_default<V, V2: Default>(grid: &Grid<V>) -> Vec<Vec<V2>> {
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

pub fn iter_pos<V>(grid: &Grid<V>) -> impl Iterator<Item = ((isize, isize), &V)> + '_ {
    grid.iter().enumerate().flat_map(|(row, v)| {
        v.iter()
            .enumerate()
            .map(move |(col, v)| ((row as isize, col as isize), v))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up = 1 << 0,
    Down = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
}

impl Direction {
    pub fn all_directions() -> [Self; 4] {
        [Self::Up, Self::Down, Self::Left, Self::Right]
    }

    pub fn all_diagonals() -> [[Self; 2]; 4] {
        [
            [Self::Up, Self::Left],
            [Self::Up, Self::Right],
            [Self::Down, Self::Left],
            [Self::Down, Self::Right],
        ]
    }

    pub fn horizontal(self) -> [Self; 2] {
        [Self::Up, Self::Down]
    }

    pub fn vertical(self) -> [Self; 2] {
        [Self::Left, Self::Right]
    }

    pub fn is_horizontal(self) -> bool {
        self.horizontal().contains(&self)
    }

    pub fn is_vertical(self) -> bool {
        self.vertical().contains(&self)
    }

    pub fn adjacent(self) -> [Self; 2] {
        if self.is_horizontal() {
            self.vertical()
        } else {
            self.horizontal()
        }
    }

    pub fn invert(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub fn rotate_90_right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }

    pub fn apply(self, cur: (isize, isize)) -> (isize, isize) {
        let delta = match self {
            Self::Up => (-1, 0),
            Self::Right => (0, 1),
            Self::Left => (0, -1),
            Self::Down => (1, 0),
        };

        (cur.0 + delta.0, cur.1 + delta.1)
    }

    pub fn apply_inverse(self, cur: (isize, isize)) -> (isize, isize) {
        self.invert().apply(cur)
    }
}
