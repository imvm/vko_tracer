use vulkano;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use image::ImageBuffer;
use image::Rgba;
use vulkano::image::StorageImage;
use vulkano::image::Dimensions;
use vulkano::format::Format;

use std::sync::Arc;

const DEFAULT_FILENAME: &str = "scene.obj";

/// Initialize raytracing context
pub fn init() {
    println!("Initializing!");
}

/// Process user specified configurations
pub fn process_config(option: Option<&str>) {
    match option {
        Some(filename) => parse_file(filename),
        None => parse_file(DEFAULT_FILENAME),
    }
}

/// Parse scene file
fn parse_file(filename: &str) {
    println!("{}", filename);
}

/// Raytracing function
pub fn raytrace() {
    // As with other examples, the first step is to create an instance.
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("failed to create Vulkan instance");

    // Choose which physical device to use.
    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next().expect("no device available");

    // Choose the queue of the physical device which is going to run our compute operation.
    //
    // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
    // that supports compute operations.
    let queue = physical.queue_families().find(|&q| q.supports_compute()).unwrap();

    // Now initializing the device.
    let (device, mut queues) = {
        Device::new(physical, physical.supported_features(), &DeviceExtensions::none(),
                    [(queue, 0.5)].iter().cloned()).expect("failed to create device")
    };

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retreive the first and only element of the
    // iterator and throw it away.
    let queue = queues.next().unwrap();

    println!("Device initialized");

    let image_width = 1024;
    let image_height = 1024;

    let image = StorageImage::new(device.clone(), 
        Dimensions::Dim2d {width: image_width, height: image_height},
        Format::R8G8B8A8Unorm, 
        Some(queue.family())).unwrap();

    let image_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0 .. image_width * image_height * 4).map(|_| 0u8))
        .expect("failed to create image buffer");

    // Now let's get to the actual example.
    //
    // What we are going to do is very basic: we are going to fill a buffer with 64k integers
    // and ask the GPU to multiply each of them by 12.
    //
    // GPUs are very good at parallel computations (SIMD-like operations), and thus will do this
    // much more quickly than a CPU would do. While a CPU would typically multiply them one by one
    // or four by four, a GPU will do it by groups of 32 or 64.
    //
    // Note however that in a real-life situation for such a simple operation the cost of
    // accessing memory usually outweights the benefits of a faster calculation. Since both the CPU
    // and the GPU will need to access data, there is no other choice but to transfer the data
    // through the slow PCI express bus.

    // We need to create the compute pipeline that describes our operation.
    //
    // If you are familiar with graphics pipeline, the principle is the same except that compute
    // pipelines are much more simple to create.
    let pipeline = Arc::new({
        // TODO: explain
        mod cs {
            #[derive(VulkanoShader)]
            #[ty = "compute"]
            #[src = "
#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (local_size_x = 32, local_size_y = 32) in;
layout (binding = 0, rgba8) uniform image2D computeImage;

#define PI 3.141592
#define Inf 1000000.0
#define Epsilon 0.0001

struct Ray
{
	vec3 origin;
	vec3 direction;
};

struct Sphere
{
	vec3 position;
	float radius;
	vec3 color;
};

layout (binding = 1) buffer Spheres
{
	Sphere spheres[];
};

//////////////////////////////

vec3 Camera (in float x, in float y)
{
	ivec2 dimensions = imageSize(computeImage);
	float w = dimensions.x;
	float h = dimensions.y;

	float fovX = PI / 4;
	float fovY = (h / w) * fovX;

	float _x = ((2 * x - w) / w) * tan(fovX);
	float _y = -((2 * y - h) / h) * tan(fovY);

	return vec3(_x, _y, -1.0);
}

float SphereIntersection (in Ray ray, in Sphere sphere)
{
	vec3 delta = ray.origin - sphere.position;
	float b = dot((delta * 2), ray.direction);
	float c = dot(delta, delta) - (sphere.radius * sphere.radius);

	float disc = b * b - 4 * c;
	if (disc < 0)
		return 0;
	else
		disc = sqrt(disc);

	// Always 2 solutions when pulling the square root.
	float result1 = -b + disc;
	float result2 = -b - disc;

	return (result2 > Epsilon) ? result2 / 2 : ((result1 > Epsilon) ? result1 / 2 : 0);
}

bool TryGetIntersection (in Ray ray, out int id)
{
	id = -1;
	
	for (int i = 0; i < 4; i++)
	{
		Sphere s = spheres[i];
		float dist = SphereIntersection(ray, s);
		if (dist > Epsilon && dist < Inf)
		{
			id = i;
		}
	}

	return (id > -1) ? true : false;
}

//////////////////////////////

vec3 Trace (inout Ray ray)
{
	vec3 finalColor = vec3(1.0, 1.0, 1.0);

    int id;
    bool intersection = TryGetIntersection(ray, id);
    if (intersection) {
        Sphere s = spheres[id];
        finalColor = s.color;
    }

	return finalColor;
}


void main()
{
	uint idx = gl_GlobalInvocationID.x;
	uint idy = gl_GlobalInvocationID.y;

	Ray ray;
	ray.origin = vec3(0, 0, -0.1);
	vec3 cam = Camera(idx, idy);
	ray.direction = normalize( (cam - ray.origin));

	vec3 finalColor = Trace(ray);	

	imageStore(computeImage, ivec2(gl_GlobalInvocationID.xy), vec4(finalColor, 1.0));
}
"
]
            struct Dummy;
        }
        
        let shader = cs::Shader::load(device.clone())
            .expect("failed to create shader module");
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline")
    });

    #[derive(Copy, Clone)]
    struct Sphere {
        position: [f32; 3],
        radius: f32,
        color: [f32; 3],
        padding: f32,
    };

    let spheres = vec![
        Sphere { position: [1.0, 0.0, -3.0], radius: 0.4, color: [0.0, 0.0, 1.0], padding: 0.0 }, 
        Sphere { position: [0.0, 0.5, -2.0], radius: 0.6, color: [0.0, 1.0, 0.0], padding: 0.0 },
        Sphere { position: [-1.0, -1.0, -2.0], radius: 0.5, color: [1.0, 0.0, 0.0], padding: 0.0 }, 
        Sphere { position: [0.0, 0.0, 1.0], radius: 0.0, color: [0.0, 1.0, 0.0], padding: 0.0 }, 
    ];

    // We start by creating the buffer that will store the data.
    let sphere_buffer = {

        CpuAccessibleBuffer::from_iter(device.clone(), 
        BufferUsage::all(), 
        spheres.into_iter()).unwrap()
        
        //ImmutableBuffer::from_iter(spheres.iter().cloned(),
        //     BufferUsage::all(), queue)
        //    .expect("failed to create buffer")
    };

    // In order to let the shader access the buffer, we need to build a *descriptor set* that
    // contains the buffer.
    //
    // The resources that we bind to the descriptor set must match the resources expected by the
    // pipeline which we pass as the first parameter.
    //
    // If you want to run the pipeline on multiple different buffers, you need to create multiple
    // descriptor sets that each contain the buffer you want to run the shader on.
    let set = Arc::new(PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_image(image.clone()).unwrap()
        .add_buffer(sphere_buffer.clone()).unwrap()
        .build().unwrap()
    );

    // In order to execute our operation, we have to build a command buffer.
    let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
        // The command buffer only does one thing: execute the compute pipeline.
        // This is called a *dispatch* operation.
        //
        // Note that we clone the pipeline and the set. Since they are both wrapped around an
        // `Arc`, this only clones the `Arc` and not the whole pipeline or set (which aren't
        // clonable anyway). In this example we would avoid cloning them since this is the last
        // time we use them, but in a real code you would probably need to clone them.
        .dispatch([image_width, image_height, 1], pipeline.clone(), set.clone(), ()).unwrap()

        // Copy drawn image to buffer
        .copy_image_to_buffer(image.clone(), image_buffer.clone()).unwrap()
        // Finish building the command buffer by calling `build`.

        .build().unwrap();

    // Let's execute this command buffer now.
    // To do so, we TODO: this is a bit clumsby, probably needs a shortcut
    let future = now(device.clone())
        .then_execute(queue.clone(), command_buffer).unwrap()

        // This line instructs the GPU to signal a *fence* once the command buffer has finished
        // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
        // reached a certain point.
        // We need to signal a fence here because below we want to block the CPU until the GPU has
        // reached that point in the execution.
        .then_signal_fence_and_flush().unwrap();
    
    // Blocks execution until the GPU has finished the operation. This method only exists on the
    // future that corresponds to a signalled fence. In other words, this method wouldn't be
    // available if we didn't call `.then_signal_fence_and_flush()` earlier.
    // The `None` parameter is an optional timeout.
    //
    // Note however that dropping the `future` variable (with `drop(future)` for example) would
    // block execution as well, and this would be the case even if we didn't call
    // `.then_signal_fence_and_flush()`.
    // Therefore the actual point of calling `.then_signal_fence_and_flush()` and `.wait()` is to
    // make things more explicit. In the future, if the Rust language gets linear types vulkano may
    // get modified so that only fence-signalled futures can get destroyed like this.
    future.wait(None).unwrap();

    // Now that the GPU is done, the content of the buffer should have been modified. Let's
    // check it out.
    // The call to `read()` would return an error if the buffer was still in use by the GPU.
    
    let buffer_content = image_buffer.read().unwrap();
    let rendered_image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_width, image_height, &buffer_content[..]).unwrap();
    rendered_image.save("render.png").unwrap();
}

/// Cleanup scene context
pub fn cleanup() {
    println!("Cleaning up!");
}
