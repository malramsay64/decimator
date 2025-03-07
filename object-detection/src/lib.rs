use std::path::Path;
use std::time::Instant;

use burn::backend::Wgpu;
use burn::tensor::backend::Backend;
use burn::tensor::{Device, Element, Tensor, TensorData};
use image::{DynamicImage, ImageBuffer};
use yolox_burn::model::boxes::nms;
use yolox_burn::model::yolox::Yolox;
use yolox_burn::model::{weights, BoundingBox};

const HEIGHT: usize = 640;
const WIDTH: usize = 640;

const MODEL_CLASSES: [&'static str; 80] = [
    "person",
    "bicycle",
    "car",
    "motorcycle",
    "airplane",
    "bus",
    "train",
    "truck",
    "boat",
    "traffic light",
    "fire hydrant",
    "stop sign",
    "parking meter",
    "bench",
    "bird",
    "cat",
    "dog",
    "horse",
    "sheep",
    "cow",
    "elephant",
    "bear",
    "zebra",
    "giraffe",
    "backpack",
    "umbrella",
    "handbag",
    "tie",
    "suitcase",
    "frisbee",
    "skis",
    "snowboard",
    "sports ball",
    "kite",
    "baseball bat",
    "baseball glove",
    "skateboard",
    "surfboard",
    "tennis racket",
    "bottle",
    "wine glass",
    "cup",
    "fork",
    "knife",
    "spoon",
    "bowl",
    "banana",
    "apple",
    "sandwich",
    "orange",
    "broccoli",
    "carrot",
    "hot dog",
    "pizza",
    "donut",
    "cake",
    "chair",
    "couch",
    "potted plant",
    "bed",
    "dining table",
    "toilet",
    "tv",
    "laptop",
    "mouse",
    "remote",
    "keyboard",
    "cell phone",
    "microwave",
    "oven",
    "toaster",
    "sink",
    "refrigerator",
    "book",
    "clock",
    "vase",
    "scissors",
    "teddy bear",
    "hair drier",
    "toothbrush",
];

fn to_tensor<B: Backend, T: Element>(
    data: Vec<T>,
    shape: [usize; 3],
    device: &Device<B>,
) -> Tensor<B, 3> {
    Tensor::<B, 3>::from_data(
        TensorData::new(data, shape).convert::<B::FloatElem>(),
        device,
    )
    // [H, W, C] -> [C, H, W]
    .permute([2, 0, 1])
}

/// Draws bounding boxes on the given image.
///
/// # Arguments
///
/// * `image`: Original input image.
/// * `boxes` - Bounding boxes, grouped per class.
/// * `color` - [R, G, B] color values to draw the boxes.
/// * `ratio` - [x, y] aspect ratio to scale the predicted boxes.
///
/// # Returns
///
/// The image annotated with bounding boxes.
fn draw_boxes(
    image: DynamicImage,
    boxes: &[Vec<BoundingBox>],
    color: &[u8; 3],
    ratio: &[f32; 2], // (x, y) ratio
) -> DynamicImage {
    // Assumes x1 <= x2 and y1 <= y2
    fn draw_rect(
        image: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>,
        x1: u32,
        x2: u32,
        y1: u32,
        y2: u32,
        color: &[u8; 3],
    ) {
        for x in x1..=x2 {
            let pixel = image.get_pixel_mut(x, y1);
            *pixel = image::Rgb(*color);
            let pixel = image.get_pixel_mut(x, y2);
            *pixel = image::Rgb(*color);
        }
        for y in y1..=y2 {
            let pixel = image.get_pixel_mut(x1, y);
            *pixel = image::Rgb(*color);
            let pixel = image.get_pixel_mut(x2, y);
            *pixel = image::Rgb(*color);
        }
    }

    // Annotate the original image and print boxes information.
    let (image_h, image_w) = (image.height(), image.width());
    let mut image = image.to_rgb8();
    for (class_index, bboxes_for_class) in boxes.iter().enumerate() {
        for b in bboxes_for_class.iter() {
            let xmin = (b.xmin * ratio[0]).clamp(0., image_w as f32 - 1.);
            let ymin = (b.ymin * ratio[1]).clamp(0., image_h as f32 - 1.);
            let xmax = (b.xmax * ratio[0]).clamp(0., image_w as f32 - 1.);
            let ymax = (b.ymax * ratio[1]).clamp(0., image_h as f32 - 1.);

            println!(
                "Predicted {} ({:.2}) at [{:.2}, {:.2}, {:.2}, {:.2}]",
                MODEL_CLASSES[class_index], b.confidence, xmin, ymin, xmax, ymax,
            );

            draw_rect(
                &mut image,
                xmin as u32,
                xmax as u32,
                ymin as u32,
                ymax as u32,
                color,
            );
        }
    }
    DynamicImage::ImageRgb8(image)
}
