import { ClipLoader } from "react-spinners";
import useDarkMode from "../../hooks/useDarkMode";
import usePromise from "../../hooks/usePromise";
import { loader, Editor as editorComponent } from "@monaco-editor/react";

type Props = {
    language: string;
    height: string;
    width: string;
    value: string;
    onChange?: (value: string) => void;
};

export default function CodeEditor({ language, value, onChange, height, width }: Props) {
    const darkMode = useDarkMode();

    // Monaco is HUGE. Only load it when we need it.
    // @ts-expect-error: It is so huge we build it seperately.
    const [Editor, promiseState] = usePromise(() => import("/monaco/monacoSetup.mjs?url").then(
        async ({ monaco }) => {
            loader.config({ monaco });
            return loader.init().then(() => editorComponent);
        },
    ), []);

    if (promiseState !== "resolved") return <ClipLoader color={darkMode ? "white" : "black"} size={100} />;
    return <Editor
        height={height}
        width={width}
        language={language}
        value={value}
        onChange={onChange}
        options={{ readOnly: !onChange }}
        theme={darkMode ? "vs-dark" : "light"}
    />;
}
