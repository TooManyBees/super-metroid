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

mod palette {
    #[allow(dead_code, non_camel_case_types)]
    #[derive(SamusPalette)]
    #[Addr = "D9400"]
    struct whatever;
}

fn main() {
    let pose = standing::pose();
    let palette = &palette::PALETTE;
    println!("{:?}", pose.name);
    println!("{:?}", palette);
}
