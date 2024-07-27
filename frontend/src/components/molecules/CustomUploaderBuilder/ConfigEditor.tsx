import { useCallback, useState } from "react";
import { AllOptionsExceptEmbedded } from "../../../bridge/CustomUploader";
import Button from "../../atoms/Button";

function useConfigState<T>(config: any, key: string, defaultValue: T): [T, (value: T) => void] {
    if (!config[key]) config[key] = defaultValue;
    const [v, setV] = useState<T>(config[key]);
    return [v, useCallback(valOrFn => {
        const newVal = typeof valOrFn === "function" ? valOrFn(v) : valOrFn;
        config[key] = newVal;
        setV(newVal);
    }, [config, key, v])];
}

type ConfigOptions = [string, AllOptionsExceptEmbedded | null][];

type RowProps = {
    item: ConfigOptions[number];
    validate: () => void;
};

function ConfigID({ item, validate }: RowProps) {
    const [id, setId] = useState(item[0]);

    const update = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        setId(e.target.value);
        item[0] = e.target.value;
        validate();
    }, [item, validate]);

    return <form autoComplete="off" onSubmit={e => e.preventDefault()}>
        <input
            type="text"
            placeholder="ID"
            value={id}
            onChange={update}
            className="w-full rounded-lg p-[5px] dark:bg-slate-900 bg-slate-50"
        />
    </form>;
}

function ConfigMetadata({ item, validate }: RowProps) {
    return <>
        <td className="p-1">
            hello
        </td>
    </>;
}

function ConfigRow({ item, validate }: RowProps) {
    return <tr>
        <td className="p-1 max-w-[12vw]">
            <ConfigID item={item} validate={validate} />
        </td>
        <ConfigMetadata item={item} validate={validate} />
    </tr>;
}

type Props = {
    config: any;
    setOk: (ok: boolean) => void;
};

export default function ConfigEditor(props: Props) {
    const [options, setOptions] = useConfigState<ConfigOptions>(props.config, "config", []);

    const validate = useCallback(() => {
        for (let i = 0; i < options.length; i++) {
            const [key, value] = options[i];

            // Handle if it is unset.
            if (!value || key === "") {
                props.setOk(false);
                return;
            }

            // Handle if it isn't a unique ID.
            for (let j = 0; j < i; j++) {
                if (options[j][0] === key) {
                    props.setOk(false);
                    return;
                }
            }
        }
    }, [options]);

    const rows = options.map((a, i) => {
        const reactKey = `config-${i}-${JSON.stringify(a)}`;
        return <ConfigRow key={reactKey} item={a} validate={validate} />;
    });

    return <>
        <table className="w-full mb-4">
            <thead>
                <tr>
                    <th className="p-1">ID</th>
                    <th className="p-1">Type</th>
                    <th className="p-1">Value</th>
                    <th className="p-1">Default</th>
                    <th className="p-1">Required</th>
                </tr>
            </thead>

            <tbody>
                {rows}
            </tbody>
        </table>

        <Button
            color="primary"
            onClick={() => {
                setOptions([...options, ["", null]]);
                props.setOk(false);
            }}
        >
            Add Option
        </Button>
    </>;
}
