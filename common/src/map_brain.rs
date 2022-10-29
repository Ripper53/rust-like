use bevy::prelude::Component;
use crate::physics::Position;

#[derive(Component)]
pub struct BehaviorData<T> {
    pub behavior: T,
    pub conditions: Vec<fn() -> bool>,
}
impl<T> BehaviorData<T> {
    pub fn new(behavior: T) -> Self {
        BehaviorData { behavior, conditions: Vec::default() }
    }
    pub fn run_if(mut self, condition: fn() -> bool) -> Self {
        self.conditions.push(condition);
        self
    }
    pub fn check_conditions(&self) -> bool {
        if self.conditions.len() == 0 {
            true
        } else {
            for condition in &self.conditions {
                if condition() {
                    return true;
                }
            }
            false
        }
    }
}

#[derive(Component, Debug)]
pub enum CharacterBehaviorData {
    Human {
        state: HumanState,
    },
    Werewolf {
        state: WerewolfState,
    },
}
impl CharacterBehaviorData {
    pub const fn default_human() -> Self {
        CharacterBehaviorData::Human { state: HumanState::Idle(None) }
    }
    pub const fn default_werewolf() -> Self {
        CharacterBehaviorData::Werewolf { state: WerewolfState::Hunt(None) }
    }
}
#[derive(Debug)]
pub enum HumanState {
    Idle(Option<NewObjective>),
    /// Moving to objective from index.
    Moving(usize),
    /// Saw beast and is unarmed or outnumbered!
    Panic,
}
#[derive(Debug)]
pub enum NewObjective {
    /// Choose point to wander towards,
    /// but exclude the interest point index.
    WanderButExclude(usize),
}

#[derive(Debug)]
pub enum WerewolfState {
    Hunt(Option<Position>),
    Panic((Option<Position>, Option<usize>)),
}
