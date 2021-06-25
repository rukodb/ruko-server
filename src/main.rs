use ruko_server::Ruko;

fn main() {
    let db = Ruko::new();
    println!("Hello, world! {:?}", db);
}
