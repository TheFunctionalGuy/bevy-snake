/// I like CREQ
use std::collections::HashSet;

use bevy::prelude::*;

use bevy::core::FixedTimestep;
use rand::seq::IteratorRandom;

// Constants
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

// Resources
#[derive(Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Deref, DerefMut)]
struct LastDirection(Direction);

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Default, Deref, DerefMut)]
struct FreePositionsTemplate(HashSet<Position>);

// Events
struct GrowthEvent;

struct GameOverEvent;

// Components
#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeSegment;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component, PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Left => Self::Right,
            Direction::Up => Self::Down,
            Direction::Right => Self::Left,
            Direction::Down => Self::Up,
        }
    }
}

#[derive(Component)]
struct Food;

// Systems
fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_free_position_template(mut free_position_template: ResMut<FreePositionsTemplate>) {
    for x in 0..ARENA_WIDTH {
        for y in 0..ARENA_HEIGHT {
            free_position_template.insert(Position {
                x: x as i32,
                y: y as i32,
            });
        }
    }
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead)
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .insert(Direction::Up)
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn snake_movement(
    mut last_direction: ResMut<LastDirection>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    mut head: Query<(Entity, &Direction)>,
    mut positions: Query<&mut Position, With<SnakeSegment>>,
) {
    let old_positions: Vec<Position> = positions.iter().copied().collect();

    // Move head
    let (head_entity, head_direction) = head.single_mut();
    let mut head_pos = positions.get_mut(head_entity).unwrap();

    match head_direction {
        Direction::Left => {
            head_pos.x -= 1;
            **last_direction = Direction::Left;
        }
        Direction::Up => {
            head_pos.y += 1;
            **last_direction = Direction::Up;
        }
        Direction::Right => {
            head_pos.x += 1;
            **last_direction = Direction::Right;
        }
        Direction::Down => {
            head_pos.y -= 1;
            **last_direction = Direction::Down;
        }
    };

    let updated_head_pos = *head_pos;

    if head_pos.x < 0
        || head_pos.y < 0
        || head_pos.x as u32 >= ARENA_WIDTH
        || head_pos.y as u32 >= ARENA_HEIGHT
    {
        game_over_writer.send(GameOverEvent);
    }

    // Move rest of segments
    positions
        .iter_mut()
        .skip(1)
        .zip(old_positions.iter())
        .for_each(|(mut pos, old_pos)| {
            *pos = *old_pos;

            // Check if head touches any updated segment
            if *pos == updated_head_pos {
                game_over_writer.send(GameOverEvent);
            }
        });

    // Update last tail position
    *last_tail_position = LastTailPosition(Some(*positions.iter().last().unwrap()));
}

fn snake_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    last_direction: Res<LastDirection>,
    mut q: Query<&mut Direction, With<SnakeHead>>,
) {
    let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
        Direction::Left
    } else if keyboard_input.pressed(KeyCode::Right) {
        Direction::Right
    } else if keyboard_input.pressed(KeyCode::Down) {
        Direction::Down
    } else if keyboard_input.pressed(KeyCode::Up) {
        Direction::Up
    } else {
        return;
    };

    let mut head_direction = q.single_mut();

    if dir != last_direction.opposite() {
        *head_direction = dir;
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_position: Query<&Position, With<SnakeHead>>,
) {
    let head_pos = head_position.single();

    for (ent, food_pos) in food_positions.iter() {
        if food_pos == head_pos {
            commands.entity(ent).despawn();
            growth_writer.send(GrowthEvent);
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();

    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();

    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    mut commands: Commands,
    free_position_template: Res<FreePositionsTemplate>,
    q: Query<&Position>,
) {
    // Determine free fields
    let mut free_fields: HashSet<Position> = free_position_template.clone();

    for pos in q.iter() {
        free_fields.remove(pos);
    }

    // Pick free field
    let free_field = free_fields.iter().choose(&mut rand::thread_rng());

    if let Some(free_field) = free_field {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: FOOD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Food)
            .insert(*free_field)
            .insert(Size::square(0.8));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.iter().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        spawn_snake(commands, segments_res);
    }
}

// TODO: Add win screen if snake is maximum length

fn main() {
    App::new()
        // Resources
        .insert_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastDirection(Direction::Up))
        .insert_resource(LastTailPosition::default())
        .insert_resource(FreePositionsTemplate::default())
        // Startup Systems
        .add_startup_system(setup_camera)
        .add_startup_system(setup_free_position_template)
        .add_startup_system(spawn_snake)
        // Systems
        .add_system(snake_movement_input.before(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.5))
                .with_system(snake_movement)
                .with_system(snake_eating.after(snake_movement))
                .with_system(snake_growth.after(snake_eating)),
        )
        .add_system(game_over.after(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner.after(snake_eating)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        // Events
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        // Plugins
        .add_plugins(DefaultPlugins)
        .run();
}
