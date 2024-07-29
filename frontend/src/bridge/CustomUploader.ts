// Defines the base string options.
type StringOptionBase = {
    name: string;
    description: string;
    default: string | null;
    required: boolean;
};

// Defines all the configuration options other than Embedded.
export type AllOptionsExceptEmbedded = ({
    option_type: "string";
    password: boolean;
    regex: string | null;
    validation_error_message: string | null;
} & StringOptionBase) | ({
    option_type: "long_string";
} & StringOptionBase) | {
    option_type: "number";
    name: string;
    description: string;
    default: number | null;
    min: number | null;
    max: number | null;
    required: boolean;
} | {
    option_type: "boolean";
    name: string;
    description: string;
    default: boolean | null;
} | {
    option_type: "custom";
    name: string;
    description: string;
    frame_html: string;
};

export type FormDataEntryKey = string;

export type RewriteTypeWithValue = {
    type: "Config" | "Static";
    value: string;
};

export type Rewrite = { type: "Filename" | "MIME" } | RewriteTypeWithValue;

export type URLEncodedBody = {
    type: "URLEncoded";
    content: [
        { [key: string]: string },
        {
            name: string;
            encoding_type: "hex" | "b64url" | "b64";
        },
    ];
};

export type MultipartBody = {
    type: "Multipart";
    content: [
        { [key: string]: string },
        FormDataEntryKey,
    ];
};

export type HTTPBody = { type: "Raw" } | URLEncodedBody | MultipartBody;

export type CustomUploaderHandler = {
    type: "php";
    code: string;
} | {
    type: "http";
    rewrites: { [key: string]: Rewrite };
    url_template: string;
    method: "GET" | "POST" | "PUT" | "PATCH";
    header_templates: { [key: string]: string };
    body: HTTPBody;
    response_expr: string;
};

export type CustomUploader = {
    version: "v1";
    id: string;
    name: string;
    description: string;
    encoded_icon: string;
    config: [string, AllOptionsExceptEmbedded][];
    handler: CustomUploaderHandler;
};
