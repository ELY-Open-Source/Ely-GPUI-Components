mod de;
mod en;
mod fr;
mod he;
mod ja;
mod zh_cn;

/// Ely's own table for each locale; en-GB reads the English one.
const TABLES: [(&str, &[(&str, &str)]); 7] = [
    ("en-US", en::TABLE),
    ("en-GB", en::TABLE),
    ("de-DE", de::TABLE),
    ("fr-FR", fr::TABLE),
    ("ja-JP", ja::TABLE),
    ("zh-CN", zh_cn::TABLE),
    ("he-IL", he::TABLE),
];

/// Ely's message for `key` in `locale`; fails on a key or a locale it lacks, with no fallback.
pub(crate) fn builtin(locale: &str, key: &str) -> &'static str {
    (TABLES.iter())
        .filter(|(tag, _)| *tag == locale)
        .flat_map(|(_, table)| table.iter())
        .find_map(|(k, v)| (*k == key).then_some(*v))
        .unwrap_or_else(|| panic!("no builtin message {key:?} in {locale}"))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::{TABLES, en};
    use crate::i18n::LOCALES;

    fn arguments(message: &str) -> BTreeSet<&str> {
        message
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn every_declared_locale_has_a_table() {
        for locale in LOCALES {
            assert!(
                TABLES.iter().any(|(tag, _)| *tag == locale.tag),
                "no table for {}",
                locale.tag
            );
        }
        assert_eq!(TABLES.len(), LOCALES.len());
    }

    #[test]
    fn every_table_holds_every_key_and_asks_only_given_arguments() {
        let english: HashSet<&str> = en::TABLE.iter().map(|(key, _)| *key).collect();
        assert_eq!(english.len(), en::TABLE.len(), "en repeats a key");
        for (tag, table) in TABLES {
            let keys: HashSet<&str> = table.iter().map(|(key, _)| *key).collect();
            assert_eq!(keys.len(), table.len(), "{tag} repeats a key");
            let mut missing: Vec<_> = english.difference(&keys).collect();
            let mut extra: Vec<_> = keys.difference(&english).collect();
            missing.sort();
            extra.sort();
            assert!(missing.is_empty(), "{tag} misses {missing:?}");
            assert!(extra.is_empty(), "{tag} has {extra:?}, which en lacks");
            for (key, message) in table.iter() {
                let given: BTreeSet<&str> = arguments(super::builtin("en-US", key))
                    .union(&arguments(super::builtin("zh-CN", key)))
                    .copied()
                    .collect();
                let asked = arguments(message);
                assert!(
                    asked.is_subset(&given),
                    "{tag} {key} asks for {asked:?}; callers give {given:?}"
                );
            }
        }
    }
}
