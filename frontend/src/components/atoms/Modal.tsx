import type { PropsWithChildren } from "react";
import ReactDOM from "react-dom";

type Props = PropsWithChildren<{
    title: string;
    open: boolean;
    onClose: () => void;
}>;

function ModalPortal({ children, title, onClose }: Props) {
    return <dialog ref={el => void el?.showModal()} className="fixed inset-0 z-50" onClick={onClose}>
        <div className="fixed inset-0 bg-black bg-opacity-50" />
        <div className="fixed inset-0 flex items-center justify-center">
            <div
                className="bg-white dark:bg-slate-800 dark:text-white p-4 rounded-lg"
                onClick={e => e.stopPropagation()}
            >
                <div className="flex mb-2 items-center align-middle">
                    <div className="flex-col">
                        <form autoComplete="off" onSubmit={e => {
                            e.preventDefault();
                            onClose();
                        }}>
                            <button className="cursor-default" aria-label="Close">
                                <i className="fas fa-times" />
                            </button>
                        </form>
                    </div>
                    <div className="flex-col ml-2">
                        <h1 className="text-md">{title}</h1>
                    </div>
                </div>

                {children}
            </div>
        </div>
    </dialog>;
}

export default function Modal(props: Props) {
    return props.open && ReactDOM.createPortal(
        <ModalPortal {...props} />,
        document.getElementById("modal_portal")!,
    );
}
