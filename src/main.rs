extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;
use image::{GenericImage, GenericImageView};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ffmpeg::init().unwrap();

    let video_files = [
        "media/snip_1.mp4",
        "media/snip_1.mp4",
        "media/snip_1.mp4",
        "media/snip_1.mp4",
    ];
    let mut final_frame = None;
    let mut frame_number = 0;
    for (i, file) in video_files.iter().enumerate() {
        let mut ictx = input(&file)?;
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let video_stream_index = input.index();
        let mut decoder = context_decoder.decoder().video()?;
        let packets: Vec<_> = ictx.packets().collect();

        for (stream, packet) in packets {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    process_frame(&mut decoded, i, &mut final_frame, frame_number)?;
                    frame_number += 1;
                }
            }
        }
    }

    Ok(())
}

fn process_frame(
    frame: &mut Video,
    index: usize,
    final_frame: &mut Option<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>>,
    frame_number: usize,
) -> Result<(), Box<dyn Error>> {
    let mut scaler = Context::get(
        frame.format(),
        frame.width(),
        frame.height(),
        ffmpeg::format::Pixel::RGB24,
        frame.width(),
        frame.height(),
        Flags::BILINEAR,
    )?;

    let mut rgb_frame = Video::empty();
    scaler.run(frame, &mut rgb_frame)?;
    // println!(
    //     "Frame dimensions: {}x{}",
    //     rgb_frame.width(),
    //     rgb_frame.height()
    // );
    // println!("rgb_frame format: {:?}", rgb_frame.format());
    // println!("Data length: {}", rgb_frame.data(0).len());

    let img = match image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
        rgb_frame.width(),
        rgb_frame.height(),
        rgb_frame.data(0).to_vec(),
    ) {
        Some(img) => img,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to create image buffer",
            )))
        }
    };

    let (width, height) = img.dimensions();
    let quarter_width = width / 2;
    let quarter_height = height / 2;

    if final_frame.is_none() {
        *final_frame = Some(image::ImageBuffer::new(width, height));
    }

    let quarter = match index {
        0 => img.view(0, 0, quarter_width, quarter_height).to_image(),
        1 => img
            .view(quarter_width, 0, quarter_width, quarter_height)
            .to_image(),
        2 => img
            .view(0, quarter_height, quarter_width, quarter_height)
            .to_image(),
        _ => img
            .view(quarter_width, quarter_height, quarter_width, quarter_height)
            .to_image(),
    };
    if let Some(ref mut final_img) = final_frame {
        final_img.copy_from(
            &quarter,
            (index as u32 % 2) * quarter_width,
            (index as u32 / 2) * quarter_height,
        )?;
    }
    final_frame
        .clone()
        .unwrap()
        .save(format!("media/pngs/frame{}.png", frame_number))?;

    Ok(())
}
