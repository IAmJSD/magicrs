#ifndef _MAGICCAP_LINUX_RECORDER_C
#define _MAGICCAP_LINUX_RECORDER_C
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <stdlib.h>
#include <stdint.h>

void* magiccap_recorder_x11_open_display() {
    return (void*)XOpenDisplay(NULL);
}

void magiccap_recorder_x11_close_display(void* display) {
    XCloseDisplay((Display*)display);
}

uint8_t* magiccap_recorder_x11_get_region_rgba(void* display, int x, int y, unsigned int w, unsigned int h) {
    Display* d = (Display*)display;
    Window root = DefaultRootWindow(d);
    XImage* img = XGetImage(d, root, x, y, w, h, AllPlanes, ZPixmap);
    if (img == NULL) {
        return NULL;
    }
    uint8_t* data = (uint8_t*)malloc(w * h * 4);
    if (data == NULL) {
        XDestroyImage(img);
        return NULL;
    }
    for (unsigned int i = 0; i < h; i++) {
        for (unsigned int j = 0; j < w; j++) {
            unsigned long pixel = XGetPixel(img, j, i);
            data[(i * w + j) * 4 + 0] = (pixel & img->red_mask) >> 16;
            data[(i * w + j) * 4 + 1] = (pixel & img->green_mask) >> 8;
            data[(i * w + j) * 4 + 2] = (pixel & img->blue_mask);
            data[(i * w + j) * 4 + 3] = 0xFF;
        }
    }
    XDestroyImage(img);
    return data;
}

#endif // _MAGICCAP_LINUX_RECORDER_C
