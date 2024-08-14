from PIL import Image
import os


def find_files(directory, extension):
    file_list = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith(extension):
                file_list.append(os.path.join(root, file))
    return file_list


def preprocess_images(input_dir, max_size=512, exclude_list=None):
    if exclude_list is None:
        exclude_list = []

    files = find_files(input_dir, ".png")
    files += find_files(input_dir, ".jpg")
    files += find_files(input_dir, ".jpeg")
    files += find_files(input_dir, ".bmp")
    files += find_files(input_dir, ".tiff")
    files += find_files(input_dir, ".hdr")

    for file_path in files:
        filename = os.path.basename(file_path)
        if filename in exclude_list:
            print(f"Skipping {filename} (excluded)")
            continue

        with Image.open(file_path) as img:
            # Convert to RGBA mode
            rgba_img = img.convert("RGBA")

            # Resize if larger than max_size
            if max(rgba_img.size) > max_size:
                rgba_img.thumbnail((max_size, max_size), Image.LANCZOS)
                print(f"Resized {filename} to {rgba_img.size}")

            # Save as PNG
            rgba_img.save(file_path, "PNG", optimize=True)
            print(f"Processed {filename} to RGBA PNG")


# Example usage
input_directory = "../src/res/textures"
excluded_textures = ["skybox.png", "skybox.hdr"]
image_max_size = 512

preprocess_images(
    input_directory,
    max_size=image_max_size,
    exclude_list=excluded_textures,
)
