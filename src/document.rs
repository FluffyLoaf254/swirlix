use std::cell::RefCell;
use std::rc::Weak;
use std::collections::HashSet;

use std::hash::Hash;

use std::hash::Hasher;
use std::rc::Rc;

/// A 3D sculpt "document".
///
/// This is stored using a sparse voxel octree (SVO) data
/// structure, contained at `root`.
///
/// The `palette` stores distinct materials to be referenced
/// by the voxel data.
pub struct Document {
	root: Node,
	palette: MaterialPalette,
}

impl Document {
	pub fn new() -> Self {
		Self {
			root: Node::new(None, 2.0, 0.0, 0.0, 0.0),
			palette: MaterialPalette::new(),
		}
	}

	/// Gets the raw data for the voxel buffer.
	pub fn get_voxel_buffer(&self) -> Vec<u32> {
		self.root.to_buffer()
	}

	/// Gets the raw data for the material palette buffer.
	pub fn get_material_buffer(&self) -> Vec<u32> {
		self.palette.to_buffer()
	}

	pub fn subdivide(&mut self, fill: Material, is_filled: &dyn Fn(f32, f32, f32, f32) -> bool) {
		let material = Rc::new(RefCell::new(fill));
		self.palette.push(&material);
		self.root.subdivide(&material, is_filled);
	}
}

/// A node/voxel in the sparse voxel octree.
struct Node {
	material: Option<Rc<RefCell<Material>>>,
	children: [Option<Box<Node>>; 8],
	x: f32,
	y: f32,
	z: f32,
	size: f32,
}

impl Node {
	pub fn new(material: Option<Rc<RefCell<Material>>>, size: f32, x: f32, y: f32, z: f32) -> Self {
		Self {
			material,
			children: [None, None, None, None, None, None, None, None],
			size,
			x,
			y,
			z,
		}
	}

	pub fn subdivide(&mut self, fill: &Rc<RefCell<Material>>, is_filled: &dyn Fn(f32, f32, f32, f32) -> bool) {
		let half_size = self.size / 2.0;
		let quarter_size = self.size / 4.0;

		let lfb = if is_filled(half_size, self.x - quarter_size, self.y - quarter_size, self.z - quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x - quarter_size, self.y - quarter_size, self.z - quarter_size)))
		} else {
			None
		};
		let rfb = if is_filled(half_size, self.x + quarter_size, self.y - quarter_size, self.z - quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x + quarter_size, self.y - quarter_size, self.z - quarter_size)))
		} else {
			None
		};
		let lbb = if is_filled(half_size, self.x - quarter_size, self.y + quarter_size, self.z - quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x - quarter_size, self.y + quarter_size, self.z - quarter_size)))
		} else {
			None
		};
		let rbb = if is_filled(half_size, self.x + quarter_size, self.y + quarter_size, self.z - quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x + quarter_size, self.y + quarter_size, self.z - quarter_size)))
		} else {
			None
		};
		let lft = if is_filled(half_size, self.x - quarter_size, self.y - quarter_size, self.z + quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x - quarter_size, self.y - quarter_size, self.z + quarter_size)))
		} else {
			None
		};
		let rft = if is_filled(half_size, self.x + quarter_size, self.y - quarter_size, self.z + quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x + quarter_size, self.y - quarter_size, self.z + quarter_size)))
		} else {
			None
		};
		let lbt = if is_filled(half_size, self.x - quarter_size, self.y + quarter_size, self.z + quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x - quarter_size, self.y + quarter_size, self.z + quarter_size)))
		} else {
			None
		};
		let rbt = if is_filled(half_size, self.x + quarter_size, self.y + quarter_size, self.z + quarter_size) {
			Some(Box::new(Node::new(Some(fill.clone()), half_size, self.x + quarter_size, self.y + quarter_size, self.z + quarter_size)))
		} else {
			None
		};

		self.children = [lfb, rfb, lbb, rbb, lft, rft, lbt, rbt];

		if half_size <= 0.005 {
			return;
		}

		for child in &mut self.children {
			if let Some(ref mut to_subdivide) = child {
				to_subdivide.subdivide(fill, is_filled);
			}
		}
	}

	/// Convert the node and its children to the buffer format for the GPU.
	pub fn to_buffer(&self) -> Vec<u32> {
		self.append_to_buffer(Vec::<u32>::new(), 1)
	}

	/// Handle the actual, recursive logic for generating the buffer.
	pub fn append_to_buffer(&self, mut buffer: Vec<u32>, parent_child_count: u32) -> Vec<u32> {
		let mut value = 0u32;

		let pointer = buffer.len() as u32 + parent_child_count;

		let mut child_mask = 0;
		let mut leaf_mask = 0;
		let mut child_count = 0;

		for index in 0..8 {
			if let Some(child) = &self.children[index as usize] {
				if child.children.iter().any(|s| s.is_some()) {
					leaf_mask |= 2u32.pow(index);
				}
				child_mask |= 2u32.pow(index);
				child_count += 1;
			}
		}

		if child_count == 0 {
			// a leaf node
			value |= self.material.as_ref().unwrap().as_ref().borrow().position;
		} else {
			// an interior node
			value |= pointer << 16;
			value |= child_mask << 8;
			value |= leaf_mask;
		}

		buffer.push(value);

		for child in &self.children {
			if let Some(child) = child {
				buffer = child.append_to_buffer(buffer, child_count);
				child_count -= 1;
			}
		}

		buffer
	}
}

/// The `MaterialPalette` stores the materials that are used in the current sculpt.
/// They should be pruned if they are no longer in use.
struct MaterialPalette {
	materials: Vec<Weak<RefCell<Material>>>,
	set: HashSet<Material>,
}

impl MaterialPalette {
	pub fn new() -> Self {
		Self {
			materials: Vec::new(),
			set: HashSet::new(),
		}
	}

	/// Converts the palette materials to a buffer for use on the GPU.
	pub fn to_buffer(&self) -> Vec<u32> {
		let mut buffer = Vec::new();

		for material in &self.materials {
			if let Some(upgraded) = material.upgrade() {
				buffer.extend(upgraded.as_ref().borrow().to_buffer());
			}
		}

		buffer
	}

	/// Prunes unused materials and re-indexes.
	pub fn prune(&mut self) {
		let mut pruned = Vec::new();
		let mut number_pruned = 0;

		for index in 0..self.materials.len() {
			if let Some(material) = self.materials[index].upgrade() {
				let borrow = &mut material.as_ref().borrow_mut();
				borrow.position = index as u32 - number_pruned;
				pruned.push(Rc::downgrade(&material));
			} else {
				number_pruned += 1;
			}
		}
	}

	/// Re-applies the index to the materials.
	pub fn reindex(&mut self) {
		for index in 0..self.materials.len() {
			if let Some(material) = self.materials[index].upgrade() {
				let borrow = &mut material.as_ref().borrow_mut();
				borrow.position = index as u32;
			}
		}
	}

	/// Pushes a new material onto the palette.
	pub fn push(&mut self, value: &Rc<RefCell<Material>>) {
		if !self.set.insert(*value.as_ref().borrow()) {
			return;
		}

		{
			let borrow = &mut value.as_ref().borrow_mut();
			borrow.position = (self.materials.len() + 1) as u32;
		}

		self.materials.push(Rc::downgrade(value));
	}
}

/// A material to be referenced by voxels to describe the surface.
#[derive(Clone, Copy)]
pub struct Material {
	position: u32,
	color: [u32; 4],
	roughness: u32,
	metallic: u32,
}

impl Material {
	pub fn new(color: [f32; 4], roughness: f32, metallic: f32) -> Self {
		Self {
			position: 0,
			color: [
				(color[0] * 255.0).floor() as u32,
				(color[1] * 255.0).floor() as u32,
				(color[2] * 255.0).floor() as u32,
				(color[3] * 255.0).floor() as u32,
			],
			roughness: (roughness * 255.0).floor() as u32,
			metallic: (metallic * 255.0).floor() as u32
		}
	}

	/// Convert the material to the buffer data structure.
	pub fn to_buffer(&self) -> [u32; 6] {
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

impl PartialEq for Material {
	/// Material IDs should not affect equality.
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
        && self.roughness == other.roughness
        && self.metallic == other.metallic
    }
}

impl Eq for Material {}

impl Hash for Material {
	/// Material IDs should not affect hash comparisons.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.roughness.hash(state);
        self.metallic.hash(state);
    }
}
