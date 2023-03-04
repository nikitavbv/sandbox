use {
    std::{path::Path, fs::{self, File, create_dir_all}, io::Write, collections::HashMap},
    tracing::info,
    img_hash::{HasherConfig, Hasher, image::{Rgb, ImageBuffer, DynamicImage}},
    ffmpeg_next::{
        media::Type,
        software::scaling::{context::Context, flag::Flags},
        format::Pixel,
        util::frame::Video,
    },
};

fn convert_video_to_frames() {
    ffmpeg_next::init().unwrap();

    let video_index = 0;

    let file_path = format!("data/data-labeling/videos/video{}.mp4", video_index);
    let file_path = Path::new(&file_path);

    let mut ictx = ffmpeg_next::format::input(&file_path).unwrap();

    let input = ictx
        .streams()
        .best(Type::Video)
        .unwrap();
    let video_stream_index = input.index();

    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters()).unwrap();
    let mut decoder = context_decoder.decoder().video().unwrap();
    
    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    ).unwrap();

    let mut frame_index = 0;

    let hasher = HasherConfig::new().to_hasher();

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame).unwrap();
                save_frame(&hasher, &rgb_frame, video_index, frame_index);
                frame_index += 1;
            }

            info!("received frame {}", frame_index);
            frame_index += 1;

            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).unwrap();
            receive_and_process_decoded_frames(&mut decoder).unwrap();
        }
    }
    decoder.send_eof().unwrap();
    receive_and_process_decoded_frames(&mut decoder).unwrap();
}

fn save_frame(hasher: &Hasher, frame: &Video, video_index: usize, frame_index: usize) {
    /*let file_path = format!("data/data-labeling/frames/video{}/{}.png", video_index, frame_index);
    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_dir_all(parent).unwrap();
        }
    }*/

    let img_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(frame.width(), frame.height(), frame.data(0).to_vec()).unwrap();
    let image = DynamicImage::ImageRgb8(img_buf);
    
    let hash = hasher.hash_image(&image).to_base64();
    info!("hash: {}", hash);

    // img_buf.save(path).unwrap();
}

fn read_frame_hashes_for_video(video_index: usize) -> HashMap<String, usize> {
    let file_path = format!("data/data-labeling/frames/video{}/hashes.json", video_index);
    let path = Path::new(&file_path);
    if !path.exists() {
        return HashMap::new();
    }

    serde_json::from_reader(File::open(path).unwrap()).unwrap()
}

fn write_frame_hashes_for_video(video_index: usize, hashes: &HashMap<String, usize>) {
    let file_path = format!("data/data-labeling/frames/video{}/hashes.json", video_index);
    let path = Path::new(&file_path);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent).unwrap();
            }
        }

        File::create(path).unwrap();
    }

    fs::write(path, serde_json::to_vec(hashes).unwrap()).unwrap()
}

pub fn run_data_labeling_tasks() {
    convert_video_to_frames();
}