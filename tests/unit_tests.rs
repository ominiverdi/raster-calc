// tests/unit_tests.rs
use raster_calc::utils::fixed_point::to_fixed_point;

#[test]
fn test_fixed_point_conversion() {
    let input = vec![0.0, 0.5, -0.5, 1.0, -1.0, -999.0];
    let scale = 10000;
    let nodata = -10000;
    
    let result = to_fixed_point(&input, scale, nodata);
    
    assert_eq!(result, vec![0, 5000, -5000, 9999, -9999, -10000]);
}