import { useId, useState, useEffect, useCallback } from "react";
import { getConfigOption, setConfigOption, setUploaderConfigOption } from "../../../bridge/api";

type Props = {
    uploader?: {
        id: string;
        items: {[key: string]: any};
    };
    defaultValue: boolean;
    dbKey: string;
    label: string;
};

export default function Checkbox({ uploader, defaultValue, dbKey, label }: Props) {
    const id = useId();
    const [checked, setChecked] = useState(defaultValue);

    useEffect(() => {
        if (uploader) {
            const val = uploader.items[dbKey];
            if (typeof val === "boolean") setChecked(val);
            return;
        }

        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (cancelled) return;
            if (typeof val === "boolean") setChecked(val);
        });
        return () => { cancelled = true; };
    }, [uploader, dbKey]);

    const cb = useCallback((checked: boolean) => {
        // Do the sync updates.
        setChecked(checked);
        if (uploader) uploader.items[dbKey] = checked;

        // Do the async update.
        (
            uploader ? setUploaderConfigOption(uploader.id, dbKey, checked) : setConfigOption(dbKey, checked)
        ).catch(e => {
            setChecked(!checked);
            throw e;
        });
    }, []);

    // Return the form that controls the checkbox.
    return <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
        <label htmlFor={id} className="flex items-center align-middle">
            <input
                type="checkbox" className="mr-1"
                checked={checked}
                onChange={e => cb(e.target.checked)}
                id={id}
            />

            {label}
        </label>
    </form>;
}
