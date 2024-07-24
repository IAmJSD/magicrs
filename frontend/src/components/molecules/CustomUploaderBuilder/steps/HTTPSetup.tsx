import { BuilderProps } from "../shared";

export default function HTTPSetup({ setNextStep, config }: BuilderProps) {
    const finalize = () => setNextStep(0);

    return <p>Hello</p>;
}
