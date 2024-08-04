import React from "react";
import type { LicenseData, RustLibraryLicenseIterator } from "../../licenses";
import CodeEditor from "../atoms/CodeEditor";

type SearchProps = {
    query: string;
    setQuery: (query: string) => void;
    libsAndVersions: { index: number; name: string; version: string }[];
    itemSelected: (index: number) => void;
    editorId: string;
};

function Search({ query, setQuery, libsAndVersions, itemSelected, editorId }: SearchProps) {
    const [libsAndVersionsIndex, setLibsAndVersionsIndex] = React.useState(0);

    return <div className="mr-4">
        <div className="mx-auto w-full">
            <div className="block">
                <input
                    type="text"
                    value={query}
                    onChange={e => {
                        setQuery(e.target.value);
                        setLibsAndVersionsIndex(0);
                    }}
                    placeholder="Search..."
                    className="justify-center w-full mx-[2px] my-2 dark:bg-zinc-800 bg-slate-50 p-1 rounded-lg"
                    aria-controls={editorId}
                />
            </div>
        </div>
        <div className="flex flex-col">
            {libsAndVersions.map((lib, i) => {
                return <button
                    key={i}
                    className={`p-2 text-left cursor-pointer mx-[4px] ${libsAndVersionsIndex === i ? "bg-gray-200 dark:bg-blue-700" : ""}`}
                    onClick={() => {
                        setLibsAndVersionsIndex(i);
                        itemSelected(lib.index);
                    }}
                    aria-controls={editorId}
                >
                    <div className="font-bold">{lib.name}</div>
                    {lib.version}
                </button>;
            })}
        </div>
    </div>;
}

export type Props = {
    licenses: LicenseData[] | RustLibraryLicenseIterator;
};

export default function LicenseBrowser({ licenses }: Props) {
    const [text, setText] = React.useState(() => {
        const license = "get" in licenses ? licenses.get(0) : licenses[0];
        return license.text;
    });
    const [query, setQuery] = React.useState("");

    const libsAndVersions = React.useMemo(() => {
        let i = 0;
        const a: { index: number; name: string; version: string }[] = [];
        for (const license of licenses) {
            if (license.name.toLowerCase().includes(query.toLowerCase())) {
                a.push({
                    index: i, name: license.name, version: license.version,
                });
            }
            i++;
        }
        return a;
    }, [licenses, query]);

    React.useEffect(() => {
        if (libsAndVersions.length === 0) {
            setText("");
            return;
        }
        const license = "get" in licenses ? licenses.get(libsAndVersions[0].index) : licenses[libsAndVersions[0].index];
        setText(license.text);
    }, [libsAndVersions]);

    const editorId = React.useId();
    return <div className="flex">
        <div className="flex flex-col w-1/3">
            <div className="block overflow-y-scroll h-[70vh]">
                <Search
                    query={query} setQuery={setQuery} libsAndVersions={libsAndVersions}
                    itemSelected={i => {
                        const license = "get" in licenses ? licenses.get(i) : licenses[i];
                        setText(license.text);
                    }} editorId={editorId}
                />
            </div>
        </div>
        <div className="flex flex-col flex-grow">
            <CodeEditor
                value={text}
                height="100%"
                width="100%"
                language="plaintext"
                id={editorId}
            />
        </div>
    </div>;
}
