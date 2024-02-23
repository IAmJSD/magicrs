import { useId, useState, useEffect, useCallback } from "react";
import type { ZodError, ZodType } from "zod";
import { getConfigOption, setConfigOption, setUploaderConfigOption } from "../../../bridge/api";
import Alert from "../Alert";
import Description from "../Description";

type Props = {
    uploader?: {
        id: string;
        items: {[key: string]: any};
    };
    dbKey: string;
    label: string;
    password?: boolean;
    description?: string;
    defaultValue?: string;
    validator?: ZodType<string>;
};

export default function Textbox({
    uploader, dbKey, label, description, defaultValue, validator,
    password,
}: Props) {
    const labelId = useId();
    const [value, setValue] = useState(defaultValue);
    const [validationError, setValidationError] = useState<ZodError | null>(null);

    useEffect(() => {
        if (uploader) {
            const val = uploader.items[dbKey];
            if (typeof val === "string") setValue(val);
            return;
        }

        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (cancelled) return;
            if (typeof val === "string") setValue(val);
        });
        return () => { cancelled = true; };
    }, [uploader, dbKey]);

    const cb = useCallback((value: string) => {
        // Update the local state.
        setValue(value);

        // Validate the input if we have a validator.
        if (validator) {
            try {
                validator.parse(value);
                setValidationError(null);
            } catch (e) {
                setValidationError(e as ZodError);
                return;
            }
        }

        // If we got this far, update the items in the config object.
        if (uploader) uploader.items[dbKey] = value;

        // Do the async update.
        (
            uploader ? setUploaderConfigOption(uploader.id, dbKey, value) : setConfigOption(dbKey, value)
        ).catch(e => {
            setValue(defaultValue);
            throw e;
        });
    }, [validator]);

    return <form autoComplete="off" className="block m-2" onSubmit={e => e.preventDefault()}>
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            {
                description && <Description description={description} />
            }

            {
                validationError && <div style={{maxWidth: "60%"}}>
                    <Alert type="error" message={validationError.errors[0].message} />
                </div>
            }
        </div>

        <div>
            <input
                type={password ? "password" : "text"} placeholder={label}
                className="w-3/5 mt-2 dark:bg-zinc-800 bg-slate-50 p-1 rounded-lg"
                value={value} aria-labelledby={labelId}
                onChange={e => cb(e.target.value)}
            />
        </div>
    </form>;
}
