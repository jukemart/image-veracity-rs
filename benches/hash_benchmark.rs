use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use std::fs::File;
use std::io::{BufReader, Read};

use glob::glob;
use image_veracity::hash::hash_image;

fn jpg_benchmark(c: &mut Criterion) {
    let group_name = "hashing_jpg";
    let pattern = "resources/test/**/*.jpg";

    create_resource_bench(c, group_name, pattern);
}

fn png_benchmark(c: &mut Criterion) {
    let group_name = "hashing_png";
    let pattern = "resources/test/**/*.png";

    create_resource_bench(c, group_name, pattern);
}

fn create_resource_bench(c: &mut Criterion, group_name: &str, pattern: &str) {
    let mut group = c.benchmark_group(group_name);
    group.sampling_mode(SamplingMode::Flat);

    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let display = path.display().to_string();
                let file = File::open(path).expect("should be a valid file");
                let mut file_reader = BufReader::new(file);
                let mut img_bytes = Vec::new();

                file_reader
                    .read_to_end(&mut img_bytes)
                    .expect("image file should be readable");
                group.bench_with_input(
                    BenchmarkId::new("phash_image", display.as_str()),
                    &img_bytes,
                    |b, img| {
                        b.iter(|| {
                            hash_image(img).expect("image to be valid");
                        });
                    },
                );
            }
            Err(e) => println!("{:?}", e),
        }
    }

    group.finish();
}

criterion_group!(benches, jpg_benchmark, png_benchmark);
criterion_main!(benches);
