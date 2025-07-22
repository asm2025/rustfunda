// components/Home.tsx
import React from "react";

const Home: React.FC = () => {
    return (
        <div className="max-w-4xl mx-auto bg-white rounded-lg shadow-sm border border-gray-200 p-8">
            <h1 className="text-4xl font-extrabold text-gray-900 mb-6 text-center">Welcome to Your Image Gallery!</h1>
            <p className="text-lg text-gray-700 mb-4 leading-relaxed">
                This is your personal space to manage, organize, and showcase all your favorite images. Whether it's stunning landscapes, cherished memories, or creative projects, our gallery provides a seamless experience to keep your visual collection in
                perfect order.
            </p>
            <p className="text-lg text-gray-700 mb-6 leading-relaxed">
                Navigate to the <strong className="font-semibold text-blue-600">Images</strong> section to view your existing collection or upload new pictures. You can easily add titles, descriptions, and tags to each image, making them easy to find and
                categorize.
            </p>
            <div className="bg-blue-50 border-l-4 border-blue-500 text-blue-800 p-4 mb-6" role="alert">
                <p className="font-bold">Get Started:</p>
                <p>Click on "Images" in the navigation bar to begin uploading and managing your photos!</p>
            </div>
            <h2 className="text-2xl font-bold text-gray-800 mb-4">Key Features:</h2>
            <ul className="list-disc list-inside text-gray-700 space-y-2 mb-6">
                <li>Effortless image uploading with drag-and-drop support.</li>
                <li>Organize images with custom titles, descriptions, and tags.</li>
                <li>Responsive grid view for a beautiful display on any device.</li>
                <li>Detailed image view with editing and deletion options.</li>
                <li>Securely store your images on the backend.</li>
            </ul>
            <p className="text-lg text-gray-700 text-center">We hope you enjoy using your new image gallery!</p>
        </div>
    );
};

export default Home;
