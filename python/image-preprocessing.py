from PIL import Image
import os


def preprocess_images(input_dir, output_dir, max_size=512, exclude_list=None):
    if exclude_list is None:
        exclude_list = []

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    for filename in os.listdir(input_dir):
        if filename in exclude_list:
            print(f"Skipping {filename} (excluded)")
            continue

        if filename.lower().endswith((".png", ".jpg", ".jpeg", ".bmp", ".tiff")):
            input_path = os.path.join(input_dir, filename)
            output_filename = os.path.splitext(filename)[0] + ".png"
            output_path = os.path.join(output_dir, output_filename)

            with Image.open(input_path) as img:
                # Convert to RGBA mode
                rgba_img = img.convert("RGBA")

                # Resize if larger than max_size
                if max(rgba_img.size) > max_size:
                    rgba_img.thumbnail((max_size, max_size), Image.LANCZOS)
                    print(f"Resized {filename} to {rgba_img.size}")

                # Save as PNG
                rgba_img.save(output_path, "PNG", optimize=True)
                print(f"Processed {filename} to RGBA PNG")


# Example usage
input_directory = "../src/res/textures"
output_directory = "../src/res/textures"
excluded_textures = ["skybox.png", "skybox.hdr"]
image_max_size = 512

preprocess_images(
    input_directory,
    output_directory,
    max_size=image_max_size,
    exclude_list=excluded_textures,
)
