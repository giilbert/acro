use std::time::Duration;

use fnv::FnvHashMap;
use tracing::info;

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
    time_accumulator: Duration,
    time_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
}

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
        Self {
            current_tick: Tick::new(1),
            stages: Default::default(),
            time_accumulator: Duration::new(0, 0),
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

    pub fn run_stage(&mut self, stage: Stage, world: &mut World) {
        let systems = match self.stages.get_mut(&stage) {
            Some(systems) => systems,
            None => return,
        };

        for system in systems.iter_mut() {
            self.current_tick = self.current_tick.next();
            (system.run)(
                SystemRunContext {
                    world,
                    tick: self.current_tick,
                    last_run_tick: system.last_run_tick,
                },
                system.parameters.as_mut(),
            );
            system.last_run_tick = self.current_tick;
        }
    }

    pub fn run_once(&mut self, world: &mut World) {
        let now = std::time::Instant::now();
        self.run_stage(Stage::PreUpdate, world);
        self.run_stage(Stage::Update, world);
        self.run_stage(Stage::PostUpdate, world);

        self.run_stage(Stage::PreRender, world);
        self.run_stage(Stage::Render, world);
        self.run_stage(Stage::PostRender, world);
        let elapsed = now.elapsed();

        self.time_accumulator += elapsed;
        self.time_count += 1;

        const FRAME_TIME_INTERVAL: Duration = Duration::from_secs(5);
        if self.time_accumulator > FRAME_TIME_INTERVAL {
            let average_frame_time = self.time_accumulator / self.time_count;
            info!(
                "average frame time over {FRAME_TIME_INTERVAL:?}: {:?} = {:.02}fps",
                average_frame_time,
                1.0 / average_frame_time.as_secs_f32()
            );
            self.time_accumulator = Duration::new(0, 0);
            self.time_count = 0;
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
            run: Box::new(move |_, _| {}),
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
