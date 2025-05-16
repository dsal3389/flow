
use super::profile::Profile;

pub struct Config {
    profile: Profile
}

impl Config {
    pub fn new(profile: Profile) -> Config {
        Config { profile }
    }
}
