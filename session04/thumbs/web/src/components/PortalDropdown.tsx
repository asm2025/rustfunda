import React, { useLayoutEffect, useRef } from "react";
import { createPortal } from "react-dom";

interface PortalDropdownProps<T extends HTMLElement = HTMLElement> {
    anchorRef: React.RefObject<T | null>;
    open: boolean;
    children: React.ReactNode;
    onClose: () => void;
    minWidth?: number | string;
}

const PortalDropdown: React.FC<PortalDropdownProps> = <T extends HTMLElement = HTMLElement>({ anchorRef, open, children, onClose, minWidth }: PortalDropdownProps<T>) => {
    const dropdownRef = useRef<HTMLDivElement | null>(null);

    // Position the dropdown
    const [style, setStyle] = React.useState<React.CSSProperties>({});

    useLayoutEffect(() => {
        if (open && anchorRef && anchorRef.current) {
            const rect = anchorRef.current.getBoundingClientRect();
            const width = minWidth || rect.width;
            let top = rect.bottom + 8;
            let left = rect.left;
            // Optional: prevent overflow at bottom
            const dropdownHeight = 250; // Adjust as needed
            if (top + dropdownHeight > window.innerHeight) {
                top = Math.max(rect.top - dropdownHeight - 8, 8);
            }
            setStyle({
                position: "fixed",
                top,
                left,
                minWidth: typeof width === "number" ? `${width}px` : width,
                zIndex: 9999,
            });
        }
    }, [open, anchorRef, minWidth]);

    // Click outside to close
    React.useEffect(() => {
        if (!open || !anchorRef) return;
        function handleClick(event: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node) && anchorRef.current && !anchorRef.current.contains(event.target as Node)) {
                onClose();
            }
        }
        document.addEventListener("mousedown", handleClick);
        return () => document.removeEventListener("mousedown", handleClick);
    }, [open, onClose, anchorRef]);

    if (!open || !anchorRef) return null;

    return createPortal(
        <div ref={dropdownRef} style={style} className="bg-white border border-gray-200 rounded-lg shadow-lg" tabIndex={-1}>
            {children}
        </div>,
        document.body,
    );
};

export default PortalDropdown;
