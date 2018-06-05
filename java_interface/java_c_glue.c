#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include "Raytracer.h"
 
void raytrace(){
    printf("Running executable\n");

    printf("Loading dynamic library\n");

    void* handle = dlopen("libvko_tracer.dylib", RTLD_LAZY);
    if (!handle) { 
        fprintf(stderr, "dlopen failure: %s\n", dlerror()); 
        exit (EXIT_FAILURE); 
    }
    
    printf("Calling render function\n");
    void * render_funciton = dlsym(handle, "render");
    if (!render_funciton)  { 
        fprintf(stderr, "dlsym failure: %s\n", dlerror()); 
        exit (EXIT_FAILURE); 
    } else {
        ((void(*)()) raytrace_funciton)();
    }

    dlclose(handle);
}
 
/*
 * Método com a mesma assinatura do CalculadoraJNI.h
 */
JNIEXPORT void JNICALL Java_Raytracer_raytrace
  (JNIEnv * env, jobject jobj) {
    raytrace();
}