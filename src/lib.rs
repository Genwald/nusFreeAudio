#![feature(proc_macro_hygiene)]
use nus3audio::{Nus3audioFile, AudioFile};
use std::fs;
use std::collections::HashMap;
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use core::mem::size_of;
use arcropolis_api as arc_api;
#[macro_use]
extern crate lazy_static;

const ARC_FOLDER: &str = "rom:/nusFreeAudio/";
struct AudioFileInfo {
    name: String,
    size: usize,
    path: PathBuf,
}

lazy_static! {
    static ref FILE_MAP: Mutex<HashMap<u64, Vec<AudioFileInfo>>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };    
}

// Calculates the size as if all files are used. May not be the case due to duplicates.
// Based on a function from libnus3audio, modified to not require reading the files
fn calc_nus3_size(files: &[AudioFileInfo]) -> usize {
    fn get_padding_amount(offset: usize) -> usize {
        ((0x18 - (offset as isize % 0x10)) % 0x10) as usize
    }
    let nus3_size = "NUS3".len() + size_of::<u32>();
    let audi_size = "AUDIINDX".len() + (size_of::<u32>() * 2);
    let tnid_size = "TNID".len() + size_of::<u32>() + (size_of::<u32>() * files.len());
    let nmof_size = tnid_size;
    let adof_size = "ADOF".len() + size_of::<u32>() + (size_of::<u32>() * files.len() * 2);

    let string_section_start = nus3_size
        + audi_size
        + tnid_size
        + nmof_size
        + adof_size
        + "TNNM".len()
        + size_of::<u32>();

    let mut string_section_size = 0u32;
    for file in files.iter() {
        string_section_size += file.name.len() as u32 + 1;
    }

    let junk_pad = get_padding_amount(
        string_section_start + string_section_size as usize + "JUNK".len() + size_of::<u32>(),
    );
    let junk_size = "JUNK".len() + size_of::<u32>() + junk_pad;

    let pack_section_start = string_section_start
        + string_section_size as usize
        + junk_size
        + "PACK".len()
        + size_of::<u32>();

    let mut pack_section_size = 0u32;
    let mut pack_section_size_no_pad = 0u32;
    for file in files.iter() {
        pack_section_size_no_pad = pack_section_size + file.size as u32;
        pack_section_size += ((file.size + 0xF) / 0x10) as u32 * 0x10;
    }

    pack_section_start
        + if files.len() == 1 {
            pack_section_size_no_pad
        } else {
            pack_section_size
        } as usize
}

fn make_nus3audio(audio_list: Vec<AudioFile>) -> Vec<u8>{
    let nus3_file = Nus3audioFile{
        files: audio_list
    };
    let size = nus3_file.calc_size();
    let mut file_bytes: Vec<u8> = Vec::with_capacity(size);
    nus3_file.write(&mut file_bytes);
    file_bytes
}

#[arc_api::stream_callback]
fn nus3_stream_callback(hash: u64) -> Option<Vec<u8>> {
    let map = FILE_MAP.lock().unwrap();
    match map.get(&hash) {
        Some(info_vec) => {
            let mut audio_files = Vec::with_capacity(info_vec.len());
            for value in info_vec.iter().enumerate() {
                let (idx, info) = value;
                let file_data = fs::read(&info.path).unwrap();
                let audio = AudioFile {
                    id: idx as u32,
                    name: info.name.clone(),
                    data: file_data
                };
                audio_files.push(audio);
            }
            Some(make_nus3audio(audio_files))
        },
        None => {
            println!("No file matching the hash: {:#x}", hash);
            None
        }
    }
}

#[arc_api::arc_callback]
fn nus3_callback(hash: u64, data: &mut [u8]) -> Option<usize> {
    let map = FILE_MAP.lock().unwrap();
    match map.get(&hash) {
        Some(info_vec) => {
            let mut audio_files = Vec::with_capacity(info_vec.len());
            for value in info_vec.iter().enumerate() {
                let (idx, info) = value;
                let file_data = fs::read(&info.path).unwrap();
                let audio = AudioFile {
                    id: idx as u32,
                    name: info.name.clone(),
                    data: file_data
                };
                audio_files.push(audio);
            }
            let nus3data = make_nus3audio(audio_files);
            data[..nus3data.len()].copy_from_slice(&nus3data);
            Some(nus3data.len())
        },
        None => {
            println!("No file matching the hash: {:#x}", hash);
            None
        }
    }
}

fn get_infos(root: &Path) -> Vec<AudioFileInfo> {
    let mut vec = Vec::new();
    // Sorted so that users have an easy way to depend on file order.
    // Should this be done differently?
    for entry in walkdir::WalkDir::new(root)
                                .min_depth(1)
                                .max_depth(1)
                                .sort_by_file_name() {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            let name = match path.file_name() {
                Some(name) => name,
                None => continue
            };
            let size = entry.metadata().unwrap().len() as usize;
            
            let file_info = AudioFileInfo {
                name: name.to_string_lossy().to_string(),
                size,
                path,
            };
            vec.push(file_info);
        }
    }
    vec
}

fn entry_has_extension(entry: &walkdir::DirEntry, ext: &str) -> bool {
    match entry.path().extension() {
        Some(entry_ext) => entry_ext == ext,
        _ => false
    }
}

fn get_arc_path(path: &Path) -> String {
    path.strip_prefix(ARC_FOLDER).unwrap().to_string_lossy().replace(";", ":")
}

#[skyline::main(name = "nusFreeAudio")]
pub fn main() {
    let mut dir_it = walkdir::WalkDir::new(ARC_FOLDER)
                                .min_depth(1)
                                .into_iter();
    loop {
        let entry = match dir_it.next() {
            None => break,
            Some(entry) => entry.unwrap(),
        };
        if entry.file_type().is_dir() && entry_has_extension(&entry, "nus3audio") {
            dir_it.skip_current_dir();
            let path = entry.path();
            let file_infos = get_infos(path);
            let arc_path = get_arc_path(path);
            let path_hash = arc_api::hash40(&arc_path);
            if arc_path.starts_with("stream") {
                nus3_stream_callback::install(path_hash);
            }
            else {
                let size = calc_nus3_size(&file_infos);
                nus3_callback::install(path_hash, size);
            }
            
            FILE_MAP.lock().unwrap().insert(path_hash.as_u64(), file_infos);
        }
    }
}
