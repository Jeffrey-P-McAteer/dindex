use dindex::get_config;


fn main() {
  let config = get_config();
  println!("config = {:?}", config);
}
