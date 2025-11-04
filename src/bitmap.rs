use image::GenericImageView;

pub fn png_to_bmp_bytes(path: &str) -> Vec<u8> {
    // Load the image
    let img = image::open(path).expect("Failed to open image");
    let (width, height) = img.dimensions();

    // Convert to RGBA8 (32-bit)
    let img = img.to_rgba8();

    // BMP headers
    let file_header_size = 14;
    let dib_header_size = 40;
    let pixel_data_offset = file_header_size + dib_header_size;
    let bytes_per_pixel = 4;
    let row_bytes = width * bytes_per_pixel;
    let pixel_data_size = row_bytes * height;
    let file_size = pixel_data_offset + pixel_data_size;

    let mut bmp_bytes = Vec::with_capacity(file_size as usize);

    // --- BMP File Header ---
    bmp_bytes.extend(b"BM"); // Signature
    bmp_bytes.extend(&(file_size as u32).to_le_bytes()); // File size
    bmp_bytes.extend(&[0u8; 4]); // Reserved
    bmp_bytes.extend(&(pixel_data_offset as u32).to_le_bytes()); // Pixel data offset

    // --- DIB Header (BITMAPINFOHEADER) ---
    bmp_bytes.extend(&(dib_header_size as u32).to_le_bytes()); // DIB header size
    bmp_bytes.extend(&(width as i32).to_le_bytes()); // Width
    bmp_bytes.extend(&(height as i32).to_le_bytes()); // Height
    bmp_bytes.extend(&1u16.to_le_bytes()); // Planes
    bmp_bytes.extend(&(32u16).to_le_bytes()); // Bits per pixel
    bmp_bytes.extend(&0u32.to_le_bytes()); // Compression (0 = none)
    bmp_bytes.extend(&(pixel_data_size as u32).to_le_bytes()); // Image size
    bmp_bytes.extend(&0u32.to_le_bytes()); // X pixels per meter
    bmp_bytes.extend(&0u32.to_le_bytes()); // Y pixels per meter
    bmp_bytes.extend(&0u32.to_le_bytes()); // Total colors
    bmp_bytes.extend(&0u32.to_le_bytes()); // Important colors

    // --- Pixel Data ---
    // BMP stores pixels **bottom-up** (last row first)
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // BMP uses BGRA order
            bmp_bytes.push(pixel[2]); // B
            bmp_bytes.push(pixel[1]); // G
            bmp_bytes.push(pixel[0]); // R
            bmp_bytes.push(pixel[3]); // A
        }
    }

    bmp_bytes
}
