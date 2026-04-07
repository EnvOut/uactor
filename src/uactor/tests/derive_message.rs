#![allow(dead_code)]

use uactor::actor::message::Message;

#[derive(uactor::Message)]
struct SimpleMessage;

#[derive(uactor::Message)]
struct TupleMessage(String, i32);

#[derive(uactor::Message)]
struct StructMessage {
    field: String,
}

#[derive(uactor::Message)]
struct GenericMessage<T> {
    data: T,
}

#[test]
fn derive_message_returns_struct_name() {
    assert_eq!(SimpleMessage::static_name(), "SimpleMessage");
    assert_eq!(TupleMessage::static_name(), "TupleMessage");
    assert_eq!(StructMessage::static_name(), "StructMessage");
    assert_eq!(GenericMessage::<i32>::static_name(), "GenericMessage");
}

#[test]
fn derive_message_name_instance_method() {
    let msg = StructMessage {
        field: "hello".into(),
    };
    assert_eq!(msg.name(), "StructMessage");
}

#[derive(uactor::Message)]
enum EventMessage {
    Created(u64),
    Updated { id: u64, value: String },
    Shutdown,
}

#[test]
fn derive_message_enum_static_name() {
    assert_eq!(EventMessage::static_name(), "EventMessage");
}

#[test]
fn derive_message_enum_variant_name() {
    assert_eq!(EventMessage::Created(1).name(), "EventMessage::Created");
    assert_eq!(
        EventMessage::Updated {
            id: 1,
            value: "x".into()
        }
        .name(),
        "EventMessage::Updated"
    );
    assert_eq!(EventMessage::Shutdown.name(), "EventMessage::Shutdown");
}

#[test]
fn message_impl_macro_works_without_trait_import() {
    struct LocalMsg;
    uactor::message_impl!(LocalMsg);
    assert_eq!(LocalMsg::static_name(), "LocalMsg");
}
