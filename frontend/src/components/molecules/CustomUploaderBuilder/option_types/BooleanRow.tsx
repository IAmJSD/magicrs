import { useCallback, useState } from "react";
import type { RowProps } from "../ConfigEditor";
import { AllOptionsExceptEmbedded } from "../../../../bridge/CustomUploader";

function unwrap(opts: AllOptionsExceptEmbedded | null) {
    if (!opts || opts.option_type !== "boolean") return null;
    return opts;
}

export default function BooleanRow({ item, validate }: RowProps) {
    const boolOpts = unwrap(item[1]);
    const [defaultValue, setDefaultValue] = useState(boolOpts?.default || false);
    const [name, setName] = useState(boolOpts?.name || "");
    const [description, setDescription] = useState(boolOpts?.description || "");

    const update = useCallback(() => {
        if (!name || !description) {
            item[1] = null;
            return validate();
        }
        item[1] = {
            option_type: "boolean",
            default: defaultValue,
            name,
            description,
        };
        validate();
    }, [item, validate, name, description, defaultValue]);

    return <>
        
    </>;
}
