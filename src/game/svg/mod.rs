use bevy::prelude::*;

mod pipeline;

pub use pipeline::{SvgEntityKind, SvgInteractionState, SvgRenderCache};

pub struct SvgPlugin;

impl Plugin for SvgPlugin {
    fn build(&self, app: &mut App) {
        pipeline::build_svg_pipeline(app);
    }
}
