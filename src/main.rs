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

    vko_tracer::raytracer::init();

    vko_tracer::raytracer::process_file(matches.value_of("INPUT"));

    vko_tracer::raytracer::raytrace();

    vko_tracer::raytracer::cleanup();
}
