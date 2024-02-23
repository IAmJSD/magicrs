// Defines watchers for captures.
const watchers = new Map<number, (html: string) => void>();

// Defines the next ID.
let nextId = 0;

// Adds a watcher for captures.
export function addCaptureWatcher(watcher: (html: string) => void) {
    const id = nextId++;
    watchers.set(id, watcher);
    return id;
}

// Removes a watcher for captures.
export function removeCaptureWatcher(id: number) {
    watchers.delete(id);
}

// Handles watching for captures.
export function dispatchCaptureHtml(htmlU8: Uint8Array) {
    for (const watcher of watchers.values()) {
        watcher(new TextDecoder().decode(htmlU8));
    }
}
