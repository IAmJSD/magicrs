type ErrorProps = {
    title: string;
    error: Error;
};

export default function Error({ title, error }: ErrorProps) {
    return <div className="w-full flex justify-center items-center">
        <div className="block text-center m-5">
            <p style={{fontSize: "5em"}}>
                <i className="fas fa-exclamation-triangle"></i>
            </p>
            <h1 className="text-xl justify-center mt-2 font-semibold">{title}</h1>
            <p className="mt-2">
                {error.message}
            </p>
        </div>
    </div>;
}
