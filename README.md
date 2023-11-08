# RZIV
Simple Image Viewer writtern in rust.


## Supported Image Format 

| Format | Decoding | Encoding |
| ------ | -------- | -------- |
| AVIF   | Only 8-bit \*\* | Lossy |
| BMP    | Yes | Rgb8, Rgba8, Gray8, GrayA8 |
| DDS    | DXT1, DXT3, DXT5 | No |
| Farbfeld | Yes | Yes |
| GIF    | Yes | Yes |
| ICO    | Yes | Yes |
| JPEG   | Baseline and progressive | Baseline JPEG |
| OpenEXR  | Rgb32F, Rgba32F (no dwa compression) | Rgb32F, Rgba32F (no dwa compression) |
| PNG    | All supported color types | Same as decoding |
| PNM    | PBM, PGM, PPM, standard PAM | Yes |
| QOI    | Yes | Yes |
| TGA    | Yes | Rgb8, Rgba8, Bgr8, Bgra8, Gray8, GrayA8 |
| TIFF   | Baseline(no fax support) + LZW + PackBits | Rgb8, Rgba8, Gray8 |
| WebP   | Yes | Rgb8, Rgba8 \* |

- thanks to rust crate [image](https://crates.io/crates/image) for providing implementations of common image format encoders and decoders.

## Clipboard support
support copy and paste image, path, or URI, from system clipboard to

## TODO
- [ ] TODO: region copy from opened image
- [ ] TODO: basic editing support, like:
    - [ ] blur: Performs a Gaussian blur on the supplied image.
    - [ ] brighten: Brighten the supplied image.
    - [ ] huerotate: Hue rotate the supplied image by degrees.
    - [ ] contrast: Adjust the contrast of the supplied image.
    - [ ] crop: Return a mutable view into an image.
    - [ ] filter3x3: Perform a 3x3 box filter on the supplied image.
    - [ ] flip_horizontal: Flip an image horizontally.
    - [ ] flip_vertical: Flip an image vertically.
    - [ ] grayscale: Convert the supplied image to grayscale.
    - [ ] invert: Invert each pixel within the supplied image This function operates in place.
    - [ ] resize: Resize the supplied image to the specified dimensions.
    - [ ] rotate180: Rotate an image 180 degrees clockwise.
    - [ ] rotate270: Rotate an image 270 degrees clockwise.
    - [ ] rotate90: Rotate an image 90 degrees clockwise.
    - [ ] unsharpen: Performs an unsharpen mask on the supplied image.

