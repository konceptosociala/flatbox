use std::{fmt::Debug, path::{Path, PathBuf}};
use flatbox_core::logger::error;
use gl::types::GLuint;
use image::{EncodableLayout, RgbaImage};
use serde::{
    de::{Error as DeError, MapAccess, SeqAccess, Visitor}, 
    ser::SerializeStruct, 
    Deserialize, Deserializer, Serialize, Serializer
};

use crate::{
    macros::glenum_wrapper, 
    error::RenderError,
    pbr::color::Color,
};

glenum_wrapper! {
    wrapper: Filter,
    variants: [
        Linear,
        Nearest
    ]
}

glenum_wrapper! {
    wrapper: WrapMode,
    variants: [
        Repeat,
        ClampToEdge,
        MirroredRepeat
    ]
}

glenum_wrapper! {
    wrapper: ColorMode,
    variants: [
        Srgb8Alpha8,
        Rgba
    ]
}

glenum_wrapper! {
    wrapper: TextureOrder,
    variants: [
        Texture0, Texture1, Texture2, Texture3,
        Texture4, Texture5, Texture6, Texture7,
        Texture8, Texture9, Texture10, Texture11,
        Texture12, Texture13, Texture14, Texture15,
        Texture16, Texture17, Texture18, Texture19,
        Texture20, Texture21, Texture22, Texture23,
        Texture24, Texture25, Texture26, Texture27,
        Texture28, Texture29, Texture30, Texture31
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "RawImage")]
pub struct SerializeRawImage {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
}

impl From<SerializeRawImage> for RgbaImage {
    fn from(image: SerializeRawImage) -> Self {
        RgbaImage::from_raw(image.width, image.height, image.buffer).unwrap()
    }
}

impl From<RgbaImage> for SerializeRawImage {
    fn from(image: RgbaImage) -> Self {
        SerializeRawImage {
            width: image.width(),
            height: image.height(),
            buffer: image.into_raw(),
        }
    }
}

pub type TextureId = GLuint;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageType {
    Image2D,
    SubImage2D([usize; 2]),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureDescriptor {
    pub filter: Filter,
    pub wrap_mode: WrapMode,
    pub color_mode: ColorMode,
    pub image_type: ImageType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextureLoadType {
    Path(PathBuf),
    Color(Color, u32, u32),
    Generic,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        TextureDescriptor {
            filter: Filter::Linear,
            wrap_mode: WrapMode::Repeat,
            color_mode: ColorMode::Rgba,
            image_type: ImageType::Image2D,
        }
    }
}

#[readonly::make]
pub struct Texture {
    #[readonly]
    load_type: TextureLoadType,
    descriptor: TextureDescriptor,
    raw_data: RgbaImage,
    id: TextureId,
}

impl Clone for Texture {
    fn clone(&self) -> Self {
        unsafe { 
            Texture::new_internal(
                self.raw_data.as_bytes(), 
                self.raw_data.width(), 
                self.raw_data.height(), 
                Some(self.descriptor.clone()), 
                self.load_type.clone(),
            )
            .expect("Cannot clone texture: texture may be invalid or renderer may be not initialized")
        }
    }
}

impl Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("load_type", &self.load_type)
            .field("descriptor", &self.descriptor)
            .finish()
    }
}

impl Serialize for Texture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer
    {
        let mut texture = serializer.serialize_struct("Texture", 4)?;
        texture.serialize_field("load_type", &self.load_type)?;
        texture.serialize_field("descriptor", &self.descriptor)?;

        match self.load_type {
            TextureLoadType::Generic => {
                texture.serialize_field("raw_data", &Some(SerializeRawImage::from(self.raw_data.clone())))?;
            },
            _ => {
                texture.serialize_field("raw_data", &Option::<SerializeRawImage>::None)?;
            }
        }

        texture.end()
    }
}

impl<'de> Deserialize<'de> for Texture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum TextureField { 
            LoadType,
            Descriptor,
            RawData,
        }

        struct TextureVisitor;

        impl<'de> Visitor<'de> for TextureVisitor {
            type Value = Texture;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Texture")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Texture, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let load_type: TextureLoadType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;
                let descriptor: TextureDescriptor = seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))?;

                match load_type {
                    TextureLoadType::Path(ref path) => {
                        Texture::new(path, Some(descriptor)).map_err(V::Error::custom)
                    },
                    TextureLoadType::Color(color, width, height) => {
                        Texture::new_from_color(width, height, color).map_err(V::Error::custom)
                    },
                    TextureLoadType::Generic => {
                        let raw_data: Option<SerializeRawImage> = seq.next_element()?.ok_or_else(|| DeError::invalid_length(3, &self))?;
                        if let Some(image) = raw_data {
                            Texture::new_from_raw(
                                image.width, 
                                image.height, 
                                &image.buffer, 
                                Some(descriptor)
                            ).map_err(V::Error::custom)
                        } else {
                            error!("Error loading generic texture: image data is empty");
                            Texture::error().map_err(V::Error::custom)
                        }
                    },
                }
            }

            fn visit_map<V>(self, mut map: V) -> Result<Texture, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut load_type: Option<TextureLoadType> = None;
                let mut descriptor: Option<TextureDescriptor> = None;
                let mut raw_data: Option<Option<SerializeRawImage>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        TextureField::LoadType => {
                            if load_type.is_some() {
                                return Err(DeError::duplicate_field("load_type"));
                            }
                            load_type = Some(map.next_value()?);
                        },
                        TextureField::Descriptor => {
                            if descriptor.is_some() {
                                return Err(DeError::duplicate_field("descriptor"));
                            }
                            descriptor = Some(map.next_value()?);
                        },
                        TextureField::RawData => {
                            if raw_data.is_some() {
                                return Err(DeError::duplicate_field("raw_data"));
                            }
                            raw_data = Some(map.next_value()?);
                        },
                    }
                }

                let load_type = load_type.ok_or_else(|| DeError::missing_field("load_type"))?;
                let descriptor = descriptor.ok_or_else(|| DeError::missing_field("descriptor"))?;

                match load_type {
                    TextureLoadType::Path(ref path) => {
                        Texture::new(path, Some(descriptor)).map_err(V::Error::custom)
                    },
                    TextureLoadType::Color(color, width, height) => {
                        Texture::new_from_color(width, height, color).map_err(V::Error::custom)
                    },
                    TextureLoadType::Generic => {
                        let raw_data: Option<SerializeRawImage> = raw_data.ok_or_else(|| DeError::missing_field("raw_data"))?;
                        if let Some(image) = raw_data {
                            Texture::new_from_raw(
                                image.width, 
                                image.height, 
                                &image.buffer, 
                                Some(descriptor)
                            ).map_err(V::Error::custom)
                        } else {
                            error!("Error loading generic texture: image data is empty");
                            Texture::error().map_err(V::Error::custom)
                        }
                    },
                }
            }
        }

        const FIELDS: &[&str] = &[
            "texture_load_type",
            "texture_type",
            "filter",
            "raw_data"
        ];
        
        deserializer.deserialize_struct("Texture", FIELDS, TextureVisitor)
    }
}

impl Texture {
    pub fn new<P: AsRef<Path>>(path: P, descr: Option<TextureDescriptor>) -> Result<Texture, RenderError> {
        let img = image::open(path.as_ref())?.into_rgba8();
        unsafe { Texture::new_internal(
            img.as_bytes(), 
            img.width(), 
            img.height(), 
            descr,
            TextureLoadType::Path(path.as_ref().to_owned())
        ) }
    }

    pub fn new_from_color(width: u32, height: u32, color: Color) -> Result<Texture, RenderError> {
        let img = RgbaImage::from_pixel(width, height, image::Rgba(color.into()));
        unsafe { Texture::new_internal(
            img.as_bytes(), 
            img.width(), 
            img.height(), 
            Some(TextureDescriptor { filter: Filter::Nearest, ..Default::default()}),
            TextureLoadType::Color(color, width, height),
        ) }
    }

    pub fn new_from_raw(
        width: u32, 
        height: u32, 
        buf: &[u8],
        descr: Option<TextureDescriptor>,
    ) -> Result<Texture, RenderError> {
        unsafe { Texture::new_internal(
            buf, 
            width, 
            height, 
            descr, 
            TextureLoadType::Generic
        ) }
    }

    pub fn error() -> Result<Texture, RenderError> {
        Texture::new_from_raw(
            2, 2, 
            &[
                0, 0, 0, 255,
                255, 0, 255, 255,
                255, 0, 255, 255,
                0, 0, 0, 255,
            ],
            Some(TextureDescriptor {
                filter: Filter::Nearest,
                wrap_mode: WrapMode::Repeat,
                ..Default::default()
            })
        )
    }

    unsafe fn new_internal(
        buf: &[u8], 
        width: u32, 
        height: u32, 
        descr: Option<TextureDescriptor>,
        load_type: TextureLoadType,
    ) -> Result<Texture, RenderError> {
        let mut id: TextureId = 0;
        gl::GenTextures(1, &mut id);

        let texture = Texture {
            load_type,
            descriptor: descr.clone().unwrap_or_default(),
            raw_data: RgbaImage::from_raw(width, height, buf.to_vec()).ok_or(RenderError::WrongImageData)?,
            id
        };
        texture.bind();

        let descr = descr.unwrap_or_default();

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, descr.filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, descr.filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, descr.wrap_mode as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, descr.wrap_mode as i32);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

        match descr.image_type {
            ImageType::Image2D => gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                descr.color_mode as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as *const _,
            ),
            ImageType::SubImage2D([x, y]) => gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x as _,
                y as _,
                width as _,
                height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as *const _,
            )
        };

        Ok(texture)
    }

    pub fn activate(&self, order: TextureOrder) {
        unsafe { gl::ActiveTexture(order as u32); }
        self.bind();
    }

    pub fn bind(&self){
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Texture::new_from_color(16, 16, Color::WHITE)
            .expect("Cannot create texture: renderer may be not initialized")
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()); }
    }
}

pub fn load_image_from_memory(buf: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    match image::load_from_memory(buf) {
        Ok(img) => {
            let img = img.into_rgba8();
            let width = img.width();
            let height = img.height();
            
            Some((width, height, img.into_raw()))
        },
        _ => None,
    }
}

#[macro_export]
macro_rules! include_texture {
    ($file:expr $(,)?) => {
        {
            let buf = include_bytes!($file);
            let img = $crate::pbr::texture::load_image_from_memory(buf)
                .expect(format!("Cannot load `{}`", $file).as_str());
            
            Texture::new_from_raw(img.0, img.1, &img.2, None)
                .expect(format!("Cannot create texture from `{}`", $file).as_str())
        }
    };
}