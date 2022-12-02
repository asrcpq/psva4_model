use serde::{Serialize, Deserialize};

use crate::rawmodel::Vid;

// auxiliary structural contraint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asc {
	pub deps: Vec<Vid>,
	pub ps: Vec<Vid>,
	pub ty: AscType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AscType {
	// comp
	Dist(f32),
	// min/max (comp = 0 if out of range)
	Angle(f32, f32),
}

