use burn::record::{FullPrecisionSettings, Recorder};
use burn::tensor::backend::Backend;
use burn::tensor::{Device, Element, Shape, Tensor, TensorData};
use burn_import::pytorch::{LoadArgs, PyTorchFileRecorder};
use facial_recognition::model::BlazeFace;
#[test]
fn test_load() {
    let device = burn::backend::wgpu::WgpuDevice::default();

    let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load("tests/data/blazeface.pth".into(), &device)
        .with_key_remap("backbone.0.weight", "init_block.conv.weight")
        .with_key_remap("backbone.0.bias", "init_block.conv.bias")
        .expect("Should decode state successfully");

    let model = BlazeFace::<burn::backend::Wgpu>::init(&device);

    let load_args = LoadArgs::new();
}
