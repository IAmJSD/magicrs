import React from "react";
import Description from "../Description";
import { deleteUploaderConfigOption, setUploaderConfigOption } from "../../../bridge/api";

type Props = {
    label: string;
    uploader: {
        id: string;
        items: {[key: string]: any};
    };
    dbKey: string;
    description: string;
    frameHtml: string;
};

export default function Custom({ label, uploader, dbKey, description, frameHtml }: Props) {
    const labelId = React.useId();
    const ref = React.useRef<HTMLIFrameElement>(null);
    const rootRef = React.useRef<HTMLDivElement>(null);

    React.useEffect(() => {
        if (!ref.current) return;

        // Handle the config API.
        const listener = (event: MessageEvent) => {
            // Typical preamble to make sure this is a message from the iframe.
            if (event.source !== ref.current.contentWindow) return;
            event.preventDefault();
            event.stopPropagation();

            // Handle the message.
            if (typeof event.data !== "object") throw new Error("Invalid message data.");

            // Handle the DOM size changing.
            if (event.data.height) {
                rootRef.current!.style.height = `${event.data.height + 20}px`;
                return;
            }

            if (event.data.get === true) {
                // Get the value from the config.
                ref.current.contentWindow.postMessage({v: uploader.items[dbKey]}, "*");
                return;
            }

            // Handle setting.
            if (event.data.set === undefined) {
                delete uploader.items[dbKey];
                deleteUploaderConfigOption(uploader.id, dbKey);
            } else {
                setUploaderConfigOption(uploader.id, dbKey, event.data.set);
            }
        };
        window.addEventListener("message", listener);

        // Handle the page loading and getting the height.
        const onLoad = () => {
            rootRef.current!.style.height = `${ref.current!.contentWindow!.document.body.scrollHeight + 20}px`;
        }
        ref.current.addEventListener("load", onLoad);

        // Return the destructor.
        return () => {
            window.removeEventListener("message", listener);
            ref.current?.removeEventListener("load", onLoad);
        };
    }, [ref.current]);

    return <div>
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            {description && <Description description={description} />}
        </div>

        <div ref={rootRef}>
            <iframe
                aria-labelledby={labelId}
                ref={ref}
                title=""
                srcDoc={frameHtml}
                frameBorder={0}
                scrolling="no"
                style={{
                    backgroundColor: "transparent",
                    border: "none",
                    width: "100%",
                    height: "100%",
                }}
            />
        </div>
    </div>;
}
