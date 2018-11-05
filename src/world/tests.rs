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

#[test]
fn test_set_direction_when_eaten() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Wrapping> = World::from_ascii(snake_string);

    assert_matches!(world.state, SnakeState::Eaten);

    let current_dir = world.get_head_dir().unwrap();
    let allowed_dir_1 = current_dir.turn_left();
    let allowed_dir_2 = current_dir.turn_right();
    let not_allowed_dir = current_dir.opposite();

    let next_dir_1 = _set_direction_when_eaten(&mut world, allowed_dir_1);

    _set_direction_when_eaten(&mut world, current_dir); // restore
    let next_dir_2 = _set_direction_when_eaten(&mut world, allowed_dir_2);

    _set_direction_when_eaten(&mut world, current_dir); // restore
    let next_dir_3 = _set_direction_when_eaten(&mut world, not_allowed_dir);

    assert_eq!(allowed_dir_1, next_dir_1);
    assert_eq!(allowed_dir_2, next_dir_2);
    assert_eq!(current_dir, next_dir_3);
}
fn _set_direction_when_eaten(
    world: &mut World<SmallRng, Wrapping>,
    dir: Direction,
) -> Direction {
    while SnakeState::Eaten != world.state {
        world.step(None);
    }
    world.set_direction(dir).unwrap();
    world.get_head_dir().unwrap()
}

#[test]
fn test_set_direction_when_consuming() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.step(None).unwrap();
    assert_matches!(world.state, SnakeState::Consuming(_, _));

    let current_dir = world.get_head_dir().unwrap();
    world.set_direction(current_dir.turn_left()).unwrap();
    let next_dir = world.get_head_dir().unwrap();

    assert_eq!(current_dir, next_dir);
}

#[test]
fn test_set_direction_when_turning() {
    let snake_string = indoc!(
        "
        ..........
        .>>>>.....
        ..........
        ....*.....
        .........."
    );
    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    world.step(None).unwrap();

    let dir = world.get_head_dir().map(Direction::turn_left).ok();
    world.step(dir).unwrap();

    assert_matches!(world.state, SnakeState::Turning(_));

    let current_dir = world.get_head_dir().unwrap();
    world.set_direction(current_dir.turn_right()).unwrap();

    assert_matches!(world.state, SnakeState::Eaten);
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
