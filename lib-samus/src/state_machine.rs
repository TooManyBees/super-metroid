use pose::{Pose, Frame, Next};
use controller_input::ControllerInput;

pub struct StateMachine<'a> {
    current: Pose<'a>,
    input: ControllerInput,
    lookup: fn(usize) -> Option<&'a Pose<'a>>,
}

impl<'a> StateMachine<'a> {
    pub fn new(initial: usize, lookup: fn(usize) -> Option<&'a Pose<'a>>) -> Self {
        StateMachine {
            current: (lookup)(initial).expect("Passed a nonexistant initial pose state to StateMachine::new").clone(),
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

    pub fn input(&mut self, pressed: ControllerInput) -> bool {
        if let Some(transition) = self.current.transitions.iter().find(|t| t.input == pressed) {
            if transition.to_pose != self.current.id as u8 {
                // FIXME: may want to extend an exception to
                // space-jumping into the same animation
                if self.goto(transition.to_pose as usize) {
                    self.input = pressed;
                    return true
                }
            }
        }
        false
    }

    pub fn fall(&mut self) {
        // start falling in same direction
    }

    pub fn land(&mut self) {
        // land from jump in same direction
        // if not jumping or falling, no op
    }

    #[inline]
    pub fn goto(&mut self, state: usize) -> bool {
        match (self.lookup)(state) {
            Some(pose) => {
                self.current = pose.clone();
                true
            },
            None => false,
        }
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
