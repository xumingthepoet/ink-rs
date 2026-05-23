use std::collections::BTreeMap;

use serde_json::json;

use super::{program_from_value, program_to_value};
use crate::{
    Container, ControlCommand, DictKey, DictKeyType, DictValue, InterfaceDefinition,
    InterfaceMemberKind, InternalFunction, NativeFunction, Object, Program,
};

#[test]
fn writes_plain_text_story_json() {
    let program = Program::new(Container::unnamed(vec![Object::Container(
        Container::unnamed(vec![
            Object::String("Line.".to_string()),
            Object::String("\n".to_string()),
            Object::Container(Container::named(
                "g-0",
                vec![Object::ControlCommand(ControlCommand::NoOp)],
            )),
        ]),
    )]));

    assert_eq!(
        program.to_json_value(),
        json!({
            "root": [["^Line.", "\n", ["nop", {"#n": "g-0"}], null], null]
        })
    );
}

#[test]
fn roundtrips_named_content_and_command_tokens() {
    let input = json!({
        "root": [
            ["#", "^tag", "/#", {"->t->": "knot"}, {"#n": "g-0"}],
            "nop",
            {
                "knot": ["ev", "str", "^value", "/str", "/ev", null],
                "global decl": ["ev", 2, {"VAR=": "x"}, "/ev", null]
            }
        ]
    });

    let program = program_from_value(input.clone()).expect("format should parse");

    assert_eq!(program_to_value(&program), input);
}

#[test]
fn roundtrips_module_shaped_named_content_without_schema_changes() {
    let input = json!({
        "root": [
            {"->": "game.main"},
            {
                "game": [
                    {
                        "main": ["^Start", "\n", {"->": "support.helper"}, null]
                    }
                ],
                "support": [
                    {
                        "helper": ["^Support", "\n", null]
                    }
                ],
                "global decl": [
                    "ev",
                    1,
                    {"VAR=": "support::shown"},
                    "/ev",
                    null
                ]
            }
        ]
    });

    let program = program_from_value(input.clone()).expect("format should parse modules");

    assert_eq!(program_to_value(&program), input);
}

#[test]
fn roundtrips_internal_function_metadata() {
    let input = json!({
        "root": ["nop", null],
        "internalFunctions": {
            "game::read_config": {
                "path": "game.read_config",
                "args": 1,
                "argTypes": ["string"],
                "returnType": "string"
            }
        }
    });

    let program = program_from_value(input.clone()).expect("format should parse metadata");

    assert_eq!(program_to_value(&program), input);
    assert_eq!(
        program.internal_functions["game::read_config"],
        InternalFunction::new("game.read_config", vec!["string".to_string()], "string")
    );
}

#[test]
fn roundtrips_interface_metadata() {
    let input = json!({
        "root": ["nop", null],
        "interfaces": {
            "IItem": {
                "members": {
                    "score": "function",
                    "target": "knot"
                },
                "implementations": ["left", "right"]
            }
        }
    });

    let program = program_from_value(input.clone()).expect("format should parse interfaces");

    assert_eq!(program_to_value(&program), input);
    assert_eq!(
        program.interfaces["IItem"].members["target"],
        InterfaceMemberKind::Knot
    );
    assert_eq!(
        program.interfaces["IItem"].members["score"],
        InterfaceMemberKind::Function
    );
    assert_eq!(
        program.interfaces["IItem"].implementations,
        vec!["left".to_string(), "right".to_string()]
    );
}

#[test]
fn writes_interface_metadata_from_typed_model() {
    let mut members = BTreeMap::new();
    members.insert("target".to_string(), InterfaceMemberKind::Knot);
    members.insert("score".to_string(), InterfaceMemberKind::Function);

    let mut program = Program::new(Container::unnamed(vec![Object::ControlCommand(
        ControlCommand::NoOp,
    )]));
    program.interfaces.insert(
        "IItem".to_string(),
        InterfaceDefinition::new(members, vec!["left".to_string(), "right".to_string()]),
    );

    assert_eq!(
        program.to_json_value(),
        json!({
            "root": ["nop", null],
            "interfaces": {
                "IItem": {
                    "members": {
                        "score": "function",
                        "target": "knot"
                    },
                    "implementations": ["left", "right"]
                }
            }
        })
    );
}

#[test]
fn rejects_unknown_interface_member_kind() {
    let input = json!({
        "root": ["nop", null],
        "interfaces": {
            "IItem": {
                "members": {
                    "target": "room"
                },
                "implementations": ["left"]
            }
        }
    });

    let error = program_from_value(input).expect_err("metadata should be rejected");

    assert!(error
        .to_string()
        .contains("interface 'IItem' member 'target' has unsupported kind: room"));
}

#[test]
fn rejects_internal_function_metadata_with_mismatched_arg_count() {
    let input = json!({
        "root": ["nop", null],
        "internalFunctions": {
            "game::bad": {
                "path": "game.bad",
                "args": 2,
                "argTypes": ["int"],
                "returnType": "int"
            }
        }
    });

    let error = program_from_value(input).expect_err("metadata should be rejected");

    assert!(error
        .to_string()
        .contains("internal function 'game::bad' args does not match argTypes length"));
}

#[test]
fn native_function_tokens_roundtrip_as_typed_objects() {
    for function in NativeFunction::ALL {
        let token = function.token();
        let value = json!(token);

        assert_eq!(
            Object::from_json_value(value.clone()).unwrap(),
            Object::NativeFunction(function)
        );
        assert_eq!(Object::NativeFunction(function).to_json_value(), value);
    }
}

#[test]
fn rejects_unknown_native_function_tokens() {
    let error = Object::from_json_value(json!("UNKNOWN_NATIVE")).unwrap_err();

    assert_eq!(
        error.message(),
        "unsupported native function token: UNKNOWN_NATIVE"
    );
}

#[test]
fn rejects_removed_control_command_tokens() {
    for token in [
        "done",
        "end",
        "thread",
        "choiceCnt",
        "turn",
        "turns",
        "readc",
        "visit",
        "seq",
    ] {
        let error = Object::from_json_value(json!(token)).unwrap_err();

        assert!(
            error
                .message()
                .contains(&format!("unsupported native function token: {token}")),
            "unexpected error for {token}: {}",
            error.message()
        );
        assert!(
            ControlCommand::from_token(token).is_none(),
            "{token} should not be a current control command"
        );
    }
}

#[test]
fn rejects_removed_read_count_object() {
    let error = Object::from_json_value(json!({ "CNT?": "knot" })).unwrap_err();

    assert!(error
        .message()
        .contains("read-count object 'CNT?' is not supported"));
}

#[test]
fn rejects_removed_container_count_flags() {
    let error = Container::from_json_value(json!(["^Line.", {"#f": 1}]), None).unwrap_err();

    assert!(error
        .message()
        .contains("container count flags '#f' are not supported"));
}

#[test]
fn rejects_removed_choice_only_flag() {
    let error = Object::from_json_value(json!({ "*": "choice.target", "flg": 4 })).unwrap_err();

    assert!(error
        .message()
        .contains("choice point flags contain unsupported current-format bits"));
}

#[test]
fn roundtrips_dynamic_interface_instruction_objects() {
    let target = Object::DynamicInterfaceTarget {
        interface: "IItem".to_string(),
        member: "target".to_string(),
    };
    let target_value = target.to_json_value();

    assert_eq!(
        target_value,
        json!({
            "i->": "target",
            "interface": "IItem"
        })
    );
    assert_eq!(Object::from_json_value(target_value).unwrap(), target);

    let call = Object::DynamicInterfaceFunctionCall {
        interface: "IItem".to_string(),
        member: "score".to_string(),
        args: 2,
    };
    let call_value = call.to_json_value();

    assert_eq!(
        call_value,
        json!({
            "i()": "score",
            "interface": "IItem",
            "args": 2
        })
    );
    assert_eq!(Object::from_json_value(call_value).unwrap(), call);
}

#[test]
fn rejects_dynamic_interface_function_without_arg_count() {
    let error = Object::from_json_value(json!({
        "i()": "score",
        "interface": "IItem"
    }))
    .unwrap_err();

    assert_eq!(error.message(), "compiled story JSON is missing args");
}

#[test]
fn roundtrips_dynamic_array_values() {
    let object = Object::ValueArray(vec![
        Object::Int(1),
        Object::Float(2.5),
        Object::Bool(true),
        Object::String("text".to_string()),
        Object::ValueArray(vec![Object::Int(3)]),
    ]);

    let value = object.to_json_value();

    assert_eq!(value, json!([1, 2.5, true, "^text", [3]]));
    assert_eq!(Object::from_json_value(value).unwrap(), object);
}

#[test]
fn roundtrips_string_key_dict_values() {
    let mut entries = BTreeMap::new();
    entries.insert(DictKey::String("ada".to_string()), Object::Int(10));
    entries.insert(
        DictKey::String("grace".to_string()),
        Object::String("compiler".to_string()),
    );
    let object =
        Object::ValueDict(DictValue::new(DictKeyType::String, entries).expect("keys should match"));
    let value = object.to_json_value();

    assert_eq!(
        value,
        json!(["dict", "string", [["ada", 10], ["grace", "^compiler"]]])
    );
    assert_eq!(Object::from_json_value(value).unwrap(), object);
}

#[test]
fn roundtrips_int_key_dict_values() {
    let mut entries = BTreeMap::new();
    entries.insert(DictKey::Int(1), Object::String("one".to_string()));
    entries.insert(DictKey::Int(2), Object::Bool(true));
    let object =
        Object::ValueDict(DictValue::new(DictKeyType::Int, entries).expect("keys should match"));
    let value = object.to_json_value();

    assert_eq!(value, json!(["dict", "int", [[1, "^one"], [2, true]]]));
    assert_eq!(Object::from_json_value(value).unwrap(), object);
}

#[test]
fn roundtrips_empty_dict_values_with_key_type() {
    let string_dict = Object::ValueDict(DictValue::empty(DictKeyType::String));
    let string_value = string_dict.to_json_value();

    assert_eq!(string_value, json!(["dict", "string", []]));
    assert_eq!(Object::from_json_value(string_value).unwrap(), string_dict);

    let int_dict = Object::ValueDict(DictValue::empty(DictKeyType::Int));
    let int_value = int_dict.to_json_value();

    assert_eq!(int_value, json!(["dict", "int", []]));
    assert_eq!(Object::from_json_value(int_value).unwrap(), int_dict);
}

#[test]
fn roundtrips_nested_dict_values_with_arrays_and_objects() {
    let mut object_fields = BTreeMap::new();
    object_fields.insert("hp".to_string(), Object::Int(10));
    object_fields.insert(
        "tags".to_string(),
        Object::ValueArray(vec![
            Object::String("front".to_string()),
            Object::String("line".to_string()),
        ]),
    );

    let mut inner_entries = BTreeMap::new();
    inner_entries.insert(
        DictKey::String("player".to_string()),
        Object::ValueObject(object_fields),
    );
    let inner = Object::ValueDict(
        DictValue::new(DictKeyType::String, inner_entries).expect("keys should match"),
    );

    let mut outer_entries = BTreeMap::new();
    outer_entries.insert(
        DictKey::Int(7),
        Object::ValueArray(vec![Object::Int(1), inner]),
    );
    let object = Object::ValueDict(DictValue::new(DictKeyType::Int, outer_entries).unwrap());
    let value = object.to_json_value();

    assert_eq!(
        value,
        json!([
            "dict",
            "int",
            [[
                7,
                [
                    1,
                    [
                        "dict",
                        "string",
                        [[
                            "player",
                            {
                                "hp": 10,
                                "tags": ["^front", "^line"]
                            }
                        ]]
                    ]
                ]
            ]]
        ])
    );
    assert_eq!(Object::from_json_value(value).unwrap(), object);
}

#[test]
fn roundtrips_dynamic_object_values() {
    let mut fields = BTreeMap::new();
    fields.insert("hp".to_string(), Object::Int(10));
    fields.insert("name".to_string(), Object::String("Ada".to_string()));
    fields.insert(
        "flags".to_string(),
        Object::ValueArray(vec![Object::Bool(true), Object::Bool(false)]),
    );

    let object = Object::ValueObject(fields);
    let value = object.to_json_value();

    assert_eq!(
        value,
        json!({
            "flags": [true, false],
            "hp": 10,
            "name": "^Ada"
        })
    );
    assert_eq!(Object::from_json_value(value).unwrap(), object);
}

#[test]
fn dict_marker_does_not_change_object_values() {
    let parsed = Object::from_json_value(json!({
        "dict": ["^not", "^marker"],
        "^dict": "^field"
    }))
    .unwrap();

    let mut fields = BTreeMap::new();
    fields.insert(
        "dict".to_string(),
        Object::ValueArray(vec![
            Object::String("not".to_string()),
            Object::String("marker".to_string()),
        ]),
    );
    fields.insert("^dict".to_string(), Object::String("field".to_string()));

    assert_eq!(parsed, Object::ValueObject(fields));
}

#[test]
fn parses_json_arrays_without_container_terminators_as_values() {
    let parsed = Object::from_json_value(json!([1, 2, 3])).unwrap();

    assert_eq!(
        parsed,
        Object::ValueArray(vec![Object::Int(1), Object::Int(2), Object::Int(3)])
    );
}

#[test]
fn still_parses_json_arrays_with_container_terminators_as_containers() {
    let parsed = Object::from_json_value(json!(["nop", null])).unwrap();

    assert_eq!(
        parsed,
        Object::Container(Container::unnamed(vec![Object::ControlCommand(
            ControlCommand::NoOp
        )]))
    );
}

#[test]
fn parses_arrays_of_objects_as_dynamic_values() {
    let parsed = Object::from_json_value(json!([{ "hp": 10 }])).unwrap();

    let mut fields = BTreeMap::new();
    fields.insert("hp".to_string(), Object::Int(10));
    assert_eq!(
        parsed,
        Object::ValueArray(vec![Object::ValueObject(fields)])
    );
}

#[test]
fn parses_arrays_of_empty_objects_as_dynamic_values() {
    let parsed = Object::from_json_value(json!([{}, {}])).unwrap();

    assert_eq!(
        parsed,
        Object::ValueArray(vec![
            Object::ValueObject(BTreeMap::new()),
            Object::ValueObject(BTreeMap::new())
        ])
    );
}
