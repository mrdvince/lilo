extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;
use image::{GenericImage, GenericImageView};
use std::error::Error;

struct DecoderContext {
    ictx: ffmpeg::format::context::Input,
    decoder: ffmpeg::decoder::Video,
    video_stream_index: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    ffmpeg::init().unwrap();

    let video_files = [
        "media/snip_1.mp4",
        "media/snip_1.mp4",
        "media/snip_1.mp4",
        "media/snip_1.mp4",
    ];

    let mut decoders: Vec<DecoderContext> = Vec::new();

    for file in &video_files {
        let ictx = input(&file)?;
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let video_stream_index = input.index();
        let decoder = context_decoder.decoder().video()?;

        decoders.push(DecoderContext {
            ictx,
            decoder,
            video_stream_index,
        });
    }

    let mut final_frame = None;
    let mut frame_number = 1;

    loop {
        let mut all_packets_empty = true;
        let mut all_frames_processed = true;

        for (i, decoder_ctx) in decoders.iter_mut().enumerate() {
            while let Some((stream, packet)) = decoder_ctx.ictx.packets().next() {
                all_packets_empty = false;
                if stream.index() == decoder_ctx.video_stream_index {
                    decoder_ctx.decoder.send_packet(&packet)?;
                    loop {
                        let mut decoded = Video::empty();
                        match decoder_ctx.decoder.receive_frame(&mut decoded) {
                            Ok(_) => {
                                all_frames_processed = false;
                                process_frame(&mut decoded, i, &mut final_frame, frame_number)?;
                                frame_number += 1; // Increment frame_number for each frame processed
                            }
                            Err(ref e)
                                if e.to_string().contains("Resource temporarily unavailable")
                                    || e.to_string().contains("35") =>
                            {
                                break; // Need more data, break and send next packet
                            }
                            Err(e) => {
                                return Err(Box::new(e)); // Some other error occurred
                            }
                        }
                    }
                }
            }
        }

        if all_packets_empty && all_frames_processed {
            break; // Exit the loop if all packets are processed and all frames are processed
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

    // Save the entire frame to disk
    img.save(format!(
        "media/individual_frames/full_frame_{}_{}.png",
        index, frame_number
    ))?;

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

    // Save the quarter frame to disk
    quarter.save(format!(
        "media/individual_frames/quarter_frame_{}_{}.png",
        index, frame_number
    ))?;

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
