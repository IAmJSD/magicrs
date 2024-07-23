import z from "zod";
import { useAtom } from "jotai";
import { useCallback, useEffect, useState, Fragment } from "react";
import usePromise from "../../hooks/usePromise";
import Container from "../atoms/Container";
import Header from "../atoms/Header";
import { uploaderIdAtom } from "../../atoms";
import {
    Uploader, ConfigOption, getUploaderConfigOptions, getUploaders, testUploader,
    setConfigOption, getCustomUploaders,
} from "../../bridge/api";
import Checkbox from "../atoms/config/Checkbox";
import Divider from "../atoms/Divider";
import Button from "../atoms/Button";
import Alert from "../atoms/Alert";
import Textbox from "../atoms/config/Textbox";
import LongText from "../atoms/config/LongText";
import NumberInput from "../atoms/config/NumberInput";
import Custom from "../atoms/config/Custom";
import Embedded from "../atoms/config/Embedded";
import Description from "../atoms/Description";
import CustomUploaderButtons from "../molecules/CustomUploaderButtons";

type UploaderProps = {
    uploader: Uploader;
    uploaderId: string;
    custom: boolean;
};

type UploaderOptionsProps = UploaderProps & {
    config: { [key: string]: any };
};

function optionSwitch(
    uploaderId: string, key: string, option: ConfigOption,
    config: { [key: string]: any },
) {
    switch (option.option_type) {
        case "boolean":
            return <>
                <p className="mb-1 font-semibold">
                    {option.name}
                </p>

                {
                    option.description && <Description description={option.description} />
                }

                <Checkbox
                    dbKey={key}
                    defaultValue={option.default || false}
                    label={option.name}
                    uploader={{ id: uploaderId, items: config }}
                />
            </>;
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
    // Handle the case where the uploader has no options.
    if (uploader.options.length === 0) return <>
        <p className="mt-2">
            This uploader has no configuration options.
        </p>
        <Divider />
    </>;

    return uploader.options.map(([key, option]) => {
        return <Fragment key={key}>
            {optionSwitch(uploaderId, key, option, config)}
            <Divider />
        </Fragment>;
    });
}

function Uploader({ uploader, uploaderId, custom }: UploaderProps) {
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

    const deleteUploader = useCallback(() => {
        // TODO: Implement this.
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
                custom={custom}
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

                <div className="flex-col mr-2">
                    <Button
                        color="secondary"
                        onClick={defaultCb}
                    >
                        Set as Default Uploader
                    </Button>
                </div>

                {
                    custom && <div className="flex-col">
                        <Button
                            color="danger"
                            onClick={deleteUploader}
                        >
                            Delete Uploader
                        </Button>
                    </div>
                }
            </div>
        </>}
    </Container>;
}

type UploaderListProps = {
    uploaders: { [id: string]: Uploader };
    setUploaderId: (id: string | null) => void;
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
    const [uploaders, uploadersPromiseState] = usePromise(getUploaders, []);
    const [revision, setRevision] = useState(0);
    const [customUploaders, customUploadersPromiseState] = usePromise(getCustomUploaders, [revision]);

    // Check if the hash explicitly contains an uploader ID.
    useEffect(() => {
        const chunk = window.location.hash.slice(1).match(/^\d_(.+)/);
        if (chunk) setUploaderId(chunk[1]);
    }, []);

    if (uploaderId) {
        // Handle the odd case where the uploaders promise is not done.
        if (uploadersPromiseState !== "resolved" || customUploadersPromiseState !== "resolved") return <></>;

        // Try official uploaders first.
        let uploader = uploaders[uploaderId];
        if (uploader) return <Uploader uploaderId={uploaderId} uploader={uploader} custom={false} />;

        // Now try custom uploaders.
        uploader = customUploaders[uploaderId];
        if (uploader) return <Uploader uploaderId={uploaderId} uploader={uploader} custom={true} />;
    }

    // Make sure no custom uploaders that are official are included.
    const remappedCustomUploaders = customUploadersPromiseState === "resolved" ? Object.fromEntries(
        Object.entries(customUploaders).filter(([id]) => !uploaders[id]),
    ) : {};

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

        {
            customUploadersPromiseState === "resolved" && <>
                {Object.keys(customUploaders).length === 0 ? <p className="text-sm">
                    There are no custom uploaders available.
                </p> : <UploaderList
                    uploaders={remappedCustomUploaders} setUploaderId={uploaderId => {
                        setUploaderId(uploaderId);

                        // Abstract away the hash setting here.
                        if (uploaderId) {
                            window.location.hash = `${window.location.hash.slice(1).split("_")[0]}_${uploaderId}`;
                        } else {
                            window.location.hash = window.location.hash.slice(1).split("_")[0];
                        }
                    }}
                />}
                <div className="mt-4">
                    <CustomUploaderButtons revise={() => setRevision((v) => v + 1)} />
                </div>
            </>
        }

        <Divider />

        <h2 className="text-lg font-semibold mb-4">
            Supported Uploaders
        </h2>

        <h3 className="text-sm mb-4">
            These uploaders are officially maintained by MagicCap:
        </h3>

        {uploadersPromiseState === "resolved" && <UploaderList
            uploaders={uploaders} setUploaderId={uploaderId => {
                setUploaderId(uploaderId);

                // Abstract away the hash setting here.
                window.location.hash = `${window.location.hash.slice(1).split("_")[0]}_${uploaderId}`;
            }}
        />}
    </Container>;
}
