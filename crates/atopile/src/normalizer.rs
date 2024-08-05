use schematics::{NormalizationError, Normalizer};

#[derive(Default)]
pub struct AtopileNormalizer {}

impl Normalizer for AtopileNormalizer {
    fn normalize_component_name(
        &self,
        name: &str,
    ) -> Result<String, schematics::NormalizationError> {
        // Parts and components have the same name normalization rules.
        self.normalize_part_name(name)
    }

    fn normalize_net_name(&self, name: &str) -> Result<String, schematics::NormalizationError> {
        let mut normalized = name.to_string();

        if normalized.starts_with('~') {
            normalized.replace_range(0..1, "n");
        }

        normalized = normalized.replace("+", "P");
        normalized = normalized.replace("-", "_");

        normalized = normalized
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();

        if normalized.is_empty() {
            return Err(NormalizationError::InvalidName(name.to_string()));
        }

        if !normalized.chars().next().unwrap().is_ascii_alphabetic() {
            normalized.insert(0, 'S');
        }

        Ok(normalized)
    }

    fn normalize_part_name(&self, name: &str) -> Result<String, schematics::NormalizationError> {
        let mut normalized: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

        if !normalized.chars().next().unwrap().is_ascii_alphabetic() {
            normalized.insert(0, 'S');
        }

        if normalized.is_empty() {
            return Err(NormalizationError::InvalidName(name.to_string()));
        }

        Ok(normalized)
    }

    fn normalize_port_name(
        &self,
        pin_name: &str,
        signal_name: &str,
    ) -> Result<String, schematics::NormalizationError> {
        let mut normalized = signal_name.to_string();
        if normalized.is_empty() {
            normalized = pin_name.to_string();
        }

        if normalized.starts_with('~') {
            normalized.replace_range(0..1, "n");
        }

        normalized = normalized.replace("+", "P");
        normalized = normalized.replace("-", "_");

        normalized = normalized
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();

        if normalized.is_empty() {
            return Err(NormalizationError::InvalidName(signal_name.to_string()));
        }

        if !normalized.chars().next().unwrap().is_ascii_alphabetic() {
            normalized.insert(0, 'S');
        }

        Ok(normalized)
    }
}
