#ifndef _MAGICCAP_LINUX_RECORDER_C
#define _MAGICCAP_LINUX_RECORDER_C
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/cursorfont.h>
#include <X11/extensions/Xfixes.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

void* magiccap_recorder_x11_open_display() {
    return (void*)XOpenDisplay(NULL);
}

void magiccap_recorder_x11_close_display(void* display) {
    XCloseDisplay((Display*)display);
}

bool magiccap_recorder_x11_get_region_rgba(void* display, int x, int y, unsigned int w, unsigned int h, uint8_t* buf) {
    Display* d = (Display*)display;
    Window root = DefaultRootWindow(d);
    XImage* img = XGetImage(d, root, x, y, w, h, AllPlanes, ZPixmap);
    if (img == NULL) {
        return false;
    }

    for (unsigned int i = 0; i < h; i++) {
        for (unsigned int j = 0; j < w; j++) {
            unsigned long pixel = XGetPixel(img, j, i);
            buf[(i * w + j) * 4 + 0] = (pixel & img->red_mask) >> 16;
            buf[(i * w + j) * 4 + 1] = (pixel & img->green_mask) >> 8;
            buf[(i * w + j) * 4 + 2] = (pixel & img->blue_mask);
            buf[(i * w + j) * 4 + 3] = 0xFF;
        }
    }
    XDestroyImage(img);

    XFixesCursorImage* cursor = XFixesGetCursorImage(d);
    if (cursor != NULL) {
        int cursor_x = cursor->x - cursor->xhot - x;
        int cursor_y = cursor->y - cursor->yhot - y;

        for (int cy = 0; cy < cursor->height; cy++) {
            for (int cx = 0; cx < cursor->width; cx++) {
                if (cursor_x + cx >= 0 && cursor_x + cx < (int)w &&
                    cursor_y + cy >= 0 && cursor_y + cy < (int)h) {

                    unsigned long cursor_pixel = cursor->pixels[cy * cursor->width + cx];
                    uint8_t cursor_a = (cursor_pixel >> 24) & 0xFF;
                    uint8_t cursor_r = (cursor_pixel >> 16) & 0xFF;
                    uint8_t cursor_g = (cursor_pixel >> 8) & 0xFF;
                    uint8_t cursor_b = cursor_pixel & 0xFF;

                    int buf_index = ((cursor_y + cy) * w + (cursor_x + cx)) * 4;
                    uint8_t bg_r = buf[buf_index + 0];
                    uint8_t bg_g = buf[buf_index + 1];
                    uint8_t bg_b = buf[buf_index + 2];
                    uint8_t bg_a = buf[buf_index + 3];

                    // Alpha blending
                    buf[buf_index + 0] = (cursor_r * cursor_a + bg_r * (255 - cursor_a)) / 255;
                    buf[buf_index + 1] = (cursor_g * cursor_a + bg_g * (255 - cursor_a)) / 255;
                    buf[buf_index + 2] = (cursor_b * cursor_a + bg_b * (255 - cursor_a)) / 255;
                    buf[buf_index + 3] = bg_a; // Keep the original background alpha
                }
            }
        }
        XFree(cursor);
    }

    return true;
}

#endif // _MAGICCAP_LINUX_RECORDER_C
