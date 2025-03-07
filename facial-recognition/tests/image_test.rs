use burn::tensor::backend::Backend;
use burn::tensor::{Device, Element, Shape, Tensor, TensorData};
use facial_recognition::model::BlazeFace;

fn to_tensor<B: Backend, T: Element>(
    data: Vec<T>,
    shape: [usize; 3],
    device: &Device<B>,
) -> Tensor<B, 3> {
    Tensor::<B, 3>::from_data(TensorData::new(data, Shape::new(shape)), device)
        // [H, W, C] -> [C, H, W]
        .permute([2, 0, 1])
}

#[test]
fn test_image() {
    let device = burn::backend::wgpu::WgpuDevice::default();
    let model = BlazeFace::<burn::backend::Wgpu>::init(&device);

    let img = image::open("tests/data/img_136.jpg").unwrap().resize_exact(
        256,
        256,
        image::imageops::FilterType::Triangle,
    );

    let x = to_tensor(img.into_rgb8().into_raw(), [256, 256, 3], &device).unsqueeze::<4>();
    let out = model.predict(x);
    dbg!(out);
}
