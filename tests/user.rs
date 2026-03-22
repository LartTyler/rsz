use assert_matches::assert_matches;
use rsz::layout::LayoutMap;
use rsz::rsz::content::SliceStream;
use rsz::{Field, Object, User, Value, Values};
use std::rc::Rc;
use uuid::uuid;

const INPUT: &'static [u8] = include_bytes!("assets/sample.user.3");
const LAYOUT: &'static str = include_str!("assets/sample.user.3.layout.json");

#[test]
fn test_user() {
    env_logger::init();

    let layouts = LayoutMap::from_json(LAYOUT).unwrap();
    let doc = User::parse(&mut SliceStream::from(INPUT), &layouts).unwrap();

    assert_eq!(doc.content.root_objects.len(), 1);

    let root = doc.content.root_objects[0].clone();
    assert_matches!(root.name.as_ref(), "app.user_data.AccessoryData");
    assert_eq!(root.fields.len(), 1);

    let field = root.fields[0].clone();
    assert_matches!(field.name.as_ref(), "_Values");
    assert_matches!(field.value, Value::Array(_));

    let first = field.value.try_as_array().unwrap();
    let first = first[0].clone();
    assert_matches!(first, Value::Object(ref v) if v.name == "app.user_data.AccessoryData.cData" && v.fields.len() == 12);

    let expected = vec![
        // region Expected Fields
        Field {
            name: "_Index".to_owned(),
            value: Value::S32(0),
        },
        Field {
            name: "_AccessoryId".to_owned(),
            value: Value::Object(Rc::new(Object {
                name: "app.EquipDef.ACCESSORY_ID_Serializable".to_owned(),
                fields: vec![Field {
                    name: "_Value".to_owned(),
                    value: Value::S32(130724024),
                }],
            })),
        },
        Field {
            name: "_Name".to_owned(),
            value: Value::Guid(uuid!("56bff7fd-6ef8-4038-9ce7-006a0fdc88a1")),
        },
        Field {
            name: "_Explain".to_owned(),
            value: Value::Guid(uuid!("abb148ea-836c-4a2a-9cb8-5b74ab3c845c")),
        },
        Field {
            name: "_AccessoryType".to_owned(),
            value: Value::Object(Rc::new(Object {
                name: "app.EquipDef.ACCESSORY_TYPE_Serializable".to_owned(),
                fields: vec![Field {
                    name: "_Value".to_owned(),
                    value: Value::S32(-1638455296),
                }],
            })),
        },
        Field {
            name: "_SortId".to_owned(),
            value: Value::U32(101),
        },
        Field {
            name: "_Rare".to_owned(),
            value: Value::Object(Rc::new(Object {
                name: "app.ItemDef.RARE_Serializable".to_owned(),
                fields: vec![Field {
                    name: "_Value".to_owned(),
                    value: Value::S32(16),
                }],
            })),
        },
        Field {
            name: "_IconColor".to_owned(),
            value: Value::S32(4),
        },
        Field {
            name: "_Price".to_owned(),
            value: Value::U32(150),
        },
        Field {
            name: "_SlotLevelAcc".to_owned(),
            value: Value::Object(Rc::new(Object {
                name: "app.EquipDef.SlotLevel_Serializable".to_owned(),
                fields: vec![Field {
                    name: "_Value".to_owned(),
                    value: Value::S32(1),
                }],
            })),
        },
        Field {
            name: "_Skill".to_owned(),
            value: Value::Array(Values(vec![
                Value::Object(Rc::new(Object {
                    name: "app.HunterDef.Skill_Serializable".to_owned(),
                    fields: vec![Field {
                        name: "_Value".to_owned(),
                        value: Value::S32(1),
                    }],
                })),
                Value::Object(Rc::new(Object {
                    name: "app.HunterDef.Skill_Serializable".to_owned(),
                    fields: vec![Field {
                        name: "_Value".to_owned(),
                        value: Value::S32(0),
                    }],
                })),
            ])),
        },
        Field {
            name: "_SkillLevel".to_owned(),
            value: Value::Array(Values(vec![Value::U32(1), Value::U32(0)])),
        },
        // endregion
    ];

    for (index, field) in first.try_as_object().unwrap().fields.iter().enumerate() {
        assert_eq!(field, &expected[index]);
    }
}
