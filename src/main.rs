mod config;
mod layering;

fn main() {
    let config = layering::get_layered_configs().unwrap();
    println!("Overlaid config: {config:?}");
}
