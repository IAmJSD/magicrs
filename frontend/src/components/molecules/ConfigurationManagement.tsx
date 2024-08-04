import React from "react";
import Button from "../atoms/Button";
import Modal from "../atoms/Modal";
import Divider from "../atoms/Divider";

function ConfigModal() {
    return <div className="max-w-xl mt-4">
        Defines tooling for managing the configuration of MagicCap:

        <Divider thin={true} />

        <div className="flex">
            <div className="flex-col mr-4">
                <div className="my-auto w-max mt-1">
                    <Button
                        color="primary"
                        onClick={() => { }}
                    >
                        Save Configuration
                    </Button>
                </div>
            </div>

            <div className="flex-col h-full">
                Saves the current configuration and captures for MagicCap so that it can
                be written to another device.
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
            <ConfigModal />
        </Modal>
    </>;
}
