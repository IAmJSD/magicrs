import React from "react";
import Button from "../atoms/Button";
import Modal from "../atoms/Modal";
import Divider from "../atoms/Divider";
import { wipeSearchIndex, wipeConfig, saveConfig, loadConfig } from "../../bridge/api";

function Confirmation({
    action, close, english,
}: {
    action: () => Promise<void>; close: () => void;
    english: string;
}) {
    const [enabled, setEnabled] = React.useState(false);

    const inputId = React.useId();
    return <div className="max-w-xl mt-4">
        Are you sure you want to {english}?

        <Divider thin={true} />

        <label htmlFor={inputId}>
            Please type <code className="mx-1">I want to {english}</code> below to confirm:
        </label>

        <input
            className="w-full my-4 dark:bg-zinc-800 bg-slate-50 p-2 rounded-lg"
            id={inputId} onChange={(e) => {
                setEnabled(e.target.value === `I want to ${english}`);
            }}
            placeholder={`Type "I want to ${english}" to continue...`}
        />

        <Button
            color="danger"
            disabled={!enabled}
            onClick={async () => {
                await action();
                close();
            }}
        >
            Confirm
        </Button>
    </div>;
}

function ConfigModal({ close }: { close: () => void }) {
    const [el, setEl] = React.useState<JSX.Element | null>(null);
    const confirm = React.useCallback((action: () => Promise<void>, english: string) => {
        setEl(<Confirmation
            action={action}
            close={() => { setEl(null); close(); }}
            english={english}
        />);
    }, [setEl]);

    if (el) return el;

    return <div className="max-w-xl mt-4">
        Defines tooling for managing the configuration of MagicCap:

        <Divider thin={true} />

        <div className="flex">
            <div className="flex-col mr-4">
                <div className="my-auto w-max mt-1">
                    <Button
                        color="primary"
                        onClick={() => saveConfig().then(() => close())}
                    >
                        Save Configuration
                    </Button>
                </div>
            </div>

            <div className="flex-col h-full">
                Saves the current configuration and captures for MagicCap so that it can
                be written to another device. Please ensure you keep this file safe since
                it contains sensitive information.
            </div>
        </div>

        <div className="flex mt-4">
            <div className="flex-col mr-4">
                <div className="my-auto w-max mt-1">
                    <Button
                        color="primary"
                        onClick={() => loadConfig().then(() => window.location.reload())}
                    >
                        Load Configuration
                    </Button>
                </div>
            </div>

            <div className="flex-col h-full">
                Loads a configuration from a file and applies it to MagicCap. This will wipe and update
                your current configuration.
            </div>
        </div>

        <div className="flex mt-4">
            <div className="flex-col mr-4">
                <div className="my-auto w-max mt-1">
                    <Button
                        color="danger"
                        onClick={() => confirm(
                            wipeSearchIndex, "wipe the search index",
                        )}
                    >
                        &nbsp;Wipe Search Index&nbsp;
                    </Button>
                </div>
            </div>

            <div className="flex-col h-full">
                Wipes the search index for MagicCap. This will cause MagicCap to start
                from scratch when indexing files and destroy all OCR/text indexing.
            </div>
        </div>

        <div className="flex mt-4">
            <div className="flex-col mr-4">
                <div className="my-auto w-max mt-1">
                    <Button
                        color="danger"
                        onClick={() => confirm(
                            () => wipeConfig().then(() => window.location.reload()),
                            "wipe the entire configuration",
                        )}
                    >
                        <span className="mx-[1px]">Wipe Configuration</span>
                    </Button>
                </div>
            </div>

            <div className="flex-col h-full">
                Wipes the entire configuration for MagicCap. This will cause MagicCap to
                refresh the configuration and start from scratch.
            </div>
        </div>
    </div>;
}

export default function ConfigurationManagement() {
    const [open, setOpen] = React.useState(false);

    return <>
        <Button
            color="default"
            onClick={() => setOpen(true)}
        >
            Configuration Management
        </Button>

        <Modal
            title="Configuration Management"
            onClose={() => setOpen(false)} open={open}
        >
            <ConfigModal close={() => setOpen(false)} />
        </Modal>
    </>;
}
