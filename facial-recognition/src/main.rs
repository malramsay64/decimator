use burn::backend::Wgpu;
use burn::tensor::Tensor;

type Backend = Wgpu;

fn main() {
    let device = Default::default();
    // Creation of two tensors, the first with explicit values and the second one with ones, with the same shape as the first
    let tensor_1 = Tensor::<Backend, 2>::from_data([[2., 3.], [4., 5.]], &device);
    let tensor_2 = Tensor::<Backend, 2>::ones_like(&tensor_1);
    let tensor_res = Tensor::<Backend, 2>::from_data([[2., 3.]], &device);

    // Print the element-wise addition (done with the WGPU backend) of the two tensors.
    println!("{}", tensor_1.clone() + tensor_2);
    let len = tensor_1.shape().dims[0];
    let tensor_out = tensor_1.clone().slice([0..1, 0..len]);
    dbg!(&tensor_res, &tensor_out);
    // assert!(false)
    println!("{:.2}", tensor_out);
    println!("{:.2}", tensor_res);

    let result: bool = tensor_out.equal(tensor_res).all().into_scalar();
    assert!(result);
    // assert!(false);
    // let val = false;
    // assert_eq!(true, val);
}
