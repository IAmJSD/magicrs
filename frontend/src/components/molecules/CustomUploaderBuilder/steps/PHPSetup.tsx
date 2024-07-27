import { useState } from "react";
import CodeEditor from "../../../atoms/CodeEditor";
import { BuilderProps } from "../shared";
import ConfigEditor from "../ConfigEditor";
import Button from "../../../atoms/Button";

const DEFAULT_PHP_CODE = `<?php
// This is a PHP code snippet.
`;

function PHPCode({ config }: { config: any }) {
    // Ensure the handler is set.
    if (!config.handler) config.handler = {
        type: "php",
        code: DEFAULT_PHP_CODE,
    };

    // Mount Monaco.
    return <CodeEditor
        height="70vh"
        width="40vw"
        language="php"
        onChange={(code) => config.handler.code = code}
        value={config.handler.code}
    />;
}

export default function PHPSetup({ setNextStep, config }: BuilderProps) {
    const [ok, setOk] = useState(true);
    const finalize = () => setNextStep(0);

    return <>    
        <p>
            Use this editor to build your uploader with PHP. The left is the code editor for your PHP logic, and the right
            is the configuration editor for your uploader.
        </p>

        <div className="flex my-4">
            <div className="flex-col mr-1">
                <PHPCode config={config} />
            </div>
            <div className="flex-col ml-1">
                <div className="block max-h-[70vh] overflow-y-scroll w-[50vw]">
                    <ConfigEditor config={config} setOk={setOk} />
                </div>
            </div>
        </div>

        <Button
            color="primary"
            onClick={finalize}
            disabled={!ok}
        >
            Finish
        </Button>
    </>;
}
