use bevy::prelude::Component;

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
