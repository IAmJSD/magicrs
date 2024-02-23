import { useEffect, useState } from "react";

type Props = {
    description: string;
};

const markedImport = import("marked").then(m => m.parse);

export default function Description({ description }: Props) {
    const [html, setHtml] = useState(`<pre>${description}</pre>`);

    useEffect(() => {
        markedImport.then(async marked => {
            setHtml(await marked(description));
        });
    }, [description]);

    return <div
        className="space-y-2 my-2"
        dangerouslySetInnerHTML={{ __html: html }}
    />;
}
