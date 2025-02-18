use crate::util::Point;
use crate::material::Material;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Weak, Rc};

/// The 3D sculpt.
///
/// A sparse voxel octree with associated material
/// information.
pub struct Sculpt {
	root: SculptNode,
	resolution: u32,
	palette: Rc<RefCell<SculptPalette>>,
}

impl Sculpt {
	/// Creates a new sculpt object.
	pub fn new(resolution: u32) -> Self {
		let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));
		Self {
			root: SculptNode::new(Rc::downgrade(&palette_ref), material, 1.0, Point { x: 0.5, y: 0.5, z: 0.5 }),
			palette: palette_ref,
			resolution: resolution,
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
	pub fn get_material_buffer(&self) -> Vec<u8> {
		self.palette.borrow().to_buffer()
	}

	/// Subdivides space to fill the sculpt.
	pub fn subdivide(&mut self, fill: Material, is_filled: Box<dyn Fn(f32, Point) -> bool>, is_contained: Box<dyn Fn(f32, Point) -> bool>) {
		self.palette.borrow_mut().push(fill);
		let material = self.palette.borrow().get(fill);
		self.root.subdivide(material.clone(), &is_filled, &is_contained, self.min_leaf_size(), false);
		self.root.set_child_count();
	}

	/// Subdivides space to fill the sculpt.
	pub fn unsubdivide(&mut self, fill: Material, is_filled: Box<dyn Fn(f32, Point) -> bool>, is_contained: Box<dyn Fn(f32, Point) -> bool>) {
		self.palette.borrow_mut().push(fill);
		let material = self.palette.borrow().get(fill);
		self.root.unsubdivide(material.clone(), &is_filled, &is_contained, self.min_leaf_size());
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
	palette: Weak<RefCell<SculptPalette>>,
	material: Rc<Material>,
	children: [Option<Box<SculptNode>>; 8],
	center: Point,
	size: f32,
	child_count: u32,
	kind: SculptNodeKind,
}

impl SculptNode {
	/// Make a sculpt node with the given parameters and no children.
	fn new(palette: Weak<RefCell<SculptPalette>>, material: Rc<Material>, size: f32, center: Point) -> Self {
		Self {
			palette,
			material,
			children: [None, None, None, None, None, None, None, None],
			size,
			center,
			child_count: 0,
			kind: SculptNodeKind::None,
		}
	}

	/// Handles the sparse voxel octree subdividing modifications, recursively.
	///
	/// Returns whether or not the result is a leaf.
	fn subdivide(&mut self, fill: Rc<Material>, is_filled: &Box<dyn Fn(f32, Point) -> bool>, is_contained: &Box<dyn Fn(f32, Point) -> bool>, min_leaf_size: f32, invert: bool) {
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

		let lfb = Point {
			x: self.center.x - quarter_size,
			y: self.center.y - quarter_size,
			z: self.center.z - quarter_size,
		};

		let rfb = Point {
			x: self.center.x + quarter_size,
			y: self.center.y - quarter_size,
			z: self.center.z - quarter_size,
		};

		let lbb = Point {
			x: self.center.x - quarter_size,
			y: self.center.y + quarter_size,
			z: self.center.z - quarter_size,
		};

		let rbb = Point {
			x: self.center.x + quarter_size,
			y: self.center.y + quarter_size,
			z: self.center.z - quarter_size,
		};

		let lft = Point {
			x: self.center.x - quarter_size,
			y: self.center.y - quarter_size,
			z: self.center.z + quarter_size,
		};

		let rft = Point {
			x: self.center.x + quarter_size,
			y: self.center.y - quarter_size,
			z: self.center.z + quarter_size,
		};

		let lbt = Point {
			x: self.center.x - quarter_size,
			y: self.center.y + quarter_size,
			z: self.center.z + quarter_size,
		};

		let rbt = Point {
			x: self.center.x + quarter_size,
			y: self.center.y + quarter_size,
			z: self.center.z + quarter_size,
		};

		if (is_filled(half_size, lfb) == !invert) && !self.children[0].is_some() {
			self.children[0] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lfb)));
		};
		if (is_filled(half_size, rfb) == !invert) && !self.children[1].is_some() {
			self.children[1] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rfb)));
		};
		if (is_filled(half_size, lbb) == !invert) && !self.children[2].is_some() {
			self.children[2] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lbb)));
		};
		if (is_filled(half_size, rbb) == !invert) && !self.children[3].is_some() {
			self.children[3] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rbb)));
		};
		if (is_filled(half_size, lft) == !invert) && !self.children[4].is_some() {
			self.children[4] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lft)));
		};
		if (is_filled(half_size, rft) == !invert) && !self.children[5].is_some() {
			self.children[5] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rft)));
		};
		if (is_filled(half_size, lbt) == !invert) && !self.children[6].is_some() {
			self.children[6] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lbt)));
		};
		if (is_filled(half_size, rbt) == !invert) && !self.children[7].is_some() {
			self.children[7] = Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rbt)));
		};

		let mut all_leaves = true;

		for index in 0..self.children.len() {
			if let Some(ref mut child) = self.children[index] {
				child.subdivide(fill.clone(), &is_filled, &is_contained, min_leaf_size, invert);
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
	///
	/// Returns whether or not the result is gone.
	fn unsubdivide(&mut self, fill: Rc<Material>, is_filled: &Box<dyn Fn(f32, Point) -> bool>, is_contained: &Box<dyn Fn(f32, Point) -> bool>, min_leaf_size: f32) {
		if !is_filled(self.size, self.center) {
			return;
		}

		let mut removed_all = self.children.iter().any(|child| child.is_some());
		for index in 0..self.children.len() {
			let mut should_remove = false;
			if let Some(ref mut child) = self.children[index] {
				child.unsubdivide(fill.clone(), &is_filled, &is_contained, min_leaf_size);
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

		self.subdivide(fill.clone(), &is_filled, &is_contained, min_leaf_size, true);

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
				self.child_count += 1;
				child.set_child_count();
				self.child_count += child.child_count;
			}
		}
	}

	/// Convert the node and its children to the buffer format for the GPU.
	fn to_buffer(&self) -> Vec<u32> {
		let mut buffer = Vec::<u32>::new();

		let root = self.to_u32(1);

		buffer.push(root.0);
		buffer.push(root.1);

		self.append_to_buffer(&mut buffer, 1);

		let length = buffer.len();
		println!("{length}");

		buffer
	}

	/// Convert a node to an integer to send to the GPU.
	fn to_u32(&self, pointer: u32) -> (u32, u32) {
		let mut value = 0u32;

		let mut child_mask = 0;
		let mut leaf_mask = 0;
		let mut child_count = 0;

		for index in 0..8 {
			if let Some(child) = &self.children[index as usize] {
				let bit = 1u32 << index;
				if !child.children.iter().any(|s| s.is_some()) {
					leaf_mask |= bit;
				}
				child_mask |= bit;
				child_count += 1;
			}
		}

		if child_count == 0 {
			// a leaf node
			value = 0;
			// value |= self.palette.upgrade().unwrap().borrow().index(*self.material);
		} else {
			// an interior node
			// value |= pointer << 16;
			value |= child_mask << 8;
			value |= leaf_mask;
		}

		(value, pointer * 2)
	}

	/// Handle the actual, recursive logic for generating the buffer.
	fn append_to_buffer(&self, buffer: &mut Vec<u32>, mut pointer: u32) {
		for index in 0..8 {
			if let Some(child) = &self.children[index] {
				pointer += 1;
			}
		}

		let mut first_child_pointer = pointer;
		for index in 0..8 {
			if let Some(child) = &self.children[index] {
				let child_u32 = child.to_u32(first_child_pointer);
				buffer.push(child_u32.0);
				buffer.push(child_u32.1);
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
	materials: Vec<Rc<Material>>,
	set: HashSet<Material>,
}

impl SculptPalette {
	/// Create a sculpt palette with default materials.
	fn new() -> Self {
		let material = Material {
			color: [0, 0, 0, 255],
			roughness: 128,
			metallic: 0,
		};

		Self {
			materials: vec![
				Rc::new(material)
			],
			set: HashSet::from([material]),
		}
	}

	/// Gets the first material reference.
	fn first(&self) -> Rc<Material> {
		self.materials.first().unwrap().clone()
	}

	/// Get a material reference.
	fn get(&self, material: Material) -> Rc<Material> {
		self.materials.iter().find(|&reference| **reference == material).unwrap().clone()
	}

	/// Gets the index to use with the GPU buffer.
	fn index(&self, material: Material) -> u32 {
		self.materials.iter().position(|reference| **reference == material).unwrap() as u32
	}

	/// Converts the palette materials to a buffer for use on the GPU.
	fn to_buffer(&self) -> Vec<u8> {
		let mut buffer = Vec::new();

		for material in &self.materials {
			buffer.extend(material.to_buffer());
		}

		buffer
	}

	/// Prunes unused materials and re-indexes.
	fn prune(&mut self) {
		for index in 0..self.materials.len() {
			if Rc::strong_count(&self.materials[index]) <= 1 {
				self.set.remove(&(*self.materials[index]));
				self.materials.remove(index);
			}
		}
	}

	/// Pushes a new material onto the palette.
	fn push(&mut self, value: Material) {
		if !self.set.insert(value) {
			return;
		}

		self.materials.push(Rc::new(value));
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    use crate::brush::RoundBrushTip;

    #[test]
    fn subdivide_creates_all_root_children_with_sphere_brush_at_center() {
    	let mut sculpt = Sculpt::new(128);

    	let material = Material {
    		color: [255, 0, 0, 255],
    		roughness: 128,
    		metallic: 0,
    	};

    	sculpt.subdivide(material, RoundBrushTip::filler(0.5, Point { x: 0.5, y: 0.5, z: 0.5 }), RoundBrushTip::container(0.5, Point { x: 0.5, y: 0.5, z: 0.5 }));

    	assert_eq!(sculpt.root.children.iter().filter(|child| child.is_some()).count(), 8);
    }

    #[test]
    fn simple_sculpt_node_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });
		sculpt_node.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b11111111 << 8) + (0b11111111),

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    #[test]
    fn simple_sculpt_node_missing_children_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });
		sculpt_node.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 }))),
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			None,
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b10010101 << 8) + (0b10010101),

			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    #[test]
    fn simple_nested_sculpt_node_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });

		let mut sculpt_node_child_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 });
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.375 }))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b11111111 << 8) + (0b11111110),

			(9 << 16) + (0b11111111 << 8) + (0b11111111),
			0,
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn multiple_nested_sculpt_node_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });

		let mut sculpt_node_child_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 });
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.375 }))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.25 });
		sculpt_node_child_rfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.375, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.375 }))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b11111111 << 8) + (0b11111100),
			
			(9 << 16) + (0b11111111 << 8) + (0b11111111),
			(17 << 16) + (0b11111111 << 8) + (0b11111111),
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn deeply_nested_sculpt_node_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });

		let mut sculpt_node_child_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 });
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.375 }))),
		];

		let mut sculpt_node_nested_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.125 });
		sculpt_node_nested_lfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.0625, z: 0.0625 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.0625, z: 0.0625 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.1875, z: 0.0625 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.1875, z: 0.0625 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.0625, z: 0.1875 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.0625, z: 0.1875 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.1875, z: 0.1875 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.1875, z: 0.1875 }))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.25 });
		sculpt_node_child_rfb.children = [
			Some(Box::new(sculpt_node_nested_lfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.375, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.375 }))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b11111111 << 8) + (0b11111100),
			
			(9 << 16) + (0b11111111 << 8) + (0b11111111),
			(17 << 16) + (0b11111111 << 8) + (0b11111110),
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,

			(25 << 16) + (0b11111111 << 8) + (0b11111111),
			0,
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }

    fn complex_sculpt_node_generates_correct_buffer() {
    	let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));

		let mut sculpt_node = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 1.0, Point { x: 0.5, y: 0.5, z: 0.5 });

		let mut sculpt_node_child_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.25 });
		sculpt_node_child_lfb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.125, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.375, y: 0.375, z: 0.125 }))),
			None,
			None,
			None,
			None,
		];

		let mut sculpt_node_deeply_nested_lbb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.1875, z: 0.0625 });
		sculpt_node_deeply_nested_lbb.children = [
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.53125, y: 0.15625, z: 0.03125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.59375, y: 0.15625, z: 0.03125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.53125, y: 0.21875, z: 0.03125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.59375, y: 0.21875, z: 0.03125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.53125, y: 0.15625, z: 0.09375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.59375, y: 0.15625, z: 0.09375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.53125, y: 0.21875, z: 0.09375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.0625, Point { x: 0.59375, y: 0.21875, z: 0.09375 }))),
		];

		let mut sculpt_node_nested_lfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.125 });
		sculpt_node_nested_lfb.children = [
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.0625, z: 0.0625 }))),
			Some(Box::new(sculpt_node_deeply_nested_lbb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.1875, z: 0.0625 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.5625, y: 0.0625, z: 0.1875 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.0625, z: 0.1875 }))),
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.125, Point { x: 0.6875, y: 0.1875, z: 0.1875 }))),
		];

		let mut sculpt_node_child_rfb = SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.25 });
		sculpt_node_child_rfb.children = [
			Some(Box::new(sculpt_node_nested_lfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.125 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.625, y: 0.125, z: 0.375 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.125, z: 0.375 }))),
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.25, Point { x: 0.875, y: 0.375, z: 0.375 }))),
		];

		sculpt_node.children = [
			Some(Box::new(sculpt_node_child_lfb)),
			Some(Box::new(sculpt_node_child_rfb)),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.25 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.25, y: 0.25, z: 0.75 }))),
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.25, z: 0.75 }))),
			None,
			Some(Box::new(SculptNode::new(Rc::downgrade(&palette_ref), material.clone(), 0.5, Point { x: 0.75, y: 0.75, z: 0.75 }))),
		];

		let expected = vec![
			(1 << 16) + (0b10111111 << 8) + (0b10111100),
			
			(8 << 16) + (0b00001111 << 8) + (0b00001111),
			(12 << 16) + (0b10111111 << 8) + (0b10111110),
			0,
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,

			(19 << 16) + (0b10111100 << 8) + (0b10111000),
			0,
			0,
			0,
			0,
			0,
			0,

			0,
			(25 << 16) + (0b11111111 << 8) + (0b11111111),
			0,
			0,
			0,
			0,

			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
		];

    	assert_eq!(sculpt_node.to_buffer(), expected);
    }
}
