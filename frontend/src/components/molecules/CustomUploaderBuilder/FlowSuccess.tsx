import { insertCustomUploader, saveDialog } from "../../../bridge/api";
import usePromise from "../../../hooks/usePromise";
import Button from "../../atoms/Button";

type FlowSuccessProps = {
    config: any;
    revise: () => void;
};

export default function FlowSuccess({ config, revise }: FlowSuccessProps) {
    const [res, promiseState] = usePromise(async () => {
        await insertCustomUploader(config, true);
        revise();
    }, [config, revise]);

    if (promiseState === "loading") return <></>;

    if (promiseState === "rejected") {
        return <>
            <p className="mb-4">
                Failed to insert custom uploader: <code>{res.error.message}</code>.
                This is a bug, please report it!
            </p>

            <Button
                color="primary"
                onClick={() => {
                    const j = JSON.stringify(config, null, 4);
                    navigator.clipboard.writeText(j);
                }}
            >
                Copy Uploader To Clipboard
            </Button>
        </>;
    }

    return <>
        <p className="mb-4">
            Custom uploader successfully inserted!
        </p>

        <Button
            color="primary"
            onClick={() => {
                const j = JSON.stringify(config, null, 4);
                saveDialog(j, "uploader.json");
            }}
        >
            Save Uploader
        </Button>
    </>;
}
