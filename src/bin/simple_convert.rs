use psva4_model::rawmodel::Rawmodel;

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	let model = Rawmodel::simple_load(&args[1]).unwrap();
	model.save(&args[2]).unwrap();
}
