//! GPU raytracing library
//! Uses the Vulkano wrapper to simplify access to the Vulkan API

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate image;

pub mod raytracer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
