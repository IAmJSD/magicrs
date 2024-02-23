import useDarkMode from "../../hooks/useDarkMode";
import usePromise from "../../hooks/usePromise";
import { loader, Editor as editorComponent } from "@monaco-editor/react";

type Props = {
    language: string;
    height: string;
    width: string;
    value: string;
    onChange: (value: string) => void;
};

export default function CodeEditor({ language, value, onChange, height, width }: Props) {
    const darkMode = useDarkMode();

    // Monaco is HUGE. Only load it when we need it.
    const [Editor, promiseState] = usePromise(() => import("./utils/async/monacoSetup").then(
        () => loader.init().then(() => editorComponent),
    ), []);

    if (promiseState !== "resolved") return <></>;
    return <Editor
        height={height}
        width={width}
        language={language}
        value={value}
        onChange={onChange}
        theme={darkMode ? "vs-dark" : "light"}
    />;
}
