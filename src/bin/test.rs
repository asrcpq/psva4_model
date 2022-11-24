use psva4_model::rawmodel::Rawmodel;

fn main() {
	eprintln!("{:?}", Rawmodel::load("./test/bar.p4m").unwrap());
}
