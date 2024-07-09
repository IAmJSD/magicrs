import z from "zod";
import { useAtom } from "jotai";
import { useCallback, useEffect, useState, Fragment } from "react";
import usePromise from "../../../hooks/usePromise";
import Container from "../../atoms/Container";
import Header from "../../atoms/Header";
import { uploaderIdAtom } from "../../../atoms";
import { Uploader, ConfigOption, getUploaderConfigOptions, getUploaders, testUploader, setConfigOption } from "../../../bridge/api";
import Checkbox from "../../atoms/config/Checkbox";
import Divider from "../../atoms/Divider";
import Button from "../../atoms/Button";
import Alert from "../../atoms/Alert";
import Textbox from "../../atoms/config/Textbox";
import LongText from "../../atoms/config/LongText";
import NumberInput from "../../atoms/config/NumberInput";
import Custom from "../../atoms/config/Custom";
import { CustomPanelSelector, handleCustomPanels } from "./customUploaders";
import Embedded from "../../atoms/config/Embedded";

type UploaderProps = {
    uploader: Uploader;
    uploaderId: string;
};

type UploaderOptionsProps = UploaderProps & {
    config: { [key: string]: ConfigOption };
};

function optionSwitch(
    uploaderId: string, key: string, option: ConfigOption,
    config: { [key: string]: ConfigOption },
) {
    switch (option.option_type) {
        case "boolean":
            return <Checkbox
                dbKey={key}
                defaultValue={option.default || false}
                label={option.name}
                uploader={{ id: uploaderId, items: config }}
            />;
        case "string":
            return <Textbox
                dbKey={key}
                label={option.name}
                description={option.description}
                defaultValue={option.default || ""}
                password={option.password}
                uploader={{ id: uploaderId, items: config }}
                validator={
                    option.regex &&
                    z.string().regex(new RegExp(option.regex, "g"))
                }
            />;
        case "number":
            return <NumberInput
                dbKey={key}
                label={option.name}
                description={option.description}
                defaultValue={option.default || 0}
                uploader={{ id: uploaderId, items: config }}
                min={option.min}
                max={option.max}
            />;
        case "long_string":
            return <LongText
                dbKey={key}
                label={option.name}
                description={option.description}
                uploader={{ id: uploaderId, items: config }}
            />;
        case "custom":
            return <Custom
                dbKey={key}
                label={option.name}
                description={option.description}
                frameHtml={option.frame_html}
                uploader={{ id: uploaderId, items: config }}
            />;
        case "embedded":
            return <Embedded
                dbKey={key}
                label={option.name}
                description={option.description}
                componentName={option.component_name as any}
                uploader={{ id: uploaderId, items: config }}
            />;
    }
}

function UploaderOptions({ uploader, uploaderId, config }: UploaderOptionsProps) {
    const orderedOptions = Object.entries(uploader.options).sort((a, b) => a[1].name.localeCompare(b[1].name));

    // Handle the case where the uploader has no options.
    if (orderedOptions.length === 0) return <>
        <p className="mt-2">
            This uploader has no configuration options.
        </p>
        <Divider />
    </>;

    return orderedOptions.map(([key, option]) => {
        return <Fragment key={key}>
            {optionSwitch(uploaderId, key, option, config)}
            <Divider />
        </Fragment>;
    });
}

function Uploader({ uploader, uploaderId }: UploaderProps) {
    const [config, promiseState] = usePromise(
        () => getUploaderConfigOptions(uploaderId), [uploaderId],
    );
    const [alert, setAlert] = useState<{
        type: "error" | "success";
        message: string;
    } | null>(null);

    const testCb = useCallback(() => {
        setAlert(null);
        testUploader(uploaderId).catch(e => {
            setAlert({
                type: "error",
                message: e.message,
            });
        }).then(() => {
            setAlert({
                type: "success",
                message: "The test was successful!",
            });
        });
    }, [uploaderId]);

    const defaultCb = useCallback(() => {
        setAlert(null);
        setConfigOption("uploader_type", uploaderId).catch(e => {
            setAlert({
                type: "error",
                message: e.message,
            });
        }).then(() => {
            setAlert({
                type: "success",
                message: "The uploader has been set as the default!",
            });
        });
    }, [uploaderId]);

    return <Container>
        <Header
            title={uploader.name}
            subtitle={uploader.description}
        />

        {alert && <div className="mb-4">
            <Alert type={alert.type} message={alert.message} />
        </div>}

        {promiseState === "resolved" && <>
            <UploaderOptions
                uploaderId={uploaderId} uploader={uploader} config={config}
            />

            <div className="flex">
                <div className="flex-col mr-2">
                    <Button
                        color="primary"
                        onClick={testCb}
                    >
                        Test Uploader
                    </Button>
                </div>

                <div className="flex-col">
                    <Button
                        color="secondary"
                        onClick={defaultCb}
                    >
                        Set as Default Uploader
                    </Button>
                </div>
            </div>
        </>}
    </Container>;
}

type UploaderListProps = {
    uploaders: { [id: string]: Uploader };
    setUploaderId: (id: string) => void;
};

function UploaderList({ uploaders, setUploaderId }: UploaderListProps) {
    const s = Object.entries(uploaders).sort((a, b) => a[1].name.localeCompare(b[1].name));
    return <div className="flex flex-wrap">
        {
            s.map(([id, uploader]) => <div className="flex-col mr-2" key={id}>
                <Button onClick={() => setUploaderId(id)}>
                    <div className="flex items-center pr-1">
                        <img src={uploader.icon_path} alt="" className="h-7 rounded-2xl mr-2" />
                        {uploader.name}
                    </div>
                </Button>
            </div>)
        }
    </div>;
}

export default function Uploaders() {
    const [uploaderId, setUploaderId] = useAtom(uploaderIdAtom);
    const [uploaders, promiseState] = usePromise(getUploaders, []);

    // Check if the hash explicitly contains an uploader ID.
    useEffect(() => {
        const chunk = window.location.hash.slice(1).match(/^\d_(.+)/);
        if (chunk) setUploaderId(chunk[1]);
    }, []);

    if (uploaderId) {
        // Handle the odd case where the uploaders promise is not done.
        if (promiseState !== "resolved") return <></>;

        // This means a specific uploader is selected. Switch to that view.
        const Panel = uploaderId.startsWith("custom_") && handleCustomPanels(uploaderId);
        if (Panel) return <Panel />;
        const uploader = uploaders[uploaderId];
        if (uploader) return <Uploader uploaderId={uploaderId} uploader={uploader} />;
    }

    return <Container>
        <Header
            title="Uploaders"
            subtitle="Configure how captures are uploaded to services:"
        />

        <Checkbox
            dbKey="upload_capture"
            defaultValue={true}
            label="Upload the capture when it is finished."
        />

        <Divider />

        <h2 className="text-lg font-semibold mb-4">
            Custom Uploaders
        </h2>

        <h3 className="text-sm mb-4">
            Custom uploaders are not officially maintained by MagicCap:
        </h3>

        <CustomPanelSelector />

        <Divider />

        <h2 className="text-lg font-semibold mb-4">
            Supported Uploaders
        </h2>

        <h3 className="text-sm mb-4">
            These uploaders are officially maintained by MagicCap:
        </h3>

        {promiseState === "resolved" && <UploaderList
            uploaders={uploaders} setUploaderId={uploaderId => {
                setUploaderId(uploaderId);

                // Abstract away the hash setting here.
                window.location.hash = `${window.location.hash.slice(1).split("_")[0]}_${uploaderId}`;
            }}
        />}
    </Container>;
}
