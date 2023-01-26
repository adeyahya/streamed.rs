use rust_stream::BytesRange;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use tide::{Body, Error, Request, Response};
use urlencoding::decode;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/movies").get(list_dir);
    app.at("/movies/:title").get(handle_stream);
    app.at("/movies/:title/translation").get(serve_translation);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn list_dir(_: Request<()>) -> Result<Body, Error> {
    let dirs: Vec<PathBuf> = fs::read_dir("./MOVIES")?
        .map(|dir| dir.unwrap().path().to_owned())
        .filter(|dir| dir.is_dir())
        .collect();
    let dirs: Vec<String> = dirs
        .iter()
        .map(|dir| String::from(dir.file_name().unwrap().to_str().unwrap()))
        .collect();
    Body::from_json(&dirs)
}

fn find_mp4_file(path: &str) -> String {
    let target = fs::read_dir(format!("./MOVIES/{}", path))
        .unwrap()
        .map(|dir| dir.unwrap().path().to_owned())
        .find(|f| f.extension().unwrap().to_str().unwrap() == "mp4");
    String::from(target.unwrap().to_str().unwrap())
}

fn find_srt_file(path: &str) -> String {
    let target = fs::read_dir(format!("./MOVIES/{}", path))
        .unwrap()
        .map(|dir| dir.unwrap().path().to_owned())
        .find(|f| f.extension().unwrap().to_str().unwrap() == "srt");
    match target {
        Some(val) => String::from(val.to_str().unwrap()),
        None => String::from("./dummy.txt"),
    }
}

async fn serve_translation(req: Request<()>) -> tide::Result<Response> {
    let title = req.param("title").unwrap();
    let title = decode(title).expect("UTF-8");
    let title = find_srt_file(&title);
    let response = Response::builder(200)
        .body(Body::from_file(&title).await?)
        .build();
    Ok(response)
}

async fn handle_stream(req: Request<()>) -> tide::Result<Response> {
    let title = req.param("title").unwrap();
    let title = decode(title).expect("UTF-8");
    let title = find_mp4_file(&title);
    let mut f = File::open(&title)?;
    let file_size: usize = f.metadata()?.len().try_into()?;
    let range = req.header("range");
    match range {
        Some(val) => {
            let val = val.to_string();
            let bytes_range = BytesRange::parse(&val, file_size);
            let seek = bytes_range.start;
            f.seek(SeekFrom::Start(seek.try_into()?))?;
            let mut buf = vec![0; bytes_range.end - bytes_range.start + 1];
            f.read(&mut buf)?;
            let response = Response::builder(206)
                .body(Body::from_bytes(buf))
                .header(
                    "Content-Range",
                    format!(
                        "bytes {}-{}/{}",
                        bytes_range.start, bytes_range.end, file_size
                    ),
                )
                .header("Accept-Ranges", "bytes")
                .header("Content-Type", "video/mp4")
                .build();
            Ok(response)
        }
        None => {
            let response = Response::builder(200)
                .header("Content-Type", "video/mp4")
                .body(Body::from_file(&title).await?)
                .build();
            Ok(response)
        }
    }
}
