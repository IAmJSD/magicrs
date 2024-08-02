import { useState } from "react";
import { useAtom } from "jotai";
import usePromise from "../hooks/usePromise";
import { uploaderIdAtom, hotkeyAtom } from "../atoms";
import CrashHandler from "./CrashHandler";

type PanelItem = {
    name: string;
    component: Promise<{ default: React.ComponentType }>;
};

const panels: PanelItem[] = [
    { name: "Captures", component: import("./panels/Captures") },
    { name: "Uploaders", component: import("./panels/Uploaders") },
    { name: "File Saving", component: import("./panels/FileSaving") },
    { name: "Hotkeys", component: import("./panels/Hotkeys") },
    { name: "General", component: import("./panels/General") },
];

const PanelLoader = ({ panelIndex }: { panelIndex: number }) => {
    const [Component, state] = usePromise(
        () => panels[panelIndex].component.then(x => x.default),
        [panelIndex],
    );

    if (state !== "resolved") return <></>;
    return <Component />;
};

type PanelButtonProps = {
    selected: boolean;
    panelName: string;
    setPanel: () => void;
};

function PanelButton({ selected, panelName, setPanel }: PanelButtonProps) {
    return <div className="flex-col my-auto">
        <form autoComplete="off" onSubmit={e => {
            e.preventDefault();
            setPanel();
        }}>
            <button type="submit" className="cursor-default py-2 pl-4 text-lg">
                <span className={selected ? '' : 'dark:text-neutral-400 text-neutral-500 hover:dark:text-neutral-50 hover:text-neutral-800'}>
                    {panelName}
                </span>
            </button>
        </form>
    </div>;
}

type BottomBarProps = {
    panelIndex: number;
    setPanelIndex: (panelIndex: number) => void;
};

function BottomBar({ panelIndex, setPanelIndex }: BottomBarProps) {
    const [, setUploaderId] = useAtom(uploaderIdAtom);
    const [, setActiveHotkey] = useAtom(hotkeyAtom);
    return <div className="bg-neutral-100 dark:bg-neutral-800 h-full relative">
        <div className="flex h-full">
            {
                panels.map((panel, i) => <PanelButton
                    key={i} selected={i === panelIndex} panelName={panel.name}
                    setPanel={() => {
                        window.location.hash = i.toString();
                        setPanelIndex(i);

                        // Make sure the atoms are cleared when switching panels.
                        setUploaderId(null);
                        setActiveHotkey(null);
                    }}
                />)
            }
        </div>
    </div>;
}

export default function App() {
    const [panelIndex, setPanelIndex] = useState(() => {
        // To make hot reloading work better, store the index in the query hash.
        // We split by underscore to allow for a second parameter to be used by panels.
        let index = parseInt(window.location.hash.slice(1).split("_")[0]);
        if (isNaN(index)) index = 0;
        return index;
    });

    return <CrashHandler>
        <div className="h-screen w-screen relative">
            <div className="w-full overflow-y-scroll" style={{ maxHeight: "calc(100% - 3.5rem)" }}>
                <PanelLoader panelIndex={panelIndex} />
            </div>
            <div className="absolute bottom-0 w-full h-14">
                <BottomBar panelIndex={panelIndex} setPanelIndex={setPanelIndex} />
            </div>
        </div>
    </CrashHandler>;
}
