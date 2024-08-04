import { useId, useState, useEffect, useCallback } from "react";
import { getConfigOption, setConfigOption, setUploaderConfigOption } from "../../../bridge/api";
import Description from "../Description";

type Props = {
    uploader?: {
        id: string;
        items: { [key: string]: any };
    };
    dbKey: string;
    label: string;
    description?: string;
    defaultValue?: string;
};

export default function ColorInput({
    uploader, dbKey, label, description, defaultValue,
}: Props) {
    const labelId = useId();
    const [value, setValue] = useState(defaultValue);

    useEffect(() => {
        if (uploader) {
            const val = uploader.items[dbKey];
            if (Array.isArray(val)) {
                // Check if this is a valid color array and if so set the value to the hex representation of the color.
                if (val.length === 3 && val.every(v => typeof v === "number" && v >= 0 && v <= 255)) {
                    setValue(`#${val.map(v => v.toString(16).padStart(2, "0")).join("")}`);
                }
            }
            return;
        }

        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (cancelled) return;
            if (Array.isArray(val)) {
                // Check if this is a valid color array and if so set the value to the hex representation of the color.
                if (val.length === 3 && val.every(v => typeof v === "number" && v >= 0 && v <= 255)) {
                    setValue(`#${val.map(v => v.toString(16).padStart(2, "0")).join("")}`);
                }
            }
        });
        return () => { cancelled = true; };
    }, [uploader, dbKey]);

    const cb = useCallback((value: string) => {
        setValue(value);
        if (uploader) uploader.items[dbKey] = value;
        const dbValue = [
            parseInt(value.slice(1, 3), 16),
            parseInt(value.slice(3, 5), 16),
            parseInt(value.slice(5, 7), 16),
        ];
        (
            uploader ? setUploaderConfigOption(uploader.id, dbKey, dbValue) : setConfigOption(dbKey, dbValue)
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
                type="color"
                className="w-7 h-7 mt-2 dark:bg-zinc-800 bg-slate-50 p-1 rounded-lg"
                value={value} aria-labelledby={labelId}
                onChange={e => cb(e.target.value)}
            />
        </div>
    </form>;
}
