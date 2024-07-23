import { useCallback, useState } from "react";
import { insertCustomUploader } from "../../bridge/api";
import Modal from "../atoms/Modal";
import Button from "../atoms/Button";
import CustomUploaderBuilder from "./CustomUploaderBuilder";

function ReplaceUploaderModal(
    { uploader, onReplace }: { uploader: any; onReplace: () => void },
) {
    return <>
        <p>
            A uploader with the ID <code>{uploader.id}</code> already exists. Would you like to replace it?
        </p>
        <div className="flex mt-4">
            <Button
                color="default"
                onClick={onReplace}
            >
                Replace Uploader
            </Button>
        </div>
    </>;
}

export default function CustomUploaderButtons({ revise }: { revise: () => void }) {
    const [modalOpen, setModalOpen] = useState(false);
    const [title, setTitle] = useState("");
    const [body, setBody] = useState<JSX.Element | null>(null);

    const openModal = useCallback((title: string, body: JSX.Element) => {
        setTitle(title);
        setBody(body);
        setModalOpen(true);
    }, []);

    const closeModal = useCallback(() => {
        setBody(null);
        setModalOpen(false);
    }, []);

    const createUploader = useCallback(() => {
        // Create the file open dialog for the JSON.
        const input = document.createElement("input");
        input.type = "file";
        input.accept = ".json";

        // When the user selects a file, read it as text.
        input.addEventListener("change", () => {
            const file = input.files?.[0];
            if (!file) return;

            const reader = new FileReader();
            reader.addEventListener("load", async () => {
                // Get the JSON result.
                const resultStr = reader.result as string;

                // Parse the JSON.
                let result: any;
                try {
                    result = JSON.parse(resultStr);
                } catch (e) {
                    openModal("Custom Uploader Parse Error", <p>
                        The file you uploaded is not valid JSON. Please ensure that the file is in JSON format.
                    </p>);
                    return;
                }

                // Insert the uploader without replacement.
                let insertion: boolean;
                try {
                    insertion = await insertCustomUploader(result, false);
                } catch (e) {
                    openModal("Custom Uploader Insertion Error", <p>
                        An error occurred while inserting the custom uploader: {e.message}
                    </p>);
                    return;
                }
                if (insertion) {
                    revise();
                    return;
                }

                // Ask for replacement.
                openModal("Custom Uploader Replacement", <ReplaceUploaderModal
                    uploader={result}
                    onReplace={async () => {
                        await insertCustomUploader(result, true);
                        revise();
                        closeModal();
                    }}
                />);
            });
            reader.readAsText(file);
        });
        input.click();
    }, []);

    return <>
        <Modal open={modalOpen} onClose={closeModal} title={title}>
            {body}
        </Modal>

        <div className="flex mt-4">
            <div className="flex-col mr-2">
                <Button
                    color="default"
                    onClick={createUploader}
                >
                    <span className="text-sm">
                        Add Custom Uploader
                    </span>
                </Button>
            </div>

            <div className="flex-col">
                <Button
                    color="default"
                    onClick={() => openModal(
                        "Custom Uploader Builder", <CustomUploaderBuilder revise={revise} />)}
                >
                    <span className="text-sm">
                        Custom Uploader Builder
                    </span>
                </Button>
            </div>
        </div>
    </>;
}
