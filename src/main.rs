mod config;
mod layering;

fn main() {
    stderrlog::new()
        .show_module_names(true)
        .color(stderrlog::ColorChoice::Auto)
        .verbosity(log::Level::Trace)
        .init()
        .unwrap();

    let config = layering::get_layered_configs().unwrap();
    println!("Overlaid config: {config:?}");
}
