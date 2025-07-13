import React from "react";
import { ImageModel } from "../types";
import humanize from "humanize-plus";

interface ImageCardProps {
    image: ImageModel;
    onClick: () => void;
    isSelected?: boolean;
}

const ImageCard: React.FC<ImageCardProps> = ({ image, onClick, isSelected }) => {
    return (
        <div className={`card card-hover max-w-xs cursor-pointer animate-scale-in ${isSelected ? "ring-4 ring-blue-500 scale-105" : ""}`} onClick={onClick}>
            <div className="aspect-square bg-gray-200 m-1 flex items-center justify-center">
                <div className="text-gray-400 text-center w-full mx-auto p-4">
                    <svg className="w-16 h-16 mx-auto mb-2" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm12 12H4l4-8 3 6 2-4 3 6z" clipRule="evenodd" />
                    </svg>
                    <p className="text-sm truncate">{image.filename}</p>
                </div>
            </div>
            <div className="p-4">
                <h3 className="font-semibold text-gray-900 truncate">{image.title}</h3>
                <p className="text-sm text-gray-500 mt-1">{humanize.fileSize(image.file_size)}</p>
            </div>
        </div>
    );
};

export default ImageCard;
