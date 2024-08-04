import React from "react";
import Checkbox from "../atoms/config/Checkbox";
import ColorInput from "../atoms/config/ColorInput";
import Radio from "../atoms/config/Radio";
import Container from "../atoms/Container";
import Divider from "../atoms/Divider";
import Header from "../atoms/Header";
import ConfigurationManagement from "../molecules/ConfigurationManagement";
import OpenSourceCredits from "../molecules/OpenSourceCredits";
import { getBuildInfo } from "../../bridge/api";

function AutoupdateOption() {
    const [visible, setVisible] = React.useState(false);

    React.useEffect(() => {
        getBuildInfo("autoupdate_compiled").then(val => {
            setVisible(val);
        });
    }, []);

    if (!visible) return null;
    return <>
        <Radio
            dbKey="autoupdate"
            label="Auto Update"
            description="Defines how MagicCap should handle updates."
            defaultValue="off"
            radioItems={[
                ["off", "Do not check for updates"],
                ["stable", "Check for stable updates"],
                ["beta", "Check for beta and stable updates"],
                ["nightly", "Check for alpha, beta, and stable updates"],
            ]}
        />

        <Divider />
    </>;
}

function injectBuildData(key: string) {
    const [s, setS] = React.useState("");
    React.useEffect(() => {
        getBuildInfo(key).then(val => {
            setS(String(val));
        });
    }, [key]);
    return s;
}

export default function General() {
    const startupLabelId = React.useId();

    return <Container>
        <Header
            title="General"
            subtitle="Configures general settings for MagicCap:"
        />

        <p className="mb-1 font-semibold">
            Build Information
        </p>

        <p className="mt-3">
            Build Hash: <code className="select-all">{injectBuildData("build_hash")}</code>
        </p>

        <p className="my-1">
            Build Branch: <code className="select-all">{injectBuildData("build_branch")}</code>
        </p>

        <p className="mb-3">
            Build Date: <code className="select-all">{injectBuildData("build_date")}</code>
        </p>

        <div className="flex">
            <div className="flex-col mr-2">
                <ConfigurationManagement />
            </div>

            <div className="flex-col">
                <OpenSourceCredits />
            </div>
        </div>

        <Divider />

        <AutoupdateOption />

        <Radio
            dbKey="clipboard_action"
            label="Clipboard Action"
            description="Defines the action to take with the clipboard when a capture is taken."
            defaultValue="content"
            radioItems={[
                ["url", "Copy the URL to the clipboard"],
                ["file_path", "Copy the file path to the clipboard"],
                ["content", "Copy the content to the clipboard"],
                ["none", "Do not copy anything to the clipboard"],
            ]}
        />

        <Divider />

        <div id={startupLabelId}>
            <p className="mb-1 font-semibold">
                Start MagicCap on system startup
            </p>

            <p className="my-3">
                If enabled, MagicCap will start automatically when your system starts.
                This will automatically handle adding to your users startup configuration on your OS.
            </p>
        </div>

        <Checkbox
            dbKey="startup"
            label="Start MagicCap at boot"
            defaultValue={false}
            ariaLabelledBy={startupLabelId}
        />

        <Divider />

        <ColorInput
            dbKey="default_editor_color"
            label="Default Editor Color"
            description="Defines the default color of the editor. This color is used when rendering shapes to the screen."
            defaultValue="#FF0000"
        />
    </Container>;
}
