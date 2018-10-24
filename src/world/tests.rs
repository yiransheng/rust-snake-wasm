mod test_utils {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use std::fmt;

    impl From<char> for Block {
        fn from(c: char) -> Block {
            match c {
                'o' => Block::empty(),
                '*' => Block::food(),
                '>' => Block::from(Direction::East),
                '<' => Block::from(Direction::West),
                'v' => Block::from(Direction::South),
                '^' => Block::from(Direction::North),
                _ => Block::out_of_bound(),
            }
        }
    }

    impl fmt::Display for Block {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let tile = (*self).into();
            match tile {
                Tile::Empty => '.'.fmt(f),
                Tile::Snake => 'o'.fmt(f),
                Tile::Food => '*'.fmt(f),
                Tile::OutOfBound => "".fmt(f),
            }
        }
    }

    impl World<SmallRng> {
        #[allow(dead_code)]
        pub fn from_ascii(s: &str) -> Self {
            let height = s.lines().count() as u32;
            let width = s.lines().next().unwrap().chars().count() as u32;

            let mut grid = Grid::empty(width, height);

            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    grid.set_block(Coordinate::from_usize(x, y), c.into());
                }
            }

            let mut head = Coordinate { x: 0, y: 0 };
            let mut tail = Coordinate { x: 0, y: 0 };

            for x in 0..grid.width {
                for y in 0..grid.height {
                    let coord = Coordinate { x, y };
                    let b = grid.get_block(coord);
                    if b.is_snake() {
                        if grid.get_prev_snake_block(coord).is_none() {
                            tail = coord;
                        }
                        if grid.get_next_snake_block(coord).is_none() {
                            head = coord;
                        }
                    }
                }
            }

            let initial_snake: Vec<(Coordinate, Direction)>;

            {
                let iter = SnakeIter::new(&grid, tail);

                initial_snake = iter.collect();
            }

            World {
                grid,
                head,
                tail,
                // reading from ascii can skip initial setup
                rng: SmallRng::from_seed([0; 16]),

                initial_snake,
            }
        }
    }

    impl<R> fmt::Display for World<R> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for y in 0..self.grid.height {
                for x in 0..self.grid.width {
                    let coord = Coordinate { x, y };
                    write!(f, "{}", self.grid.get_block(coord))?;
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;
    use rand::rngs::SmallRng;
    use std::marker::PhantomData;

    struct MockSink<T> {
        _marker: PhantomData<T>,
    }

    impl<T> RenderSink<T> for MockSink<T> {
        fn is_ready(&self) -> bool {
            true
        }
        fn push(&mut self, x: RenderUnit<T>) {}
    }

    #[test]
    fn test_game() {
        let world = "oooooooooo
oooooooooo
o>>>>ooooo
oooooooooo
oooo*ooooo
oooooooooo";
        let mut world = World::from_ascii(world);
        let mut q = MockSink {
            _marker: PhantomData,
        };

        {
            let mut update = |n: usize, dir: Direction| {
                for _ in 0..n {
                    world.step_update(&mut q);
                }
                world.set_direction(dir);
            };

            // east 3
            // turn south
            update(3, Direction::South);
            // south 2
            // turn west
            update(2, Direction::West);
            // west 3
            update(3, Direction::West);
        }

        let final_state = "..........
..........
..........
.......o..
....oooo..
..........
";

        // foot eaten
        // replace food (*) with empty (.) before states comparision
        assert_eq!(final_state, &world.to_string().replace('*', "."));
    }

    #[test]
    fn test_builder() {
        let world = WorldBuilder::new()
            .width(10)
            .height(5)
            .set_snake(1, 1)
            .extend(Direction::East)
            .extend(Direction::East)
            .extend(Direction::East)
            .extend(Direction::East)
            .build_with_seed::<SmallRng>([123; 16]);
        let final_state = "..........
.oooo.....
..........
..........
..........
";

        assert_eq!(final_state, &world.to_string());
    }
}
