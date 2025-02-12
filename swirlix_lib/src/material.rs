/// A material to encode surface attributes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Material {
	pub color: [u8; 4],
	pub roughness: u8,
	pub metallic: u8,
}

impl Material {
	/// Convert the material to the buffer data structure.
	pub fn to_buffer(&self) -> [u8; 6] {
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
