#[macro_use] extern crate proc_samus;
extern crate sm;

// macro_rules! pose {
//     ($name:ident, $state:expr) => {
//         mod $name {
//             #[allow(dead_code, non_camel_case_types)]
//             #[derive(SamusPose)]
//             #[Name = $name] #[State = $state]
//             struct $name;
//         }
//     }
// }

// pose!(standing, 0);

mod standing {
    #[allow(dead_code, non_camel_case_types)]
    #[derive(SamusPose)]
    #[Name = "standing"] #[State = "0"]
    struct standing;
}

fn main() {
    let pose = &standing::POSE;
    println!("{:?}", pose.name);
}
