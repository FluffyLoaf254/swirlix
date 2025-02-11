use crate::document::Material;
use crate::document::Document;

pub struct Brush<B: Draw> {
	pub name: String,
	tip: B,
	size: f32,
	color: [f32; 4],
	roughness: f32,
	metallic: f32,
}

impl<B: Draw> Brush<B> {
	pub fn new(name: String, tip: B) -> Self {
		Self {
			name,
			tip,
			size: 10.0,
		    color: [1.0, 0.0, 0.0, 1.0],
			roughness: 0.5,
			metallic: 0.0,
		}
	}

	pub fn add(&self, document: &mut Document, x: f32, y: f32) {
		self.tip.add(document, x, y, self.size, self.color, self.roughness, self.metallic);
	}

	pub fn remove(&self, document: &mut Document, x: f32, y: f32) {
		self.tip.remove(document, x, y, self.size, self.color, self.roughness, self.metallic);
	}
}

pub trait Draw {
	fn add(&self, document: &mut Document, x: f32, y: f32, size: f32, color: [f32; 4], roughness: f32, metallic: f32);
	fn remove(&self, document: &mut Document, x: f32, y: f32, size: f32, color: [f32; 4], roughness: f32, metallic: f32);
}

pub struct RoundBrush {}

impl RoundBrush {
	pub fn new() -> Self {
		Self {}
	}
}

impl Draw for RoundBrush {
	fn add(&self, document: &mut Document, x: f32, y: f32, size: f32, color: [f32; 4], roughness: f32, metallic: f32) {
		let mouse_x = x;
		let mouse_y = y;
		let brush_size = size * 0.01;
		document.subdivide(
			Material::new(color, roughness, metallic),
			&|size: f32, x: f32, y: f32, z: f32|
			x + size / 2.0 < mouse_x + brush_size
			&& x - size / 2.0 > mouse_x - brush_size
			&& y + size / 2.0 < mouse_y + brush_size
			&& y - size / 2.0 > mouse_y - brush_size
			&& z + size / 2.0 < brush_size
			&& z - size / 2.0 > brush_size
		);
	}

	fn remove(&self, document: &mut Document, x: f32, y: f32, size: f32, color: [f32; 4], roughness: f32, metallic: f32) {
		todo!()
	}
}
