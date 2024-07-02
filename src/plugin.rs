use bevy::prelude::*;
use bevy_xpbd_2d::{math::*, prelude::*};
use bevy_particle_systems::{
    JitteredValue, ParticleSystem,
};

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>().add_systems(
            Update,
            (
                keyboard_input,
                movement,
                apply_impulse
            )
                .chain(),
        );
    }
}

#[derive(Event)]
pub enum MovementAction {
    Torque(Scalar),
    Thrust,
}

#[derive(Component)]
pub struct CharacterController;

#[derive(Component)]
pub struct EnginePower(Scalar);

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    movement: MovementBundle,
}

#[derive(Bundle)]
pub struct MovementBundle {
    engine_power: EnginePower,
    impulse: ExternalImpulse,
    angular_impulse: ExternalAngularImpulse
}

impl MovementBundle {
    pub fn new(
        engine_power: Scalar,
    ) -> Self {
        Self {
            engine_power: EnginePower(engine_power),
            impulse: ExternalImpulse::default(),
            angular_impulse: ExternalAngularImpulse::default()
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.9, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        engine_power: Scalar
    ) -> Self {
        self.movement = MovementBundle::new(engine_power);
        self
    }
}

fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let direction = horizontal as Scalar;

    if direction != 0.0 {
        movement_event_writer.send(MovementAction::Torque(direction));
    }

    if keyboard_input.pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Thrust);
    }
}

fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &mut EnginePower,
        &mut ExternalAngularImpulse,
    )>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (mut engine_power, mut angular_impulse) in
            &mut controllers
        {
            match event {
                MovementAction::Torque(direction) => {
                    angular_impulse.apply_impulse(-direction * 10000.0);
                }
                MovementAction::Thrust => {
                    engine_power.0 += 0.4 * delta_time;
                }
            }
        }
    }
}

fn apply_impulse(
    time: Res<Time>,
    mut query: Query<(
        &mut EnginePower,
        &mut ExternalImpulse,
        &mut Transform,
        &Children
    )>,
    mut child_query: Query<&mut ParticleSystem>
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for (mut engine_power, mut impulse, transform, children) in &mut query {
        engine_power.0 -= 0.2 * delta_time;
        if engine_power.0 < 0.0 {
            engine_power.0 = 0.;
        }

        impulse.apply_impulse((transform.rotation * Vec3::Y).xyz().truncate().normalize_or_zero() * 10000. * engine_power.0);

        for &child in children.iter() {
            if let Ok(mut particle_system) = child_query.get_mut(child) {
                particle_system.initial_speed = JitteredValue::jittered(500. * engine_power.0, 0.0..(100.0 * (1. + engine_power.0)));
                particle_system.spawn_rate_per_second = (1000. * engine_power.0).into();
            }
        }
    }
}
