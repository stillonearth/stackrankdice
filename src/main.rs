mod app;
mod board;
mod events;
mod game;
mod geometry;
mod hex;
mod highlights;
mod tiered_prng;
mod ui;

use app::build_app;
use clap::Parser;
use rand::rngs::OsRng;
use rand::RngCore;

use bevy::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    world_seed: u64,

    #[arg(short, long, default_value_t = 0)]
    env_seed: u64,
}

fn main() {
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

    let app = &mut App::new();
    build_app(app, args.world_seed, args.env_seed, 2, false);
    app.run();
}
