use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::random;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;
const RES_HEIGHT: f32 = 500.;
const RES_WIDTH: f32 = 500.;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
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

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct Food;

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct FoodSpawnTimer(Timer);

fn food_spawner(time: Res<Time>, mut query: Query<&mut FoodSpawnTimer>, mut commands: Commands) {
    let mut timer = query.get_single_mut().unwrap();

    if timer.tick(time.delta()).just_finished() {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: FOOD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Food)
            .insert(Position {
                x: ((random::<f32>() - 0.5) * ARENA_WIDTH as f32) as i32,
                y: ((random::<f32>() - 0.5) * ARENA_HEIGHT as f32) as i32,
            })
            .insert(Size::square(0.8));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_snake(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeHead {
            direction: Direction::Up,
        })
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8));
}

fn spawn_food_timer(mut commands: Commands) {
    commands.spawn(FoodSpawnTimer(Timer::from_seconds(
        1.,
        TimerMode::Repeating,
    )));
}

#[derive(Component, Deref, DerefMut)]
struct MovementTimer(Timer);

fn spawn_movement_timer(mut commands: Commands) {
    commands.spawn(MovementTimer(Timer::from_seconds(
        0.15,
        TimerMode::Repeating,
    )));
}

fn size_scaling(
    mut q: Query<(&Size, &mut Transform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.get_single().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.resolution.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.resolution.height(),
            1.0,
        );
    }
}

fn position_translation(
    window: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_game / 2.) + (tile_size / 2.)
    }
    let window = window.get_single().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.resolution.width(), ARENA_WIDTH as f32),
            convert(
                pos.y as f32,
                window.resolution.height(),
                ARENA_HEIGHT as f32,
            ),
            0.0,
        )
    }
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn snake_movement(
    time: Res<Time>,
    mut timer: Query<&mut MovementTimer>,
    mut heads: Query<(&mut Position, &SnakeHead)>,
) {
    let mut timer = timer.get_single_mut().unwrap();
    if timer.tick(time.delta()).just_finished() {
        if let Some((mut head_pos, head)) = heads.iter_mut().next() {
            match &head.direction {
                Direction::Left => head_pos.x = (head_pos.x - 1) % ARENA_WIDTH as i32,
                Direction::Right => head_pos.x = (head_pos.x + 1) % ARENA_WIDTH as i32,
                Direction::Up => {
                    head_pos.y = (head_pos.y + 1) % ARENA_HEIGHT as i32;
                }
                Direction::Down => {
                    head_pos.y = (head_pos.y - 1) % ARENA_HEIGHT as i32;
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_startup_system(spawn_food_timer)
        .add_startup_system(spawn_movement_timer)
        .add_system(snake_movement)
        .add_system(food_spawner)
        .add_system(snake_movement_input.before(snake_movement))
        .add_systems((position_translation, size_scaling).in_base_set(CoreSet::PostUpdate))
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake!".to_string(),
                resolution: (RES_WIDTH, RES_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .run();
}
