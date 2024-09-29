use std::path::Path;

use approx::assert_abs_diff_eq;
use facial_recognition::anchors::*;

fn read_anchor_reference<P: AsRef<Path>>(input: P) -> Vec<[f32; 4]> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_path(input)
        .expect("Unable to open file");

    reader
        .records()
        .map(|r| {
            let row: [f32; 4] = r.unwrap().deserialize(None).unwrap();
            row
        })
        .collect()
}

#[test]
fn test_options_0() {
    let options = AnchorOptions {
        min_scale: 0.117875,
        max_scale: 0.75,
        input_size_height: 256,
        input_size_width: 256,
        anchor_offset_x: 0.5,
        anchor_offset_y: 0.5,
        strides: [8, 16, 32, 32, 32],
        aspect_ratios: [1.0].into(),
        reduce_boxes_in_lowest_layer: false,
        interpolated_scale_aspect_ratio: 1.0,
        fixed_anchor_size: true,
    };

    let anchors = generate_anchors(&options);
    let reference = read_anchor_reference("tests/data/anchor_golden_file_0.txt");

    for (_index, (left, right)) in anchors.iter().zip(reference.iter()).enumerate() {
        // dbg!(index);
        assert_eq!(left, right);
    }
    assert_eq!(anchors.len(), reference.len());
    assert_eq!(anchors, reference);
}

#[test]
fn test_options_1() {
    let options = AnchorOptions {
        min_scale: 0.2,
        max_scale: 0.95,
        input_size_height: 300,
        input_size_width: 300,
        anchor_offset_x: 0.5,
        anchor_offset_y: 0.5,
        strides: [16, 32, 64, 128, 256, 512],
        aspect_ratios: [1.0, 2.0, 0.5, 3.0, 0.3333].into(),
        reduce_boxes_in_lowest_layer: true,
        interpolated_scale_aspect_ratio: 1.0,
        fixed_anchor_size: false,
    };

    let anchors = generate_anchors(&options);
    let reference = read_anchor_reference("tests/data/anchor_golden_file_1.txt");

    for (_index, (left, right)) in anchors.iter().zip(reference.iter()).enumerate() {
        // dbg!(index);
        for (l, r) in left.iter().zip(right) {
            assert_abs_diff_eq!(l, r, epsilon = 1e-5);
        }
    }
    assert_eq!(anchors.len(), reference.len());
}
