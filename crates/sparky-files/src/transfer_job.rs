use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::domain::transfer::{Ending, Outcome, Progress, Step};
use crate::domain::undo::TrashEntry;
use crate::ports::Transfer;

const REPORT_EVERY: Duration = Duration::from_millis(100);

pub fn execute(
    steps: Vec<Step>,
    skipped: usize,
    transfer: &dyn Transfer,
    cancel: &AtomicBool,
    report: &mut dyn FnMut(Progress),
) -> Outcome {
    let mut progress = Progress {
        items_total: steps.len(),
        bytes_total: steps.iter().map(|step| transfer.size(&step.source)).sum(),
        ..Progress::default()
    };
    report(progress);
    let mut done = Done::default();
    let mut reported = Instant::now();
    for step in steps {
        if cancel.load(Ordering::SeqCst) {
            return done.outcome(skipped, Ending::Cancelled);
        }
        let result = transfer.run(&step, &mut |bytes| {
            progress.bytes_done += bytes;
            if reported.elapsed() >= REPORT_EVERY {
                report(progress);
                reported = Instant::now();
            }
            !cancel.load(Ordering::SeqCst)
        });
        let displaced = match result {
            Ok(displaced) => displaced,
            Err(error) => {
                let ending = match cancel.load(Ordering::SeqCst) {
                    true => Ending::Cancelled,
                    false => Ending::Failed {
                        name: name_of(&step.source),
                        error: error.to_string(),
                    },
                };
                return done.outcome(skipped, ending);
            }
        };
        progress.items_done += 1;
        report(progress);
        done.completed.push(step);
        done.displaced.extend(displaced);
    }
    done.outcome(skipped, Ending::Finished)
}

#[derive(Default)]
struct Done {
    completed: Vec<Step>,
    displaced: Vec<TrashEntry>,
}

impl Done {
    fn outcome(self, skipped: usize, ending: Ending) -> Outcome {
        Outcome {
            completed: self.completed,
            displaced: self.displaced,
            skipped,
            ending,
        }
    }
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
