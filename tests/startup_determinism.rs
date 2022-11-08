use bevy::prelude::*;
use rand::Rng;
use stackrankdice::tiered_prng::{PrngMapResource, PrngPlugin, PrngResource};

#[test]
fn fixed_world_undef_env_seed() {
    // Setup app
    let mut app = App::new();

    // PRNG setup
    app.insert_resource(PrngResource {
        world_seed: 4242,
        env_seed: 0,
    });

    app.add_plugin(PrngPlugin); // Adds Prng based resources for subcomponents

    let mut map_prng = app.world.get_resource_mut::<PrngMapResource>().unwrap();

    let first: f32 = map_prng.rng.gen_range(0.0..=0.0001);
    assert_eq!(first, 2.1680013e-5);
}
