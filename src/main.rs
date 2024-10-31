use std::io::BufWriter;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn generate_value(rng: &mut ChaCha8Rng, range: (f64, f64)) -> f64 {
    let min = range.0.min(range.1);
    let max = range.0.max(range.1);
    rng.gen_range(min..=max)
}

#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Copy, Clone)]
struct Cluster {
    p1: Point,
    p2: Point,
}

fn generate_cluster(rng: &mut ChaCha8Rng) -> Cluster {
    let range = (-180.0, 180.0);
    let p1 = Point {
        x: generate_value(rng, range),
        y: generate_value(rng, range),
    };
    let p2 = Point {
        x: generate_value(rng, range),
        y: generate_value(rng, range),
    };
    Cluster { p1, p2 }
}

fn generate_point_in_cluster(rng: &mut ChaCha8Rng, cluster: Cluster) -> Point {
    let range_x = (cluster.p1.x, cluster.p2.x);
    let range_y = (cluster.p1.y, cluster.p2.y);
    Point {
        x: generate_value(rng, range_x),
        y: generate_value(rng, range_y),
    }
}

#[derive(serde::Serialize)]
struct JsonEntry {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

#[derive(serde::Serialize)]
struct JsonStructure {
    pairs: Vec<JsonEntry>,
}

fn radians_from_degrees(degrees: f64) -> f64 {
    0.01745329251994329577 * degrees
}

// NOTE: EarthRadius is generally expected to be 6372.8
fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: f64) -> f64 {
    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let d_lat = radians_from_degrees(lat2 - lat1);
    let d_lon = radians_from_degrees(lon2 - lon1);
    let lat1 = radians_from_degrees(lat1);
    let lat2 = radians_from_degrees(lat2);

    let a = ((d_lat / 2.0).sin()).powi(2) + lat1.cos() * lat2.cos() * ((d_lon / 2.0).sin()).powi(2);
    let c = 2.0 * (a.sqrt()).asin();

    let result = earth_radius * c;

    result
}

fn generate_entries(rng: &mut ChaCha8Rng, n: usize) -> (usize, f64, JsonStructure) {
    let mut sum = 0.0;
    let mut pairs = Vec::with_capacity(n);
    let n = n / 4;
    for _ in 0..4 {
        let cluster = generate_cluster(rng);
        for _ in 0..n {
            let p1 = generate_point_in_cluster(rng, cluster);
            let p2 = generate_point_in_cluster(rng, cluster);
            let haversine = reference_haversine(p1.x, p1.y, p2.x, p2.y, 6372.8);
            sum += haversine;
            pairs.push(JsonEntry {
                x0: p1.x,
                y0: p1.y,
                x1: p2.x,
                y1: p2.y,
            })
        }
    }
    (pairs.len(), sum, JsonStructure { pairs })
}

fn main() {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2);

    let (n, expected, json) = generate_entries(&mut rng, 10_000_000);

    let file = std::fs::File::options()
        .write(true)
        .truncate(true)
        .open("resultado.json")
        .unwrap();

    serde_json::to_writer(BufWriter::new(file), &json).unwrap();
    println!("Expected sum: {}", expected);
    println!("N: {}", n);
}
