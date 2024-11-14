/// A command ID that can be associated with a shortcut.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CommandId(usize);

impl CommandId {
    pub fn new(str: &str) -> Self {
        CommandId(CommandId::id_from_str(str))
    }

    pub fn id(&self) -> usize {
        self.0
    }

    pub fn from_id(id: usize) -> Self {
        CommandId(id)
    }

    /// Passing strings around is hard across all languages and subsystems.
    /// Commands should be [Copy] to make them easier to work with. Since
    /// [String] is not [Copy], we use a [u32] hash of the command label.
    ///
    /// This hash is fast, deterministic, and has a low risk of collision.
    pub fn id_from_str(str: &str) -> usize {
        let mut s: usize = 0;
        for c in str.chars() {
            s = s.wrapping_mul(223).wrapping_add(c as usize);
        }
        s
    }
}
