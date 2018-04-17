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
                let updated = self.goto(n as usize);
                if updated {
                    // Retrigger the input to account for holding an input button down
                    let input = self.input;
                    self.input(input);
                }
                self.next()
            },
        }
    }

    pub fn input(&mut self, pressed: ControllerInput) -> bool {
        self.input = pressed;
        if let Some(transition) = self.current.transitions.iter().find(|t| t.input == pressed) {
            if transition.to_pose != self.current.id as u8 {
                // FIXME: may want to extend an exception to
                // space-jumping into the same animation
                return self.goto(transition.to_pose as usize);
            }
        }
        false
    }

    pub fn bonk(&mut self) -> bool {
        // let next_pose = match self.current.id {
        //     0x09 | 0x0B => 0x89, // running right
        //     0x0A | 0x0C => 0x8A, // running left
        //     0x11 => 0xCF, // running right aiming down
        //     0x10 => 0xD0, // running left aiming up
        //     0x0F => 0xD1, // running right aiming up
        //     0x12 => 0xD2, // running left aiming down
        //     _ => return self.input(ControllerInput::empty()),
        // };
        // self.goto(next_pose)
        self.input(ControllerInput::empty())
    }

    pub fn fall(&mut self) -> bool {
        let next_pose = match self.current.id {
            0x13 => 0x67, // right jump gun extended
            0x15 => 0x2B, // right jump aiming up
            0x17 => 0x2D, // right jump aiming down
            0x30 => 0x29, // turning right
            0x4D => 0x29, // right jump gun not extended
            0x51 => 0x67, // right jump gun extended
            0x14 => 0x68, // left jump gun extended
            0x16 => 0x2C, // left jump aiming up
            0x18 => 0x2E, // left jump aiming down
            0x2F => 0x2A, // turning left
            0x4E => 0x2A, // left jump gun not extended
            0x52 => 0x68, // left jump gun extended
            _ => return false,
        };
        self.goto(next_pose)
    }

    pub fn land(&mut self) -> bool {
        let next_pose = match self.current.id {
            0x19 | 0x1B | 0x81 => 0xA6, // spin/space/screw jump right
            0x1A | 0x1C | 0x82 => 0xA7, // spin/space/screw jump left

            0x29 | 0x30 => 0xA4, // falling right, falling turning right
            0x2A | 0x2F => 0xA5, // falling left, falling turning left
            0x2B => 0xE0, // falling right aiming up
            0x2C => 0xE1, // falling left aming up
            0x2D => 0xA4, // falling right aiming down
            0x2E => 0xA5, // falling left aiming down

            0x6D | 0x94 => 0xE2, // falling aiming upright
            0x6E | 0x93 => 0xE3, // falling aiming upleft
            0x6F | 0x96 => 0xE4, // falling aiming downright
            0x70 | 0x95 => 0xE5, // falling aiming downleft

            0x67 => 0xE6, // falling right firing
            0x68 => 0xE7, // falling left firing
            _ => {
                return false;
            },
        };
        self.goto(next_pose)
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
