import { useSyncExternalStore } from "react";

export default function useDarkMode() {
    const m = window.matchMedia("(prefers-color-scheme: dark)");
    return useSyncExternalStore(
        storeChange => {
            m.addEventListener("change", storeChange);
            return () => m.removeEventListener("change", storeChange);
        },
        () => m.matches,
    );
}
