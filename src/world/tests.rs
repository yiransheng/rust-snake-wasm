use rand::rngs::SmallRng;

use super::*;

use data::{Bounding, Wrapping};

#[test]
fn test_two_steps_inchworm() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let step_1 = indoc!(
        "
        ..........
        .ooooo....
        ..........
        ....*.....
        .........."
    );
    let step_2 = indoc!(
        "
        ..........
        ..oooo....
        ..........
        ....*.....
        .........."
    );

    let mut world: World<SmallRng, Wrapping> = World::from_ascii(snake_string);

    world.step(None).unwrap();

    assert_eq!(&step_1, &world.grid.to_string());

    world.step(None).unwrap();

    assert_eq!(&step_2, &world.grid.to_string());
}

#[test]
fn test_eat_food() {
    let snake_string = indoc!(
        "
        ..........
        .>>>v.....
        ..........
        ....*.....
        .........."
    );
    let digesting = indoc!(
        "
        ..........
        ..ooo.....
        ....o.....
        ....o.....
        .........."
    );

    let mut world: World<SmallRng, Wrapping> = World::from_ascii(snake_string);

    world.step(None).unwrap();
    world.step(None).unwrap();
    world.step(None).unwrap();

    assert_eq!(&digesting, &world.grid.to_string());

    let update = world.step(None).unwrap().unwrap();

    assert_matches!(update, WorldUpdate::SetBlock{ at: _, block: Block::Food });

    // erase food, as it's generated randomly, no assumptions on its position
    match update {
        WorldUpdate::SetBlock { at, .. } => {
            assert_eq!(world.grid[at], Block::Food);
        }
        _ => {}
    }
}

#[test]
fn test_wrapping() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let afterwards = indoc!(
        "
        ..........
        o......ooo
        ..........
        ....*.....
        .........."
    );

    let mut world: World<SmallRng, Wrapping> = World::from_ascii(snake_string);

    for _ in 0..12 {
        world.step(None).unwrap();
    }

    assert_eq!(&afterwards, &world.grid.to_string());
}

#[test]
fn test_bounding() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let afterwards = indoc!(
        "
        ..........
        ......oooo
        ..........
        ....*.....
        .........."
    );

    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    while let Ok(_) = world.step(None) {}

    assert_matches!(world.step(None), Err(UpdateError::OutOfBound));

    assert_eq!(&afterwards, &world.grid.to_string());
}

impl<R: Rng, BB: BoundingBehavior> World<R, BB> {
    fn head_dir(&self) -> Direction {
        self.get_block(self.head).snake().unwrap()
    }
    fn with_direction<F>(&mut self, dir: Direction, f: F)
    where
        F: Fn(Direction, Direction),
    {
        let prev_dir = self.head_dir();
        self.set_direction(dir).unwrap();
        let next_dir = self.head_dir();

        f(prev_dir, next_dir);

        let head = self.head;
        self.set_block(head, Block::Snake(prev_dir));
    }
}

#[test]
fn test_set_direction_one_block_snake() {
    let snake_string = indoc!(
        "
        ..........
        ....>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.with_direction(Direction::North, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::North);
    });

    world.with_direction(Direction::East, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::East);
    });

    world.with_direction(Direction::South, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::South);
    });

    world.with_direction(Direction::West, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::West);
    });
}

#[test]
fn test_set_direction_straight() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.with_direction(Direction::North, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::North);
    });

    world.with_direction(Direction::East, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::East);
    });

    world.with_direction(Direction::South, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::South);
    });

    world.with_direction(Direction::West, |prev_dir, next_dir| {
        assert_eq!(next_dir, prev_dir);
    });
}

#[test]
fn test_set_direction_turning() {
    let snake_string = indoc!(
        "
        ..........
        .>>>^.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.with_direction(Direction::North, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::North);
    });

    world.with_direction(Direction::East, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::East);
    });

    world.with_direction(Direction::South, |_prev_dir, next_dir| {
        assert_eq!(next_dir, Direction::South);
    });

    world.with_direction(Direction::West, |prev_dir, next_dir| {
        assert_eq!(next_dir, prev_dir);
    });
}

#[test]
fn test_set_direction_when_consuming_does_not_kill_it() {
    // gh issue: https://github.com/yiransheng/rust-snake-wasm/issues/11
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.step(None);
    world.step(Some(Direction::North));
    assert_matches!(world.step(Some(Direction::West)), Ok(_));
}

#[test]
fn test_death() {
    let snake_string = indoc!(
        "
        ..........
        >>>>>>v...
        ......v...
        ..^<<<<...
        .........*"
    );
    let afterwards = indoc!(
        "
        ..........
        .oooooo...
        ..o...o...
        ..ooooo...
        .........*"
    );

    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    while let Ok(_) = world.step(None) {}

    assert_matches!(world.step(None), Err(UpdateError::CollideBody));

    assert_eq!(&afterwards, &world.grid.to_string());
}
