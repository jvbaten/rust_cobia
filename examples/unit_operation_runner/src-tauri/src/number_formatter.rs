/// Formats an optional floating-point value into a compact, human-readable string.
///
/// # Parameters
/// - `value`: `Some(f64)` to format, or `None` for an absent value.
///
/// # Returns
/// - `""` when `value` is `None`.
/// - `"N/A"` when the value is `NaN`.
/// - `"0"` when the value is exactly zero.
/// - Scientific notation (e.g. `"1.23e-5"`) for magnitudes outside `[1e-4, 1e4]`.
/// - Fixed-point notation with 4–9 decimal places otherwise.
///
/// Trailing zeros and unnecessary decimal points are stripped in both formats.
/// Scientific exponents preserve sign as returned by Rust's `{:.6e}` formatter.
pub fn format_number(value: Option<f64>) -> String {
    match value {
        Some(v) => {
            if f64::is_nan(v) {
                "N/A".to_string()
            } else if v == 0.0 {
                "0".to_string()
            } else {
                if (v < 1e-4 && v > -1e-4) || !(-1e4..=1e4).contains(&v) {
                    let fmt = format!("{:.6e}", v);
                    //check for dot
                    if fmt.contains('.') {
                        //remove trailing zeros before the e
                        let parts: Vec<&str> = fmt.split('e').collect();
                        let base = parts[0].trim_end_matches('0').trim_end_matches('.');
                        let exponent = parts[1];
                        format!("{}e{}", base, exponent)
                    } else {
                        fmt
                    }
                } else {
                    //figure out precision based on order of magnitude, to show 6 significant digits
                    let vabs = v.abs();
                    let mut digits = 3;
                    let mut order = 1e2;
                    while vabs < order {
                        digits += 1;
                        order *= 0.1;
                    }
                    let fmt = format!("{:.N$}", v, N = digits);
                    //check for dot
                    if fmt.contains('.') {
                        //remove trailing zeros
                        let fmt = fmt.trim_end_matches('0').trim_end_matches('.');
                        fmt.to_string()
                    } else {
                        fmt
                    }
                }
            }
        }
        None => "".to_string(),
    }
}
