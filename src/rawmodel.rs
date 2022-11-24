use std::io::{BufRead, BufReader};

#[derive(Default, Debug)]
pub struct Rawmodel {
	pub vs: Vec<[f32; 2]>,
	pub fs: Vec<[usize; 3]>,
}

impl Rawmodel {
	pub fn load(file: &str) -> std::io::Result<Self> {
		let mut vid_lookup: std::collections::HashMap<usize, usize> = Default::default();
		let mut vs = Vec::new();
		let mut fs = Vec::new();
		let file = std::fs::File::open(file)?;
		let reader = BufReader::new(file);
		for line in reader.lines() {
			let line = line?;
			let split: Vec<_> = line.split_whitespace().collect();
			if split.len() == 0 { continue }
			match split[0] {
				"v" => {
					if split.len() != 4 {
						panic!("v error");
					}
					vid_lookup.insert(split[1].parse::<usize>().unwrap(), vs.len());
					vs.push([
						split[2].parse::<f32>().unwrap(),
						split[3].parse::<f32>().unwrap(),
					]);
				},
				"f" => {
					if split.len() != 4 {
						panic!("v error");
					}
					fs.push(core::array::from_fn(|idx|
						vid_lookup[&split[idx + 1].parse::<usize>().unwrap()]
					));
				}
				_ => panic!("line error {}", split[0]),
			}
		}
		Ok(Self {
			vs,
			fs,
		})
	}
}
