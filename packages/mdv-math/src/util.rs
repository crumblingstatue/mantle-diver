use num_traits::{Bounded, Num, Signed};

/// Checks if `x` and `y` are within the circle defined by `cx, cy, radius`
///
/// It requires i64 due to the nature of the calculation,
/// which would cause underflow for unsigned numbers, and large distance between two points
/// (like camera being very far away from player) would cause overflow for squaring.
pub fn point_within_circle(cx: i64, cy: i64, radius: i64, x: i64, y: i64) -> bool {
    let distance_squared = (x - cx).pow(2) + (y - cy).pow(2);
    let radius_squared = radius.pow(2);
    distance_squared <= radius_squared
}

/// Returns (hspeed, vspeed) necessary to move from (src_x, src_y) towards (dst_x, dst_y)
/// at `speed`.
pub fn move_towards_hspeed_vspeed(
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    speed: f32,
) -> (f32, f32) {
    let dx = dst_x - src_x;
    let dy = dst_y - src_y;

    let distance = ((dx * dx + dy * dy) as f32).sqrt();
    if distance == 0. {
        return (0., 0.);
    }
    let hspeed = speed * dx as f32 / distance;
    let vspeed = speed * dy as f32 / distance;

    (hspeed, vspeed)
}

pub fn step_towards(current: &mut f32, desired: f32, step: f32) {
    let diff = desired - *current; // Calculate the difference between current and desired values
    let direction = diff.signum(); // Get the sign of the difference to know which way to move

    if diff.abs() < step {
        // Check if the difference is less than the closeness threshold
        *current = desired; // If it is, set current to desired and return
        return;
    }

    let step = step * direction; // Multiply the step by the sign to get the correct direction
    *current += step; // Add the step to the current value
}

/// A smooth triangle-wave like transform of the input value, oscillating between 0 and the ceiling.
pub fn smoothwave<T: Num + From<u8> + PartialOrd + Copy>(input: T, max: T) -> T {
    let period = max * T::from(2);
    let value = input % period;
    if value < max {
        value
    } else {
        period - value
    }
}

#[test]
fn test_smooth_wave() {
    assert_eq!(smoothwave(0, 100), 0);
    assert_eq!(smoothwave(50, 100), 50);
    assert_eq!(smoothwave(125, 100), 75);
    assert_eq!(smoothwave(150, 100), 50);
    assert_eq!(smoothwave(175, 100), 25);
    assert_eq!(smoothwave(199, 100), 1);
    assert_eq!(smoothwave(200, 100), 0);
    assert_eq!(smoothwave(201, 100), 1);
}

/// Get the offset required to center an object of `xw` width inside an object of `yw` width.
///
/// For example, let's say `xw` (+) is 10 and we want to center it inside `yw` (-), which is 20
///
/// ++++++++++           (x uncentered)
/// -------------------- (y)
///      ++++++++++      (x centered)
///
/// In this case, we needed to add 5 to x to achieve centering.
/// This is the offset that this function calculates.
///
/// We can calulate it by subtracting `xw` from `yw` (10), and dividing it by 2.
pub fn center_offset<N: From<u8> + Copy + Signed>(xw: N, yw: N) -> N {
    let diff = yw - xw;
    diff / N::from(2)
}

pub fn min_max_clamp<T: Num + Bounded + PartialOrd>(val: &mut T, min_to_clamp: T, max_to_clamp: T) {
    if *val < min_to_clamp {
        *val = T::min_value();
    }
    if *val > max_to_clamp {
        *val = T::max_value();
    }
}
