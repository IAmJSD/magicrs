import callBridge from "./implementation";

// Proxy a filesystem load through the main process. Returns the data URI to load.
export async function fileSystemProxy(path: string) {
    const res = await callBridge("fs_proxy", path);

    const r = res[0];
    if (r === 1) {
        return {
            ok: true as const,
            data: new TextDecoder().decode(res.subarray(1)),
        };
    }

    return { ok: false as const };
}
