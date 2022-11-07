use bevy::app::App;
use bevy::app::Plugin;
use bevy::ecs::world::FromWorld;
use bevy::ecs::world::World;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug)]
pub struct PrngResource {
    pub world_seed: u64,
    pub env_seed: u64,
}

pub struct PrngPlugin;

#[derive(Debug)]
pub struct PrngMapResource {
    pub rng: ChaCha20Rng,
}

impl Plugin for PrngPlugin {
    fn build(&self, app: &mut App) {
        let seeds = app.world.get_resource::<PrngResource>().unwrap();

        let map_rng = ChaCha20Rng::seed_from_u64(seeds.world_seed);
        app.insert_resource(PrngMapResource { rng: map_rng });
    }
}

impl FromWorld for PrngResource {
    fn from_world(_world: &mut World) -> Self {
        // Values zero will be considered uninitialized

        //world.insert_resource(); // MapPrng
        //world.insert_resource(); // AiPrng
        PrngResource {
            world_seed: 0,
            env_seed: 0,
        }
    }
}
