use std::collections::HashMap;

use rand::seq::IteratorRandom;
use rand::Rng;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_hex::{hex::HexCoord, *};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_outline::*;

const BOARD_SIZE: isize = 16;
const NUMBER_OF_PLAYERS: usize = 2;
const NUMBER_OF_PATCHES: usize = 16;

#[derive(Debug, Default)]
struct Board {
    hexes: HashMap<(isize, isize), (usize, f32)>,
    regions: Vec<Region>,
}

#[derive(Debug, Default)]
struct Region {
    hexes: Vec<(isize, isize)>,
    #[allow(dead_code)]
    owner: usize,
    height: f32,
}

impl Region {
    pub fn center_of_mass(&self) -> (f32, f32) {
        let mut x = 0.0;
        let mut y = 0.0;
        for (hx, hy) in self.hexes.iter() {
            x += *hx as f32;
            y += *hy as f32;
        }

        (x / self.hexes.len() as f32, y / self.hexes.len() as f32)
    }

    pub fn center_hex(&self) -> HexCoord {
        let center = self.center_of_mass();
        let mut nearest_hex: (isize, isize) = (0, 0);
        let mut min_distance = f32::MAX;
        for point in self.hexes.iter() {
            let distance =
                ((center.0 - point.0 as f32).powi(2) + (center.1 - point.1 as f32).powi(2)).sqrt();

            if distance < min_distance {
                min_distance = distance;
                nearest_hex = *point;
            }
        }

        HexCoord::new(nearest_hex.0, nearest_hex.1)
    }
}

fn generate_board() -> Board {
    const HALF_BOARD_SIZE: isize = BOARD_SIZE / 2 - 1;
    // Roughly half of the board occupied by patches (squads)
    const PATCH_SIZE: isize =
        (BOARD_SIZE * BOARD_SIZE) / (NUMBER_OF_PATCHES * NUMBER_OF_PLAYERS * 2) as isize;

    let mut rng = rand::thread_rng();
    let mut board = Board::default();

    for patch in 0..NUMBER_OF_PATCHES {
        for player in 0..NUMBER_OF_PLAYERS {
            let height = rng.gen_range(0.0..1.0);

            let mut is_starting_point_valid = false;

            while !is_starting_point_valid {
                let mut has_neighbours = false;

                while !has_neighbours {
                    let mut hex_snapshot = board.hexes.clone();

                    let q = rng.gen_range(-HALF_BOARD_SIZE..HALF_BOARD_SIZE);
                    let r = rng.gen_range(-HALF_BOARD_SIZE..HALF_BOARD_SIZE);

                    // check if starting position is empty
                    let initial_coord = (q, r);
                    if board.hexes.get(&initial_coord).is_none() {
                        is_starting_point_valid = true;
                        hex_snapshot.insert(initial_coord, (player, height));
                    } else {
                        // try over
                        continue;
                    }

                    // expand until size limit is reached or no more space to grow
                    let mut patch_hexes: Vec<(isize, isize)> = vec![initial_coord];

                    for _ in 0..PATCH_SIZE {
                        // find a bordering hex. use random iterating order to avoid bias
                        let mut border_hex: Option<HexCoord> = None;
                        for coord in patch_hexes
                            .iter()
                            .choose_multiple(&mut rng, patch_hexes.iter().len())
                        {
                            let hex = HexCoord::new(coord.0, coord.1);
                            // iterate over all neighbors and find a free one
                            for neighbor in hex.neighbors() {
                                let neighbour_coord = (neighbor.q, neighbor.r);
                                if hex_snapshot.get(&neighbour_coord).is_none() {
                                    border_hex = Some(hex.clone());
                                    break;
                                }
                            }

                            // continue expanding a border hex
                            if border_hex.is_some() {
                                break;
                            }
                        }

                        // no more hex cells in this patch
                        if border_hex.is_none() {
                            break;
                        }

                        // add a new hex to the patch
                        let mut candidates: Vec<(isize, isize)> = vec![];
                        for neighbor in border_hex.unwrap().neighbors() {
                            let neighbour_coord = (neighbor.q, neighbor.r);
                            if board.hexes.get(&neighbour_coord).is_none() {
                                candidates.push(neighbour_coord);
                            }
                        }
                        let candidate = candidates.iter().choose(&mut rng).unwrap();
                        patch_hexes.push(*candidate);
                        hex_snapshot.insert(*candidate, (player, height));
                    }

                    // check whether patch has any neighbors or start over
                    for patch_hex in patch_hexes.iter() {
                        let hex = HexCoord::new(patch_hex.0, patch_hex.1);
                        for neighbor in hex.neighbors() {
                            if board.hexes.get(&(neighbor.q, neighbor.r)).is_some() {
                                has_neighbours = true;
                                break;
                            }
                        }
                    }

                    // if could not generate a patch with a neightbours, start over
                    // except for the first patch
                    if player == 0 && patch == 0 && patch_hexes.len() > 1 {
                        has_neighbours = true;
                    }

                    // if patch has neighbours, add it to the board
                    // else, start over
                    if has_neighbours {
                        board.hexes = hex_snapshot;
                        board.regions.push(Region {
                            hexes: patch_hexes,
                            owner: player,
                            height,
                        });
                        break;
                    }
                }
            }
        }
    }

    board
}

/// Generate a single hex mesh
fn generate_hex_region_mesh(hexes: Vec<(isize, isize)>) -> Mesh {
    let mut pts: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut indices = vec![];

    for (hex_num, hex) in hexes.iter().enumerate() {
        let c = hex::HexCoord::new(0, 0);
        geometry::bevel_hexagon_points(&mut pts, 1.0, 1.0, &c);
        geometry::bevel_hexagon_normals(&mut normals);
        geometry::bevel_hexagon_indices(&mut indices, 0 as u32);

        let pos = geometry::center(1.0, &hex::HexCoord::new(hex.0, hex.1), &[0., 0.0, 0.]);

        for _ in 0..pts.len() {
            uvs.push([0.0, 0.0]);
        }

        for p in pts.len() - 22..pts.len() {
            pts[p][0] += pos[0];
            pts[p][1] += pos[1];
            pts[p][2] += pos[2];
        }

        for p in normals.len() - 22..normals.len() {
            normals[p][0] += pos[0];
            normals[p][1] += pos[1];
            normals[p][2] += pos[2];
        }

        for p in uvs.len() - 22..uvs.len() {
            uvs[p][0] += pos[0];
            uvs[p][1] += pos[1];
        }
        println!("uvs: {:?}", uvs);
        println!("normals: {:?}", normals);
        println!("pts: {:?}", pts);

        break;
    }

    println!("indices: {:?}", indices);

    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
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
            transform: Transform::from_translation(Vec3::new(40.0, 42., 0.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        });

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
    let board = generate_board();

    // Draw board
    for region in board.regions.iter() {
        let color = colors[region.owner as usize];
        let material = materials.add(color.into());

        let mut mesh = generate_hex_region_mesh(region.hexes.clone());
        mesh.generate_outline_normals().unwrap();
        let mesh = meshes.add(mesh);

        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..Default::default()
            })
            .insert(Name::new("Hex"));

        break;
    }

    return;

    // Place dice on areas
    let dice_handle = asset_server.load("models/dice/scene.gltf#Scene0");
    for region in board.regions.iter() {
        let center_hex = region.center_hex();

        let pos = geometry::center(1.0, &center_hex, &[0., 0.0, 0.]);

        commands
            .spawn_bundle(SceneBundle {
                scene: dice_handle.clone(),
                transform: Transform::from_xyz(pos[0], pos[1] + 0.383, pos[2])
                    .with_scale(Vec3::splat(0.9)),
                ..default()
            })
            .insert(Name::new("Dice"));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(OutlinePlugin)
        .add_startup_system(setup)
        .insert_resource(ClearColor(Color::DARK_GREEN))
        .run();
}
