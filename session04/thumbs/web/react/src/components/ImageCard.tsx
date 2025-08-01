import React, { useState, useTransition } from "react";
import toast from "react-hot-toast";
import { ImageModel } from "../types";
import humanize from "humanize-plus";
import { thumbsApi } from "../services/api";
import ImageWithFallback from "./ImageWithFallback";

interface ImageCardProps {
    image: ImageModel;
    onClick: () => void;
    isSelected?: boolean;
    onDelete: (imageId: number) => void;
}

const ImageCard: React.FC<ImageCardProps> = ({ image, onClick, isSelected, onDelete }) => {
    const [isPending, startTransition] = useTransition();

    const handleDelete = async () => {
        if (window.confirm("Are you sure you want to delete this image?")) {
            startTransition(async () => {
                try {
                    await thumbsApi.deleteImage(image.id!);
                    onDelete(image.id!);
                    toast.success("Image deleted successfully!");
                } catch (error) {
                    toast.error("Failed to delete image.");
                    console.error(error);
                }
            });
        }
    };

    const filename = `${image.id}.${image.extension}`;

    return (
        <div className={`card card-hover max-w-xs cursor-pointer animate-scale-in flex flex-col ${isSelected ? "ring-4 ring-blue-500 scale-105" : ""}`} onClick={onClick}>
            <div className="aspect-square m-1 flex items-center justify-center">
                <div className="text-gray-400 text-center w-full mx-auto p-1">
                    <ImageWithFallback src={thumbsApi.getThumbUri(filename)} alt={image.alt_text} className="w-full h-auto rounded" phClassName="w-16 h-16 mx-auto mb-2" />
                    <p className="text-sm truncate">{filename}</p>
                </div>
            </div>
            <div className="p-3 flex-1">
                <h3 className="font-semibold text-gray-900 truncate">{image.title}</h3>
                <p className="text-sm text-gray-500 mt-1">{humanize.fileSize(image.file_size)}</p>
            </div>
            <div className="flex justify-end space-x-1 pt-4 m-1 mt-auto">
                <button
                    type="button"
                    title="Delete"
                    onClick={handleDelete}
                    disabled={isPending}
                    className="text-red-700 hover:text-white border border-red-700 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 font-medium rounded-lg text-sm p-1 text-center me-2 mb-2 dark:border-red-500 dark:text-red-500 dark:hover:text-white dark:hover:bg-red-600 dark:focus:ring-red-900 flex items-center gap-2">
                    {isPending ? (
                        "Deleting..."
                    ) : (
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1-1H8a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    )}
                </button>
            </div>
        </div>
    );
};

export default ImageCard;
