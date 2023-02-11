use {
    std::{fs::File, io::BufReader},
    tracing::info,
    tch::{
        nn::{self, VarStore, ConvConfig, Conv2D, Linear, OptimizerConfig},
        data::Iter2,
        Kind::Float,
        Tensor,
        Device,
    },
    npyz::npz::NpzArchive,
    image::{DynamicImage, imageops::FilterType, GenericImageView},
};

pub struct SimpleMnistModel {
    vs: VarStore,

    conv1: Conv2D,
    conv2: Conv2D,
    linear1: Linear,
}

impl SimpleMnistModel {
    pub fn new() -> Self {
        let mut vs = VarStore::new(Device::cuda_if_available());

        let root = vs.root();

        let conv1 = nn::conv2d(&root, 1, 16, 3, Default::default());
        let conv2 = nn::conv2d(&root, 16, 2, 3, Default::default());
        
        let linear1 = nn::linear(&root, 2 * 24 * 24, 10, Default::default());

        vs.load("./server/data/model.model").unwrap();

        Self {
            vs,

            conv1,
            conv2,
            linear1,
        }
    }

    fn forward(&self, xs: &Tensor, _train: bool) -> Tensor {
        xs
            .apply(&self.conv1)
            .relu()
            .apply(&self.conv2)
            .relu()
            .view([-1, 2 * 24 * 24])
            .apply(&self.linear1)
    }

    pub fn run(&self, image: &DynamicImage) {
        let tensor = self.image_to_tensor(image);
    }

    pub fn train(&self) {
        let mut npz = npyz::npz::NpzArchive::open("./server/data/mnist.npz").unwrap();
        let x_train = self.xs_dataset_from_npz(&mut npz, "x_train");
        let y_train = self.ys_dataset_from_npz(&mut npz, "y_train");

        let x_test = self.xs_dataset_from_npz(&mut npz, "x_test");
        let y_test = self.ys_dataset_from_npz(&mut npz, "y_test");
        
        let mut opt = nn::Adam::default().build(&self.vs, 1e-4).unwrap();
   
        for epoch in 0..100 {
            info!("running epoch {}", epoch);

            let mut iter = Iter2::new(&x_train, &y_train, 1024);
            iter.shuffle();
            iter.return_smaller_last_batch();

            while let Some((xs, ys)) = iter.next() {
                let loss = self
                    .forward(&xs, true)
                    .cross_entropy_for_logits(&ys);

                opt.backward_step(&loss);
            }

            let test_acc = self
                .forward(&x_test, false)
                .accuracy_for_logits(&y_test);

            info!("epoch {}, test accuracy: {}", epoch, f32::from(test_acc));
        }

        self.vs.save("./server/data/model.model").unwrap();
    }

    fn xs_dataset_from_npz(&self, npz: &mut NpzArchive<BufReader<File>>, name: &str) -> Tensor {
        let npy = npz.by_name(name).unwrap().unwrap();
    
        let shape = npy.shape().to_vec();
        self.dataset_to_tensor(
            npy.data().unwrap().map(|v: Result<u8, _>| v.unwrap() as f32).collect(), 
            shape.iter().map(|v| *v as i64).collect(),
        ).divide_scalar(255).view([-1, 1, 28, 28])
    }

    fn ys_dataset_from_npz(&self, npz: &mut NpzArchive<BufReader<File>>, name: &str) -> Tensor {
        let npy = npz.by_name(name).unwrap().unwrap();

        let shape = npy.shape().to_vec();
        self.dataset_to_tensor(
            npy.data().unwrap().map(|v: Result<u8, _>| v.unwrap() as i64).collect(), 
            shape.iter().map(|v| *v as i64).collect(),
        )
    }

    fn dataset_to_tensor<T: tch::kind::Element>(&self, data: Vec<T>, shape: Vec<i64>) -> Tensor {
        Tensor::of_slice(&data)
            .reshape(&shape)
            .to_device(self.vs.device())
    }

    fn image_to_tensor(&self, image: &DynamicImage) -> Tensor {
        let resized = image.resize_exact(28, 28, FilterType::Lanczos3);
        
        let grayscale = resized.grayscale();

        let mut data = vec![0.0; 28 * 28];
        for y in 0..28 {
            for x in 0..28 {
                let pixel = grayscale.get_pixel(x, y).0;
                let pixel = 1.0 - (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / (3.0 * 255.0);
                data[(y * 28 + x) as usize] = pixel;
            }
        }

        Tensor::of_slice(&data).view([-1, 1, 28, 28])
    }
}

pub fn run_simple_model_inference() {
    let model = SimpleMnistModel::new();
    let image = image::io::Reader::open("./server/data/seven.jpeg").unwrap().decode().unwrap();
    let input_tensor = model.image_to_tensor(&image);
    info!("size of tensor is: {:?}", input_tensor.size());
    let result = model.forward(&input_tensor, false);
    result.print();
}