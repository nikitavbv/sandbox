use {
    std::path::Path,
    tracing::info,
};

fn convert_video_to_frames() {
    ffmpeg_next::init().unwrap();

    let file_path = Path::new("data/data-labeling/videos/video0.mp4");

    let mut input_ctx = ffmpeg_next::format::input(&file_path).unwrap();

    let video_stream_index = input_ctx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .unwrap()
        .index();

    // TODO: see https://github.com/zmwangx/rust-ffmpeg/blob/master/examples/dump-frames.rs
    let video_stream = input_ctx.streams().nth(video_stream_index).unwrap();

    for (stream, packet) in input_ctx.packets() {
        if stream.index() == video_stream_index {
            info!("packet");
            
            /*let decoded = packet.decode_video().unwrap();

            // Convert the decoded frame to an image
            let image = decoded.into_rgb8().unwrap();

            // Process the image here (e.g. save it to a file)
            // ...*/
        }
    }
}

pub fn run_data_labeling_tasks() {
    convert_video_to_frames();
}