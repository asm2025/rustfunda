import { useState, useEffect, useTransition } from "react";
import { Toaster } from "react-hot-toast";
import logo from "./assets/gallery.svg";
import Hero from "./components/Hero";
import ImageUpload from "./components/ImageUpload";
import ImageGrid from "./components/ImageGrid";
import ImageModal from "./components/ImageModal";
import { ImageModel } from "./types";
import { imageApi } from "./services/api";
import toast from "react-hot-toast";

function App() {
    const [images, setImages] = useState<ImageModel[]>([]);
    const [selectedImage, setSelectedImage] = useState<ImageModel | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isPending, startTransition] = useTransition();

    useEffect(() => {
        loadImages();
    }, []);

    const loadImages = () => {
        startTransition(async () => {
            try {
                const response = await imageApi.getImages();
                setImages(response.data);
            } catch (error) {
                toast.error("Failed to load images");
                console.error("Load images error:", error);
            } finally {
                setIsLoading(false);
            }
        });
    };

    const handleImageUploaded = (newImage: ImageModel) => {
        setImages((prev) => [newImage, ...prev]);
    };

    const handleImageUpdate = (updatedImage: ImageModel) => {
        setImages((prev) => prev.map((img) => (img.id === updatedImage.id ? updatedImage : img)));
        setSelectedImage(updatedImage);
    };

    const handleImageDelete = (imageId: number) => {
        setImages((prev) => prev.filter((img) => img.id !== imageId));
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

                {!images || images.length === 0 ? (
                    <div className="text-center py-12">
                        <p className="text-gray-500 text-lg">No images uploaded yet. Upload your first image above!</p>
                    </div>
                ) : (
                    <ImageGrid images={images} selectedImage={selectedImage} onImageSelect={setSelectedImage} />
                )}
            </div>

            {selectedImage && <ImageModal image={selectedImage} onClose={() => setSelectedImage(null)} onUpdate={handleImageUpdate} onDelete={handleImageDelete} />}
        </div>
    );
}

export default App;
