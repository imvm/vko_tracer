#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
 
int main(){
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

    return 0;
}