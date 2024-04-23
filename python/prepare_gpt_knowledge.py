import os
from pathlib import Path


def concatenate_files(output_file, directories):
    with open(output_file, "w", encoding="utf-8") as outfile:
        # Writing metadata about the structure and linking files to directory
        for directory in directories:
            outfile.write(f"# Directory: {directory}\n")
            for root, _, files in os.walk(directory):
                for file in files:
                    if file.endswith(".rs") or file.endswith(
                        ".glsl"
                    ):  # Assuming Rust files have .rs extension or a GLSL file
                        file_path = Path(root) / file
                        relative_path = file_path.relative_to(directory)
                        # Adding structured header with file metadata
                        outfile.write(
                            f"\n// File: {relative_path}\n// Path: {file_path}\n"
                        )
                        outfile.write(f"/* Start of file {relative_path} */\n")
                        with open(file_path, "r", encoding="utf-8") as infile:
                            outfile.write(infile.read() + "\n")
                        outfile.write(f"/* End of file {relative_path} */\n\n")


if __name__ == "__main__":
    # Define directories to search for .rs files
    directories = ["../src"]
    concatenate_files("gpt-knowledge.txt", directories)
