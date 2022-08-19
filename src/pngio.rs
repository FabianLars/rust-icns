use image::{Image, PixelFormat};
use png;
use std::io::{self, Read, Write};

impl Image {
    /// Reads the raw pixel data from a PNG file.
    pub fn read_png<R: Read>(input: R) -> io::Result<Image> {
        Image::decode_from_png(input)
    }

    /// Reads an image from a PNG file without changing the data (useful for compressed files).
    pub fn read_png_unchanged<R: Read>(mut input: R) -> io::Result<Image> {
        let mut v = Vec::new();
        input.read_to_end(&mut v)?;
        let image = Image::decode_from_png(v.as_slice())?;
        Ok(Image {
            format: PixelFormat::PNG,
            width: image.width,
            height: image.height,
            data: v.into_boxed_slice(),
        })
    }

    /// Internal function to decode a PNG.
    pub(crate) fn decode_from_png<R: Read>(input: R) -> io::Result<Image> {
        let decoder = png::Decoder::new(input);
        let mut reader = decoder.read_info()?;
        let info = reader.info();
        let pixel_format = match info.color_type {
            png::ColorType::Rgba => PixelFormat::RGBA,
            png::ColorType::Rgb => PixelFormat::RGB,
            png::ColorType::GrayscaleAlpha => PixelFormat::GrayAlpha,
            png::ColorType::Grayscale => PixelFormat::Gray,
            _ => {
                // TODO: Support other color types.
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "unsupported PNG color \
                                                   type: {:?}",
                        info.color_type
                    ),
                ));
            }
        };
        if info.bit_depth != png::BitDepth::Eight {
            // TODO: Support other bit depths.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported PNG bit depth: \
                                               {:?}",
                    info.bit_depth
                ),
            ));
        }
        let mut image = Image::new(pixel_format, info.width, info.height);
        assert_eq!(image.data().len(), reader.output_buffer_size());
        reader.next_frame(image.data_mut())?;
        Ok(image)
    }

    /// Writes the image to a PNG file.
    pub fn write_png<W: Write>(&self, mut output: W) -> io::Result<()> {
        let color_type = match self.format {
            PixelFormat::RGBA => png::ColorType::Rgba,
            PixelFormat::RGB => png::ColorType::Rgb,
            PixelFormat::GrayAlpha => png::ColorType::GrayscaleAlpha,
            PixelFormat::Gray => png::ColorType::Grayscale,
            PixelFormat::Alpha => {
                return self.convert_to(PixelFormat::GrayAlpha).write_png(output);
            }
            PixelFormat::PNG => {
                return output.write(&self.data).map(|_| ());
            }
        };
        let mut encoder = png::Encoder::new(output, self.width, self.height);
        encoder.set_color(color_type);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer
            .write_image_data(&self.data)
            .map_err(|err| match err {
                png::EncodingError::IoError(err) => err,
                png::EncodingError::Format(err) => {
                    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
                }
                png::EncodingError::Parameter(err) => {
                    io::Error::new(io::ErrorKind::InvalidInput, err.to_string())
                }
                png::EncodingError::LimitsExceeded => {
                    io::Error::new(io::ErrorKind::Other, err.to_string())
                }
            })
    }
}
