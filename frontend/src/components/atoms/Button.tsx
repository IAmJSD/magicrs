import type { PropsWithChildren } from "react";

const styles = {
    default: "dark:bg-zinc-800 bg-slate-50 hover:dark:bg-zinc-700 hover:bg-slate-100",
    primary: "dark:bg-blue-800 bg-blue-100 hover:dark:bg-blue-700 hover:bg-blue-50",
    secondary: "dark:bg-green-800 bg-green-100 hover:dark:bg-green-700 hover:bg-green-50",
    danger: "dark:bg-red-800 bg-red-50 hover:dark:bg-red-700 hover:bg-red-100",
};

type Props = PropsWithChildren<{
    color?: keyof typeof styles;
    onClick: () => void;
}>;

export default function Button({ children, color, onClick }: Props) {
    color = color || "default";

    return <form onSubmit={e => {
        e.preventDefault();
        onClick();
    }}>
        <button
            className={`block p-2 rounded-lg cursor-default ${styles[color]}`}
            type="submit"
        >
            {children}
        </button>
    </form>;
}
