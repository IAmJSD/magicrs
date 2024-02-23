import Divider from "./Divider";

type Props = {
    title: string;
    subtitle: string;
};

export default function Header({ title, subtitle }: Props) {
    return <header>
        <h1 className="text-2xl font-bold">
            {title}
        </h1>
        <h2 className="text-md mt-2">
            {subtitle}
        </h2>
        <Divider />
    </header>;
}
