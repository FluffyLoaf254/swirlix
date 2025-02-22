use crate::brush::{SquareBrushTip, RoundBrushTip};
use crate::brush::Brush;
use crate::sculpt::Sculpt;

/// The owner of sculpt-related stuff.
///
/// Holds the document information as well as
/// session configuration.
pub struct Editor {
	sculpt: Sculpt,
	current_brush: usize,
	brushes: Vec<Brush>,
}

impl Default for Editor {
	/// A default editor/document.
	fn default() -> Self {
		Editor {
			sculpt: Sculpt::new(512),
			current_brush: 0,
			brushes: vec![
				Brush::new("Round Brush".to_owned(), Box::new(RoundBrushTip::new())),
				Brush::new("Square Brush".to_owned(), Box::new(SquareBrushTip::new())),
			],
		}
	}
}

impl Editor {
	/// Get the density of the sculpt in voxels per axis.
	pub fn get_sculpt_resolution(&self) -> u32 {
		self.sculpt.get_resolution()
	}

	/// Set the brush type.
	pub fn set_brush(&mut self, brush: usize) {
		self.current_brush = brush.clamp(0, self.brushes.len());
	}

	/// Get the buffer for the sculpted voxels.
	pub fn get_voxel_buffer(&self) -> Vec<u32> {
		self.sculpt.get_voxel_buffer()
	}

	/// Get the buffer for the used materials.
	pub fn get_material_buffer(&self) -> Vec<f32> {
		self.sculpt.get_material_buffer()
	}

	/// Draw additively on the sculpt.
	pub fn add(&mut self, x: f32, y: f32) {
		self.brushes[self.current_brush].add(&mut self.sculpt, x, y);
	}

	/// Draw subtractively on the sculpt.
	pub fn remove(&mut self, x: f32, y: f32) {
		self.brushes[self.current_brush].remove(&mut self.sculpt, x, y);
	}
}
