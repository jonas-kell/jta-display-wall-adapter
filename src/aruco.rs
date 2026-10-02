use calib_targets_aruco::builtins::builtin_dictionary;

const FALLBACK: [[bool; 2]; 2] = [[false, false], [false, false]];
const DICT_NAME: &str = "DICT_6X6_1000";

fn aruco_bits_no_border(id: usize) -> Vec<Vec<bool>> {
    if let Some(dict) = builtin_dictionary(DICT_NAME) {
        if let Some(code) = dict.codes().get(id) {
            let marker_width = dict.marker_size();

            return (0..marker_width)
                .map(|y| {
                    (0..marker_width)
                        .map(|x| {
                            let bit = y * marker_width + x;
                            ((code >> bit) & 1) == 0
                        })
                        .collect()
                })
                .collect();
        } else {
            warn!("Selection outside of marker range");
        }
    } else {
        error!("Pre-selected dict not available");
    }

    return FALLBACK.iter().map(|a| a.into()).collect();
}

pub fn aruco_bits(id: usize) -> Vec<Vec<bool>> {
    let mut inner = aruco_bits_no_border(id);

    const BORDER_ELEMENT: bool = false;

    let width: usize = inner.get(0).map_or(0, |a| a.len());

    let line: Vec<bool> = vec![BORDER_ELEMENT; width].into();
    inner.insert(0, line.clone());
    inner.push(line);

    inner.iter_mut().for_each(|v| {
        v.insert(0, BORDER_ELEMENT);
        v.push(BORDER_ELEMENT);
    });

    let line2: Vec<bool> = vec![!BORDER_ELEMENT; width + 2].into();
    inner.insert(0, line2.clone());
    inner.push(line2);

    inner.iter_mut().for_each(|v| {
        v.insert(0, !BORDER_ELEMENT);
        v.push(!BORDER_ELEMENT);
    });

    inner
}

pub fn aruco_image(id: usize, width: usize) -> Result<Vec<u8>, image::ImageError> {
    let bitmap: Vec<Vec<bool>> = aruco_bits(id);

    let vec_height = bitmap.len() as u32;
    let vec_width = bitmap.first().map_or(0, |row| row.len()) as u32;

    let img = image::ImageBuffer::from_fn(vec_width, vec_height, |x, y| {
        image::Luma([if bitmap[y as usize][x as usize] {
            255u8
        } else {
            0u8
        }])
    });

    let imgres = image::imageops::resize(
        &img,
        width as u32,
        width as u32,
        image::imageops::FilterType::Nearest,
    );

    let mut data = std::io::Cursor::new(Vec::new());
    imgres.write_to(&mut data, image::ImageFormat::Png)?;

    Ok(data.into_inner())
}
