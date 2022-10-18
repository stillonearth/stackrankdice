use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_picking::{
    highlight::get_initial_mesh_highlight_asset, DefaultHighlighting, Highlightable, Highlighting,
    InteractablePickingPlugin, PausedForBlockers, PickingPlugin, PickingPluginsState,
    PickingSystem, Selection,
};

use crate::game::{GameState, Region};

#[derive(Default)]
pub(crate) struct StakRankDiceMaterialHighlight;
impl Highlightable for StakRankDiceMaterialHighlight {
    type HighlightAsset = StandardMaterial;

    fn highlight_defaults(
        mut materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(StandardMaterial {
                base_color: Color::rgb(0.35, 0.35, 0.35).into(),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            pressed: materials.add(StandardMaterial {
                base_color: Color::rgb(0.35, 0.75, 0.35).into(),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            selected: materials.add(StandardMaterial {
                base_color: Color::rgb(0.35, 0.35, 0.75).into(),
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
            StakRankDiceMaterialHighlight,
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

// Override

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
pub struct CustomStackRankDiceHighlightPlugin<T: 'static + Highlightable + Sync + Send>(pub T);

impl<T> Plugin for CustomStackRankDiceHighlightPlugin<T>
where
    T: 'static + Highlightable + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultHighlighting<T>>()
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
pub fn mesh_highlighting<T: 'static + Highlightable + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_default_highlight: Res<DefaultHighlighting<T>>,
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
                if let Some(highlight_asset) = &highlight.pressed {
                    highlight_asset
                } else {
                    &global_default_highlight.pressed
                }
            }
            Interaction::Hovered => {
                if region.is_ok() && region.unwrap().1.owner != game_state.turn {
                    &highlight.initial
                } else if let Some(highlight_asset) = &highlight.hovered {
                    highlight_asset
                } else {
                    &global_default_highlight.hovered
                }
            }
            Interaction::None => {
                if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &highlight.initial
                }
            }
        }
        .to_owned();
    }
}
