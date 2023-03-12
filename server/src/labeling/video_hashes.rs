use {
    std::{path::Path, collections::HashMap, fs::{self, File, create_dir_all}},
    tracing::info,
    img_hash::{HasherConfig, Hasher, image::{Rgb, ImageBuffer, DynamicImage}},
    ffmpeg_next::{
        media::Type,
        software::scaling::{context::Context, flag::Flags},
        format::Pixel,
        util::frame::Video,
    },
    crate::models::{io::ModelData, Model},
};

pub struct VideoHashesCompute {
}

impl VideoHashesCompute {
    fn new() -> Self {
        Self {
        }
    }
}

impl Model for VideoHashesCompute {
    fn run(&self, input: &ModelData) -> ModelData {
        let input_path = input.get_text("input_path");
        let video_index = input.get_u32("video_index");

        let file_path = Path::new(&input_path);

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
                    save_frame(&hasher, &rgb_frame, video_index as usize, frame_index, hashes);
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
    
        write_frame_hashes_for_video(video_index as usize, &hashes);

        ModelData::new()
    }
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
