import { Component, PropsWithChildren } from "react";
import Button from "./atoms/Button";

function ErrorMessage({ error }: { error: any }) {
    return <div className="w-full flex justify-center items-center">
        <div className="block text-center m-5">
            <p style={{fontSize: "5em"}}>
                <i className="fas fa-exclamation-triangle"></i>
            </p>
            <h1 className="text-xl justify-center mt-2 font-semibold">
                Well... this is awkward.
            </h1>
            <p className="mt-2">
                The UI has crashed with the following error:
            </p>
            <p className="mt-2 select-all">
                <code>{error.toString()}</code>
            </p>
            <div className="mt-4">
                <div className="w-full flex justify-center items-center">
                    <Button onClick={() => window.location.replace("/index.html")}>
                        Reload UI
                    </Button>
                </div>
            </div>
        </div>
    </div>;
}

// You need a class component to do this without a npm package :(
export default class CrashHandler extends Component<PropsWithChildren<{}>> {
    state = { error: null };

    static getDerivedStateFromError(error: any) {
        return { error };
    }

    render() {
        return this.state.error ?
            <ErrorMessage error={this.state.error} /> :
            this.props.children;
    }
}
