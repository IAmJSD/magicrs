#ifndef _MAGICCAP_LINUX_X11_C
#define _MAGICCAP_LINUX_X11_C
#include <X11/Xlib.h>
#include <X11/Xatom.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

// Defines the internal display handle.
Display* magiccap_internal_display = NULL;

// Defines the handler function.
void magiccap_handle_linux_x11(void* x_window_ptr, bool last) {
    // Start a connection to the X server.
    if (magiccap_internal_display == NULL) {
        magiccap_internal_display = XOpenDisplay(NULL);
    }

    // Cast the window pointer to the appropriate type.
    Window window = (Window)(uintptr_t)x_window_ptr;

    // Define the property to set the window type to dialog.
    Atom wmWindowType = XInternAtom(magiccap_internal_display, "_NET_WM_WINDOW_TYPE", False);
    Atom wmWindowTypeDialog = XInternAtom(magiccap_internal_display, "_NET_WM_WINDOW_TYPE_DIALOG", False);

    // Define the property to set the window always on top.
    Atom wmStateAbove = XInternAtom(magiccap_internal_display, "_NET_WM_STATE_ABOVE", False);
    Atom wmNetWmState = XInternAtom(magiccap_internal_display, "_NET_WM_STATE", False);

    // Prepare the event for changing the window state.
    XEvent xev;
    memset(&xev, 0, sizeof(xev));
    xev.type = ClientMessage;
    xev.xclient.window = window;
    xev.xclient.message_type = wmNetWmState;
    xev.xclient.format = 32;
    xev.xclient.data.l[0] = 1; // _NET_WM_STATE_ADD
    xev.xclient.data.l[1] = wmStateAbove;
    xev.xclient.data.l[2] = 0; // No second property
    xev.xclient.data.l[3] = 1; // Source indication (normal application)
    xev.xclient.data.l[4] = 0; // Unused

    // Send the event to the root window.
    XSendEvent(magiccap_internal_display, DefaultRootWindow(magiccap_internal_display), False, SubstructureRedirectMask | SubstructureNotifyMask, &xev);

    // Set the window type to dialog.
    XChangeProperty(magiccap_internal_display, window, wmWindowType, XA_ATOM, 32, PropModeReplace, (unsigned char *)&wmWindowTypeDialog, 1);

    // Flush the request to the X server.
    XFlush(magiccap_internal_display);

    // Set the OverrideRedirect attribute to true
    XSetWindowAttributes attrs;
    attrs.override_redirect = True;
    XChangeWindowAttributes(magiccap_internal_display, window, CWOverrideRedirect, &attrs);

    // Raise the window to the top
    XRaiseWindow(magiccap_internal_display, window);

    // Flush the request to the X server.
    XFlush(magiccap_internal_display);

    // Close the connection to the X server.
    if (last) {
        XCloseDisplay(magiccap_internal_display);
        magiccap_internal_display = NULL;
    }
}

#endif // _MAGICCAP_LINUX_X11_C
