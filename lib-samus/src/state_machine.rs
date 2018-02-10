use pose::{Pose, Frame, Next};

pub struct StateMachine<'a> {
    current: Pose<'a>,
    lookup: fn(usize) -> &'a Pose<'a>,
}

impl<'a> StateMachine<'a> {
    pub fn new(initial: usize, lookup: fn(usize) -> &'a Pose<'a>) -> Self {
        StateMachine {
            current: (lookup)(initial).clone(),
            lookup,
        }
    }

    pub fn next(&mut self) -> (&'a Frame<'a>, u8) {
        match self.current.next() {
            Next::Frame(frame, duration) => (frame, duration),
            Next::NewPose(n) => {
                self.current = (self.lookup)(n as usize).clone();
                self.next()
            },
        }
    }

    pub fn goto(&mut self, state: usize) {
        self.current = (self.lookup)(state).clone();
    }

    pub fn pose_name(&self) -> &'a str {
        self.current.name
    }

    pub fn pose_state(&self) -> usize {
        self.current.id
    }
}
