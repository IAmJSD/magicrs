import z from "zod";
import Container from "../atoms/Container";
import Header from "../atoms/Header";
import Checkbox from "../atoms/config/Checkbox";
import Textbox from "../atoms/config/Textbox";
import FolderOpen from "../atoms/config/FolderOpen";
import Divider from "../atoms/Divider";

// @ts-expect-error: The description is a markdown file.
import { plainText as filenameFormatDescription } from "./descriptions/filename_format.md";

const filesystemRegex = /[<>:"/\\|?*]/g;
const supportedExceptions = [
    "{date}", "{time}", "{random:emoji}",
    /{random:emoji:\d+}/, /{random:\d+:\d+}/g,
    "{random:alphabet}", /{random:alphabet:\d+}/g,
    /{random:alphabet:([a-zA-Z])-([a-zA-Z])(:\d+)?}/g,
];

function supportedFilename(path: string) {
    // Remove the supported exceptions.
    for (const exception of supportedExceptions) {
        path = path.replace(exception, "");
    }

    // Needs to be filesystem safe.
    return !path.match(filesystemRegex);
}

export default function FileSaving() {
    return <Container>
        <Header
            title="File Saving"
            subtitle="Configure how captures are saved to disk:"
        />

        <Checkbox
            dbKey="save_capture"
            defaultValue={true}
            label="Save the capture when it is finished."
        />

        <Divider />

        <FolderOpen
            dbKey="folder_path"
            label="Folder Path"
            description="The folder where captures are saved. By default, they are stored in `MagicCap` in your users pictures folder."
        />

        <Divider />

        <Textbox
            dbKey="filename_format"
            label="Filename Format"
            description={filenameFormatDescription}
            defaultValue="screenshot_{date}_{time}"
            validator={
                z.string().
                    min(1, "The filename cannot be empty.").
                    refine(
                        supportedFilename,
                        "The filename needs to be a valid filesystem name.",
                    )
            }
        />
    </Container>;
}
