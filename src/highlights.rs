use bevy::{app::PluginGroupBuilder, asset::Asset, ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_picking::{
    highlight::get_initial_mesh_highlight_asset, Highlighting, InteractablePickingPlugin,
    PausedForBlockers, PickingPlugin, PickingPluginsState, PickingSystem, Selection,
};

use crate::{
    game::{GameState, Region},
    SelectedRegion,
};

// This code is based on bevy_mod_picking. Standard use-case for bevy_mod_picking is limited
// and doesn't allow to customize colors of objects highlighted based on their metadata.
// This code is a workaround for this limitation and is based on bevy_mod_picking internals.

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`Highlighting`] component.
pub struct StackRankDiceDefaultHighlighting<T: StackRankDiceHighlightable + ?Sized> {
    pub hovered: Handle<T::HighlightAsset>,
    pub pressed: Handle<T::HighlightAsset>,
    pub selected: Handle<T::HighlightAsset>,
    pub opponent: Handle<T::HighlightAsset>,
}

impl<T: StackRankDiceHighlightable> FromWorld for StackRankDiceDefaultHighlighting<T> {
    fn from_world(world: &mut World) -> Self {
        T::highlight_defaults(T::materials(world))
    }
}

/// This trait makes it possible for highlighting to be generic over any type of asset.
pub trait StackRankDiceHighlightable: Default {
    /// The asset used to highlight the picked object. For a 3D mesh, this would probably be [`StandardMaterial`].
    type HighlightAsset: Asset;
    fn highlight_defaults(
        materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> StackRankDiceDefaultHighlighting<Self>;
    fn materials(world: &mut World) -> Mut<Assets<Self::HighlightAsset>> {
        world
            .get_resource_mut::<Assets<Self::HighlightAsset>>()
            .expect("Failed to get resource")
    }
}

#[derive(Default)]
pub(crate) struct StackRankDiceMaterialHighlight;
impl StackRankDiceHighlightable for StackRankDiceMaterialHighlight {
    type HighlightAsset = StandardMaterial;

    fn highlight_defaults(
        mut materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> StackRankDiceDefaultHighlighting<Self> {
        StackRankDiceDefaultHighlighting {
            hovered: materials.add(StandardMaterial {
                base_color: Color::rgb(0.85, 0.0, 0.85),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            pressed: materials.add(StandardMaterial {
                base_color: Color::rgb(0.85, 0.0, 0.85),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            selected: materials.add(StandardMaterial {
                base_color: Color::rgb(0.95, 0.0, 0.85),
                metallic: 0.01,
                reflectance: 0.0,
                ..default()
            }),
            opponent: materials.add(StandardMaterial {
                base_color: Color::rgba(0.95, 0.0, 0.0, 0.7),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
        }
    }
}

pub(crate) struct StakRankDiceHighlightablePickingPlugins;
impl PluginGroup for StakRankDiceHighlightablePickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CustomStackRankDiceHighlightPlugin(
            StackRankDiceMaterialHighlight,
        ));
    }
}

pub(crate) struct StackRankDicePickingPlugins;

impl PluginGroup for StackRankDicePickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PickingPlugin);
        group.add(InteractablePickingPlugin);
        StakRankDiceHighlightablePickingPlugins.build(group);
    }
}

fn simple_criteria(flag: bool) -> ShouldRun {
    if flag {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
#[derive(Default)]
pub struct CustomStackRankDiceHighlightPlugin<T: 'static + StackRankDiceHighlightable + Sync + Send>(
    pub T,
);

impl<T> Plugin for CustomStackRankDiceHighlightPlugin<T>
where
    T: 'static + StackRankDiceHighlightable + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<StackRankDiceDefaultHighlighting<T>>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_mesh_highlight_asset::<T::HighlightAsset>
                            .after(PickingSystem::UpdateIntersections)
                            .before(PickingSystem::Highlighting),
                    )
                    .with_system(
                        mesh_highlighting::<T>
                            .label(PickingSystem::Highlighting)
                            .before(PickingSystem::Events),
                    ),
            );
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting<T: 'static + StackRankDiceHighlightable + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_default_highlight: Res<StackRankDiceDefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut Handle<T::HighlightAsset>,
            Option<&Selection>,
            &Highlighting<T::HighlightAsset>,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
    regions: Query<(Entity, &Region)>,
    game_state: Res<GameState>,
    selected_region: Res<SelectedRegion>,
) {
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, _, mut material, selection, highlight) in interaction_query.iter_mut() {
                *material = if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &highlight.initial
                }
                .to_owned();
            }
            return;
        }
    }

    for (entity, interaction, mut material, selection, highlight) in interaction_query.iter_mut() {
        let region = regions.get(entity);

        *material = match *interaction {
            Interaction::Clicked => {
                if selected_region.entity.is_some() && selected_region.entity.unwrap() == entity {
                    &global_default_highlight.pressed
                } else if region.is_ok() && region.unwrap().1.owner != game_state.turn_of_player {
                    &highlight.initial
                } else if let Some(highlight_asset) = &highlight.pressed {
                    highlight_asset
                } else {
                    &global_default_highlight.pressed
                }
            }
            Interaction::Hovered => {
                if selected_region.entity.is_some() && selected_region.entity.unwrap() == entity {
                    &global_default_highlight.selected
                } else if selected_region.entity.is_some()
                    && region.is_ok()
                    && region
                        .unwrap()
                        .1
                        .is_opponent(selected_region.region.as_ref().unwrap())
                {
                    &global_default_highlight.opponent
                } else if region.is_ok() && region.unwrap().1.owner != game_state.turn_of_player {
                    &highlight.initial
                } else if let Some(highlight_asset) = &highlight.hovered {
                    highlight_asset
                } else {
                    &global_default_highlight.hovered
                }
            }
            Interaction::None => {
                if selection.filter(|s| s.selected()).is_some() {
                    if selected_region.entity.is_some() && selected_region.entity.unwrap() == entity
                    {
                        &global_default_highlight.selected
                    } else if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &highlight.initial
                    }
                } else {
                    &highlight.initial
                }
            }
        }
        .to_owned();
    }
}
