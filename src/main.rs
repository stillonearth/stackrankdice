use std::collections::HashMap;

use bevy::prelude::*;
use bevy_hex::{hex::HexCoord, *};
use rand::seq::IteratorRandom;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .insert_resource(ClearColor(Color::WHITE))
        .run();
}

const BOARD_SIZE: isize = 32;
const NUMBER_OF_PLAYERS: usize = 2;
const NUMBER_OF_PATCHES: usize = 16;

#[derive(Debug)]
struct Board {
    hexes: HashMap<(isize, isize), (usize, f32)>,
}

fn generate_board() -> Board {
    const HALF_BOARD_SIZE: isize = BOARD_SIZE / 2 - 1;
    // Roughly half of the board occupied by patches (squads)
    const PATCH_SIZE: isize =
        (BOARD_SIZE * BOARD_SIZE) / (NUMBER_OF_PATCHES * NUMBER_OF_PLAYERS * 2) as isize;

    let mut rng = rand::thread_rng();
    let mut board = Board {
        hexes: HashMap::new(),
    };

    for patch in 0..NUMBER_OF_PATCHES {
        for player in 0..NUMBER_OF_PLAYERS {
            let height = rng.gen_range(0.0..3.0);

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
                    if player == 0 && patch == 0 {
                        has_neighbours = true;
                    }

                    // if patch has neighbours, add it to the board
                    // else, start over
                    if has_neighbours {
                        board.hexes = hex_snapshot;
                        break;
                    }
                }
            }
        }
    }

    return board;
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        // camera
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(3.0, 64., 0.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        });

    // Lightning
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1000.0, 0.0)),
        ..Default::default()
    });

    let colors = [Color::PURPLE, Color::CYAN];
    let mesh = meshes.add(generate_hex_mesh()).clone();
    let board = generate_board();

    for (coord, hex_data) in board.hexes.iter() {
        let color = colors[hex_data.0];
        let material = materials.add(color.into());
        let pos = geometry::center(
            1.0,
            &hex::HexCoord::new(coord.0, coord.1),
            &[0., hex_data.1, 0.],
        );
        commands.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(pos[0], pos[1], pos[2])),
            ..Default::default()
        });
    }
}
