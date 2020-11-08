use nus3audio::Nus3audioFile;
use smash_arc::{ArcFile, ArcLookup, hash40};
use binread::{BinReaderExt, BinRead};
use bgm_property::{BgmPropertyFile, Entry};

use std::io::Cursor;

#[derive(BinRead)]
#[br(magic = b"OPUS")]
struct OpusHeader {
    unk: u32,
    total_samples: u32,
    channel_count: u32,
    sample_rate: u32,
    loop_start: u32,
    loop_end: u32
}

fn main() {
    let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();

    let bgm_files = arc.get_stream_listing("stream:/sound/bgm").unwrap();


    let mut bgm_prop_entries = Vec::with_capacity(bgm_files.len());

    for file in bgm_files {
        let data = arc.get_stream_file_contents(file.hash40()).unwrap();

        let nus3audio = if &data[..4] == &b"NUS3"[..] {
            match Nus3audioFile::try_from_bytes(&data) {
                Some(file) => file,
                None => continue
            }
        } else {
            continue
        };
    
        let mut cursor = Cursor::new(&nus3audio.files[0].data);
        let opus_header: OpusHeader = cursor.read_be().unwrap();

        let sample_rate = opus_header.sample_rate as f64;
        let loop_start_ms = (((opus_header.loop_start as f64) * 1000.) / sample_rate).round() as u32;
        let loop_end_ms = (((opus_header.loop_end as f64) * 1000.) / sample_rate).round() as u32;
        let total_time_ms = (((opus_header.total_samples as f64) * 1000.) / sample_rate).round() as u32;

        bgm_prop_entries.push(Entry {
            name_id: hash40(&nus3audio.files[0].name).as_u64(),
            loop_start_sample: opus_header.loop_start,
            loop_end_sample: opus_header.loop_end,
            total_samples: opus_header.total_samples,
            total_time_ms,
            loop_start_ms,
            loop_end_ms,
        });
    }

    BgmPropertyFile(bgm_prop_entries).save("bgm_property.bin").unwrap();
}
