Defines the expression used to calculate the response string. Under the hood, MagicCap uses `evalexpr` to evaluate the expression. You can use all of its built in functions along with the following:

- `get_rewrite(key)`: Get the value of a rewrite by the key.
- `get_header(key)`: Get the value of a header by the key.
- `json_path(path)`: Get the value of a JSON path from the response body. If path is unset, gets from the root.
- `xml_path(path)`: Get the value of an XML path from the response body. If path is unset, gets from the root.
- `base64_encode(value)`: Encode a value in base64.
- `base64_decode(value)`: Decode a value from base64.
- `string_body()` : Get the response body as a string.
- `url_encode(value)`: Encode a value in URL encoding.
