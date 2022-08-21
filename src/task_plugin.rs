use crate::{command_runner::CommandRunner, config_file::ConfigFile, tasks};
use bevy_app::{App, AppExit, Plugin};
use bevy_ecs::prelude::*;
use bevy_hierarchy::{BuildChildren, Children, Parent};
use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_time::FixedTimestep;
use bevy_utils::HashMap;
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

pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        CommandRunner::init();
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

    // add all entities
    for config_file in &*config {
        let parent = config_file.parent.as_deref().unwrap_or("root");
        for name in [parent, &config_file.id] {
            entities
                .entry(name)
                .or_insert_with(|| commands.spawn().insert(Name(name.to_string())).id());
        }
    }

    // create hierarchy
    for config_file in &*config {
        let parent = config_file.parent.as_deref().unwrap_or("root");
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

type ReadyChildren = (With<Ready>, With<Parent>);

fn start_tasks(
    mut commands: Commands,
    mut q_child: Query<(Entity, &Name), ReadyChildren>,
    config: Res<Vec<ConfigFile>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (child, child_name) in &mut q_child {
        let child_name = child_name.0.clone();
        if let Some(config_file) = config.iter().cloned().find(|c| c.id == child_name) {
            let task = thread_pool.spawn(async move { tasks::run_task(&config_file).await });
            commands.entity(child).remove::<Ready>();
            commands.entity(child).insert(Run(task));
        }
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
