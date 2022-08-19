use crate::{command_runner_plugin::CommandRunner, config_file::ConfigFile, tasks};
use bevy_app::{App, AppExit, Plugin};
use bevy_ecs::prelude::*;
use bevy_hierarchy::{BuildChildren, Children, Parent};
use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_time::FixedTimestep;
use bevy_utils::{prelude::*, HashMap};
use color_eyre::eyre::Result;
use futures_lite::future;

#[derive(Component, Default)]
struct Instance;

#[derive(Component, Default)]
struct Name(String);

/// ready to run
#[derive(Component, Default)]
struct Ready;

/// running
#[derive(Component)]
struct Run(Task<Result<String>>);

/// finished
#[derive(Component)]
struct Done;

#[derive(Bundle, Default)]
struct TaskBundle {
    name: Name,
    _p: Instance,
}

pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_tasks).add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(make_ready)
                .with_system(start_tasks)
                .with_system(monitor_tasks)
                .with_system(terminate),
        );
    }
}

fn add_tasks(mut commands: Commands, config: Res<Vec<ConfigFile>>) {
    let mut entities = HashMap::new();

    for config_file in config.iter() {
        let parent = if let Some(parent) = &config_file.parent {
            parent
        } else {
            "root"
        };
        for k in [parent, &config_file.id] {
            entities
                .entry(k)
                .or_insert_with(|| commands.spawn_bundle(make_task(k)).id());
        }
    }

    for config_file in config.iter() {
        let parent = if let Some(parent) = &config_file.parent {
            parent
        } else {
            "root"
        };
        commands
            .entity(*entities.get(parent).unwrap())
            .push_children(&[*entities.get(&config_file.id.as_ref()).unwrap()]);
    }
}

type New = (Without<Ready>, Without<Run>, Without<Done>);

fn make_ready(
    mut commands: Commands,
    q_parent: Query<(Entity, Option<&Children>), New>,
    q_child: Query<(Entity, &Name), Without<Done>>,
) {
    for (parent, children) in &q_parent {
        let is_ready = if let Some(children) = children {
            let has_unfinished_child = children.iter().any(|child| q_child.get(*child).is_ok());
            !has_unfinished_child
        } else {
            true
        };
        if is_ready {
            commands.entity(parent).insert(Ready);
        }
    }
}

fn start_tasks(
    mut commands: Commands,
    mut q_child: Query<(Entity, &Parent, &Name), With<Ready>>,
    q_parent: Query<&Name>,
    config: Res<Vec<ConfigFile>>,
    runner: Res<&'static CommandRunner>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let runner = *runner;
    for (child, parent, child_name) in &mut q_child {
        if let Ok(parent_name) = q_parent.get(parent.get()) {
            let parent_name = parent_name.0.clone();
            let child_name = child_name.0.clone();
            println!("starting {child_name}");
            if let Some(config_file) = config
                .iter()
                .cloned()
                .find(|c| c.id == child_name && c.parent.as_ref() == Some(&parent_name))
            {
                let task =
                    thread_pool.spawn(async move { tasks::run_task(&config_file, runner).await });
                commands.entity(child).remove::<Ready>();
                commands.entity(child).insert(Run(task));
            }
        };
    }
}

fn monitor_tasks(mut commands: Commands, mut q_child: Query<(Entity, &mut Run)>) {
    for (entity, mut task) in &mut q_child {
        if let Some(Ok(child)) = future::block_on(future::poll_once(&mut task.0)) {
            println!("done: {child}");
            commands.entity(entity).remove::<Run>();
            commands.entity(entity).insert(Done);
        }
    }
}

fn terminate(
    q_running: Query<Or<(Without<Done>, Without<Parent>)>>,
    mut exit: EventWriter<AppExit>,
) {
    let all_done = q_running.iter().len() == 1;
    if all_done {
        exit.send(AppExit);
    }
}

fn make_task(name: &str) -> TaskBundle {
    TaskBundle {
        name: Name(name.to_string()),
        ..default()
    }
}
