import React, { useState, useEffect } from "react";
import toast from "react-hot-toast";
import ImageUpload from "../components/ImageUpload";
import ImageGrid from "../components/ImageGrid";
import ImageModal from "../components/ImageModal";
import { ResultSet, ImageModel, ModelWithRelated, TagModel } from "../types";
import { thumbsApi } from "../services/api";

const Images: React.FC = () => {
    const [imagesSet, setImagesSet] = useState<ResultSet<ModelWithRelated<ImageModel, TagModel>>>({
        data: [],
        total: 0,
    });
    const [selectedImage, setSelectedImage] = useState<ModelWithRelated<ImageModel, TagModel> | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [showUpload, setShowUpload] = useState(false);

    useEffect(() => {
        loadImages();
    }, []);

    const loadImages = async () => {
        try {
            const response = await thumbsApi.getImages();
            setImagesSet(response.data);
        } catch (error) {
            toast.error("Failed to load images");
            console.error("Load images error:", error);
        } finally {
            setIsLoading(false);
        }
    };

    const handleImageUploaded = async (newImage: ImageModel) => {
        try {
            const response = await thumbsApi.getImage(newImage.id!);
            setImagesSet((prev) => ({
                data: [response.data, ...prev.data],
                total: prev.total + 1,
                pagination: prev.pagination,
            }));
            setShowUpload(false); // Close upload modal after successful upload
        } catch (error) {
            toast.error("Failed to load images");
            console.error("Load images error:", error);
        }
    };

    const handleImageUpdate = (updatedImage: ImageModel) => {
        setImagesSet((prev) => ({
            ...prev,
            data: prev.data.map((model) => (model.item.id === updatedImage.id ? { item: updatedImage, related: model.related } : model)),
        }));
        const selectedImg = imagesSet.data.find((e) => e.item.id === updatedImage.id);
        setSelectedImage(selectedImg || null);
    };

    const handleImageDelete = (imageId: number) => {
        setImagesSet((prev) => ({
            ...prev,
            data: prev.data.filter((model) => model.item.id !== imageId),
            total: prev.total - 1,
        }));
        setSelectedImage(null);
    };

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-12">
                <div className="text-xl">Loading images...</div>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {/* Upload Button */}
            <div className="flex justify-between items-center">
                <h1 className="text-3xl font-bold text-gray-900">Images</h1>
                <button type="button" onClick={() => setShowUpload(true)} className="btn-primary flex items-center gap-2">
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                    </svg>
                    Upload
                </button>
            </div>

            {/* Images Grid */}
            {!imagesSet.data || imagesSet.data.length === 0 ? (
                <div className="text-center py-12">
                    <p className="text-gray-500 text-lg">No images uploaded yet. Upload your first image!</p>
                </div>
            ) : (
                <ImageGrid images={imagesSet.data} selectedImage={selectedImage} onImageSelect={setSelectedImage} />
            )}

            {/* Upload Modal */}
            {showUpload && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
                    <div className="bg-white rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto">
                        <div className="flex justify-between items-center p-4 border-b">
                            <h2 className="text-xl font-semibold">Upload New Image</h2>
                            <button type="button" onClick={() => setShowUpload(false)} className="text-gray-500 hover:text-gray-700">
                                x
                            </button>
                        </div>
                        <div className="p-4">
                            <ImageUpload onUpload={handleImageUploaded} />
                        </div>
                    </div>
                </div>
            )}

            {/* Image Modal */}
            {selectedImage && <ImageModal image={selectedImage.item} tags={selectedImage.related} onClose={() => setSelectedImage(null)} onUpdate={handleImageUpdate} onDelete={handleImageDelete} />}
        </div>
    );
};

export default Images;
