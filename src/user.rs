use nix::unistd::{Uid, User};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TargetUser {
    pub name: String,
    pub uid: u32,
}

impl TargetUser {
    pub fn label(&self) -> String {
        let uid_label = format!("uid={}", self.uid);
        if self.name == uid_label {
            uid_label
        } else {
            format!("{} {}", self.name, uid_label)
        }
    }
}

pub fn resolve_user_or_uid(input: &str) -> Result<TargetUser, String> {
    if input.is_empty() {
        return Err("empty user or UID".to_string());
    }

    if input.chars().all(|character| character.is_ascii_digit()) {
        let uid = input
            .parse::<u32>()
            .map_err(|_| format!("invalid UID: {input}"))?;
        let name = username_for_uid(uid).unwrap_or_else(|| format!("uid={uid}"));
        return Ok(TargetUser { name, uid });
    }

    let user = User::from_name(input)
        .map_err(|error| format!("failed to resolve user {input}: {error}"))?
        .ok_or_else(|| format!("unknown user: {input}"))?;

    Ok(TargetUser {
        name: user.name,
        uid: user.uid.as_raw(),
    })
}

pub fn current_user() -> Result<TargetUser, String> {
    let uid = Uid::current().as_raw();
    let name = username_for_uid(uid).unwrap_or_else(|| format!("uid={uid}"));

    Ok(TargetUser { name, uid })
}

fn username_for_uid(uid: u32) -> Option<String> {
    User::from_uid(Uid::from_raw(uid))
        .ok()
        .flatten()
        .map(|user| user.name)
}
