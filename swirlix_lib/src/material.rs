use std::hash::{Hash, Hasher};

/// A material to encode surface attributes.
#[derive(Clone, Copy)]
pub struct Material {
	pub index: u32,
	pub color: [f32; 4],
	pub roughness: f32,
	pub metallic: f32,
}

impl Material {
	/// Convert the material to the buffer data structure.
	pub fn to_buffer(&self) -> [f32; 6] {
		[
			self.color[0],
			self.color[1],
			self.color[2],
			self.color[3],
			self.roughness,
			self.metallic,
		]
	}
}

impl Default for Material {
	/// The default material is white with standard roughness.
	fn default() -> Self {
		Self  {
			index: 0,
			color: [0.8, 0.8, 0.8, 1.0],
			roughness: 0.5,
			metallic: 0.0,
		}
	}
}

impl PartialEq for Material {
	/// Materials are compared based on index.
	fn eq(&self, other: &Self) -> bool {
		other.index == self.index
	}
}

impl Eq for Material {}

impl Hash for Material {
	/// Materials are hashed based on index.
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.index.hash(state);
	}
}
