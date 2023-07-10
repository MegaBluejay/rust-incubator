mod geo {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Clone, Debug)]
    pub struct Polyline(Vec<Point>);

    #[derive(Debug)]
    pub enum PolylineError {
        EmptyVec,
    }

    impl Polyline {
        pub fn new(points: Vec<Point>) -> Result<Self, PolylineError> {
            if points.is_empty() {
                Err(PolylineError::EmptyVec)
            } else {
                Ok(Self(points))
            }
        }
    }
}

use geo::*;

fn main() {
    let default_point = Point::default();
    let point = Point { x: 1, y: 2 };

    let line1 = Polyline::new(vec![default_point, point]).unwrap();

    // still accesible, since Point is Copy
    println!("{:?}", default_point);

    println!("{:?}, {:?}", line1, line1.clone());
}
