enum State {
    Init,
    OpenBracket,
    ClosedBracket,
    Version,
    FirstSpace,
    Tirret,
    SecondSpace,
    Title,
}

pub fn match_release_title(text: &str) -> Option<(&str, &str)> {
    let mut state = State::Init;

    let version = 

    for c in text.chars() {
        match state {
            State::Init => {
                if c != '[' {
                    return None;
                }
                state = State::OpenBracket;
            }
            State::OpenBracket => {
                if c == '[' || c == ']' {
                    return None;
                }
            }
            State::ClosedBracket => todo!(),
            State::Version => todo!(),
            State::FirstSpace => todo!(),
            State::Tirret => todo!(),
            State::SecondSpace => todo!(),
            State::Title => todo!(),
        };
    }

    todo!()
}
