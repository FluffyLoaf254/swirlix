use crate::material::Material;

use glam::{Vec3, vec3};

/// The 3D sculpt.
///
/// A sparse voxel octree with associated material
/// information.
pub struct Sculpt {
	root: SculptNode,
	resolution: u32,
	palette: SculptPalette,
}

impl Sculpt {
	/// Creates a new sculpt object.
	pub fn new(resolution: u32) -> Self {
		Self {
			root: SculptNode::new(SculptNodeKind::None, 0, 1.0, vec3(0.5, 0.5, 0.5)),
			palette: SculptPalette::new(),
			resolution,
		}
	}

	/// Retrieve the set resolution.
	pub fn get_resolution(&self) -> u32 {
		self.resolution
	}

	/// Get the minimum voxel leaf node size.
	fn min_leaf_size(&self) -> f32 {
		1.0 / (self.resolution as f32)
	}

	/// Gets the raw data for the voxel buffer.
	pub fn get_voxel_buffer(&self) -> Vec<u32> {
		self.root.to_buffer()
	}

	/// Gets the raw data for the material palette buffer.
	pub fn get_material_buffer(&self) -> Vec<f32> {
		self.palette.to_buffer()
	}

	/// Subdivides space to fill the sculpt.
	pub fn subdivide(&mut self, is_filled: Box<dyn Fn(f32, Vec3) -> bool>, is_contained: Box<dyn Fn(f32, Vec3) -> bool>) {
		self.root.subdivide(0, &is_filled, &is_contained, self.min_leaf_size(), false);
		self.root.set_child_count();
	}

	/// Remove voxels from the sculpt.
	pub fn unsubdivide(&mut self, is_filled: Box<dyn Fn(f32, Vec3) -> bool>, is_contained: Box<dyn Fn(f32, Vec3) -> bool>) {
		self.root.unsubdivide(0, &is_filled, &is_contained, self.min_leaf_size());
		self.root.set_child_count();
	}
}

/// The classification of a sculpt node.
#[derive(PartialEq, Eq)]
enum SculptNodeKind {
	Leaf,
	Interior,
	None,
}

/// A node/voxel in the sparse voxel octree.
struct SculptNode {
	kind: SculptNodeKind,
	children: [Option<Box<SculptNode>>; 8],
	center: Vec3,
	size: f32,
	material: u32,
	child_count: u32,
}

impl SculptNode {
	/// Make a sculpt node with the given parameters and no children.
	fn new(kind: SculptNodeKind, material: u32, size: f32, center: Vec3) -> Self {
		Self {
			kind,
			children: [None, None, None, None, None, None, None, None],
			center,
			size,
			material,
			child_count: 0,
		}
	}

	/// Handles the sparse voxel octree subdividing modifications, recursively.
	fn subdivide(&mut self, fill: u32, is_filled: &Box<dyn Fn(f32, Vec3) -> bool>, is_contained: &Box<dyn Fn(f32, Vec3) -> bool>, min_leaf_size: f32, invert: bool) {
		if !invert && self.kind == SculptNodeKind::Leaf {
			return;
		}
		
		if self.size <= min_leaf_size || (is_contained(self.size, self.center) == !invert) {
			self.children = [None, None, None, None, None, None, None, None];
			self.kind = SculptNodeKind::Leaf;

			return;
		}

		let half_size = self.size / 2.0;
		let quarter_size = self.size / 4.0;

		let lfb = vec3(self.center.x - quarter_size, self.center.y - quarter_size, self.center.z - quarter_size);
		let rfb = vec3(self.center.x + quarter_size, self.center.y - quarter_size, self.center.z - quarter_size);
		let lbb = vec3(self.center.x - quarter_size, self.center.y + quarter_size, self.center.z - quarter_size);
		let rbb = vec3(self.center.x + quarter_size, self.center.y + quarter_size, self.center.z - quarter_size);
		let lft = vec3(self.center.x - quarter_size, self.center.y - quarter_size, self.center.z + quarter_size);
		let rft = vec3(self.center.x + quarter_size, self.center.y - quarter_size, self.center.z + quarter_size);
		let lbt = vec3(self.center.x - quarter_size, self.center.y + quarter_size, self.center.z + quarter_size);
		let rbt = vec3(self.center.x + quarter_size, self.center.y + quarter_size, self.center.z + quarter_size);

		if (is_filled(half_size, lfb) == !invert) && !self.children[0].is_some() {
			self.children[0] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, lfb)));
		};
		if (is_filled(half_size, rfb) == !invert) && !self.children[1].is_some() {
			self.children[1] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, rfb)));
		};
		if (is_filled(half_size, lbb) == !invert) && !self.children[2].is_some() {
			self.children[2] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, lbb)));
		};
		if (is_filled(half_size, rbb) == !invert) && !self.children[3].is_some() {
			self.children[3] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, rbb)));
		};
		if (is_filled(half_size, lft) == !invert) && !self.children[4].is_some() {
			self.children[4] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, lft)));
		};
		if (is_filled(half_size, rft) == !invert) && !self.children[5].is_some() {
			self.children[5] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, rft)));
		};
		if (is_filled(half_size, lbt) == !invert) && !self.children[6].is_some() {
			self.children[6] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, lbt)));
		};
		if (is_filled(half_size, rbt) == !invert) && !self.children[7].is_some() {
			self.children[7] = Some(Box::new(SculptNode::new(SculptNodeKind::None, fill, half_size, rbt)));
		};

		let mut all_leaves = true;

		for index in 0..self.children.len() {
			if let Some(ref mut child) = self.children[index] {
				child.subdivide(fill, &is_filled, &is_contained, min_leaf_size, invert);
				all_leaves = all_leaves && (child.kind == SculptNodeKind::Leaf);
			} else {
				all_leaves = false;
			}
		}

		if all_leaves {
			self.children = [None, None, None, None, None, None, None, None];

			self.kind = SculptNodeKind::Leaf;
		} else if self.children.iter().any(|child| child.is_some()) {
			self.kind = SculptNodeKind::Interior;
		}
	}

	/// Handles the sparse voxel octree unsubdividing modifications, recursively.
	fn unsubdivide(&mut self, fill: u32, is_filled: &Box<dyn Fn(f32, Vec3) -> bool>, is_contained: &Box<dyn Fn(f32, Vec3) -> bool>, min_leaf_size: f32) {
		if !is_filled(self.size, self.center) {
			return;
		}

		let mut removed_all = self.children.iter().any(|child| child.is_some());
		for index in 0..self.children.len() {
			let mut should_remove = false;
			if let Some(ref mut child) = self.children[index] {
				child.unsubdivide(fill, &is_filled, &is_contained, min_leaf_size);
				if (child.kind == SculptNodeKind::None) || is_contained(child.size, child.center) {
					should_remove = true;
				}
				removed_all = removed_all && should_remove;
			}
			if should_remove {
				self.children[index] = None;
			}
		}

		if removed_all {
			self.kind = SculptNodeKind::None;

			return;
		}

		// If it isn't a leaf, return
		if self.children.iter().any(|child| child.is_some()) {
			self.kind = SculptNodeKind::Interior;

			return;
		}

		self.subdivide(fill, &is_filled, &is_contained, min_leaf_size, true);

		if !self.children.iter().any(|child| child.is_some()) {
			self.kind = SculptNodeKind::None;
		} else {
			self.kind = SculptNodeKind::Interior;
		}
	}

	/// Set the child counts recursively.
	///
	/// The child count is needed by the buffer generation
	/// algorithm.
	fn set_child_count(&mut self) {
		self.child_count = 0;

		for index in 0..8 {
			if let Some(child) = &mut self.children[index as usize] {
				if child.kind == SculptNodeKind::Interior {
					self.child_count += 2;
				} else {
					self.child_count += 1;
				}
				child.set_child_count();
				self.child_count += child.child_count;
			}
		}
	}

	/// Convert the node and its children to the buffer format for the GPU.
	fn to_buffer(&self) -> Vec<u32> {
		let mut buffer = Vec::<u32>::new();

		buffer.push(self.to_u32());
		buffer.push(2);

		self.append_to_buffer(&mut buffer, 2);

		let length = buffer.len();
		println!("{length}");

		buffer
	}

	/// Convert a node to an integer to send to the GPU.
	fn to_u32(&self) -> u32 {
		let mut value = 0u32;

		let mut child_mask = 0;
		let mut leaf_mask = 0;
		let mut child_count = 0;

		for index in 0..8 {
			if let Some(child) = &self.children[index as usize] {
				let bit = 1u32 << index;
				if child.kind == SculptNodeKind::Leaf {
					leaf_mask |= bit;
				}
				child_mask |= bit;
				child_count += 1;
			}
		}

		if child_count == 0 {
			// a leaf node
			value = self.material;
		} else {
			// an interior node
			value |= child_mask << 8;
			value |= leaf_mask;
		}

		value
	}

	/// Handle the actual, recursive logic for generating the buffer.
	fn append_to_buffer(&self, buffer: &mut Vec<u32>, mut pointer: u32) {
		for index in 0..8 {
			if let Some(child) = &self.children[index] {
				if child.kind == SculptNodeKind::Interior {
					pointer += 2;
				} else {
					pointer += 1;
				}
			}
		}

		let mut first_child_pointer = pointer;
		for index in 0..8 {
			if let Some(child) = &self.children[index] {
				buffer.push(child.to_u32());
				if child.kind == SculptNodeKind::Interior {
					buffer.push(first_child_pointer);
				}
				first_child_pointer += child.child_count;
			}
		}

		let mut second_child_pointer = pointer;
		for index in 0..8 {
			if let Some(child) = &self.children[index] {
				child.append_to_buffer(buffer, second_child_pointer);
				second_child_pointer += child.child_count;
			}
		}
	}
}

/// The `SculptPalette` stores the materials that are used in the current sculpt.
/// They should be pruned if they are no longer in use.
struct SculptPalette {
	materials: Vec<Material>,
}

impl SculptPalette {
	/// Create a sculpt palette with default materials.
	fn new() -> Self {
		Self {
			materials: vec![Material::default()],
		}
	}

	/// Get a material reference.
	fn get(&self, index: u32) -> Option<&Material> {
		self.materials.get(index as usize)
	}

	/// Converts the palette materials to a buffer for use on the GPU.
	fn to_buffer(&self) -> Vec<f32> {
		let mut buffer = Vec::new();

		for material in &self.materials {
			buffer.extend(material.to_buffer());
		}

		buffer
	}

	/// Pushes a new material onto the palette.
	fn push(&mut self, value: Material) {
		self.materials.push(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    use crate::brush::RoundBrushTip;

    #[test]
    fn subdivide_creates_all_root_children_with_sphere_brush_at_center() {
    	let mut sculpt = Sculpt::new(32);

    	let material = Material {
    		index: 1,
    		..Default::default()
    	};

    	sculpt.subdivide(RoundBrushTip::filler(0.5, vec3(0.5, 0.5, 0.5)), RoundBrushTip::container(0.5, vec3(0.5, 0.5, 0.5)));

    	assert_eq!(sculpt.root.children.iter().filter(|child| child.is_some()).count(), 8);
    }

    #[test]
    fn simple_sculpt_node_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));
		sculpt_node.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b11111111 << 8) + (0b11111111),
			2,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    #[test]
    fn sculpt_nodes_with_different_materials_generate_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));
		sculpt_node.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 2, 0.5, vec3(0.25, 0.25, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 3, 0.5, vec3(0.75, 0.25, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 4, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 5, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 6, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 7, 0.5, vec3(0.75, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 8, 0.5, vec3(0.25, 0.75, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 9, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b11111111 << 8) + (0b11111111),
			2,

			2,
			3,
			4,
			5,
			6,
			7,
			8,
			9,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    #[test]
    fn simple_sculpt_node_missing_children_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));
		sculpt_node.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.25)))),
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			None,
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b10010101 << 8) + (0b10010101),
			2,

			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    #[test]
    fn simple_nested_sculpt_node_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));

		let mut sculpt_node_child_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.25, 0.25, 0.25));
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.375)))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b11111111 << 8) + (0b11111110),
			2,

			(0b11111111 << 8) + (0b11111111),
			11,
			1,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn multiple_nested_sculpt_node_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));

		let mut sculpt_node_child_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.25, 0.25, 0.25));
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.375)))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.75, 0.25, 0.25));
		sculpt_node_child_rfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.375, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.375)))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b11111111 << 8) + (0b11111100),
			2,
			
			(0b11111111 << 8) + (0b11111111),
			12,
			(0b11111111 << 8) + (0b11111111),
			20,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn deeply_nested_sculpt_node_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));

		let mut sculpt_node_child_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.25, 0.25, 0.25));
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.375)))),
		];

		let mut sculpt_node_nested_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.25, vec3(0.625, 0.125, 0.125));
		sculpt_node_nested_lfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.5625, 0.0625, 0.0625)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.0625, 0.0625)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.5625, 0.1875, 0.0625)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.1875, 0.0625)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.5625, 0.0625, 0.1875)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.0625, 0.1875)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.5625, 0.1875, 0.1875)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.1875, 0.1875)))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.75, 0.25, 0.25));
		sculpt_node_child_rfb.children = [
			Some(Box::new(sculpt_node_nested_lfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.375, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.375)))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b11111111 << 8) + (0b11111100),
			2,
			
			(0b11111111 << 8) + (0b11111111),
			12,
			(0b11111111 << 8) + (0b11111110),
			20,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,

			(0b11111111 << 8) + (0b11111111),
			29,
			1,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn complex_sculpt_node_generates_correct_buffer() {
		let mut sculpt_node = SculptNode::new(SculptNodeKind::Interior, 1, 1.0, vec3(0.5, 0.5, 0.5));

		let mut sculpt_node_child_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.25, 0.25, 0.25));
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.125, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.375, 0.375, 0.125)))),
			None,
			None,
			None,
			None,
		];

		let mut sculpt_node_deeply_nested_lbb = SculptNode::new(SculptNodeKind::Interior, 1, 0.125, vec3(0.5625, 0.1875, 0.0625));
		sculpt_node_deeply_nested_lbb.children = [
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.53125, 0.15625, 0.03125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.59375, 0.15625, 0.03125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.53125, 0.21875, 0.03125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.59375, 0.21875, 0.03125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.53125, 0.15625, 0.09375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.59375, 0.15625, 0.09375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.53125, 0.21875, 0.09375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.0625, vec3(0.59375, 0.21875, 0.09375)))),
		];

		let mut sculpt_node_nested_lfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.25, vec3(0.625, 0.125, 0.125));
		sculpt_node_nested_lfb.children = [
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.0625, 0.0625)))),
			Some(Box::new(sculpt_node_deeply_nested_lbb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.1875, 0.0625)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.5625, 0.0625, 0.1875)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.0625, 0.1875)))),
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.125, vec3(0.6875, 0.1875, 0.1875)))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(SculptNodeKind::Interior, 1, 0.5, vec3(0.75, 0.25, 0.25));
		sculpt_node_child_rfb.children = [
			Some(Box::new(sculpt_node_nested_lfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.125)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.625, 0.125, 0.375)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.125, 0.375)))),
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.25, vec3(0.875, 0.375, 0.375)))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.25)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.25, 0.25, 0.75)))),
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.25, 0.75)))),
			None,
			Some(Box::new(SculptNode::new(SculptNodeKind::Leaf, 1, 0.5, vec3(0.75, 0.75, 0.75)))),
		];

		let expected = vec![
			(0b10111111 << 8) + (0b10111100),
			2,
			
			(0b00001111 << 8) + (0b00001111),
			11,
			(0b10111111 << 8) + (0b10111110),
			15,
			1,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,

			(0b10111100 << 8) + (0b10111000),
			23,
			1,
			1,
			1,
			1,
			1,
			1,

			1,
			(0b11111111 << 8) + (0b11111111),
			30,
			1,
			1,
			1,
			1,

			1,
			1,
			1,
			1,
			1,
			1,
			1,
			1,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }
}
