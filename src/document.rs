use std::cell::RefCell;
use std::rc::Weak;
use std::collections::HashSet;

use std::hash::Hash;

use std::hash::Hasher;
use std::rc::Rc;

struct Document {
	root: Node,
	palette: MaterialPalette,
}

impl Document {
	pub fn get_voxel_buffer(&self) -> Vec<u32> {
		self.root.to_buffer()
	}

	pub fn get_material_buffer(&self) -> Vec<u8> {
		self.palette.to_buffer()
	}
}

struct Node {
	material: Rc<RefCell<Material>>,
	children: [Option<Box<Node>>; 8],
}

impl Node {
	pub fn to_buffer(&self) -> Vec<u32> {
		self.append_to_buffer(Vec::<u32>::new(), 1)
	}

	pub fn append_to_buffer(&self, mut buffer: Vec<u32>, parent_child_count: u32) -> Vec<u32> {
		let mut value = 0u32;

		let pointer = buffer.len() as u32 + parent_child_count;

		let mut child_mask = 0u32;
		let mut child_count = 0;

		for index in 0..8 {
			if self.children[index as usize].is_some() {
				child_mask |= 2u32.pow(index);
				child_count += 1;
			}
		}

		value |= pointer << 16;
		value |= child_mask << 8;
		value |= self.material.as_ref().borrow().position as u32;

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

struct MaterialPalette {
	materials: Vec<Weak<RefCell<Material>>>,
	set: HashSet<Material>,
}

impl MaterialPalette {
	pub fn to_buffer(&self) -> Vec<u8> {
		let mut buffer = Vec::<u8>::new();

		for material in &self.materials {
			if let Some(upgraded) = material.upgrade() {
				buffer.extend(upgraded.as_ref().borrow().to_buffer());
			}
		}

		buffer
	}

	pub fn reindex(&mut self) {
		for index in 0..self.materials.len() {
			if let Some(material) = self.materials[index].upgrade() {
				let borrow = &mut material.as_ref().borrow_mut();
				borrow.position = index as u8;
			}
		}
	}

	pub fn push(&mut self, value: &Rc<RefCell<Material>>) {
		if !self.set.insert(*value.as_ref().borrow()) {
			return;
		}

		{
			let borrow = &mut value.as_ref().borrow_mut();
			borrow.position = (self.materials.len() + 1) as u8;
		}

		self.materials.push(Rc::downgrade(value));
	}
}

#[derive(Clone, Copy)]
struct Material {
	position: u8,
	color: [u8; 4],
	roughness: u8,
	metallic: u8,
}

impl Material {
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

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
        && self.roughness == other.roughness
        && self.metallic == other.metallic
    }
}

impl Eq for Material {}

impl Hash for Material {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.roughness.hash(state);
        self.metallic.hash(state);
    }
}
