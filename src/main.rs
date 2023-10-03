extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ffmpeg::init().unwrap();

    let video_files = [
        "media/snip_1.mp4",
        "media/snip_2.mp4",
        "media/snip_3.mp4",
        "media/snip_4.mp4",
    ];
    let mut final_frame = None;

    for (i, file) in video_files.iter().enumerate() {
        let mut ictx = input(&file)?;
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let video_stream_index = input.index();
        let mut decoder = context_decoder.decoder().video()?;
        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg::format::Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;
        let packets: Vec<_> = ictx.packets().collect();

        for (stream, packet) in packets {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    process_frame(&mut rgb_frame, i, &mut final_frame)?;
                    break;
                }
            }
        }
    }
    if let Some(frame) = final_frame {
        frame.save("final_frame.png")?;
    }
    Ok(())
}

fn process_frame(
    frame: &mut Video,
    index: usize,
    final_frame: &mut Option<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>>,
) -> Result<(), Box<dyn Error>> {
    let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
        frame.width(),
        frame.height(),
        frame.data(0).to_vec(),
    )
    .unwrap();

    Ok(())
}
