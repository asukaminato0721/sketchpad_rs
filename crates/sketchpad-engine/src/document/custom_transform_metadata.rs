use super::records::u32_at;

/// A custom-transform definition stored by Native at document level.
///
/// The construction itself is the ordinary object graph between `seed` and
/// `output`; record `0x2331` supplies the name and the two endpoint IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomTransformDefinition {
    name: String,
    seed: usize,
    output: usize,
}

impl CustomTransformDefinition {
    pub(super) fn from_remapped(name: &str, seed: usize, output: usize) -> Self {
        Self {
            name: name.to_owned(),
            seed,
            output,
        }
    }

    pub(super) fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 19 || u32_at(payload, 8) != 0 {
            return None;
        }
        let name_len = u32_at(payload, 12) as usize;
        if payload.len() != 19usize.checked_add(name_len)? {
            return None;
        }
        let name = std::str::from_utf8(&payload[16..16 + name_len])
            .ok()?
            .to_owned();
        let seed = u32_at(payload, 0) as usize;
        let output = u32_at(payload, 4) as usize;
        if name.is_empty() || seed == 0 || output == 0 {
            return None;
        }
        Some(Self { name, seed, output })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn seed(&self) -> usize {
        self.seed
    }

    pub fn output(&self) -> usize {
        self.output
    }
}
