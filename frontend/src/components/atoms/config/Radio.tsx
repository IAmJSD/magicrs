import React from "react";
import { getConfigOption, setConfigOption } from "../../../bridge/api";
import Description from "../Description";

export type Props = {
    radioItems: [string, string][];
    defaultValue: string;
    dbKey: string;
    label: string;
    description: string;
};

export default function Radio({ radioItems, defaultValue, dbKey, label, description }: Props) {
    const [value, setValue] = React.useState(defaultValue);

    React.useEffect(() => {
        getConfigOption(dbKey).then(value => {
            setValue(value || defaultValue);
        });
    }, [dbKey]);

    return <form onSubmit={e => e.preventDefault()}>
        <p className="mb-1 font-semibold">
            {label}
        </p>

        {
            description && <Description description={description} />
        }

        {radioItems.map(([optValue, label]) => (
            <div className="block" key={optValue}>
                <label>
                    <input
                        className="mr-1"
                        type="radio"
                        name={dbKey}
                        value={optValue}
                        checked={optValue === value}
                        onChange={() => {
                            setValue(optValue);
                            setConfigOption(dbKey, optValue);
                        }}
                    />
                    {label}
                </label>
            </div>
        ))}
    </form>;
}
