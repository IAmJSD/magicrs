import { BuilderProps } from "../shared";

export default function BasicMetadata({ setNextStep, config }: BuilderProps) {
    const nextPage = () => setNextStep(0);

    return <p>Hello</p>;
}
