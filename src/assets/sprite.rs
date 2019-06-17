use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::types::{BoundingBox, Dimensions, Point, Version};
use crate::util::{bgra2rgba, rgba2bgra};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 800;
pub const VERSION_COLLISION: Version = 800;
pub const VERSION_FRAME: Version = 800;

pub struct CollisionMap {
    pub bounds: BoundingBox,
    pub data: Box<[u8]>,
}

pub struct Sprite {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The origin within the sprite.
    pub origin: Point,

    /// The raw RGBA pixeldata for each frame.
    pub frames: Vec<(Dimensions, Box<[u8]>)>,

    /// The collider associated with one or each frame.
    /// If `per_frame_colliders` is false, this contains 1 map.
    pub colliders: Vec<CollisionMap>,

    /// Whether each individual frame has its own collision map.
    pub per_frame_colliders: bool,
}

impl Sprite {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION as u32)?;
        result += writer.write_i32_le(self.origin.x)?;
        result += writer.write_i32_le(self.origin.y)?;
        if self.frames.len() != 0 {
            result += writer.write_u32_le(self.frames.len() as u32)?;
            for frame in self.frames.iter() {
                result += writer.write_u32_le(VERSION_FRAME)?;
                result += writer.write_u32_le(frame.0.width)?;
                result += writer.write_u32_le(frame.0.height)?;
                result += writer.write_u32_le(frame.1.len() as u32)?;

                let mut pixeldata = frame.1.clone();
                rgba2bgra(&mut pixeldata);
                result += writer.write(&pixeldata)?;

                result += writer.write_u32_le(if self.per_frame_colliders { 1 } else { 0 })?;
                for collider in self.colliders.iter() {
                    result += writer.write_u32_le(VERSION_COLLISION)?;
                    result += writer.write_u32_le(collider.bounds.width)?;
                    result += writer.write_u32_le(collider.bounds.height)?;
                    result += writer.write_u32_le(collider.bounds.left)?;
                    result += writer.write_u32_le(collider.bounds.right)?;
                    result += writer.write_u32_le(collider.bounds.bottom)?;
                    result += writer.write_u32_le(collider.bounds.top)?;
                    result += writer.write(&collider.data)?;
                    for pixel in collider.data.iter() {
                        result += writer.write_u32_le(*pixel as u32)?;
                    }
                }
            }
        } else {
            result += writer.write_u32_le(0)?;
        }

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Sprite>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()? as Version;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let x = reader.read_i32_le()?;
        let y = reader.read_i32_le()?;
        let frame_count = reader.read_u32_le()?;
        let (frames, colliders, per_frame_colliders) = if frame_count != 0 {
            let mut frames = Vec::with_capacity(frame_count as usize);
            for _ in 0..frame_count {
                if options.strict {
                    let version = reader.read_u32_le()? as Version;
                    assert_eq!(version, VERSION_FRAME);
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let frame_width = reader.read_u32_le()?;
                let frame_height = reader.read_u32_le()?;

                let pixeldata_len = reader.read_u32_le()? as usize;
                let pixeldata_pixels = frame_width as usize * frame_height as usize;

                // sanity check
                if pixeldata_len != (pixeldata_pixels * 4) {
                    panic!("Inconsistent pixel data length with dimensions");
                }

                // BGRA -> RGBA
                let pos = reader.position() as usize;
                reader.seek(SeekFrom::Current(pixeldata_len as i64))?;
                let mut buf = reader.get_ref()[pos..pos + pixeldata_len].to_vec();
                bgra2rgba(&mut buf);

                // RMakeImage lol
                frames.push((
                    Dimensions {
                        width: frame_width,
                        height: frame_height,
                    },
                    buf.into_boxed_slice(),
                ));
            }

            fn read_collision<T>(reader: &mut io::Cursor<T>, options: &ParserOptions) -> io::Result<CollisionMap>
            where
                T: AsRef<[u8]>,
            {
                if options.strict {
                    let version = reader.read_u32_le()? as Version;
                    assert_eq!(version, VERSION_COLLISION);
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let width = reader.read_u32_le()?;
                let height = reader.read_u32_le()?;
                let left = reader.read_u32_le()?;
                let right = reader.read_u32_le()?;
                let bottom = reader.read_u32_le()?;
                let top = reader.read_u32_le()?;

                let mask_size = width as usize * height as usize;
                let mut pos = reader.position() as usize;
                reader.seek(SeekFrom::Current(4 * mask_size as i64))?;
                let mut mask = vec![0u8; mask_size];
                let src = reader.get_ref().as_ref();
                for i in 0..mask_size {
                    mask[i] = src[pos];
                    pos += 4;
                }

                Ok(CollisionMap {
                    bounds: BoundingBox {
                        width,
                        height,
                        top,
                        bottom,
                        left,
                        right,
                    },
                    data: mask.into_boxed_slice(),
                })
            }

            let mut colliders: Vec<CollisionMap>;
            let per_frame_colliders = reader.read_u32_le()? != 0;
            if per_frame_colliders {
                colliders = Vec::with_capacity(frame_count as usize);
                for _ in 0..frame_count {
                    colliders.push(read_collision(&mut reader, options)?);
                }
            } else {
                colliders = vec![read_collision(&mut reader, options)?];
            }
            (frames, colliders, per_frame_colliders)
        } else {
            (Vec::new(), Vec::new(), false)
        };

        Ok(Sprite {
            name,
            origin: Point { x, y },
            frames,
            colliders,
            per_frame_colliders,
        })
    }

    pub fn width(&self) -> u32 {
        self.frames.iter().next().map_or(1, |frame| frame.0.width)
    }

    pub fn height(&self) -> u32 {
        self.frames.iter().next().map_or(1, |frame| frame.0.height)
    }
}
