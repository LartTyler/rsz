use assert_matches::assert_matches;
use half::f16;
use uuid::Uuid;
use rsz::layout::{FieldKind, FieldLayout};
use rsz::rsz::content::{ParseField, RszStream, SliceStream};
use rsz::{Content, Field, Value};

#[test]
fn parse_boolean() {
    let value = parse_test_value(&[1], mock_field_layout(FieldKind::Boolean, 1, 1));
    assert_matches!(value, Value::Boolean(true))
}

#[test]
fn parse_f16() {
    let pi = f16::from_f32(std::f32::consts::PI);
    let data = pi.to_le_bytes();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::F16, 4, 2));
    assert_matches!(value, Value::F16(v) if v == pi);
}

#[test]
fn parse_f32() {
    let pi = std::f32::consts::PI;
    let data = pi.to_le_bytes();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::F32, 4, 4));
    assert_matches!(value, Value::F32(v) if v == pi);
}

#[test]
fn parse_f64() {
    let pi = std::f64::consts::PI;
    let data = pi.to_le_bytes();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::F64, 8, 8));
    assert_matches!(value, Value::F64(v) if v == pi);
}

#[test]
fn parse_guid() {
    let expected = Uuid::new_v4();
    let data = expected.to_bytes_le();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::Guid, 16, 16));
    assert_matches!(value, Value::Guid(v) if v == expected);
}

#[test]
fn parse_s8() {
    let data = (-121_i8).to_le_bytes();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::S8, 1, 1));
    assert_matches!(value, Value::S8(-121));
}

#[test]
fn parse_s16() {
    let data = (-2550_i16).to_le_bytes();

    let value = parse_test_value(&data, mock_field_layout(FieldKind::S16, 2, 2));
    assert_matches!(value, Value::S16(-2550));
}

fn parse_test_value(data: &[u8], field: FieldLayout<'_>) -> Value {
    let mut stream = SliceStream::from(data);
    let Field { value, .. } = field.parse(&mut stream, &Content::default()).unwrap();

    let actual = stream.position();
    assert_eq!(field.size, actual);

    value
}

fn mock_field_layout<'a>(kind: FieldKind, align: usize, size: usize) -> FieldLayout<'a> {
    FieldLayout {
        align,
        size,
        kind,
        name: "mock",
        is_array: false,
        is_native: false,
        original_type_name: "mock",
    }
}
