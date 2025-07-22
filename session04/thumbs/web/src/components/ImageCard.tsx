import React from "react";
import { ImageModel } from "../types";
import humanize from "humanize-plus";
import ImageWithFallback from "./ImageWithFallback";
import { imageApi } from "../services/api";

interface ImageCardProps {
    image: ImageModel;
    onClick: () => void;
    isSelected?: boolean;
}

const ImageCard: React.FC<ImageCardProps> = ({ image, onClick, isSelected }) => {
    const filename = `${image.id}.${image.extension}`;
    return (
        <div className={`card card-hover max-w-xs cursor-pointer animate-scale-in ${isSelected ? "ring-4 ring-blue-500 scale-105" : ""}`} onClick={onClick}>
            <div className="aspect-square bg-gray-200 m-1 flex items-center justify-center">
                <div className="text-gray-400 text-center w-full mx-auto p-4">
                    <ImageWithFallback src={imageApi.getThumbUri(filename)} alt={image.alt_text} className="w-full h-auto rounded" phClassName="w-16 h-16 mx-auto mb-2" />
                    <p className="text-sm truncate">{filename}</p>
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
