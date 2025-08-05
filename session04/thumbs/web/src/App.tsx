import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { Toaster } from "react-hot-toast";
import logo from "./assets/gallery.svg";
import Hero from "./components/Hero";
import Navigation from "./components/Navigation";
import Home from "./pages/Home";
import Images from "./pages/Images";
import About from "./pages/About";

function App() {
    return (
        <Router>
            <div className="min-h-screen bg-gray-50">
                <Toaster position="top-right" />
                <Hero title="Image Gallery" subtitle="Manage and organize your images with tags" logo={logo} />
                <Navigation />
                <main className="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" element={<Home />} />
                        <Route path="/images" element={<Images />} />
                        <Route path="/about" element={<About />} />
                    </Routes>
                </main>
            </div>
        </Router>
    );
}

export default App;
