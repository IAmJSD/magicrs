import { useState, useEffect, type DependencyList } from "react";

type PromiseResult<T> = [T, "resolved"] | [{error: any}, "rejected"] | [undefined, "loading"];

export default function usePromise<T>(promise: () => Promise<T>, deps: DependencyList): PromiseResult<T> {
    const [value, setValue] = useState<PromiseResult<T>>([undefined, "loading"]);

    useEffect(() => {
        let cancelled = false;
        promise().then(value => {
            if (cancelled) return;
            setValue([value, "resolved"]);
        }, error => {
            if (cancelled) return;
            setValue([{ error }, "rejected"]);
        });
        return () => {
            cancelled = true;
        };
    }, deps);

    return value;
}
