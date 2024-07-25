import z from "zod";
import { useCallback, useEffect, useId, useState } from "react";
import { BuilderProps } from "../shared";
import Description from "../../../atoms/Description";
import Alert from "../../../atoms/Alert";
import Button from "../../../atoms/Button";

type MetadataTextProps = {
    validator: z.ZodString;
    setOk: (ok: boolean) => void;
    configKey: string;
    config: any;
    name: string;
    description: string;
};

function MetadataText({ validator, setOk, configKey, config, name, description }: MetadataTextProps) {
    const [val, setVal] = useState(config[configKey] || "");
    const [error, setError] = useState<string | null>(null);

    const onChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const newVal = e.target.value;
        setVal(newVal);
        try {
            validator.parse(newVal);
            setError(null);
            config[configKey] = newVal;
            setOk(true);
        } catch (e) {
            setError(e.errors[0].message);
            setOk(false);
        }
    }, [validator, setOk]);

    const labelId = useId();
    return <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {name}
            </p>

            {
                description && <Description description={description} />
            }

            {
                error && <div style={{maxWidth: "60%"}}>
                    <Alert type="error" message={error} />
                </div>
            }
        </div>

        <div>
            <input
                type="text" placeholder={name}
                className="w-full mt-1 mb-2 dark:bg-slate-900 bg-slate-50 p-1 rounded-lg"
                value={val} aria-labelledby={labelId}
                onChange={onChange}
            />
        </div>
    </form>;
}

type MetadataIconProps = {
    setOk: (ok: boolean) => void;
    configKey: string;
    config: any;
};

// Take a image file and turn it into a encoded URI.
function MetadataIcon({ setOk, configKey, config }: MetadataIconProps) {
    const [val, setVal] = useState(config[configKey] || "");
    const [error, setError] = useState<string | null>(null);

    const onChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) {
            setOk(false);
            setError("No file selected");
            return;
        }

        const reader = new FileReader();
        reader.addEventListener("load", () => {
            // Check it is a valid image.
            const result = reader.result as string;
            if (!result.startsWith("data:image/")) {
                setOk(false);
                setError("File is not an image");
                return;
            }

            // Set the value and update the config.
            setVal(result);
            setError(null);
            config[configKey] = result;
            setOk(true);
        });
        reader.readAsDataURL(file);
    }, [setOk, config, configKey]);

    const labelId = useId();
    return <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                Icon
            </p>

            <Description description="The icon of the uploader. This is used to represent the uploader in the UI." />

            {
                error && <div style={{maxWidth: "60%"}}>
                    <Alert type="error" message={error} />
                </div>
            }
        </div>

        <div>
            <input
                type="file"
                className="w-full mt-1 mb-2 dark:bg-slate-900 bg-slate-50 p-1 rounded-lg"
                aria-labelledby={labelId}
                onChange={onChange}
            />
        </div>

        {
            val && <img src={val} alt="Icon" className="w-12 h-12 rounded-lg" />
        }
    </form>;
}

function bubbleMany(...args: boolean[]) {
    const [val, setVal] = useState(args.every((x) => x));
    useEffect(() => {
        setVal(args.every((x) => x));
    }, args);
    return val;
}

export default function BasicMetadata({ setNextStep, config }: BuilderProps) {
    // Defines each metadata field that needs to be okay.
    const [idOk, setIdOk] = useState(false);
    const [nameOk, setNameOk] = useState(false);
    const [descriptionOk, setDescriptionOk] = useState(false);
    const [iconOk, setIconOk] = useState(false);

    // Defines a button state that changes depending on if everything else is okay.
    const buttonEnabled = bubbleMany(idOk, nameOk, descriptionOk, iconOk);

    // Defines a function that goes to the next step.
    const nextPage = () => setNextStep(0);

    return <>
        <MetadataText
            validator={z.string().min(1).max(64).regex(/^[a-z0-9_]+$/, "ID must be lowercase, alphanumeric, and underscores only")}
            setOk={setIdOk}
            configKey="id"
            config={config}
            name="ID"
            description="The ID of the uploader. This is used to identify the uploader in the database."
        />

        <MetadataText
            validator={z.string().min(2).max(64).regex(/^[a-zA-Z0-9_][a-zA-Z0-9_ ]+$/, "Name must be alphanumeric, spaces, and underscores only")}
            setOk={setNameOk}
            configKey="name"
            config={config}
            name="Name"
            description="The name of the uploader. This is used to identify the uploader in the UI."
        />

        <MetadataText
            validator={z.string().min(1).max(256)}
            setOk={setDescriptionOk}
            configKey="description"
            config={config}
            name="Description"
            description="A description of the uploader. This is used to describe the uploader in the UI."
        />

        <MetadataIcon
            setOk={setIconOk}
            configKey="encoded_icon"
            config={config}
        />

        <div className="mt-2">
            <Button
                color="primary"
                onClick={nextPage}
                disabled={!buttonEnabled}
            >
                Next Step
            </Button>
        </div>
    </>;
}
