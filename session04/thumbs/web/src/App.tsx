import { useState, useEffect } from "react";
import { Toaster } from "react-hot-toast";
import logo from "./assets/gallery.svg";
import Hero from "./components/Hero";
import ImageUpload from "./components/ImageUpload";
import ImageGrid from "./components/ImageGrid";
import ImageModal from "./components/ImageModal";
import { ResultSet, ImageModel, ModelWithRelated, TagModel } from "./types";
import { imageApi } from "./services/api";
import toast from "react-hot-toast";

function App() {
    const [imagesSet, setImagesSet] = useState<ResultSet<ModelWithRelated<ImageModel, TagModel>>>({
        data: [],
        total: 0,
    });
    const [selectedImage, setSelectedImage] = useState<ModelWithRelated<ImageModel, TagModel> | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        loadImages();
    }, []);

    const loadImages = async () => {
        try {
            const response = await imageApi.getImages();
            setImagesSet(response.data);
        } catch (error) {
            toast.error("Failed to load images");
            console.error("Load images error:", error);
        } finally {
            setIsLoading(false);
        }
    };

    const handleImageUploaded = (newImage: ImageModel) => {
        setImagesSet((prev) => ({
            data: [{ item: newImage, related: [] }, ...prev.data],
            total: prev.total + 1,
            pagination: prev.pagination,
        }));
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
            <div className="min-h-screen flex items-center justify-center">
                <div className="text-xl">Loading...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-50">
            <Toaster position="top-right" />

            <Hero title="Image Gallery" subtitle="Manage and organize your images with tags" logo={logo} />

            <div className="container mx-auto px-4 py-8">
                <ImageUpload onUpload={handleImageUploaded} />

                {!imagesSet.data || imagesSet.data.length === 0 ? (
                    <div className="text-center py-12">
                        <p className="text-gray-500 text-lg">No images uploaded yet. Upload your first image above!</p>
                    </div>
                ) : (
                    <ImageGrid images={imagesSet.data} selectedImage={selectedImage} onImageSelect={setSelectedImage} />
                )}
            </div>

            {selectedImage && <ImageModal image={selectedImage.item} tags={selectedImage.related} onClose={() => setSelectedImage(null)} onUpdate={handleImageUpdate} onDelete={handleImageDelete} />}
        </div>
    );
}

export default App;
