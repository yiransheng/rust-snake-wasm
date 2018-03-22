use std::ops;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}
impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            &Direction::North => Direction::South,
            &Direction::South => Direction::North,
            &Direction::East => Direction::West,
            &Direction::West => Direction::East,
        }
    }
}
impl From<i32> for Direction {
    fn from(d: i32) -> Direction {
        match d {
            1 => Direction::North,
            2 => Direction::South,
            3 => Direction::East,
            4 => Direction::West,
            _ => panic!("Unknow direction"),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct Dimension(pub u32, pub u32);

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    #[inline]
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Mul<i32> for Point {
    type Output = Point;

    #[inline]
    fn mul(self, rhs: i32) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl ops::Mul<u32> for Point {
    type Output = Point;

    #[inline]
    fn mul(self, rhs: u32) -> Point {
        self.mul(rhs as i32)
    }
}
impl ::std::convert::From<Direction> for Point {
    fn from(d: Direction) -> Point {
        match d {
            Direction::North => Point { x: 0, y: -1 },
            Direction::South => Point { x: 0, y: 1 },
            Direction::East => Point { x: 1, y: 0 },
            Direction::West => Point { x: -1, y: 0 },
        }
    }
}

impl Dimension {
    pub fn size(&self) -> usize {
        (self.0 * self.1) as usize
    }
    pub fn calc_index(&self, pt: Point) -> Option<usize> {
        let w = self.0 as i32;
        let h = self.1 as i32;

        if !(pt.x >= 0 && pt.x < w && pt.y >= 0 && pt.y < h) {
            None
        } else {
            let index = pt.x + (pt.y * w);
            Some(index as usize)
        }
    }
}

pub struct Vec2d<T> {
    dimension: Dimension,
    storage: Vec<T>,
}

impl<T> Vec2d<T> {
    fn of_dimension(dimension: Dimension) -> Self {
        Vec2d {
            dimension,
            storage: Vec::with_capacity(dimension.size()),
        }
    }
}
impl<T: Copy + Default> Vec2d<T> {
    pub fn new(dimension: Dimension) -> Self {
        let mut v = Self::of_dimension(dimension);
        v.fill(T::default());
        v
    }
    pub fn fill(&mut self, x: T) {
        for _ in 0..self.dimension.size() {
            self.storage.push(x);
        }
    }
    pub fn get(&self, pt: Point) -> Option<T> {
        self.dimension
            .calc_index(pt)
            .map(|index| self.storage[index])
    }
    pub fn set(&mut self, pt: Point, x: T) -> Option<T> {
        if let Some(index) = self.dimension.calc_index(pt) {
            Some(::std::mem::replace(&mut self.storage[index], x))
        } else {
            None
        }
    }
}
impl<T: Copy> ops::Index<Point> for Vec2d<T> {
    type Output = T;

    fn index(&self, pt: Point) -> &T {
        let index = self.dimension.calc_index(pt).unwrap();
        &self.storage[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_direction() {
        let n = Direction::North;
        let s = Direction::South;
        let w = Direction::West;
        let e = Direction::East;

        assert_eq!(Point::from(n.opposite()), Point::from(s));
        assert_eq!(Point::from(w.opposite()), Point::from(e));
    }

    #[test]
    fn test_vec2d() {
        let mut world: Vec2d<Option<Direction>> = Vec2d::new(Dimension(2, 3));

        assert_eq!(world.get(Point::new(0, 0)), Some(None));
        assert_eq!(world.get(Point::new(2, 3)), None);
        assert_eq!(
            world.set(Point::new(1, 1), Some(Direction::North)),
            Some(None)
        );
        assert_eq!(world.get(Point::new(1, 1)), Some(Some(Direction::North)));
    }
}
