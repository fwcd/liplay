extern crate ffmpeg_next as ffmpeg;

use std::{path::Path, time::Duration};

use anyhow::Result;
use ffmpeg::{format::Pixel, frame::Video};
use lighthouse_client::{protocol::{Color, Frame, LIGHTHOUSE_COLS, LIGHTHOUSE_ROWS}, Lighthouse, TokioWebSocket};
use tokio::time;
use tracing::info;

// Based on https://github.com/zmwangx/rust-ffmpeg/blob/a7b50dd5f/examples/dump-frames.rs

pub async fn run(path: &Path, mut lh: Lighthouse<TokioWebSocket>) -> Result<()> {
    ffmpeg::init()?;
    
    if let Ok(mut ictx) = ffmpeg::format::input(path) {
        let input = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_idx = input.index();

        let ctx_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = ctx_decoder.decoder().video()?;

        let mut scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            LIGHTHOUSE_COLS as u32,
            LIGHTHOUSE_ROWS as u32,
            ffmpeg::software::scaling::Flags::BILINEAR
        )?;

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_idx {
                decoder.send_packet(&packet)?;
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    let lh_frame = video_to_lh_frame(rgb_frame);
                    lh.put_model(lh_frame).await?;
                }
            }
        }

        loop {
            lh.put_model(Frame::fill(rand::random())).await?;
            info!("Sent frame");

            time::sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

fn video_to_lh_frame(video: Video) -> Frame {
    let bytes = video.data(0);
    let width = video.width() as usize;
    let height = video.height() as usize;

    let mut lh_frame = Frame::empty();
    for x in 0..width.min(LIGHTHOUSE_COLS) {
        for y in 0..height.min(LIGHTHOUSE_ROWS) {
            // TODO: Why do we need width + 4? What's going on?
            let i = (y * (width + 4) + x) * 3;
            let color = Color::new(bytes[i], bytes[i + 1], bytes[i + 2]);
            lh_frame.set(x, y, color);
        }
    }

    lh_frame
}
