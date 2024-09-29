#[derive(Debug, Clone)]
pub struct AnchorOptions<const L: usize> {
    // num_layers: u64,
    pub min_scale: f32,
    pub max_scale: f32,
    pub input_size_height: u64,
    pub input_size_width: u64,
    pub anchor_offset_x: f32,
    pub anchor_offset_y: f32,
    pub strides: [u64; L],
    pub aspect_ratios: Vec<f32>,
    pub reduce_boxes_in_lowest_layer: bool,
    pub interpolated_scale_aspect_ratio: f32,
    pub fixed_anchor_size: bool,
}

impl<const L: usize> AnchorOptions<L> {
    pub fn calculate_scale(&self, stride_index: usize) -> f32 {
        self.min_scale + (self.max_scale - self.min_scale) * stride_index as f32 / (L as f32 - 1.)
    }
}
// ref: https://github.com/hollance/BlazeFace-PyTorch/blob/master/Anchors.ipynb
impl AnchorOptions<4> {
    pub fn back() -> Self {
        Self {
            // num_layers: 4,
            min_scale: 0.15625,
            max_scale: 0.75,
            input_size_height: 256,
            input_size_width: 256,
            anchor_offset_x: 0.5,
            anchor_offset_y: 0.5,
            strides: [16, 32, 32, 32],
            aspect_ratios: vec![1.0],
            reduce_boxes_in_lowest_layer: false,
            interpolated_scale_aspect_ratio: 1.0,
            fixed_anchor_size: true,
        }
    }
}

pub fn generate_anchors<const L: usize>(options: &AnchorOptions<L>) -> Vec<[f32; 4]> {
    let mut anchors = vec![];

    let mut layer_id = 0;
    while layer_id < L {
        let mut anchor_height: Vec<f32> = vec![];
        let mut anchor_width: Vec<f32> = vec![];
        let mut aspect_ratios: Vec<f32> = vec![];
        let mut scales: Vec<f32> = vec![];

        let mut last_same_stride_layer = layer_id;
        dbg!(last_same_stride_layer);
        while (last_same_stride_layer < L)
            && (options.strides[last_same_stride_layer] == options.strides[layer_id])
        {
            println!("Layers: {last_same_stride_layer}, {layer_id}");

            let scale = options.calculate_scale(last_same_stride_layer);

            if last_same_stride_layer == 0 && options.reduce_boxes_in_lowest_layer {
                // For the first layer specify pre-defined anchors
                aspect_ratios.extend_from_slice(&[1., 2., 0.5]);
                scales.extend_from_slice(&[0.1, scale, scale]);
            } else {
                aspect_ratios.extend_from_slice(&options.aspect_ratios);
                scales.append(&mut options.aspect_ratios.iter().map(|_| scale).collect());

                if options.interpolated_scale_aspect_ratio > 0. {
                    let scale_next = if last_same_stride_layer == L - 1 {
                        1.0
                    } else {
                        options.calculate_scale(last_same_stride_layer + 1)
                    };
                    scales.push((scale * scale_next).sqrt());
                    aspect_ratios.push(options.interpolated_scale_aspect_ratio);
                }
            }
            last_same_stride_layer += 1;
        }

        for (ratio, scale) in aspect_ratios.iter().zip(scales.iter()) {
            anchor_height.push(scale / ratio.sqrt());
            anchor_width.push(scale * ratio.sqrt());
        }
        dbg!(&scales);
        dbg!(&aspect_ratios);
        dbg!(&anchor_width);
        dbg!(&anchor_height);

        // Calculate values

        let stride = options.strides[layer_id];
        // dbg!(stride);
        let feature_map_height = num::integer::div_ceil(options.input_size_height, stride);
        let feature_map_width = num::integer::div_ceil(options.input_size_width, stride);

        // dbg!(feature_map_width, feature_map_height);

        for y in 0..feature_map_height {
            for x in 0..feature_map_width {
                for (&a_width, &a_height) in anchor_width.iter().zip(anchor_height.iter()) {
                    let x_center = (x as f32 + options.anchor_offset_x) / feature_map_width as f32;
                    let y_center = (y as f32 + options.anchor_offset_y) / feature_map_height as f32;

                    let (width, height) = if options.fixed_anchor_size {
                        (1., 1.)
                    } else {
                        (a_width, a_height)
                    };

                    anchors.push([x_center, y_center, width, height]);
                }
            }
        }
        layer_id = last_same_stride_layer;
    }

    anchors
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_len_back() {
        let options = AnchorOptions::back();
        let anchors = generate_anchors(&options);
        dbg!(anchors.len());
        assert!(anchors.len() == 896);
    }
}
