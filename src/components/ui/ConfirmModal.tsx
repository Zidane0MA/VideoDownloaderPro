import { useEffect, useRef } from 'react';
import { AlertTriangle, X } from 'lucide-react';

interface ConfirmModalProps {
    isOpen: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    onConfirm: () => void;
    onCancel: () => void;
    isDanger?: boolean;
}

export function ConfirmModal({
    isOpen,
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    onConfirm,
    onCancel,
    isDanger = true,
}: ConfirmModalProps) {
    const dialogRef = useRef<HTMLDialogElement>(null);

    useEffect(() => {
        const dialog = dialogRef.current;
        if (!dialog) return;

        if (isOpen) {
            dialog.showModal();
        } else {
            dialog.close();
        }
    }, [isOpen]);

    if (!isOpen) return null;

    return (
        <dialog
            ref={dialogRef}
            className="backdrop:bg-black/50 backdrop:backdrop-blur-sm bg-transparent w-full max-w-md p-0 m-auto rounded-xl shadow-2xl open:animate-in open:fade-in open:zoom-in-95 duration-200"
            onCancel={(e) => {
                e.preventDefault();
                onCancel();
            }}
        >
            <div className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden flex flex-col">
                <div className="p-4 border-b border-surface-700 flex items-center justify-between bg-surface-850">
                    <h2 className="text-lg font-semibold text-surface-100 flex items-center gap-2">
                        {isDanger ? <AlertTriangle className="text-red-500 w-5 h-5" /> : null}
                        {title}
                    </h2>
                    <button
                        onClick={onCancel}
                        className="p-1 text-surface-400 hover:text-surface-200 hover:bg-surface-700 rounded transition-colors"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <div className="p-5">
                    <p className="text-surface-300 text-sm whitespace-pre-wrap leading-relaxed">{message}</p>
                </div>

                <div className="p-4 border-t border-surface-700 bg-surface-900 flex justify-end gap-3">
                    <button
                        onClick={onCancel}
                        className="px-4 py-2 text-sm font-medium text-surface-300 hover:text-surface-100 bg-surface-700 hover:bg-surface-600 rounded-lg transition-colors border border-surface-600"
                    >
                        {cancelText}
                    </button>
                    <button
                        onClick={onConfirm}
                        className={`px-4 py-2 text-sm font-medium text-white rounded-lg transition-colors shadow-md ${isDanger
                            ? 'bg-red-600 hover:bg-red-500 shadow-red-600/20'
                            : 'bg-brand-600 hover:bg-brand-500 shadow-brand-600/20'
                            }`}
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </dialog>
    );
}
