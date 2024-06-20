import * as monacoImport from "monaco-editor";
import { initialize as tsWorker } from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";
import { initialize as editorWorker } from "monaco-editor/esm/vs/editor/editor.worker?worker";

self.MonacoEnvironment = {
    getWorker(_, label) {
        if (label === "typescript" || label === "javascript") {
            return new tsWorker();
        }
        return new editorWorker();
    },
};

export const monaco = monacoImport;
