// the descriptor the manager registers the mod from; it names the claim, the override keybind and the settings schema
pub struct PingDescriptor;

impl PingDescriptor {
    pub const TEXT: &'static str = include_str!("../../descriptor.hcl");
}
