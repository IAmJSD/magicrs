#ifndef _ENGINE_M
#define _ENGINE_M
#include <dispatch/dispatch.h>

// Blocks on the main thread until the method is done.
void region_selector_main_thread_block(void (*callback)(void*), void* data) {
    dispatch_sync(dispatch_get_main_queue(), ^{
        callback(data);
    });
}

#endif // _ENGINE_M
