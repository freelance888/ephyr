//! Pool of [`TeamspeakToFIFO`] processes performing redirection
//! of a audio traffic.
use crate::{
    audio_redirect::{
        teamspeak,
        teamspeak_to_fifo::{TeamspeakInput, TeamspeakToFIFO},
    },
    state,
    state::MixinId,
};
use std::collections::HashMap;

/// Pool of [`TeamspeakToFIFO`] processes performing redirection
/// of a audio traffic.
#[derive(Debug, Default)]
pub struct AudioProcessingPool {
    /// Pool of currently running [`TeamspeakToFIFO`] re-streaming
    /// processes identified by an ID of the correspondent element
    /// in a [`State`].
    ///
    /// [`State`]: crate::state::State
    pool: HashMap<MixinId, TeamspeakToFIFO>,
}

impl AudioProcessingPool {
    /// Adjusts this [`AudioProcessingPool`] to run audio processing
    /// according to the given renewed [`state::Restream`]s.
    pub fn apply(&mut self, restreams: &[state::Restream]) {
        // The most often case is when one new TeamspeakToFIFO process is added.
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for r in restreams {
            if !r.input.enabled || !r.input.is_ready_to_serve() {
                continue;
            }

            for o in &r.outputs {
                let _ = self.apply_output(o, &mut new_pool);
            }
        }

        self.pool = new_pool;
    }

    /// Inspects the given [`state::Output`] filling the `new_pool` with a
    /// required [`TeamspeakToFIFO`] process. Tries to preserve already
    /// running [`TeamspeakToFIFO`] processes in its `pool` as much as possible.
    fn apply_output(
        &mut self,
        output: &state::Output,
        new_pool: &mut HashMap<MixinId, TeamspeakToFIFO>,
    ) -> Option<()> {
        if !output.enabled {
            return None;
        }

        let inputs = output
            .mixins
            .iter()
            .filter_map(|m| (m.src.scheme() == "ts").then(|| m))
            .map(|m| {
                TeamspeakInput::new(
                    m,
                    output.label.as_ref(),
                    self.pool.get(&m.id).map(|m| &m.input),
                )
            })
            .filter_map(|ts| ts.is_some().then_some(ts.unwrap()));

        for input in inputs {
            let process = TeamspeakToFIFO::run(input);
            let old_process = new_pool.insert(process.mixin_id, process);
            drop(old_process);
        }
        Some(())
    }
}

impl Drop for AudioProcessingPool {
    fn drop(&mut self) {
        // Wait for all the async `Drop`s to proceed well.
        drop(tokio::spawn(teamspeak::finish_all_disconnects()));
    }
}
