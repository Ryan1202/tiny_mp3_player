use std::{fs::File, io::{BufReader, Read}};

use frames::ID3v2Frame;

pub mod frames;

pub struct Id3v2 {
    pub major_version: u8,
    pub revision: u8,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub comment: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
}

fn get_id3_size(header: &[u8; 10]) -> usize {
    if header[0..3] == [b'I', b'D', b'3'] {
        (header[6] as usize & 0x7f) << 21
            | (header[7] as usize & 0x7f) << 14
            | (header[8] as usize & 0x7f) << 7
            | (header[9] as usize & 0x7f) + 10
    } else {
        0
    }
}

impl Id3v2 {
    pub fn new(reader: &mut BufReader<File>) -> Option<Id3v2> {
        let mut header = [0; 10];
        reader.read_exact(&mut header).unwrap();
        let size = get_id3_size(&header);

        if size == 0 {
            return None;
        }
        let major_version = header[3];
        let revision = header[4];

        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut year = None;
        let mut comment = None;
        let mut genre = None;
        let mut track_number = None;

        let mut count = 0;
        while count < (size - 10) {
            reader.read_exact(&mut header).unwrap();

            let frame_size = (header[4] as usize & 0x7f) << 21
                | (header[5] as usize & 0x7f) << 14
                | (header[6] as usize & 0x7f) << 7
                | (header[7] as usize & 0x7f);
            let mut buffer = vec![0; frame_size as usize];
            reader.read_exact(&mut buffer).unwrap();

            let frame = ID3v2Frame::new(header[0..4].try_into().unwrap(), &buffer, revision);
            count += frame_size as usize + 10;

            match frame {
                ID3v2Frame::Title(str) => {
                    title = Some(str);
                }
                ID3v2Frame::Artist(str) => {
                    artist = Some(str);
                }
                ID3v2Frame::Album(str) => {
                    album = Some(str);
                }
                ID3v2Frame::Year(str) => {
                    year = Some(str);
                }
                ID3v2Frame::Comment(str) => {
                    comment = Some(str);
                }
                ID3v2Frame::Genre(str) => {
                    genre = Some(str);
                }
                ID3v2Frame::TrackNumber(str) => {
                    track_number = Some(str);
                }
                _ => {}
            }
        }
        Some(Self {
            major_version: major_version,
            revision: revision,
            title: title,
            artist: artist,
            album: album,
            year: year,
            comment: comment,
            genre: genre,
            track_number: track_number,
        })
    }
}
