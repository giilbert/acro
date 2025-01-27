use std::cell::RefCell;

use chrono::{DateTime, TimeDelta, Utc};
use fnv::FnvHashMap;
use tracing::{error, info, warn};

use crate::{
    pointer::change_detection::Tick,
    systems::{SystemData, SystemId, SystemRunContext},
    world::World,
};

// TODO: Make a more sophisticated (multithreaded) scheduling system
#[derive(Debug)]
pub struct Schedule {
    pub current_tick: Tick,
    pub stages: FnvHashMap<Stage, Vec<SystemData>>,
    last_render_run: DateTime<Utc>,
    render_interval: TimeDelta,
    time_accumulator: TimeDelta,
    time_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    FixedUpdate,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
}

#[derive(Debug)]
pub enum SystemSchedulingRequirement {
    RunBefore(SystemId),
    RunAfter(SystemId),
}

#[derive(Debug, Clone)]
pub enum ScheduleError {
    ContradictorySchedulingRequirements,
}

impl Schedule {
    pub fn new() -> Self {
        // TODO: Make this configurable
        const FRAMES_PER_SECOND: f64 = 60.0;

        Self {
            current_tick: Tick::new(1),
            stages: Default::default(),
            last_render_run: Utc::now(),
            render_interval: TimeDelta::from_std(std::time::Duration::from_secs_f64(
                1.0 / FRAMES_PER_SECOND,
            ))
            .expect("invalid render interval"),
            time_accumulator: TimeDelta::zero(),
            time_count: 0,
        }
    }

    pub fn add_system(&mut self, stage: Stage, system: SystemData) -> Result<(), ScheduleError> {
        let stage_data = self.stages.entry(stage).or_default();

        // Find the last system that should run before this one
        let lower_index = system
            .scheduling_requirements
            .iter()
            .filter_map(|req| match req {
                SystemSchedulingRequirement::RunAfter(id) => Some(id),
                _ => None,
            })
            .map(|id| stage_data.iter().position(|s| s.id == *id))
            .max()
            .flatten();

        // Find the first system that should run after this one
        let upper_index = system
            .scheduling_requirements
            .iter()
            .filter_map(|req| match req {
                SystemSchedulingRequirement::RunBefore(id) => Some(id),
                _ => None,
            })
            .map(|id| stage_data.iter().position(|s| s.id == *id))
            .min()
            .flatten();

        let index = match (lower_index, upper_index) {
            (Some(lower), Some(upper)) => {
                if lower > upper {
                    return Err(ScheduleError::ContradictorySchedulingRequirements);
                }

                lower + 1
            }
            (Some(lower), None) => lower + 1,
            (None, Some(upper)) => upper,
            (None, None) => {
                stage_data.push(system);
                return Ok(());
            }
        };

        stage_data.insert(index, system);

        Ok(())
    }

    pub fn run_stage(&mut self, stage: Stage, world: &RefCell<World>) {
        let systems = match self.stages.get_mut(&stage) {
            Some(systems) => systems,
            None => return,
        };

        for system in systems.iter_mut() {
            self.current_tick = self.current_tick.next();

            let result = (system.run)(
                SystemRunContext {
                    world: &*world.borrow(),
                    tick: self.current_tick,
                    last_run_tick: system.last_run_tick,
                },
                system.parameters.as_mut(),
            );

            world.borrow_mut().check_swap();

            match result {
                Ok(()) => {}
                Err(err) => {
                    error!("system {} failed: {:?}", system.name, err);
                }
            }

            system.last_run_tick = self.current_tick;

            // world.borrow().archetypes.print_debug();
        }
    }

    pub fn run_once(&mut self, world: &RefCell<World>) {
        let start = chrono::Utc::now();
        let time_since_last_render = start.signed_duration_since(self.last_render_run);
        let should_render = time_since_last_render > self.render_interval;

        self.run_stage(Stage::PreUpdate, world);
        self.run_stage(Stage::Update, world);
        if should_render {
            self.run_stage(Stage::FixedUpdate, world);
        }
        self.run_stage(Stage::PostUpdate, world);

        if should_render {
            self.last_render_run = start;

            self.run_stage(Stage::PreRender, world);
            self.run_stage(Stage::Render, world);
            self.run_stage(Stage::PostRender, world);

            let elapsed = Utc::now().signed_duration_since(start);

            self.time_accumulator += elapsed;
            self.time_count += 1;

            const FRAME_TIME_INTERVAL: TimeDelta = TimeDelta::seconds(5);
            if self
                .render_interval
                .checked_mul(self.time_count)
                .expect("frame time accumulator overflow")
                > FRAME_TIME_INTERVAL
            {
                let average_frame_time = self.time_accumulator / self.time_count;
                info!(
                    "{FRAME_TIME_INTERVAL:?} average frame time: {:?} (theoretical max = {:.02}fps)",
                    average_frame_time,
                    1.0 / average_frame_time.num_seconds() as f64
                );
                self.time_accumulator = TimeDelta::zero();
                self.time_count = 0;
            }

            if elapsed > self.render_interval {
                warn!(
                    "frame took too long: {elapsed:?} (target = {:?})",
                    self.render_interval
                );
            }
        }
    }

    pub fn get_systems(&self, stage: Stage) -> &[SystemData] {
        self.stages
            .get(&stage)
            .map_or(&[], |systems| systems.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        pointer::change_detection::Tick,
        systems::{SystemData, SystemId},
    };

    use super::{Schedule, Stage, SystemSchedulingRequirement};

    fn create_faux_system(
        id: usize,
        name: impl ToString,
        scheduling_requirements: impl IntoIterator<Item = SystemSchedulingRequirement>,
    ) -> SystemData {
        SystemData {
            id: SystemId::Faux(id),
            name: name.to_string(),
            run: Box::new(move |_, _| Ok(())),
            last_run_tick: Tick::new(0),
            parameters: Box::new(()),
            scheduling_requirements: scheduling_requirements.into_iter().collect(),
        }
    }

    fn get_system_ids(systems: &[SystemData]) -> Vec<SystemId> {
        systems.iter().map(|s| s.id).collect()
    }

    #[test]
    fn schedule_systems() {
        use SystemId as S;
        use SystemSchedulingRequirement::*;

        let mut schedule = Schedule::new();
        schedule
            .add_system(Stage::Update, create_faux_system(1, "one", []))
            .expect("add system 1 failed");
        schedule
            .add_system(Stage::Update, create_faux_system(2, "two", []))
            .expect("add system 2 failed");
        schedule
            .add_system(Stage::Update, create_faux_system(3, "three", []))
            .expect("add system 3 failed");

        assert_eq!(
            get_system_ids(schedule.get_systems(Stage::Update)),
            &[S::Faux(1), S::Faux(2), S::Faux(3)]
        );

        schedule
            .add_system(
                Stage::Update,
                create_faux_system(4, "four", [RunBefore(S::Faux(2))]),
            )
            .expect("add system 4 failed");

        assert_eq!(
            get_system_ids(schedule.get_systems(Stage::Update)),
            &[S::Faux(1), S::Faux(4), S::Faux(2), S::Faux(3)]
        );

        schedule
            .add_system(
                Stage::Update,
                create_faux_system(5, "five", [RunAfter(S::Faux(3))]),
            )
            .expect("add system 5 failed");

        assert_eq!(
            get_system_ids(schedule.get_systems(Stage::Update)),
            &[S::Faux(1), S::Faux(4), S::Faux(2), S::Faux(3), S::Faux(5)]
        );

        schedule
            .add_system(
                Stage::Update,
                create_faux_system(6, "six", [RunAfter(S::Faux(1)), RunBefore(S::Faux(3))]),
            )
            .expect("add system 6 failed");

        assert_eq!(
            get_system_ids(schedule.get_systems(Stage::Update)),
            &[
                S::Faux(1),
                S::Faux(6),
                S::Faux(4),
                S::Faux(2),
                S::Faux(3),
                S::Faux(5)
            ]
        );

        schedule
            .add_system(
                Stage::Update,
                create_faux_system(7, "seven", [RunAfter(S::Faux(3)), RunBefore(S::Faux(1))]),
            )
            .expect_err("add system 7 should have failed because of contradictory requirements");
    }
}
