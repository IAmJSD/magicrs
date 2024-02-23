import type { PropsWithChildren } from "react";

export default function Container({ children }: PropsWithChildren<{}>) {
    return <div className="m-3 block">
        {children}
    </div>;
}
