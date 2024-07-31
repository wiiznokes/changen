use anyhow::Result;




enum CommitKind {
    Fix,
    Improve,
    Feat,
    HotFix,
    Ignore,
}

fn parse(message: &str) -> Result<CommitKind> {

    enum State {
        Init,
        /// (
        ParO,
        /// )
        ParC,
        /// :
        Colon,
        F,
        Fi,
        Fix,
        I,
        Im,
        Imp,
        Impr,
        Impro,
        Improv,
        Improve,
        Fe,
        Fea,
        Feat,
        H,
        Ho,
        Hot,
        Hotf,
        Hotfi,
        Hotfix,
        In,
        Ing,
        Ingr,
        Ingro,
        I
    }


    let mut iter = message.chars();

    while let Some(c) = iter.next() {
        
        match c {
            'c' => {

            }
            _ => {

            }
        }
    }

    todo!()
}



#[cfg(test)]
mod test {

    #[test]
    fn fix() {

        let m = "";



    }
}
