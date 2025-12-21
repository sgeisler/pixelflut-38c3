use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::OnceLock;

use ffmpeg::{format, media, software::scaling, util::frame};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::Dictionary;

use super::Frame;
use super::RGB;

const CHANNEL_SIZE: usize = 8;

pub fn open_stream(
    path: String,
    scale_factor: f64,
    frame_size: Arc<OnceLock<(usize, usize)>>,
) -> Result<Receiver<Frame>, ffmpeg::Error> {
    let (sender, receiver) = mpsc::sync_channel(CHANNEL_SIZE);

    std::thread::spawn(move || {
        send_frames(sender, path, scale_factor, frame_size).expect("sender died")
    });

    return Ok(receiver);
}

pub fn send_frames(
    sender: SyncSender<Frame>,
    path: String,
    scale_factor: f64,
    frame_size: Arc<OnceLock<(usize, usize)>>,
) -> Result<(), ffmpeg::Error> {
    ffmpeg::init()?;

    let mut options = Dictionary::new();

    options.set("vcodec", "h264_omx");

    let mut ictx = format::input_with_dictionary(&path, options)?;
    let input = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let framerate: f64 = input.avg_frame_rate().into();
    let frame_duration: f64 = 1.0 / framerate;

    let video_stream_index = input.index();
    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().video()?;

    let width = decoder.width();
    let height = decoder.height();

    let scaled_width = ((width as f64) * scale_factor) as u32;
    let scaled_height = ((height as f64) * scale_factor) as u32;

    frame_size
        .set((scaled_width as usize, scaled_height as usize))
        .expect("setting frame size failed");

    let mut scaler = scaling::Context::get(
        decoder.format(),
        width,
        height,
        format::Pixel::RGB24,
        scaled_width,
        scaled_height,
        scaling::Flags::BILINEAR,
    )?;

    let mut receive_and_process =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = frame::Video::empty();

                scaler.run(&decoded, &mut rgb_frame)?;

                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0);

                let img_data: Vec<Vec<RGB>> = (0..scaled_height as usize)
                    .map(|y| {
                        let row_start = y * stride;
                        (0..scaled_width as usize)
                            .map(|x| {
                                let i = row_start + x * 3;
                                RGB::new(data[i], data[i + 1], data[i + 2])
                            })
                            .collect()
                    })
                    .collect();
                let frame = Frame {
                    img: img_data,
                    frame_duration: frame_duration,
                };
                sender.send(frame).expect("sender couldn't send");
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() != video_stream_index {
            continue;
        }
        decoder.send_packet(&packet)?;
        receive_and_process(&mut decoder)?;
    }

    decoder.send_eof()?;
    receive_and_process(&mut decoder)?;

    Ok(())
}

#[allow(unused)]
pub fn extract_frames(path: &str, scale_factor: f64) -> Result<Vec<Vec<Vec<RGB>>>, ffmpeg::Error> {
    ffmpeg::init()?;

    let mut ictx = format::input(path)?;
    let input = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let video_stream_index = input.index();
    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().video()?;

    let width = decoder.width();
    let height = decoder.height();

    let scaled_width = ((width as f64) * scale_factor) as u32;
    let scaled_height = ((height as f64) * scale_factor) as u32;

    let mut scaler = scaling::Context::get(
        decoder.format(),
        width,
        height,
        format::Pixel::RGB24,
        scaled_width,
        scaled_height,
        scaling::Flags::BICUBIC,
    )?;

    let mut frames = Vec::new();

    let mut receive_and_process =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = frame::Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;

                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0);

                let frame: Vec<Vec<RGB>> = (0..scaled_height as usize)
                    .map(|y| {
                        let row_start = y * stride;
                        (0..scaled_width as usize)
                            .map(|x| {
                                let i = row_start + x * 3;
                                RGB::new(data[i], data[i + 1], data[i + 2])
                            })
                            .collect()
                    })
                    .collect();
                frames.push(frame);
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() != video_stream_index {
            continue;
        }
        decoder.send_packet(&packet)?;
        receive_and_process(&mut decoder)?;
    }

    decoder.send_eof()?;
    receive_and_process(&mut decoder)?;

    Ok(frames)
}
