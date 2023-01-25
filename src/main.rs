use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str;
use tide::{Body, Request, Response};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/").get(|req: Request<()>| async move {
        let mut f = File::open("Premium.Rush.2012.1080p.BluRay.x264.YIFY.mp4").unwrap();
        let file_size: usize = f.metadata().unwrap().len().try_into().unwrap();
        let range = req.header("range");
        match range {
            Some(val) => {
                let val = val.to_string();
                let val = val
                    .replace("bytes=", "")
                    .replace("[", "")
                    .replace("]", "")
                    .replace('"', "");
                let bytes_range: Vec<&str> = val.split("-").collect();
                let bytes_start = bytes_range[0].parse::<usize>().unwrap_or(0);
                let bytes_end = bytes_range[1].parse::<usize>().unwrap_or(0);
                let bytes_end = if bytes_end == 0 {
                    file_size - 1
                } else {
                    bytes_end
                };
                let bytes_end = if bytes_end > file_size {
                    file_size
                } else {
                    bytes_end
                };
                let seek = bytes_start;
                f.seek(SeekFrom::Start(seek.try_into().unwrap())).unwrap();
                let mut buf = vec![0; bytes_end - bytes_start + 1];
                f.read(&mut buf).unwrap();
                let response = Response::builder(206)
                    .body(Body::from_bytes(buf))
                    .header(
                        "Content-Range",
                        format!("bytes {}-{}/{}", bytes_start, bytes_end, file_size),
                    )
                    .header("Accept-Ranges", "bytes")
                    .header("Content-Type", "video/mp4")
                    .build();
                Ok(response)
            }
            None => {
                let response = Response::builder(200)
                    .header("Content-Type", "video/mp4")
                    .body(Body::from_file("Premium.Rush.2012.1080p.BluRay.x264.YIFY.mp4").await?)
                    .build();
                Ok(response)
            }
        }
    });
    app.listen("127.0.0.1:8081").await?;
    Ok(())
}
