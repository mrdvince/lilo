extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::input;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ffmpeg::init().unwrap();

    let video_files = vec![
        "media/snip_1.mp4",
        "media/snip_2.mp4",
        "media/snip_3.mp4",
        "media/snip_4.mp4",
    ];

    let mut frames = Vec::new();

    for file in &video_files {
        let mut ictx = input(&file)?;
        let video_stream = ictx.streams().best(ffmpeg::media::Type::Video).unwrap();
        let video_decoder =
            ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;

    }
    Ok(())
}
