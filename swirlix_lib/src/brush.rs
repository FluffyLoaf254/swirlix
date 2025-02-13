use crate::material::Material;
use crate::util::Point;
use crate::sculpt::Sculpt;

/// A brush for sculpting.
pub struct Brush<B: Draw> {
	pub name: String,
	tip: B,
	size: f32,
	material: Material,
}

impl<B: Draw> Brush<B> {
	/// Create a new brush with a tip/effector.
	pub fn new(name: String, tip: B) -> Self {
		Self {
			name,
			tip,
			size: 0.25,
		    material: Material {
		    	color: [255, 0, 0, 255],
		    	roughness: 128,
		    	metallic: 0,
		    },
		}
	}

	/// Sculpt by adding geometry.
	pub fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.add(sculpt, x, y, self.size, self.material);
	}

    /// Sculpt by removing geometry.
	pub fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.remove(sculpt, x, y, self.size, self.material);
	}
}

pub trait Draw {
	/// Sculpt by adding geometry.
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material);

	/// Sculpt by removing geometry.
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material);
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
			let point_1 = Point {
				x: center.x - half_size,
				y: center.y - half_size,
				z: center.z - half_size,
			};
			let point_2 = Point {
				x: center.x + half_size,
				y: center.y + half_size,
				z: center.z + half_size,
			};
			let mut dist_squared = brush_size.powi(2);
			if brush_position.x < point_1.x {
				dist_squared -= (brush_position.x - point_1.x).powi(2);
			} else if brush_position.x > point_2.x {
				dist_squared -= (brush_position.x - point_2.x).powi(2);
			}
			if brush_position.y < point_1.y {
				dist_squared -= (brush_position.y - point_1.y).powi(2);
			} else if brush_position.y > point_2.y {
				dist_squared -= (brush_position.y - point_2.y).powi(2);
			}
			if brush_position.z < point_1.z {
				dist_squared -= (brush_position.z - point_1.z).powi(2);
			} else if brush_position.z > point_2.z {
				dist_squared -= (brush_position.z - point_2.z).powi(2);
			}

			dist_squared > 0.0
		})
	}
}

impl Draw for RoundBrushTip {
	/// Sculpt by adding geometry.
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material) {
		let brush_position = Point {
			x,
			y,
			z: 0.5,
		};
		let brush_size = size;
		sculpt.subdivide(material, RoundBrushTip::filler(brush_size, brush_position));
	}

	/// Sculpt by removing geometry.
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material) {
		todo!()
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
    	assert!(filler(0.1, Point { x: 0.75, y: 0.75, z: 0.75 }))
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
}
