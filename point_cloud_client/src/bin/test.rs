use clap::Clap;
use nalgebra::Point3;
use point_cloud_client::PointCloudClientBuilder;
use point_viewer::errors::{ErrorKind, Result};
use point_viewer::geometry::Aabb;
use point_viewer::iterator::{PointLocation, PointQuery};
use point_viewer::PointsBatch;

// size for batch
const BATCH_SIZE: usize = 1_000_000;

fn point3f64_from_str(s: &str) -> std::result::Result<Point3<f64>, &'static str> {
    let coords: std::result::Result<Vec<f64>, &'static str> = s
        .split(|c| c == ' ' || c == ',' || c == ';')
        .map(|s| s.parse::<f64>().map_err(|_| "Could not parse point."))
        .collect();
    let coords = coords?;
    if coords.len() != 3 {
        return Err("Wrong number of coordinates.");
    }
    Ok(Point3::new(coords[0], coords[1], coords[2]))
}

#[derive(Clap)]
#[clap(about = "Simple point_cloud_client test.")]
struct CommandlineArguments {
    /// The locations containing the octree data.
    #[clap(parse(from_str), required = true)]
    locations: Vec<String>,

    /// The minimum value of the bounding box to be queried.
    #[clap(
        long,
        default_value = "-500.0 -500.0 -500.0",
        parse(try_from_str = point3f64_from_str)
    )]
    min: Point3<f64>,

    /// The maximum value of the bounding box to be queried.
    #[clap(
        long,
        default_value = "500.0 500.0 500.0",
        parse(try_from_str = point3f64_from_str)
    )]
    max: Point3<f64>,

    /// The maximum number of points to return.
    #[clap(long, default_value = "50000000")]
    num_points: usize,

    /// The maximum number of threads to be running.
    #[clap(long, default_value = "30")]
    num_threads: usize,

    /// The maximum number of points sent through batch.
    #[clap(long, default_value = "500000")]
    batch_size: usize,
}

fn main() {
    let args = CommandlineArguments::parse();
    let num_points = args.num_points;
    let point_cloud_client = PointCloudClientBuilder::new(&args.locations)
        .num_threads(args.num_threads)
        .num_points_per_batch(args.batch_size)
        .build()
        .expect("Couldn't create point cloud client.");

    let point_location = PointQuery {
        attributes: vec!["color", "intensity"],
        location: PointLocation::Aabb(Aabb::new(args.min, args.max)),
        ..Default::default()
    };
    let mut point_count: usize = 0;
    let mut print_count: usize = 1;
    let callback_func = |points_batch: PointsBatch| -> Result<()> {
        point_count += points_batch.position.len();
        if point_count >= print_count * BATCH_SIZE {
            print_count += 1;
            eprintln!("Streamed {}M points", point_count / BATCH_SIZE);
        }
        if point_count >= num_points {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                format!("Maximum number of {} points reached.", num_points),
            )
            .into());
        }
        Ok(())
    };
    match point_cloud_client.for_each_point_data(&point_location, callback_func) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::Io(ref e) if e.kind() == std::io::ErrorKind::Interrupted => (),
            _ => {
                eprintln!("Encountered error:\n{}", e);
                std::process::exit(1);
            }
        },
    }
}
