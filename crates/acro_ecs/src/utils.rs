use chrono::TimeDelta;

pub trait TimeDeltaExt {
    fn get_frac_secs(&self) -> f32;
    fn subsec_millis(&self) -> u32;
    fn subsec_micros(&self) -> u32;
    fn subsec_nanos(&self) -> u32;

    fn pretty(&self) -> String {
        let secs = self.get_frac_secs();
        let millis = self.subsec_millis();
        let micros = self.subsec_micros();
        let nanos = self.subsec_nanos();

        if secs > 1.0 {
            format!("{:.2}s", secs)
        } else if millis > 0 {
            format!("{:.2}ms", millis)
        } else if micros > 0 {
            format!("{:.2}µs", micros)
        } else {
            format!("{:.2}ns", nanos)
        }
    }
}

impl TimeDeltaExt for std::time::Duration {
    fn get_frac_secs(&self) -> f32 {
        self.as_secs_f32()
    }

    fn subsec_micros(&self) -> u32 {
        self.subsec_micros()
    }

    fn subsec_millis(&self) -> u32 {
        self.subsec_millis()
    }

    fn subsec_nanos(&self) -> u32 {
        self.subsec_nanos()
    }
}

const NANOS_PER_SEC: f32 = 1_000_000_000.0;
const NANOS_PER_MICRO: u32 = 1_000;
const NANOS_PER_MILLI: u32 = 1_000_000;

impl TimeDeltaExt for TimeDelta {
    fn get_frac_secs(&self) -> f32 {
        self.num_seconds() as f32 + (self.subsec_nanos() as f32 / NANOS_PER_SEC)
    }

    fn subsec_micros(&self) -> u32 {
        self.subsec_nanos() as u32 / NANOS_PER_MICRO
    }

    fn subsec_millis(&self) -> u32 {
        self.subsec_nanos() as u32 / NANOS_PER_MILLI
    }

    fn subsec_nanos(&self) -> u32 {
        self.subsec_nanos() as u32
    }
}
