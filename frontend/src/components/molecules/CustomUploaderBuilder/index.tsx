import React from "react";
import { BuilderProps } from "./shared";
import FlowSuccess from "./FlowSuccess";
import BasicMetadata from "./steps/BasicMetadata";
import LanguageSelection from "./steps/LanguageSelection";
import HTTPSetup from "./steps/HTTPSetup";
import PHPSetup from "./steps/PHPSetup";

type BuilderFlow = [React.FC<BuilderProps>, BuilderFlow[]];

const rootFlow: BuilderFlow = [
    BasicMetadata,
    [
        [
            LanguageSelection,
            [
                [HTTPSetup, []],
                [PHPSetup, []],
            ],
        ],
    ],
];

export default function CustomUploaderBuilder({ revise }: { revise: () => void }) {
    const [flow, setFlow] = React.useState<BuilderFlow | undefined>(rootFlow);
    const [config] = React.useState<any>({
        version: "v1",
    });

    const progressFlow = (index: number) => {
        setFlow((flow) => {
            const newFlow = flow?.[1]?.[index];
            if (!newFlow) return undefined;
            return newFlow;
        });
    };

    const Component = flow?.[0];
    if (!Component) return <FlowSuccess config={config} revise={revise} />;
    return <Component setNextStep={progressFlow} config={config} />;
}
