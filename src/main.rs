extern crate clap;
use clap::{Arg, App};

extern crate vko_tracer;

fn main() {
let matches = App::new("vko_tracer")
                      .version("1.0")
                      .author("Ian M. <imvm@cin.ufpe.br>")
                      .about("Does raytracing")
                      .arg(Arg::with_name("INPUT")
                           .help("Sets the input file to use")
                           .required(false)
                           .index(1))
                      .get_matches();
                      
    let string = process_config(matches.value_of("INPUT"));

    vko_tracer::raytracer::render();
}

/// Process user specified configurations
fn process_config(option: Option<&str>) -> &str {
    match option {
        Some(filename) => filename,
        None => "scene.obj",
    }
}