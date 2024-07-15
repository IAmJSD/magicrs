import { toBytes } from "fast-base64";
import { dispatchCaptureHtml } from "./newCaptures";
import { getHotkeyCallback } from "./api";

// Defines a map of promises.
const promises = new Map<number, (data: Uint8Array) => void>();

// Defines the next ID.
let nextId = 0;

// Defines persistent bridge handlers. These MUST be negative.
const persistentHandlers = new Map<number, (data: Uint8Array) => void>([
    [-1, dispatchCaptureHtml],
    [-2, val => getHotkeyCallback()?.(new TextDecoder().decode(val))],
]);

// @ts-expect-error: Handling the bridge response.
window.bridgeResponse = function bridgeResponse(id: number, data: string) {
    if (id < 0) {
        const x = persistentHandlers.get(id);
        toBytes(data).then(data => {
            if (x) x(data);
        });
        return;
    }

    const x = promises.get(id);
    if (!x) return;
    toBytes(data).then(data => x(data));
    promises.delete(id);
};

// @ts-expect-error: We know one of these has to be true.
const messageHandler = window.webkit ? window.webkit.messageHandlers.bridge : window.chrome.webview;

// The low level API to call the bridge.
export default function callBridge(type: string, content: string) {
    const id = nextId++;
    return new Promise<Uint8Array>(res => {
        promises.set(id, res);
        messageHandler.postMessage(`${id}\n${type}\n${content}`);
    });
}
