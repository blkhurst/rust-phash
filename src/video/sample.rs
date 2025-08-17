use ffmpeg_next as ffmpeg;

pub fn get_total_frames(stream: &ffmpeg::format::stream::Stream) -> Option<usize> {
    // Exact count
    let nb = stream.frames();
    if nb > 0 {
        return Some(nb as usize);
    }

    // Estiamte = duration * avg_fps
    let duration_tb = stream.duration();
    if duration_tb <= 0 {
        return None;
    }

    let tb = stream.time_base();
    let fps = stream.avg_frame_rate();
    if tb.denominator() == 0 || fps.denominator() == 0 {
        return None;
    }

    let seconds = (duration_tb as f64) * (tb.numerator() as f64) / (tb.denominator() as f64);
    let fps_f = (fps.numerator() as f64) / (fps.denominator() as f64);
    if !seconds.is_finite() || !fps_f.is_finite() || fps_f <= 0.0 {
        return None;
    }

    let est = (seconds * fps_f).round();
    if est.is_sign_positive() {
        Some(est as usize)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SamplingPlan {
    pub start: usize,
    pub step: usize,
    pub take: usize, // How many samples we can take within the window (<= sample_count)
}

// Generate `step` to distribute samples evenly within `sample_window`
pub fn plan_even_sampling(
    total_frames: usize,
    sample_count: usize,
    sample_start: usize,
    sample_window: usize,
) -> SamplingPlan {
    if total_frames == 0 || sample_count == 0 {
        return SamplingPlan {
            start: 0,
            step: 1,
            take: 0,
        };
    }

    // If start > total, set start half-way, reducing intro/outro bias.
    let start = if sample_start > total_frames {
        total_frames / 2
    } else {
        sample_start.min(total_frames - 1)
    };

    // Frames available from start to end.
    let available = total_frames - start;

    // Window length (0 auto); shrink if it would overflow the end.
    let window_len = if sample_window == 0 {
        available
    } else {
        available.min(sample_window)
    };

    // Integer stride approximating even spacing across the window.
    let step = if sample_count <= 1 {
        1
    } else {
        ((window_len) / (sample_count)).max(1)
    };

    // How many we can actually take with that step inside the window.
    let take = (window_len / step).min(sample_count);

    SamplingPlan { start, step, take }
}
