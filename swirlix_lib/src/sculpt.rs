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
	palette: Rc<RefCell<SculptPalette>>,
}

impl Sculpt {
	/// Creates a new sculpt object.
	pub fn new() -> Self {
		let palette = SculptPalette::new();
		let material = palette.first();
		let palette_ref = Rc::new(RefCell::new(palette));
		Self {
			root: SculptNode::new(Rc::downgrade(&palette_ref), material, 1.0, Point { x: 0.5, y: 0.5, z: 0.5 }),
			palette: palette_ref,
		}
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
	pub fn subdivide(&mut self, fill: Material, is_filled: &dyn Fn(f32, Point) -> bool) {
		self.palette.borrow_mut().push(fill);
		let material = self.palette.borrow().get(fill);
		self.root.subdivide(material.clone(), is_filled);
	}
}

/// A node/voxel in the sparse voxel octree.
struct SculptNode {
	palette: Weak<RefCell<SculptPalette>>,
	material: Rc<Material>,
	children: [Option<Box<SculptNode>>; 8],
	center: Point,
	size: f32,
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
		}
	}

	/// Handles the sparse voxel octree modifications, recursively.
	fn subdivide(&mut self, fill: Rc<Material>, is_filled: &dyn Fn(f32, Point) -> bool) -> bool {
		if self.size < 0.1 {
			self.children = [None, None, None, None, None, None, None, None];
			return true;
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

		let lfb_child = if is_filled(half_size, lfb) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lfb)))
		} else {
			None
		};
		let rfb_child = if is_filled(half_size, rfb) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rfb)))
		} else {
			None
		};
		let lbb_child = if is_filled(half_size, lbb) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lbb)))
		} else {
			None
		};
		let rbb_child = if is_filled(half_size, rbb) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rbb)))
		} else {
			None
		};
		let lft_child = if is_filled(half_size, lft) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lft)))
		} else {
			None
		};
		let rft_child = if is_filled(half_size, rft) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rft)))
		} else {
			None
		};
		let lbt_child = if is_filled(half_size, lbt) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, lbt)))
		} else {
			None
		};
		let rbt_child = if is_filled(half_size, rbt) {
			Some(Box::new(SculptNode::new(self.palette.clone(), fill.clone(), half_size, rbt)))
		} else {
			None
		};

		self.children = [
			lfb_child, 
			rfb_child, 
			lbb_child, 
			rbb_child, 
			lft_child, 
			rft_child, 
			lbt_child, 
			rbt_child
		];

		let mut all_leaves = true;

		for child in &mut self.children {
			if let Some(ref mut to_subdivide) = child {
				let leaf = to_subdivide.subdivide(fill.clone(), is_filled);
				all_leaves = all_leaves && leaf;
			} else {
				all_leaves = false;
			}
		}

		if all_leaves {
			self.children = [None, None, None, None, None, None, None, None];
		}

		return all_leaves;
	}

	/// Convert the node and its children to the buffer format for the GPU.
	fn to_buffer(&self) -> Vec<u32> {
		let mut buffer = Vec::<u32>::new();

		self.append_to_buffer(&mut buffer, 1);

		buffer
	}

	/// Handle the actual, recursive logic for generating the buffer.
	fn append_to_buffer(&self, buffer: &mut Vec<u32>, mut counter: u32) -> u32 {
		let pointer = counter;
		let mut children = Vec::<u32>::new();
		let mut value = 0u32;

		let mut child_mask = 0;
		let mut leaf_mask = 0;
		let child_count = self.children.iter().fold(0, |carry, child| if child.is_some() { carry + 1 } else { carry });
		counter += child_count;

		for index in 0..8 {
			if let Some(child) = &self.children[index as usize] {
				if !child.children.iter().any(|s| s.is_some()) {
					leaf_mask |= 2u32.pow(index);
				}
				child_mask |= 2u32.pow(index);
				counter += child.append_to_buffer(&mut children, counter);
			}
		}

		if child_count == 0 && pointer > 1 {
			// a leaf node
			value |= self.palette.upgrade().unwrap().borrow().index(*self.material);
		} else {
			// an interior node
			value |= pointer << 16;
			value |= child_mask << 8;
			value |= leaf_mask;
		}

		println!("pointer: {pointer}, child: {child_mask}, leaf: {leaf_mask}, count: {child_count}");
		buffer.push(value);
		buffer.extend(children);

		child_count
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
