use std::str;

pub struct BytesRange {
    pub start: usize,
    pub end: usize,
}

impl BytesRange {
    pub fn parse(value: &str, file_size: usize) -> BytesRange {
        let val = value
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

        BytesRange {
            start: bytes_start,
            end: bytes_end,
        }
    }
}
