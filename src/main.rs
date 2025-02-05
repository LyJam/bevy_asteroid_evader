use bevy::ecs::system::SystemId;
use bevy::window::{Window, WindowMode};
use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

const SPACESHIP_ACCELERATION: f32 = 1.0;
const DRAG: f32 = 0.96;
const ASTEROID_COUNT: usize = 10;
const ASTEROID_DIFICULTY_SCALING: usize = 2; // for every N points one extra astroids get's added
const STARTS_COUNT: usize = 3;
const ASTROID_SPEED_MIN: f32 = 1.5;
const ASTROID_SPEED_MAX: f32 = 4.0;
const SPACESHIP_SIZE: f32 = 50.0;
const ASTROID_SIZE: f32 = 80.0;
const STAR_SIZE: f32 = 40.0;
const COLLISION_MARIGN: f32 = 25.0; // tolerance granted in overlap before it is defined as a collision

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Star;

#[derive(Component)]
struct StarScoreText;

#[derive(Component)]
struct HasDrag;

#[derive(Component)]
struct TargetPosition(Vec2);

#[derive(Component)]
struct CurrentPosition(Vec2);

#[derive(Component)]
struct Velocity {
    velocity: Vec2,
}

#[derive(Resource)]
struct ResetGame(SystemId);

#[derive(Resource)]
struct StarScore(u32);

impl Velocity {
    fn new(x: f32, y: f32) -> Self {
        Self {
            velocity: Vec2::new(x, y),
        }
    }

    fn accelerate(&mut self, acceleration: Vec2) {
        self.velocity = self.velocity + acceleration;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, init_resources))
        .add_systems(
            FixedUpdate,
            (
                destroy_asteroids,
                spawn_asteroids,
                spawn_stars,
                player_input,
                move_objects,
                collect_stars,
                update_scoreboard,
                check_collision,
                apply_drag,
            )
                .chain(),
        )
        .add_systems(Update, update_position)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut spaceship_sprite = Sprite::from_image(asset_server.load("Sprites/spaceship.png"));
    spaceship_sprite.custom_size = Some(Vec2::new(SPACESHIP_SIZE, SPACESHIP_SIZE));

    let position = Vec2::new(0.0, 0.0);

    commands.spawn((
        spaceship_sprite,
        Transform::from_xyz(position.x, position.y, 0.0),
        CurrentPosition(position),
        TargetPosition(position),
        Spaceship,
        Velocity::new(0.0, 0.0),
        HasDrag,
    ));

    commands.spawn((
        Text::new("Score: 0"),
        TextFont::from_font_size(50.0),
        TextColor(Color::srgb(255.0 / 255.0, 215.0 / 255.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
        StarScoreText,
    ));
}

fn init_resources(mut commands: Commands) {
    let system_id = commands.register_system(reset_game);
    commands.insert_resource(ResetGame(system_id));

    commands.insert_resource(StarScore(0));
}

fn reset_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    all_objects: Query<Entity, With<Transform>>,
    mut score: ResMut<StarScore>,
) {
    for entity in all_objects.iter() {
        commands.entity(entity).despawn();
    }

    score.0 = 0;
    setup(commands, asset_server);
}

fn spawn_asteroids(
    mut commands: Commands,
    asteroids: Query<&Asteroid>,
    asset_server: Res<AssetServer>,
    score: Res<StarScore>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mut total_asteroids = asteroids.iter().count();

    let asteroid_target = ASTEROID_COUNT + ((score.0 as usize) / ASTEROID_DIFICULTY_SCALING);

    while total_asteroids < asteroid_target {
        let mut rng = rand::rng();
        let window = windows.single();
        let width = window.width();
        let height = window.height();

        // Define spawn margin outside the screen
        let margin = 100.0;

        // Determine spawn position outside screen bounds
        let (spawn_x, spawn_y) = match rng.random_range(0..4) {
            0 => (
                // Top
                rng.random_range(-width / 2.0 - margin..width / 2.0 + margin),
                height / 2.0 + margin,
            ),
            1 => (
                // Right
                width / 2.0 + margin,
                rng.random_range(-height / 2.0 - margin..height / 2.0 + margin),
            ),
            2 => (
                // Bottom
                rng.random_range(-width / 2.0 - margin..width / 2.0 + margin),
                -height / 2.0 - margin,
            ),
            _ => (
                // Left
                -width / 2.0 - margin,
                rng.random_range(-height / 2.0 - margin..height / 2.0 + margin),
            ),
        };

        // Generate random target point within screen bounds
        let target_x = rng.random_range(-width / 2.0 + 100.0..width / 2.0 - 100.0);
        let target_y = rng.random_range(-height / 2.0 + 100.0..height / 2.0 - 100.0);

        // Calculate direction vector towards target
        let dx = target_x - spawn_x;
        let dy = target_y - spawn_y;
        let length = (dx * dx + dy * dy).sqrt();
        let direction = Vec2::new(dx / length, dy / length);

        // Generate random speed
        let speed = rng.random_range(ASTROID_SPEED_MIN..ASTROID_SPEED_MAX);
        let velocity = direction.normalize() * speed;

        let mut asteroid_sprite = Sprite::from_image(asset_server.load("Sprites/asteroid.png"));
        asteroid_sprite.custom_size = Some(Vec2::new(ASTROID_SIZE, ASTROID_SIZE));

        let transform = Transform::from_xyz(spawn_x, spawn_y, 0.0);

        commands.spawn((
            asteroid_sprite,
            transform,
            Velocity::new(velocity.x, velocity.y),
            Asteroid,
            CurrentPosition(Vec2::new(spawn_x, spawn_y)),
            TargetPosition(Vec2::new(spawn_x, spawn_y)),
        ));

        total_asteroids += 1;
    }
}

fn spawn_stars(
    mut commands: Commands,
    stars: Query<&Star>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mut total_stars = stars.iter().count();

    while total_stars < STARTS_COUNT {
        let mut rng = rand::rng();
        let window = windows.single();
        let width = window.width();
        let height = window.height();

        // random point on screen with 150px margin from edge
        let point_x = rng.random_range(-width / 2.0 + 150.0..width / 2.0 - 150.0);
        let point_y = rng.random_range(-height / 2.0 + 150.0..height / 2.0 - 150.0);

        let transform = Transform::from_translation(Vec3::new(point_x, point_y, -1.0));

        let mut star_sprite = Sprite::from_image(asset_server.load("Sprites/star.png"));
        star_sprite.custom_size = Some(Vec2::new(STAR_SIZE, STAR_SIZE));

        commands.spawn((star_sprite, transform, Star));

        total_stars += 1;
    }
}

fn collect_stars(
    spaceship_query: Query<&Transform, With<Spaceship>>,
    star_query: Query<(Entity, &Transform), With<Star>>,
    mut commands: Commands,
    mut score: ResMut<StarScore>,
) {
    let spaceship_transform = spaceship_query.single();

    for (star_entity, star_transform) in star_query.iter() {
        let distance_between = spaceship_transform
            .translation
            .truncate()
            .distance(star_transform.translation.truncate());
        if distance_between < (SPACESHIP_SIZE / 2.0 + STAR_SIZE / 2.0) {
            commands.entity(star_entity).despawn();
            score.0 += 1;
        }
    }
}

fn update_scoreboard(mut scoreboard: Query<&mut Text, With<StarScoreText>>, score: Res<StarScore>) {
    scoreboard.single_mut().0 = format!("Score: {:?}", score.0);
}

fn destroy_asteroids(
    mut commands: Commands,
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let margin = 120.0;

    for (entity, transform) in asteroids.iter() {
        let position = transform.translation;

        // Check boundaries relative to center (0,0)
        if position.x < -half_width - margin
            || position.x > half_width + margin
            || position.y < -half_height - margin
            || position.y > half_height + margin
        {
            commands.entity(entity).despawn();
        }
    }
}

fn player_input(
    mut spaceship: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            let (mut spaceship_transform, mut spaceship_speed) = spaceship.single_mut();

            // Calculate the direction vector from spaceship to mouse
            let direction = world_pos - spaceship_transform.translation.truncate();

            // Calculate the angle using atan2
            let angle = direction.y.atan2(direction.x);

            // Set the rotation (subtract 90 degrees since sprite faces up by default)
            spaceship_transform.rotation =
                Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);

            if buttons.pressed(MouseButton::Left) {
                let acceleration = direction.normalize() * SPACESHIP_ACCELERATION;

                spaceship_speed.accelerate(acceleration);
            }
        }
    }
}

fn move_objects(mut objects: Query<(&mut CurrentPosition, &mut TargetPosition, &Velocity)>) {
    for (mut current, mut target, velocity) in objects.iter_mut() {
        current.0 = target.0;

        target.0 += velocity.velocity;
    }
}

fn apply_drag(mut objects: Query<&mut Velocity, With<HasDrag>>) {
    for mut velocity in objects.iter_mut() {
        velocity.velocity.x *= DRAG;
        velocity.velocity.y *= DRAG;
    }
}

fn update_position(
    fixed_time: Res<Time<Fixed>>,
    mut objects: Query<(&mut Transform, &CurrentPosition, &TargetPosition)>,
) {
    for (mut transform, current, target) in &mut objects {
        let a = fixed_time.overstep_fraction();

        transform.translation = current.0.lerp(target.0, a).extend(0.0);
    }
}

fn check_collision(
    spaceship_query: Query<&Transform, With<Spaceship>>,
    asteroid_query: Query<&Transform, With<Asteroid>>,
    reset_fn: Res<ResetGame>,
    mut commands: Commands,
) {
    let spaceship_transform = spaceship_query.single();

    for astroid_transform in asteroid_query.iter() {
        let distance_between = spaceship_transform
            .translation
            .truncate()
            .distance(astroid_transform.translation.truncate());
        if distance_between < (SPACESHIP_SIZE / 2.0 + ASTROID_SIZE / 2.0 - COLLISION_MARIGN) {
            commands.run_system(reset_fn.0);
        }
    }
}
