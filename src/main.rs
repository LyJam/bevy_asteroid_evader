use bevy::window::{Window, WindowMode};
use bevy::{prelude::*, transform, window::PrimaryWindow};
use rand::Rng;

const SPACESHIP_ACCELERATION: f32 = 1.0;
const DRAG: f32 = 0.96;
const ASTEROID_COUNT: usize = 10;
const ASTROID_SPEED_MIN: f32 = 1.5;
const ASTROID_SPEED_MAX: f32 = 4.0;

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct HasDrag;

#[derive(Component)]
struct Velocity {
    velocity: Vec2,
}

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
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                destroy_asteroids,
                spawn_asteroids,
                player_input,
                move_objects,
                apply_drag,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut spaceship = commands.spawn((
        Sprite::from_image(asset_server.load("Sprites/spaceship.png")),
        Transform::from_scale(Vec3::new(0.3, 0.3, 0.3)),
    ));

    spaceship.insert(Spaceship);
    spaceship.insert(Velocity::new(0.0, 0.0));
    spaceship.insert(HasDrag);
}

fn spawn_asteroids(
    mut commands: Commands,
    asteroids: Query<&Asteroid>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mut total_asteroids = asteroids.iter().count();

    while total_asteroids < ASTEROID_COUNT {
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

        let transform =
            Transform::from_xyz(spawn_x, spawn_y, 0.0).with_scale(Vec3::new(0.3, 0.3, 0.3));

        let mut asteroid = commands.spawn((
            Sprite::from_image(asset_server.load("Sprites/asteroid.png")),
            transform,
        ));

        asteroid.insert(Velocity::new(velocity.x, velocity.y));

        asteroid.insert(Asteroid);

        total_asteroids += 1;
    }
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

fn move_objects(mut objects: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in objects.iter_mut() {
        transform.translation.x += velocity.velocity.x;
        transform.translation.y += velocity.velocity.y;
    }
}

fn apply_drag(mut objects: Query<&mut Transform, With<HasDrag>>) {
    for mut transform in objects.iter_mut() {
        transform.translation.x = transform.translation.x * DRAG;
        transform.translation.y = transform.translation.y * DRAG;
    }
}
