#ifndef MACOS_SUPPORT_H_MAGICCAP
#define MACOS_SUPPORT_H_MAGICCAP
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <AppKit/AppKit.h>
#include <UserNotifications/UserNotifications.h>

typedef struct {
    const char* name;
    const char* id;
    bool default_uploader;
} UploaderItem;

typedef struct {
    const char* name;
    int type;
    // TODO: Add modifiers
} CaptureType;

@interface MagicCapMenuItem : NSMenuItem
@property (nonatomic, copy) void (^callback)();

- (id)initWithTitle:(NSString *)string 
    callback:(void (^)())callback
    keyEquivalent:(NSString *)charCode;
@end

@interface MagicCapNotificationDelegate : NSObject
- (void)userNotificationCenter:(UNUserNotificationCenter *)center 
    didReceiveNotificationResponse:(UNNotificationResponse *)response 
    withCompletionHandler:(void (^)(void))completionHandler;
@end

size_t open_file_dialog(bool folder);
void copy_file_to_clipboard(const char* file_path, const char* filename, uint8_t* data, size_t data_len);
void send_ok_dialog(const char* message);
void hook_notif_center();
void transform_process_type(bool show);
void tray_capture_buttons(
    NSMenu* menu, void (*on_capture_type_clicked)(int), CaptureType* capture_types,
    size_t capture_types_len
);
size_t create_tray(
    UploaderItem* uploader_items, size_t uploader_items_len, CaptureType* capture_types,
    size_t capture_types_len,
    void (*on_click)(uint8_t* name_ptr, size_t name, uint8_t* path_ptr, size_t path),
    void (*on_quit)(),
    void (*on_capture_type_clicked)(int),
    void (*on_config)()
);

#endif // MACOS_SUPPORT_H_MAGICCAP
