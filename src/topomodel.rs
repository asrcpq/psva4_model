// topo and geom analysis(non-manifold checker)
// 1. ccw winding(auto fix)
// 2. overlap(and almost overlap)
// 3. singularity (connected to more than 1 surfaces)
// 4. connectivity (single object)

pub struct Topomodel {
	l0: Vec<L0r>,
	l1: Vec<L1r>,
	l2: Vec<L2r>,
}

struct L0 {
	// ccw
	// if open l1.len = l2.len + 1
	// if close l1.len == l2.len
	id: usize,
	l0: Vec<L0r>,
	l1: Vec<L1r>,
	l2: Vec<L2r>,
}

struct L1 {
	// l2.len() = {open: 1, close: 2}
	l0: [L0r; 2],
	l2: Vec<L2r>,
}

struct L2 {
	l0: [L0r; 3],
	l1: [L1r; 3],
}
