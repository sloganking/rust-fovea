use std::{fs, io, path::Path};

use image::{
    imageops::{self, FilterType},
    DynamicImage, GenericImage, GenericImageView,
};

fn resize_image(img: &DynamicImage, factor: u32) -> DynamicImage {
    img.resize(
        img.dimensions().0 / factor,
        img.dimensions().1 / factor,
        FilterType::Lanczos3,
    )
}

/// remove contents inside a directory, without deleting the directory itself.
fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry.unwrap().path();

        if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            panic!("https://i.kym-cdn.com/entries/icons/original/000/013/306/2dd.jpg")
        }
    }
    Ok(())
}

/// Erases all content of an existing directory, or creates an empty new one.
fn clean_dir(path: &Path) {
    // clear any existing output_dir
    if path.is_dir() {
        remove_dir_contents(path).unwrap();
    } else {
        fs::create_dir(path).unwrap();
    }
}

/// returns a sector of an image.
fn slice_image(img: &DynamicImage, point1: (u32, u32), point2: (u32, u32)) -> DynamicImage {
    let smallest_x = point1.0.min(point2.0);
    let smallest_y = point1.1.min(point2.1);
    let largest_x = point1.0.max(point2.0);
    let largest_y = point1.1.max(point2.1);

    let width = largest_x - smallest_x;
    let height = largest_y - smallest_y;

    // make new image
    let mut out = DynamicImage::new_rgb8(width, height);

    for x in smallest_x..largest_x {
        for y in smallest_y..largest_y {
            let pixel = img.get_pixel(x, y);
            // put pixel in new image
            out.put_pixel(x - smallest_x, y - smallest_y, pixel);
        }
    }
    out
}

fn reconstruct_image(
    images: &[DynamicImage],
    dimensions: (u32, u32),
    divisions: u32,
) -> DynamicImage {
    let mut out = DynamicImage::new_rgb8(dimensions.0, dimensions.1);

    for (i, img) in images.iter().enumerate().rev() {
        let x_size = dimensions.0 / (divisions - i as u32);
        let y_size = dimensions.1 / (divisions - i as u32);

        // resize to correct size (size depends on layer)
        let img = img.resize(x_size, y_size, FilterType::Nearest);

        // overlay image in center of output image
        let (x, y) = (
            out.dimensions().0 / 2 - img.dimensions().0 / 2,
            out.dimensions().1 / 2 - img.dimensions().1 / 2,
        );
        imageops::overlay(&mut out, &img, x as i64, y as i64);
    }
    out
}

fn main() {
    println!("Resizing image...");

    clean_dir(Path::new("./output/"));

    let mut images: Vec<DynamicImage> = Vec::new();

    images.push(image::open("input.png").unwrap());

    let divisions: u32 = 4;
    let factor: u32 = 2;

    // resize images
    for x in 0..divisions - 1 {
        let input_img = images[x as usize].clone();
        images.push(resize_image(&input_img, factor));
    }

    // slice images
    let mut new_images: Vec<DynamicImage> = Vec::new();
    for (i, image) in images.iter_mut().enumerate() {
        let (width, height) = image.dimensions();
        let center_coords = (width / 2, height / 2);

        let x_size = width / (divisions - i as u32);
        let y_size = height / (divisions - i as u32);

        let slice = slice_image(
            image,
            (center_coords.0 - x_size / 2, center_coords.1 - y_size / 2),
            (center_coords.0 + x_size / 2, center_coords.1 + y_size / 2),
        );
        new_images.push(slice);
    }
    let images = new_images;

    // save images to output directory
    for (x, image) in images.iter().enumerate() {
        image.save(format!("./output/{}.png", x)).unwrap();
    }

    let reconstruction = reconstruct_image(&images, (512, 512), divisions);

    reconstruction
        .save(Path::new("./output/reconstruction.png"))
        .unwrap();

    // DEBUG test function
    // let img = slice_image(images[0].clone(), (0,256), (512,512));
    // // save
    // img.save("slice_test.png").unwrap();
}
