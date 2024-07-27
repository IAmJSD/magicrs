import type { FC } from "react";
import type { RowProps } from "../ConfigEditor";
import BooleanRow from "./BooleanRow";

const selectOpts: [string, string, FC<RowProps>][] = [
    ["boolean", "Boolean", BooleanRow],
];

export default selectOpts;
