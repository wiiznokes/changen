use anyhow::bail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl TryFrom<&str> for Repo {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let slip = value.split('/').collect::<Vec<_>>();

        if slip.len() != 2 {
            bail!("wrong repo format");
        }

        Ok(Repo {
            owner: slip[0].to_owned(),
            name: slip[1].to_owned(),
        })
    }
}

// impl From<&str> for Repo {
//     fn from(value: &str) -> Self {
//         let slip = value.split('/').collect::<Vec<_>>();

//         if slip.len() != 2 {
//             panic!("wrong repo format")
//         }

//         Repo {
//             owner: slip[0].to_owned(),
//             name: slip[1].to_owned(),
//         }
//     }
// }

pub struct TextInterpolate {
    buffer: String,
    start_pattern: String,
    end_pattern: String,
}

impl TextInterpolate {
    pub fn new(buffer: String, start_pattern: &str, end_pattern: &str) -> Self {
        TextInterpolate {
            buffer,
            start_pattern: start_pattern.to_string(),
            end_pattern: end_pattern.to_string(),
        }
    }

    pub fn interpolate(&mut self, name: &str, content: &str) {
        let replaced = self.buffer.replace(
            &format!("{}{}{}", self.start_pattern, name, self.end_pattern),
            content,
        );
        self.buffer = replaced;
    }

    pub fn text(self) -> String {
        self.buffer
    }
}

#[cfg(test)]
mod test {
    use super::Repo;

    #[test]
    fn test() {
        let repo = Repo::try_from("wiiznokes/fan-control").unwrap();

        assert_eq!(
            repo,
            Repo {
                owner: "wiiznokes".into(),
                name: "fan-control".into()
            }
        );
    }
}
