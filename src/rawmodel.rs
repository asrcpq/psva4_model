use serde_with::serde_as;
use serde::{Serialize, Deserialize};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::{HashMap, HashSet};

use crate::{M2, V2};
use cgalg::d2::angle_dist;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RawVertex {
	pub pos: V2,
	pub tex: V2,
	pub im: f32,
}

pub type Vid = u64;

#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Rawmodel {
	pub name: HashMap<String, Vid>,
	pub id_alloc: Vid,
	pub neigh: HashMap<Vid, Vec<Vid>>,
	pub border: HashSet<Vid>,
	pub vs: HashMap<Vid, RawVertex>,
	#[serde_as(as = "Vec<(_, _)>")]
	pub dcs: HashMap<[Vid; 2], f32>,
	pub fs: Vec<[Vid; 3]>,
	pub tex_layer: i32,
	pub is_static: bool,
}

impl Rawmodel {
	pub fn load<P: AsRef<Path>>(file: P) -> std::io::Result<Self> {
		let string: String = std::fs::read_to_string(file)?;
		Ok(serde_json::from_str(&string)?)
	}

	pub fn fix(&mut self) {
		let mut bad_ent = Vec::new();
		for k in self.dcs.keys() {
			if !self.neigh.contains_key(&k[0]) ||
				!self.neigh.contains_key(&k[1]) ||
				k[0] == k[1]
			{
				bad_ent.push(k.clone());
			}
		}
		for ent in bad_ent.into_iter() {
			eprintln!("fix: dcs bad ent {:?}", ent);
			self.dcs.remove(&ent);
		}

		for (k, v) in self.neigh.iter_mut() {
			if let Some(pos) = v.iter().position(|x| x == k) {
				eprintln!("fix: self-self neigh {}", k);
				v.remove(pos);
			}
		}
	}

	// fix tex points to pos
	pub fn pos2tex(&mut self) {
		for v in self.vs.values_mut() {
			v.tex = v.pos;
		}
	}

	pub fn save<P: AsRef<Path>>(&self, file: P) -> std::io::Result<()> {
		let string = serde_json::to_string(self)?;
		std::fs::write(file, string)?;
		Ok(())
	}

	// build fs from vs
	pub fn build_topo2(&mut self) {
		self.fs.clear();
		self.border.clear();
		let mut faceset: HashSet<[Vid; 3]> = Default::default();

		// sort vs
		for (k, v) in self.vs.iter() {
			let k = *k;
			let p0 = v.pos;
			let neighs = self.neigh.get(&k).unwrap();
			for neigh in neighs.iter() {
				if *neigh == k {
					eprintln!("ERROR: nm, self-self");
					return
				}
				let mut ids = [*neigh, k];
				ids.sort_unstable();
				if !self.dcs.contains_key(&ids) {
					eprintln!("WARN: dc {:?} not exist, inst with 1e-6", ids);
					self.dcs.insert(ids, 1e-6);
				}
			}
			let mut angs: Vec<(Vid, f32)> = neighs
				.iter()
				.map(|v| {
					let vo = self.vs.get(v).unwrap();
					let p1 = vo.pos;
					let dp = p1 - p0;
					(*v, dp[1].atan2(dp[0]))
				}).collect();
			angs.sort_unstable_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
			let (mut vs, angs): (Vec<Vid>, Vec<_>) = angs.into_iter().unzip();
			let neilen = neighs.len();
			if neilen <= 1 {
				eprintln!("ERROR: nm, neigh <= 1 {}", k);
				return
			}
			let mut nonface = None;

			// sweep around k, check all point pairs are connect, or border
			// but neilen == 2 must be border
			if neilen == 2 {
				let nf = if angle_dist(angs[0], angs[1]) > 0f32 {
					0
				} else {
					1
				};
				nonface = Some(nf);
				let mut ids = [vs[0], vs[1], k];
				ids.sort_unstable();
				faceset.insert(ids);
			} else {
				for i in 0..neilen {
					let i2 = (i + 1) % neilen;
					if !self.neigh.get(&vs[i]).unwrap().contains(&vs[i2]) {
						if nonface.is_none() {
							nonface = Some(i2);
						} else {
							eprintln!("ERROR: nm, {} contains 2 nonfaces", k);
						}
					} else {
						let mut ids = [vs[i], vs[i2], k];
						ids.sort_unstable();
						faceset.insert(ids);
					}
				}
			}
			if let Some(x) = nonface {
				vs.rotate_left(x);
				self.border.insert(k);
			}
			self.neigh.insert(k, vs);
		}
		self.fs = faceset.into_iter().collect();
	}

	pub fn add_neigh_raw(&mut self, ids: [Vid; 2]) {
		let e = self.neigh.entry(ids[0]).or_insert_with(Vec::new);
		e.push(ids[1]);
		let e = self.neigh.entry(ids[1]).or_insert_with(Vec::new);
		e.push(ids[0]);
	}

	pub fn squash_neigh(&mut self) {
		for v in self.neigh.values_mut() {
			v.sort();
			v.dedup();
		}
	}

	// build vs from fs
	fn build_topo(&mut self) {
		self.dcs.clear();
		self.neigh.clear();
		self.border.clear();
		// build unsorted neigh
		for ids in self.fs.iter() {
			for i in 0..3 {
				for j in 0..3 {
					if i == j { continue }
					let vs = self.neigh.entry(ids[i]).or_insert_with(Default::default);
					let mut lids = [ids[i], ids[j]];
					lids.sort_unstable();
					self.dcs.insert(lids, 1e-6);
					vs.push(ids[j]);
				}
			}
		}
		self.squash_neigh();

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
			angs.sort_unstable_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
			let (mut vs, angs): (Vec<Vid>, Vec<_>) = angs.into_iter().unzip();
			let neilen = neighs.len();
			let rot = if neilen <= 1 {
				eprintln!("ERROR: Non-manifold found");
				None
			} else if neilen == 2 {
				if angle_dist(angs[0], angs[1]) > 0f32 {
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
				self.border.insert(k);
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
				_ => panic!("line error {}", split[0]),
			}
		}
		let mut result = Self {
			name,
			neigh: Default::default(),
			border: Default::default(),
			dcs: Default::default(),
			vs,
			fs,
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
