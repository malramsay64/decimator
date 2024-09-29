use std::iter::zip;

use burn::backend::wgpu::select_device;
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::{BatchNorm, BatchNormConfig};
use burn::prelude::*;
use burn::tensor::activation::{relu, sigmoid};
use nn::pool::{MaxPool2d, MaxPool2dConfig};

use crate::anchors::{generate_anchors, AnchorOptions};

#[derive(Module, Debug)]
struct BlazeBlock<B: Backend> {
    conv1: Conv2d<B>,
    bn1: BatchNorm<B, 2>,
    conv2: Conv2d<B>,
    bn2: BatchNorm<B, 2>,
}

impl<B: Backend> BlazeBlock<B> {
    pub fn init(
        device: &B::Device,
        in_channels: usize,
        out_channels: usize,
        kernel_size: usize,
    ) -> Self {
        let conv1 = Conv2dConfig::new([in_channels, in_channels], [kernel_size; 2]).init(device);
        let bn1 = BatchNormConfig::new(in_channels).init(device);
        let conv2 = Conv2dConfig::new([in_channels, out_channels], [1; 2]).init(device);
        let bn2 = BatchNormConfig::new(out_channels).init(device);

        Self {
            conv1,
            bn1,
            conv2,
            bn2,
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(input.clone());
        let x = self.bn1.forward(x);
        let x = relu(x);
        let x = self.conv2.forward(x);
        let x = self.bn2.forward(x);

        relu(x + input)
    }
}

#[derive(Module, Debug)]
struct BlazeBlockStride<B: Backend> {
    conv1: Conv2d<B>,
    bn1: BatchNorm<B, 2>,
    conv2: Conv2d<B>,
    bn2: BatchNorm<B, 2>,
    max_pool_s: MaxPool2d,
    conv_s: Conv2d<B>,
    bn_s: BatchNorm<B, 2>,
}

impl<B: Backend> BlazeBlockStride<B> {
    pub fn init(
        device: &B::Device,
        in_channels: usize,
        out_channels: usize,
        kernel_size: usize,
    ) -> Self {
        let stride = 2;
        let conv1 = Conv2dConfig::new([in_channels; 2], [kernel_size; 2])
            .with_stride([stride; 2])
            .init(device);
        let bn1 = BatchNormConfig::new(in_channels).init(device);
        let conv2 = Conv2dConfig::new([in_channels, out_channels], [1; 2]).init(device);
        let bn2 = BatchNormConfig::new(out_channels).init(device);

        let max_pool_s = MaxPool2dConfig::new([stride; 2]).init();
        let conv_s = Conv2dConfig::new([in_channels, out_channels], [1; 2]).init(device);
        let bn_s = BatchNormConfig::new(out_channels).init(device);

        Self {
            conv1,
            bn1,
            conv2,
            bn2,
            max_pool_s,
            conv_s,
            bn_s,
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(input.clone());
        let x = self.bn1.forward(x);
        let x = relu(x);
        let x = self.conv2.forward(x);
        let x = self.bn2.forward(x);

        let h = self.max_pool_s.forward(input);
        let h = self.conv_s.forward(h);
        let h = self.bn_s.forward(h);
        let h = relu(h);

        relu(x + h)
    }
}

#[derive(Module, Debug)]
struct BlazeBlockFirst<B: Backend> {
    conv: Conv2d<B>,
    bn: BatchNorm<B, 2>,
}

impl<B: Backend> BlazeBlockFirst<B> {
    fn init(device: &B::Device, channels: usize, kernel_size: usize) -> Self {
        let conv = Conv2dConfig::new([3, channels], [kernel_size; 2])
            .with_stride([2; 2])
            .init(device);
        let bn = BatchNormConfig::new(channels).init(device);

        Self { conv, bn }
    }

    fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input);
        let x = self.bn.forward(x);
        relu(x)
    }
}

#[derive(Module, Debug)]
struct FinalBlazeBlock<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
}

impl<B: Backend> FinalBlazeBlock<B> {
    pub fn init(device: &B::Device, channels: usize, kernel_size: usize) -> Self {
        let conv1 = Conv2dConfig::new([channels; 2], [kernel_size; 2])
            .with_stride([2; 2])
            .init(device);
        let conv2 = Conv2dConfig::new([channels; 2], [1; 2]).init(device);

        Self { conv1, conv2 }
    }

    pub fn forward(self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(input);
        let x = self.conv2.forward(x);

        relu(x)
    }
}

#[derive(Module, Debug)]
pub struct BlazeFace<B: Backend> {
    init_block: BlazeBlockFirst<B>,
    group1_block1: BlazeBlock<B>,
    group1_block2: BlazeBlock<B>,
    group1_block3: BlazeBlock<B>,
    group1_block4: BlazeBlock<B>,
    group1_block5: BlazeBlock<B>,
    group1_block6: BlazeBlock<B>,
    group1_block7: BlazeBlock<B>,
    group1_block8: BlazeBlockStride<B>,
    group2_block1: BlazeBlock<B>,
    group2_block2: BlazeBlock<B>,
    group2_block3: BlazeBlock<B>,
    group2_block4: BlazeBlock<B>,
    group2_block5: BlazeBlock<B>,
    group2_block6: BlazeBlock<B>,
    group2_block7: BlazeBlock<B>,
    group2_block8: BlazeBlockStride<B>,
    group3_block1: BlazeBlock<B>,
    group3_block2: BlazeBlock<B>,
    group3_block3: BlazeBlock<B>,
    group3_block4: BlazeBlock<B>,
    group3_block5: BlazeBlock<B>,
    group3_block6: BlazeBlock<B>,
    group3_block7: BlazeBlock<B>,
    group3_block8: BlazeBlockStride<B>,
    group4_block1: BlazeBlock<B>,
    group4_block2: BlazeBlock<B>,
    group4_block3: BlazeBlock<B>,
    group4_block4: BlazeBlock<B>,
    group4_block5: BlazeBlock<B>,
    group4_block6: BlazeBlock<B>,
    group4_block7: BlazeBlock<B>,
    final_block: FinalBlazeBlock<B>,
    classifier_8: Conv2d<B>,
    classifier_16: Conv2d<B>,
    regressor_8: Conv2d<B>,
    regressor_16: Conv2d<B>,
    anchors: Tensor<B, 2>,
}

impl<B: Backend> BlazeFace<B> {
    pub fn init(device: &B::Device) -> Self {
        let group1_channels = 24;
        let group2_channels = 24;
        let group3_channels = 48;
        let group4_channels = 96;
        let final_channels = 96;
        let kernel_size = 5;
        let init_block = BlazeBlockFirst::init(device, group1_channels, kernel_size);
        let group1_block1 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block2 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block3 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block4 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block5 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block6 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block7 = BlazeBlock::init(device, group1_channels, group1_channels, kernel_size);
        let group1_block8 =
            BlazeBlockStride::init(device, group1_channels, group2_channels, kernel_size);
        let group2_block1 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block2 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block3 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block4 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block5 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block6 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block7 = BlazeBlock::init(device, group2_channels, group2_channels, kernel_size);
        let group2_block8 =
            BlazeBlockStride::init(device, group2_channels, group3_channels, kernel_size);
        let group3_block1 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block2 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block3 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block4 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block5 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block6 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block7 = BlazeBlock::init(device, group3_channels, group3_channels, kernel_size);
        let group3_block8 =
            BlazeBlockStride::init(device, group3_channels, group4_channels, kernel_size);
        let group4_block1 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block2 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block3 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block4 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block5 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block6 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);
        let group4_block7 = BlazeBlock::init(device, group4_channels, group4_channels, kernel_size);

        let final_block = FinalBlazeBlock::init(device, final_channels, kernel_size);
        let classifier_8 = Conv2dConfig::new([final_channels, 2], [1; 2]).init(device);
        let classifier_16 = Conv2dConfig::new([final_channels, 6], [1; 2]).init(device);
        let regressor_8 = Conv2dConfig::new([final_channels, 32], [1; 2]).init(device);
        let regressor_16 = Conv2dConfig::new([final_channels, 96], [1; 2]).init(device);

        let anchor_vec: Vec<_> = generate_anchors(&AnchorOptions::back())
            .into_iter()
            .map(|i| Tensor::<B, 1>::from_floats(i, device).unsqueeze())
            .collect();
        let anchors = Tensor::cat(anchor_vec, 0);

        Self {
            init_block,
            group1_block1,
            group1_block2,
            group1_block3,
            group1_block4,
            group1_block5,
            group1_block6,
            group1_block7,
            group1_block8,
            group2_block1,
            group2_block2,
            group2_block3,
            group2_block4,
            group2_block5,
            group2_block6,
            group2_block7,
            group2_block8,
            group3_block1,
            group3_block2,
            group3_block3,
            group3_block4,
            group3_block5,
            group3_block6,
            group3_block7,
            group3_block8,
            group4_block1,
            group4_block2,
            group4_block3,
            group4_block4,
            group4_block5,
            group4_block6,
            group4_block7,
            final_block,
            classifier_8,
            classifier_16,
            regressor_8,
            regressor_16,
            anchors,
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> (Tensor<B, 3>, Tensor<B, 3>) {
        let x = self.init_block.forward(input);
        let x = self.group1_block1.forward(x);
        let x = self.group1_block2.forward(x);
        let x = self.group1_block3.forward(x);
        let x = self.group1_block4.forward(x);
        let x = self.group1_block5.forward(x);
        let x = self.group1_block6.forward(x);
        let x = self.group1_block7.forward(x);
        let x = self.group1_block8.forward(x);
        let x = self.group2_block1.forward(x);
        let x = self.group2_block2.forward(x);
        let x = self.group2_block3.forward(x);
        let x = self.group2_block4.forward(x);
        let x = self.group2_block5.forward(x);
        let x = self.group2_block6.forward(x);
        let x = self.group2_block7.forward(x);
        let x = self.group2_block8.forward(x);
        let x = self.group3_block1.forward(x);
        let x = self.group3_block2.forward(x);
        let x = self.group3_block3.forward(x);
        let x = self.group3_block4.forward(x);
        let x = self.group3_block5.forward(x);
        let x = self.group3_block6.forward(x);
        let x = self.group3_block7.forward(x);
        let x = self.group3_block8.forward(x);
        let x = self.group4_block1.forward(x);
        let x = self.group4_block2.forward(x);
        let x = self.group4_block3.forward(x);
        let x = self.group4_block4.forward(x);
        let x = self.group4_block5.forward(x);
        let x = self.group4_block6.forward(x);
        let x = self.group4_block7.forward(x);
        let h = self.final_block.clone().forward(x.clone());

        let c1 = self
            .classifier_8
            .forward(x.clone())
            .permute([0, 2, 3, 1])
            .reshape([0, -1, 1]);

        let c2 = self
            .classifier_16
            .forward(h.clone())
            .permute([0, 2, 3, 1])
            .reshape([0, -1, 1]);

        let c = Tensor::cat(vec![c1, c2], 1);

        let r1 = self
            .regressor_8
            .forward(x)
            .permute([0, 2, 3, 1])
            .reshape([0, -1, 1]);
        let r2 = self
            .regressor_16
            .forward(h)
            .permute([0, 2, 3, 1])
            .reshape([0, -1, 1]);

        let r = Tensor::cat(vec![r1, r2], 1);

        return (r, c);
    }

    pub fn predict(&self, input: Tensor<B, 4>) -> Vec<Tensor<B, 2>> {
        let (detections, scores) = self.forward(input);
        let predictions = self.tensors_to_detections(detections, scores, self.anchors.clone());
        predictions
    }

    // https://github.com/google-ai-edge/mediapipe/blob/master/mediapipe/calculators/tflite/tflite_tensors_to_detections_calculator.cc
    fn tensors_to_detections(
        &self,
        raw_box_tensor: Tensor<B, 3>,
        raw_score_tensor: Tensor<B, 3>,
        anchors: Tensor<B, 2>,
    ) -> Vec<Tensor<B, 2>> {
        let boxes = self.decode_boxes(raw_box_tensor, anchors);
        let thresh = 0.2;
        let raw_score_tensor = raw_score_tensor.clamp(-thresh, thresh);
        let detection_scores: Tensor<B, 2> = sigmoid(raw_score_tensor).squeeze(2);
        let mask = detection_scores.clone().lower_equal_elem(0.1).argwhere();

        let mut detections = vec![];
        for ((feature, score), m) in boxes
            .iter_dim(0)
            .zip(detection_scores.iter_dim(1))
            .zip(mask.iter_dim(0))
        {
            let x = feature.select(1, m.clone().squeeze(0)).squeeze(0);
            let s = score.select(1, m.squeeze(0));
            detections.push(Tensor::cat([x, s].into(), 0))
        }

        detections
    }

    fn decode_boxes(&self, raw_boxes: Tensor<B, 3>, anchors: Tensor<B, 2>) -> Tensor<B, 3> {
        assert_eq!(raw_boxes.dims()[2], anchors.dims()[0]);
        assert!(anchors.dims()[1] == 4);
        let shape_boxes = raw_boxes.clone().dims();
        let shape_anchors = anchors.clone().dims();
        let x_center = (raw_boxes
            .clone()
            .slice([0..shape_boxes[0], 0..shape_boxes[1], 0..1])
            / 256.)
            * anchors
                .clone()
                .slice([0..shape_anchors[0], 2..3])
                .unsqueeze()
            + anchors
                .clone()
                .slice([0..shape_anchors[0], 0..1])
                .unsqueeze();
        let y_center = raw_boxes
            .clone()
            .slice([0..shape_boxes[0], 0..shape_boxes[1], 1..2])
            / 256.
            * anchors
                .clone()
                .slice([0..shape_anchors[0], 3..4])
                .unsqueeze()
            + anchors
                .clone()
                .slice([0..shape_anchors[0], 1..2])
                .unsqueeze();

        let w = raw_boxes
            .clone()
            .slice([0..shape_boxes[0], 0..shape_boxes[1], 2..3])
            / 256.
            * anchors
                .clone()
                .slice([0..shape_anchors[0], 2..3])
                .unsqueeze();
        let h = raw_boxes
            .clone()
            .slice([0..shape_boxes[0], 0..shape_boxes[1], 3..4])
            / 256.
            * anchors
                .clone()
                .slice([0..shape_anchors[0], 3..4])
                .unsqueeze();

        Tensor::cat([x_center, y_center, w, h].into(), 2)
    }
}

#[cfg(test)]
mod tests {
    use burn::backend::wgpu::WgpuDevice;
    use burn::backend::Wgpu;
    use burn::module::Devices;
    use burn::tensor::Tensor;

    // use super::*;

    #[test]
    fn test_slice() {
        // Type alias for the backend to use.
        type Backend = Wgpu;

        let device = Default::default();
        // Creation of two tensors, the first with explicit values and the second one with ones, with the same shape as the first
        let tensor_1 = Tensor::<Backend, 2>::from_data([[2., 3.], [4., 5.]], &device);
        let tensor_2 = Tensor::<Backend, 2>::ones_like(&tensor_1);
        let tensor_res = Tensor::<Backend, 1>::from_data([2., 4.], &device);

        // Print the element-wise addition (done with the WGPU backend) of the two tensors.
        println!("{}", tensor_1.clone() + tensor_2);
        let len = tensor_1.shape().dims[0];
        let tensor_out = tensor_1.clone().slice([0..len, 0..1]);
        dbg!(tensor_res.clone(), tensor_out.clone());
        assert_eq!(tensor_res.into_data().value, tensor_out.into_data().value);
    }
}
