import { useCallback, useState, type FC } from "react";
import { AllOptionsExceptEmbedded } from "../../../bridge/CustomUploader";
import Button from "../../atoms/Button";
import selectOpts from "./option_types";

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

export type RowProps = {
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
    const [Component, setLoadedComponent] = useState<FC<RowProps> | null>(null);

    const onSelectUpdate = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
        const value = e.target.value;
        const component = selectOpts.find(([key]) => key === value);
        if (component) {
            setLoadedComponent(component[2]);
            item[1] = null;
            validate();
        }
    }, [item, validate]);

    return <>
        <td className="p-1 w-[12vw]">
            <select
                onChange={onSelectUpdate}
                className="w-full rounded-lg dark:text-black"
                defaultValue={item[1] ? item[1].option_type : "Select a type..."}
            >
                <option value="Select a type..." disabled>Select a type...</option>
                {selectOpts.map(([key, val]) => <option key={key} value={key}>{val}</option>)}
            </select>
        </td>
        {Component ? <Component item={item} validate={validate} /> : <>
            <td></td>
            <td></td>
        </>}
    </>;
}

function ConfigRow({ item, validate, drop }: RowProps & { drop: () => void }) {
    return <tr>
        <td className="p-1 w-[12vw]">
            <ConfigID item={item} validate={validate} />
        </td>
        <ConfigMetadata item={item} validate={validate} />
        <td className="p-1">
            <Button
                color="danger"
                onClick={drop}
            >
                <span aria-label="Remove">
                    <i className="fas fa-trash" />
                </span>
            </Button>
        </td>
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
        props.setOk(true);
    }, [options, props.setOk]);

    const rows = options.map((a, i) => {
        const reactKey = `config-${i}-${JSON.stringify(a)}`;
        return <ConfigRow
            key={reactKey} item={a} validate={validate}
            drop={() => {
                setOptions(options.filter((_, j) => j !== i));
                validate();
            }}
        />;
    });

    return <>
        <table className="w-full mb-4">
            <thead>
                <tr>
                    <th className="p-1">ID</th>
                    <th className="p-1">Type</th>
                    <th className="p-1">Required</th>
                    <th className="p-1">Value</th>
                    <th></th>
                </tr>
            </thead>

            <tbody>
                {rows}
            </tbody>
        </table>

        <div className="ml-1">
            <Button
                color="primary"
                onClick={() => {
                    setOptions([...options, ["", null]]);
                    props.setOk(false);
                }}
            >
                Add Option
            </Button>
        </div>
    </>;
}
