import callBridge from "./implementation";

// Get the HTML for all the captures.
export async function getCapturesHtml(query: string) {
    return new TextDecoder().decode(await callBridge("captures_html", query));
}

// Defines a API error.
export class APIError extends Error {
    userFacing: boolean;

    constructor(message: string, userFacing: boolean) {
        super(message);
        this.userFacing = userFacing;
    }
}

// Defines the low level API response.
type APIResponse = {
    err: {
        message: string,
        user_facing: boolean,
    } | null;
    data: any;
};

// Get the JSON from a response.
async function getJson(r: string) {
    const json = JSON.parse(r) as APIResponse;
    if (json.err) {
        throw new APIError(json.err.message, json.err.user_facing);
    }
    return json.data;
}

// Defines the base requestor.
async function baseRequestor(type: string, params?: {[key: string]: string}) {
    params = params || {};
    params["_t"] = type;
    return getJson(
        new TextDecoder().decode(await callBridge("api", JSON.stringify(params))),
    );
}

// Handles deleting a capture.
export async function deleteCapture(id: string) {
    await baseRequestor("delete_capture", { id });
}

// Opens the URL for a capture.
export async function openUrl(id: string) {
    await baseRequestor("open_url", { id });
}

// Opens a file for a capture.
export async function openFile(id: string) {
    await baseRequestor("open_file", { id });
}

// Show a capture in the file manager.
export async function showInFolder(id: string) {
    await baseRequestor("show_in_folder", { id });
}

// Copies a URL to the clipboard.
export async function copyUrl(id: string) {
    const url: string | null = await baseRequestor("get_url", { id });
    if (url) {
        // Use the standard clipboard API in JS to do this for ease of use reasons. Let Tim Apple's
        // API do this.
        await navigator.clipboard.writeText(url);
    }
}

// Sets a configuration value.
export async function setConfigOption(key: string, value: any) {
    await baseRequestor("set_config_option", { key, value });
}

// Deletes a configuration value.
export async function deleteConfigOption(key: string) {
    await baseRequestor("delete_config_option", { key });
}

// Gets a configuration value.
export async function getConfigOption(key: string) {
    return baseRequestor("get_config_option", { key });
}

// Sets a upload configuration value.
export async function setUploaderConfigOption(uploaderId: string, key: string, value: any) {
    await baseRequestor("set_uploader_config_option", { uploaderId, key, value });
}

// Deletes a upload configuration value.
export async function deleteUploaderConfigOption(uploaderId: string, key: string) {
    await baseRequestor("delete_uploader_config_option", { uploaderId, key });
}

// Gets a uploaders configuration items.
export async function getUploaderConfigOptions(uploaderId: string): Promise<{[key: string]: any}> {
    return baseRequestor("get_uploader_config_options", { uploaderId });
}

// Opens a folder selector and returns the folder path.
export async function selectFolder(): Promise<string | null> {
    return baseRequestor("select_folder");
}

// Opens a file selector and returns the file contents.
export async function selectFile(): Promise<string | null> {
    return baseRequestor("select_file");
}

// Defines the base string options.
type StringOptionBase = {
    name: string;
    description: string;
    default: string | null;
    required: boolean;
};

// Defines a config option.
export type ConfigOption = ({
    option_type: "string";
    password: boolean;
    regex: string | null;
    validation_error_message: string | null;
} & StringOptionBase) | ({
    option_type: "long_string";
} & StringOptionBase) | {
    option_type: "number";
    name: string;
    description: string;
    default: number | null;
    min: number | null;
    max: number | null;
    required: boolean;
} | {
    option_type: "boolean";
    name: string;
    description: string;
    default: boolean | null;
    required: boolean;
} | {
    option_type: "custom";
    name: string;
    description: string;
    frame_html: string;
} | {
    option_type: "embedded";
    name: string;
    description: string;
    component_name: string;
    required: boolean;
};

// Defines how a uploader is structured.
export type Uploader = {
    name: string;
    description: string;
    icon_path: string;
    options: {[id: string]: ConfigOption};
};

// Gets the uploaders.
export async function getUploaders(): Promise<{[id: string]: Uploader}> {
    return baseRequestor("get_uploaders");
}

// Defines the callback for hotkeys.
export let hotkeyCallback: (hotkey: string) => void | null = null;

// Gets the hotkey callback.
export function getHotkeyCallback() {
    return hotkeyCallback;
}

// Starts the hotkey capture.
export async function startHotkeyCapture(cb: (hotkey: string) => void) {
    hotkeyCallback = cb;
    await baseRequestor("start_hotkey_capture");
}

// Stops the hotkey capture.
export async function stopHotkeyCapture() {
    hotkeyCallback = null;
    await baseRequestor("stop_hotkey_capture");
}

// Allows you to test a uploaders configuration.
export async function testUploader(id: string) {
    await baseRequestor("test_uploader", { id });
}
