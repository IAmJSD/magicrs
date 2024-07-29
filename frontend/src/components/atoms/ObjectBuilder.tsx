import { type FC, useCallback, useEffect, useMemo, useRef, useState } from "react";
import Button from "./Button";

export type KeyedObject<T> = {
    keys: () => string[];
    get: (key: string) => T | undefined;
    set: (key: string, value: T) => void;
    delete: (key: string) => void;
    entries: () => [string, T][];
    underlyingObject: any;
};

export function wrapObject<T>(obj: { [key: string]: T }): KeyedObject<T> {
    return {
        keys: () => Object.keys(obj).sort(([a], [b]) => a.localeCompare(b)),
        get: key => obj[key],
        set: (key, value) => obj[key] = value,
        delete: key => delete obj[key],
        entries: () => Object.entries(obj),
        underlyingObject: obj,
    };
}

export function wrapPreSortedArray<T>(arr: [string, T][]): KeyedObject<T> {
    return {
        keys: () => arr.map(([key]) => key),
        get: key => arr.find(([k]) => k === key)?.[1],
        set: (key, value) => {
            const index = arr.findIndex(([k]) => k === key);
            if (index === -1) arr.push([key, value]);
            else arr[index][1] = value;
        },
        delete: key => {
            const index = arr.findIndex(([k]) => k === key);
            if (index !== -1) arr.splice(index, 1);
        },
        entries: () => [...arr],
        underlyingObject: arr,
    };
}

export function wrapMap<T>(map: Map<string, T>): KeyedObject<T> {
    return {
        keys: () => Array.from(map.keys()).sort(([a], [b]) => a.localeCompare(b)),
        get: key => map.get(key),
        set: (key, value) => map.set(key, value),
        delete: key => map.delete(key),
        entries: () => Array.from(map.entries()),
        underlyingObject: map,
    };
}

type ObjectBuilderRowProps<T> = {
    obj: KeyedObject<T>;
    kv: [string, T];
    componentsWithIrrelevantBitNoRealloc: (readonly [number, string, FC<{}>])[];
    deleteRow: () => void;
    callbackMap: Map<number, [number, () => boolean]>;
    setOk: (ok: boolean) => void;
    columns: string[];
    index: number;
};

let nextId = 0;

function ObjectBuilderRow<T>({
    obj, kv, componentsWithIrrelevantBitNoRealloc, deleteRow, callbackMap, setOk, columns,
    index,
}: ObjectBuilderRowProps<T>) {
    // Defines the ID state.
    const [id, setId] = useState(kv[0]);

    // Defines the ok checker.
    useEffect(() => {
        const cbId = nextId++;
        callbackMap.set(cbId, [index, () => id === kv[0] && id !== ""]);
        return () => void callbackMap.delete(cbId);
    }, [id, kv, callbackMap, index]);

    // Edits the key in the object.
    const editKey = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        // Set the local state to the new value.
        setId(e.target.value);

        // Deconstruct the key-value pair.
        const [key, value] = kv;

        // Set the value within the object.
        const newKey = e.target.value;
        if (newKey === key) return;
        if (newKey === "" || obj.get(newKey)) {
            setOk(false);
            return;
        }
        obj.delete(key);
        obj.set(newKey, value);
        kv[0] = newKey;

        // Set ok to if all the callbacks return true.
        setOk([...callbackMap.values()].every(([, cb]) => cb()));
    }, [obj, callbackMap, setOk, kv]);

    // Build the row.
    return <tr>
        <td className="p-1">
            <form autoComplete="off" onSubmit={e => e.preventDefault()}>
                <input
                    type="text"
                    placeholder="Key"
                    value={id}
                    onChange={editKey}
                    className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
                />
            </form>
        </td>
        {
            columns.map((key, i) => {
                const [,, Component] = componentsWithIrrelevantBitNoRealloc.find(
                    ([, k]) => k === key,
                ) || [0, "", (() => <td></td>)];
                return <td key={i} className="p-1">
                    <Component />
                </td>;
            })
        }
        <td className="p-1">
            <Button
                color="danger"
                onClick={deleteRow}
            >
                <span aria-label="Remove">
                    <i className="fas fa-trash" />
                </span>
            </Button>
        </td>
    </tr>;
}

type Destruct = () => void;
type RecheckOkState = () => void;
export type InnerProps<T> = {
    value: T;
    onColumnsEdit: () => void;
    deleteRow: () => void;
    addOkCallback: (cb: () => boolean) => readonly [Destruct, RecheckOkState];
};

export function useValueOkHandler(
    defaultOk: boolean, addOkCallback: (cb: () => boolean) => readonly [Destruct, RecheckOkState],
) {
    const ref = useRef<[() => void, boolean]>([() => {}, defaultOk]);

    useEffect(() => {
        const [destruct, recheck] = addOkCallback(() => ref.current[1]);
        ref.current[0] = recheck;
        return destruct;
    }, [addOkCallback]);

    return (ok: boolean) => {
        ref.current[1] = ok;
        ref.current[0]();
    };
}

export type Props<T> = {
    obj: KeyedObject<T>;
    setOk: (ok: boolean) => void;
    newInstance: () => T;
    componentsBuilder: (value: T, key: string) => [number, string, FC<InnerProps<T>>][];
};

export function ObjectBuilder<T>({ obj, setOk, newInstance, componentsBuilder }: Props<T>) {
    // Build the object as an array of key-value pairs.
    const [objAsArrayPtr, setObjAsArrayPtr] = useState<[[string, T][]]>(() => [obj.entries()]);

    // If the underlying object changes, update the array.
    useEffect(() => setObjAsArrayPtr([obj.entries()]), [obj.underlyingObject]);

    // Force a re-render when the object changes.
    const changePtr = () => setObjAsArrayPtr(a => [a[0]]);

    // Handles pushing or deleting items.
    const pushItem = useCallback(() => {
        const [objAsArray] = objAsArrayPtr;
        objAsArray.push(["", newInstance()]);
        changePtr();
        setOk(false);
    }, objAsArrayPtr);
    const deleteItem = useCallback((index: number) => {
        const [objAsArray] = objAsArrayPtr;
        const k = objAsArray[index][0];
        obj.delete(k);
        objAsArray.splice(index, 1);
        changePtr();

        // Check if everything is ok in case this was a not-ok row that changes things.
        setOk([...callbackMap.values()].every(([i, cb]) => {
            if (i !== index) return cb();
            return true;
        }));
    }, objAsArrayPtr);

    // Handles transforming the object into a list of components.
    const [callbackMap] = useState(() => new Map<number, [number, () => boolean]>());
    const objArrAsComponents = useMemo(
        () => {
            const [objAsArray] = objAsArrayPtr;
            return objAsArray.map(([key, value], i) => {
                const components = componentsBuilder(value, key);
                return components.map(([weight, name, Component]) => [weight, name, () => (
                    <Component
                        key={`${i}-${weight}`}
                        value={value}
                        onColumnsEdit={changePtr}
                        deleteRow={() => deleteItem(i)}
                        addOkCallback={cb => {
                            const id = nextId++;
                            callbackMap.set(id, [i, cb]);
                            return [
                                () => callbackMap.delete(id),
                                () => {
                                    let ok = cb();
                                    if (ok) {
                                        // Check if everything else is ok too.
                                        ok = [...callbackMap.values()].every(([, cb]) => cb());
                                    }
                                    setOk(ok);
                                },
                            ];
                        }}
                    />
                )] as const);
            });
        },
        [objAsArrayPtr, componentsBuilder, callbackMap, setOk],
    );

    // Get all the table columns as keys.
    const columns = useMemo(
        () => {
            const flattened = objArrAsComponents.flatMap(x => x);
            return flattened.filter(([_, key], i) => {
                return flattened.findIndex(([_, k]) => k === key) === i;
            }).sort(([aWeight, aKey], [bWeight, bKey]) => {
                if (aWeight !== bWeight) return aWeight - bWeight;
                return aKey.localeCompare(bKey);
            }).map(([_, key]) => key);
        },
        [objArrAsComponents],
    );

    // Build the table.
    const [objAsArray] = objAsArrayPtr;
    return <>
        {
            objAsArray.length !== 0 && <table className="w-full mb-4">
                <thead>
                    <tr>
                        <th className="p-1">Key</th>
                        {columns.map((key, i) => <th key={i} className="p-1">{key}</th>)}
                        <th className="p-1"></th>
                    </tr>
                </thead>

                <tbody>
                    {
                        objAsArray.map((arrPtr, i) => {
                            return <ObjectBuilderRow<T>
                                key={i} kv={arrPtr} componentsWithIrrelevantBitNoRealloc={objArrAsComponents[i]}
                                deleteRow={() => deleteItem(i)} callbackMap={callbackMap}
                                setOk={setOk} columns={columns} obj={obj} index={i}
                            />;
                        })
                    }
                </tbody>
            </table>
        }

        <Button
            color="primary"
            onClick={pushItem}
        >
            Add Row
        </Button>
    </>;
}
