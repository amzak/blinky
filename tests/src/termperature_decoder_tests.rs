#[test]
fn should_decode_23_degrees() {
    let data: u8 = 0x00;

    let result = decode_temperature(data);

    assert_eq!(result, 23);
}

#[test]
fn should_decode_150_degrees() {
    let data: u8 = 0x7f;

    let result = decode_temperature(data);

    assert_eq!(result, 150);
}

#[test]
fn should_decode_minus_104_degrees() {
    let data: u8 = 0x81;

    let result = decode_temperature(data);

    assert_eq!(result, -104);
}

fn decode_temperature(data: u8) -> i32 {
    const BMA4_SCALE_TEMP: i32 = 1000;
    const BMA4_OFFSET_TEMP: i32 = 23;

    let tmpr: i32 = if (data / 128) > 0 {
        -1 * !(data - 1) as i32
    } else {
        data.into()
    };

    let tempr_raw = (tmpr + BMA4_OFFSET_TEMP) * BMA4_SCALE_TEMP;

    return tempr_raw / BMA4_SCALE_TEMP;
}
