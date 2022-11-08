mod board;
mod events;
mod game;
mod geometry;
mod hex;
mod highlights;
mod tiered_prng;
mod ui;

use board::draw_board;
use clap::Parser;
use rand::rngs::OsRng;
use rand::RngCore;

use bevy::prelude::*;
use bevy_dice::{DicePlugin, DicePluginSettings};
use bevy_mod_outline::*;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use ui::{dice_roll_result_text_update, player_turn_text_update, setup_ui};

use crate::events::*;
use crate::game::{generate_board, GameState, Region};
use crate::tiered_prng::get_randomness;

#[derive(Default)]
pub struct SelectedRegion {
    pub entity: Option<Entity>,
    pub region: Option<Region>,
}

impl SelectedRegion {
    pub fn select(&mut self, entity: Entity, region: Region) {
        self.entity = Some(entity);
        self.region = Some(region);
    }

    pub fn deselect(&mut self) {
        self.entity = None;
        self.region = None;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    world_seed: u64,

    #[arg(short, long, default_value_t = 0)]
    env_seed: u64,
}

fn main() {
    let number_of_players = 2;

    let mut args = Args::parse();

    if args.world_seed == 0 || args.env_seed == 0 {
        let mut key = [0u8; 16];
        OsRng.fill_bytes(&mut key);

        // If one, or the other is set, only generate for the unset one.
        // This will allow easier testing later, for fixed world random env_seed.
        // Or for specific AI testing, fixed env_seed but random world.
        if args.world_seed == 0 {
            args.world_seed = OsRng.next_u64();
        }
        if args.env_seed == 0 {
            args.env_seed = OsRng.next_u64();
        }
    }

    // Source of randomness for the game
    let prng_resource = tiered_prng::PrngResource {
        world_seed: args.world_seed,
        env_seed: args.env_seed,
    };

    // Generate game map
    let map = generate_board(number_of_players, get_randomness(prng_resource.world_seed));

    App::new()
        // PRNG setup
        .insert_resource(prng_resource)
        // Plugins
        .add_plugin(tiered_prng::PrngPlugin) // Adds Prng based resources for subcomponents
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_kira_audio::prelude::AudioPlugin)
        .add_plugins(highlights::StackRankDicePickingPlugins)
        .add_plugin(OutlinePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(DicePlugin)
        // Resources
        .insert_resource(DicePluginSettings {
            render_size: (640 * 2, 720 * 2),
            number_of_fields: 2,
            ..default()
        })
        .insert_resource(GameState {
            board: map,
            number_of_players,
            turn_of_player: 0,
            turn_counter: 0,
            game_log: Vec::new(),
        })
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<SelectedRegion>()
        // Startup Systems
        .add_startup_system(setup_ui.after("dice_plugin_init").label("setup"))
        .add_startup_system(draw_board.after("setup"))
        // UI Systems
        .add_system(player_turn_text_update)
        .add_system(dice_roll_result_text_update)
        // Control Handling
        .add_system_to_stage(CoreStage::PostUpdate, event_region_selected)
        // Event Handlers
        .add_system(event_region_clash)
        .add_system(event_dice_roll_result)
        .add_system(event_dice_rolls_complete)
        .add_system(event_region_clash_end)
        .add_system(event_game_over)
        // Events
        .add_event::<EventPlayerMoveStart>()
        .add_event::<EventPlayerMoveEnd>()
        .add_event::<EventGameOver>()
        .add_event::<EventTurnStart>()
        .add_event::<EventTurnEnd>()
        // Ignite Engine
        .run();
}
