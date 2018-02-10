use sm::pose::*;

pub struct StateMachine<'a> {
    current: Pose<'a>,
    lookup: fn(usize) -> &'a Pose<'a>,
}

impl<'a> StateMachine<'a> {
    pub fn new(initial: &'a Pose<'a>, lookup: fn(usize) -> &'a Pose<'a>) -> Self {
        StateMachine {
            current: initial.clone(),
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
}
