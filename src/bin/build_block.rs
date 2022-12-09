use psva4_model::rawmodel::{Rawmodel, Vid, RawVertex};
use psva4_model::V2;
use std::io::{BufRead, BufReader};

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	let file = std::fs::File::open(&args[1]).unwrap();
	let reader = BufReader::new(file);
	let mut lines = reader.lines();
	let line = lines.next().unwrap().unwrap();
	let r = line.parse::<f32>().unwrap();
	eprintln!("r: {}", r);
	let mut struc = vec![];
	let mut linelen = None;
	for line in lines {
		let line = line.unwrap();
		let line = line.trim();
		if line.len() == 0 || line.bytes().next().unwrap() == b'#' {
			continue
		}
		let struc_line: Vec<_> = line.split_whitespace()
			.map(|x| x.parse::<u32>().unwrap() != 0)
			.collect();
		if let Some(l) = linelen {
			assert_eq!(struc_line.len(), l);
		}
		linelen = Some(struc_line.len());
		struc.push(struc_line);
	}
	let mut rawmo = Rawmodel::default();
	let linelen = linelen.unwrap() + 1; // block line len + 1
	let get_pos = |l: usize, c: usize| -> Vid {
		(l * linelen + c) as Vid
	};
	for (idl, v) in struc.iter().enumerate() {
		for (idc, v) in v.iter().enumerate() {
			if !v { continue }
			let mut vids = Vec::new();
			let mut pos = Vec::new();
			for dl in 0..2 {
				for dc in 0..2 {
					let l = idl + dl;
					let c = idc + dc;
					vids.push(get_pos(l, c));
					pos.push(V2::new(c as f32 * r, l as f32 * r));
				}
			}
			for i in 0..4 {
				if rawmo.vs.contains_key(&vids[i]) { continue }
				rawmo.vs.insert(vids[i], RawVertex {
					tex: pos[i],
					pos: pos[i],
					mass: 1.0,
					break_thresh: 0.1,
				});
			}
			rawmo.add_neigh_raw([vids[0], vids[1]]);
			rawmo.add_neigh_raw([vids[0], vids[2]]);
			rawmo.add_neigh_raw([vids[0], vids[3]]);
			rawmo.add_neigh_raw([vids[1], vids[3]]);
			rawmo.add_neigh_raw([vids[2], vids[3]]);
		}
	}
	rawmo.name.insert("lu".to_string(), get_pos(0, 0));
	rawmo.name.insert("ru".to_string(), get_pos(0, linelen - 1));
	rawmo.name.insert("ld".to_string(), get_pos(struc.len(), 0));
	rawmo.name.insert("rd".to_string(), get_pos(struc.len(), linelen - 1));

	// offset id_alloc to unused part
	rawmo.id_alloc = ((struc.len() + 1) * linelen) as u64;
	rawmo.squash_neigh();
	rawmo.build_topo2();
	let path = if args.len() == 2 {
		format!("{}.tmp.json", args[1].strip_suffix(".p4b").unwrap())
	} else {
		args[2].to_string()
	};
	rawmo.save(path).unwrap();
}
