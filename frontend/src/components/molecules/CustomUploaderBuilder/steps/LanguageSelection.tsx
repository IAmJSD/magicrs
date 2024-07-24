import { BuilderProps } from "../shared";

export default function LanguageSelection({ setNextStep, config }: BuilderProps) {
    const nextPage = (php: boolean) => setNextStep(php ? 1 : 0);

    return <p>Hello</p>;
}
