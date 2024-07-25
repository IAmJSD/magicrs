import Button from "../../../atoms/Button";
import { BuilderProps } from "../shared";

function HTTPDescription() {
    return <div className="block text-center w-64">
        <p>
            <img
                src="/icons/http.png"
                alt=""
                className="w-24 mx-auto mb-4"
            />
        </p>
        <p className="font-bold">
            HTTP Extension
        </p>
        <p>
            HTTP extensions is the best method to use when you just want to make one HTTP request to upload a file unless you need to do extra maths or other logic during the upload.
        </p>
    </div>;
}

function PHPDescription() {
    return <div className="block text-center w-64">
        <p>
            <img
                src="/icons/php.png"
                alt=""
                className="w-24 mx-auto mb-4"
            />
        </p>
        <p className="font-bold">
            PHP Extension
        </p>
        <p>
            PHP extensions can do more with uploads than using HTTP or can do very advanced upload logic. However, this will prompt the user to install PHP on their server.
        </p>
    </div>;
}

export default function LanguageSelection({ setNextStep }: BuilderProps) {
    return <>
        <p>
            Please select the language you want to use for your uploader:
        </p>
        <div className="block mt-4">
            <div className="flex mx-auto w-max">
                <div className="flex-col mr-4">
                    <Button
                        color="primary"
                        onClick={() => setNextStep(0)}
                    >
                        <HTTPDescription />
                    </Button>
                </div>

                <div className="flex-col">
                    <Button
                        color="primary"
                        onClick={() => setNextStep(1)}
                    >
                        <PHPDescription />
                    </Button>
                </div>
            </div>
        </div>
    </>;
}
