// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;
mod file;
mod menu;

#[macro_use]
extern crate json;

use core::f64;
use std::{f64::consts::PI, sync::Mutex};
use file::{dump_route, read_route, write_to_file};
use menu::create_app_menu;
use num::ToPrimitive;
use tauri::{Manager, State};
use util::{dist_between, Movement};

struct CurrPos(Mutex<[f64; 3]>);
struct CtrlPointList(Mutex<Vec<Movement>>);
struct SelectedPoint(Mutex<(i32, i32)>);
struct StartPos(Mutex<[f64; 3]>);
struct StateId(Mutex<i32>);

pub const START_POS: [f64; 3] = [0.0,0.0,PI/2.0];

fn repopulate(mut control_points: Vec<Movement>, changed: (usize, usize), start_pos: [f64; 3]) -> (Vec<Movement>, [f64; 3]) {

    let mut curr_pos: [f64; 3] = start_pos.into(); // start current position at default start position

    for (i,mvmt) in control_points.iter_mut().enumerate() {
        if mvmt.ctrl.len() > 0 {
            let curr_j = mvmt.ctrl.len()-1; // last control point in route

            if changed.0 != i && changed.1 != 0 { // if the start position was not what was edited
                mvmt.ctrl[0] = curr_pos.into(); // change start position into current position
            }

            if mvmt.distance != 0.0 { // if movement is lateral
                mvmt.ctrl[curr_j] = (curr_pos[0] + mvmt.distance * curr_pos[2].cos(), curr_pos[1] + mvmt.distance * curr_pos[2].sin(), curr_pos[2]); // last (2nd) point is start po
            }
            else if !mvmt.angle.is_nan() { // if movement is turn
                mvmt.ctrl[curr_j] = mvmt.ctrl[0]; // set position to be the same
                mvmt.ctrl[curr_j].2 = mvmt.angle; // end angle is movement angle
            }
            else if mvmt.name.len() > 0 { // if movement is command
                mvmt.ctrl[0] = curr_pos.into(); // set position to be the same as the start position
                mvmt.ctrl[curr_j] = mvmt.ctrl[0]; // set the end to be same as start
            }

            if curr_j > 0 && mvmt.angle.is_nan() { // if movement is not an angle and has more than 1 control point
                mvmt.ctrl[curr_j].2 = 
                    (mvmt.ctrl[curr_j].1 - mvmt.ctrl[curr_j-1].1).atan2(mvmt.ctrl[curr_j].0 - mvmt.ctrl[curr_j-1].0); // set angle to arctangent of previous control points
                
                if mvmt.lookahead < 0.0 { // if backwards movement
                    mvmt.ctrl[curr_j].2 -= PI; // reverse the angle
                }
            }
            

            curr_pos = mvmt.ctrl[curr_j].into(); // update current position for further points
        }
    }

    return (control_points,curr_pos);
}

fn redraw(control_points: Vec<Movement>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    let mut output_xvals: Vec<i32> = vec![];
    let mut output_yvals: Vec<i32> = vec![];

    for mvmt in control_points.iter() {
        let mut n_times = 20; // number of points to calculate on the curve
        let points: &Vec<(f64, f64, f64)> = &mvmt.ctrl; // grab points
        if points.len() == 0 {
            continue;
        }
        if points.len() == 2 && mvmt.distance != 0.0 { // if lateral movement
            n_times = 3;
        }
        match util::bezier_curve(&points, n_times) { // generate bezier curves
            Ok((xvals, yvals)) => {
                // convert to pixels
                let mut new_x: Vec<i32>= xvals.iter().map(|x| util::inch_to_pixel_x(*x)).collect();
                let mut new_y: Vec<i32>= yvals.iter().map(|x| util::inch_to_pixel_y(*x)).collect();
                output_xvals.append(&mut new_x);
                output_yvals.append(&mut new_y);
            }
            Err(e) => {
                println!("Error generating Bezier curve: {}", e);
            }
        }
    }

    let mut output_control_points: (Vec<i32>, Vec<i32>) = (vec![], vec![]);

    // convert into input for frontend
    for group in control_points.iter() {
        if group.ctrl.len() == 0 {
            continue;
        }
        for pt in group.ctrl.iter() {
            output_control_points.0.push(util::inch_to_pixel_x(pt.0));
            output_control_points.1.push(util::inch_to_pixel_y(pt.1));
        }
    } 

    return ((output_xvals, output_yvals), output_control_points, control_points); // send to frontend
}

#[tauri::command]
fn change_start_pos(position: [f64; 3], state_start_pos: State<StartPos>, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    // unwrap start values
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();
    let mut start_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_start_pos.0.lock().unwrap();
    
    // repopulate with new start position
    let repopulate_val: (Vec<Movement>, [f64; 3]) = repopulate(new_mvmt_list, (0 as usize, 0 as usize), [position[0], position[1], position[2] * PI/180.0]);    
    new_mvmt_list = repopulate_val.0;

    // update state
    *curr_pos = repopulate_val.1.into();
    *start_pos = [position[0], position[1], position[2] * PI/180.0];
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec());
}

#[tauri::command]
fn move_ctrl(pos_x: i32, pos_y: i32, state_mvmt_list: State<CtrlPointList>, state_selected_pt: State<SelectedPoint>, state_curr_pos: State<CurrPos>, state_start_pos: State<StartPos>) -> (((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>), (f64,f64,f64)) {
    // get user mouse pos into inch position
    let ctrl_point_pos: [f64; 3] = [
        util::pixel_to_inch([pos_x, pos_y])[0],
        util::pixel_to_inch([pos_x, pos_y])[1],
        -1.0
    ];

    // unwrap state vars
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let start_pos: [f64; 3] = *state_start_pos.0.lock().unwrap();

    let selected_point = state_selected_pt.0.lock().unwrap();
    let new_selected_point = selected_point.to_owned();
    let i = new_selected_point.0;
    let j = new_selected_point.1;

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    if (i > -1) && (j > -1) { // make sure i and j are not set to none
        // may be redundant with the repopulate function - these if statements are the primitive version of that
        if i == (new_mvmt_list.len() - 1) as i32 && j == (new_mvmt_list[i as usize].ctrl.len() - 1) as i32 {
            let y_dif: f64 = new_mvmt_list[i as usize].ctrl[j as usize].1 - new_mvmt_list[(i) as usize as usize].ctrl[(j - 1) as usize].1;
            let x_dif: f64 = new_mvmt_list[i as usize].ctrl[j as usize].0 - new_mvmt_list[(i) as usize as usize].ctrl[(j - 1) as usize].0;

            *curr_pos = [ctrl_point_pos[0], ctrl_point_pos[1], y_dif.atan2(x_dif)];
        }
        else if j == (new_mvmt_list[i as usize].ctrl.len() - 1) as i32 && j < new_mvmt_list.len() as i32 {
            let mut curr_iterator = i + 1;
            while curr_iterator < (new_mvmt_list.len()-1) as i32 && new_mvmt_list[curr_iterator as usize].ctrl.len() == 0 {
                curr_iterator += 1;
            }
            if new_mvmt_list[curr_iterator as usize].ctrl.len() > 0 {
                new_mvmt_list[curr_iterator as usize].ctrl[0] = ctrl_point_pos.into();
            }
        }
        else if i > 0 && j == 0 {
            let mut curr_iterator = i - 1;
            while curr_iterator >= 0 && new_mvmt_list[curr_iterator as usize].ctrl.len() == 0 {
                curr_iterator -= 1;
            }
            let last_index = new_mvmt_list[curr_iterator as usize].ctrl.len() - 1;
            new_mvmt_list[curr_iterator as usize].ctrl[last_index] = ctrl_point_pos.into();
        }

        // set the selected point to mouse position
        new_mvmt_list[i as usize].ctrl[j as usize] = ctrl_point_pos.into();

        // repopulate the list with new position
        let repopulate_val: (Vec<Movement>, [f64; 3]) = repopulate(new_mvmt_list, (i as usize, j as usize), start_pos.into()); // not working because repopulate goes start to end -> changes the newly updated value

        // update state
        new_mvmt_list = repopulate_val.0;
        *curr_pos = repopulate_val.1.into();
        *mvmt_list = new_mvmt_list;

        return (redraw(mvmt_list.to_vec()), mvmt_list[i as usize].ctrl[j as usize]); // redraw on frontend
    }

    return (redraw(mvmt_list.to_vec()), (100.0, 100.0, -1.0)); // redraw anyway -  edited position is arbitrarily high
}

#[tauri::command]
fn select_ctrl(pos_x: i32, pos_y: i32, state_mvmt_list: State<CtrlPointList>, state_selected_pt: State<SelectedPoint>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    // unwrap state
    let ctrl_point_pos: [f64; 2] = util::pixel_to_inch([pos_x, pos_y]);

    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let mut selected_point = state_selected_pt.0.lock().unwrap();

    // find the movement
    for (i, list) in new_mvmt_list.iter_mut().enumerate() {
        if !(list.distance != 0.0 || !list.angle.is_nan() || list.name.len() > 0) {
            for (j, pt) in list.ctrl.iter_mut().enumerate() {
                if (pt.0 == ctrl_point_pos[0] && pt.1 == ctrl_point_pos[1]) || dist_between((pt.0, pt.1), ctrl_point_pos.into()) < 3.0 { // account for pixel errors
                    *selected_point = (i.try_into().unwrap(), j.try_into().unwrap()); // select point

                    // do not break so it selects the earliest possible occurence of the point
                }
            }
        }
    }

    *mvmt_list = new_mvmt_list; // update state

    return redraw(mvmt_list.to_vec()); // redraw on the rfontend
}

#[tauri::command]
fn deselect_ctrl(state_selected_pt: State<SelectedPoint>) {
    // unwrap state
    let mut selected_point = state_selected_pt.0.lock().unwrap();
    *selected_point = (-1, -1); // set selected point to invalid point
}

#[tauri::command]
fn change_mvmt(endtol: f64, timeout: i32, specific: String, id: i32, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_start_pos: State<StartPos>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();
    let start_pos: [f64; 3] = *state_start_pos.0.lock().unwrap();

    let mut selected_i: Option<usize> = None;

    for (i, list) in new_mvmt_list.iter_mut().enumerate() { // iterate through ids
        if list.id == id { // find the selected id

            // change the values of the selected movement
            list.end_tol = endtol;
            list.timeout = timeout;

            if !list.angle.is_nan() {
                list.angle = specific.parse::<f64>().unwrap() * PI / 180.0;
            }
            else if list.distance != 0.0 {
                list.distance = specific.parse().unwrap();
            }
            else if list.name.len() > 0 {
                list.name = specific;
            }
            else {
                list.lookahead = specific.parse().unwrap()
            }
            selected_i = Some(i); // find the index
            break;
        }
    }

    if selected_i.is_none() { // ensure i exists
        return redraw(mvmt_list.to_vec()); // redraw with no change
    }


    // repopulate with changed value
    let repopulate_val: (Vec<Movement>, [f64; 3]) = repopulate(new_mvmt_list, (selected_i.unwrap(), 0 as usize), start_pos.into());    
    new_mvmt_list = repopulate_val.0;

    // Recurse twice in order to smooth out any temporary issues
    let repopulate_val: (Vec<Movement>, [f64; 3]) = repopulate(new_mvmt_list, (0 as usize, 0 as usize), start_pos.into());    
    new_mvmt_list = repopulate_val.0;

    // update state
    *curr_pos = repopulate_val.1.into();
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec()); // redraw frontend
}

#[tauri::command]
fn delete_last(state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_start_pos: State<StartPos>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {

    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    if new_mvmt_list.len() == 0 {
        return redraw(mvmt_list.to_vec()); // ensure that we are not deleting from a list with no movements
    }

    let mut end_curr_pos: [f64; 3] = [-1.0, -1.0, 0.0];
    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();
    let start_pos: [f64; 3] = *state_start_pos.0.lock().unwrap();

    // delete last index
    new_mvmt_list.remove(new_mvmt_list.len()-1);

    if new_mvmt_list.len() == 0 { // no need to find current position b/c empty list
        *curr_pos = start_pos;
        *mvmt_list = new_mvmt_list;
        return redraw(mvmt_list.to_vec());
    }

    let i: usize = new_mvmt_list.len()-1;

    while end_curr_pos == [-1.0, -1.0, 0.0] { // iterate until valid
        if new_mvmt_list[i].ctrl.len() > 0 { // if movement has control points

            // use control point angle to find the end position
            let j: usize = new_mvmt_list[i].ctrl.len();

            let y_dif: f64 = new_mvmt_list[i].ctrl[j-1].1 - new_mvmt_list[i].ctrl[j - 2].1;
            let x_dif: f64 = new_mvmt_list[i].ctrl[j-1].0 - new_mvmt_list[i].ctrl[j - 2].0;

            end_curr_pos = [
                new_mvmt_list[i].ctrl[new_mvmt_list[new_mvmt_list.len()-1].ctrl.len() - 1].0,
                new_mvmt_list[i].ctrl[new_mvmt_list[new_mvmt_list.len()-1].ctrl.len() - 1].1,
                y_dif.atan2(x_dif)
            ];

            break;
        }
    }

    // update state
    *curr_pos = end_curr_pos;
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec());
}

#[tauri::command]
fn click(pos_x: i32, pos_y: i32, state_curr_pos: State<CurrPos>, state_mvmt_list: State<CtrlPointList>, state_curr_id: State<StateId>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    let field_pos = util::pixel_to_inch([pos_x, pos_y]);

    // unwrap state vars
    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    // used for arctan
    let y_dif: f64 = field_pos[0] - curr_pos[0];
    let x_dif: f64 = field_pos[1] - field_pos[1];

    // easy way of getting control points - user can move but these work technically
    let ctrl_float: (Vec<f64>, Vec<f64>, Vec<f64>) = (
        vec![curr_pos[0], curr_pos[0], field_pos[0]], 
        vec![curr_pos[1], field_pos[1], field_pos[1]],
        vec![curr_pos[2], -1.0, y_dif.atan2(x_dif)] // calculate angle between the two points
    );

    *curr_pos = [field_pos[0], field_pos[1], y_dif.atan2(x_dif)];

    let mut state_id = state_curr_id.0.lock().unwrap();
    let id = state_id.to_i32();
    
    *state_id = id.expect("State id is not an id") + 1; // increment state ID

    let mut new_mvmt: Movement = Movement::new( // to house new mvmt
        vec![],
        0.0, f64::NAN, 2.0, 5000, "".to_string(), 18.0, id.expect("State id is not an id")
    );

    for (i, _) in ctrl_float.0.iter().enumerate() {
        new_mvmt.ctrl.push((ctrl_float.0[i], ctrl_float.1[i], ctrl_float.2[i])); // add generated control points
    }

    new_mvmt_list.push(new_mvmt); // add to state list

    *mvmt_list = new_mvmt_list; // update state

    return redraw(mvmt_list.to_vec()); // redraw for frontend
}

#[tauri::command]
fn add_lateral_movement(distance: f64, endtol: f64, timeout: i32, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_curr_id: State<StateId>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {

    println!("{:?} {:?} {:?}", distance, endtol, timeout);

    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    let start_pt: (f64, f64, f64) = (curr_pos[0], curr_pos[1], curr_pos[2]);
    let end_pt: (f64, f64, f64) = (curr_pos[0] + distance * curr_pos[2].cos(), curr_pos[1] + distance * curr_pos[2].sin(), curr_pos[2]);

    // update curr pos
    *curr_pos = [end_pt.0, end_pt.1, curr_pos[2]];

    let mut state_id = state_curr_id.0.lock().unwrap();
    let id = state_id.to_i32();
    
    *state_id = id.expect("State id is not an id") + 1;

    // create new movement
    let new_mvmt: Movement = Movement::new(
        vec![start_pt, end_pt],
        distance, f64::NAN, endtol, timeout, "".to_string(), 0.0, id.expect("State id is not an id")
    );

    // add to movement list
    new_mvmt_list.push(new_mvmt);

    // update state
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec()); // redraw frontend
}

#[tauri::command]
fn add_turn_movement(angle: f64, endtol: f64, timeout: i32, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_curr_id: State<StateId>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {

    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    let mut state_id = state_curr_id.0.lock().unwrap();
    let id = state_id.to_i32();
    
    *state_id = id.expect("State id is not an id") + 1;

    // create the new movement
    let new_mvmt: Movement = Movement::new(
        vec![(curr_pos[0], curr_pos[1], angle * (PI/180.0)), (curr_pos[0], curr_pos[1], angle * (PI/180.0))],
        0.0, angle * (PI/180.0), endtol, timeout, "".to_string(), 0.0, id.expect("State id is not an id")
    );
    
    // add movement to list
    new_mvmt_list.push(new_mvmt);

    // update state
    *curr_pos = [curr_pos[0], curr_pos[1], angle * (PI/180.0)];
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec()); // redraw frontend
}

#[tauri::command]
fn add_command(name: String, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_curr_id: State<StateId>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>){
    
    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut new_mvmt_list = mvmt_list.to_vec();

    let curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    let mut state_id = state_curr_id.0.lock().unwrap();
    let id = state_id.to_i32();
    
    *state_id = id.expect("State id is not an id") + 1;

    // create new movement
    let new_mvmt: Movement = Movement::new(
        vec![(curr_pos[0], curr_pos[1], curr_pos[2]), (curr_pos[0], curr_pos[1], curr_pos[2])],
        0.0, f64::NAN, 0.0, 0, name.replace('"', ""), 0.0, id.expect("State id is not an id")
    );

    // add movement to list
    new_mvmt_list.push(new_mvmt);

    // update state
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec()); // redraw frontend
}

#[tauri::command]
fn clear(state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_curr_id: State<StateId>, state_start_pos: State<StartPos>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {
    // unwrap state
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();

    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();
    let start_pos: [f64; 3] = *state_start_pos.0.lock().unwrap();

    // clear state
    *mvmt_list = vec![];
    *curr_pos = start_pos;

    let mut state_id = state_curr_id.0.lock().unwrap();
    *state_id = 0;

    return redraw(mvmt_list.to_vec()); // redraw frontend
}

#[tauri::command]
fn save_file(state_mvmt_list: State<CtrlPointList>, path: String) {

    let mvmt_list = state_mvmt_list.0.lock().unwrap(); // get state

    dump_route(path, mvmt_list.to_vec()); // pass to save file with path
}

#[tauri::command]
fn load_file(path: String, state_mvmt_list: State<CtrlPointList>, state_curr_pos: State<CurrPos>, state_curr_id: State<StateId>, state_start_pos: State<StartPos>) -> ((Vec<i32>, Vec<i32>), (Vec<i32>, Vec<i32>), Vec<Movement>) {

    let mut new_mvmt_list = read_route(path).unwrap(); // get value rfom file
    let mut start_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_start_pos.0.lock().unwrap(); // get state

    if (new_mvmt_list.len() > 0) {
        *start_pos = new_mvmt_list[0].ctrl[0].into();
    }

    // refresh current movement ID
    let mut curr_max_id = 0;

    for list in new_mvmt_list.iter() {
        curr_max_id = std::cmp::max(list.id, curr_max_id);
    }

    let mut state_id = state_curr_id.0.lock().unwrap();
    *state_id = curr_max_id;

    // recurse over values to ensure correct
    let repopulate_val: (Vec<Movement>, [f64; 3]) = repopulate(new_mvmt_list, (0 as usize, 0 as usize), *start_pos);    
    new_mvmt_list = repopulate_val.0;

    // update current movement list and position
    let mut mvmt_list = state_mvmt_list.0.lock().unwrap();
    let mut curr_pos: std::sync::MutexGuard<'_, [f64; 3]> = state_curr_pos.0.lock().unwrap();

    *curr_pos = repopulate_val.1.into();
    *mvmt_list = new_mvmt_list;

    return redraw(mvmt_list.to_vec());
}

#[tauri::command]
fn export_file(path: String, state_mvmt_list: State<CtrlPointList>) {

    let mvmt_list = state_mvmt_list.0.lock().unwrap();

    let mut print_string: String = "".to_owned(); // string to store print

    for j in 0..mvmt_list.len() { 
        if mvmt_list[j].ctrl.len() > 2 { // if follow movement
            let start_string: String = format!("rs\n{:?} {:?} {:?}\n", mvmt_list[j].end_tol, mvmt_list[j].timeout, mvmt_list[j].lookahead); // start follow route
            print_string.push_str(&start_string);
            match util::bezier_curve(&(mvmt_list[j].ctrl), 20) { // generate bezier curve
                Ok((xvals, yvals)) => {
                    let output_x: Vec<f64>= xvals.into_iter().collect();
                    let output_y: Vec<f64>= yvals.into_iter().collect();

                    for i in 0..output_x.len() {
                        let add_string: String = format!("p {} {}\n", output_x[i], output_y[i]).to_owned(); // write each individual point
                        print_string.push_str(&add_string); // add to write string
                    }
                }
                Err(e) => {
                    println!("Error generating Bezier curve: {}", e);
                }
            }
            let start_string: &str = "re\n"; // end follow route
            print_string.push_str(start_string); // add to write string
        }
        else if mvmt_list[j].distance != 0.0 { //  lateral movement
            let start_string: String = format!("ps\n{:?} {:?} {:?}\n", mvmt_list[j].distance, mvmt_list[j].end_tol, mvmt_list[j].timeout); // lateral start
            print_string.push_str(&start_string); // add to write string
        }
        else if mvmt_list[j].name != "".to_string() {
            let start_string: String = format!("cs\n{}\n", mvmt_list[j].name); // command start
            print_string.push_str(&start_string); // add to write string
        }
        else {
            let start_string: String = format!("ts\n{:?} {:?} {:?}\n", mvmt_list[j].angle, mvmt_list[j].end_tol * (PI/180.0), mvmt_list[j].timeout); // turn start
            print_string.push_str(&start_string); // add to write string
        }

    }
    let start_string: String = format!("eof\n"); // end of file o reader knows when to stop
    print_string.push_str(&start_string);

    let _ = file::write_to_file(path, print_string);
}

#[tauri::command]
fn export_movement(path: String, id: i32, state_mvmt_list: State<CtrlPointList>) {
    let mvmt_list = state_mvmt_list.0.lock().unwrap().to_vec();

    for mvmt in mvmt_list { // iterate over all movements
        if mvmt.id == id && mvmt.ctrl.len() > 2 { // find the current selected ID

            let mut print_string = "".to_owned(); // string to write

            match util::bezier_curve(&mvmt.ctrl, 10) {
                Ok((xvals, yvals)) => { // generate bezier curve
                    let output_x: Vec<f64>= xvals.into_iter().collect();
                    let output_y: Vec<f64>= yvals.into_iter().collect();


                    for i in 0..output_x.len() {
                        let add_string: String = format!("{} {}\n", output_x[i], output_y[i]).to_owned(); // strng for each point
                        print_string.push_str(&add_string); // add string to write string
                    }
                }
                Err(e) => {
                    println!("Error generating Bezier curve: {}", e);
                }
            }

            let _ = write_to_file(path, print_string); // write to file, save non used variable

            break; // end iteration
        }
    }
}

fn main() {
    let menu = create_app_menu();
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            let window_ = window.clone();
            window.on_menu_event(move |event| {
                match event.menu_item_id() {
                    "export" => {
                        window_.emit("export-backend", "").unwrap();
                    }
                    "new" => {
                        window_.emit("new-backend", "").unwrap();
                    }
                    "save" => {
                        window_.emit("save-backend", "").unwrap();
                    }
                    "load" => {
                        window_.emit("load-backend", "").unwrap();
                    }
                    "change_field" => {
                        window_.emit("change_field-backend", "").unwrap();
                    }
                    "export_mvmt" => {
                        window_.emit("export_mvmt-backend", "").unwrap();
                    }
                    _ => {}
                }
            });
            Ok(())
        })
        .menu(menu)
        .manage(CurrPos(START_POS.into()))
        .manage(CtrlPointList(vec![].into()))
        .manage(SelectedPoint((-1,-1).into()))
        .manage(StateId(1.into()))
        .manage(StartPos(START_POS.into()))
        .invoke_handler(tauri::generate_handler![
            click, select_ctrl, move_ctrl, deselect_ctrl, export_file, add_lateral_movement, 
            add_turn_movement, add_command, delete_last, change_mvmt, clear, save_file, load_file,
            change_start_pos, export_movement
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
