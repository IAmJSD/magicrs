import { type FC, useCallback, useEffect, useRef, useState } from "react";
import Button from "../../../atoms/Button";
import Divider from "../../../atoms/Divider";
import Description from "../../../atoms/Description";
import {
    ObjectBuilder, InnerProps, wrapObject, useValueOkHandler, valueInitiallyOk,
} from "../../../atoms/ObjectBuilder";
import {
    URLEncodedBody, MultipartBody, HTTPBody, Rewrite, RewriteTypeWithValue,
} from "../../../../bridge/CustomUploader";
import { BuilderProps } from "../shared";
import ConfigEditor from "../ConfigEditor";

// @ts-expect-error: The description is a markdown file.
import { plainText as responseExprMd } from "./descriptions/response_expr.md";

function RewriteTypeSelector({ value, onColumnsEdit, addOkCallback }: InnerProps<Rewrite>) {
    const setOk = useValueOkHandler(true, addOkCallback);
    return <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
        <select
            defaultValue={value.type}
            onChange={e => {
                for (const key in value) delete value[key];
                value.type = e.target.value as any;
                onColumnsEdit();
                setOk(true);
            }}
            className="dark:text-black w-full"
        >
            {
                ["Filename", "MIME", "Config", "Static"].map(type => (
                    <option key={type} value={type}>{type}</option>
                ))
            }
        </select>
    </form>;
}

function NonBlankValueColumn({ value, addOkCallback }: InnerProps<RewriteTypeWithValue>) {
    if (value.value === undefined) value.value = "";
    const setOk = useValueOkHandler(value.value !== "", addOkCallback);
    return <NonBlankString handler={value} valueKey="value" setOk={setOk} />;
}

function rewritesComponentsBuilder(rewrite: Rewrite): [number, string, FC<InnerProps<Rewrite>>][] {
    switch (rewrite.type) {
        case "Filename":
        case "MIME":
            return [[-1, "Type", RewriteTypeSelector]] as const;
        case "Config":
            return [[-1, "Type", RewriteTypeSelector], [0, "Option Key", NonBlankValueColumn]] as const;
        case "Static":
            return [[-1, "Type", RewriteTypeSelector], [0, "Value", NonBlankValueColumn]] as const;
    }
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

function stringKvWithObject(obj: { [key: string]: string }, setOk: (ok: boolean) => void) {
    return <ObjectBuilder<string>
        obj={wrapObject(obj)}
        setOk={setOk}
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
    />;
}

function EncodedBodyHandler({ body, setOk }: { body: URLEncodedBody; setOk: (ok: boolean) => void }) {
    if (!body.content) body.content = [{}, { name: "data", encoding_type: "b64" }];

    const kvRef = useRef(true);
    const [name, setName] = useState(body.content[1].name);
    const [encodingType, setEncodingType] = useState(body.content[1].encoding_type);

    return <>
        <h4 className="text-md font-semibold mt-8 mb-4">
            Other Query Parameters
        </h4>

        <h5 className="text-sm mt-2 mb-4">
            This is a list of query parameters that will be sent with the request.
        </h5>

        {stringKvWithObject(body.content[0], ok => {
            kvRef.current = ok;
            setOk(ok ? name.trim().length > 0 : false);
        })}

        <h4 className="text-md font-semibold mt-8 mb-4">
            Data Name
        </h4>

        <h5 className="text-sm mt-2 mb-4">
            This is the name of the data that will be sent with the request.
        </h5>

        <form onSubmit={e => e.preventDefault()}>
            <input
                type="text"
                value={name}
                onChange={e => {
                    const name = e.target.value;
                    setName(name);
                    body.content[1].name = name;
                    setOk(kvRef.current && name.trim().length > 0);
                }}
                className="w-full dark:bg-slate-900 bg-slate-50 p-2 rounded-lg"
            />
        </form>

        <h4 className="text-md font-semibold mt-8 mb-4">
            Encoding Type
        </h4>

        <h5 className="text-sm mt-2 mb-4">
            This is the encoding type of the data that will be sent with the request.
        </h5>

        <form onSubmit={e => e.preventDefault()}>
            <select
                value={encodingType}
                onChange={e => {
                    const encodingType = e.target.value;
                    setEncodingType(encodingType as any);
                    body.content[1].encoding_type = encodingType as any;
                }}
                className="dark:text-black w-full"
            >
                <option value="b64">Base64</option>
                <option value="b64url">Base64 URL</option>
                <option value="hex">Hexadecimal</option>
            </select>
        </form>
    </>;
}

function MultipartBodyHandler({ body, setOk }: { body: MultipartBody; setOk: (ok: boolean) => void }) {
    if (!body.content) body.content = [{}, "data"];

    const kvRef = useRef(true);
    const [name, setName] = useState(body.content[1]);

    return <>
        <h4 className="text-md font-semibold mt-8 mb-4">
            Other Form Parameters
        </h4>

        <h5 className="text-sm mt-2 mb-4">
            This is a list of form parameters that will be sent with the request.
        </h5>

        {stringKvWithObject(body.content[0], ok => {
            kvRef.current = ok;
            setOk(ok ? name.trim().length > 0 : false);
        })}

        <h4 className="text-md font-semibold mt-8 mb-4">
            Data Name
        </h4>

        <h5 className="text-sm mt-2 mb-4">
            This is the name of the data that will be sent with the request.
        </h5>

        <form onSubmit={e => e.preventDefault()}>
            <input
                type="text"
                value={name}
                onChange={e => {
                    const name = e.target.value;
                    setName(name);
                    body.content[1] = name;
                    setOk(kvRef.current && name.trim().length > 0);
                }}
                className="w-full dark:bg-slate-900 bg-slate-50 p-2 rounded-lg"
            />
        </form>
    </>;
}

function HTTPBodyHandler({ body, setOk }: { body: HTTPBody; setOk: (ok: boolean) => void }) {
    const [type, setType] = useState(body.type);

    let Component: FC<{ body: any; setOk: (ok: boolean) => void }> | null = null;
    switch (type) {
        case "Raw":
            // Just do nothing.
            break;
        case "URLEncoded":
            Component = EncodedBodyHandler;
            break;
        case "Multipart":
            Component = MultipartBodyHandler;
            break;
    }

    return <>
        <form autoComplete="off" className="block" onSubmit={e => e.preventDefault()}>
            <select
                defaultValue={type}
                onChange={e => {
                    setType(e.target.value as any);
                    for (const key in body) delete body[key];
                    body.type = e.target.value as any;
                    setOk(true);
                }}
                className="dark:text-black w-full"
            >
                {
                    ["Raw", "URL Encoded", "Multipart"].map(type => {
                        const value = type.replace(" ", "");
                        return <option key={type} value={value}>{type}</option>;
                    })
                }
            </select>
        </form>
        {Component && <div className="mt-4">
            <Component body={body} setOk={setOk} />
        </div>}
    </>;
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

            {stringKvWithObject(config.handler.header_templates, setHeaderTemplatesOk)}

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

            <div className="mb-2">
                <NonBlankString
                    handler={config.handler}
                    setOk={setResponseExprOk}
                    valueKey="response_expr"
                />
            </div>
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
