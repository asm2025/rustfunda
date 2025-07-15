import React, { useState } from "react";

interface Props {
    src: string;
    alt?: string | null;
    className?: string;
    phClassName?: string;
    style?: React.CSSProperties;
}

const ImageWithFallback: React.FC<Props> = ({ src, alt, className, phClassName, style }) => {
    const [imgError, setImgError] = useState(false);

    const FallbackSVG = () => (
        <svg viewBox="0 0 20 20" fill="currentColor" className={phClassName} style={style}>
            <path fillRule="evenodd" d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm12 12H4l4-8 3 6 2-4 3 6z" clipRule="evenodd" />
        </svg>
    );

    if (imgError) {
        return <FallbackSVG />;
    }

    return <img src={src} alt={alt || ""} className={className} style={style} onLoad={() => setImgError(false)} onError={() => setImgError(true)} />;
};

export default ImageWithFallback;
