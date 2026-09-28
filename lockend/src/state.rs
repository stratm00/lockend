pub type UserID = u64;
pub type LockerID = usize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
pub struct StateTransitionError {
    attempt_user: UserID,
    attempt_locker: String,
}

impl StateTransitionError {
    pub fn format(&self) -> String {
        format!(
            "User {} failed to open L#{}",
            self.attempt_user, self.attempt_locker
        )
        .to_owned()
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LockerRoomCommand {
    Actuate(UserID, LockerID),
}
pub enum LockerCommand {
    Actuate(UserID),
}
impl From<LockerRoomCommand> for LockerCommand {
    fn from(lrc: LockerRoomCommand) -> Self {
        match lrc {
            LockerRoomCommand::Actuate(uid, _) => Self::Actuate(uid),
        }
    }
}

#[derive(Clone)]
pub struct LockerRoomState<const N: usize> {
    pub lockers: Arc<Mutex<[LockerState; N]>>,
}

impl<const N: usize> LockerRoomState<N> {
    pub fn new() -> Self {
        const STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
        Self {
            lockers: Arc::new(Mutex::new(core::array::from_fn(|i| {
                LockerState::new_named(&STR[i..i + 1])
            }))),
        }
    }

    pub async fn apply(&mut self, cmd: LockerRoomCommand) -> Result<(), StateTransitionError> {
        let mut lockers_crit = self.lockers.lock().await;
        match cmd {
            LockerRoomCommand::Actuate(_, locker) => {
                if locker < N {
                    return lockers_crit[locker].apply(cmd.into());
                }
            }
        }
        Ok(())
    }
    pub async fn apply_in_series(
        &mut self,
        cmds: &[LockerRoomCommand],
    ) -> Vec<StateTransitionError> {
        let mut vec = Vec::new();
        for cmd in cmds {
            if let Err(ste) = self.apply(*cmd).await {
                vec.push(ste);
            }
        }
        vec
    }
    pub async fn grab_copy(&mut self) -> [LockerState; N] {
        let lockers_crit = self.lockers.lock().await;
        lockers_crit.clone()
    }
}

#[derive(Serialize)]
pub struct LockerState {
    name: String,
    state: LockerStateInternal,
}

#[derive(Copy, Clone, Serialize)]
pub enum LockerStateInternal {
    Locked(UserID),
    Unlocked,
}

impl Clone for LockerState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            state: self.state,
        }
    }
}

impl LockerState {
    pub fn new_named(given_name: &str) -> Self {
        Self {
            name: given_name.to_string(),
            state: LockerStateInternal::Unlocked,
        }
    }
    pub fn apply(&mut self, cmd: LockerCommand) -> Result<(), StateTransitionError> {
        match cmd {
            LockerCommand::Actuate(user) => match self.state {
                LockerStateInternal::Locked(owner_uid) => {
                    if owner_uid == user {
                        self.state = LockerStateInternal::Unlocked;
                        Ok(())
                    } else {
                        Err(StateTransitionError {
                            attempt_user: user,
                            attempt_locker: self.name.clone(),
                        })
                    }
                }
                LockerStateInternal::Unlocked => {
                    self.state = LockerStateInternal::Locked(user);
                    Ok(())
                }
            },
        }
    }
}
