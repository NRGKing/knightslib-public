use json::JsonValue;
use ndarray::{Array1, Array};
use serde::{Deserialize, Serialize};

pub const FIELD_SIZE: f64 = 468.0;
pub const TILE: f64 = FIELD_SIZE / 6.0;

pub const ORIGIN_X: f64 = 638.0;
pub const ORIGIN_Y: f64 = 339.0;

pub fn pixel_to_inch(pixel_pos: [i32; 2]) -> [f64; 2] {
    let x: f64 = pixel_pos[0] as f64; let y: f64 = pixel_pos[1] as f64;
    return [
        // (((pixel_pos[0] - X_MARGIN)/ TILE) * 24 - (FIELD_SIZE / (2*TILE)) * 24), 
        // -(((pixel_pos[1] - Y_MARGIN)/ TILE) * 24 - (FIELD_SIZE / (2*TILE)) * 24)
        ((x - ORIGIN_X)/ TILE) * 24.0,
        -((y - ORIGIN_Y)/ TILE) * 24.0
    ];
}

pub fn dist_between(pt1: (f64, f64), pt2: (f64, f64)) -> f64 {
    return ((pt2.0 - pt1.0 ).powf(2.0) + (pt2.1 - pt1.1).powf(2.0)).sqrt();
}

pub fn inch_to_pixel(inch_pos: [f64; 2]) -> [i32; 2] {
    let x: f64 = inch_pos[0]/24.0 * TILE + ORIGIN_X;
    let y: f64 = -inch_pos[1]/24.0 * TILE + ORIGIN_Y;

    return [x as i32, y as i32];
}

pub fn inch_to_pixel_x(inch_pos: f64) -> i32 {
    let x: f64 = inch_pos/24.0 * TILE + ORIGIN_X;
    return x as i32;
}

pub fn inch_to_pixel_y(inch_pos: f64) -> i32 {
    let y: f64 = -inch_pos/24.0 * TILE + ORIGIN_Y;
    return y as i32;
}


fn factorial(n: usize) -> f64 {
    (1..=n).map(|x| x as f64).product()
}

fn binomial(n: usize, k: usize) -> f64 {
    factorial(n) / (factorial(k) * factorial(n - k))
}

fn bernstein_poly(i: usize, n: usize, t: &Array1<f64>) -> Array1<f64> {
    t.mapv(|t_val| binomial(n, i) * (t_val.powf(i as f64)) * (1.0 - t_val).powf((n - i) as f64))
}

pub fn bezier_curve(points: &[(f64, f64, f64)], n_times: usize) -> Result<(Array1<f64>, Array1<f64>), String> {
    let n_points = points.len();

    let x_points: Array1<f64> = Array::from(points.iter().map(|&(x, _, _)| x).collect::<Vec<f64>>());
    let y_points: Array1<f64> = Array::from(points.iter().map(|&(_, y, _)| y).collect::<Vec<f64>>());

    // generate t values from 0 to 1
    let t = Array::linspace(0.0, 1.0, n_times);

    // create result arrays
    let mut xvals = Array1::<f64>::zeros(n_times);
    let mut yvals = Array1::<f64>::zeros(n_times);

    // calculate Bezier curve points
    for i in 0..n_points {
        let b_poly = bernstein_poly(i, n_points - 1, &t);
        xvals = xvals + *x_points.get(i).unwrap() * &b_poly;
        yvals = yvals + *y_points.get(i).unwrap() * &b_poly;
    }

    Ok((xvals, yvals))
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Movement {
    pub ctrl: Vec<(f64, f64, f64)>,
    pub distance: f64,
    pub angle: f64,
    pub end_tol: f64,
    pub timeout: i32,
    pub lookahead: f64,
    pub name: String,
    pub id: i32,
}

impl Movement {
    pub fn new(ctrl: Vec<(f64, f64, f64)>, distance: f64, angle: f64, end_tol: f64, timeout: i32, name: String, lookahead: f64, id: i32) -> Movement {
        Movement {
            ctrl: ctrl,
            distance: distance,
            angle: angle,
            end_tol: end_tol,
            timeout: timeout,
            lookahead: lookahead,
            name: name,
            id: id
        }
    }

    pub fn to_json(&self) -> JsonValue {

        let points_json: Vec<JsonValue> = self.ctrl.iter().map(|(x, y, z)| {
            JsonValue::Array(vec![JsonValue::from(*x), JsonValue::from(*y), JsonValue::from(*z)])
        }).collect();

        object! {
            "ctrl" => points_json,
            "distance" => self.distance,
            "angle" => self.angle,
            "end_tol" => self.end_tol,
            "timeout" => self.timeout,
            "lookahead" => self.lookahead,
            "name" => self.name.clone(),
            "id" => self.id,
        }
    }

    pub fn from_json(json: &JsonValue) -> Result<Movement, String> {
        let ctrl = match &json["ctrl"] {
            JsonValue::Array(arr) => arr.iter()
                .map(|item| {
                    match item {
                        JsonValue::Array(inner) if inner.len() == 3 => Ok((
                            inner[0].as_f64().ok_or("Invalid value for x")?,
                            inner[1].as_f64().ok_or("Invalid value for y")?,
                            inner[2].as_f64().ok_or("Invalid value for z")?
                        )),
                        _ => Err("Invalid format for control points".to_string())
                    }
                })
                .collect::<Result<Vec<(f64, f64, f64)>, String>>()?,
            _ => return Err("Missing or invalid control points".to_string()),
        };

        let distance = json["distance"].as_f64()
            .ok_or("Missing or invalid distance")?;

        let angle = json["angle"].as_f64()
            .ok_or("Missing or invalid angle")?;

        let end_tol = json["end_tol"].as_f64()
            .ok_or("Missing or invalid end tolerance")?;

        let timeout = json["timeout"].as_i32()
            .ok_or("Missing or invalid timeout")?;

        let lookahead = json["lookahead"].as_f64()
            .ok_or("Missing or invalid lookahead distance")?;

        let name = json["name"].as_str()
            .ok_or("Missing or invalid name")?
            .to_string();

        let id = json["id"].as_i32()
            .ok_or("Missing or invalid id")?;

        Ok(Movement {
            ctrl,
            distance,
            angle,
            end_tol,
            timeout,
            lookahead,
            name,
            id,
        })
    }

    pub fn from_json_array(json_array: &JsonValue) -> Result<Vec<Movement>, String> {
        match json_array {
            JsonValue::Array(arr) => arr.iter()
                .map(Movement::from_json)
                .collect(),
            _ => Err("Expected a JSON array".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MovementWrapper {
    pub list: Vec<Movement>
}