import { useAtom } from "jotai";
import { useCallback, useEffect, useId, useState } from "react";
import { hotkeyAtom } from "../../../atoms";
import { getConfigOption, startHotkeyCapture, stopHotkeyCapture } from "../../../bridge/api";
import Description from "../Description";
import Button from "../Button";

type Props = {
    dbKey: string;
    label: string;
    description: string;
};

export default function Hotkey({ dbKey, label, description }: Props) {
    const [activeHotkey, setActiveHotkey] = useAtom(hotkeyAtom);
    const [hotkey, setHotkey] = useState("--");
    const labelId = useId();

    // Get the hotkey from the database.
    useEffect(() => {
        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (typeof val === "string") {
                if (!cancelled) setHotkey(val);
            }
        });
        return () => { cancelled = true; };
    }, [dbKey]);

    // Handle the hotkey capture.
    useEffect(() => {
        // If we are not capturing, return.
        if (activeHotkey !== dbKey) return;

        // Start capturing the hotkey.
        startHotkeyCapture(hotkey => {
            setHotkey(hotkey);
            setActiveHotkey(null);
        });

        // Handle stopping when the component is unmounted if we are capturing.
        return () => {
            if (activeHotkey === dbKey) setActiveHotkey(null);
        };
    }, [activeHotkey, dbKey]);

    // Defines the callback to setup the state for the hotkey capture.
    const cb = useCallback(() => {
        // If this is the active hotkey, clear it and stop capturing.
        if (activeHotkey === dbKey) {
            setActiveHotkey(null);
            stopHotkeyCapture();
            return;
        }

        // Set the active hotkey to ourselves.
        setActiveHotkey(dbKey);
    }, [activeHotkey, dbKey]);

    // Return the UI to capture the hotkey.
    return <div className="block m-2">
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            <Description description={description} />
        </div>

        <div aria-labelledby={labelId} className="flex mt-4">
            <div className="flex-col my-auto">
                {hotkey}
            </div>
            <div className="flex-col ml-2">
                <Button onClick={cb}>
                    {activeHotkey === dbKey ? "Stop" : "Start"} Capturing
                </Button>
            </div>
        </div>
    </div>;
}
