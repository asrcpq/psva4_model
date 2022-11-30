use serde::{Serialize, Deserialize};
use std::io::{BufRead, BufReader};
use std::path::Path;

use std::collections::HashMap;
use crate::{M2, V2};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct RawVertex {
	pub pos: V2,
	pub tex: V2,
	pub im: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Rawmodel {
	pub name: HashMap<String, u32>,
	pub neigh: HashMap<u32, Vec<u32>>,
	pub vs: HashMap<u32, RawVertex>,
	pub fs: Vec<[u32; 3]>,
	pub tex_layer: i32,
	pub is_static: bool,
}

impl Rawmodel {
	pub fn load<P: AsRef<Path>>(
		file: P,
	) -> std::io::Result<Self> {
		let mut name = HashMap::default();
		let mut vs = HashMap::default();
		let mut id_alloc = 0u32;
		let mut fs = Vec::new();
		let file = std::fs::File::open(file)?;
		let reader = BufReader::new(file);
		for line in reader.lines() {
			let line = line?;
			let split: Vec<_> = line.split_whitespace().collect();
			if split.is_empty() { continue }
			match split[0] {
				"v" => {
					if split.len() < 4 {
						panic!("v error");
					}
					let v = V2::new(
						split[2].parse::<f32>().unwrap(),
						split[3].parse::<f32>().unwrap(),
					);
					let im = split.get(3).map(|x| x.parse::<f32>().unwrap()).unwrap_or(1f32);
					name.insert(split[0].to_string(), id_alloc);
					vs.insert(id_alloc, RawVertex {
						pos: v,
						tex: v,
						im,
					});
					id_alloc += 1;
				},
				"f" => {
					if split.len() != 4 {
						panic!("v error");
					}
					fs.push(core::array::from_fn(
						|idx| *name.get(split[idx + 1]).unwrap()
					));
				}
				_ => panic!("line error {}", split[0]),
			}
		}
		Ok(Self {
			name,
			neigh: Default::default(),
			vs,
			fs,
			tex_layer: -2,
			is_static: false,
		})
	}

	pub fn set_static(&mut self) {
		self.is_static = true;
	}

	pub fn transform(&mut self, m: M2) {
		for rv in self.vs.values_mut() {
			rv.pos = m * rv.pos;
		}
	}

	pub fn offset(&mut self, o: V2) {
		for rv in self.vs.values_mut() {
			rv.pos += o;
		}
	}
}
