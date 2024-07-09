import { useCallback, useEffect, useState } from "react";
import { deleteConfigOption, deleteUploaderConfigOption, getConfigOption, setConfigOption, setUploaderConfigOption } from "../../../bridge/api";
import Description from "../Description";

// Get the embedded map.
const embeddedMap = import("../../embedded_customs").then(module => module.default);

// Defines a bit of TS to take the promise type and resolve it.
type Resolved<T> = T extends Promise<infer U> ? U : T;

// Defines the map keys.
type MapKeys = keyof Resolved<typeof embeddedMap>;

// Defines the props for the embedded handler.
type Props = {
    uploader?: {
        id: string;
        items: {[key: string]: any};
    };
    componentName: MapKeys;
    dbKey: string;
    label: string;
    description?: string;
};

// Defines a unset item.
const UNSET = Symbol("UNSET");

// Define the embedded handler.
export default function Embedded({ uploader, componentName, dbKey, label, description }: Props) {
    const [value, setValue] = useState(UNSET as typeof UNSET | [any, any]);

    useEffect(() => {
        Promise.all([
            embeddedMap.then(e => e[componentName]),
            (async () => {
                if (uploader) {
                    const val = uploader.items[dbKey];
                    if (val !== undefined) return val;
                }

                return await getConfigOption(dbKey);
            })(),
        ]).then(v => setValue(v));
    }, [componentName, dbKey]);

    // Defines the set callback.
    const set = useCallback((value: any) => {
        if (uploader) {
            if (value === undefined) {
                delete uploader.items[dbKey];
                return deleteUploaderConfigOption(uploader.id, dbKey);
            }
            uploader.items[dbKey] = value;
            return setUploaderConfigOption(uploader.id, dbKey, value);
        }
        if (value === undefined) {
            return deleteConfigOption(dbKey);
        }
        return setConfigOption(dbKey, value);
    }, [uploader, dbKey]);

    // Return nothing whilst this loads.
    if (value === UNSET) return null;

    // Return the embedded component with a description.
    const [Embedded, itemValue] = value;
    return <>
        <p className="mb-1 font-semibold">
            {label}
        </p>

        {
            description && <Description description={description} />
        }

        <Embedded value={itemValue} set={set} />
    </>;
}
