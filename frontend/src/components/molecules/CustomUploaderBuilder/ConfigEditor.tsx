import { useCallback, useState, type FC } from "react";
import { AllOptionsExceptEmbedded } from "../../../bridge/CustomUploader";
import { InnerProps, ObjectBuilder, useValueOkHandler, wrapPreSortedArray } from "../../atoms/ObjectBuilder";

type Props = {
    config: any;
    setOk: (ok: boolean) => void;
};

type ColumnVal = AllOptionsExceptEmbedded | {};

function TypeSelect({ value, onColumnsEdit, addOkCallback }: InnerProps<any>) {
    const setOk = useValueOkHandler("option_type" in value, addOkCallback);
    return <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
        <select
            defaultValue={"option_type" in value ? value.option_type : "Select a type..."}
            onChange={e => {
                for (const key in value) {
                    if (key !== "name" && key !== "description") delete value[key];
                }
                value.option_type = e.target.value;
                onColumnsEdit();
                setOk(true);
            }}
            className="dark:text-black w-full"
        >
            <option value="Select a type..." disabled>Select a type...</option>
            <option value="string">String</option>
            <option value="long_string">Long String</option>
            <option value="number">Number</option>
            <option value="boolean">Boolean</option>
            <option value="custom">Custom HTML</option>
        </select>
    </form>;
}

function NameEdit({ value }: InnerProps<any>) {
    const [name, setName] = useState("name" in value ? value.name : "");

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="text"
            placeholder="Name"
            value={name}
            onChange={e => {
                setName(e.target.value);
                value.name = e.target.value;
            }}
            className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
        />
    </form>;
}

function DescriptionEdit({ value }: InnerProps<any>) {
    const [description, setDescription] = useState("description" in value ? value.description : "");

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="text"
            placeholder="Description"
            value={description}
            onChange={e => {
                setDescription(e.target.value);
                value.description = e.target.value;
            }}
            className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
        />
    </form>;
}

function BooleanComponent({ value }: InnerProps<AllOptionsExceptEmbedded>) {
    if (value.option_type !== "boolean") throw new Error("Invalid type");
    if (value.default === undefined) value.default = false;
    const [defaultVal, setDefaultVal] = useState(value.default);

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="checkbox"
            checked={defaultVal}
            onChange={e => {
                setDefaultVal(e.target.checked);
                value.default = e.target.checked;
            }}
        />
    </form>;
}

function RequiredHandler({ value }: InnerProps<any>) {
    if (value.required === undefined) value.required = false;
    const [required, setRequired] = useState(value.required);

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="checkbox"
            checked={required}
            onChange={e => {
                setRequired(e.target.checked);
                value.required = e.target.checked;
            }}
        />
    </form>;
}

function StringDefault({ value }: InnerProps<AllOptionsExceptEmbedded>) {
    if (value.option_type !== "string" && value.option_type !== "long_string") throw new Error("Invalid type");
    if (value.default === undefined) value.default = null;
    const [defaultVal, setDefaultVal] = useState(value.default);

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="text"
            value={defaultVal}
            onChange={e => {
                setDefaultVal(e.target.value);
                value.default = e.target.value;
            }}
            className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
        />
    </form>;
}

function wrapNumberHandler(key: "default" | "min" | "max") {
    return function NumberHandler({ value }: InnerProps<AllOptionsExceptEmbedded>) {
        if (value.option_type !== "number") throw new Error("Invalid type");
        if (value[key] === undefined) value[key] = null;
        const [val, setVal] = useState(value[key]);

        return <form autoComplete="off" className="flex" onSubmit={e => e.preventDefault()}>
            <div className="flex-col mr-1">
                <input
                    type="checkbox"
                    checked={val !== null}
                    onChange={e => {
                        if (e.target.checked) {
                            value[key] = 0;
                            setVal(0);
                        } else {
                            value[key] = null;
                            setVal(null);
                        }
                    }}
                />
            </div>

            <div className="flex-col">
                {
                    val !== null && <input
                        type="number"
                        value={val}
                        onChange={e => {
                            setVal(Number(e.target.value));
                            value[key] = Number(e.target.value);
                        }}
                        className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
                    />
                }
            </div>
        </form>;
    };
}

function FrameHTMLSetter({ value }: InnerProps<AllOptionsExceptEmbedded>) {
    if (value.option_type !== "custom") throw new Error("Invalid type");
    if (value.frame_html === undefined) value.frame_html = "";

    // Use file input to update the frame HTML.
    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="file"
            onChange={e => {
                const file = e.target.files?.[0];
                if (!file) return;
                const reader = new FileReader();
                reader.onload = () => {
                    value.frame_html = reader.result as string;
                };
                reader.readAsText(file);
            }}
        />
    </form>;
}

function componentsBuilder(val: AllOptionsExceptEmbedded | {}) {
    const items: [number, string, FC<InnerProps<ColumnVal>>][] = [
        [-3, "Type", TypeSelect],
        [-2, "Name", NameEdit],
        [-1, "Description", DescriptionEdit],
    ];

    if (!("option_type" in val)) return items;
    switch (val.option_type) {
    case "string":
    case "long_string":
        items.push(
            [0, "Required", RequiredHandler],
            [1, "Default", StringDefault],
        );
        break;
    case "number":
        items.push(
            [0, "Required", RequiredHandler],
            [1, "Default", wrapNumberHandler("default")],
            [2, "Minimum", wrapNumberHandler("min")],
            [3, "Maximum", wrapNumberHandler("max")],
        );
        break;
    case "boolean":
        items.push([1, "Default", BooleanComponent]);
        break;
    case "custom":
        items.push([4, "Frame HTML", FrameHTMLSetter]);
        break;
    }

    return items;
}

export default function ConfigEditor(props: Props) {
    if (!props.config.config) props.config.config = [];

    const setOk = useCallback((ok: boolean) => {
        if (ok) {
            for (const c of props.config.config) {
                if (!("option_type" in c[1])) {
                    props.setOk(false);
                    return;
                }
            }
        }
        props.setOk(ok);
    }, [props.setOk]);

    return <ObjectBuilder<ColumnVal>
        obj={wrapPreSortedArray(props.config.config)}
        setOk={setOk}
        newInstance={() => ({
            name: "",
            description: "",
        })}
        componentsBuilder={componentsBuilder}
    />;
}
