#ifndef _ENGINE_C
#define _ENGINE_C
#include <inttypes.h>
#include <GLFW/glfw3.h>
#include <stdbool.h>
#ifdef __APPLE__
#include <OpenGL/gl3.h>
#include "./engine.m"
#endif
#ifdef __linux__
#include <GL/gl.h>
#endif

// Defines a co-ordinate.
typedef struct region_coordinate_t {
    int32_t x;
    int32_t y;
};

// Defines a region capture result.
typedef struct region_result_t {
    struct region_coordinate_t coordinate;
    uint32_t w;
    uint32_t h;
    uint8_t* rgba;
    size_t rgba_len;
    int display_index;
};

// Defines the setup result.
typedef struct region_selector_setup_result_t {
    // Inputted by the open function.
    size_t display_count;
    struct coordinate_t* coordinates;

    // Outputted by the setup function.
    GLFWmonitor** monitors;
    GLFWwindow** windows;
};

// Sort the monitors based on the coordinates. Returns null if there are no monitors with the coordinates.
GLFWmonitor** region_selector_sort_monitors(GLFWmonitor** monitors, int monitor_count, struct region_coordinate_t* coordinates) {
    // Allocate the result.
    GLFWmonitor** result = (GLFWmonitor**)malloc(monitor_count * sizeof(GLFWmonitor*));

    // Go through each co-ordinate.
    for (int i = 0; i < monitor_count; i++) {
        // Get the co-ordinates.
        struct region_coordinate_t* coordinate = &coordinates[i];

        // Go through each monitor.
        GLFWmonitor* result_monitor = NULL;
        for (int j = 0; j < monitor_count; j++) {
            // Get the monitor.
            GLFWmonitor* monitor = monitors[j];

            // Check if the display X/Y is the same as the co-ordinate.
            int x, y;
            glfwGetMonitorPos(monitor, &x, &y);
            if (x == coordinate->x && y == coordinate->y) {
                result_monitor = monitor;
                break;
            }
        }

        // If the monitor is null, return null.
        if (result_monitor == NULL) {
            free(result);
            return NULL;
        }

        // Set the result.
        result[i] = result_monitor;
    }

    // Write result over the monitors.
    for (int i = 0; i < monitor_count; i++) monitors[i] = result[i];
    free(result);

    // Return the monitors.
    return monitors;
}

// Do the glfw setup.
void region_selector_glfw_setup(struct region_selector_setup_result_t* setup_result) {
    // Make sure glfw is initialized.
    if (glfwInit() == GLFW_FALSE) return;

    // Get the monitors.
    int monitor_count;
    GLFWmonitor** monitors = glfwGetMonitors(&monitor_count);
    if (monitor_count != (int) setup_result->display_count) return;
    setup_result->monitors = region_selector_sort_monitors(monitors, monitor_count, setup_result->coordinates);
    if (setup_result->monitors == NULL) return;

    // Create a window for each monitor.
    GLFWwindow** windows = (GLFWwindow**)malloc(monitor_count * sizeof(GLFWwindow*));
    for (int i = 0; i < monitor_count; i++) {
        // Get the monitor.
        GLFWmonitor* monitor = setup_result->monitors[i];

        // Get the video mode.
        int width, height;
        glfwGetMonitorPhysicalSize(monitor, &width, &height);

        // Create the window.
        glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE);
        glfwWindowHint(GLFW_DECORATED, GLFW_FALSE);
        glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);
        glfwWindowHint(GLFW_FOCUSED, GLFW_TRUE);
        glfwWindowHint(GLFW_AUTO_ICONIFY, GLFW_FALSE);
        glfwWindowHint(GLFW_FLOATING, GLFW_TRUE);
        glfwWindowHint(GLFW_MAXIMIZED, GLFW_TRUE);
        glfwWindowHint(GLFW_CENTER_CURSOR, GLFW_FALSE);
        glfwWindowHint(GLFW_FOCUS_ON_SHOW, GLFW_TRUE);
        glfwWindowHint(GLFW_SCALE_TO_MONITOR, GLFW_TRUE);
        GLFWwindow* window = glfwCreateWindow(width, height, "Region Selector", monitor, NULL);
        if (window == NULL) {
            // Free the monitors.
            free(setup_result->monitors);
            setup_result->monitors = NULL;
            return;
        }

        // Make the window current.
        glfwMakeContextCurrent(window);

        // Set the window position.
        int x, y;
        glfwGetMonitorPos(monitor, &x, &y);

        // Make the window full screen.
        glfwSetWindowMonitor(window, monitor, 0, 0, width, height, GLFW_DONT_CARE);

        // Set the window.
        windows[i] = window;
    }

    // Set the windows.
    setup_result->windows = windows;
}

// Defines what a screenshot should look like.
typedef struct screenshot_t {
    uint8_t* data;
    size_t w;
    size_t h;
};

// Defines all the information needed to render the UI.
typedef struct region_selector_render_ui_info_t {
    size_t window_count;
    GLFWwindow** windows;
    struct screenshot_t* screenshots;
    int active_tool_index;
    bool show_editors;

    // Unused for rendering but used in the event loop.
    struct region_result_t* result;
    bool result_set;
};

// Handles OpenGL fragment shaders.
typedef struct gl_fragment_t {
    char* data;
    char* name;
    GLuint shader;
};

// Does the UI render for a specific window.
void region_selector_render_window(
    struct region_selector_render_ui_info_t* info, size_t window_index,
    bool do_decorations
) {
    // Get the window.
    GLFWwindow* window = info->windows[window_index];

    // Set the viewport to the size of the window.
    int width, height;
    glfwGetFramebufferSize(window, &width, &height);
    glViewport(0, 0, width, height);

    // Set up orthographic projection since we're rendering UI.
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    glOrtho(0, width, height, 0, -1, 1);
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();

    // Create and bind texture.
    GLuint textureID;
    glGenTextures(1, &textureID);
    glBindTexture(GL_TEXTURE_2D, textureID);

    // Load screenshot data into a texture.
    struct screenshot_t* s = &info->screenshots[window_index];
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, s->w, s->h, 0, GL_RGBA, GL_UNSIGNED_BYTE, s->data);

    // Set texture parameters.
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    // Render textured quad.
    glBegin(GL_QUADS);
    glTexCoord2f(0, 0); glVertex2f(0, 0);
    glTexCoord2f(1, 0); glVertex2f(width, 0);
    glTexCoord2f(1, 1); glVertex2f(width, height);
    glTexCoord2f(0, 1); glVertex2f(0, height);
    glEnd();

    // Clean up the texture.
    glDeleteTextures(1, &textureID);

    // Swap the buffers.
    glFlush();
    glfwSwapBuffers(window);
}

// Does the initial UI render for each window.
void region_selector_render_ui(struct region_selector_render_ui_info_t* info) {
    // Enable textures and blending.
    glEnable(GL_TEXTURE_2D);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    // Go through each window and render it.
    for (size_t i = 0; i < info->window_count; i++)
        region_selector_render_window(info, i, true);

    // Disable textures and blending.
    glDisable(GL_BLEND);
    glDisable(GL_TEXTURE_2D);
}

// Flips a RGBA image so that it's in the correct orientation.
void region_selector_flip_rgba(uint8_t* rgba, size_t rgba_len, uint32_t w) {
    uint8_t* temp = (uint8_t*)malloc(w * 4);
    for (size_t i = 0; i < rgba_len / 2; i += w * 4) {
        size_t j = rgba_len - i - w * 4;
        memcpy(temp, rgba + i, w * 4);
        memcpy(rgba + i, rgba + j, w * 4);
        memcpy(rgba + j, temp, w * 4);
    }
    free(temp);
}

// Handles rendering the UI without decorations, screenshotting the specified region,
// and returning the result.
struct region_result_t* region_selector_generate_screenhot(
    struct region_selector_render_ui_info_t* info, size_t window_index,
    uint32_t w, uint32_t h, struct region_coordinate_t coordinate
) {
    // Enable textures and blending.
    glEnable(GL_TEXTURE_2D);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    // Render the UI without decorations.
    region_selector_render_window(info, window_index, false);

    // Disable textures and blending.
    glDisable(GL_BLEND);
    glDisable(GL_TEXTURE_2D);

    // Screenshot the region with OpenGL.
    uint8_t* rgba = (uint8_t*)malloc(w * h * 4);
    glReadPixels(coordinate.x, coordinate.y, w, h, GL_RGBA, GL_UNSIGNED_BYTE, rgba);

    // Create the result.
    struct region_result_t* result = (struct region_result_t*)malloc(sizeof(struct region_result_t));
    size_t rgba_len = w * h * 4;
    region_selector_flip_rgba(rgba, rgba_len, w);
    result->coordinate = coordinate;
    result->w = w;
    result->h = h;
    result->rgba = rgba;
    result->rgba_len = rgba_len;
    result->display_index = window_index;

    // Return the result.
    return result;
}

// Handles the glfw events.
void region_selector_handle_events(struct region_selector_render_ui_info_t* info) {
    // Poll the events via glfw.
    glfwPollEvents();

    // Go through each window.
    for (int i = 0; i < info->window_count; i++) {
        // Get the window.
        GLFWwindow* window = info->windows[i];

        if (
            // Handle if we got a close signal for whatever reason.
            glfwWindowShouldClose(window) ||

            // Handle exiting the region selector via the escape key.
            glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS
        ) {
            info->result = NULL;
            info->result_set = true;
            return;
        }

        // Handle the F key being pressed.
        if (glfwGetKey(window, GLFW_KEY_F) == GLFW_PRESS) {
            // TODO: handle if mouse is on another monitor.
            struct region_coordinate_t fullscreen_coordinate = {0, 0};
            int screen_w, screen_h;
            glfwGetWindowSize(window, &screen_w, &screen_h);
            info->result = region_selector_generate_screenhot(
                info, i, screen_w, screen_h, fullscreen_coordinate
            );
            info->result_set = true;
            return;
        }
    }
}

// Frees the region selector.
void region_selector_free(struct region_selector_render_ui_info_t* info) {
    // Go through each window.
    for (int i = 0; i < info->window_count; i++) {
        // Get the window.
        GLFWwindow* window = info->windows[i];

        // Destroy the window.
        glfwDestroyWindow(window);
    }

    // Terminate glfw.
    glfwTerminate();
}

// Defines opening the region selector and managing the event loop.
struct region_result_t* region_selector_open(
    size_t display_count, struct region_coordinate_t* coordinates,
    struct screenshot_t* screenshots, struct gl_fragment_t* fragments,
    bool show_editors
) {
    // TODO: compile the fragment shaders.

    // Call the setup function.
    struct region_selector_setup_result_t* DO_NOT_USE_setup_result = (struct region_selector_setup_result_t*)calloc(1, sizeof(struct region_selector_setup_result_t));
    DO_NOT_USE_setup_result->display_count = display_count;
    DO_NOT_USE_setup_result->coordinates = coordinates;
    region_selector_main_thread_block(region_selector_glfw_setup, DO_NOT_USE_setup_result);

    // Get the result.
    GLFWmonitor** monitors = DO_NOT_USE_setup_result->monitors;
    GLFWwindow** windows = DO_NOT_USE_setup_result->windows;
    free(DO_NOT_USE_setup_result);

    // If there are no monitors, return null.
    if (monitors == NULL) return NULL;

    // Allocate the info for the UI.
    struct region_selector_render_ui_info_t* info = (struct region_selector_render_ui_info_t*)calloc(1, sizeof(struct region_selector_render_ui_info_t));
    info->window_count = display_count;
    info->windows = windows;
    info->screenshots = screenshots;
    info->show_editors = show_editors;
    info->active_tool_index = 0;

    // Do the initial UI render.
    region_selector_main_thread_block(region_selector_render_ui, info);

    // Do the event loop.
    struct region_result_t* result;
    for (;;) {
        // Call the event loop function in the main thread.
        region_selector_main_thread_block(region_selector_handle_events, info);

        // If the result is set, break here.
        if (info->result_set) {
            result = info->result;
            break;
        }

        // Sleep for 1 second / 120fps.
        usleep(1000000 / 120);
    }

    // Kill the windows.
    region_selector_main_thread_block(region_selector_free, info);

    // Return the result.
    free(info);
    return result;
}

#endif // _ENGINE_C
