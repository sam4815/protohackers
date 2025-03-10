use std::collections::HashMap;

use crate::models::SiteVisit;

pub fn calculate_checksum(bytes: &[u8]) -> u8 {
    let byte_sum = bytes.iter().fold(0x00, |a: u8, b: &u8| a.wrapping_add(*b));

    0x00_u8.wrapping_sub(byte_sum)
}

pub fn is_valid_visit(visit: SiteVisit) -> bool {
    let mut populations = HashMap::new();

    for population in visit.populations.iter() {
        if let Some(count) = populations.get(&population.species) {
            if *count != population.count {
                return false;
            }
        }

        populations.insert(&population.species, population.count);
    }

    true
}
