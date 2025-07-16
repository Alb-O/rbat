use blend_file_reader::*;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_path() {
        // Create a temporary copy of the test file
        let test_file = "tests/test_blend_files/linked_cube.blend";
        let temp_file = "tests/test_blend_files/temp_linked_cube.blend";

        // Copy the original file to temp location
        fs::copy(test_file, temp_file).expect("Failed to copy test file");

        // Open the blend file in read+write mode
        let mut blend_file =
            BlendFile::open_read_write(temp_file).expect("Failed to open blend file");

        // Find all library link blocks
        let mut library_blocks = blend_file
            .get_blocks_by_type_mut(b"LI")
            .expect("Failed to get library blocks");

        // Change the path in the first library block
        if let Some(block) = library_blocks.first_mut() {
            let new_path = "//new_library_path.blend";
            block
                .set_string_field("name", new_path)
                .expect("Failed to set library path");
        }

        // Save the changes
        blend_file.save().expect("Failed to save blend file");

        // Reopen the file and verify the change
        let reopened_file = BlendFile::open(temp_file).expect("Failed to reopen blend file");
        let library_blocks = reopened_file
            .get_blocks_by_type(b"LI")
            .expect("Failed to get library blocks");

        if let Some(block) = library_blocks.first() {
            let path = block
                .get_string_field("name")
                .expect("Failed to get library path");
            assert_eq!(path, "//new_library_path.blend");
        }

        // Clean up
        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }

    #[test]
    fn test_block_hash() {
        // Create a temporary copy of the test file
        let test_file = "tests/test_blend_files/linked_cube.blend";
        let temp_file = "tests/test_blend_files/temp_linked_cube_hash.blend";

        // Copy the original file to temp location
        fs::copy(test_file, temp_file).expect("Failed to copy test file");

        // Open the blend file in read+write mode
        let mut blend_file =
            BlendFile::open_read_write(temp_file).expect("Failed to open blend file");

        // Get the original hash
        let original_hash = blend_file
            .blocks
            .iter()
            .map(|b| b.code)
            .collect::<Vec<_>>();

        // Find a library block and modify it
        let mut library_blocks = blend_file
            .get_blocks_by_type_mut(b"LI")
            .expect("Failed to get library blocks");
        if let Some(block) = library_blocks.first_mut() {
            let new_path = "//modified_path.blend";
            block
                .set_string_field("name", new_path)
                .expect("Failed to set library path");
        }

        // Save the changes
        blend_file.save().expect("Failed to save blend file");

        // Reopen and verify the change
        let reopened_file = BlendFile::open(temp_file).expect("Failed to reopen blend file");
        let new_hash = reopened_file
            .blocks
            .iter()
            .map(|b| b.code)
            .collect::<Vec<_>>();

        // The block structure should be the same
        assert_eq!(original_hash.len(), new_hash.len());

        // Verify the library path was changed
        let library_blocks = reopened_file
            .get_blocks_by_type(b"LI")
            .expect("Failed to get library blocks");
        if let Some(block) = library_blocks.first() {
            let path = block
                .get_string_field("name")
                .expect("Failed to get library path");
            assert_eq!(path, "//modified_path.blend");
        }

        // Clean up
        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }

    #[test]
    fn test_modify_object_location() {
        // Create a temporary copy of the test file
        let test_file = "tests/test_blend_files/linked_cube.blend";
        let temp_file = "tests/test_blend_files/temp_object_location.blend";

        // Copy the original file to temp location
        fs::copy(test_file, temp_file).expect("Failed to copy test file");

        // Open the blend file in read+write mode
        let mut blend_file =
            BlendFile::open_read_write(temp_file).expect("Failed to open blend file");

        // Find an object block
        let mut object_blocks = blend_file
            .get_blocks_by_type_mut(b"OB")
            .expect("Failed to get object blocks");

        // Modify the location of the first object
        if let Some(block) = object_blocks.first_mut() {
            let new_location = [10.0, 20.0, 30.0];
            block
                .set_float_array_field("loc", &new_location)
                .expect("Failed to set object location");
        }

        // Save the changes
        blend_file.save().expect("Failed to save blend file");

        // Reopen and verify the change
        let reopened_file = BlendFile::open(temp_file).expect("Failed to reopen blend file");
        let object_blocks = reopened_file
            .get_blocks_by_type(b"OB")
            .expect("Failed to get object blocks");

        if let Some(block) = object_blocks.first() {
            let location = block
                .get_float_array_field("loc", 3)
                .expect("Failed to get object location");
            assert_eq!(location, [10.0, 20.0, 30.0]);
        }

        // Clean up
        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }

    #[test]
    fn test_compressed_file_modification() {
        // Create a temporary copy of the compressed test file
        let test_file = "tests/test_blend_files/linked_cube_compressed.blend";
        let temp_file = "tests/test_blend_files/temp_compressed.blend";

        // Copy the original file to temp location
        fs::copy(test_file, temp_file).expect("Failed to copy test file");

        // Open the blend file in read+write mode
        let mut blend_file =
            BlendFile::open_read_write(temp_file).expect("Failed to open compressed blend file");

        // Find a library block and modify it
        let mut library_blocks = blend_file
            .get_blocks_by_type_mut(b"LI")
            .expect("Failed to get library blocks");
        if let Some(block) = library_blocks.first_mut() {
            let new_path = "//compressed_modified.blend";
            block
                .set_string_field("name", new_path)
                .expect("Failed to set library path");
        }

        // Save the changes
        blend_file
            .save()
            .expect("Failed to save compressed blend file");

        // Reopen and verify the change
        let reopened_file =
            BlendFile::open(temp_file).expect("Failed to reopen compressed blend file");
        let library_blocks = reopened_file
            .get_blocks_by_type(b"LI")
            .expect("Failed to get library blocks");

        if let Some(block) = library_blocks.first() {
            let path = block
                .get_string_field("name")
                .expect("Failed to get library path");
            assert_eq!(path, "//compressed_modified.blend");
        }

        // Clean up
        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }
}
