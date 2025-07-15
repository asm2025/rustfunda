import React from "react";
import { ImageModel, ModelWithRelated, TagModel } from "../types";
import ImageCard from "./ImageCard";

interface ImageGridProps {
    images: ModelWithRelated<ImageModel, TagModel>[];
    selectedImage: ModelWithRelated<ImageModel, TagModel> | null;
    onImageSelect: (image: ModelWithRelated<ImageModel, TagModel>) => void;
}

const ImageGrid: React.FC<ImageGridProps> = ({ images, selectedImage, onImageSelect }) => {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
            {images.length > 0 && images.map((model) => <ImageCard key={model.item.id} image={model.item} onClick={() => onImageSelect(model)} isSelected={selectedImage?.item.id === model.item.id} />)}
        </div>
    );
};

export default ImageGrid;
