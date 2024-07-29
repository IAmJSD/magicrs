import { useCallback, useState } from "react";
import Button from "../../../atoms/Button";
import { BuilderProps } from "../shared";
import ConfigEditor from "../ConfigEditor";
import Divider from "../../../atoms/Divider";
import { ObjectBuilder, InnerProps, wrapObject, useValueOkHandler, valueInitiallyOk } from "../../../atoms/ObjectBuilder";
import { HTTPBody, Rewrite } from "../../../../bridge/CustomUploader";
import Description from "../../../atoms/Description";

// @ts-expect-error: The description is a markdown file.
import { plainText as responseExprMd } from "./descriptions/response_expr.md";

function rewritesComponentsBuilder(rewrite: Rewrite) {
    // TODO
    return [];
}

type NonBlankStringProps = {
    handler: any;
    valueKey: string;
    setOk: (ok: boolean) => void;
};

function NonBlankString({ handler, valueKey, setOk }: NonBlankStringProps) {
    const [value, setValue] = useState(handler[valueKey]);

    const updateValue = useCallback((value: string) => {
        handler[valueKey] = value;
        setValue(value);
        setOk(value.trim().length > 0);
    }, [handler, valueKey, setOk]);

    return <form onSubmit={e => e.preventDefault()}>
        <input
            type="text"
            value={value}
            onChange={e => updateValue(e.target.value)}
            className="w-full dark:bg-slate-900 bg-slate-50 p-2 rounded-lg"
        />
    </form>;
}

function HTTPBodyHandler({ body, setOk }: { body: HTTPBody; setOk: (ok: boolean) => void }) {
    // TODO
    return null;
}

export default function HTTPSetup({ setNextStep, config }: BuilderProps) {
    if (!config.handler) config.handler = {
        type: "http",
        rewrites: {},
        url_template: "https://upload.example.com",
        method: "POST",
        header_templates: {},
        body: { type: "Raw" },
        response_expr: 'json_path("url")',
    };

    const [configOk, setConfigOk] = useState(true);
    const [rewritesOk, setRewritesOk] = useState(true);
    const [urlTemplateOk, setUrlTemplateOk] = useState(true);
    const [methodInput, setMethodInput] = useState(config.handler.method);
    const [headerTemplatesOk, setHeaderTemplatesOk] = useState(true);
    const [bodyOk, setBodyOk] = useState(true);
    const [responseExprOk, setResponseExprOk] = useState(true);

    return <>
        <p>
            Use this editor to build your uploader with HTTP.
        </p>

        <div className="block max-h-[50vh] overflow-y-scroll min-w-[30vw] max-w-[80vw] my-8">
            <h2 className="text-lg font-semibold mb-4">
                Configuration Options
            </h2>

            <h3 className="text-sm mb-4">
                Defines the configuration options for the HTTP uploader.
            </h3>

            <ConfigEditor
                config={config}
                setOk={setConfigOk}
            />

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                Rewrites
            </h2>

            <h3 className="text-sm mb-4">
                Defines rewrites. This allows you to effectively build a replacement that turns
                the string specified into another string.
            </h3>

            <ObjectBuilder<Rewrite>
                obj={wrapObject(config.handler.rewrites)}
                setOk={setRewritesOk}
                newInstance={() => ({
                    type: "MIME",
                })}
                componentsBuilder={rewritesComponentsBuilder}
            />

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                URL Template
            </h2>

            <h3 className="text-sm mb-4">
                Defines the URL template for the HTTP request.
            </h3>

            <NonBlankString
                handler={config.handler}
                setOk={setUrlTemplateOk}
                valueKey="url_template"
            />

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                Method Type
            </h2>

            <h3 className="text-sm mb-4">
                Defines the method type for the HTTP request.
            </h3>

            <form onSubmit={e => e.preventDefault()}>
                {
                    ["GET", "POST", "PUT", "PATCH"].map(method => (
                        <label key={method} className="flex items-center space-x-2">
                            <input
                                type="radio"
                                name="method"
                                value={method}
                                checked={methodInput === method}
                                onChange={e => {
                                    config.handler.method = method;
                                    setMethodInput(method);
                                }}
                            />
                            <span>{method}</span>
                        </label>
                    ))
                }
            </form>

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                Header Templates
            </h2>

            <h3 className="text-sm mb-4">
                Defines the header templates for the HTTP request.
            </h3>

            <ObjectBuilder<string>
                obj={wrapObject(config.handler.header_templates)}
                setOk={setHeaderTemplatesOk}
                newInstance={() => ""}
                componentsBuilder={() => [
                    [0, "Value", valueInitiallyOk(({ value: initValue, rewriteValue }) => {
                        const [value, setValue] = useState(initValue);
                        return <form onSubmit={e => e.preventDefault()}>
                            <input
                                type="text"
                                value={value}
                                onChange={e => {
                                    const value = e.target.value;
                                    setValue(value);
                                    rewriteValue(value);
                                }}
                                className="w-full dark:bg-slate-900 bg-slate-50 p-1 rounded-lg"
                            />
                        </form>;
                    })],
                ]}
            />

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                HTTP Body
            </h2>

            <h3 className="text-sm mb-4">
                Defines the body for the HTTP request.
            </h3>

            <HTTPBodyHandler
                body={config.handler.body}
                setOk={setBodyOk}
            />

            <Divider />

            <h2 className="text-lg font-semibold mb-4">
                Response Expression
            </h2>

            <div className="text-sm mb-4">
                <Description description={responseExprMd} />
            </div>

            <NonBlankString
                handler={config.handler}
                setOk={setResponseExprOk}
                valueKey="response_expr"
            />
        </div>

        <Button
            color="primary"
            onClick={() => setNextStep(0)}
            disabled={
                !configOk || !rewritesOk || !urlTemplateOk ||
                !headerTemplatesOk || !bodyOk ||
                !responseExprOk
            }
        >
            Finish
        </Button>
    </>;
}
