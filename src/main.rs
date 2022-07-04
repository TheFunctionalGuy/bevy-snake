/// I like CREQ
use bevy::prelude::*;

use bevy::core::FixedTimestep;
use rand::prelude::random;

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

#[derive(Default, Debug)]
struct LastTailPosition(Option<Position>);

// Events
struct GrowthEvent;

// Components
#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeSegment;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
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

    // Move rest of segments
    positions
        .iter_mut()
        .skip(1)
        .zip(old_positions.iter())
        .for_each(|(mut pos, old_pos)| {
            *pos = *old_pos;
        });

    // Update last tail position
    *last_tail_position = LastTailPosition(Some(*positions.iter().last().unwrap()));

    println!("Last tail position: {:?}", *last_tail_position)
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

// TODO: What about stacking food? Need to handle that
fn food_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

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
        // Startup Systems
        .add_startup_system(setup_camera)
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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        // Events
        .add_event::<GrowthEvent>()
        // Plugins
        .add_plugins(DefaultPlugins)
        .run();
}
