use nxm_tracker::product_name;

fn main() {
    println!("{} {}", product_name(), env!("CARGO_PKG_VERSION"));
}
