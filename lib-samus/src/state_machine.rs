use pose::{Pose, Frame, Next};
use controller_input::ControllerInput;

pub struct StateMachine<'a> {
    current: Pose<'a>,
    input: ControllerInput,
    lookup: fn(usize) -> &'a Pose<'a>,
}

impl<'a> StateMachine<'a> {
    pub fn new(initial: usize, lookup: fn(usize) -> &'a Pose<'a>) -> Self {
        StateMachine {
            current: (lookup)(initial).clone(),
            input: ControllerInput::empty(),
            lookup,
        }
    }

    pub fn next(&mut self) -> (&'a Frame<'a>, u8) {
        match self.current.next() {
            Next::Frame(frame, duration) => (frame, duration),
            Next::NewPose(n) => {
                self.goto(n as usize);
                self.next()
            },
        }
    }

    pub fn input(&mut self, pressed: ControllerInput) {
        if pressed != self.input {
            if let Some(transition) = self.current.transitions.iter().find(|t| t.input == pressed) {
                self.input = pressed;
                self.goto(transition.to_pose as usize);
            }
        }
    }

    pub fn fall(&mut self) {
        // start falling in same direction
    }

    pub fn land(&mut self) {
        // land from jump in same direction
        // if not jumping or falling, no op
    }

    #[inline]
    pub fn goto(&mut self, state: usize) {
        self.current = (self.lookup)(state).clone();
    }

    #[inline]
    pub fn pose_name(&self) -> &'a str {
        self.current.name
    }

    #[inline]
    pub fn pose_state(&self) -> usize {
        self.current.id
    }

    #[inline]
    pub fn current_input(&self) -> ControllerInput {
        self.input
    }
}
