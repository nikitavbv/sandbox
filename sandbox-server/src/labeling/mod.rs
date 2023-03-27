use {
    std::{path::Path, fs::{self, File, create_dir_all}, io::Write, collections::HashMap},
    tracing::{info, warn},
    img_hash::{ImageHash, HasherConfig, Hasher, image::{Rgb, ImageBuffer, DynamicImage}},
    config::Config,
    ffmpeg_next::{
        media::Type,
        software::scaling::{context::Context, flag::Flags},
        format::Pixel,
        util::frame::Video,
    },
};

mod video_hashes;

fn convert_video_to_frames(config: &Config) {
    ffmpeg_next::init().unwrap();

    let input_dir = "./data/data-labeling/videos";
    let paths = fs::read_dir(input_dir).unwrap();
    let mut video_counter = 0;
    let mut new_videos = Vec::new();

    for path in paths {
        let path = path.unwrap();
        let name = path.file_name();
        let name = name.to_string_lossy().to_lowercase();
        
        if name.starts_with("video") && name.ends_with(".mp4") {
            let parse_result = name.replace("video", "").replace(".mp4", "").parse::<usize>();
            if let Ok(parse_result) = parse_result {
                video_counter = video_counter.max(parse_result);
                continue;
            }
        }

        if name == ".ds_store" || name == "metadata.md" {
            continue;
        }

        new_videos.push(name);
    }

    info!("videos to convert: {}", video_counter);
    for new_video in &new_videos {
        video_counter += 1;

        if new_video.to_lowercase().ends_with(".mov") {
            panic!("got mov file, convert it using:\nffmpeg -i {} -c:v libx264 -preset slow -crf 22 -c:a copy converted.mp4", new_video);
        } else if !new_video.to_lowercase().ends_with(".mp4") {
            panic!("can only process mp4 files, got: {}", new_video);
        }

        let prev_name = format!("{}/{}", input_dir, new_video);
        let new_name = format!("{}/video{}.mp4", input_dir, video_counter);
        info!("moving {} to {}", prev_name, new_name);
        fs::rename(prev_name, new_name).unwrap();
    }

    for video in 0..(video_counter + 1) {
        if !are_frame_hashes_already_computed(video) {
            compute_hashes_for_video(video);
        }
    }

    let new_video = config.get("data_labeling.video").unwrap_or(135);

    for video in 0..new_video {
        let similarity = compare_videos(new_video, video);

        if similarity > 0.2 {
            warn!("similarity with {} is {}", video, similarity);
        } else {
            info!("video {} is different", video);
        }
    }
}

fn compare_videos(video_index_a: usize, video_index_b: usize) -> f64 {
    let hashes_a = read_frame_hashes_for_video(video_index_a);
    let hashes_b = read_frame_hashes_for_video(video_index_b);

    let frames_a: usize = hashes_a.values().sum();

    let mut result = 0;

    for (hash_a, hash_a_cnt) in &hashes_a {
        let hash_a: ImageHash<Vec<u8>> = ImageHash::from_base64(hash_a).unwrap();
        let hash_a_len = hash_a.as_bytes().len();

        let mut present = false;

        for (hash_b, _) in &hashes_b {
            let hash_b: ImageHash<Vec<u8>> = ImageHash::from_base64(hash_b).unwrap();
            let hash_b_len = hash_b.as_bytes().len();

            let similarity = 1.0 - (hash_a.dist(&hash_b) as f64 / ((hash_a_len.max(hash_b_len) as f64) * 8.0));

            if similarity > 0.8 {
                present = true;
                break;
            }
        }

        if present {
            result += hash_a_cnt;
        }
    }

    result as f64 / (frames_a as f64)
}

fn compute_hashes_for_video(video_index: usize) {
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
    let mut hashes = HashMap::new();

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg_next::decoder::Video, hashes: &mut HashMap<String, usize>| -> Result<(), ffmpeg_next::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame).unwrap();
                save_frame(&hasher, &rgb_frame, video_index, frame_index, hashes);
                frame_index += 1;
            }

            info!("received frame {}", frame_index);
            frame_index += 1;

            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).unwrap();
            receive_and_process_decoded_frames(&mut decoder, &mut hashes).unwrap();
        }
    }
    decoder.send_eof().unwrap();
    receive_and_process_decoded_frames(&mut decoder, &mut hashes).unwrap();

    write_frame_hashes_for_video(video_index, &hashes);
}

fn save_frame(hasher: &Hasher, frame: &Video, video_index: usize, frame_index: usize, hashes: &mut HashMap<String, usize>) {
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

    hashes.insert(hash.clone(), hashes.get(&hash).unwrap_or(&0) + 1);

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

fn are_frame_hashes_already_computed(video_index: usize) -> bool {
    let file_path = format!("data/data-labeling/frames/video{}/hashes.json", video_index);
    let path = Path::new(&file_path);
    path.exists()
}

pub fn run_data_labeling_tasks(config: &Config) {
    convert_video_to_frames(config);
}