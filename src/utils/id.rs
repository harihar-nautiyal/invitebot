use crate::utils::ALPHA;
use nanoid::nanoid;

pub fn generate() -> String {
    nanoid!(32, &ALPHA)
}
