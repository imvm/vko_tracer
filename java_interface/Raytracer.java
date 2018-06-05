public class Raytracer {
    public native void raytrace(); 
    
    static {
        System.loadLibrary("vko_tracer_java");
    }    

}  