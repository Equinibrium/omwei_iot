use crate::{Descriptor, AMBIENT_TEMPERATURE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registry {
    descriptors: &'static [Descriptor],
}

pub const CORPUS_V0_1: Registry = Registry {
    descriptors: &[AMBIENT_TEMPERATURE],
};

impl Registry {
    pub const fn new(descriptors: &'static [Descriptor]) -> Self {
        Self { descriptors }
    }

    pub fn resolve(&self, id: u16) -> Option<&'static Descriptor> {
        self.descriptors.iter().find(|d| d.id == id)
    }

    pub fn contains(&self, id: u16) -> bool {
        self.resolve(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_resolves_temperature() {
        let d = CORPUS_V0_1.resolve(0x0042).unwrap();
        assert_eq!(d.label, "ambient_temperature");
        assert_eq!(d.scale, 1000);
    }

    #[test]
    fn unknown_id_is_not_interoperable() {
        assert!(!CORPUS_V0_1.contains(0xFFFF));
    }
}
