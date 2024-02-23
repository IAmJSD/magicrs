type Props = {
    message: string;
    type: "error" | "warning" | "success";
};

const styles = {
    error: "bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100",
    warning: "bg-yellow-100 text-yellow-800 dark:bg-yellow-800 dark:text-yellow-100",
    success: "bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100",
};

export default function Alert({ message, type }: Props) {
    return <div className={`p-3 block my-2 w-max rounded-lg ${styles[type]}`}>
        {message}
    </div>;
}
