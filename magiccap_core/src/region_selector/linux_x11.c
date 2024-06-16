#ifndef _MAGICCAP_LINUX_X11_C
#define _MAGICCAP_LINUX_X11_C
#include <X11/Xlib.h>
#include <X11/Xatom.h>
#include <stdint.h>

void magiccap_handle_linux_x11(void* x_window_ptr) {
    // Start a connection to the X server.
    Display* display = XOpenDisplay(NULL);

    // Cast the window pointer to the appropriate type.
    Window window = (Window)(uintptr_t)x_window_ptr;

    // Define the property to set the window type to dialog.
    Atom wmWindowType = XInternAtom(display, "_NET_WM_WINDOW_TYPE", False);
    Atom wmWindowTypeDialog = XInternAtom(display, "_NET_WM_WINDOW_TYPE_DIALOG", False);

    // Define the property to set the window always on top.
    Atom wmStateAbove = XInternAtom(display, "_NET_WM_STATE_ABOVE", False);
    Atom wmNetWmState = XInternAtom(display, "_NET_WM_STATE", False);

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
    XSendEvent(display, DefaultRootWindow(display), False, SubstructureRedirectMask | SubstructureNotifyMask, &xev);

    // Set the window type to dialog.
    XChangeProperty(display, window, wmWindowType, XA_ATOM, 32, PropModeReplace, (unsigned char *)&wmWindowTypeDialog, 1);

    // Flush the request to the X server.
    XFlush(display);

    // Set the OverrideRedirect attribute to true
    XSetWindowAttributes attrs;
    attrs.override_redirect = True;
    XChangeWindowAttributes(display, window, CWOverrideRedirect, &attrs);

    // Raise the window to the top
    XRaiseWindow(display, window);

    // Flush the request to the X server.
    XFlush(display);

    // Close the connection to the X server.
    XCloseDisplay(display);
}

#endif // _MAGICCAP_LINUX_X11_C
