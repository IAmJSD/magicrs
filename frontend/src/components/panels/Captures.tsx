import { useEffect, useRef, useState } from "react";
import tippy from "tippy.js";
import { ClipLoader } from "react-spinners";
import {
    getCapturesHtml, deleteCapture, openUrl, openFile, showInFolder, copyUrl,
} from "../../bridge/api";
import { addCaptureWatcher, removeCaptureWatcher } from "../../bridge/newCaptures";
import { fileSystemProxy } from "../../bridge/proxy";
import ErrorPage from "../atoms/ErrorPage";
import useDarkMode from "../../hooks/useDarkMode";
import usePromise from "../../hooks/usePromise";

// Styling for tippy.js.
import "tippy.js/dist/tippy.css";

// The screen displayed when captures are loading.
function CapturesLoading() {
    const darkMode = useDarkMode();
    return <div className="w-full flex justify-center items-center">
        <div className="block text-center m-5">
            <ClipLoader color={darkMode ? "white" : "black"} size={100} />
            <h1 className="text-xl justify-center mt-2 font-semibold">Loading Captures...</h1>
            <p className="mt-2">
                This may take a few seconds if you have a lot of captures.
            </p>
        </div>
    </div>;
}

// Defines supported actions for captures.
const supportedCaptureActions: {[key: string]: (id: string) => Promise<void>} = {
    deleteCapture, openUrl, openFile, showInFolder, copyUrl,
};

// Handles when a capture form is submitted.
function captureFormSubmit(event: Event) {
    event.preventDefault();

    // Handle the forms action.
    const form = event.target as HTMLFormElement;
    const action = form.dataset.action;
    if (!action) throw new Error("Form has no action.");

    // Get the capture ID from the form contents.
    const captureId = (form.querySelector("[name=capture_id]") as HTMLInputElement).value;
    const actionFn = supportedCaptureActions[action] || (
        () => { throw new Error("unsupported action") }
    );

    // Call the action function.
    actionFn(captureId).catch(error => {
        console.error("Capture action failed:", error);
    });

    // If the action is deleteCapture, remove the capture root item.
    if (action === "deleteCapture") {
        let x: HTMLElement = form;
        while (x) {
            if (x.dataset.captureRoot) {
                x.remove();
                break;
            }
            x = x.parentElement;
        }
    }

    // Prevent standard form submission in some WebKit browsers.
    return false;
}

// Hooks all the forms/images that are children of the given element.
let tick = 0;
function hookCaptureChildren(element: HTMLElement) {
    element.querySelectorAll("[data-filesystem-path]").forEach((img: HTMLImageElement) => {
        const magicUrl = "magiccap-internal://frontend-dist/placeholder.png";
        if (!img.src.startsWith(magicUrl)) return;
        img.src = `${magicUrl}#${tick++}`;
        img.addEventListener("load", () => {
            const fp = img.dataset.filesystemPath!;
            if (!fp) return;
            fileSystemProxy(fp).then(res => {
                if (res.ok) img.src = res.data;
            });
        }, { once: true });
    });

    element.querySelectorAll("[data-action]").forEach(form => {
        // Find all buttons that contain aria-label and add a tooltip.
        form.querySelectorAll("button[aria-label]").forEach(button => {
            tippy(button, {
                aria: {
                    content: null,
                },
                content: button.getAttribute("aria-label")!,
                animation: false,
            });
        });

        // Hook the form submit event.
        form.addEventListener("submit", captureFormSubmit);
    });
}

// Handles any new elements that are added to the given element.
function handleNewElements(element: HTMLElement) {
    function listener(html: string) {
        let div = document.createElement("div");
        div.innerHTML = html;
        div = div.firstChild as HTMLDivElement;

        // Hook any events within this node.
        hookCaptureChildren(div.firstChild as HTMLElement);

        element.prepend(div);
    }
    const id = addCaptureWatcher(listener);
    return () => removeCaptureWatcher(id);
}

// Defines the renderer and hooks for the captures HTML component.
function CapturesHTML({ html }: { html: string }) {
    // Defines a ref to the span.
    const ref = useRef<HTMLSpanElement>(null);

    // Handle the capture forms.
    useEffect(() => {
        // Ref will only be set when the HTML is set.
        if (!ref.current) return;

        // Hook all capture form children.
        hookCaptureChildren(ref.current);

        // Handle a event stream of new elements.
        return handleNewElements(ref.current.firstChild! as HTMLElement);
    }, [ref, html]);

    // Render the HTML.
    return <span dangerouslySetInnerHTML={{ __html: html }} ref={ref} />;
}

// This component should render exactly once after the HTML is set. The HTML generated from the
// Rust takes over inside the function.
export default function Captures() {
    // Defines the query string for the captures.
    const [query, setQuery] = useState("");

    // Get the HTML.
    const [htmlOrError, state] = usePromise(() => getCapturesHtml(query), [query]);

    // Defines the captures state.
    const capturesState = (() => {
        // If we are loading, show a loading message.
        if (state === "loading") return <CapturesLoading />;

        // If we have a error, render that component.
        if (state === "rejected") return <ErrorPage title="Failed to load captures" error={htmlOrError.error} />;

        // Render the captures HTML.
        return <CapturesHTML html={htmlOrError} />;
    })();

    // Return the captures and a search bar.
    return <>
        <form onSubmit={e => e.preventDefault()} className="flex justify-center">
            <input
                type="search"
                placeholder="Search captures..."
                className="w-5/6 max-w-xl mt-3 mb-2 dark:bg-zinc-800 bg-slate-50 p-2 rounded-lg"
                onChange={e => setQuery(e.target.value)}
            />
        </form>
        {capturesState}
    </>;
}
