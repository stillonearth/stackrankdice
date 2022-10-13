mod game;
mod geometry;
mod hex;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
// use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_outline::*;
use bevy_mod_picking::*;

use rand::Rng;

use crate::geometry::flat_hexagon_points;
use crate::hex::HexCoord;

/// Generate a single hex mesh
fn generate_hex_region_mesh(hexes: Vec<(isize, isize)>) -> Mesh {
    let mut pts: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut indices: Vec<u32> = vec![];

    for (hex_num, hex) in hexes.iter().enumerate() {
        let c = HexCoord::new(hex.0, hex.1);
        let hex_num = hex_num as u32;

        // Populate the points for the top face, as a slightly scaled hexagon
        flat_hexagon_points(&mut pts, 1.0, &c);
        for _ in 0..9 {
            normals.push([0., 1., 0.]);
        }
        for i in 0..=6 {
            //           first-time     second-time
            indices.push(18 * hex_num); // Center
            indices.push(18 * hex_num + i + 1); // Point       East           North-east
            indices.push(18 * hex_num + i + 2); // Next point  North-east     North-west
        }

        // Duplicate points with an offset as a bottom face
        for p in pts.len() - 9..pts.len() {
            pts.push([pts[p][0], pts[p][1] - 0.005, pts[p][2]]);
        }
        for _ in 0..9 {
            normals.push([0., -1., 0.]);
        }

        // Populate indices for bottom
        for i in 0..=6 {
            //           first-time     second-time
            indices.push(18 * hex_num + 9); // Center
            indices.push(18 * hex_num + i + 1 + 9); // Point       East           North-east
            indices.push(18 * hex_num + i + 2 + 9); // Next point  North-east     North-west
        }

        // Populate indices sides
        for i in 0..=6 {
            indices.push(18 * hex_num + i + 2);
            indices.push(18 * hex_num + i + 1 + 9);
            indices.push(18 * hex_num + i + 2 + 9);

            indices.push(18 * hex_num + i + 2);
            indices.push(18 * hex_num + i + 1);
            indices.push(18 * hex_num + i + 1 + 9);
        }

        for _ in 0..18 {
            uvs.push([1.0, 1.0]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        // camera
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(50.0, 32., 0.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());

    // Lightning
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    let colors = [Color::PURPLE, Color::CYAN];
    let board = game::generate_board();

    // Draw board
    let mut rng = rand::thread_rng();
    for region in board.regions.iter() {
        let color = colors[region.owner as usize];
        let material = materials.add(color.into());

        let mut mesh = generate_hex_region_mesh(region.hexes.clone());
        mesh.generate_outline_normals().unwrap();
        let mesh = meshes.add(mesh);

        let height = rng.gen_range(0.0..0.05);

        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, height, 0.0)),
                ..Default::default()
            })
            .insert(Name::new("Hex"))
            .insert_bundle(OutlineBundle {
                outline: Outline {
                    visible: true,
                    colour: Color::rgba(1.0, 1.0, 1.0, 0.5),
                    width: 1.0,
                },
                ..default()
            })
            .insert_bundle(PickableBundle::default());
    }

    // Place dice on areas
    let dice_handle = asset_server.load("models/dice/scene.gltf#Scene0");
    for region in board.regions.iter() {
        let center_hex = region.center_hex();
        let pos = geometry::center(1.0, &center_hex, &[0., 0.0, 0.]);

        for i in 0..region.number_of_dice {
            let mut y_pos = pos[1] + 0.383 + (i as f32) * (2.0 * 0.383);
            let mut z_pos = pos[2];
            if i > 3 {
                y_pos = pos[1] + 0.383 + ((i - 4) as f32) * (2.0 * 0.383);
                z_pos += 0.383 * 2.0;
            }

            commands
                .spawn_bundle(SceneBundle {
                    scene: dice_handle.clone(),
                    transform: Transform::from_xyz(pos[0], y_pos, z_pos)
                        .with_scale(Vec3::splat(0.9)),
                    ..default()
                })
                .insert(Name::new("Dice"));
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(OutlinePlugin)
        .add_startup_system(setup)
        .insert_resource(ClearColor(Color::WHITE))
        .run();
}
