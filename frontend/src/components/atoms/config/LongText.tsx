import { useId, useState, useEffect, useCallback } from "react";
import {
    deleteConfigOption, deleteUploaderConfigOption, getConfigOption,
    selectFile, setConfigOption, setUploaderConfigOption,
} from "../../../bridge/api";
import Description from "../Description";
import Button from "../Button";
import Modal from "../Modal";
import CodeEditor from "../CodeEditor";

type Props = {
    uploader?: {
        id: string;
        items: {[key: string]: any};
    };
    language?: string;
    dbKey: string;
    label: string;
    description?: string;
};

export default function LongText({ uploader, language, dbKey, label, description }: Props) {
    const labelId = useId();
    const [content, setContent] = useState("");
    const [editorOpen, setEditorOpen] = useState(false);

    useEffect(() => {
        if (uploader) {
            const val = uploader.items[dbKey];
            if (typeof val === "string") setContent(val);
            return;
        }

        let cancelled = false;
        getConfigOption(dbKey).then(val => {
            if (cancelled) return;
            if (typeof val === "string") setContent(val);
        });
        return () => { cancelled = true; };
    }, [uploader, dbKey]);

    // Handles updating the content.
    const updateContent = useCallback((content: string) => {
        // Immediately update the local state.
        setContent(content);

        // If we have an uploader, update the items in the config object.
        if (uploader) uploader.items[dbKey] = content;

        // Update the config option.
        if (content === "") {
            // Delete the option if the content is empty.
            uploader ? deleteUploaderConfigOption(uploader.id, dbKey) : deleteConfigOption(dbKey);
        }
        uploader ? setUploaderConfigOption(uploader.id, dbKey, content) : setConfigOption(dbKey, content);
    }, [uploader, dbKey]);

    return <div className="block m-2">
        <Modal title="Configuration Editor" open={editorOpen} onClose={() => setEditorOpen(false)}>
            <CodeEditor
                language={language || "plaintext"}
                height="50vh"
                width="70vh"
                value={content}
                onChange={updateContent}
            />
        </Modal>

        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            {description && <Description description={description} />}
        </div>

        <div aria-labelledby={labelId} className="flex mt-4 items-center align-middle">
            <div className="flex-col">
                {content === "" ? "Content is empty" : "Content is set"}
            </div>

            <div className="flex-col ml-3">
                <Button
                    onClick={() => selectFile().then(x => {
                        if (x) updateContent(x);
                    })}
                >
                    Select File
                </Button>
            </div>

            <div className="flex-col ml-2">
                <Button
                    onClick={() => setEditorOpen(true)}
                >
                    Open in Editor
                </Button>
            </div>
        </div>
    </div>;
}
