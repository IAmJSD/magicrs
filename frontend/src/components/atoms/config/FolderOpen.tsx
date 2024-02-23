import { useId, useState, useEffect } from "react";
import Description from "../Description";
import { getConfigOption, selectFolder, setConfigOption } from "../../../bridge/api";
import Button from "../Button";

type Props = {
    dbKey: string;
    label: string;
    description: string;
};

export default function FolderOpen({ dbKey, label, description }: Props) {
    const labelId = useId();
    const [folder, setFolder] = useState<string | null>(null);

    useEffect(() => {
        getConfigOption(dbKey).then(val => {
            if (typeof val === "string") setFolder(val);
        });
    }, [dbKey]);

    return <div className="block m-2">
        <div id={labelId}>
            <p className="mb-1 font-semibold">
                {label}
            </p>

            <Description description={description} />
        </div>

        <div aria-labelledby={labelId} className="flex mt-4">
            <div className="flex-col my-auto">
                {folder || "--"}
            </div>
            <div className="flex-col ml-2">
                <Button
                    onClick={() => {
                        selectFolder().then(folder => {
                            if (!folder) return;
                            setFolder(folder);
                            setConfigOption(dbKey, folder).catch(e => {
                                setFolder(null);
                                throw e;
                            });
                        });
                    }}
                >
                    Select Folder
                </Button>
            </div>
        </div>
    </div>;
}
