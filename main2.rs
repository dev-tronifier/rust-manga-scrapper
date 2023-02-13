extern crate raster;
extern crate editor;

use std::fs::{self, File};
use std::io::BufWriter;

use jpeg_to_pdf::JpegToPdf;


fn savePdf() {
    let imgs = [
        "./src/files/00-205.jpg",
        "./src/files/01-323.jpg",
        "./src/files/02-323.jpg",
        "./src/files/03-320.jpg",
        "./src/files/04-320.jpg",
        "./src/files/05-319.jpg",
        "./src/files/06-318.jpg",
        "./src/files/07-318.jpg",
        "./src/files/08-298.jpg",
        "./src/files/09-275.jpg",
        "./src/files/page100-10.jpg",
        "./src/files/ENDING-PAGE.jpg",
    ];
    let mut doc = JpegToPdf::new();
    for img in imgs {
        doc = doc.add_image(fs::read(img).unwrap());
    };
    doc.create_pdf(&mut BufWriter::new(File::create("./src/files/out.pdf").unwrap())).unwrap();
    // let out_file = File::create("./src/files/out.pdf").unwrap();
    // JpegToPdf::new()
    // .add_image(fs::read("./src/files/00-205.jpg").unwrap())
    // .add_image(fs::read("./src/files/01-323.jpg").unwrap())
    // .add_image(fs::read("./src/files/02-323.jpg").unwrap())
    // .add_image(fs::read("./src/files/03-320.jpg").unwrap())
    // .create_pdf(&mut BufWriter::new(out_file))
    // .unwrap();
}

fn main() {
    // savePdf();
    let mut doc = JpegToPdf::new();
    let imgs = [
        "./src/files/00-205.jpg",
        "./src/files/01-323.jpg",
        "./src/files/02-323.jpg",
        "./src/files/03-320.jpg",
        "./src/files/04-320.jpg",
        "./src/files/05-319.jpg",
        "./src/files/06-318.jpg",
        "./src/files/07-318.jpg",
        "./src/files/08-298.jpg",
        "./src/files/09-275.jpg",
        "./src/files/page100-10.jpg",
        "./src/files/ENDING-PAGE.jpg",
    ];
    let mut pcnt: u32 = 1;
    let mut ph = 1754;
    let mut pw = 1240;
    let mut page = raster::Image::blank(pw, ph);
    for img_name in imgs {
        println!("Processing Image: {}", img_name);
        let mut img_off = 0;
        let img = raster::open(img_name).unwrap();
        let w = img.width;
        let mut dh = img.height;
        while true {
            let mut cpy = img.clone();
            if dh >= ph {
                raster::editor::crop(&mut cpy, w, ph, raster::PositionMode::TopLeft, 0, img_off).unwrap();
                img_off += ph;
                dh -= ph;
                let fname = "./src/files/test".to_string() + &pcnt.to_string() + &".jpg".to_string();
                // let fname_cpy = "./src/files/cpy".to_string() + &pcnt.to_string() + &".jpg".to_string();
                page = raster::editor::blend(&mut page, &cpy, raster::BlendMode::Normal, 1.0, raster::PositionMode::TopCenter, 0, 1754 - ph).unwrap();
                raster::save(&mut page, &fname).unwrap();
                doc = doc.add_image(fs::read(&fname).unwrap());
                // raster::save(&mut cpy, &fname_cpy).unwrap();
                page = raster::Image::blank(pw, 1754);
                ph = 1754;
                pcnt += 1;
                if dh == 0 { break; }
            } else {
                raster::editor::crop(&mut cpy, w, dh, raster::PositionMode::TopLeft, 0, img_off).unwrap();
                page = raster::editor::blend(&mut page, &cpy, raster::BlendMode::Normal, 1.0, raster::PositionMode::TopCenter, 0, 1754 - ph).unwrap();
                ph -= dh;
                // pcnt += 1;
                break;
            }
        }
    }
    let out_file = File::create("./src/files/out1.pdf").unwrap();
    doc.create_pdf(&mut BufWriter::new(out_file)).unwrap();
}