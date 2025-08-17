use std::path::Path;

use ffmpeg::{codec, format, frame, media, software::scaling, util::format::pixel};
use ffmpeg_next as ffmpeg;

use img_hash::image::{DynamicImage, ImageBuffer, Rgb};
use img_hash::{Hasher, ImageHash};

use crate::errors::{AppError, VideoError};
use crate::video::sample;

#[inline]
pub fn init_ffmpeg() -> Result<(), AppError> {
    ffmpeg::init().map_err(|e| VideoError::Ffmpeg(e.to_string()))?;
    // Only log errors, not warnings.
    ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Error);
    Ok(())
}

fn open_input_and_decoder(
    path: &Path,
) -> Result<(format::context::Input, usize, codec::decoder::Video), VideoError> {
    let ictx = format::input(path)
        .map_err(|e| VideoError::Decode(format!("open {}: {e}", path.display())))?;
    let stream = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or_else(|| VideoError::Decode("no video stream found".into()))?;
    let stream_index = stream.index();

    let ctx = codec::Context::from_parameters(stream.parameters())
        .map_err(|e| VideoError::Decode(format!("decoder ctx: {e}")))?;
    let decoder = ctx
        .decoder()
        .video()
        .map_err(|e| VideoError::Decode(format!("open decoder: {e}")))?;

    Ok((ictx, stream_index, decoder))
}

/// TODO: Downscale to increase performance + FAST_BILINEAR.
fn build_rgb_scaler(dec: &codec::decoder::Video) -> Result<scaling::Context, VideoError> {
    let (w, h) = (dec.width(), dec.height());
    let src_fmt = dec.format();
    scaling::Context::get(
        src_fmt,
        w,
        h,
        pixel::Pixel::RGB24,
        w,
        h,
        scaling::Flags::BILINEAR,
    )
    .map_err(|e| VideoError::Decode(format!("sws ctx: {e}")))
}

/// Copy a packed RGB24 frame (with stride) into an owned `DynamicImage`
fn copy_rgb_to_image(rgb: &frame::Video) -> Result<DynamicImage, AppError> {
    let (w, h) = (rgb.width(), rgb.height());
    let data = rgb.data(0);
    let stride = rgb.stride(0) as usize;
    let mut owned = vec![0u8; (w as usize) * (h as usize) * 3];
    for y in 0..(h as usize) {
        let src = &data[y * stride..y * stride + (w as usize) * 3];
        let dst = &mut owned[y * (w as usize) * 3..(y + 1) * (w as usize) * 3];
        dst.copy_from_slice(src);
    }
    let buf: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(w, h, owned)
        .ok_or_else(|| AppError::Video(VideoError::Decode("image buffer alloc failed".into())))?;
    Ok(DynamicImage::ImageRgb8(buf))
}

/// Decode, Sample `sample_count` frames, evenly spaced over a window up to `sample_window`
pub fn decode_sample_even_window_hash(
    path: &Path,
    sample_start: usize,
    sample_count: usize,
    sample_window: usize,
    hasher: &Hasher,
) -> Result<Vec<ImageHash>, AppError> {
    if sample_count == 0 {
        return Ok(Vec::new());
    }

    let (mut ictx, sidx, mut dec) = open_input_and_decoder(path)?;
    let stream = ictx
        .stream(sidx)
        .ok_or_else(|| VideoError::Decode("stream index out of range".into()))?;

    let total = sample::get_total_frames(&stream).unwrap_or(0);
    let sample_plan = sample::plan_even_sampling(total, sample_count, sample_start, sample_window);
    // Debug
    // eprintln!(
    //     "Start({:?}) Step({:?}) Take({:?}) Total({:?}) File({:?})",
    //     sample_plan.start, sample_plan.step, sample_plan.take, total, path
    // );

    let mut scaler = build_rgb_scaler(&dec)?;
    let mut decoded = frame::Video::empty();
    let mut rgb = frame::Video::empty();

    let mut out: Vec<ImageHash> = Vec::with_capacity(sample_count);
    let mut idx = 0usize;
    let mut remaining = sample_plan.take;

    #[inline]
    fn should_take(idx: usize, start: usize, step: usize, remaining: usize) -> bool {
        remaining > 0 && idx >= start && ((idx - start) % step == 0)
    }

    // Read packets & decode frames.
    'packets: for (st, pkt) in ictx.packets() {
        if st.index() != sidx {
            continue;
        }
        if dec.send_packet(&pkt).is_err() {
            continue;
        }

        while dec.receive_frame(&mut decoded).is_ok() {
            if should_take(idx, sample_plan.start, sample_plan.step, remaining) {
                scaler
                    .run(&decoded, &mut rgb)
                    .map_err(|e| VideoError::Decode(format!("sws run: {e}")))?;
                let img = copy_rgb_to_image(&rgb)?;
                out.push(hasher.hash_image(&img));
                remaining -= 1;
                if remaining <= 0 {
                    break 'packets;
                }
            }
            idx += 1;
        }
    }

    if out.is_empty() {
        return Err(AppError::Video(VideoError::NoSamples));
    }

    if out.len() < sample_count {
        eprintln!(
            "note: sampled {} of requested {} ({} frames available in window) for {}",
            out.len(),
            sample_count,
            sample_plan.take,
            path.display()
        );
    }

    Ok(out)
}
