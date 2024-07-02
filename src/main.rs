mod plugin;

use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages},
    sprite::MaterialMesh2dBundle
};
use bevy::window::PrimaryWindow;
use bevy_xpbd_2d::{math::*, prelude::*};
use bevy_asset::AssetServer;
use bevy_particle_systems::{
    CircleSegment, ColorOverTime, Curve, CurvePoint, JitteredValue, ParticleSpace, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, Playing, VelocityModifier
};
use bevy_procedural_meshes::*;
use noise::{Perlin, NoiseFn, BasicMulti};
use rand::Rng;
use plugin::*;


#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            CharacterControllerPlugin,
            ParticleSystemPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity(Vector::NEG_Y * 50.0))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let ww = window.width();
    let wh = window.height();
    let whw = ww / 2.;
    let whh = wh / 2.;

    // Player
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("lander.png").into(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                ..default()
            },
            transform: Transform::from_xyz(-whw + 10.,  whh, 0.0).with_rotation(Quat::from_rotation_z(-PI / 2.)),
            ..default()
        },
        CharacterControllerBundle::new(Collider::rectangle(40., 40.0)).with_movement(0.2),
        Friction::new(1.0).with_combine_rule(CoefficientCombine::Multiply),
        Restitution::PERFECTLY_INELASTIC,
    ))
    .with_children(|parent| {
        parent.spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 10000,
                emitter_shape: CircleSegment {
                    radius: JitteredValue::jittered(10.0, -2.0..2.0),
                    opening_angle: std::f32::consts::PI / 6.0,
                    direction_angle: -std::f32::consts::PI / 2.0
                }
                .into(),
                texture: asset_server.load("px.png").into(),
                spawn_rate_per_second: 0.0.into(),
                initial_speed: JitteredValue::from(0.0),
                lifetime: JitteredValue::jittered(0.5, -0.1..0.1),
                color: ColorOverTime::Gradient(Curve::new(vec![
                    CurvePoint::new(Color::RED, 0.0),
                    CurvePoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                velocity_modifiers: vec![
                    VelocityModifier::Vector(Vec3::new(0.0, -50.0, 0.0).into()),
                ],
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 3.0.into(),
                initial_rotation: (-90.0_f32).to_radians().into(),
                rotate_to_movement_direction: true,
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(0.0, -20.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
    });

    // a comet
    // commands.spawn((
    //     SpriteBundle {
    //         texture: asset_server.load("comet.png").into(),
    //         sprite: Sprite {
    //             custom_size: Some(Vec2::new(60., 60.)),
    //             ..default()
    //         },
    //         transform: Transform::from_xyz(50.0, 100.0, 0.0),
    //         ..default()
    //     },
    //     RigidBody::Dynamic,
    //     Collider::circle(20.0),
    // ));

    let noise = BasicMulti::<Perlin>::new(rand::thread_rng().gen_range(0..1000));

    let mut mesh = PMesh::<u32>::new();
    let mut points2d: Vec<Vec2> = vec![];
    mesh.fill(0.01, |builder| {
        builder.begin_here();

        let mut x = 0.;
        while x <= 1. {
            let y = noise.get([x as f64, 0.]) as f32;
            let point = Vec2 { x: x * window.width(), y: y * whh + whh };
            points2d.push(point);
            builder.line_to(point);
            x += 0.01;
        }

        builder.line_to(Vec2 { x: ww, y: 0. });

        builder.close();
    });

    let bevy_mesh = mesh.to_bevy(RenderAssetUsages::all());
    let collider = Collider::polyline(points2d, None);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(bevy_mesh).into(),
            material: materials.add(Color::rgb(0.4, 0.4, 0.5)),
            transform: Transform::from_xyz(-whw, -whh, 0.0),
            ..default()
        },
        RigidBody::Static,
        Friction::new(1.0).with_combine_rule(CoefficientCombine::Multiply),
        Restitution::PERFECTLY_ELASTIC,
        collider,
    ));

    // Camera
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
