#ifndef MACOS_SUPPORT_M_MAGICCAP
#define MACOS_SUPPORT_M_MAGICCAP
#include <stddef.h>
#include <stdint.h>
#include <dispatch/dispatch.h>
#include <AppKit/AppKit.h>
#include <UserNotifications/UserNotifications.h>
#include "macos.h"

@implementation MagicCapMenuItem
- (void)cb_handler {
    self.callback();
}

- (id)initWithTitle:(NSString *)string
    callback:(void (^)())cb
    keyEquivalent:(NSString *)charCode
{
    self = [super initWithTitle:string action:@selector(cb_handler) keyEquivalent:charCode];
    if (self) {
        self.callback = cb;
        self.target = self;
    }
    return self;
}
@end

@implementation MagicCapNotificationDelegate
- (void)userNotificationCenter:(UNUserNotificationCenter *)_
    didReceiveNotificationResponse:(UNNotificationResponse *)response
    withCompletionHandler:(void (^)(void))completionHandler {
    // Get the action identifier.
    NSString* actionIdentifier = response.actionIdentifier;

    // If it starts with fp=, open it as a file.
    if ([actionIdentifier hasPrefix:@"fp="]) {
        NSString* path = [actionIdentifier substringFromIndex:3];
        #pragma clang diagnostic push
        #pragma clang diagnostic ignored "-Wdeprecated-declarations"
        [[NSWorkspace sharedWorkspace] openFile:path];
        #pragma clang diagnostic pop
        [path release];
    }

    // If it starts with url=, open it as a URL.
    else if ([actionIdentifier hasPrefix:@"url="]) {
        NSString* url = [actionIdentifier substringFromIndex:4];
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:url]];
        [url release];
    }

    // Call the completion handler.
    completionHandler();
}
@end

size_t open_file_dialog(bool folder) {
    NSURL* __block url_ptr = nil;
    NSCondition* waitHandle = [NSCondition new];
    [waitHandle lock];
    dispatch_async(dispatch_get_main_queue(), ^{
        NSOpenPanel* panel = [NSOpenPanel openPanel];
        [panel setCanCreateDirectories:YES];
        if (folder) {
            [panel setCanChooseDirectories:YES];
            [panel setCanChooseFiles:NO];
        } else {
            [panel setCanChooseDirectories:NO];
            [panel setCanChooseFiles:YES];
        }

        [panel beginSheetModalForWindow:nil completionHandler:^(NSInteger result){
            if (result == NSFileHandlingPanelOKButton) {
                NSArray* urls = [panel URLs];
                for (NSURL *url in urls) {
                    url_ptr = url;
                    [waitHandle signal];
                    return;
                }
            }
            [waitHandle signal];
        }];
    });
    [waitHandle wait];
    return (size_t) url_ptr;
}

void copy_file_to_clipboard(const char* file_path, const char* filename, uint8_t* data, size_t data_len) {
    NSPasteboard* pboard = [NSPasteboard generalPasteboard];

    // If the file path is not null, write that to the clipboard.
    if (file_path != NULL) {
        [pboard clearContents];
        [pboard writeObjects:@[
            [NSURL fileURLWithPath:[NSString stringWithUTF8String:file_path]]
        ]];
        return;
    }

    // Create a NSURL from the data.
    NSString* filename_str = [NSString stringWithUTF8String:filename];
    NSString* temp_dir = NSTemporaryDirectory();
    NSString* temp_file = [temp_dir stringByAppendingPathComponent:filename_str];
    [data writeToFile:temp_file atomically:YES];

    // Write the file to the clipboard.
    [
        [pboard clearContents];
        pboard writeObjects:@[
            [NSURL fileURLWithPath:temp_file]
        ]
    ];

    // Release the strings.
    [filename_str release];
    [temp_dir release];
    [temp_file release];
}

void send_ok_dialog(const char* message) {
    // Setup the message.
    NSString* message_str = [NSString stringWithUTF8String:message];

    // '-[NSAlert runModal] may only be invoked from the main thread. Behavior on other threads is undefined.'
    // Yes, yes, I know. Blocking the main thread until this is done is fucking stupid. Thanks Apple.
    dispatch_async(dispatch_get_main_queue(), ^ {
        // Initialize the alert.
        NSAlert* alert = [[NSAlert alloc] init];

        // Set the message.
        [alert setAlertStyle:NSAlertStyleCritical];
        [alert setMessageText:@"MagicCap"];
        [alert setInformativeText:message_str];

        // Run the alert.
        [alert runModal];
    });
}

void hook_notif_center() {
    id<UNUserNotificationCenterDelegate> delegate = (id<UNUserNotificationCenterDelegate>) [[MagicCapNotificationDelegate alloc] init];
    [
        [UNUserNotificationCenter currentNotificationCenter]
        setDelegate:delegate
    ];
}

void transform_process_type(bool show) {
    ProcessSerialNumber psn = {0, kCurrentProcess};
    if (show)
        TransformProcessType(&psn, kProcessTransformToForegroundApplication);
    else
        TransformProcessType(&psn, kProcessTransformToUIElementApplication);
}

void tray_capture_buttons(
    NSMenu* menu, void (*on_capture_type_clicked)(int), CaptureType* capture_types,
    size_t capture_types_len
) {
    // TODO: Add modifiers.
    // Go through each capture type and add a button for it.
    for (size_t i = 0; i < capture_types_len; i++) {
        CaptureType capture_type = capture_types[i];

        // Create a NSString from name as a C string.
        NSString* name = [NSString stringWithUTF8String:capture_type.name];

        // Add the item to the menu.
        MagicCapMenuItem* captureItem = [
            [MagicCapMenuItem alloc] initWithTitle:name
            callback:^() { on_capture_type_clicked(capture_type.type); }
            keyEquivalent:@""
        ];
        [menu addItem:captureItem];
    }
}

size_t create_tray(
    UploaderItem* uploader_items, size_t uploader_items_len, CaptureType* capture_types,
    size_t capture_types_len,
    void (*on_click)(uint8_t* name_ptr, size_t name, uint8_t* path_ptr, size_t path),
    void (*on_quit)(),
    void (*on_capture_type_clicked)(int),
    void (*on_config)()
) {
    // Create the menu item.
    NSStatusItem* item = [
        [NSStatusBar systemStatusBar] statusItemWithLength:NSVariableStatusItemLength
    ];

    // Create the menu.
    NSMenu* menu = [[NSMenu alloc] initWithTitle:@"MagicCap"];

    // Create the 'Upload to...' submenu.
    NSMenu* uploadMenu = [[NSMenu alloc] initWithTitle:@"Upload to..."];
    NSString* uploadTo = @"Upload to ";
    for (size_t i = 0; i < uploader_items_len; i++) {
        UploaderItem uploader_item = uploader_items[i];

        // Create a NSString from name as a C string.
        NSString* name = [NSString stringWithUTF8String:uploader_item.name];

        // Create a 'Upload to <name>' item.
        NSString* title = [uploadTo stringByAppendingString:name];
        if (uploader_item.default_uploader) {
            title = [title stringByAppendingString:@" (default)"];
        }

        // Create a copy of the id.
        size_t id_len = strlen(uploader_item.id);
        uint8_t* id_copy = malloc(id_len);
        memcpy(id_copy, uploader_item.id, id_len);

        // Add the item to the menu.
        MagicCapMenuItem* uploadItem = [
            [MagicCapMenuItem alloc] initWithTitle:title
            callback:^() {
                NSOpenPanel* openPanel = [NSOpenPanel openPanel];
                [openPanel setCanChooseFiles:YES];
                [openPanel setCanChooseDirectories:NO];
                [openPanel setAllowsMultipleSelection:NO];
                #pragma clang diagnostic push
                #pragma clang diagnostic ignored "-Wnonnull" // It doesn't seem to actually care.
                [openPanel beginSheetModalForWindow:nil completionHandler:^(NSInteger result) {
                    if (result == NSModalResponseOK) {
                        NSURL* url = [openPanel URL];
                        NSString* path = [url path];
                        on_click(id_copy, id_len, (uint8_t*)[path UTF8String], strlen([path UTF8String]));
                    }
                }];
                #pragma clang diagnostic pop
                [openPanel makeKeyAndOrderFront:nil];
            }
            keyEquivalent:@""
        ];
        [uploadMenu addItem:uploadItem];
    }

    // Add the capture options.
    tray_capture_buttons(menu, on_capture_type_clicked, capture_types, capture_types_len);
    [menu addItem:[NSMenuItem separatorItem]];

    // Add the 'Upload to...' submenu to the main menu.
    NSMenuItem* uploadMenuItem = [[NSMenuItem alloc] initWithTitle:@"Upload to..." action:nil keyEquivalent:@""];
    [uploadMenuItem setSubmenu:uploadMenu];
    [menu addItem:uploadMenuItem];

    // Add a divider.
    [menu addItem:[NSMenuItem separatorItem]];

    // Add the config item.
    MagicCapMenuItem* configItem = [
        [MagicCapMenuItem alloc] initWithTitle:@"Captures/Config"
        callback:^() { on_config(); }
        keyEquivalent:@""
    ];
    [menu addItem:configItem];

    // Add the quit item.
    MagicCapMenuItem* quitItem = [
        [MagicCapMenuItem alloc] initWithTitle:@"Quit"
        callback:^() { on_quit(); }
        keyEquivalent:@""
    ];
    [menu addItem:quitItem];

    // Set the menu.
    [item setMenu:menu];

    // Set the title.
    // TODO: Make this an icon.
    #pragma clang diagnostic push
    #pragma clang diagnostic ignored "-Wdeprecated-declarations"
    [item setTitle:@"MagicCap"];
    #pragma clang diagnostic pop

    // Return the pointer to the status item but without the Apple type information (unneeded in Rust).
    return (size_t) item;
}

#endif // MACOS_SUPPORT_M_MAGICCAP
