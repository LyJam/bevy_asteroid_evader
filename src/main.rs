use bevy::{prelude::*, window::PrimaryWindow};

const SPACESHIP_ACCELERATION: f32 = 1.0;
const DRAG: f32 = 0.96;

#[derive(Component)]
struct Spaceship;

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
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (player_input, move_objects, apply_drag).chain(),
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
