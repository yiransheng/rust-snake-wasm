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