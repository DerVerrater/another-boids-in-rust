
use bevy::{input::keyboard::Key, math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume}, prelude::*, sprite::MaterialMesh2dBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            apply_velocity,
            move_paddle,
            check_for_collisions,
        ).chain())
        .add_systems(Update, update_scoreboard)
        .run();
}

// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// These values are exact
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);


#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Brick;

#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi;

#[derive(Bundle)]
struct WallBundle{
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    fn new(location: WallLocation) -> Self {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}
enum WallLocation { Right, Top, Left, Bottom }

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
        }
    }
    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            },
            WallLocation::Top | WallLocation::Bottom => {
                Vec2::new(WALL_THICKNESS + arena_width, WALL_THICKNESS)
            }
        }
    }
}

enum Collision { Right, Top, Left, Bottom }

fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    
    // paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE.extend(1.0),
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
    ));

    // ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // scoreboard
    commands.spawn((
        ScoreboardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        })
    ));

    // walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2.0 * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // brick count is dynamic based on available game board space (and brick size, but that's a constant)
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // Bevy uses the center as the origin of an entity
    // so calculate offset from left-edge to get the correct position
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.0;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.0;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new (
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );
            // spawn sprite, brick, collider
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: BRICK_SIZE.extend(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        let vel_acc = **velocity * time.delta_seconds();
        let vel_acc = vel_acc.extend(0.0);
        transform.translation += vel_acc;
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    let new_paddle_position = paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn check_for_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (collider_entity, collider_transform, maybe_brick) in &collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.0),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.0,
            ),
        );

        if let Some(collision) = collision {
            collision_events.send_default();

            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn();
                **score += 1;
            }

            let mut reflect = (false, false);
            match collision {
                Collision::Right => reflect.0 = ball_velocity.x < 0.0,
                Collision::Top => reflect.1 = ball_velocity.y < 0.0,
                Collision::Left => reflect.0 = ball_velocity.x > 0.0,
                Collision::Bottom => reflect.1 = ball_velocity.y > 0.0,
            }

            if reflect.0 {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect.1 {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text, With<ScoreboardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = score.to_string();
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("John".to_string())));
    commands.spawn((Person, Name("Jack".to_string())));
    commands.spawn((Person, Name("Bill".to_string())));
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn greet_people(
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    query: Query<&Name, With<Person>>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("Hello {}!", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Bill" {
            name.0 = "Jill".to_string();
            break;
        }
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App){
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, add_people)
            .add_systems(Update, (update_people, greet_people).chain());
    }
}
