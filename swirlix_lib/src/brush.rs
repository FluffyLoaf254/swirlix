use crate::material::Material;
use crate::util::Point;
use crate::sculpt::Sculpt;

pub struct Brush<B: Draw> {
	pub name: String,
	tip: B,
	size: f32,
	material: Material,
}

impl<B: Draw> Brush<B> {
	pub fn new(name: String, tip: B) -> Self {
		Self {
			name,
			tip,
			size: 0.5,
		    material: Material {
		    	color: [255, 0, 0, 255],
		    	roughness: 128,
		    	metallic: 0,
		    },
		}
	}

	pub fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.add(sculpt, x, y, self.size, self.material);
	}

	pub fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32) {
		self.tip.remove(sculpt, x, y, self.size, self.material);
	}
}

pub trait Draw {
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material);
	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material);
}

pub struct RoundBrushTip {}

impl RoundBrushTip {
	pub fn new() -> Self {
		Self {}
	}
}

impl Draw for RoundBrushTip {
	fn add(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material) {
		let brush_position = Point {
			x,
			y,
			z: 0.0,
		};
		let brush_size = size;
		sculpt.subdivide(
			material,
			&|size: f32, center: Point| {
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
			}
		);
	}

	fn remove(&self, sculpt: &mut Sculpt, x: f32, y: f32, size: f32, material: Material) {
		todo!()
	}
}
