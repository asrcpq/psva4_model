use serde::{Serialize, Deserialize};
use std::io::{BufRead, BufReader};
use std::path::Path;

use std::collections::HashMap;
use crate::{M2, V2};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RawVertex {
	pub pos: V2,
	pub tex: V2,
	pub im: f32,
}

type Vid = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Control {
	pub vs: [Vid; 2],
	pub key: u8,
	pub k: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Rawmodel {
	pub name: HashMap<String, Vid>,
	pub id_alloc: Vid,
	pub neigh: HashMap<Vid, Vec<Vid>>,
	pub border: Vec<Vid>,
	pub vs: HashMap<Vid, RawVertex>,
	pub fs: Vec<[Vid; 3]>,
	pub control: Vec<Control>,
	pub tex_layer: i32,
	pub is_static: bool,
}

impl Rawmodel {
	pub fn load<P: AsRef<Path>>(file: P) -> std::io::Result<Self> {
		let string: String = std::fs::read_to_string(file)?;
		Ok(serde_json::from_str(&string)?)
	}

	pub fn save<P: AsRef<Path>>(&self, file: P) -> std::io::Result<()> {
		let string = serde_json::to_string(self)?;
		std::fs::write(file, string)?;
		Ok(())
	}

	pub fn build_topo(&mut self) {
		assert!(self.neigh.is_empty());
		for ids in self.fs.iter() {
			for i in 0..3 {
				for j in 0..3 {
					if i == j { continue }
					let vs = self.neigh.entry(ids[i]).or_insert_with(Default::default);
					if vs.iter().any(|&x| x == ids[j]) { continue }
					vs.push(ids[j]);
				}
			}
		}

		for (k, v) in self.vs.iter() {
			let k = *k;
			let p0 = v.pos;
			let neighs = self.neigh.get(&k).unwrap();
			let mut angs: Vec<(Vid, f32)> = neighs
				.iter()
				.map(|v| {
					let vo = self.vs.get(v).unwrap();
					let p1 = vo.pos;
					let dp = p1 - p0;
					(*v, dp[1].atan2(dp[0]))
				}).collect();
			angs.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
			let (mut vs, angs): (Vec<Vid>, Vec<_>) = angs.into_iter().unzip();
			let neilen = neighs.len();
			let rot = if neilen <= 1 {
				eprintln!("ERROR: Non-manifold found");
				None
			} else if neilen == 2 {
				if cgalg::d2::angle_dist(angs[0], angs[1]) > 0f32 {
					Some(1)
				} else {
					Some(0)
				}
			} else {
				let mut rot = None;
				for idx in 0..neilen {
					let id1 = vs[idx];
					let id2 = vs[(idx + 1) % neilen];
					let mut ids = [k, id1, id2];
					ids.sort();
					if !self.fs.contains(&ids) {
						if rot.is_some() {
							eprintln!("ERROR: Non-manifold found");
						} else {
							rot = Some(idx + 1);
							break
						}
					}
				}
				rot
			};
			if let Some(idx) = rot {
				vs.rotate_left(idx);
				self.border.push(k);
			}
			// assert!(vs.iter().all(|x| *x != k));
			self.neigh.insert(k, vs);
		}
	}

	pub fn simple_load<P: AsRef<Path>>(
		file: P,
	) -> std::io::Result<Self> {
		let mut name = HashMap::default();
		let mut vs = HashMap::default();
		let mut id_alloc = Vid::default();
		let mut fs = Vec::new();
		let mut control = Vec::new();
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
					let im = split.get(4).map(|x| x.parse::<f32>().unwrap()).unwrap_or(1f32);
					name.insert(split[1].to_string(), id_alloc);
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
					let mut f = core::array::from_fn(|idx| *name.get(split[idx + 1]).unwrap());
					f.sort_unstable();
					fs.push(f);
				}
				"c" => {
					if split.len() != 5 {
						panic!("c error");
					}
					control.push(Control {
						vs: [
							*name.get(split[1]).unwrap(),
							*name.get(split[2]).unwrap(),
						],
						key: split[3].bytes().next().unwrap(),
						k: split[4].parse::<f32>().unwrap(),
					});
				}
				_ => panic!("line error {}", split[0]),
			}
		}
		let mut result = Self {
			name,
			neigh: Default::default(),
			border: Vec::new(),
			vs,
			fs,
			control,
			tex_layer: -2,
			is_static: false,
			id_alloc,
		};
		result.build_topo();
		Ok(result)
	}

	pub fn alloc_id(&mut self) -> Vid {
		self.id_alloc += 1;
		self.id_alloc - 1
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
