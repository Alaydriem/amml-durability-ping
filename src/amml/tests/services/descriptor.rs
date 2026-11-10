use alaydriem_durability_ping::services::{PingDescriptor, SettingsReader};
use amm_core::services::DescriptorChecker;
use amm_core::services::settings::{ChordText, SchemaChecker};
use amm_core::structs::descriptor::{Capability, ClaimMode, Descriptor, FieldKind, KeyScope};
use amm_core::structs::settings::HclDocument;
use durability_ping_core::config::PingConfig;

fn descriptor() -> Descriptor {
    DescriptorChecker::new(128, 256).parse(PingDescriptor::TEXT).unwrap()
}

#[test]
fn descriptor_passes_manager_checker() {
    let descriptor = descriptor();

    assert_eq!(descriptor.namespace, "alaydriem");
    assert_eq!(descriptor.name, "durability-ping");
    assert_eq!(descriptor.id(), "@alaydriem/durability-ping");
    let claims: Vec<_> = descriptor.claims.iter().map(|claim| (claim.capability, claim.mode)).collect();
    assert_eq!(claims, [(Capability::Input, ClaimMode::Shared)]);
    let keybinds: Vec<_> = descriptor
        .keybinds
        .iter()
        .map(|keybind| (keybind.action.as_str(), keybind.capability, keybind.scope, keybind.default_chord))
        .collect();
    let alt = ChordText::parse("Alt").unwrap();
    assert_eq!(keybinds, [("override", Capability::Input, KeyScope::Global, alt)]);
}

#[test]
fn every_int_field_has_the_spec_range() {
    let expected = [
        ("alert.percent", 0, 100, 1),
        ("alert.uses", 0, 2000, 1),
        ("alert.volume", 0, 100, 5),
        ("block.percent", 0, 100, 1),
        ("block.uses", 0, 2000, 1),
    ];
    let descriptor = descriptor();
    let ints: Vec<_> = descriptor
        .settings
        .iter()
        .flat_map(|section| section.fields.iter())
        .filter_map(|field| match field.kind {
            FieldKind::Int { min, max, step } => Some((field.path.as_str(), min, max, step)),
            _ => None,
        })
        .collect();

    assert_eq!(ints, expected);
}

#[test]
fn chord_alt_parses() {
    assert!(ChordText::parse("Alt").is_some());
}

#[test]
fn defaults_match_spec() {
    let checked = SchemaChecker::check(&descriptor().settings, HclDocument::empty());

    let config = SettingsReader::read(&checked.document.text());

    assert_eq!(config, PingConfig::default());
}
