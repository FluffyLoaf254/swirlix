use crate::util::Point;
use crate::sculpt::Sculpt;

/// A brush for sculpting.
pub struct Brush {
	pub name: String,
	tip: Box<dyn Draw>,
	size: f32,
}

impl Brush {
	/// Create a new brush with a tip/effector.
	pub fn new(name: String, tip: Box<dyn Draw>) -> Self {
		Self {
			name,
			tip,
			size: 0.125,
		}
	}

	/// Sculpt by adding geometry.
	pub fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.add(sculpt, x, y, self.size);
	}

    /// Sculpt by removing geometry.
	pub fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.remove(sculpt, x, y, self.size);
	}
}

pub trait Draw {
	/// Sculpt by adding geometry.
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32);

	/// Sculpt by removing geometry.
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32);
}

/// A brush tip for drawing spherical shapes.
pub struct RoundBrushTip {}

impl RoundBrushTip {
	/// Create a new round brush tip/effector.
	pub fn new() -> Self {
		Self {}
	}

	/// Function for implicitly defining a spherical shape for the brush.
	pub fn filler(brush_size: f32, brush_position: Point) -> Box<dyn Fn (f32, Point) -> bool> {
		Box::new(move |size: f32, center: Point| {
			let half_size = size / 2.0;
			let low_point = Point {
				x: center.x - half_size,
				y: center.y - half_size,
				z: center.z - half_size,
			};
			let high_point = Point {
				x: center.x + half_size,
				y: center.y + half_size,
				z: center.z + half_size,
			};
			let mut dist_squared = brush_size.powi(2);
			if brush_position.x < low_point.x {
				dist_squared -= (brush_position.x - low_point.x).powi(2);
			} else if brush_position.x > high_point.x {
				dist_squared -= (brush_position.x - high_point.x).powi(2);
			}
			if brush_position.y < low_point.y {
				dist_squared -= (brush_position.y - low_point.y).powi(2);
			} else if brush_position.y > high_point.y {
				dist_squared -= (brush_position.y - high_point.y).powi(2);
			}
			if brush_position.z < low_point.z {
				dist_squared -= (brush_position.z - low_point.z).powi(2);
			} else if brush_position.z > high_point.z {
				dist_squared -= (brush_position.z - high_point.z).powi(2);
			}

			dist_squared >= 0.0
		})
	}

	/// Function for determining interior leaf nodes for a sphere.
	pub fn container(brush_size: f32, brush_position: Point) -> Box<dyn Fn (f32, Point) -> bool> {
		Box::new(move |size: f32, center: Point| {
			let half_size = size / 2.0;
			let low_point = Point {
				x: center.x - half_size,
				y: center.y - half_size,
				z: center.z - half_size,
			};
			let high_point = Point {
				x: center.x + half_size,
				y: center.y + half_size,
				z: center.z + half_size,
			};
			let mut dist_squared = brush_size.powi(2);
			if brush_position.x > center.x {
				dist_squared -= (brush_position.x - low_point.x).powi(2);
			} else {
				dist_squared -= (brush_position.x - high_point.x).powi(2);
			}
			if brush_position.y > center.y {
				dist_squared -= (brush_position.y - low_point.y).powi(2);
			} else {
				dist_squared -= (brush_position.y - high_point.y).powi(2);
			}
			if brush_position.z > center.z {
				dist_squared -= (brush_position.z - low_point.z).powi(2);
			} else {
				dist_squared -= (brush_position.z - high_point.z).powi(2);
			}

			dist_squared > 0.0
		})
	}
}

impl Draw for RoundBrushTip {
	/// Sculpt by adding geometry.
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32) {
		let brush_position = Point {
			x,
			y,
			z: 0.5,
		};
		let brush_size = size;
		sculpt.subdivide(
			RoundBrushTip::filler(brush_size, brush_position),
			RoundBrushTip::container(brush_size, brush_position)
		);
	}

	/// Sculpt by removing geometry.
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32) {
		let brush_position = Point {
			x,
			y,
			z: 0.5,
		};
		let brush_size = size;
		sculpt.unsubdivide(
			RoundBrushTip::filler(brush_size, brush_position),
			RoundBrushTip::container(brush_size, brush_position)
		);
	}
}

/// A brush tip for drawing cubical shapes.
pub struct SquareBrushTip {}

impl SquareBrushTip {
	/// Create a new square brush tip/effector.
	pub fn new() -> Self {
		Self {}
	}

	/// Function for implicitly defining a cubical shape for the brush.
	pub fn filler(brush_size: f32, brush_position: Point) -> Box<dyn Fn (f32, Point) -> bool> {
		Box::new(move |size: f32, center: Point| {
			let half_size = size / 2.0;
			let low_point = Point {
				x: center.x - half_size,
				y: center.y - half_size,
				z: center.z - half_size,
			};
			let high_point = Point {
				x: center.x + half_size,
				y: center.y + half_size,
				z: center.z + half_size,
			};
			

			let x_in_range = (brush_position.x - brush_size < low_point.x && brush_position.x + brush_size > low_point.x)
				|| (brush_position.x - brush_size < high_point.x && brush_position.x + brush_size > high_point.x)
				|| (brush_position.x - brush_size > low_point.x && brush_position.x + brush_size < high_point.x);
			let y_in_range = (brush_position.y - brush_size < low_point.y && brush_position.y + brush_size > low_point.y)
				|| (brush_position.y - brush_size < high_point.y && brush_position.y + brush_size > high_point.y)
				|| (brush_position.y - brush_size > low_point.y && brush_position.y + brush_size < high_point.y);
			let z_in_range = (brush_position.z - brush_size < low_point.z && brush_position.z + brush_size > low_point.z)
				|| (brush_position.z - brush_size < high_point.z && brush_position.z + brush_size > high_point.z)
				|| (brush_position.z - brush_size > low_point.z && brush_position.z + brush_size < high_point.z);

			x_in_range && y_in_range && z_in_range
		})
	}

	/// Function for determining interior leaf nodes for a cube.
	pub fn container(brush_size: f32, brush_position: Point) -> Box<dyn Fn (f32, Point) -> bool> {
		Box::new(move |size: f32, center: Point| {
			let half_size = size / 2.0;
			let low_point = Point {
				x: center.x - half_size,
				y: center.y - half_size,
				z: center.z - half_size,
			};
			let high_point = Point {
				x: center.x + half_size,
				y: center.y + half_size,
				z: center.z + half_size,
			};
			

			let x_in_range = (brush_position.x - brush_size < low_point.x && brush_position.x + brush_size > low_point.x)
				&& (brush_position.x - brush_size < high_point.x && brush_position.x + brush_size > high_point.x);
			let y_in_range = (brush_position.y - brush_size < low_point.y && brush_position.y + brush_size > low_point.y)
				&& (brush_position.y - brush_size < high_point.y && brush_position.y + brush_size > high_point.y);
			let z_in_range = (brush_position.z - brush_size < low_point.z && brush_position.z + brush_size > low_point.z)
				&& (brush_position.z - brush_size < high_point.z && brush_position.z + brush_size > high_point.z);

			x_in_range && y_in_range && z_in_range
		})
	}
}

impl Draw for SquareBrushTip {
	/// Sculpt by adding geometry.
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32) {
		let brush_position = Point {
			x,
			y,
			z: 0.5,
		};
		let brush_size = size;
		sculpt.subdivide(
			SquareBrushTip::filler(brush_size, brush_position),
			SquareBrushTip::container(brush_size, brush_position)
		);
	}

	/// Sculpt by removing geometry.
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32) {
		let brush_position = Point {
			x,
			y,
			z: 0.5,
		};
		let brush_size = size;
		sculpt.unsubdivide(
			SquareBrushTip::filler(brush_size, brush_position),
			SquareBrushTip::container(brush_size, brush_position)
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn round_brush_filler_contains_small_center_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(filler(0.25, Point { x: 0.5, y: 0.5, z: 0.5 }))
    }

    #[test]
    fn round_brush_filler_contains_large_center_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(filler(1.0, Point { x: 0.5, y: 0.5, z: 0.5 }))
    }

    #[test]
    fn round_brush_filler_contains_small_offcenter_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(filler(0.05, Point { x: 0.75, y: 0.75, z: 0.75 }))
    }

    #[test]
    fn round_brush_filler_contains_large_offcenter_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(filler(1.0, Point { x: 0.75, y: 0.75, z: 0.75 }))
    }

    #[test]
    fn round_brush_filler_contains_large_far_off_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(filler(4.0, Point { x: 2.0, y: 2.0, z: 2.0 }))
    }

    #[test]
    fn round_brush_filler_does_not_contains_far_off_point() {
    	let filler = RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(!filler(0.25, Point { x: 2.0, y: 2.0, z: 2.0 }))
    }

    #[test]
    fn round_brush_container_contains_small_center_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(container(0.25, Point { x: 0.5, y: 0.5, z: 0.5 }))
    }

    #[test]
    fn round_brush_container_does_not_contain_large_center_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(!container(1.0, Point { x: 0.5, y: 0.5, z: 0.5 }))
    }

    #[test]
    fn round_brush_container_contains_small_offcenter_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(container(0.05, Point { x: 0.75, y: 0.75, z: 0.75 }))
    }

    #[test]
    fn round_brush_container_does_not_contain_large_offcenter_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(!container(1.0, Point { x: 0.75, y: 0.75, z: 0.75 }))
    }

    #[test]
    fn round_brush_container_does_not_contain_large_far_off_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(!container(4.0, Point { x: 2.0, y: 2.0, z: 2.0 }))
    }

    #[test]
    fn round_brush_container_does_not_contains_far_off_point() {
    	let container = RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 });
    	assert!(!container(0.25, Point { x: 2.0, y: 2.0, z: 2.0 }))
    }
}
