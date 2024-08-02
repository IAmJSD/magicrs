import React from "react";
import Button from "../atoms/Button";
import Modal from "../atoms/Modal";
import LicenseBrowser from "./LicenseBrowser";
import { jsLicenses, LicenseData, RustLibraryLicenseIterator, rustLicenses } from "../../licenses";

function CreditModal() {
    const [iterator, setIterator] = React.useState<
        LicenseData[] | RustLibraryLicenseIterator | null
    >(null);

    if (iterator) {
        // Render the license viewer.
        return <LicenseBrowser licenses={iterator} />;
    }

    return <>
        <p className="mb-8">
            MagicCap is built on the shoulders of giants. In building it, we decided
            to use a number of open-source libraries and tools. We would like to
            thank the authors of these libraries for their hard work and dedication.
        </p>

        <div className="flex mx-auto w-max">
            <div className="flex-col mr-4 max-w-64">
                <Button
                    color="primary"
                    onClick={() => rustLicenses.then(setIterator)}
                >
                    <p className="font-bold">
                        View Rust Licenses
                    </p>
                    <p>
                        MagicCap's application and almost everything except the configuration
                        is written in Rust. Rust is licensed under the MIT license.
                    </p>
                </Button>
            </div>
            <div className="flex-col max-w-64">
                <Button
                    color="primary"
                    onClick={() => jsLicenses.then(setIterator)}
                >
                    <p className="font-bold">
                        View JS Licenses
                    </p>
                    <p>
                        MagicCap's configuration is written in TS and JS. We use a number of
                        libraries to make the configuration work, and they are licensed under
                        various licenses.
                    </p>
                </Button>
            </div>
        </div>

        <p className="mt-8">
            We also want to thank Christian Robertson for the Roboto font, which
            is used in this application and is licensed under the Apache-2.0 license.
        </p>
        <p className="mt-2">
            Additionally, we would like to thank the Tailwind CSS team for their
            work on the Tailwind CSS framework, which is used in this application to
            make the beautiful UI you see and is licensed under the MIT license. Icons
            are by Dave Gandy of Font Awesome, which is licensed under the SIL OFL 1.1
            license.
        </p>
    </>;
}

export default function OpenSourceCredits() {
    const [open, setOpen] = React.useState(false);

    return <>
        <Modal title="Open Source Credits" onClose={() => setOpen(false)} open={open}>
            <div className="max-w-4xl">
                <CreditModal />
            </div>
        </Modal >
        <Button
            color="primary"
            onClick={() => setOpen(!open)}
        >
            Open Source Credits
        </Button>
    </>;
}
