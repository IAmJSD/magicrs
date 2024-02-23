import { useId, useState, useEffect, useCallback } from "react";
import { getConfigOption, setConfigOption, setUploaderConfigOption } from "../../../bridge/api";
import Description from "../Description";

type Props = {
    uploader?: {
        id: string;
        items: {[key: string]: any};
    };
    dbKey: string;
    label: string;
    description?: string;
    defaultValue?: number;
    min?: number;
    max?: number;
};

export default function NumberInput({
    uploader, dbKey, label, description, defaultValue, min, max,
}: Props) {
    const labelId = useId();
    const [value, setValue] = useState(defaultValue);

    useEffect(() => {
        if (uploader) {
            const val = uploader.items[dbKey];
            if (typeof val === "number") setValue(val);
            return;
        }

        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (cancelled) return;
            if (typeof val === "number") setValue(val);
        });
        return () => { cancelled = true; };
    }, [uploader, dbKey]);

    const cb = useCallback((value: number) => {
        setValue(value);
        if (uploader) uploader.items[dbKey] = value;
        (
            uploader ? setUploaderConfigOption(uploader.id, dbKey, value) : setConfigOption(dbKey, value)
        ).catch(e => {
            setValue(defaultValue);
            throw e;
        });
    }, []);

    return <form autoComplete="off" className="block m-2" onSubmit={e => e.preventDefault()}>
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            {
                description && <Description description={description} />
            }
        </div>

        <div>
            <input
                type="number" min={min} max={max} placeholder={label}
                className="w-3/5 mt-2 dark:bg-zinc-800 bg-slate-50 p-1 rounded-lg"
                value={value} aria-labelledby={labelId}
                onChange={e => cb(e.target.valueAsNumber)}
            />
        </div>
    </form>;
}
