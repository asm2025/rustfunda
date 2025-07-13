import React from "react";
import { ImageModel } from "../types";
import ImageCard from "./ImageCard";

interface ImageGridProps {
    images: ImageModel[];
    selectedImage: ImageModel | null;
    onImageSelect: (image: ImageModel) => void;
}

const ImageGrid: React.FC<ImageGridProps> = ({ images, selectedImage, onImageSelect }) => {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
            {images.map((image) => (
                <ImageCard key={image.id} image={image} onClick={() => onImageSelect(image)} isSelected={selectedImage?.id === image.id} />
            ))}
        </div>
    );
};

export default ImageGrid;
