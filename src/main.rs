mod game;
mod geometry;
mod hex;
mod highlight;

use bevy::{
    prelude::*,
    render::{camera::ScalingMode, mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_outline::*;

use bevy_mod_picking::{PickableBundle, PickingCameraBundle};
use game::{generate_board, Board, GameState, Region};
use geometry::center;
use rand::Rng;

use crate::geometry::flat_hexagon_points;
use crate::hex::HexCoord;

/// Generate a single hex mesh
fn generate_hex_region_mesh(region: &Region) -> Mesh {
    let hexes = region.hexes.clone();
    let center = center(1.0, &region.center_hex(), &[0.0, 0.0, 0.0]);

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
            indices.push(18 * hex_num); // Center
            indices.push(18 * hex_num + i + 1); // Point       East           North-east
            indices.push(18 * hex_num + i + 2); // Next point  North-east     North-west
        }

        // Adjust location and duplicate points with an offset as a bottom face
        for p in pts.len() - 9..pts.len() {
            pts[p][0] -= center[0];
            pts[p][1] -= center[1];
            pts[p][2] -= center[2];
            pts.push([pts[p][0], pts[p][1] - 0.0001, pts[p][2]]);
        }
        for _ in 0..9 {
            normals.push([0., -1., 0.]);
        }

        // Populate indices for bottom
        for i in 0..=6 {
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

        // Finally, UVs
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

const PLAYER_COLORS: [Color; 8] = [
    Color::PURPLE,
    Color::CYAN,
    Color::GREEN,
    Color::YELLOW,
    Color::RED,
    Color::ORANGE,
    Color::PINK,
    Color::OLIVE,
];

#[derive(Component)]
struct TitleText;

#[derive(Component)]
struct CurrentTurnText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands
        // camera
        .spawn_bundle(Camera3dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(3.0),
                scale: 10.0,
                ..default()
            }
            .into(),
            transform: Transform::from_translation(Vec3::new(50.0, 32., 0.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());

    // Title Text
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "STACK RANK DICE",
                TextStyle {
                    font: asset_server.load("fonts/HEXAGON_.TTF"),
                    font_size: 50.0,
                    color: Color::BLACK,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(TitleText);

    // Current Turn Text
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "current turn",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::BLACK,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(CurrentTurnText);
}

fn draw_board(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    board: ResMut<Board>,
) {
    // let board = game::generate_board(2);
    let mut rng = rand::thread_rng();

    // Draw board
    for region in board.regions.iter() {
        let color = PLAYER_COLORS[region.owner as usize];
        let material = materials.add(StandardMaterial {
            base_color: color.into(),
            metallic: 0.0,
            reflectance: 0.0,
            ..default()
        });
        let center_coord = center(1.0, &region.center_hex(), &[0.0, 0.0, 0.0]);

        let mut mesh = generate_hex_region_mesh(region);
        mesh.generate_outline_normals().unwrap();
        let mesh = meshes.add(mesh);
        let height: f32 = rng.gen_range(0.0..=0.0001);
        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(
                    center_coord[0],
                    center_coord[1] + height,
                    center_coord[2],
                )),
                ..Default::default()
            })
            .insert_bundle(OutlineBundle {
                outline: Outline {
                    visible: true,
                    colour: Color::rgba(0.0, 0.0, 0.0, 1.0),
                    width: 0.5,
                },
                ..default()
            })
            .insert(region.clone())
            .insert_bundle(PickableBundle::default())
            .insert(Name::new("Hex"));
    }

    // Place dice on areas
    let dice_mesh_handle = asset_server.load("models/dice/scene.gltf#Mesh0/Primitive0");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server
                .load("models/dice/textures/Dice_baseColor.png")
                .clone(),
        ),
        normal_map_texture: Some(
            asset_server
                .load("models/dice/textures/Dice_normal.png")
                .clone(),
        ),
        metallic_roughness_texture: Some(
            asset_server
                .load("models/dice/textures/Dice_metallicRoughness.png")
                .clone(),
        ),
        ..default()
    });

    for region in board.regions.iter() {
        let center_hex = region.center_hex();
        let pos = geometry::center(1.0, &center_hex, &[0., 0.0, 0.]);

        for i in 0..region.number_of_dice {
            let mut y_pos = pos[1] + 0.383 + (i as f32) * (2.0 * 0.383);
            let mut z_pos = pos[2];
            if i > 3 {
                y_pos = pos[1] + 0.383 + ((i - 4) as f32) * (2.0 * 0.383);
                z_pos += 0.383 * 2.0 + 0.01;
            }

            if region.number_of_dice > 3 {
                z_pos -= 0.383;
            }

            commands
                .spawn_bundle(PbrBundle {
                    mesh: dice_mesh_handle.clone(),
                    material: material_handle.clone(),
                    transform: Transform::from_xyz(pos[0], y_pos, z_pos)
                        .with_scale(Vec3::splat(0.4)),
                    ..default()
                })
                .insert(OutlineStencil {})
                .insert(Name::new("Dice"));
        }

        commands
            .spawn_bundle(PointLightBundle {
                transform: Transform::from_xyz(pos[0] + 2.0, 2.0, pos[2]),
                point_light: PointLight {
                    intensity: 100.0,
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("RegionLight"));
    }
}

fn player_turn_text_update(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<CurrentTurnText>>,
) {
    for mut text in &mut query {
        text.sections[0].value = format!("Player {} turn", game_state.turn + 1);
    }
}

// #[derive(Default)]
// pub struct SelectedPiece {
//     pub entity: Option<Entity>,
// }

// impl SelectedPiece {
//     pub fn deselect(&mut self) {
//         self.entity = None;
//     }
// }

// fn highlight_region(
//     selected_piece: Res<SelectedPiece>,
//     square_materials: Res<RegionMaterials>,
//     mut query: Query<(Entity, &game::Piece, &mut Handle<StandardMaterial>)>,
// ) {
//     for (entity, piece, mut material) in query.iter_mut() {
//         if Some(entity) == selected_piece.entity {
//             *material = square_materials.selected.clone();
//         } else if piece.color == game::Color::White {
//             *material = square_materials.white_color.clone();
//         } else {
//             *material = square_materials.blue_color.clone();
//         }
//     }
// }

// pub struct RegionMaterials {
//     pub selected: Handle<StandardMaterial>,
//     pub highlighted: Handle<StandardMaterial>,
// }

// impl FromWorld for RegionMaterials {
//     fn from_world(world: &mut World) -> Self {
//         let world = world.cell();
//         let mut materials_asset = world
//             .get_resource_mut::<Assets<StandardMaterial>>()
//             .unwrap();

//         RegionMaterials {
//             selected: materials_asset.add(bevy::prelude::Color::rgb(0.9, 0.1, 0.1).into()),
//             highlighted: materials_asset.add(bevy::prelude::Color::rgb(0., 0.1, 0.1).into()),
//         }
//     }
// }

fn main() {
    let number_of_players = 2;

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugins(highlight::StackRankDicePickingPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(OutlinePlugin)
        .add_startup_system(setup.label("setup"))
        .add_startup_system(draw_board.after("setup"))
        .add_system(player_turn_text_update)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(generate_board(number_of_players))
        .insert_resource(GameState {
            number_of_players,
            turn: 0,
        })
        // .init_resource::<RegionMaterials>()
        .run();
}
