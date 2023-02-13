extern crate reqwest;

use std::fs;
use std::io;
use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use scraper;
use url::Url;

use std::io::BufWriter;

use jpeg_to_pdf::JpegToPdf;

#[tokio::main]
async fn main() {
    let mut manga_name = String::new();
    print!("Search Manga: ");
    io::Write::flush(&mut io::stdout()).expect("flush failed!");
    io::stdin().read_line(&mut manga_name).expect("Read");
    let manga_url_option = await_search(&manga_name.trim()).await;
    match manga_url_option {
        Some(manga_url) => {
            println!("{}", manga_url);
            let chapter_list = await_select_chapter(&manga_url).await;
            match chapter_list {
                Some (urls) => {
                    // let tasks: Vec<_> = Vec::new();
                    // for url in urls {
                    //     tasks.push(tokio::spawn(async {
                    //         get_chapter(&url).await;
                    //     }));
                    // }
                    // for task in tasks {
                    //     task.await.unwrap();
                    // }
                    get_chapter(urls).await;
                },
                None => {
                    println!("No Chapters found!");
                }
            }
        },
        None => {
            println!("No search found!");
            return;
        }
    }
}

async fn html_parser(url: String, parse_fragment: bool) -> scraper::Html {
    if parse_fragment {
        return scraper::Html::parse_fragment(&url);
    }
    let search = reqwest::get(url)
        .await.unwrap()
        .text()
        .await.unwrap();
    return scraper::Html::parse_document(&search);
}

async fn await_search (manga_name: &str) -> Option<String> {
    let url = "https://asura.gg/?s=".to_string() + &manga_name.replace(" ", "+");
    let document = html_parser(url, false).await;
    let selector = scraper::Selector::parse(&"div.bsx>a").unwrap();
    let result = document.select(&selector).map(|x| x.value().clone());
    let mut index: u32 = 1;
    let mut manga_list: Vec<HashMap<String, String>> = Vec::new();
    for res in result {
        let mut foo = HashMap::<String, String>::new();
        for attr in res.attrs() {
            let (k, v) = attr;
            foo.insert(k.clone().to_string(), v.clone().to_string());
        }
        println!("[{index}] {}", foo.get("title").unwrap());
        manga_list.push(foo);
        index += 1;
    }
    if index == 1 {
        return None;
    }
    print!("Manga Number: ");
    io::Write::flush(&mut io::stdout()).expect("flush failed!");
    let mut x: String = String::new();
    let manga_number: u32;
    io::stdin().read_line(&mut x).expect("Read");
    manga_number = x.trim().parse().expect("parsing");
    let ret = &manga_list[(manga_number-1) as usize];
    return Some(ret.get("href").unwrap().to_string());
}

async fn await_select_chapter (manga_url: &str) -> Option<Vec<String>> {
    let document = html_parser(manga_url.to_string(), false).await;
    let selector = scraper::Selector::parse(&"div.eph-num>a").unwrap();
    let urls_iter = document.select(&selector).map(|x| x.value());
    let mut urls = Vec::new();
    for x in urls_iter {
        for attr in x.attrs() {
            let (_k, v) =  attr;
            urls.push(v);
        }
    }
    let selector = scraper::Selector::parse(&"div.eph-num>a").unwrap();
    let iter = document.select(&selector).map(|x| x.inner_html());
    let mut i = 1;
    for x in iter {
        if i == 1 {
            println!("Select chapter:");
        }
        let inner_html = html_parser(x, true).await;
        let inner_selector_chno = scraper::Selector::parse(&"span.chapternum").unwrap();
        let inner_iter = inner_html.select(&inner_selector_chno).map(|x| x.inner_html());
        for y in inner_iter {
            println!("[{i}] {y}");
            i += 1;
        }
    }
    if i == 1 {
        return None;
    }
    println!("[{i}] All Chapters");
    print!("Chapter > ");
    io::Write::flush(&mut io::stdout()).expect("flush failed!");
    let mut x: String = String::new();
    io::stdin().read_line(&mut x).expect("Read");
    let chapter_number: u32 = x.trim().parse().expect("parsing");
    let mut ret: Vec<String> = Vec::new();
    if i != chapter_number {
        ret.push(urls[(chapter_number - 1) as usize].to_string());
    } else {
        for url in urls {
            ret.push(url.to_string());
        }
    }
    return Some(ret);
}

async fn download(x: &str, fname: &str) {
    println!("[S] Downloading {x} {fname}");
    let mut file = File::create(fname).expect("err");
    let resp = reqwest::get(x).await.expect("err").bytes().await.expect("err");
    file.write_all(&resp).unwrap();
    println!("[E] Downloading {x} {fname}");
}

async fn create_pdf(imgs: &Vec<String>, fname: &str) {
    let mut doc = JpegToPdf::new();
    let mut ph = 1754;
    let pw = 1240;
    let mut page = raster::Image::blank(pw, ph);
    for img_name in imgs {
        println!("Processing Image: {}", img_name);
        let mut img_off = 0;
        let img = raster::open(&img_name).unwrap();
        let w = img.width;
        let mut dh = img.height;
        loop {
            let mut cpy = img.clone();
            if dh >= ph {
                raster::editor::crop(&mut cpy, w, ph, raster::PositionMode::TopLeft, 0, img_off).unwrap();
                img_off += ph;
                dh -= ph;
                let fname = "./src/files/tmp.jpg".to_string();
                page = raster::editor::blend(&mut page, &cpy, raster::BlendMode::Normal, 1.0, raster::PositionMode::TopCenter, 0, 1754 - ph).unwrap();
                raster::save(&mut page, &fname).unwrap();
                doc = doc.add_image(fs::read(&fname).unwrap());
                fs::remove_file(fname).unwrap();
                page = raster::Image::blank(pw, 1754);
                ph = 1754;
                if dh == 0 { break; }
            } else {
                raster::editor::crop(&mut cpy, w, dh, raster::PositionMode::TopLeft, 0, img_off).unwrap();
                page = raster::editor::blend(&mut page, &cpy, raster::BlendMode::Normal, 1.0, raster::PositionMode::TopCenter, 0, 1754 - ph).unwrap();
                ph -= dh;
                break;
            }
        }
    }
    let out_file = File::create(fname).unwrap();
    doc.create_pdf(&mut BufWriter::new(out_file)).unwrap();
}

async fn get_chapter(links: Vec<String>) {
    // let mut tasks: Vec<_> = Vec::new();
    for link in links {
        // tasks.push(tokio::spawn(async {
            let split_arr: Vec<&str> = link.trim().split(&"/").collect();
            let outfname = "./src/files/".to_owned() + split_arr[split_arr.len() - 2] + ".pdf";
            let document = html_parser(link, false).await;
            let selector = scraper::Selector::parse("img.size-full").unwrap();
            let imgs = document.select(&selector).map(|x| x.value());
            let mut item = 0;
            let mut img_list: Vec<String> = Vec::new();
            let mut img_arr: Vec<(String, String)> = Vec::new();
            for img in imgs {
                item += 1;
                match img.attr("data-cfsrc") {
                    Some(x) => {
                        println!("{}\t:\t{:?}", item, x);
                        let parsed = Url::parse(x).expect("parsed");
                        let fname = parsed
                                            .path_segments()
                                            .and_then(|segment| segment.last())
                                            .and_then(|name| if name.is_empty() { None } else { Some(name) })
                                            .unwrap_or("tmp.bin");
                        // download(x, &out).await;
                        img_arr.push((x.to_string(), "./src/files/".to_string() + fname));
                        img_list.push("./src/files/".to_string() + fname);
                    },
                    None => { println!("{}\t:\tNone", item) },
                }
            }
            let mut tasks:Vec<_> = Vec::new();
            for img in img_arr {
                tasks.push(tokio::spawn(async {
                    let (x, out) = img;
                    download(&x, &out).await;
                }))
            }
            for task in tasks {
                task.await.unwrap();
            }
            create_pdf(&img_list, &outfname).await;
            for img in img_list {
                fs::remove_file(img).unwrap();
            }
        // }));
    }
    // for task in tasks {
    //     task.await.unwrap();
    // }
}
