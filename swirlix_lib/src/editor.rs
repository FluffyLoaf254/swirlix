use crate::brush::RoundBrushTip;
use crate::brush::Brush;
use crate::sculpt::Sculpt;

/// The owner of sculpt-related stuff.
///
/// Holds the document information as well as
/// session configuration.
pub struct Editor {
	sculpt: Sculpt,
	brush: Brush<RoundBrushTip>,
}

impl Default for Editor {
	/// A default editor/document.
	fn default() -> Self {
		Editor {
			sculpt: Sculpt::new(),
			brush: Brush::new("Round Brush".to_owned(), RoundBrushTip::new()),
		}
	}
}

impl Editor {
	/// Get the buffer for the sculpted voxels.
	pub fn get_voxel_buffer(&self) -> Vec<u32> {
		self.sculpt.get_voxel_buffer()
	}

	/// Draw additively on the sculpt.
	pub fn add(&mut self, x: f32, y: f32) {
		self.brush.add(&mut self.sculpt, x, y);
	}

	/// Draw subtractively on the sculpt.
	pub fn remove(&mut self, x: f32, y: f32) {
		self.brush.remove(&mut self.sculpt, x, y);
	}
}
